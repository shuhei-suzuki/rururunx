//! Sealed original-custody witness for a start that ended before dispatch.
use super::*;

pub(crate) enum PreparationYield {
    Closed,
    RolledBack,
    Conflict,
    Uncertain,
}
pub(crate) enum NativeClosureStep {
    NotEligible,
    Preparation(PreparationYield),
    Held(anyhow::Error),
    Proof(Arc<NativeNoDispatchClosureProof>),
}
pub(crate) struct NativeNoDispatchClosureProof {
    custody: Weak<NativePreparationCustody>,
    actor: Arc<NativePreparationActor>,
    no_dispatch: Arc<prepared::PreparedPhaseNoCurrentDispatch>,
    closure: Arc<crate::state::NativeQuotaClosurePlan>,
    closed: Arc<crate::state::NativePreparationClosureCommit>,
}
impl NativeNoDispatchClosureProof {
    pub(super) fn issue(
        custody: &Arc<NativePreparationCustody>,
        actor: Arc<NativePreparationActor>,
        no_dispatch: Arc<prepared::PreparedPhaseNoCurrentDispatch>,
        closure: Arc<crate::state::NativeQuotaClosurePlan>,
        closed: Arc<crate::state::NativePreparationClosureCommit>,
    ) -> Result<Self> {
        actor.validate_original()?;
        no_dispatch.validate_original(&actor)?;
        ensure!(
            actor.is_revoked()
                && closed.matches_plan(&closure)
                && closure.matches_no_dispatch(&no_dispatch),
            "non-success proof differs from known preparation closure"
        );
        Ok(Self {
            custody: Arc::downgrade(custody),
            actor,
            no_dispatch,
            closure,
            closed,
        })
    }
    pub(crate) fn launch(&self) -> &Arc<crate::state::managed_binding::PhaseLaunchParts> {
        self.actor.launch()
    }
    pub(crate) fn validate_original(&self) -> Result<()> {
        let custody = self
            .custody
            .upgrade()
            .context("non-success custody ended")?;
        custody.validate_nonsuccess_original(
            self,
            &self.actor,
            &self.no_dispatch,
            &self.closure,
            &self.closed,
        )?;
        self.actor.validate_original()?;
        ensure!(self.actor.is_revoked(), "non-success actor is not revoked");
        self.no_dispatch.validate_original(&self.actor)
    }
    pub(crate) fn closure(
        &self,
        _: &crate::state::NonSuccessReader,
    ) -> &Arc<crate::state::NativeQuotaClosurePlan> {
        &self.closure
    }
}
