//! Committed Workflow inputs from registered preparation or retained artifacts.
//! This is result protection, not a process-ownership or security boundary.
use super::{attempts::PreparedExecutor, git_io::UnitGit, retained_io::RetainedGit, *};
use crate::{
    adapter::PreparedInput,
    config::{Config, WorkflowClass},
    context::{
        Budget, SelectionRequest,
        committed::{CommittedFile, CommittedIndex, CommittedOutcome},
    },
    domain::*,
    workflow::{
        Actor, CommittedWorkflowInput, ContextBudget, Phase, SourceSnapshot, WorkflowFuture,
        WorkflowSnapshot, WorkflowSources,
    },
};
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Component, Path},
    sync::{Arc, Mutex},
};

pub(crate) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
const PAYLOAD_BOUND: usize = 1024 * 1024;

/// Only this producer owns the live preparation capability. Ledger hints cannot
/// recreate it. Runtime must call prepare before Workflow initialization.
pub struct ManagedWorkflowSources {
    owner: Arc<RuntimeOwner>,
    runtime: Config,
    tasks: Mutex<BTreeMap<TaskId, Arc<tokio::sync::Mutex<Option<TaskSources>>>>>,
}
struct TaskSources {
    prepared: Option<PreparedExecutor>,
    frame: Arc<Frame>,
    recovery: Option<crate::state::SourceReadBinding>,
}
struct Frame {
    scope: Scope,
    revision: String,
    artifact: Option<ArtifactId>,
    index: CommittedIndex,
    config: Config,
    rules: String,
    versions: BTreeMap<String, String>,
    mandatory: BTreeMap<String, String>,
    governing_digest: String,
}
/// Actual first committed frame and its still-owned preparation. This token
/// retains the real Sources slot, never an input DTO/native permission.
pub(crate) struct InitialInputFrame {
    producer: Arc<ManagedWorkflowSources>,
    slot: Arc<tokio::sync::Mutex<Option<TaskSources>>>,
    frame: Arc<Frame>,
    unit: ExecutionUnit,
}
/// Produced only by the actual Sources-owned preparation after its registered
/// Git checks complete. Neither a namespace DTO nor a native launch permission.
pub(crate) struct InitialExecutorFrame {
    frame: InitialInputFrame,
}
impl InitialExecutorFrame {
    pub(crate) fn into_frame(self) -> InitialInputFrame {
        self.frame
    }
}
impl InitialInputFrame {
    pub(crate) fn unit(&self) -> &ExecutionUnit {
        &self.unit
    }
    pub(crate) fn governing(&self) -> &str {
        &self.frame.governing_digest
    }
    pub(crate) fn validate_context(
        &self,
        task: &Task,
        record: &Record,
        context: &ContextVersion,
    ) -> Result<()> {
        let workflow: WorkflowSnapshot = serde_json::from_value(record.data.clone())?;
        let phase: Phase = serde_json::from_value(context.data["phase"].clone())?;
        let budget = crate::workflow::budget(workflow.workflow, phase, &self.frame.config);
        let expected = self.frame.render(task, phase, &budget)?;
        let mut hashes = expected.source_versions.clone();
        hashes.insert("workflow:phase".into(), phase.key().into());
        hashes.insert(
            "workflow:generation".into(),
            workflow.generation.to_string(),
        );
        ensure!(
            context.scope == self.frame.scope
                && task.scope() == self.frame.scope
                && record.scope == self.frame.scope
                && context.revision == expected.revision
                && context.source_hashes == hashes
                && workflow.sources == expected
                && context.data
                    == serde_json::json!({"phase":phase,"workflow":workflow.workflow,
                "generation":workflow.generation,"budget":budget,"payload":expected.payload}),
            "prepared gate Context differs from actual immutable frame"
        );
        crate::workflow::validate_context(task, record, context)
    }

