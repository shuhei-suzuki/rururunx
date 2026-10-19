use super::{PipelineRole, ProviderOutcome, classify_claude, classify_codex};
use crate::{
    adapter::{ProcessGroup, cleanup_group},
    execution::{
        RuntimeOwner,
        verification::{StreamEvidence, inspect, retain},
    },
    state::pipeline::{PipelineCursor, PipelineDeathEvidence},
};
use anyhow::{Context, Result, ensure};
use rustix::process::{Pid, test_kill_process_group};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{Notify, watch},
    task::JoinHandle,
    time::{Instant, timeout},
};

const STDERR_BYTES: usize = 1024 * 1024;
const CODEX_BYTES: usize = 16 * 1024 * 1024;
const CLAUDE_BYTES: usize = 1024 * 1024;
const LAST_BYTES: usize = 256 * 1024;
const DEATH_BOUND: Duration = Duration::from_secs(2);
const DISCOVERY_RETRY_PAUSE: Duration = Duration::from_millis(5);

/// Observation metadata only; this carries no authority to signal a process.
struct DeathDiscovery {
    coverage: rrx_process_tracker::Coverage,
    matching_processes: usize,
}

impl From<rrx_process_tracker::Discovery> for DeathDiscovery {
    fn from(discovery: rrx_process_tracker::Discovery) -> Self {
        Self {
            coverage: discovery.coverage,
            matching_processes: discovery.processes.len(),
        }
    }
}

/// Only a complete empty scan before the shared deadline proves disappearance.
async fn discover_death<F, Fut>(deadline: Instant, mut scan: F) -> Result<u32>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<DeathDiscovery>>,
{
    let mut attempts = 0;
    loop {
        ensure!(
            Instant::now() < deadline,
            "cookie death unconfirmed: deadline"
        );
        attempts += 1;
        let discovery = tokio::time::timeout_at(deadline, scan()).await??;
        ensure!(
            discovery.matching_processes == 0,
            "cookie death unconfirmed: matching process"
        );
        ensure!(
            Instant::now() < deadline,
            "cookie death unconfirmed: deadline"
        );
        if !discovery.coverage.limited() {
            return Ok(attempts);
        }
        tokio::time::sleep_until(deadline.min(Instant::now() + DISCOVERY_RETRY_PAUSE)).await;
    }
}

/// A retained monitor: dropping this handle closes the stop channel, never custody.
pub struct StepCustody {
    stop: watch::Sender<bool>,
    task: JoinHandle<Result<()>>,
}
impl StepCustody {
    pub fn request_stop(&self) {
        self.stop.send_replace(true);
    }
    pub async fn wait(self) -> Result<()> {
        let Self { stop, task } = self;
        let result = task.await.context("pipeline supervisor join failed")?;
        drop(stop);
        result
    }
    pub(super) fn start(monitor: Monitor) -> Self {
        let (stop, receiver) = watch::channel(false);
        let owner = monitor.owner.clone();
        let cursor = monitor.cursor.clone();
        let wake = monitor.wake.clone();
        let task = tokio::spawn(async move {
            let observed = tokio::spawn(monitor.run(receiver)).await;
            let result = match observed {
                Ok(Ok(())) => Ok(()),
                _ => owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_record_launch_unknown(&owner, &cursor, "custody_monitor_failed"),
            };
            wake.notify_one();
            result
        });
        Self { stop, task }
    }
}

pub(super) struct Monitor {
    pub owner: Arc<RuntimeOwner>,
    pub wake: Arc<Notify>,
    pub cursor: PipelineCursor,
    pub process: ProcessGroup,
    pub cookie: String,
    pub attempt: PathBuf,
    pub prompt: Vec<u8>,
    pub pin: String,
    pub role: PipelineRole,
}

pub(super) struct DeathReceipt {
    pub evidence: PipelineDeathEvidence,
    status: std::process::ExitStatus,
}

