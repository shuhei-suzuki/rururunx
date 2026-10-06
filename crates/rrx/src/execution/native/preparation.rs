//! Original preparation custody. Empty cells and readiness commits grant no
//! helper, input, Session or process authority.
use super::*;
use crate::{execution::phase::NativeAllocation, state::managed_binding::PhaseLaunchParts};
use std::sync::atomic::{AtomicBool, Ordering};

/// The real Runtime job creates this empty cell before its start future. The
/// selected Native issuer alone installs an actor; public DTOs cannot do so.
pub(crate) struct NativePreparationCustody {
    allocation: Arc<NativeAllocation>,
    state: Mutex<CustodyState>,
}
#[derive(Default)]
struct CustodyState {
    abandoned: bool,
    starting: bool,
    actor: Option<Arc<NativePreparationActor>>,
    plan: Option<Arc<crate::state::NativePreparationPlan>>,
    known: Option<crate::state::NativePreparationCommit>,
}
impl NativePreparationCustody {
    pub(crate) fn new(allocation: Arc<NativeAllocation>) -> Arc<Self> {
        Arc::new(Self {
            allocation,
            state: Mutex::new(CustodyState::default()),
        })
    }
    pub(crate) fn matches_allocation(&self, allocation: &Arc<NativeAllocation>) -> bool {
        Arc::ptr_eq(&self.allocation, allocation)
    }
    pub(crate) fn abandon(&self) {
        // No Store/Unit destruction, guard release or no-child inference. The
        // original sibling plan survives a returned error or canceled future.
        if let Ok(mut state) = self.state.lock() {
            state.abandoned = true;
            if let Some(actor) = &state.actor {
                actor.revoked.store(true, Ordering::Release);
            }
        }
    }
    fn claim_start(&self) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned && !state.starting && state.actor.is_none(),
            "original preparation start already claimed or abandoned"
        );
        state.starting = true;
        Ok(())
    }
    fn install_actor(&self, actor: Arc<NativePreparationActor>) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned && state.starting && state.actor.is_none() && state.plan.is_none(),
            "original preparation was already installed or abandoned"
        );
        state.actor = Some(actor);
        Ok(())
    }
    fn retain_plan(&self, plan: Arc<crate::state::NativePreparationPlan>) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned
                && state.plan.is_none()
                && state
                    .actor
                    .as_ref()
                    .is_some_and(|actor| plan.matches_actor(actor)),
            "original preparation plan custody changed"
        );
        state.plan = Some(plan);
        Ok(())
    }
    fn retain_commit(&self, commit: crate::state::NativePreparationCommit) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            state.known.is_none()
                && state
                    .plan
                    .as_ref()
                    .is_some_and(|plan| commit.matches_plan(plan)),
            "known preparation commit differs from saved original plan"
        );
        // A stop can revoke future work while this factual known commit remains.
        state.known = Some(commit);
        Ok(())
    }
    /// Nongrant confirmation of the same saved postimage. A wake cannot
    /// replace the actor/plan, replay preparation, or reopen a revoked actor.
    pub(crate) async fn reconcile_known_commit(self: &Arc<Self>) -> Result<()> {
        let plan = {
            let state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
            if state.known.is_some() {
                return Ok(());
            }
            state
                .plan
                .clone()
                .context("original preparation plan unavailable")?
        };
        let actor = plan.actor();
        let sessions = actor
            .sessions
            .upgrade()
            .context("actual preparation issuer ended")?;
        let launch = actor.launch().clone();
        let admission = launch.admission().enter(launch.clone()).await?;
        let commit = {
            let mut store = sessions
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            store.confirm_native_preparation(plan, &admission)?
        };
        self.retain_commit(commit)?;
        Ok(())
    }
}
impl Drop for NativePreparationCustody {
    fn drop(&mut self) {
        // Final memory destruction is no logical closure or across-epoch
        // proof. Revoke first; release the final siblings outside any lock.
        let siblings = {
            let state = self
                .state
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(actor) = &state.actor {
                actor.revoked.store(true, Ordering::Release);
            }
            (state.known.take(), state.plan.take(), state.actor.take())
        };
        drop(siblings);
    }
}