    pub(crate) fn validate(&self, owner: &Arc<RuntimeOwner>) -> Result<()> {
        self.validate_preparation(owner, None)
    }
    /// Only a saved adoption plan supplies its prescribed post Unit. This does
    /// not recapture a row or replace the original frame/preparation owner.
    pub(crate) fn validate_adoption(
        &self,
        owner: &Arc<RuntimeOwner>,
        adopted: &ExecutionUnit,
    ) -> Result<()> {
        self.validate_preparation(owner, Some(adopted))
    }
    fn validate_preparation(
        &self,
        owner: &Arc<RuntimeOwner>,
        adopted: Option<&ExecutionUnit>,
    ) -> Result<()> {
        ensure!(
            Arc::ptr_eq(owner, &self.producer.owner),
            "initial frame foreign Runtime"
        );
        let tasks = self
            .producer
            .tasks
            .lock()
            .map_err(|_| anyhow::anyhow!("Sources poisoned"))?;
        ensure!(
            tasks
                .get(&self.unit.scope.task_id.context("Task missing")?)
                .is_some_and(|s| Arc::ptr_eq(s, &self.slot)),
            "initial frame slot replaced"
        );
        let slot = self
            .slot
            .try_lock()
            .map_err(|_| anyhow::anyhow!("initial frame slot busy"))?;
        let state = slot.as_ref().context("initial frame removed")?;
        ensure!(
            Arc::ptr_eq(&state.frame, &self.frame)
                && state.recovery.is_none()
                && state
                    .prepared
                    .as_ref()
                    .is_some_and(|p| p.retains(owner, &self.unit).unwrap_or(false)
                        || adopted.is_some_and(|u| p.retains(owner, u).unwrap_or(false))),
            "initial frame preparation no longer owned"
        );
        Ok(())
    }
    pub(crate) fn publish_adoption(
        &self,
        adopted: &ExecutionUnit,
        plan: &Arc<crate::state::DriverPreparationAdvance>,
    ) -> Result<()> {
        self.validate_adoption(&self.producer.owner, adopted)?;
        let mut slot = self
            .slot
            .try_lock()
            .map_err(|_| anyhow::anyhow!("adoption Sources busy"))?;
        slot.as_mut()
            .context("adoption Sources removed")?
            .prepared
            .as_mut()
            .context("adoption preparation removed")?
            .publish_driven_adoption(&self.unit, adopted, plan)
    }
}
/// Only the complete corpus/rule/config producer below creates this proof.
pub(crate) struct ReconstructedFrame {
    frame: Arc<Frame>,
    digest: String,
}
impl ReconstructedFrame {
    pub(crate) fn scope(&self) -> &Scope {
        &self.frame.scope
    }
    pub(crate) fn revision(&self) -> &str {
        &self.frame.revision
    }
    pub(crate) fn artifact(&self) -> ArtifactId {
        self.frame
            .artifact
            .expect("reconstructed retained artifact")
    }
    pub(crate) fn versions(&self) -> &BTreeMap<String, String> {
        &self.frame.versions
    }
    pub(crate) fn governing(&self) -> &str {
        &self.frame.governing_digest
    }
    pub(crate) fn digest(&self) -> &str {
        &self.digest
    }
}
#[cfg(test)]
struct RecoveryPause {
    after_verify: bool,
    reached: tokio::sync::oneshot::Sender<()>,
    release: tokio::sync::oneshot::Receiver<()>,
}
struct RecoveryGuard {
    owner: Arc<RuntimeOwner>,
    binding: Option<crate::state::SourceReadBinding>,
}
impl Drop for RecoveryGuard {
    fn drop(&mut self) {
        if let Some(binding) = self.binding.take()
            && let Ok(mut store) = self.owner.store.lock()
        {
            // An error retains Preparing and requires explicit epoch fencing.
            // Dropping cannot install a cache or overwrite helper observations.
            let _ = store.abandon_retained_source_recovery(&binding);
        }
    }
}
/// Single-use private provenance for the actual first native phase.
pub struct InitialWorkflowExecutor {
    prepared: PreparedExecutor,
    expected: SourceSnapshot,
}
impl InitialWorkflowExecutor {
    pub(crate) async fn adopt(
        self,
        task_version: u64,
        reservation: &WorkflowReservation,
        phase: &str,
        provider: &str,
        input: &PreparedInput,
    ) -> Result<ExecutionUnit> {
        let versions = input
            .source_versions
            .iter()
            .filter(|(k, _)| !k.starts_with("workflow:"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<BTreeMap<_, _>>();
        let envelope: serde_json::Value = serde_json::from_str(&input.payload)?;
        ensure!(
            input.scope == self.expected.scope
                && input.revision == self.expected.revision
                && versions == self.expected.source_versions
                && envelope["payload"].as_str() == Some(self.expected.payload.as_str()),
            "first native input differs from prepared committed frame"
        );
        self.prepared
            .adopt(
                task_version,
                reservation,
                phase,
                provider,
                &versions,
                &input.payload,
            )
            .await
    }
}
impl ManagedWorkflowSources {
    pub(crate) fn belongs_to(&self, owner: &Arc<RuntimeOwner>) -> bool {
        Arc::ptr_eq(&self.owner, owner)
    }
    pub(crate) async fn verify_initial(
        &self,
        project: &Project,
        task: &Task,
        phase: Phase,
        budget: &ContextBudget,
        expected: &SourceSnapshot,
    ) -> Result<ExecutionUnit> {
        let frame = self.frame(project, task).await?;
        ensure!(
            matches!(phase, Phase::Issue | Phase::Worktree)
                && frame.artifact.is_none()
                && frame.render(task, phase, budget)? == *expected,
            "initial gate differs from prepared committed input"
        );
        let slot = self.slot(task.id)?;
        let state = slot.lock().await;
        let prepared = state
            .as_ref()
            .and_then(|s| s.prepared.as_ref())
            .context("live initial preparation capability unavailable")?;
        ensure!(
            prepared.unit().scope == task.scope() && prepared.unit().base_sha == frame.revision,
            "initial preparation identity changed"
        );
        prepared.verify_namespace().await?;
        Ok(prepared.unit().clone())
    }
    pub fn new(owner: Arc<RuntimeOwner>, runtime: Config) -> Result<Self> {
        runtime.validate()?;
        Ok(Self {
            owner,
            runtime,
            tasks: Mutex::new(BTreeMap::new()),
        })
    }
    fn slot(&self, task: TaskId) -> Result<Arc<tokio::sync::Mutex<Option<TaskSources>>>> {
        Ok(self
            .tasks
            .lock()
            .map_err(|_| anyhow::anyhow!("Workflow sources poisoned"))?
            .entry(task)
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(None)))
            .clone())
    }
    /// Register before Git, capture the exact prepared commit, and retain the
    /// abandonment guard through initial non-native Workflow phases.
    pub async fn prepare(&self, task: TaskId, provider: &str) -> Result<ExecutionUnit> {
        self.prepare_inner(task, provider, None).await
    }
    pub(crate) async fn prepare_driven(
        &self,
        task: TaskId,
        provider: &str,
        driver: &crate::runtime::driver::WorkerLifetime,
    ) -> Result<ExecutionUnit> {
        self.prepare_inner(task, provider, Some(driver)).await
    }
    /// Only the actual installed Sources/Engine lane calls this constructor.
    /// Render and complete metadata comparisons happen before SharedStore.
    pub(crate) async fn initial_input_frame(
        self: &Arc<Self>,
        original: &Task,
        record: &Record,
        context: &ContextVersion,
    ) -> Result<(InitialInputFrame, ExecutionUnit)> {
        let slot = self.slot(original.id)?;
        let state = slot.lock().await;
        let state = state.as_ref().context("first frame was not prepared")?;
        let prepared = state
            .prepared
            .as_ref()
            .context("first preparation missing")?;
        let unit = prepared.unit().clone();
        ensure!(
            state.recovery.is_none()
                && state.frame.artifact.is_none()
                && unit.scope == original.scope()
                && unit.phase == WORKFLOW_SOURCE_BOOTSTRAP
                && unit.state == UnitState::Preparing
                && unit.work.is_none()
                && unit.session_id.is_none()
                && unit.artifact_id.is_none()
                && state.frame.scope == original.scope()
                && state.frame.revision == unit.base_sha
                && prepared.retains(&self.owner, &unit)?,
            "initial input lacks actual preparation"
        );
        let workflow: WorkflowSnapshot = serde_json::from_value(record.data.clone())?;
        let phase = *workflow
            .configured_phases
            .first()
            .context("initial phase missing")?;
        let budget: ContextBudget = serde_json::from_value(context.data["budget"].clone())?;
        let expected = state.frame.render(original, phase, &budget)?;
        let mut hashes = expected.source_versions.clone();
        hashes.insert("workflow:phase".into(), phase.key().into());
        hashes.insert("workflow:generation".into(), "1".into());
        ensure!(
            record.kind == RecordKind::Workflow
                && record.scope == original.scope()
                && record.version == 0
                && context.scope == original.scope()
                && context.version == 1
                && context.revision == expected.revision
                && context.source_hashes == hashes
                && context.data
                    == serde_json::json!({"phase":phase,"workflow":workflow.workflow,
                "generation":1,"budget":budget,"payload":expected.payload})
                && serde_json::to_value(&workflow.sources)? == serde_json::to_value(&expected)?,
            "initial Context/frame payload or metadata differs"
        );
        let proof = InitialInputFrame {
            producer: self.clone(),
            slot: slot.clone(),
            frame: state.frame.clone(),
            unit: unit.clone(),
        };
        Ok((proof, unit))
    }
    /// Same real bootstrap preparation and immutable frame for initial gates.
    /// Unlike first initialization, this validates the actual phase/generation
    /// of an already-owned Workflow Context; it cannot adopt a native Unit.
    pub(crate) async fn initial_gate_frame(
        self: &Arc<Self>,
        original: &Task,
        record: &Record,
        context: &ContextVersion,
    ) -> Result<(InitialInputFrame, ExecutionUnit)> {
        let slot = self.slot(original.id)?;
        let state = slot.lock().await;
        let state = state.as_ref().context("initial gate frame missing")?;
        let prepared = state
            .prepared
            .as_ref()
            .context("initial preparation missing")?;
        let unit = prepared.unit().clone();
        ensure!(
            state.recovery.is_none()
                && state.frame.artifact.is_none()
                && unit.scope == original.scope()
                && unit.phase == WORKFLOW_SOURCE_BOOTSTRAP
                && unit.state == UnitState::Preparing
                && unit.work.is_none()
                && unit.session_id.is_none()
                && unit.artifact_id.is_none()
                && state.frame.scope == original.scope()
                && state.frame.revision == unit.base_sha
                && prepared.retains(&self.owner, &unit)?,
            "initial gate lacks owned preparation"
        );
        let workflow: WorkflowSnapshot = serde_json::from_value(record.data.clone())?;
        let phase: Phase = serde_json::from_value(context.data["phase"].clone())?;
        let budget: ContextBudget = serde_json::from_value(context.data["budget"].clone())?;
        let expected = state.frame.render(original, phase, &budget)?;
        let mut hashes = expected.source_versions.clone();
        hashes.insert("workflow:phase".into(), phase.key().into());
        hashes.insert(
            "workflow:generation".into(),
            workflow.generation.to_string(),
        );
        ensure!(
            record.kind == RecordKind::Workflow
                && record.scope == original.scope()
                && record.version > 0
                && workflow.generation == 1
                && context.scope == original.scope()
                && context.version > 0
                && context.revision == expected.revision
                && context.source_hashes == hashes
                && context.data
                    == serde_json::json!({"phase":phase,"workflow":workflow.workflow,
                "generation":workflow.generation,"budget":budget,"payload":expected.payload})
                && workflow.sources == expected,
            "initial gate Context/frame metadata or bytes differs"
        );
        Ok((
            InitialInputFrame {
                producer: self.clone(),
                slot: slot.clone(),
                frame: state.frame.clone(),
                unit: unit.clone(),
            },
            unit,
        ))
    }
    pub(crate) async fn first_executor_frame(
        self: &Arc<Self>,
        task: &Task,
        record: &Record,
        context: &ContextVersion,
        ticket: crate::state::DriverReadTicket,
    ) -> Result<(InitialExecutorFrame, crate::state::DriverReadTicket)> {
        let (frame, _) = self.initial_gate_frame(task, record, context).await?;
        frame.validate(&self.owner)?;
        let ticket = {
            // Keep the original object in its slot across helper waits/Drop.
            // No take_initial_executor or newly constructed guard is used.
            let state = frame.slot.lock().await;
            let state = state.as_ref().context("first Executor Sources removed")?;
            ensure!(
                Arc::ptr_eq(&state.frame, &frame.frame) && state.recovery.is_none(),
                "first Executor source frame replaced"
            );
            let prepared = state
                .prepared
                .as_ref()
                .context("first Executor preparation removed")?;
            ensure!(
                prepared.retains(&self.owner, &frame.unit)?,
                "first Executor guard changed"
            );
            prepared.verify_namespace_driven(ticket).await?
        };
        frame.validate(&self.owner)?;
        Ok((InitialExecutorFrame { frame }, ticket))
    }
    async fn prepare_inner(
        &self,
        task: TaskId,
        provider: &str,
        driver: Option<&crate::runtime::driver::WorkerLifetime>,
    ) -> Result<ExecutionUnit> {
        let slot = self.slot(task)?;
        let mut state = slot.lock().await;
        ensure!(state.is_none(), "Workflow source already prepared");
        {
            let store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let task = store.task(task)?.context("Task missing")?;
            ensure!(
                store
                    .records(&task.scope(), RecordKind::Workflow)?
                    .is_empty(),
                "existing Workflow requires explicit retained-input recovery"
            );
        }
        let manager = attempts::AttemptManager::new(self.owner.clone());
        let prepared = match driver {
            Some(driver) => {
                manager
                    .prepare_driver_source(task, provider, driver)
                    .await?
            }
            None => manager.prepare_workflow_source(task, provider).await?,
        };
        let unit = prepared.unit().clone();
        let (project, goal, task) = self.owners(task)?;
        let io = UnitGit::new(self.owner.clone(), &unit, true)?;
        let io = match driver {
            Some(driver) => io.with_driver_ticket(crate::state::read_driver_ticket(
                self.owner.clone(),
                driver.association()?,
            )?)?,
            None => io,
        };
        let files =
            read_corpus(CorpusReader::Prepared(&io), &unit.worktree, &unit.base_sha).await?;
        let frame = Frame::build(
            &project,
            &goal,
            &task,
            &self.runtime,
            unit.base_sha.clone(),
            None,
            files,
        )?;
        let configured = frame
            .config
            .agents
            .get(&task.executor)
            .context("Task executor is not configured")?;
        ensure!(
            configured.provider.as_deref() == Some(provider),
            "prepared provider differs from committed configuration"
        );
        // Bind the installed in-memory frame to the SAME ticket which read all
        // committed source bytes; CPU parsing cannot ratify later authority.
        io.validate_driver_current()?;
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .validate_execution(&unit.authority(), true, true)?;
        *state = Some(TaskSources {
            prepared: Some(prepared),
            frame: Arc::new(frame),
            recovery: None,
        });
        Ok(unit)
    }
    /// Rebuild only the sole Workflow's exact Published frame under a genuine
    /// current-owner source claim. This does not reconstruct a Runtime driver.
    // The Runtime driver is a subsequent consumer; this private source port is
    // exercised through real retained producers below, never a persisted grant.
    #[allow(dead_code)]
    pub(crate) async fn recover_retained(&self, task: TaskId) -> Result<()> {
        self.recover_retained_inner(
            task,
            #[cfg(test)]
            None,
        )
        .await
    }
    async fn recover_retained_inner(
        &self,
        task: TaskId,
        #[cfg(test)] mut pause: Option<RecoveryPause>,
    ) -> Result<()> {
        let slot = self.slot(task)?;
        let mut state = slot.lock().await;
        let claim = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .begin_retained_source_recovery(task, self.owner.epoch)?;
        let binding = claim.binding();
        let mut guard = RecoveryGuard {
            owner: self.owner.clone(),
            binding: Some(binding.clone()),
        };
        #[cfg(test)]
        if pause.as_ref().is_some_and(|p| !p.after_verify) {
            let p = pause.take().unwrap();
            let _ = p.reached.send(());
            let _ = p.release.await;
        }
        let result = results::ResultStore::new(self.owner.clone());
        result.verify_recovery(&claim.artifact, &binding).await?;
        let io = RetainedGit::for_recovery(self.owner.clone(), &claim.artifact, binding.clone())?;
        let files = read_corpus(
            CorpusReader::Retained(&io),
            &claim.artifact.repository,
            &claim.artifact.revision,
        )
        .await?;
        let frame = Frame::build(
            &claim.project,
            &claim.goal,
            &claim.task,
            &self.runtime,
            claim.artifact.revision.clone(),
            Some(claim.artifact.id),
            files,
        )?;
        ensure!(
            frame.versions == claim.artifact.dependencies,
            "reconstructed retained dependency frame changed"
        );
        result.verify_recovery(&claim.artifact, &binding).await?;
        let digest = digest(&serde_json::to_vec(
            &serde_json::json!({"scope":frame.scope,"revision":frame.revision,"artifact":frame.artifact,"versions":frame.versions,"governing":frame.governing_digest,"rules":frame.rules,"config":frame.config,"mandatory":frame.mandatory}),
        )?);
        let proof = ReconstructedFrame {
            frame: Arc::new(frame),
            digest,
        };
        #[cfg(test)]
        if let Some(p) = pause {
            let _ = p.reached.send(());
            let _ = p.release.await;
        }
        let installed = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .accept_retained_source_recovery(&claim, &proof)?;
        *state = Some(TaskSources {
            prepared: None,
            frame: proof.frame,
            recovery: Some(installed),
        });
        guard.binding = None;
        Ok(())
    }
    fn owners(&self, task: TaskId) -> Result<(Project, Goal, Task)> {
        let store = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        let task = store.task(task)?.context("Task missing")?;
        let project = store.project(task.project_id)?.context("Project missing")?;
        let goal = store.goal(task.goal_id)?.context("Goal missing")?;
        Ok((project, goal, task))
    }
    async fn frame(&self, project: &Project, task: &Task) -> Result<Arc<Frame>> {
        let slot = self.slot(task.id)?;
        let mut slot = slot.lock().await;
        let state = slot
            .as_mut()
            .context("committed Workflow source preparation required")?;
        ensure!(
            state.frame.scope == task.scope() && project.id == task.project_id,
            "foreign Workflow source"
        );
        if let Some(binding) = &state.recovery {
            let store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            store.validate_source_read(binding)?;
            ensure!(
                serde_json::to_value(store.project(project.id)?.context("Project missing")?)?
                    == serde_json::to_value(project)?
                    && serde_json::to_value(store.task(task.id)?.context("Task missing")?)?
                        == serde_json::to_value(task)?,
                "recovered source caller DTO changed"
            );
        }
        let (unit, artifact, goal) = {
            let store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let records = store.records(&task.scope(), RecordKind::Workflow)?;
            ensure!(records.len() <= 1, "ambiguous Workflow source");
            let workflow = records
                .first()
                .map(|r| serde_json::from_value::<WorkflowSnapshot>(r.data.clone()))
                .transpose()?;
            let attempt = workflow
                .as_ref()
                .and_then(|w| w.active.and_then(|i| w.history.get(i)));
            let unit = attempt
                .and_then(|a| a.unit.as_ref())
                .map(|id| {
                    let unit = store.execution_unit(id.unit)?;
                    ensure!(
                        id == &ManagedUnitRef::from(&unit),
                        "Workflow source unit changed"
                    );
                    Ok::<_, anyhow::Error>(unit)
                })
                .transpose()?;
            let artifacts = store.result_artifacts(&task.scope())?;
            let ready = unit.as_ref().and_then(|unit| {
                artifacts
                    .iter()
                    .find(|a| {
                        a.unit_id == unit.id
                            && matches!(a.state, ArtifactState::Ready | ArtifactState::Published)
                    })
                    .cloned()
            });
            let artifact = ready.or_else(|| {
                workflow
                    .as_ref()
                    .and_then(|w| w.sources.artifact)
                    .and_then(|id| artifacts.into_iter().find(|a| a.id == id))
            });
            (
                unit,
                artifact,
                store.goal(task.goal_id)?.context("Goal missing")?,
            )
        };
        ensure!(
            state.frame.governing_digest
                == crate::state::execution_governing_digest(project, &goal)?,
            "committed Project/Goal instructions changed; fresh input recovery required"
        );
        let result = results::ResultStore::new(self.owner.clone());
        if let Some(unit) = unit.filter(|u| {
            u.kind == UnitKind::Executor
                && u.work == Some(WorkOutcome::Success)
                && u.result_finalization_open
                && artifact.as_ref().is_none_or(|a| a.unit_id != u.id)
        }) {
            let io = UnitGit::new(self.owner.clone(), &unit, false)?;
            let revision = io.text(&unit.worktree, ["rev-parse", "HEAD"]).await?;
            ensure!(valid_oid(&revision), "result source requires exact commit");
            let files = read_corpus(CorpusReader::Prepared(&io), &unit.worktree, &revision).await?;
            let mut frame = Frame::build(
                project,
                &goal,
                task,
                &self.runtime,
                revision.clone(),
                None,
                files,
            )?;
            let artifact = result
                .capture(&unit.authority(), &revision, frame.versions.clone())
                .await?;
            result.verify(&artifact).await?;
            frame.artifact = Some(artifact.id);
            state.frame = Arc::new(frame);
        } else if let Some(artifact) = artifact {
            ensure!(artifact.scope == task.scope(), "foreign retained input");
            if let Some(binding) = state
                .recovery
                .as_ref()
                .filter(|b| b.owns_artifact(artifact.id))
            {
                result.verify_recovery(&artifact, binding).await?;
            } else {
                result.verify(&artifact).await?;
            }
            if state.frame.artifact != Some(artifact.id)
                || state.frame.revision != artifact.revision
            {
                let io = if let Some(binding) = state
                    .recovery
                    .as_ref()
                    .filter(|b| b.owns_artifact(artifact.id))
                {
                    RetainedGit::for_recovery(self.owner.clone(), &artifact, binding.clone())?
                } else {
                    RetainedGit::new(self.owner.clone(), &artifact)?
                };
                let files = read_corpus(
                    CorpusReader::Retained(&io),
                    &artifact.repository,
                    &artifact.revision,
                )
                .await?;
                let frame = Frame::build(
                    project,
                    &goal,
                    task,
                    &self.runtime,
                    artifact.revision.clone(),
                    Some(artifact.id),
                    files,
                )?;
                ensure!(
                    frame.versions == artifact.dependencies,
                    "retained dependency frame changed"
                );
                state.frame = Arc::new(frame);
            } else {
                ensure!(
                    state.frame.versions == artifact.dependencies,
                    "cached retained dependency frame changed"
                );
            }
        } else if let Some(prepared) = &state.prepared {
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .validate_execution(&prepared.unit().authority(), true, true)?;
        }
        Ok(state.frame.clone())
    }
}
#[cfg(test)]
mod recovery_tests;
impl WorkflowSources for ManagedWorkflowSources {
    fn capture(
        &self,
        project: Project,
        task: Task,
        phase: Phase,
        budget: ContextBudget,
    ) -> WorkflowFuture<'_, SourceSnapshot> {
        Box::pin(async move {
            self.frame(&project, &task)
                .await?
                .render(&task, phase, &budget)
        })
    }
    fn committed_input(
        &self,
        project: Project,
        task: Task,
        phase: Phase,
        class: WorkflowClass,
    ) -> WorkflowFuture<'_, Option<CommittedWorkflowInput>> {
        Box::pin(async move {
            let frame = self.frame(&project, &task).await?;
            let budget = crate::workflow::budget(class, phase, &frame.config);
            let source = frame.render(&task, phase, &budget)?;
            Ok(Some(CommittedWorkflowInput {
                config: frame.config.clone(),
                source,
                budget,
            }))
        })
    }
    fn take_initial_executor(
        &self,
        project: &Project,
        task: &Task,
        phase: Phase,
        budget: &ContextBudget,
    ) -> WorkflowFuture<'_, Option<InitialWorkflowExecutor>> {
        let project = project.clone();
        let task = task.clone();
        let budget = budget.clone();
        Box::pin(async move {
            ensure!(
                phase.actor() == Actor::Executor,
                "initial adoption requires Executor"
            );
            let frame = self.frame(&project, &task).await?;
            let expected = frame.render(&task, phase, &budget)?;
            let slot = self.slot(task.id)?;
            let mut state = slot.lock().await;
            let state = state.as_mut().context("committed source missing")?;
            if let Some(prepared) = state.prepared.take() {
                ensure!(
                    frame.artifact.is_none() && prepared.unit().base_sha == expected.revision,
                    "initial source changed before adoption"
                );
                return Ok(Some(InitialWorkflowExecutor { prepared, expected }));
            }
            Ok(None)
        })
    }
    fn retire_initial(&self, scope: &Scope) -> Result<()> {
        let task = scope.task_id.context("Task required")?;
        let actual = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .task(task)?
            .context("Task missing")?;
        ensure!(actual.scope() == *scope, "foreign source retirement");
        // Remove access immediately. A bounded in-flight source read retains its
        // guard until it finishes; terminal Task fencing already forbids launch.
        self.tasks
            .lock()
            .map_err(|_| anyhow::anyhow!("Workflow sources poisoned"))?
            .remove(&task);
        Ok(())
    }
}
impl Frame {
    fn build(
        project: &Project,
        goal: &Goal,
        task: &Task,
        runtime: &Config,
        revision: String,
        artifact: Option<ArtifactId>,
        files: Vec<CommittedFile>,
    ) -> Result<Self> {
        ensure!(
            task.project_id == project.id
                && task.goal_id == goal.id
                && goal.project_id == project.id,
            "source owners differ"
        );
        let mut config = runtime.clone();
        let mut rules = String::new();
        let mut versions = BTreeMap::from([("code".into(), revision.clone())]);
        let read = |reference: &Path| -> Result<(String, &[u8])> {
            let path = reference
                .strip_prefix(&project.root)
                .context("unsupported external committed rule/config origin")?
                .to_str()
                .context("non-UTF8 rule path")?;
            relative(path)?;
            let file = files
                .iter()
                .find(|f| f.path == path)
                .context("mandatory rule/config is not committed")?;
            Ok((
                path.into(),
                file.bytes
                    .as_deref()
                    .context("mandatory rule/config content skipped")?,
            ))
        };
        if let Some(reference) = &project.config_ref {
            let (_, bytes) = read(reference)?;
            let text = std::str::from_utf8(bytes)?;
            ensure!(!text.contains('\0'), "binary config");
            versions.insert("rules:config".into(), digest(bytes));
            config = config.with_project_text(text)?;
        }
        for reference in &project.rule_refs {
            let (path, bytes) = read(reference)?;
            let text = std::str::from_utf8(bytes)?;
            ensure!(!text.contains('\0'), "binary mandatory rule");
            versions.insert(format!("rules:{path}"), digest(bytes));
            rules.push_str(&format!("\nMandatory Project rule {path}:\n{text}\n"));
        }
        config.validate()?;
        // Registry/program identity is Runtime-frozen; tracked policy/model/effort
        // may change, but a different native executable requires a fresh registry.
        for (alias, agent) in &config.agents {
            let original = runtime
                .agents
                .get(alias)
                .context("committed agent requires a configured Runtime registry")?;
            ensure!(
                agent.provider == original.provider && agent.command == original.command,
                "committed native command/provider differs from Runtime registry"
            );
        }
        let goal_text = serde_json::to_string(&serde_json::json!({"objective":goal.objective,
            "completion_criteria":goal.completion_criteria,"constraints":goal.constraints,"non_goals":goal.non_goals,
            "source_refs":goal.source_refs,"dag":goal.dag,"blockers":goal.blockers}))?;
        let mandatory = BTreeMap::from([("goal".into(), goal_text)]);
        versions.insert(
            "instructions:goal".into(),
            digest(mandatory["goal"].as_bytes()),
        );
        versions.insert("instructions:task".into(), task_digest(task)?);
        let index = CommittedIndex::build(
            task.scope(),
            revision.clone(),
            BTreeMap::from([("code".into(), revision.clone())]),
            files,
        )?;
        versions.insert(
            "context:committed_inventory".into(),
            index.source_versions()["context:committed_inventory"].clone(),
        );
        ensure!(
            versions.len() <= 128
                && versions
                    .iter()
                    .all(|(k, v)| k.len() <= 128 && v.len() <= 256),
            "committed dependency frame exceeds retained-result bounds"
        );
        Ok(Self {
            scope: task.scope(),
            revision,
            artifact,
            index,
            config,
            rules,
            versions,
            mandatory,
            governing_digest: crate::state::execution_governing_digest(project, goal)?,
        })
    }
    fn render(&self, task: &Task, phase: Phase, budget: &ContextBudget) -> Result<SourceSnapshot> {
        ensure!(task.scope() == self.scope, "foreign committed frame");
        let mut mandatory = self.mandatory.clone();
        let task_text = serde_json::to_string(
            &serde_json::json!({"title":task.title,"acceptance_criteria":task.acceptance_criteria,
            "issue":task.issue,"executor":task.executor,"reviewers":task.reviewers,"phase":phase}),
        )?;
        mandatory.insert("task".into(), task_text.clone());
        let request = SelectionRequest {
            task_text: task.title.clone(),
            mandatory_evidence: if self.config.context.enabled {
                vec![]
            } else {
                self.index.files().keys().cloned().collect()
            },
            ..Default::default()
        };
        ensure!(
            self.rules.len() < PAYLOAD_BOUND,
            "mandatory rules exceed native payload bound"
        );
        let maximum = Budget {
            estimated_tokens: PAYLOAD_BOUND,
            bytes: PAYLOAD_BOUND,
        };
        let required = match self
            .index
            .select(&self.scope, &request, &mandatory, maximum)?
        {
            CommittedOutcome::Ready { slice } => slice.evidence().required_bytes,
            CommittedOutcome::NeedsBudget { .. } => {
                anyhow::bail!("mandatory committed context exceeds native payload bound")
            }
        };
        let bytes = required
            .checked_add(budget.discretionary_tokens)
            .context("context budget overflow")?
            .min(PAYLOAD_BOUND.saturating_sub(self.rules.len() + 1));
        let selected = match self.index.select(
            &self.scope,
            &request,
            &mandatory,
            Budget {
                estimated_tokens: bytes,
                bytes,
            },
        )? {
            CommittedOutcome::Ready { slice } => slice,
            CommittedOutcome::NeedsBudget { .. } => {
                anyhow::bail!("mandatory committed context needs a larger budget")
            }
        };
        let mut versions = self.versions.clone();
        // Task semantics remain an explicit dependency across every phase.
        versions.insert("instructions:task".into(), task_digest(task)?);
        let payload = format!("{}\n{}", self.rules, selected.payload());
        ensure!(
            payload.len() <= PAYLOAD_BOUND,
            "committed native payload exceeds bound"
        );
        Ok(SourceSnapshot {
            scope: self.scope.clone(),
            revision: self.revision.clone(),
            artifact: self.artifact,
            source_versions: versions,
            payload,
        })
    }
}
pub(crate) fn task_digest(task: &Task) -> Result<String> {
    Ok(digest(&serde_json::to_vec(
        &serde_json::json!({"title":task.title,"criteria":task.acceptance_criteria,
        "issue":task.issue,"executor":task.executor,"reviewers":task.reviewers}),
    )?))
}

