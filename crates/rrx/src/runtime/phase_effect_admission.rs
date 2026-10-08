//! The actual stop boundary; its guard alone grants no Native effect.
use super::phase_supervisor::PhaseLaunchParts;
use crate::execution::RuntimeOwner;
use anyhow::{Result, ensure};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Mutex, OwnedMutexGuard};

/// Shared original Runtime objects, with no strong Runtime/queue/job backlink.
pub(crate) struct PhaseEffectAdmission {
    owner: Arc<RuntimeOwner>,
    control: Arc<Mutex<()>>,
    running: Arc<AtomicBool>,
    stopping: Arc<AtomicBool>,
}

/// A local synchronous section, never stored in a Native actor or I/O future.
/// Native writers must ALSO validate current, Driver, preparation and pair.
pub(crate) struct PhaseEffectAdmissionGuard {
    admission: Arc<PhaseEffectAdmission>,
    launch: Arc<PhaseLaunchParts>,
    _control: OwnedMutexGuard<()>,
}

impl PhaseEffectAdmission {
    pub(super) fn new(
        owner: Arc<RuntimeOwner>,
        control: Arc<Mutex<()>>,
        running: Arc<AtomicBool>,
        stopping: Arc<AtomicBool>,
    ) -> Arc<Self> {
        Arc::new(Self {
            owner,
            control,
            running,
            stopping,
        })
    }

    fn accepting(&self) -> bool {
        self.running.load(Ordering::SeqCst) && !self.stopping.load(Ordering::SeqCst)
    }

    fn matches_launch(self: &Arc<Self>, launch: &PhaseLaunchParts) -> bool {
        Arc::ptr_eq(self, launch.admission())
            && std::ptr::eq(
                self.owner.as_ref(),
                launch.allocation().selected_port().owner(),
            )
    }

    pub(crate) async fn enter(
        self: &Arc<Self>,
        launch: Arc<PhaseLaunchParts>,
    ) -> Result<PhaseEffectAdmissionGuard> {
        let control = self.control.clone().lock_owned().await;
        self.admit(launch, control)
    }

    /// Root's non-waiting variant: the same checks, but a held control
    /// admission (for example a shutdown joining the service loop) is `None`.
    pub(crate) fn try_enter(
        self: &Arc<Self>,
        launch: Arc<PhaseLaunchParts>,
    ) -> Result<Option<PhaseEffectAdmissionGuard>> {
        let Ok(control) = self.control.clone().try_lock_owned() else {
            return Ok(None);
        };
        self.admit(launch, control).map(Some)
    }

    fn admit(
        self: &Arc<Self>,
        launch: Arc<PhaseLaunchParts>,
        control: OwnedMutexGuard<()>,
    ) -> Result<PhaseEffectAdmissionGuard> {
        ensure!(self.accepting(), "Runtime is not accepting Native effects");
        ensure!(
            self.matches_launch(&launch),
            "foreign effect admission origin"
        );
        // Queue inspection ends before the guard reaches any Store consumer.
        // It is original retained membership, not a current/Driver/input grant.
        ensure!(launch.is_retained(), "Native launch is no longer retained");
        Ok(PhaseEffectAdmissionGuard {
            admission: self.clone(),
            launch,
            _control: control,
        })
    }
}

impl PhaseEffectAdmissionGuard {
    /// Safe to conjoin inside Immediate: no queue/Source mutex or snapshot read.
    /// Identity/state only; never substitutes for the private Native eligibility.
    pub(crate) fn validate_for(&self, launch: &PhaseLaunchParts) -> Result<()> {
        ensure!(
            std::ptr::eq(self.launch.as_ref(), launch)
                && self.admission.matches_launch(launch)
                && self.admission.accepting(),
            "original Native effect admission changed"
        );
        Ok(())
    }
}
