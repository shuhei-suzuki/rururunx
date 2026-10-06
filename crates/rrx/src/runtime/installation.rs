//! Private transport for the actual retained Sources/Engine/native selection.
//! Issued only by the one installed Runtime graph; rows and metadata are nongrant.
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
    /// SAME original Weak only; worker holds no strong Runtime across Source.
    pub(crate) fn runtime(&self) -> &Weak<Runtime> {
        &self.runtime
    }
}

impl InstalledDriverComposition {
    pub(crate) fn selected(&self) -> &Arc<NativePhasePort> {
        &self.selected
    }
    pub(crate) fn original_task(&self) -> &Task {
        &self.original_task
    }
    pub(crate) fn is_current(&self) -> bool {
        self.runtime
            .upgrade()
            .zip(self.phases.upgrade())
            .is_some_and(|(runtime, phases)| runtime.composition_is_current(&phases))
    }
}
pub(super) struct InstalledNativeGraph {
    registry: Arc<crate::adapter::AgentRegistry>,
    sources: Arc<ManagedWorkflowSources>,
    engine: Arc<WorkflowEngine>,
}
/// Bounded static diagnostic only; no retry/availability flag or authority.
pub(super) struct InstallationRefusal(Box<str>);
impl InstalledNativeGraph {
    pub(super) fn new(
        owner: Arc<RuntimeOwner>,
        config: &crate::config::Config,
    ) -> std::result::Result<Self, InstallationRefusal> {
        let registry = Arc::new(
            crate::adapter::AgentRegistry::from_managed_config(config, owner.clone())
                .map_err(|_| InstallationRefusal("managed Registry installation refused".into()))?,
        );
        let sources = Arc::new(
            ManagedWorkflowSources::new(owner.clone(), config.clone())
                .map_err(|_| InstallationRefusal("managed Sources installation refused".into()))?,
        );
        let gates = crate::execution::workflow_gates::ManagedWorkflowGates::new(
            owner.clone(),
            sources.clone(),
        )
        .map_err(|_| InstallationRefusal("managed Gates installation refused".into()))?;
        let verifier =
            crate::execution::verification::ManagedVerifier::new(owner.clone(), sources.clone())
                .map_err(|_| InstallationRefusal("managed Verifier installation refused".into()))?;
        let engine = WorkflowEngine::new(
            owner.store(),
            registry.clone(),
            config.clone(),
            sources.clone(),
            Arc::new(gates),
        )
        .and_then(|engine| engine.with_verifier(Arc::new(verifier)))
        .map_err(|_| InstallationRefusal("managed Engine installation refused".into()))?;
        Ok(Self {
            registry,
            sources,
            engine: Arc::new(engine),
        })
    }
}
impl Runtime {
    pub(crate) fn installed_driver_composition(
        self: &Arc<Self>,
        task: &Task,
    ) -> Result<InstalledDriverComposition> {
        let graph = self
            .installed
            .as_ref()
            .map_err(|refusal| anyhow::anyhow!("{}", refusal.0))?;
        ensure!(
            self.service_running(),
            "installed Runtime is not accepting Drivers"
        );
        let selected = graph.registry.native_phase_port(&task.executor)?;
        let adapter = selected.selected_adapter()?;
        ensure!(
            adapter.compatibility.is_some(),
            "installed Native compatibility declaration absent"
        );
        ensure!(
            graph
                .engine
                .matches_composition(&self.owner, &graph.sources, &selected),
            "installed Engine graph differs"
        );
        Ok(InstalledDriverComposition {
            runtime: Arc::downgrade(self),
            owner: self.owner.clone(),
            phases: Arc::downgrade(&self.phases),
            original_task: task.clone(),
            sources: graph.sources.clone(),
            engine: graph.engine.clone(),
            selected,
        })
    }
}
