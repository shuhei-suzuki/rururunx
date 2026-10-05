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
        let prepared = attempts::AttemptManager::new(self.owner.clone())
            .prepare_workflow_source(task, provider)
            .await?;
        let unit = prepared.unit().clone();
        let (project, goal, task) = self.owners(task)?;
        let io = UnitGit::new(self.owner.clone(), &unit, true)?;
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
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .validate_execution(&unit.authority(), true, true)?;
        *state = Some(TaskSources {
            prepared: Some(prepared),
            frame: Arc::new(frame),
        });
        Ok(unit)
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
            result.verify(&artifact).await?;
            if state.frame.artifact != Some(artifact.id)
                || state.frame.revision != artifact.revision
            {
                let io = RetainedGit::new(self.owner.clone(), &artifact)?;
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
fn task_digest(task: &Task) -> Result<String> {
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