pub(super) async fn birth(cookie: &str, pid: u32) -> Result<String> {
    let cookie = rrx_process_tracker::Cookie::new(cookie)?;
    tokio::task::spawn_blocking(move || -> Result<String> {
        let found = rrx_process_tracker::discover(&cookie, Default::default())?;
        let identity = found
            .processes
            .iter()
            .map(|p| p.identity())
            .find(|id| id.pid == pid)
            .context("owned leader birth unavailable")?;
        Ok(format!("{}:{}", identity.birth.0, identity.birth.1))
    })
    .await?
}

/// Every true field is derived from an observation; expiration only refuses proof.
pub(super) async fn confirm_death(
    mut process: ProcessGroup,
    cookie: &str,
    terminate: bool,
) -> Result<DeathReceipt> {
    let pid = Pid::from_raw(process.child.id().context("leader PID missing")? as i32)
        .context("invalid leader PID")?;
    if terminate {
        process = cleanup_group(process).await?;
    }
    timeout(DEATH_BOUND, process.observe_exit()).await??;
    process = cleanup_group(process).await?; // Leader is still unreaped here.
    let status = timeout(DEATH_BOUND, process.reap()).await??;
    let deadline = Instant::now() + DEATH_BOUND;
    loop {
        if test_kill_process_group(pid) == Err(rustix::io::Errno::SRCH) {
            break;
        }
        ensure!(
            Instant::now() < deadline,
            "process group not observed empty"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let cookie = Arc::new(rrx_process_tracker::Cookie::new(cookie)?);
    let discovery_attempts = discover_death(deadline, || {
        let cookie = Arc::clone(&cookie);
        async move {
            let discovery = tokio::task::spawn_blocking(move || {
                rrx_process_tracker::discover(&cookie, Default::default())
            })
            .await??;
            Ok(discovery.into())
        }
    })
    .await?;
    Ok(DeathReceipt {
        status,
        evidence: PipelineDeathEvidence {
            leader_exit_observed: true,
            group_kill_delivered: true,
            leader_reaped: true,
            group_empty: true,
            discovery_complete: true,
            matching_processes: 0,
            discovery_attempts,
        },
    })
}

impl Monitor {
    async fn run(mut self, mut stop: watch::Receiver<bool>) -> Result<()> {
        let mut stdin = self
            .process
            .child
            .stdin
            .take()
            .context("pipeline stdin missing")?;
        let mut stdout = self
            .process
            .child
            .stdout
            .take()
            .context("pipeline stdout missing")?;
        let mut stderr = self
            .process
            .child
            .stderr
            .take()
            .context("pipeline stderr missing")?;
        let codex = !matches!(self.role, PipelineRole::Review);
        let out_limit = if codex { CODEX_BYTES } else { CLAUDE_BYTES };
        let mut out = Capture::new(out_limit, false);
        let mut err = Capture::new(STDERR_BYTES, true);
        out.start(&self.attempt.join("stdout.pending"))?;
        err.start(&self.attempt.join("stderr.pending"))?;
        let mut out_buffer = [0u8; 8192];
        let mut err_buffer = [0u8; 8192];
        let mut out_done = false;
        let mut err_done = false;
        let mut input_done = false;
        let mut observed_exit = false;
        let mut issue = None;
        let prompt = &self.prompt;
        let input = timeout(Duration::from_secs(30), async move {
            let result = stdin.write_all(prompt).await;
            // ChildStdin::shutdown does not close the pipe; EOF requires drop.
            drop(stdin);
            result
        });
        tokio::pin!(input);
        let seconds = match self.role {
            PipelineRole::Implement | PipelineRole::Fix => 3600,
            PipelineRole::Review => 1800,
            PipelineRole::Confirm => 1200,
        };
        let mut deadline = Instant::now() + Duration::from_secs(seconds);
        loop {
            if observed_exit && out_done && err_done && input_done {
                break;
            }
            tokio::select! {
                result = &mut input, if !input_done => {
                    input_done = true;
                    if !matches!(result, Ok(Ok(()))) { issue = Some("stdin_failed"); break; }
                },
                read = stdout.read(&mut out_buffer), if !out_done => {
                    match read {
                        Ok(0) => out_done = true,
                        Ok(n) => match out.append(&out_buffer[..n]) {
                            Ok(true) => {},
                            Ok(false) => { issue = Some("output_overflow"); break; },
                            Err(_) => { issue = Some("output_io"); break; }
                        },
                        Err(_) => { issue = Some("output_io"); break; }
                    }
                },
                read = stderr.read(&mut err_buffer), if !err_done => {
                    match read {
                        Ok(0) => err_done = true,
                        Ok(n) => if err.append(&err_buffer[..n]).is_err() { issue = Some("output_io"); break; },
                        Err(_) => { issue = Some("output_io"); break; }
                    }
                },
                exit = self.process.observe_exit(), if !observed_exit => {
                    if exit.is_err() { issue = Some("child_observation_failed"); break; }
                    observed_exit = true;
                    // Kill pipe-holding descendants while the owned leader is a zombie.
                    self.process = match cleanup_group(self.process).await {
                        Ok(process) => process,
                        Err(_) => {
                            return self.owner.store.lock().map_err(|_| anyhow::anyhow!("state poisoned"))?
                                .pipeline_hold(&self.owner, &self.cursor, "death_unconfirmed", None);
                        }
                    };
                    deadline = Instant::now() + Duration::from_secs(10);
                },
                _ = tokio::time::sleep_until(deadline) => { issue = Some(if observed_exit { "drain_incomplete" } else { "timeout" }); break; },
                changed = stop.changed() => {
                    if changed.is_err() || *stop.borrow() { issue = Some("interrupted"); break; }
                },
            }
        }
        drop(stdout);
        drop(stderr);
        let death = confirm_death(self.process, &self.cookie, !observed_exit).await;
        let dead = match death {
            Ok(dead) => dead,
            Err(_) => {
                return self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_hold(&self.owner, &self.cursor, "death_unconfirmed", None);
            }
        };
        let stdout_name = if codex { "stdout.jsonl" } else { "result.json" };
        let streams = (|| -> Result<_> {
            let stdout = out.retain(&self.attempt, "stdout", out_done)?;
            let stderr = err.retain(&self.attempt, "stderr", err_done)?;
            let bytes = inspect(&self.attempt, "stdout", &stdout, out_limit)?;
            inspect(&self.attempt, "stderr", &stderr, STDERR_BYTES)?;
            // Preserve the collector's exact leaf-name contract, then publish
            // the already verified, read-only files under the CLI artifact names.
            std::fs::rename(self.attempt.join("stdout"), self.attempt.join(stdout_name))?;
            std::fs::rename(self.attempt.join("stderr"), self.attempt.join("stderr.log"))?;
            Ok((stdout, stderr, bytes))
        })();
        let (stdout, stderr, bytes) = match streams {
            Ok(streams) => streams,
            Err(_) => {
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_record_exit(
                        &self.owner,
                        &self.cursor,
                        &dead.evidence,
                        Some("other"),
                        &json!({"issue":"output_io"}),
                    )?;
                let mut cursor = self.cursor.clone();
                cursor.step_version += 1;
                return self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_reject(&self.owner, &cursor, "output_io");
            }
        };
        if check_last(&self.attempt, matches!(self.role, PipelineRole::Confirm)).is_err() {
            issue.get_or_insert("last_output_invalid");
        }
        let outcome = if issue.is_some() || !out_done {
            ProviderOutcome::Other
        } else if codex {
            classify_codex(&self.pin, dead.status.code(), &bytes, out.truncated)
        } else {
            classify_claude(&self.pin, dead.status.code(), &bytes, out.truncated)
        };
        let outcome = match outcome {
            ProviderOutcome::Success => "success",
            ProviderOutcome::Limit => "limit",
            ProviderOutcome::Other => "other",
        };
        let mut cursor = self.cursor;
        {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            store.pipeline_record_exit(
                &self.owner,
                &cursor,
                &dead.evidence,
                Some(outcome),
                &json!({
                    "code":dead.status.code(),"issue":issue,"stdout":stdout,"stderr":stderr,
                    "stdout_truncated":out.truncated,"stderr_truncated":err.truncated
                }),
            )?;
            cursor.step_version += 1;
            // Exit may have settled the global probe; observe its new CAS receipt.
            cursor.wait_version = store
                .pipeline_takeover_view()?
                .limit_wait
                .map(|w| w.version);
            match issue {
                Some("interrupted") => {
                    store.pipeline_record_interrupted(&self.owner, &cursor, &dead.evidence)?
                }
                Some(reason) => store.pipeline_reject(&self.owner, &cursor, reason)?,
                None => {}
            }
        }
        Ok(())
    }
}

struct Capture {
    bytes: Vec<u8>,
    limit: usize,
    tail: bool,
    truncated: bool,
    digest: Sha256,
    live: Option<(std::fs::File, PathBuf)>,
}
impl Capture {
    fn new(limit: usize, tail: bool) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
            tail,
            truncated: false,
            digest: Sha256::new(),
            live: None,
        }
    }
    fn start(&mut self, path: &Path) -> Result<()> {
        use std::os::unix::fs::OpenOptionsExt;
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)?;
        self.live = Some((file, path.to_owned()));
        Ok(())
    }
    fn append(&mut self, bytes: &[u8]) -> Result<bool> {
        use std::io::{Seek, Write};
        let overflow = self.bytes.len().saturating_add(bytes.len()) > self.limit;
        self.truncated |= overflow;
        let before = self.bytes.len();
        if self.tail {
            self.bytes.extend_from_slice(bytes);
            if self.bytes.len() > self.limit {
                self.bytes.drain(..self.bytes.len() - self.limit);
            }
        } else {
            self.bytes.extend_from_slice(
                &bytes[..bytes.len().min(self.limit.saturating_sub(self.bytes.len()))],
            );
        }
        if self.tail {
            self.digest = Sha256::new();
            self.digest.update(&self.bytes);
        } else {
            self.digest.update(&self.bytes[before..]);
        }
        if let Some((file, _)) = &mut self.live {
            if self.tail {
                file.rewind()?;
                file.write_all(&self.bytes)?;
                file.set_len(self.bytes.len() as u64)?;
            } else {
                file.write_all(&self.bytes[before..])?;
            }
        }
        Ok(self.tail || !overflow)
    }
    fn retain(&self, root: &Path, name: &str, complete: bool) -> Result<StreamEvidence> {
        if let Some((file, _)) = &self.live {
            file.sync_all()?;
        }
        retain(root, name, &self.bytes)?;
        if let Some((_, path)) = &self.live {
            std::fs::remove_file(path)?;
        }
        Ok(StreamEvidence {
            bytes: self.bytes.len() as u64,
            sha256: format!("{:x}", self.digest.clone().finalize()),
            complete,
        })
    }
}

fn check_last(root: &Path, confirm: bool) -> Result<()> {
    use std::{io::Read, os::unix::fs::PermissionsExt};
    let path = root.join(if confirm { "last.json" } else { "last.md" });
    if !path.try_exists()? {
        return Ok(());
    } // Diagnostic only; never an adoption result.
    let meta = std::fs::symlink_metadata(&path)?;
    ensure!(
        meta.is_file() && !meta.file_type().is_symlink() && meta.len() <= LAST_BYTES as u64,
        "last output exceeds bound or aliases"
    );
    let mut bytes = Vec::new();
    std::fs::File::open(&path)?
        .take((LAST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= LAST_BYTES, "last output grew beyond bound");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o400))?;
    Ok(())
}

#[cfg(test)]
mod tests;
