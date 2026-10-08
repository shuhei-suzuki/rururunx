//! Typed "remains pending" shutdown outcomes. Only the six classified pending
//! exits construct the marker; poison and every other error stay unmarked.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownSite {
    /// P1: control admission not acquired within its deadline.
    Admission,
    /// P2: supervisor join not complete within its deadline.
    SupervisorJoin,
    /// P3: Task Driver poll deadline with owned handles.
    DriverPoll,
    /// P4: accepted Source originals retained.
    AcceptedOriginals,
    /// P5: retained Native jobs.
    NativeJobs,
    /// P6: retained Source handoffs.
    SourceHandoffs,
}

/// Held owned work, not a failure and never cleanup evidence.
#[derive(Debug)]
pub(crate) struct ShutdownPending {
    site: ShutdownSite,
}
impl ShutdownPending {
    pub(super) fn at(site: ShutdownSite) -> Self {
        Self { site }
    }
    pub(crate) fn site(&self) -> ShutdownSite {
        self.site
    }
}
impl std::fmt::Display for ShutdownPending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.site {
            ShutdownSite::Admission => "Runtime control admission shutdown remains pending",
            ShutdownSite::SupervisorJoin => "Runtime control loop shutdown remains pending",
            ShutdownSite::DriverPoll => "Task Driver shutdown remains pending with owned handles",
            ShutdownSite::AcceptedOriginals => {
                "Source phase shutdown remains pending with accepted originals"
            }
            ShutdownSite::NativeJobs => "Native phase shutdown remains pending with retained jobs",
            ShutdownSite::SourceHandoffs => {
                "Source handoff shutdown remains pending with retained originals"
            }
        })
    }
}
impl std::error::Error for ShutdownPending {}

/// The classified pending site of a shutdown error, if any.
pub(crate) fn pending_site(error: &anyhow::Error) -> Option<ShutdownSite> {
    error
        .downcast_ref::<ShutdownPending>()
        .map(ShutdownPending::site)
}

/// Site-scoped test park of a service loop (C-S1d P2): the selected state's
/// loop genuinely does not finish its join until the gate opens.
#[cfg(test)]
pub(crate) mod park {
    use std::{
        path::{Path, PathBuf},
        sync::{Arc, Condvar, Mutex},
    };
    type Gate = Arc<(Mutex<bool>, Condvar)>;
    static PARKED: Mutex<Vec<(PathBuf, Gate)>> = Mutex::new(Vec::new());

    pub(crate) struct LoopPark(PathBuf, Gate);
    impl LoopPark {
        pub(crate) fn open(&self) {
            *self.1.0.lock().unwrap() = true;
            self.1.1.notify_all();
            PARKED.lock().unwrap().retain(|(path, _)| path != &self.0);
        }
    }
    impl Drop for LoopPark {
        fn drop(&mut self) {
            self.open();
        }
    }
    /// Arm before the loop starts; `state` is the canonical state path.
    pub(crate) fn park_loop(state: &Path) -> LoopPark {
        let gate: Gate = Arc::new((Mutex::new(false), Condvar::new()));
        PARKED
            .lock()
            .unwrap()
            .push((state.to_path_buf(), gate.clone()));
        LoopPark(state.to_path_buf(), gate)
    }
    pub(in crate::runtime) fn wait(state: &Path) {
        let gate = PARKED
            .lock()
            .unwrap()
            .iter()
            .find(|(path, _)| path == state)
            .map(|(_, gate)| gate.clone());
        let Some(gate) = gate else { return };
        tokio::task::block_in_place(|| {
            let mut open = gate.0.lock().unwrap();
            while !*open {
                open = gate.1.wait(open).unwrap();
            }
        });
    }
}
