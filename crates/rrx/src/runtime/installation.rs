//! Private transport for the actual retained Sources/Engine/native selection.
//! Issued only by the one installed Runtime graph; rows and metadata are nongrant.
use crate::state::managed_binding::{member_digest, roster_digest};
use crate::{
    adapter::native::NativePhasePort,
    domain::{SessionRole, Task, TaskId},
    execution::{RuntimeOwner, workflow_source::ManagedWorkflowSources},
    runtime::{Runtime, phase_supervisor::PhaseSupervisor},
    workflow::WorkflowEngine,
};
use crate::{runtime::driver::WorkerLifetime, state::DriverPreparationAdvance};
use anyhow::{Result, ensure};
use std::sync::{Arc, Weak};

struct CompositionIdentity {
    task: TaskId,
}
/// Only the original composition issuer's module constructs this identity.
pub(crate) struct ActivationRoster {
    composition: Arc<CompositionIdentity>,
    task: TaskId,
    executor: String,
    reviewers: Vec<String>,
    installation: uuid::Uuid,
    members: Vec<String>,
    digest: String,
    runtime: Weak<Runtime>,
    phases: Weak<PhaseSupervisor>,
    owner: Arc<RuntimeOwner>,
}
impl ActivationRoster {
    fn new(
        composition: &InstalledDriverComposition,
        installation: uuid::Uuid,
        members: Vec<String>,
    ) -> Result<Self> {
        let original = composition.original_task();
        let digest = roster_digest(&members)?;
        Ok(Self {
            composition: composition.identity.clone(),
            task: original.id,
            executor: original.executor.clone(),
            reviewers: original.reviewers.clone(),
            installation,
            members,
            digest,
            runtime: composition.runtime.clone(),
            phases: composition.phases.clone(),
            owner: composition.owner.clone(),
        })
    }
    pub(crate) fn task(&self) -> TaskId {
        self.task
    }
    pub(crate) fn executor(&self) -> &str {
        &self.executor
    }
    pub(crate) fn reviewers(&self) -> &[String] {
        &self.reviewers
    }
    pub(crate) fn installation(&self) -> uuid::Uuid {
        self.installation
    }
    pub(crate) fn members(&self) -> &[String] {
        &self.members
    }
    pub(crate) fn digest(&self) -> &str {
        &self.digest
    }
    pub(crate) fn owner(&self) -> &Arc<RuntimeOwner> {
        &self.owner
    }
    pub(crate) fn is_same_composition(&self, composition: &InstalledDriverComposition) -> bool {
        Arc::ptr_eq(&self.composition, &composition.identity)
            && self.composition.task == self.task
            && self.task == composition.original_task.id
            && Weak::ptr_eq(&self.runtime, &composition.runtime)
            && Weak::ptr_eq(&self.phases, &composition.phases)
            && Arc::ptr_eq(&self.owner, &composition.owner)
    }
}
/// Stop exclusion only: no SQL permission or Native grant. Field order drops
/// admission before the last strong Runtime reference.
pub(crate) struct ActivationAdmission {
    _admission: tokio::sync::OwnedMutexGuard<()>,
    _runtime: Arc<Runtime>,
}

/// Stop exclusion for one admitted success segment (closure, confirmation or
/// Driver publication): no SQL permission or Native grant. Field order drops
/// the admission before the last strong Runtime reference.
pub(crate) struct SuccessAdmission {
    _admission: tokio::sync::OwnedMutexGuard<()>,
    runtime: Arc<Runtime>,
}
impl SuccessAdmission {
    pub(crate) fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }
}
impl Runtime {
    /// Root path: upgrade the service task's Weak, then a NON-BLOCKING
    /// admission try while holding that strong Arc. Ok(None) when busy (for
    /// example, shutdown holds it while joining the service task): the
    /// action is kept and nothing is attempted. The Root never awaits it.
    pub(crate) fn try_admit_root_success(weak: &Weak<Runtime>) -> Result<Option<SuccessAdmission>> {
        let runtime = weak
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("success Runtime ended"))?;
        let Ok(admission) = runtime.control_admission.clone().try_lock_owned() else {
            return Ok(None);
        };
        ensure!(
            runtime.service_running()
                && !runtime.stopping.load(std::sync::atomic::Ordering::SeqCst),
            "Runtime stopped before success action"
        );
        Ok(Some(SuccessAdmission {
            _admission: admission,
            runtime,
        }))
    }
}

