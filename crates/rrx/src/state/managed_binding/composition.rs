//! Private transport for the actual retained Sources/Engine/native selection.
//! There is deliberately NO producer until the marker, sole binder and protected
//! Native writers are connected. Rows, metadata and callbacks cannot build it.
use crate::{
    adapter::native::NativePhasePort,
    domain::Task,
    execution::{RuntimeOwner, workflow_source::ManagedWorkflowSources},
    runtime::{Runtime, phase_supervisor::PhaseSupervisor},
    workflow::WorkflowEngine,
};
use anyhow::{Result, ensure};
use std::sync::{Arc, Weak};

/// Non-Clone/non-Deserialize; private fields retain REAL component objects,
/// rather than a boolean readiness switch or a caller's SQL/capability witness.
pub(crate) struct InstalledDriverComposition {
    runtime: Weak<Runtime>,
    owner: Arc<RuntimeOwner>,
    phases: Weak<PhaseSupervisor>,
    original_task: Task,
    sources: Arc<ManagedWorkflowSources>,
    engine: Arc<WorkflowEngine>,
    selected: Arc<NativePhasePort>,
}
impl InstalledDriverComposition {
    pub(crate) fn validate_for(&self, owner: &RuntimeOwner, task: &Task) -> Result<()> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("installed Runtime composition ended"))?;
        let phases = self
            .phases
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("installed phase supervisor ended"))?;
        ensure!(
            std::ptr::eq(self.owner.as_ref(), owner)
                && runtime.composition_is_current(&phases)
                && phases.belongs_to(&self.owner)
                && std::ptr::eq(self.selected.owner(), owner)
                && matches!(self.selected.provider(), "claude" | "codex")
                && task.executor == self.selected.alias()
                && serde_json::to_value(task)? == serde_json::to_value(&self.original_task)?
                && self.sources.belongs_to(&self.owner)
                && self
                    .engine
                    .matches_composition(&self.owner, &self.sources, &self.selected),
            "actual installed Driver composition identity changed"
        );
        Ok(())
    }
    pub(crate) fn sources(&self) -> &Arc<ManagedWorkflowSources> {
        &self.sources
    }
    pub(crate) fn engine(&self) -> &Arc<WorkflowEngine> {
        &self.engine
    }
    pub(crate) fn provider(&self) -> &str {
        self.selected.provider()
    }
}

impl Runtime {
    /// Static absence is checked before Source preparation/Driver effects. This
    /// refusal can only be replaced by the actual private installation producer,
    /// with the real marker/binder/Native writer/retention composition complete.
    pub(crate) fn installed_driver_composition(
        &self,
        _task: &Task,
    ) -> Result<InstalledDriverComposition> {
        anyhow::bail!("managed marker, Native input and record-only binder are not composed")
    }
}
