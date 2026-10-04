//! Private preparation cancellation. Owned children remain outside cancellable
//! waits so cancellation cannot replace verified cleanup with future destruction.
use std::{
    ffi::OsString,
    path::Path,
    process::Stdio,
    sync::{Arc, Mutex, atomic::AtomicBool},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    sync::watch,
};

use super::protocol::failure;
use crate::adapter::{AdapterError, AdapterResult, ErrorKind, ProcessGroup, cleanup_group};

const OUTPUT_LIMIT: usize = 64 * 1024;

#[derive(Clone, Debug)]
enum Cause {
    #[cfg(test)]
    Cancelled,
    Failed(ErrorKind, String),
}
impl Cause {
    fn error(&self) -> AdapterError {
        match self {
            #[cfg(test)]
            Self::Cancelled => failure(
                ErrorKind::StateConflict,
                "native preparation cancelled before admission",
            ),
            Self::Failed(kind, message) => failure(*kind, message.clone()),
        }
    }
}

/// A first-cause latch, distinct from native process/turn completion. The owner
/// selecting cancellation has no authority to infer that cleanup succeeded.
pub(super) struct Preparation {
    cause: Mutex<Option<Cause>>,
    cancelled: watch::Sender<bool>,
}
impl Preparation {
    pub fn new() -> Self {
        let (cancelled, _) = watch::channel(false);
        Self {
            cause: Mutex::new(None),
            cancelled,
        }
    }
    #[cfg(test)]
    pub fn cancel(&self) {
        if let Ok(mut cause) = self.cause.lock()
            && cause.is_none()
        {
            *cause = Some(Cause::Cancelled);
            self.cancelled.send_replace(true);
        }
    }
    pub fn check(&self) -> AdapterResult<()> {
        match self.cause.lock() {
            Ok(cause) => match cause.as_ref() {
                Some(cause) => Err(cause.error()),
                None => Ok(()),
            },
            Err(_) => Err(failure(
                ErrorKind::StateFailure,
                "native preparation cause poisoned",
            )),
        }
    }
    pub fn failed(&self, error: AdapterError) -> AdapterError {
        match self.cause.lock() {
            Ok(mut cause) => cause
                .get_or_insert_with(|| Cause::Failed(error.kind, error.message))
                .error(),
            Err(_) => failure(ErrorKind::StateFailure, "native preparation cause poisoned"),
        }
    }
    pub async fn wait_cancelled(&self) {
        let mut receiver = self.cancelled.subscribe();
        loop {
            if *receiver.borrow_and_update() {
                return;
            }
            if receiver.changed().await.is_err() {
                return;
            }
        }
    }
    /// Only borrow child ownership through this wait. An exclusive child-owning
    /// future must perform its own cancellation-aware cleanup instead.
    pub async fn wait<T>(
        &self,
        future: impl std::future::Future<Output = AdapterResult<T>>,
    ) -> AdapterResult<T> {
        self.check()?;
        let result = tokio::select! {
            biased;
            _ = self.wait_cancelled() => self.check().and_then(|()| Err(failure(ErrorKind::StateFailure, "cancellation missing its cause"))),
            result = future => result,
        };
        result.map_err(|error| self.failed(error))
    }
}