/// Non-Clone/non-Deserialize; private fields retain REAL component objects,
/// rather than a boolean readiness switch or a caller's SQL/capability witness.
pub(crate) struct InstalledDriverComposition {
    identity: Arc<CompositionIdentity>,
    runtime: Weak<Runtime>,
    owner: Arc<RuntimeOwner>,
    phases: Weak<PhaseSupervisor>,
    original_task: Task,
    sources: Arc<ManagedWorkflowSources>,
    gates: Arc<crate::execution::workflow_gates::ManagedWorkflowGates>,
    engine: Arc<WorkflowEngine>,
    selected: Arc<NativePhasePort>,
}
impl InstalledDriverComposition {
    pub(crate) fn activation_roster(&self, task: &Task) -> Result<ActivationRoster> {
        ensure!(
            self.is_current()
                && task.id == self.original_task.id
                && task.executor == self.original_task.executor
                && task.reviewers == self.original_task.reviewers,
            "activation composition Task differs"
        );
        let installation = self.selected.installation_id();
        ensure!(
            self.original_task.executor == self.selected.alias()
                && Arc::ptr_eq(
                    &self.engine.installed_native_port(&task.executor)?,
                    &self.selected
                ),
            "activation executor port differs"
        );
        ensure!(
            (1..=8).contains(&task.reviewers.len())
                && task
                    .reviewers
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    == task.reviewers.len(),
            "activation reviewer roster invalid"
        );
        let mut members = Vec::with_capacity(task.reviewers.len() + 1);
        for (role, alias) in std::iter::once((SessionRole::Executor, task.executor.as_str())).chain(
            task.reviewers
                .iter()
                .map(|alias| (SessionRole::Reviewer, alias.as_str())),
        ) {
            let port = self.engine.installed_native_port(alias)?;
            ensure!(
                std::ptr::eq(port.owner(), self.owner.as_ref())
                    && matches!(port.provider(), "claude" | "codex")
                    && port.installation_id() == installation
                    && port.selected_adapter()?.compatibility.is_some(),
                "activation installed roster port differs"
            );
            members.push(member_digest(role, &port)?);
        }
        members.sort();
        ActivationRoster::new(self, installation, members)
    }
    pub(crate) async fn admit_activation(
        &self,
        plan: &Arc<DriverPreparationAdvance>,
        lifetime: &WorkerLifetime,
    ) -> Result<ActivationAdmission> {
        self.admit_activation_inner(plan, lifetime, true).await
    }
    pub(crate) async fn admit_activation_recovery(
        &self,
        plan: &Arc<DriverPreparationAdvance>,
        lifetime: &WorkerLifetime,
    ) -> Result<ActivationAdmission> {
        self.admit_activation_inner(plan, lifetime, false).await
    }
    async fn admit_activation_inner(
        &self,
        plan: &Arc<DriverPreparationAdvance>,
        lifetime: &WorkerLifetime,
        require_retained: bool,
    ) -> Result<ActivationAdmission> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("activation Runtime ended"))?;
        let admission = tokio::select! { biased;
            ()=lifetime.cancelled()=>return Err(anyhow::anyhow!("activation Driver cancelled")),
            guard=runtime.control_admission.clone().lock_owned()=>guard,
        };
        ensure!(
            runtime.service_running() && self.is_current(),
            "Runtime stopped before activation"
        );
        let association = lifetime.association()?;
        ensure!(
            plan.activation_roster()?.is_same_composition(self)
                && self.original_task.id == association.task()
                && plan.belongs_to(&association),
            "activation original composition/worker linkage differs"
        );
        if require_retained {
            ensure!(plan.is_retained()?, "activation preparation lost custody");
        }
        Ok(ActivationAdmission {
            _admission: admission,
            _runtime: runtime,
        })
    }
    /// Driver worker path, like `admit_activation`: Weak upgrade, then a
    /// cancel-biased admission wait, then service/composition/association.
    pub(crate) async fn admit_success(
        &self,
        lifetime: &WorkerLifetime,
    ) -> Result<SuccessAdmission> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("success Runtime ended"))?;
        let admission = tokio::select! { biased;
            ()=lifetime.cancelled()=>{
                #[cfg(test)]
                crate::runtime::count(Some(self.original_task.id), "success admission cancelled");
                return Err(anyhow::anyhow!("success Driver cancelled"))
            }
            guard=runtime.control_admission.clone().lock_owned()=>guard,
        };
        ensure!(
            runtime.service_running() && self.is_current(),
            "Runtime stopped before success closure"
        );
        let association = lifetime.association()?;
        ensure!(
            self.original_task.id == association.task(),
            "success worker linkage differs"
        );
        Ok(SuccessAdmission {
            _admission: admission,
            runtime,
        })
    }
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
    /// The SAME installed gates the Engine holds, as their concrete type.
    pub(crate) fn gates(&self) -> &Arc<crate::execution::workflow_gates::ManagedWorkflowGates> {
        &self.gates
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
    gates: Arc<crate::execution::workflow_gates::ManagedWorkflowGates>,
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
        let gates = Arc::new(
            crate::execution::workflow_gates::ManagedWorkflowGates::new(
                owner.clone(),
                sources.clone(),
            )
            .map_err(|_| InstallationRefusal("managed Gates installation refused".into()))?,
        );
        let verifier =
            crate::execution::verification::ManagedVerifier::new(owner.clone(), sources.clone())
                .map_err(|_| InstallationRefusal("managed Verifier installation refused".into()))?;
        let engine = WorkflowEngine::new(
            owner.store(),
            registry.clone(),
            config.clone(),
            sources.clone(),
            gates.clone(),
        )
        .and_then(|engine| engine.with_verifier(Arc::new(verifier)))
        .map_err(|_| InstallationRefusal("managed Engine installation refused".into()))?;
        Ok(Self {
            registry,
            sources,
            gates,
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
            (1..=8).contains(&task.reviewers.len())
                && task
                    .reviewers
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    == task.reviewers.len(),
            "installed reviewer roster invalid"
        );
        for alias in &task.reviewers {
            let port = graph.registry.native_phase_port(alias)?;
            ensure!(
                std::ptr::eq(port.owner(), self.owner.as_ref())
                    && matches!(port.provider(), "claude" | "codex")
                    && port.installation_id() == selected.installation_id()
                    && port.selected_adapter()?.compatibility.is_some(),
                "installed reviewer declaration absent"
            );
        }
        ensure!(
            graph
                .engine
                .matches_composition(&self.owner, &graph.sources, &selected),
            "installed Engine graph differs"
        );
        Ok(InstalledDriverComposition {
            identity: Arc::new(CompositionIdentity { task: task.id }),
            runtime: Arc::downgrade(self),
            owner: self.owner.clone(),
            phases: Arc::downgrade(&self.phases),
            original_task: task.clone(),
            sources: graph.sources.clone(),
            gates: graph.gates.clone(),
            engine: graph.engine.clone(),
            selected,
        })
    }
}

#[cfg(test)]
mod tests;
