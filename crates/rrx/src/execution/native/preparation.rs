//! Original preparation custody. Empty cells and readiness commits grant no
//! helper, input, Session or process authority.
use super::*;
use crate::{execution::phase::NativeAllocation, state::managed_binding::PhaseLaunchParts};
use std::sync::atomic::{AtomicBool, Ordering};

type ClosureOriginal = (
    Arc<NativePreparationActor>,
    Arc<super::prepared::PreparedPhaseNoCurrentDispatch>,
    Arc<crate::state::NativeReadyLineage>,
    Option<Arc<crate::state::NativeQuotaClosurePlan>>,
);
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct PreparationFacts {
    pub revoked: bool,
    pub helpers_completed: bool,
    pub no_dispatch: bool,
    pub transport: bool,
    pub closure: bool,
    pub closed: bool,
    pub saved_closure: Option<(usize, i64)>,
}

/// The real Runtime job creates this empty cell before its start future. The
/// selected Native issuer alone installs an actor; public DTOs cannot do so.
pub(crate) struct NativePreparationCustody {
    allocation: Arc<NativeAllocation>,
    state: Mutex<CustodyState>,
    revoked: tokio::sync::Notify,
    parked_level: watch::Sender<bool>,
    // Display-only attention; never consulted by any admission or issuer.
    quota_unresolved: AtomicBool,
}
#[derive(Default)]
struct CustodyState {
    abandoned: bool,
    starting: bool,
    actor: Option<Arc<NativePreparationActor>>,
    plan: Option<Arc<crate::state::NativePreparationPlan>>,
    known: Option<Arc<crate::state::NativePreparationCommit>>,
    helpers: Vec<Arc<super::version::NativeVersionHelperCustody>>,
    completion: Option<Arc<super::version::NativeReadonlyHelperCompletion>>,
    compat: Option<Arc<super::compat::NativeCompatQualification>>,
    command: Option<Arc<super::prepared::NativeTransportCommand>>,
    no_dispatch: Option<Arc<super::prepared::PreparedPhaseNoCurrentDispatch>>,
    quota_plan: Option<Arc<crate::state::NativeQuotaPlan>>,
    lineage: Option<Arc<crate::state::NativeReadyLineage>>,
    parked: Option<Arc<crate::state::NativeParkedPhase>>,
    admitted: Option<Arc<crate::state::NativeQuotaAdmitted>>,
    prepared: Option<Arc<super::prepared::PreparedNativePhase>>,
    transport: Option<Arc<super::transport::NativeTransportCustody>>,
    first_parked_at: Option<i64>,
    closure: Option<Arc<crate::state::NativeQuotaClosurePlan>>,
    closed: Option<Arc<crate::state::NativePreparationClosureCommit>>,
    nonsuccess: Option<Arc<super::NativeNoDispatchClosureProof>>,
}
impl NativePreparationCustody {
    /// Reads retained facts; these booleans cannot be used by an issuer.
    #[cfg(test)]
    pub(crate) fn observed_facts(&self) -> PreparationFacts {
        let state = self.state.lock().unwrap();
        PreparationFacts {
            revoked: state.actor.as_ref().is_some_and(|a| a.is_revoked()),
            helpers_completed: state.completion.is_some(),
            no_dispatch: state.no_dispatch.is_some(),
            transport: state.transport.is_some(),
            closure: state.closure.is_some(),
            closed: state.closed.is_some(),
            saved_closure: state
                .closure
                .as_ref()
                .map(|p| (Arc::as_ptr(p) as usize, p.observed_at())),
        }
    }
    pub(super) fn validate_nonsuccess_original(
        &self,
        proof: &super::NativeNoDispatchClosureProof,
        actor: &Arc<NativePreparationActor>,
        no_dispatch: &Arc<super::prepared::PreparedPhaseNoCurrentDispatch>,
        closure: &Arc<crate::state::NativeQuotaClosurePlan>,
        closed: &Arc<crate::state::NativePreparationClosureCommit>,
    ) -> Result<()> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            state.abandoned
                && state.transport.is_none()
                && state
                    .nonsuccess
                    .as_ref()
                    .is_some_and(|p| std::ptr::eq(p.as_ref(), proof))
                && state.actor.as_ref().is_some_and(|p| Arc::ptr_eq(p, actor))
                && state
                    .no_dispatch
                    .as_ref()
                    .is_some_and(|p| Arc::ptr_eq(p, no_dispatch))
                && state
                    .closure
                    .as_ref()
                    .is_some_and(|p| Arc::ptr_eq(p, closure))
                && state
                    .closed
                    .as_ref()
                    .is_some_and(|p| Arc::ptr_eq(p, closed))
                && closed.matches_plan(closure)
                && closure.matches_no_dispatch(no_dispatch),
            "non-success original custody changed"
        );
        Ok(())
    }
    pub(crate) fn nonsuccess_step(
        self: &Arc<Self>,
        ended: &crate::runtime::StartEnded,
    ) -> super::NativeClosureStep {
        crate::runtime::assert_nonsuccess_unlocked();
        use super::NativeClosureStep;
        let result = (|| -> Result<NativeClosureStep> {
            ensure!(
                ended.matches_allocation(&self.allocation),
                "non-success ended a different allocation"
            );
            let (actor, no_dispatch, lineage, closure, closed) = {
                let state = self
                    .state
                    .lock()
                    .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
                if let Some(proof) = &state.nonsuccess {
                    return Ok(NativeClosureStep::Proof(proof.clone()));
                }
                if !state.abandoned || state.transport.is_some() {
                    return Ok(NativeClosureStep::NotEligible);
                }
                let actor = state.actor.clone().context("non-success actor absent")?;
                if !actor.is_revoked() {
                    return Ok(NativeClosureStep::NotEligible);
                }
                let no_dispatch = state
                    .no_dispatch
                    .clone()
                    .context("pre-no-dispatch start ended; Held")?;
                (
                    actor,
                    no_dispatch.clone(),
                    state
                        .lineage
                        .clone()
                        .unwrap_or_else(|| no_dispatch.issued.clone()),
                    state.closure.clone(),
                    state.closed.clone(),
                )
            };
            let Some(closed) = closed else {
                let sessions = actor
                    .sessions
                    .upgrade()
                    .context("selected Native sessions ended")?;
                return Ok(match sessions.close_prepared_step(self) {
                    super::prepared::PreparationStep::Closed => {
                        NativeClosureStep::Preparation(super::PreparationYield::Closed)
                    }
                    super::prepared::PreparationStep::RolledBack => {
                        NativeClosureStep::Preparation(super::PreparationYield::RolledBack)
                    }
                    super::prepared::PreparationStep::Conflict => {
                        NativeClosureStep::Preparation(super::PreparationYield::Conflict)
                    }
                    super::prepared::PreparationStep::Uncertain(_) => {
                        NativeClosureStep::Preparation(super::PreparationYield::Uncertain)
                    }
                    super::prepared::PreparationStep::Held(error) => NativeClosureStep::Held(error),
                });
            };
            let closure = closure.context("known non-success preparation closure absent")?;
            ensure!(
                closed.matches_plan(&closure)
                    && closure.matches(&actor, &lineage)
                    && closure.matches_no_dispatch(&no_dispatch),
                "non-success closure differs from original custody"
            );
            actor.validate_original()?;
            no_dispatch.validate_original(&actor)?;
            let proof = Arc::new(super::NativeNoDispatchClosureProof::issue(
                self,
                actor.clone(),
                no_dispatch.clone(),
                closure.clone(),
                closed.clone(),
            )?);
            let mut state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
            ensure!(
                state.abandoned
                    && state.transport.is_none()
                    && state.actor.as_ref().is_some_and(|p| Arc::ptr_eq(p, &actor))
                    && state
                        .no_dispatch
                        .as_ref()
                        .is_some_and(|p| Arc::ptr_eq(p, &no_dispatch))
                    && state
                        .closure
                        .as_ref()
                        .is_some_and(|p| Arc::ptr_eq(p, &closure))
                    && state
                        .closed
                        .as_ref()
                        .is_some_and(|p| Arc::ptr_eq(p, &closed)),
                "non-success issuer custody raced"
            );
            if let Some(original) = &state.nonsuccess {
                return Ok(NativeClosureStep::Proof(original.clone()));
            }
            state.nonsuccess = Some(proof.clone());
            Ok(NativeClosureStep::Proof(proof))
        })();
        result.unwrap_or_else(NativeClosureStep::Held)
    }
    pub(super) fn retain_transport(
        &self,
        transport: Arc<super::transport::NativeTransportCustody>,
        prepared: &Arc<super::prepared::PreparedNativePhase>,
    ) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned
                && state.transport.is_none()
                && state
                    .prepared
                    .as_ref()
                    .is_some_and(|p| Arc::ptr_eq(p, prepared))
                && transport.matches_prepared(prepared),
            "transport does not consume SAME issued Prepared"
        );
        state.transport = Some(transport);
        // Transport consumption permanently invalidates the pre-dispatch
        // witness. Its lifecycle owner remains retained by Prepared/custody.
        let no_dispatch = state.no_dispatch.take();
        drop(state);
        drop(no_dispatch);
        Ok(())
    }
    pub(super) fn clear_definitive_closure_conflict(
        &self,
        plan: &Arc<crate::state::NativeQuotaClosurePlan>,
    ) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            state.closed.is_none() && state.closure.as_ref().is_some_and(|p| Arc::ptr_eq(p, plan)),
            "closure pre-write conflict plan differs"
        );
        state.closure = None;
        Ok(())
    }
    pub(super) fn closure_original(&self) -> Result<ClosureOriginal> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            state.abandoned && state.closed.is_none() && state.transport.is_none(),
            "nongrant closure not revoked or already closed"
        );
        let no_dispatch = state
            .no_dispatch
            .clone()
            .context("same no-dispatch closure prerequisite absent")?;
        Ok((
            state.actor.clone().context("same closure actor absent")?,
            no_dispatch.clone(),
            state
                .lineage
                .clone()
                .unwrap_or_else(|| no_dispatch.issued.clone()),
            state.closure.clone(),
        ))
    }
    pub(super) fn retain_closure_plan(
        &self,
        plan: Arc<crate::state::NativeQuotaClosurePlan>,
    ) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        let no_dispatch = state
            .no_dispatch
            .as_ref()
            .context("closure no-dispatch absent")?;
        let lineage = state.lineage.as_ref().unwrap_or(&no_dispatch.issued);
        ensure!(
            state.abandoned
                && state.closed.is_none()
                && state.closure.is_none()
                && state
                    .actor
                    .as_ref()
                    .is_some_and(|a| plan.matches(a, lineage)),
            "closure plan replaced original lineage"
        );
        state.closure = Some(plan);
        Ok(())
    }
    pub(super) fn retain_closed(
        &self,
        known: crate::state::NativePreparationClosureCommit,
    ) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            state.closed.is_none()
                && state
                    .closure
                    .as_ref()
                    .is_some_and(|p| known.matches_plan(p)),
            "known closure differs from SAME retained plan"
        );
        state.closed = Some(Arc::new(known));
        Ok(())
    }
    pub(super) fn prepared_original(&self) -> Result<super::prepared::PreparedNativePhase> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned && state.prepared.is_none() && state.parked.is_none(),
            "prepared issuer operation already ended or parked"
        );
        let value = super::prepared::PreparedNativePhase {
            actor: state.actor.clone().context("same prepared actor absent")?,
            known: state
                .known
                .clone()
                .context("same known preparation absent")?,
            version: state
                .helpers
                .first()
                .cloned()
                .context("same version helper absent")?,
            completion: state
                .completion
                .clone()
                .context("same helper completion absent")?,
            compat: state
                .compat
                .clone()
                .context("same captured compatibility absent")?,
            command: state
                .command
                .clone()
                .context("same retained command absent")?,
            quota: state
                .admitted
                .clone()
                .context("same quota admission absent")?,
        };
        ensure!(
            state
                .no_dispatch
                .as_ref()
                .is_some_and(|n| value.quota.matches(&value.actor, n)),
            "prepared quota belongs to another no-dispatch/actor"
        );
        Ok(value)
    }
    pub(super) fn retain_prepared(
        &self,
        value: Arc<super::prepared::PreparedNativePhase>,
    ) -> Result<Arc<super::prepared::PreparedNativePhase>> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned
                && state
                    .actor
                    .as_ref()
                    .is_some_and(|a| Arc::ptr_eq(a, &value.actor))
                && state
                    .known
                    .as_ref()
                    .is_some_and(|k| Arc::ptr_eq(k, &value.known))
                && state
                    .helpers
                    .first()
                    .is_some_and(|v| Arc::ptr_eq(v, &value.version))
                && state
                    .completion
                    .as_ref()
                    .is_some_and(|c| Arc::ptr_eq(c, &value.completion))
                && state
                    .compat
                    .as_ref()
                    .is_some_and(|c| Arc::ptr_eq(c, &value.compat))
                && state
                    .command
                    .as_ref()
                    .is_some_and(|c| Arc::ptr_eq(c, &value.command))
                && state
                    .admitted
                    .as_ref()
                    .is_some_and(|q| Arc::ptr_eq(q, &value.quota)),
            "prepared conjunct original custody pointers changed"
        );
        if let Some(original) = &state.prepared {
            return Ok(original.clone());
        }
        state.prepared = Some(value.clone());
        Ok(value)
    }
    pub(crate) fn parked_updates(&self) -> watch::Receiver<bool> {
        self.parked_level.subscribe()
    }
    pub(crate) fn quota_attention(&self) -> bool {
        self.quota_unresolved.load(Ordering::SeqCst)
    }
    pub(super) fn report_unresolved_head(&self, cause: &anyhow::Error) {
        self.quota_unresolved.store(true, Ordering::SeqCst);
        eprintln!("rrx native preparation: legacy head unresolved: {cause:#}");
        self.parked_level.send_replace(false);
    }
    pub(super) async fn revocation(&self) {
        loop {
            let notified = self.revoked.notified();
            if self.state.lock().is_ok_and(|s| s.abandoned) {
                return;
            }
            notified.await;
        }
    }
    pub(super) fn quota_original(
        &self,
    ) -> Result<(
        Arc<NativePreparationActor>,
        Arc<super::prepared::PreparedPhaseNoCurrentDispatch>,
        Arc<crate::state::NativeReadyLineage>,
    )> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned && state.admitted.is_none() && state.prepared.is_none(),
            "quota original operation ended or admitted"
        );
        let no_dispatch = state
            .no_dispatch
            .clone()
            .context("same no-dispatch absent")?;
        Ok((
            state.actor.clone().context("same actor absent")?,
            no_dispatch.clone(),
            state
                .lineage
                .clone()
                .unwrap_or_else(|| no_dispatch.issued.clone()),
        ))
    }
    pub(super) fn retain_quota_plan(&self, plan: Arc<crate::state::NativeQuotaPlan>) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        let no_dispatch = state
            .no_dispatch
            .as_ref()
            .context("same no-dispatch absent")?;
        let lineage = state.lineage.as_ref().unwrap_or(&no_dispatch.issued);
        ensure!(
            !state.abandoned
                && state.admitted.is_none()
                && state.actor.as_ref().is_some_and(|a| plan.matches_actor(a))
                && plan.matches_pre(lineage, no_dispatch),
            "quota plan is not SAME operation/lineage"
        );
        state.quota_plan = Some(plan);
        Ok(())
    }
    pub(super) fn retain_quota_outcome(
        &self,
        outcome: &crate::state::NativeQuotaOutcome,
    ) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        let plan = state
            .quota_plan
            .as_ref()
            .context("same quota plan absent")?;
        ensure!(state.admitted.is_none(), "quota admission already known");
        let parked = match outcome {
            crate::state::NativeQuotaOutcome::Admitted(value) => {
                ensure!(value.matches_plan(plan), "admitted another quota plan");
                state.lineage = Some(value.lineage().clone());
                state.parked = None;
                state.admitted = Some(value.clone());
                false
            }
            crate::state::NativeQuotaOutcome::Parked(value) => {
                ensure!(value.matches_plan(plan), "parked another quota plan");
                state.lineage = Some(value.lineage().clone());
                state.parked = Some(value.clone());
                state.first_parked_at.get_or_insert(now_ms());
                true
            }
        };
        drop(state);
        self.quota_unresolved.store(false, Ordering::SeqCst);
        self.parked_level.send_replace(parked);
        Ok(())
    }
    pub(super) fn no_dispatch_original(
        &self,
    ) -> Result<(
        Arc<NativePreparationActor>,
        Arc<crate::state::NativePreparationCommit>,
        Arc<super::version::NativeReadonlyHelperCompletion>,
    )> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned
                && state.no_dispatch.is_none()
                && state.compat.is_some()
                && state.command.is_some(),
            "SAME no-dispatch prerequisites absent"
        );
        let actor = state.actor.clone().context("original actor absent")?;
        let known = state
            .known
            .clone()
            .context("original known preparation absent")?;
        let completion = state
            .completion
            .clone()
            .context("original helper completion absent")?;
        let helpers = state.helpers.clone();
        drop(state);
        ensure!(
            helpers.len() == super::readonly::GIT_ACTIONS + 1,
            "finite complete helper manifest absent"
        );
        for helper in helpers {
            helper.closed()?;
        }
        Ok((actor, known, completion))
    }
    pub(super) fn retain_no_dispatch(
        &self,
        value: Arc<super::prepared::PreparedPhaseNoCurrentDispatch>,
    ) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned
                && state.no_dispatch.is_none()
                && state
                    .completion
                    .as_ref()
                    .is_some_and(|c| Arc::ptr_eq(c, &value.completion)),
            "no-dispatch install original differs"
        );
        state.no_dispatch = Some(value);
        Ok(())
    }
    pub(super) fn no_dispatch_matches(
        &self,
        value: &super::prepared::PreparedPhaseNoCurrentDispatch,
    ) -> bool {
        self.state.lock().is_ok_and(|s| {
            s.no_dispatch
                .as_ref()
                .is_some_and(|v| std::ptr::eq(v.as_ref(), value))
        })
    }
    fn retain_compatible_command(
        &self,
        compat: Arc<super::compat::NativeCompatQualification>,
        command: Arc<super::prepared::NativeTransportCommand>,
    ) -> Result<()> {
        // All physical, encoding and compatibility work preceded this lock.
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned && state.compat.is_none() && state.command.is_none(),
            "compatible command already installed or revoked"
        );
        ensure!(
            state
                .actor
                .as_ref()
                .zip(state.helpers.first())
                .is_some_and(|(a, v)| compat.matches(a, v)),
            "compatible command original actor/version differs"
        );
        state.compat = Some(compat);
        state.command = Some(command);
        Ok(())
    }
    pub(crate) fn new(allocation: Arc<NativeAllocation>) -> Arc<Self> {
        Arc::new(Self {
            allocation,
            state: Mutex::new(CustodyState::default()),
            revoked: tokio::sync::Notify::new(),
            parked_level: watch::channel(false).0,
            quota_unresolved: AtomicBool::new(false),
        })
    }
    pub(crate) fn matches_allocation(&self, allocation: &Arc<NativeAllocation>) -> bool {
        Arc::ptr_eq(&self.allocation, allocation)
    }
    pub(crate) fn abandon(&self) {
        // No Store/Unit destruction, guard release or no-child inference. The
        // original sibling plan survives a returned error or canceled future.
        let transport = if let Ok(mut state) = self.state.lock() {
            state.abandoned = true;
            if let Some(actor) = &state.actor {
                actor.revoked.store(true, Ordering::Release);
            }
            for helper in &state.helpers {
                helper.abandon();
            }
            state.transport.clone()
        } else {
            None
        };
        if let Some(transport) = transport {
            transport.request_stop();
        }
        self.revoked.notify_waiters();
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
        state.known = Some(Arc::new(commit));
        Ok(())
    }
    pub(super) fn version_original(&self) -> Result<Arc<crate::state::NativePreparationCommit>> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned && state.helpers.is_empty(),
            "original version stage held or already installed"
        );
        state
            .known
            .clone()
            .context("same known readiness commit absent")
    }
    pub(super) fn state_actor(&self) -> Result<Arc<NativePreparationActor>> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(!state.abandoned, "original preparation abandoned");
        state
            .actor
            .clone()
            .context("original preparation actor absent")
    }
    pub(super) fn retain_helper(
        &self,
        helper: Arc<super::version::NativeVersionHelperCustody>,
    ) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned
                && state.helpers.len() < super::readonly::HELPER_LIMIT
                && state
                    .actor
                    .as_ref()
                    .is_some_and(|actor| helper.matches_actor(actor)),
            "original version helper custody changed"
        );
        state.helpers.push(helper);
        Ok(())
    }
    pub(super) fn next_output_limit(
        &self,
        action: &super::readonly::NativePhaseHelperAction,
    ) -> Result<usize> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned && state.helpers.len() < super::readonly::HELPER_LIMIT,
            "finite preparation helper budget exhausted or revoked"
        );
        let helpers = state.helpers.clone();
        drop(state);
        let captured = helpers.iter().try_fold(0usize, |sum, helper| {
            helper.closed()?;
            sum.checked_add(helper.captured_bytes()?)
                .context("preparation aggregate capture overflow")
        })?;
        super::readonly::next_output_budget(helpers.len(), captured, action)
    }
    pub(super) fn qualified_history(
        &self,
    ) -> Result<Vec<Arc<crate::state::NativeHelperSettlementCommit>>> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned && state.completion.is_none(),
            "original helper history completion already held"
        );
        let helpers = state.helpers.clone();
        drop(state);
        helpers.iter().map(|h| h.closed()).collect()
    }
    pub(super) fn retain_completion(
        &self,
        completion: Arc<super::version::NativeReadonlyHelperCompletion>,
    ) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
        ensure!(
            !state.abandoned
                && state.completion.is_none()
                && state
                    .actor
                    .as_ref()
                    .is_some_and(|a| completion.matches_actor(a)),
            "original helper history completion custody differs"
        );
        state.completion = Some(completion);
        Ok(())
    }
    /// Nongrant confirmation of the same saved postimage. A wake cannot
    /// replace the actor/plan, replay preparation, or reopen a revoked actor.
    pub(crate) async fn reconcile_known_commit(self: &Arc<Self>) -> Result<()> {
        let transport = {
            let state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
            state.transport.clone().zip(state.actor.clone())
        };
        if let Some((transport, actor)) = transport {
            let sessions = actor
                .sessions
                .upgrade()
                .context("actual transport issuer ended")?;
            return transport.reconcile(&sessions.owner).await;
        }
        let closure_actor = {
            let state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
            if state.closed.is_some() {
                return Ok(());
            }
            if state.abandoned && state.no_dispatch.is_some() {
                state.actor.clone()
            } else {
                None
            }
        };
        if let Some(actor) = closure_actor {
            let sessions = actor
                .sessions
                .upgrade()
                .context("actual preparation closure issuer ended")?;
            return sessions.close_prepared_on_revocation(self).await;
        }
        let helper = {
            let state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
            state.helpers.last().cloned()
        };
        if let Some(helper) = helper {
            let actor = {
                let state = self
                    .state
                    .lock()
                    .map_err(|_| anyhow::anyhow!("preparation custody poisoned"))?;
                state
                    .actor
                    .clone()
                    .context("original helper actor unavailable")?
            };
            let sessions = actor
                .sessions
                .upgrade()
                .context("actual preparation issuer ended")?;
            return helper.reconcile(&sessions.owner);
        }
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
            for helper in &state.helpers {
                helper.abandon();
            }
            (
                state.completion.take(),
                std::mem::take(&mut state.helpers),
                state.known.take(),
                state.plan.take(),
                state.actor.take(),
            )
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
    pub(crate) fn is_revoked(&self) -> bool {
        self.revoked.load(Ordering::Acquire)
    }
    pub(super) fn release_gate(&self) {
        let gate = self
            ._start
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        drop(gate);
    }
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
    ) -> Result<Arc<super::prepared::PreparedNativePhase>> {
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
        let plan = crate::state::Store::plan_native_preparation(&self.owner, actor.clone())?;
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
        let compat = super::compat::qualify_compat_static(&actor)?;
        let version = self.prepare_phase_version(custody.clone()).await?;
        let completion = self
            .prepare_phase_git(custody.clone(), version.clone())
            .await?;
        let compat = compat.observe(version)?;
        #[cfg(test)]
        self.observe_preparation(PreparationObservation::BeforeCommand)
            .await;
        let command =
            super::prepared::plan_native_command(&self.owner, &actor, &completion, &compat)?;
        custody.retain_compatible_command(compat, command)?;
        self.issue_no_current_dispatch(&custody).await?;
        self.prepare_phase_quota(&custody).await?;
        self.issue_prepared(&custody)
    }
}