fn relative(path: &str) -> Result<()> {
    ensure!(
        !path.is_empty()
            && path.len() <= 4096
            && !path.contains('\0')
            && Path::new(path)
                .components()
                .all(|c| matches!(c, Component::Normal(_))),
        "committed source path must be normal relative"
    );
    Ok(())
}
enum CorpusReader<'a> {
    Prepared(&'a UnitGit),
    Retained(&'a RetainedGit),
}
impl CorpusReader<'_> {
    async fn run<const N: usize>(&self, path: &Path, args: [&str; N]) -> Result<Vec<u8>> {
        match self {
            Self::Prepared(io) => io.run(path, args).await,
            Self::Retained(io) => io.run(args).await,
        }
    }
}
async fn read_corpus(
    io: CorpusReader<'_>,
    path: &Path,
    revision: &str,
) -> Result<Vec<CommittedFile>> {
    ensure!(
        valid_oid(revision),
        "committed inventory requires exact OID"
    );
    let tree = io
        .run(path, ["ls-tree", "-r", "-z", "-l", "--full-tree", revision])
        .await?;
    let mut files = Vec::new();
    let mut total = 0usize;
    for entry in tree.split(|b| *b == 0).filter(|e| !e.is_empty()) {
        ensure!(files.len() < 4096, "committed inventory exceeds 4096 files");
        let tab = entry
            .iter()
            .position(|b| *b == b'\t')
            .context("invalid committed tree entry")?;
        let fields = std::str::from_utf8(&entry[..tab])?
            .split_whitespace()
            .collect::<Vec<_>>();
        let name = std::str::from_utf8(&entry[tab + 1..])?.to_owned();
        relative(&name)?;
        ensure!(
            fields.len() == 4 && valid_oid(fields[2]),
            "invalid committed tree header"
        );
        let ordinary = matches!(fields[0], "100644" | "100755") && fields[1] == "blob";
        let size = if ordinary {
            fields[3].parse::<usize>()?
        } else {
            0
        };
        let (bytes, skipped) = if !ordinary {
            (None, Some("unsupported Git entry type".into()))
        } else if size > 256 * 1024 {
            (None, Some("file exceeds 256 KiB".into()))
        } else {
            total = total
                .checked_add(size)
                .context("committed corpus size overflow")?;
            ensure!(total <= 16 * 1024 * 1024, "committed corpus exceeds 16 MiB");
            let bytes = io.run(path, ["cat-file", "blob", fields[2]]).await?;
            ensure!(bytes.len() == size, "committed blob size mismatch");
            (Some(bytes), None)
        };
        files.push(CommittedFile {
            path: name,
            oid: fields[2].into(),
            bytes,
            skipped,
        });
    }
    Ok(files)
}

#[cfg(test)]
mod tests;