/// Additive private helper: the shared adapter Git helper and its consumers stay
/// unchanged. The five-second caller deadline and 250ms reap budget are retained.
pub(super) async fn bounded_git(
    executable: &Path,
    cwd: &Path,
    args: &[String],
    environment: Vec<(OsString, OsString)>,
    deadline: tokio::time::Instant,
    uncertain: Arc<AtomicBool>,
    preparation: &Preparation,
) -> AdapterResult<String> {
    preparation.check()?;
    if tokio::time::Instant::now() >= deadline {
        return Err(preparation.failed(failure(
            ErrorKind::Timeout,
            "Git ownership preflight timed out",
        )));
    }
    let mut command = Command::new(executable);
    command
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .envs(environment)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .process_group(0);
    // No await occurs between the cancellation check and taking child ownership.
    preparation.check()?;
    let child = command.spawn().map_err(|error| {
        preparation.failed(failure(ErrorKind::ProcessFailure, error.to_string()))
    })?;
    let mut child =
        ProcessGroup::new(child, uncertain).map_err(|error| preparation.failed(error))?;
    let mut stdout = tokio::spawn(read_output(
        child.child.stdout.take().expect("piped Git stdout"),
    ));
    let mut stderr = tokio::spawn(read_output(
        child.child.stderr.take().expect("piped Git stderr"),
    ));
    let observed = preparation
        .wait(async {
            tokio::time::timeout_at(deadline, child.observe_exit())
                .await
                .map_err(|_| failure(ErrorKind::Timeout, "Git ownership preflight timed out"))?
                .map_err(|error| {
                    failure(
                        ErrorKind::SessionLost,
                        format!("Git child observation failed: {error}"),
                    )
                })
        })
        .await;
    // The selected cause is already latched BEFORE this cleanup await. Cleanup
    // uncertainty is retained separately and cannot authorize an exact restore.
    child = match cleanup_group(child).await {
        Ok(child) => child,
        Err(cleanup) => {
            stdout.abort();
            stderr.abort();
            return Err(super::transport::failure_after_cleanup(
                observed.err().unwrap_or_else(|| {
                    preparation.failed(failure(ErrorKind::SessionLost, "Git cleanup failed"))
                }),
                Err(cleanup),
            ));
        }
    };
    let exit = tokio::time::timeout(Duration::from_millis(250), child.reap())
        .await
        .map_err(|_| {
            failure(
                ErrorKind::SessionLost,
                "Git child death not confirmed after cleanup",
            )
        })
        .and_then(|result| {
            result.map_err(|error| {
                failure(
                    ErrorKind::SessionLost,
                    format!("Git child reap failed: {error}"),
                )
            })
        });
    let exit = match exit {
        Ok(exit) => exit,
        Err(cleanup) => {
            stdout.abort();
            stderr.abort();
            return Err(super::transport::failure_after_cleanup(
                observed.err().unwrap_or_else(|| {
                    preparation.failed(failure(ErrorKind::SessionLost, "Git reap failed"))
                }),
                Err(cleanup),
            ));
        }
    };
    if let Err(error) = observed {
        stdout.abort();
        stderr.abort();
        return Err(error);
    }
    let output = tokio::time::timeout(Duration::from_millis(250), async {
        let output = (&mut stdout)
            .await
            .map_err(|error| failure(ErrorKind::ProcessFailure, error.to_string()))??;
        (&mut stderr)
            .await
            .map_err(|error| failure(ErrorKind::ProcessFailure, error.to_string()))??;
        Ok::<_, AdapterError>(output)
    })
    .await;
    let output = match output {
        Ok(result) => result.map_err(|error| preparation.failed(error))?,
        Err(_) => {
            stdout.abort();
            stderr.abort();
            return Err(preparation.failed(failure(
                ErrorKind::ProcessFailure,
                "Git output remained open after cleanup",
            )));
        }
    };
    if !exit.success() {
        return Err(preparation.failed(failure(
            ErrorKind::OwnershipMismatch,
            "Git ownership preflight failed",
        )));
    }
    preparation.check()?;
    String::from_utf8(output)
        .map(|value| value.trim().to_owned())
        .map_err(|_| {
            preparation.failed(failure(
                ErrorKind::ParseFailure,
                "invalid Git metadata encoding",
            ))
        })
}
async fn read_output(reader: impl AsyncRead + Unpin) -> AdapterResult<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .take((OUTPUT_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(|error| failure(ErrorKind::ProcessFailure, error.to_string()))?;
    if bytes.len() > OUTPUT_LIMIT {
        return Err(failure(
            ErrorKind::InvalidInput,
            "Git metadata exceeds output budget",
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn cancelled_git_wait_reaps_owned_group_and_keeps_cancellation_cause() {
        let directory = tempfile::tempdir().unwrap();
        let ready = directory.path().join("ready");
        let preparation = Arc::new(Preparation::new());
        let uncertain = Arc::new(AtomicBool::new(false));
        let mut action = {
            let preparation = preparation.clone();
            let uncertain = uncertain.clone();
            let cwd = directory.path().to_path_buf();
            let ready = ready.clone();
            tokio::spawn(async move {
                bounded_git(
                    Path::new("/bin/sh"),
                    &cwd,
                    &[
                        "-c".into(),
                        "printf '%s' \"$$\" > \"$1\"; exec /bin/sleep 30".into(),
                        "rrx-owned-fixture".into(),
                        ready.to_str().unwrap().into(),
                    ],
                    Vec::new(),
                    tokio::time::Instant::now() + Duration::from_secs(5),
                    uncertain,
                    &preparation,
                )
                .await
            })
        };
        tokio::time::timeout(Duration::from_secs(3), async {
            while !ready.exists() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        assert!(uncertain.load(Ordering::SeqCst));
        let cancelled_at = tokio::time::Instant::now();
        preparation.cancel();
        // Retain the JoinHandle while awaiting it, including broken-control
        // verification; an assertion must not detach the owned cleanup task.
        let error = tokio::time::timeout(Duration::from_secs(7), &mut action)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::StateConflict);
        assert_eq!(
            error.message,
            "native preparation cancelled before admission"
        );
        assert!(
            !uncertain.load(Ordering::SeqCst),
            "owned cleanup must complete before cancellation returns"
        );
        let pid = std::fs::read_to_string(ready).unwrap();
        let observed = std::process::Command::new("/bin/ps")
            .args(["-p", &pid, "-o", "pid="])
            .output()
            .unwrap();
        assert!(
            observed.stdout.is_empty(),
            "the helper must reap its owned leader before returning"
        );
        assert!(
            cancelled_at.elapsed() < Duration::from_secs(2),
            "cancellation must wake the owned wait before its original Git deadline"
        );
    }

    #[tokio::test]
    async fn pre_cancelled_git_does_not_spawn_and_failed_cause_cannot_be_relabelled() {
        let directory = tempfile::tempdir().unwrap();
        let marker = directory.path().join("spawned");
        let preparation = Preparation::new();
        preparation.cancel();
        let uncertain = Arc::new(AtomicBool::new(false));
        let error = bounded_git(
            Path::new("/bin/sh"),
            directory.path(),
            &[
                "-c".into(),
                "touch \"$1\"".into(),
                "rrx-owned-fixture".into(),
                marker.to_str().unwrap().into(),
            ],
            Vec::new(),
            tokio::time::Instant::now() + Duration::from_secs(5),
            uncertain.clone(),
            &preparation,
        )
        .await
        .unwrap_err();
        assert_eq!(error.kind, ErrorKind::StateConflict);
        assert!(!marker.exists());
        assert!(!uncertain.load(Ordering::SeqCst));

        let preparation = Preparation::new();
        let error = preparation.failed(failure(
            ErrorKind::OwnershipMismatch,
            "detected actual inode replacement",
        ));
        preparation.cancel();
        assert_eq!(error.kind, ErrorKind::OwnershipMismatch);
        assert_eq!(
            preparation.check().unwrap_err().kind,
            ErrorKind::OwnershipMismatch
        );
        assert_eq!(
            preparation
                .failed(failure(ErrorKind::Timeout, "later observation"))
                .message,
            "detected actual inode replacement"
        );
    }
}
