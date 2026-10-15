//! The current adapter has no implemented workload owner/dispatch producer.
//! Test inputs exercise the same refusal boundary, never production readiness.
use crate::adapter::{AdapterResult, ErrorKind};

pub(super) const UNAVAILABLE: &str = "native workload ownership and dispatch producer unavailable";

#[derive(Clone, Default)]
pub(super) struct Availability {
    #[cfg(test)]
    inputs: std::sync::Arc<FixtureInputs>,
}

#[derive(Clone, Copy)]
pub(super) enum Site {
    GitResolve,
    GitExecution,
    ScopeAccess,
    Filesystem,
    ExecutableMetadata,
    Version,
    Transport,
    Connection,
    Frame,
    Grant,
}

#[cfg(test)]
#[derive(Default)]
struct FixtureInputs {
    backend: std::sync::atomic::AtomicBool,
    producer: std::sync::atomic::AtomicBool,
    sites: [std::sync::atomic::AtomicU64; 10],
    git_executable: std::sync::Mutex<Option<std::path::PathBuf>>,
}

impl Availability {
    pub(super) fn record(&self, _site: Site) {
        #[cfg(test)]
        self.inputs.sites[_site as usize].fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub(super) fn resolve_git(&self) -> AdapterResult<std::path::PathBuf> {
        self.require()?;
        self.record(Site::GitResolve);
        // Input substitution only: the same gate and creating route remain.
        #[cfg(test)]
        if let Some(path) = self.inputs.git_executable.lock().unwrap().clone() {
            return Ok(path);
        }
        crate::adapter::resolve_executable("git")
    }
    pub(super) fn require(&self) -> AdapterResult<()> {
        #[cfg(test)]
        if self
            .inputs
            .backend
            .load(std::sync::atomic::Ordering::SeqCst)
            && self
                .inputs
                .producer
                .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Ok(());
        }
        Err(super::protocol::failure(
            ErrorKind::UnsupportedCapability,
            UNAVAILABLE,
        ))
    }

    // The single private acceptance seam. Neither input alone permits effects.
    #[cfg(test)]
    pub(super) fn set_fixture_inputs(&self, backend: bool, producer: bool) {
        self.inputs
            .backend
            .store(backend, std::sync::atomic::Ordering::SeqCst);
        self.inputs
            .producer
            .store(producer, std::sync::atomic::Ordering::SeqCst);
    }

    #[cfg(test)]
    pub(super) fn sites(&self) -> [u64; 10] {
        std::array::from_fn(|i| self.inputs.sites[i].load(std::sync::atomic::Ordering::SeqCst))
    }

    #[cfg(test)]
    pub(super) fn set_git_input(&self, executable: std::path::PathBuf) {
        assert!(executable.is_absolute());
        *self.inputs.git_executable.lock().unwrap() = Some(executable);
    }
}

#[cfg(test)]
pub(super) fn component_availability() -> Availability {
    let availability = Availability::default();
    availability.set_fixture_inputs(true, true);
    availability
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    #[tokio::test]
    async fn direct_creating_helpers_refuse_before_work_and_preserve_uncertainty() {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        let marker = directory.path().join("effect");
        let executable = directory.path().join("sentinel");
        std::fs::write(
            &executable,
            format!(
                "#!/bin/sh\nprintf invoked > '{}'\nexit 0\n",
                marker.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        for (backend, producer) in [(false, false), (true, false), (false, true)] {
            let availability = Availability::default();
            availability.set_fixture_inputs(backend, producer);
            let uncertain = Arc::new(AtomicBool::new(false));
            let acted = Arc::new(AtomicBool::new(false));
            let action = acted.clone();
            let result = super::super::ownership::filesystem(&availability, move || {
                action.store(true, Ordering::SeqCst);
                Ok(())
            })
            .await;
            assert!(!acted.load(Ordering::SeqCst));
            assert!(result.is_err());
            let preparation = super::super::preparation::Preparation::new();
            let result = super::super::preparation::bounded_git(
                &availability,
                &executable,
                directory.path(),
                &[],
                vec![],
                tokio::time::Instant::now() + std::time::Duration::from_secs(5),
                uncertain.clone(),
                &preparation,
            )
            .await;
            assert!(!marker.exists());
            assert_eq!(availability.sites(), [0; 10]);
            assert!(result.is_err());
            let result = super::super::transport::NativeServer::launch_preparing(
                &availability,
                &executable,
                directory.path(),
                None,
                vec![],
                uncertain.clone(),
                &preparation,
            )
            .await;
            let error = match result {
                Err(error) => error,
                Ok(native) => {
                    native.shutdown().await.unwrap();
                    panic!("empty boundary created a native transport");
                }
            };
            assert!(!marker.exists());
            assert!(!uncertain.load(Ordering::SeqCst));
            assert_eq!(availability.sites(), [0; 10]);
            assert_eq!(error.kind, ErrorKind::UnsupportedCapability);
            assert_eq!(error.message, UNAVAILABLE);
        }
    }
}