/// No constructor outside the actual selected sessions entry. The guard is
/// acquired from that entry's real same-Unit gate, not a caller-supplied mutex.
pub(crate) struct NativePreparationActor {
    launch: Arc<PhaseLaunchParts>,
    sessions: Weak<NativeSessions>,
    custody: Weak<NativePreparationCustody>,
    revoked: AtomicBool,
    _start: Mutex<Option<tokio::sync::OwnedMutexGuard<()>>>,
}
impl NativePreparationActor {
    pub(crate) fn launch(&self) -> &Arc<PhaseLaunchParts> {
        &self.launch
    }
    /// Immutable originalness and actual selected vtable check; no Queue or
    /// Source mutex, snapshot/encoding, or SharedStore reentry occurs here.
    pub(crate) fn validate_original(&self) -> Result<()> {
        let original = self
            .custody
            .upgrade()
            .context("original preparation custody ended")?;
        let linked = self
            .launch
            .preparation_custody()
            .upgrade()
            .context("original job preparation custody ended")?;
        let sessions = self
            .sessions
            .upgrade()
            .context("actual selected Native sessions ended")?;
        let adapter = self
            .launch
            .allocation()
            .selected_port()
            .selected_adapter()?;
        ensure!(
            Arc::ptr_eq(&original, &linked)
                && original.matches_allocation(self.launch.allocation())
                && Arc::ptr_eq(&adapter.sessions, &sessions),
            "original Native preparation replaced"
        );
        Ok(())
    }
    pub(crate) fn validate_open(&self) -> Result<()> {
        self.validate_original()?;
        ensure!(
            !self.revoked.load(Ordering::Acquire),
            "original Native preparation revoked"
        );
        Ok(())
    }
}

impl NativeSessions {
    pub(super) async fn begin_phase_preparation(
        &self,
        launch: Arc<PhaseLaunchParts>,
        custody: Arc<NativePreparationCustody>,
    ) -> Result<()> {
        let adapter = launch.allocation().selected_port().selected_adapter()?;
        ensure!(
            std::ptr::eq(adapter.sessions.as_ref(), self)
                && Arc::ptr_eq(&adapter.owner, &self.owner),
            "preparation uses a different selected Native issuer"
        );
        let linked = launch
            .preparation_custody()
            .upgrade()
            .context("original job custody ended")?;
        ensure!(
            Arc::ptr_eq(&custody, &linked)
                && custody.matches_allocation(launch.allocation())
                && launch.is_retained(),
            "preparation does not use the same retained job custody"
        );
        custody.claim_start()?;
        let gate = {
            let mut starts = self
                .starts
                .lock()
                .map_err(|_| anyhow::anyhow!("Native start gate poisoned"))?;
            starts.retain(|_, weak| weak.strong_count() > 0);
            let id = launch.allocation().facts().unit_id;
            if let Some(gate) = starts.get(&id).and_then(Weak::upgrade) {
                gate
            } else {
                let gate = Arc::new(tokio::sync::Mutex::new(()));
                starts.insert(id, Arc::downgrade(&gate));
                gate
            }
        };
        let actor = Arc::new(NativePreparationActor {
            launch: launch.clone(),
            sessions: Arc::downgrade(&adapter.sessions),
            custody: Arc::downgrade(&custody),
            revoked: AtomicBool::new(false),
            _start: Mutex::new(Some(
                gate.try_lock_owned()
                    .map_err(|_| anyhow::anyhow!("same-Unit Native start remains held"))?,
            )),
        });
        // Independent Root job custody owns the actor before any snapshot or
        // Immediate transaction. Neither actor nor plan strongly returns to it.
        custody.install_actor(actor.clone())?;
        {
            let mut index = self
                .preparations
                .lock()
                .map_err(|_| anyhow::anyhow!("Native preparation index poisoned"))?;
            index.retain(|_, weak| weak.strong_count() > 0);
            let id = launch.allocation().facts().unit_id;
            ensure!(
                !index.contains_key(&id) && index.len() < 128,
                "Native preparation original index already held or full"
            );
            index.insert(id, Arc::downgrade(&custody));
        }
        actor.validate_open()?;
        let plan = crate::state::Store::plan_native_preparation(&self.owner, actor)?;
        custody.retain_plan(plan.clone())?;
        let admission = launch.admission().enter(launch.clone()).await?;
        let commit = {
            // This real start keeps the SAME strong custody outside Store for
            // the complete transaction. Temporary Weak upgrades under Store
            // cannot become the final custodian or release the guard there.
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            store.begin_native_preparation(plan, &admission)?
        };
        custody.retain_commit(commit)?;
        drop(admission);
        // First-stage readiness is factual only. Actual helper/quota/transport
        // consumers are not composed; never fall through to generic admission.
        anyhow::bail!(
            "original Native preparation retained; helper/quota/transport composition unavailable"
        )
    }
}
