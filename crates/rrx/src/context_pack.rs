//! Durable compact coordination artifacts. Native source observations stay in context.
pub mod workflow;

use crate::{
    adapter::{InputKind, PreparedInput, SharedStore},
    context::{
        Budget, GoalSourceSnapshot, RepositoryContext, RepositoryMap, SelectionOutcome,
        SelectionRequest,
    },
    domain::*,
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const MAX_BYTES: usize = 1024 * 1024;
const MAX_EVENTS: usize = 4096;
const MAX_TEXT: usize = 8192;
const MAX_REFS: usize = 128;
const FORMAT: &str = "rrx.task-pack.v1";
const CHECKPOINT: &str = "rrx.checkpoint.v1";
const GOAL_FORMAT: &str = "rrx.goal-pack.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRequest {
    pub kind: ArtifactKind,
    pub path: String,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Requirements,
    Design,
    Source,
    Impact,
    Verification,
    Finding,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRef {
    pub scope: Scope,
    pub kind: ArtifactKind,
    pub path: String,
    pub digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackRef {
    pub scope: Scope,
    pub version: u64,
    pub digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CheckpointRef {
    pub scope: Scope,
    pub id: RecordId,
    pub version: u64,
    pub digest: String,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Goal,
    Decision,
    Completed,
    Failure,
    Finding,
    NextAction,
    Constraint,
    CriticalReference,
    Verification,
    Transient,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HistoryEvent {
    pub sequence: u64,
    pub kind: EventKind,
    pub text: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RetainedEvent {
    pub session: SessionId,
    pub event: HistoryEvent,
}
#[derive(Debug, Clone, Copy)]
pub struct HistoryPolicy {
    /// Serialized event JSON byte estimate, never provider tokens. Zero retains no transient tail.
    pub recent_history_bytes: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub format: String,
    pub scope: Scope,
    pub session: SessionId,
    pub role: SessionRole,
    pub previous: Option<CheckpointRef>,
    pub chain_version: u64,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub input_digest: String,
    pub authority: RepositoryRef,
    pub mandatory_goal: Value,
    pub mandatory_task: Value,
    pub mandatory_rules: BTreeMap<String, String>,
    pub retained: Vec<RetainedEvent>,
    pub recent: Vec<RetainedEvent>,
    pub omitted_transient: u64,
    pub omitted_first_sequence: Option<u64>,
    pub omitted_last_sequence: Option<u64>,
    pub omitted_digest: Option<String>,
    pub recent_bytes: usize,
    pub measured_tokens: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RepositoryRef {
    pub worktree: std::path::PathBuf,
    pub source_root_file_id: String,
    pub worktree_file_id: String,
    pub revision: String,
    pub inventory_hash: String,
    pub manifest_digest: String,
    pub additional_paths: Vec<String>,
}
impl RepositoryRef {
    fn of(map: &RepositoryMap, additional_paths: Vec<String>) -> Result<Self> {
        let f = map.freshness();
        Ok(Self {
            worktree: f.worktree.clone(),
            source_root_file_id: f.source_root_file_id.clone(),
            worktree_file_id: f.worktree_file_id.clone(),
            revision: f.revision.clone(),
            inventory_hash: f.inventory_hash.clone(),
            manifest_digest: digest(&f.source_hashes)?,
            additional_paths,
        })
    }
}
#[derive(Debug, Clone, Default)]
pub struct TaskInputs {
    pub artifacts: Vec<ArtifactRequest>,
    pub additional_paths: Vec<String>,
    pub checkpoint: Option<CheckpointRef>,
    pub promoted_consultation: Option<CheckpointRef>,
    pub decisions: Vec<String>,
    pub completed_work: Vec<String>,
    pub failures: Vec<String>,
    pub verification: Vec<String>,
    pub unresolved_findings: Vec<String>,
    pub impact_summary: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceSymbolRef {
    pub name: String,
    pub line: usize,
    pub signature: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceMetadata {
    pub path: String,
    pub digest: String,
    pub changed: bool,
    pub symbols: Vec<SourceSymbolRef>,
    pub imports: Vec<String>,
    pub metadata_limited: bool,
}
fn source_metadata(map: &RepositoryMap, paths: &[String]) -> Result<Vec<SourceMetadata>> {
    paths
        .iter()
        .filter_map(|path| map.files().get(path).map(|f| (path, f)))
        .map(|(path, f)| {
            Ok(SourceMetadata {
                path: path.clone(),
                digest: map
                    .freshness()
                    .source_hashes
                    .get(&format!("worktree:{path}"))
                    .context("missing source metadata hash")?
                    .clone(),
                changed: f.changed,
                symbols: f
                    .symbols
                    .iter()
                    .take(8)
                    .map(|s| SourceSymbolRef {
                        name: s.name.clone(),
                        line: s.line,
                        signature: s.signature.clone(),
                    })
                    .collect(),
                imports: f.imports.iter().take(8).cloned().collect(),
                metadata_limited: f.lexical_limits_reached
                    || f.symbols.len() > 8
                    || f.imports.len() > 8,
            })
        })
        .collect()
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskPack {
    pub format: String,
    pub scope: Scope,
    pub repository_identity: String,
    pub task: Value,
    pub goal: Value,
    pub project_rules: BTreeMap<String, String>,
    pub authority_digest: String,
    pub repository: RepositoryRef,
    pub artifacts: Vec<ArtifactRef>,
    pub referenced_sources: Vec<SourceMetadata>,
    pub decisions: Vec<String>,
    pub completed_work: Vec<String>,
    pub failures: Vec<String>,
    pub verification: Vec<String>,
    pub unresolved_findings: Vec<String>,
    pub impact_summary: Option<String>,
    pub checkpoint: Option<CheckpointRef>,
    pub promoted_consultation: Option<CheckpointRef>,
    /// Historical facts preserve original Session/Task/revision provenance.
    pub historical_checkpoint: Option<Value>,
    pub historical_consultation: Option<Value>,
}
#[derive(Debug, Clone)]
pub struct TaskDraft {
    pack: TaskPack,
    map: RepositoryMap,
    versions: [u64; 3],
    instructions: BTreeMap<String, String>,
}
impl TaskDraft {
    pub fn pack(&self) -> &TaskPack {
        &self.pack
    }
    /// Phase-stable authority: launch bookkeeping changes payload, never hashes.
    pub fn source_versions(&self) -> BTreeMap<String, String> {
        let mut versions = self.map.freshness().source_hashes.clone();
        versions.extend(self.instructions.clone());
        versions.insert(
            "repository:inventory".into(),
            self.pack.repository.inventory_hash.clone(),
        );
        versions.insert(
            "checkpoint:head".into(),
            head_digest(self.pack.checkpoint.as_ref()),
        );
        versions
    }
    /// Engine owns the version/pointer transaction, never the source provider.
    pub fn context_version(&self, version: u64) -> Result<ContextVersion> {
        ensure!(version > 0, "pack version must be positive");
        Ok(ContextVersion {
            scope: self.pack.scope.clone(),
            version,
            revision: self.pack.repository.revision.clone(),
            source_hashes: self.map.freshness().source_hashes.clone(),
            data: serde_json::to_value(&self.pack)?,
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskDescriptor {
    pub id: TaskId,
    pub title: String,
    pub state: TaskState,
    pub phase: Option<String>,
    pub workflow: crate::config::WorkflowClass,
    pub risk: RiskClass,
    pub blockers: Vec<String>,
    pub next_action: Option<String>,
    pub context: Option<PackRef>,
    /// Opaque legacy references preserve exact envelopes without typed pack claims.
    pub typed_context: bool,
    /// Terminal Task provenance is historical and never launchable.
    pub historical: bool,
    /// Source freshness is checked by Task preparation; Goal summaries never launch refs.
    pub source_validation_required: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GoalRepositoryRef {
    pub root: std::path::PathBuf,
    pub root_file_id: String,
    pub revision: String,
    pub manifest_digest: String,
    pub additional_paths: Vec<String>,
}
impl GoalRepositoryRef {
    fn of(source: &GoalSourceSnapshot, additional_paths: Vec<String>) -> Result<Self> {
        Ok(Self {
            root: source.root.clone(),
            root_file_id: source.root_file_id.clone(),
            revision: source.revision.clone(),
            manifest_digest: digest(&source.source_hashes)?,
            additional_paths,
        })
    }
}
#[derive(Debug, Clone, Default)]
pub struct GoalInputs {
    pub artifacts: Vec<ArtifactRequest>,
    pub decisions: Vec<String>,
    pub metrics: BTreeMap<String, Option<u64>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalPack {
    pub format: String,
    pub scope: Scope,
    pub repository_identity: String,
    pub repository: GoalRepositoryRef,
    pub artifacts: Vec<ArtifactRef>,
    pub goal: Value,
    pub tasks: Vec<TaskDescriptor>,
    pub cross_task_decisions: Vec<String>,
    /// Goal/Task/DAG eligibility hints; scheduler admission remains authoritative.
    pub next_work_candidates: Vec<TaskId>,
    /// Nullable caller-supplied observations; no guessed provider measurement.
    pub metrics: BTreeMap<String, Option<u64>>,
    pub metrics_origin: MetricOrigin,
    pub authority_digest: String,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricOrigin {
    CallerSupplied,
}
#[derive(Debug)]
pub enum PreparedPack {
    Ready(PreparedInput),
    NeedsBudget {
        required_bytes: usize,
        budget: Budget,
    },
}
/// Private-authority source payload; never an Adapter PreparedInput or published version.
#[derive(Debug)]
pub struct DraftPayload {
    pub scope: Scope,
    pub revision: String,
    pub source_versions: BTreeMap<String, String>,
    pub payload: String,
}
#[derive(Debug)]
pub enum DraftPreparation {
    Ready(DraftPayload),
    NeedsBudget {
        required_bytes: usize,
        budget: Budget,
    },
}
#[derive(Clone)]
pub struct ContextPacks {
    store: SharedStore,
}
impl ContextPacks {
    pub fn new(store: SharedStore) -> Self {
        Self { store }
    }
    fn source(&self) -> RepositoryContext {
        RepositoryContext::new(self.store.clone())
    }
    fn snapshot(&self, scope: &Scope) -> Result<(Project, Goal, Task)> {
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
        let p = crate::project::registered_project(&store, scope.project_id)?;
        let g = store
            .goal(scope.goal_id.context("pack requires Goal")?)?
            .context("unknown Goal")?;
        let t = store
            .task(scope.task_id.context("pack requires bound Task")?)?
            .context("unknown Task")?;
        ensure!(
            g.project_id == p.id && t.scope() == *scope,
            "foreign pack scope"
        );
        Ok((p, g, t))
    }
    pub async fn draft_task(&self, scope: &Scope, mut inputs: TaskInputs) -> Result<TaskDraft> {
        validate_inputs(&inputs)?;
        let (p, g, t) = self.snapshot(scope)?;
        ensure!(
            !crate::state::task_terminal(t.state) && !crate::state::goal_terminal(g.state),
            "terminal Task/Goal is historical, never source preparation"
        );
        let mut additional = inputs.additional_paths.clone();
        additional.extend(inputs.artifacts.iter().map(|a| a.path.clone()));
        additional.sort();
        additional.dedup();
        ensure!(additional.len() <= MAX_REFS, "too many additional sources");
        let map = self.source().index(scope, additional.clone()).await?;
        ensure!(
            versions(&p, &g, &t) == map_versions(&map),
            "state changed before pack capture"
        );
        let artifacts = inputs
            .artifacts
            .iter()
            .map(|a| artifact(scope, a, &map))
            .collect::<Result<Vec<_>>>()?;
        // Own semantic history is mandatory even when the caller omits its ref.
        // Cross-Task consultation is an independently fixed historical snapshot.
        if inputs
            .checkpoint
            .as_ref()
            .is_some_and(|r| r.scope != *scope)
        {
            ensure!(
                inputs.promoted_consultation.is_none(),
                "duplicate consultation snapshot"
            );
            inputs.promoted_consultation = inputs.checkpoint.take();
        }
        let own = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?
            .pack_checkpoint_head(scope)?;
        if let Some(requested) = &inputs.checkpoint {
            ensure!(
                Some(requested) == own.as_ref(),
                "stale checkpoint head reference"
            );
        }
        inputs.checkpoint = own;
        ensure!(
            inputs
                .promoted_consultation
                .as_ref()
                .is_none_or(|r| r.scope != *scope),
            "consultation snapshot requires another Task"
        );
        let historical_consultation = inputs
            .promoted_consultation
            .as_ref()
            .map(|r| self.promotion(scope, r))
            .transpose()?;
        let historical_checkpoint = inputs
            .checkpoint
            .as_ref()
            .map(|r| self.promotion(scope, r))
            .transpose()?;
        let pack = TaskPack {
            format: FORMAT.into(),
            scope: scope.clone(),
            repository_identity: p.repository_identity.clone(),
            task: projection(&t)?,
            goal: projection(&g)?,
            project_rules: rules(&map),
            authority_digest: authority(&p, &g, &t)?,
            referenced_sources: source_metadata(&map, &additional)?,
            repository: RepositoryRef::of(&map, additional)?,
            artifacts,
            decisions: inputs.decisions,
            completed_work: inputs.completed_work,
            failures: inputs.failures,
            verification: inputs.verification,
            unresolved_findings: inputs.unresolved_findings,
            impact_summary: inputs.impact_summary,
            checkpoint: inputs.checkpoint,
            promoted_consultation: inputs.promoted_consultation,
            historical_checkpoint,
            historical_consultation,
        };
        bounded(&pack)?;
        let mut instructions = instruction_versions(&p, &g, &t)?;
        instructions.insert("instruction:pack_facts".into(), digest(&json!({"artifacts":pack.artifacts,"decisions":pack.decisions,"completed_work":pack.completed_work,"failures":pack.failures,"verification":pack.verification,"findings":pack.unresolved_findings,"impact":pack.impact_summary,"checkpoint":pack.checkpoint,"promoted_consultation":pack.promoted_consultation}))?);
        Ok(TaskDraft {
            pack,
            map,
            versions: versions(&p, &g, &t),
            instructions,
        })
    }
    pub async fn publish_task(&self, draft: &TaskDraft) -> Result<PackRef> {
        self.source().validate(&draft.map).await?;
        let mut store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
        let latest = store.context(&draft.pack.scope, None)?;
        let candidate = serde_json::to_value(&draft.pack)?;
        let version = match latest {
            Some(c)
                if c.data == candidate
                    && c.source_hashes == draft.map.freshness().source_hashes
                    && c.revision == draft.pack.repository.revision =>
            {
                c.version
            }
            Some(c) => c.version.checked_add(1).context("pack version overflow")?,
            None => 1,
        };
        let context = draft.context_version(version)?;
        store.publish_context_pack(&draft.pack.scope, draft.versions, &context, &[])?;
        reference(&context)
    }
    pub fn task_pack(&self, reference: &PackRef) -> Result<TaskPack> {
        let c = self.load_context(reference)?;
        let pack: TaskPack = if c.data.get("task_pack").is_some() {
            workflow::context_artifact(&c)?.pack
        } else {
            serde_json::from_value(c.data.clone())?
        };
        ensure!(
            pack.format == FORMAT
                && pack.scope == reference.scope
                && pack.repository.revision == c.revision,
            "invalid Task pack envelope"
        );
        bounded(&pack)?;
        Ok(pack)
    }
    fn load_context(&self, reference: &PackRef) -> Result<ContextVersion> {
        ensure!(reference.version > 0, "invalid context reference version");
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
        let c = store
            .context(&reference.scope, Some(reference.version))?
            .context("missing context reference")?;
        ensure!(
            digest(&c)? == reference.digest,
            "context reference digest changed"
        );
        Ok(c)
    }
    fn promotion(&self, scope: &Scope, reference: &CheckpointRef) -> Result<Value> {
        let cp = self.load_checkpoint(reference)?;
        ensure!(
            cp.scope.project_id == scope.project_id && cp.scope.goal_id == scope.goal_id,
            "foreign consultation/checkpoint promotion"
        );
        let cross = cp.scope != *scope;
        if !cross {
            self.ensure_checkpoint_head(reference)?;
            return Ok(own_history(&cp));
        }
        ensure!(
            !cross || cp.role == SessionRole::Consultant,
            "only consultation facts may promote across Tasks"
        );
        let retained = if cross {
            let store = self
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
            let mut retained = vec![];
            for event in &cp.retained {
                let source = store
                    .session(event.session)?
                    .context("missing historical Session")?
                    .0;
                if source.role == SessionRole::Consultant {
                    retained.push(event.clone());
                }
            }
            retained
        } else {
            cp.retained.clone()
        };
        Ok(
            json!({"scope":cp.scope,"session":cp.session,"revision":cp.authority.revision,"role":cp.role,
            "retained":retained,"mandatory_goal_at_checkpoint":cp.mandatory_goal,"mandatory_rules_at_checkpoint":cp.mandatory_rules,
            "mandatory_task_at_checkpoint":if cross {json!({"id":cp.mandatory_task["id"],"title":cp.mandatory_task["title"],"acceptance_criteria":cp.mandatory_task["acceptance_criteria"]})} else {cp.mandatory_task},"recent":if cross {vec![]} else {cp.recent}}),
        )
    }
    fn ensure_own_head(&self, scope: &Scope, reference: Option<&CheckpointRef>) -> Result<()> {
        ensure!(
            reference.is_none_or(|r| r.scope == *scope),
            "foreign own checkpoint head"
        );
        ensure!(
            self.store
                .lock()
                .map_err(|_| anyhow::anyhow!("Store poisoned"))?
                .pack_checkpoint_head(scope)?
                .as_ref()
                == reference,
            "stale checkpoint head reference"
        );
        Ok(())
    }
    fn ensure_checkpoint_head(&self, reference: &CheckpointRef) -> Result<()> {
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
        ensure!(
            store.pack_checkpoint_head(&reference.scope)?.as_ref() == Some(reference),
            "stale checkpoint head reference"
        );
        Ok(())
    }
    pub fn load_checkpoint(&self, reference: &CheckpointRef) -> Result<Checkpoint> {
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
        let r = store.record(reference.id)?.context("missing checkpoint")?;
        ensure!(
            r.kind == RecordKind::Checkpoint
                && r.scope == reference.scope
                && r.version == reference.version
                && r.version == 1
                && digest(&r.data)? == reference.digest,
            "stale/foreign checkpoint reference"
        );
        let cp: Checkpoint = serde_json::from_value(r.data)?;
        ensure!(
            cp.format == CHECKPOINT && cp.scope == r.scope,
            "invalid checkpoint envelope"
        );
        bounded(&cp)?;
        ensure!(
            cp.measured_tokens.is_none()
                && cp.chain_version > 0
                && cp.first_sequence > 0
                && cp.last_sequence >= cp.first_sequence
                && cp.retained.len() <= MAX_EVENTS
                && cp.recent.len() <= MAX_EVENTS,
            "invalid bounded checkpoint history"
        );
        ensure!(
            cp.retained
                .iter()
                .all(|e| e.event.kind != EventKind::Transient)
                && cp
                    .recent
                    .iter()
                    .all(|e| e.event.kind == EventKind::Transient)
                && cp
                    .retained
                    .iter()
                    .chain(&cp.recent)
                    .all(|e| e.event.sequence > 0
                        && e.event.sequence <= cp.last_sequence
                        && !e.event.text.trim().is_empty()
                        && e.event.text.len() <= MAX_TEXT),
            "checkpoint history classification/provenance invalid"
        );
        ensure!(
            (cp.omitted_transient == 0
                && cp.omitted_first_sequence.is_none()
                && cp.omitted_last_sequence.is_none()
                && cp.omitted_digest.is_none())
                || (cp.omitted_transient > 0
                    && cp.omitted_first_sequence.is_some_and(|s| s > 0)
                    && cp
                        .omitted_last_sequence
                        .is_some_and(|s| s <= cp.last_sequence)
                    && cp.omitted_first_sequence <= cp.omitted_last_sequence
                    && cp
                        .omitted_digest
                        .as_ref()
                        .is_some_and(|s| s.starts_with("sha256:") && s.len() == 71)),
            "invalid omitted transient commitment"
        );
        let native = store
            .session(cp.session)?
            .context("checkpoint Session missing")?
            .0;
        ensure!(
            native.scope == cp.scope
                && native.worktree == cp.authority.worktree
                && native.role == cp.role
                && matches!(native.role, SessionRole::Executor | SessionRole::Consultant),
            "checkpoint Session provenance mismatch"
        );
        let sessions: std::collections::BTreeSet<_> = cp
            .retained
            .iter()
            .chain(&cp.recent)
            .map(|e| e.session)
            .collect();
        for id in sessions {
            let source = store
                .session(id)?
                .context("retained source Session missing")?
                .0;
            ensure!(
                source.scope == cp.scope
                    && source.worktree == cp.authority.worktree
                    && matches!(source.role, SessionRole::Executor | SessionRole::Consultant),
                "foreign retained Session provenance"
            );
        }
        ensure!(
            cp.recent_bytes
                == cp
                    .recent
                    .iter()
                    .map(serde_json::to_vec)
                    .collect::<serde_json::Result<Vec<_>>>()?
                    .iter()
                    .map(Vec::len)
                    .sum::<usize>(),
            "checkpoint recent byte accounting mismatch"
        );
        Ok(cp)
    }
    async fn validate_task_map(&self, reference: &PackRef) -> Result<(TaskPack, RepositoryMap)> {
        ensure!(
            self.load_context(reference)?
                .data
                .get("task_pack")
                .is_none(),
            "workflow phase pack is prepared only by its Engine"
        );
        let pack = self.task_pack(reference)?;
        let (p, g, t) = self.snapshot(&reference.scope)?;
        ensure!(
            !crate::state::task_terminal(t.state) && !crate::state::goal_terminal(g.state),
            "terminal Task/Goal context is historical and never launchable"
        );
        ensure!(
            t.context_version == reference.version
                && self
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Store poisoned"))?
                    .context(&reference.scope, None)?
                    .is_some_and(|c| c.version == reference.version),
            "stale Task pack pointer"
        );
        ensure!(
            authority(&p, &g, &t)? == pack.authority_digest
                && projection(&t)? == pack.task
                && projection(&g)? == pack.goal
                && pack.repository_identity == p.repository_identity,
            "stale Task pack state"
        );
        let map = self
            .source()
            .index(&reference.scope, pack.repository.additional_paths.clone())
            .await?;
        let c = self.load_context(reference)?;
        ensure!(
            RepositoryRef::of(&map, pack.repository.additional_paths.clone())? == pack.repository
                && c.source_hashes == map.freshness().source_hashes,
            "stale Task pack sources"
        );
        ensure!(
            pack.project_rules == rules(&map),
            "Task pack rule metadata mismatch"
        );
        for a in &pack.artifacts {
            ensure!(
                a.scope == pack.scope
                    && map.files().contains_key(&a.path)
                    && map
                        .freshness()
                        .source_hashes
                        .get(&format!("worktree:{}", a.path))
                        == Some(&a.digest),
                "Task pack authoritative artifact mismatch"
            );
        }
        ensure!(
            source_metadata(&map, &pack.repository.additional_paths)? == pack.referenced_sources,
            "Task pack referenced source metadata mismatch"
        );
        self.ensure_own_head(&pack.scope, pack.checkpoint.as_ref())?;
        let consulted = pack
            .promoted_consultation
            .as_ref()
            .map(|r| self.promotion(&pack.scope, r))
            .transpose()?;
        ensure!(
            consulted == pack.historical_consultation,
            "Task consultation snapshot mismatch"
        );
        let historical = pack
            .checkpoint
            .as_ref()
            .map(|r| self.promotion(&pack.scope, r))
            .transpose()?;
        ensure!(
            historical == pack.historical_checkpoint,
            "Task pack historical checkpoint body/provenance mismatch"
        );
        ensure!(
            versions(&p, &g, &t) == map_versions(&map),
            "Task pack state changed during validation"
        );
        Ok((pack, map))
    }
    pub async fn validate_task(&self, reference: &PackRef) -> Result<()> {
        self.validate_task_map(reference).await.map(|_| ())
    }
    pub async fn prepare_task(
        &self,
        reference: &PackRef,
        request: SelectionRequest,
        budget: Budget,
    ) -> Result<PreparedPack> {
        let (pack, map) = self.validate_task_map(reference).await?;
        self.prepare(&pack, &map, Some(reference.version), request, budget)
            .await
    }
    /// Sources for an Engine-owned phase, without a fabricated native launch version.
    pub async fn prepare_draft(
        &self,
        draft: &TaskDraft,
        request: SelectionRequest,
        budget: Budget,
    ) -> Result<DraftPreparation> {
        self.source().validate(&draft.map).await?;
        self.ensure_own_head(&draft.pack.scope, draft.pack.checkpoint.as_ref())?;
        let consulted = draft
            .pack
            .promoted_consultation
            .as_ref()
            .map(|r| self.promotion(&draft.pack.scope, r))
            .transpose()?;
        ensure!(
            consulted == draft.pack.historical_consultation,
            "draft consultation snapshot mismatch"
        );
        let historical = draft
            .pack
            .checkpoint
            .as_ref()
            .map(|r| self.promotion(&draft.pack.scope, r))
            .transpose()?;
        ensure!(
            historical == draft.pack.historical_checkpoint,
            "stale draft checkpoint promotion"
        );
        let (_, g, t) = self.snapshot(&draft.pack.scope)?;
        ensure!(
            !crate::state::task_terminal(t.state) && !crate::state::goal_terminal(g.state),
            "terminal Task/Goal cannot prepare source payload"
        );
        match self
            .prepare(&draft.pack, &draft.map, None, request, budget)
            .await?
        {
            PreparedPack::Ready(input) => Ok(DraftPreparation::Ready(DraftPayload {
                scope: input.scope,
                revision: input.revision,
                source_versions: draft.source_versions(),
                payload: input.payload,
            })),
            PreparedPack::NeedsBudget {
                required_bytes,
                budget,
            } => Ok(DraftPreparation::NeedsBudget {
                required_bytes,
                budget,
            }),
        }
    }
    async fn prepare(
        &self,
        pack: &TaskPack,
        map: &RepositoryMap,
        version: Option<u64>,
        request: SelectionRequest,
        budget: Budget,
    ) -> Result<PreparedPack> {
        ensure!(
            budget.bytes > 0
                && budget.bytes <= 16 * MAX_BYTES
                && budget.estimated_tokens > 0
                && budget.estimated_tokens <= 16 * MAX_BYTES,
            "invalid bounded pack budget"
        );
        let header = format!(
            "{}\n",
            serde_json::to_string(
                &json!({"kind":"task_context_pack","scope":pack.scope,"version":version,"body":pack})
            )?
        );
        let available = budget.bytes.min(budget.estimated_tokens);
        let selection = self
            .source()
            .select(
                map,
                &request,
                Budget {
                    bytes: available.saturating_sub(header.len()).max(1),
                    estimated_tokens: available.saturating_sub(header.len()).max(1),
                },
            )
            .await?;
        match selection {
            SelectionOutcome::NeedsBudget { evidence } => {
                let required_bytes = header.len() + evidence.required_bytes;
                self.audit_preparation(
                    pack,
                    map,
                    version,
                    json!({"ready":false,"required_bytes":required_bytes,"budget":budget}),
                    None,
                )?;
                Ok(PreparedPack::NeedsBudget {
                    required_bytes,
                    budget,
                })
            }
            SelectionOutcome::Ready { slice } => {
                let mut input = slice
                    .prepared_input(&self.source(), InputKind::ContextPack, version.unwrap_or(1))
                    .await?;
                input.payload = format!("{header}{}", input.payload);
                input.source_versions.insert(
                    "checkpoint:head".into(),
                    head_digest(pack.checkpoint.as_ref()),
                );
                ensure!(
                    input.payload.len() <= available,
                    "rendered pack exceeds budget"
                );
                self.audit_preparation(pack,map,version,json!({"ready":true,"estimated_bytes":input.payload.len(),"estimated_tokens":input.payload.len(),"estimate_method":"utf8_bytes_v1","measured_tokens":null}), version.map(|_| &input))?;
                Ok(PreparedPack::Ready(input))
            }
        }
    }
    fn audit_preparation(
        &self,
        pack: &TaskPack,
        map: &RepositoryMap,
        version: Option<u64>,
        mut data: Value,
        prepared: Option<&PreparedInput>,
    ) -> Result<()> {
        data["context_version"] = json!(version);
        self.store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?
            .audit_pack_preparation(
                &map.freshness().scope,
                map_versions(map),
                pack.checkpoint.as_ref(),
                data,
                prepared,
            )?;
        Ok(())
    }
    pub async fn checkpoint(
        &self,
        scope: &Scope,
        session: SessionId,
        previous: Option<CheckpointRef>,
        events: Vec<HistoryEvent>,
        policy: HistoryPolicy,
    ) -> Result<CheckpointRef> {
        ensure!(
            events.len() <= MAX_EVENTS && !events.is_empty(),
            "checkpoint requires 1..4096 typed events"
        );
        ensure!(
            policy.recent_history_bytes <= MAX_BYTES,
            "recent history window exceeds 1 MiB"
        );
        let mut bytes = 0usize;
        for e in &events {
            ensure!(
                !e.text.trim().is_empty() && e.text.len() <= MAX_TEXT,
                "invalid event text"
            );
            bytes = bytes
                .checked_add(e.text.len())
                .context("event size overflow")?;
        }
        ensure!(bytes <= MAX_BYTES, "event batch exceeds 1 MiB");
        let (p, g, t) = self.snapshot(scope)?;
        let (native, session_version) = {
            let store = self
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
            store
                .session(session)?
                .context("unknown checkpoint Session")?
        };
        ensure!(
            native.scope == *scope
                && native.worktree == *t.worktree.as_ref().context("unbound Task")?
                && matches!(native.role, SessionRole::Executor | SessionRole::Consultant),
            "foreign/unsupported checkpoint Session"
        );
        let old = previous
            .as_ref()
            .map(|r| self.load_checkpoint(r))
            .transpose()?;
        if let Some(cp) = &old {
            ensure!(cp.scope == *scope, "foreign incremental checkpoint");
        }
        let last = old.as_ref().map_or(0, |c| c.last_sequence);
        for (i, e) in events.iter().enumerate() {
            ensure!(
                e.sequence
                    == last
                        .checked_add(i as u64 + 1)
                        .context("event sequence overflow")?,
                "checkpoint events must be consecutive delta"
            );
        }
        let mut retained = old.as_ref().map_or_else(Vec::new, |c| c.retained.clone());
        let mut recent = old.as_ref().map_or_else(Vec::new, |c| c.recent.clone());
        for e in &events {
            let row = RetainedEvent {
                session,
                event: e.clone(),
            };
            if e.kind == EventKind::Transient {
                recent.push(row);
            } else {
                retained.push(row);
            }
        }
        ensure!(
            retained.len() <= MAX_EVENTS,
            "mandatory retained events exceed limit; resolve explicitly"
        );
        let mut recent_bytes = recent
            .iter()
            .map(|e| serde_json::to_vec(e).map(|b| b.len()))
            .collect::<serde_json::Result<Vec<_>>>()?
            .into_iter()
            .sum::<usize>();
        let mut omitted = old.as_ref().map_or(0, |c| c.omitted_transient);
        let mut drop_count = 0usize;
        while (recent_bytes > policy.recent_history_bytes || recent.len() - drop_count > MAX_EVENTS)
            && drop_count < recent.len()
        {
            recent_bytes -= serde_json::to_vec(&recent[drop_count])?.len();
            drop_count += 1;
            omitted = omitted.checked_add(1).context("history count overflow")?;
        }
        let omitted_first_sequence =
            old.as_ref()
                .and_then(|c| c.omitted_first_sequence)
                .or_else(|| {
                    recent
                        .first()
                        .filter(|_| drop_count > 0)
                        .map(|e| e.event.sequence)
                });
        let omitted_last_sequence = recent
            .get(drop_count.wrapping_sub(1))
            .map(|e| e.event.sequence)
            .or_else(|| old.as_ref().and_then(|c| c.omitted_last_sequence));
        let omitted_digest = if drop_count > 0 {
            Some(digest(&(
                old.as_ref().and_then(|c| c.omitted_digest.clone()),
                &recent[..drop_count],
            ))?)
        } else {
            old.as_ref().and_then(|c| c.omitted_digest.clone())
        };
        recent.drain(..drop_count);
        let map = self.source().index(scope, vec![]).await?;
        ensure!(
            versions(&p, &g, &t) == map_versions(&map),
            "state changed before checkpoint"
        );
        let cp = Checkpoint {
            format: CHECKPOINT.into(),
            scope: scope.clone(),
            session,
            role: native.role,
            previous: previous.clone(),
            chain_version: old.as_ref().map_or(Ok(1), |c| {
                c.chain_version
                    .checked_add(1)
                    .context("checkpoint overflow")
            })?,
            first_sequence: events[0].sequence,
            last_sequence: events.last().unwrap().sequence,
            input_digest: digest(&events)?,
            authority: RepositoryRef::of(&map, vec![])?,
            mandatory_goal: projection(&g)?,
            mandatory_task: projection(&t)?,
            mandatory_rules: rules(&map),
            retained,
            recent,
            omitted_transient: omitted,
            omitted_first_sequence,
            omitted_last_sequence,
            omitted_digest,
            recent_bytes,
            measured_tokens: None,
        };
        bounded(&cp)?;
        let previous_context = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?
            .context(scope, None)?;
        let previous_pack = previous_context
            .as_ref()
            .filter(|c| c.data.get("task_pack").is_some() || c.data["format"] == FORMAT)
            .map(|c| self.task_pack(&reference(c)?))
            .transpose()?;
        workflow::validate_checkpoint_capacity(
            &map,
            &p,
            &g,
            &t,
            &cp,
            previous_pack,
            previous_context.as_ref(),
        )?;
        self.source().validate(&map).await?;
        let mut record = Record::new(
            scope.clone(),
            RecordKind::Checkpoint,
            serde_json::to_value(&cp)?,
        );
        {
            let mut store = self
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
            store.append_pack_checkpoint(
                scope,
                map_versions(&map),
                session,
                session_version,
                previous.as_ref(),
                &mut record,
            )?;
        }
        Ok(CheckpointRef {
            scope: scope.clone(),
            id: record.id,
            version: record.version,
            digest: digest(&record.data)?,
        })
    }
    fn goal_snapshot(&self, scope: &Scope) -> Result<(Project, Goal)> {
        ensure!(
            scope.goal_id.is_some() && scope.task_id.is_none(),
            "requires exact Goal scope"
        );
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
        let p = crate::project::registered_project(&store, scope.project_id)?;
        let g = store
            .goal(scope.goal_id.unwrap())?
            .context("unknown Goal")?;
        ensure!(g.scope() == *scope, "foreign Goal pack scope");
        Ok((p, g))
    }
    async fn validate_task_reference(&self, task: &Task, reference: &PackRef) -> Result<()> {
        ensure!(
            reference.scope == task.scope()
                && reference.version == task.context_version
                && self
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Store poisoned"))?
                    .context(&task.scope(), None)?
                    .is_some_and(|c| c.version == reference.version),
            "foreign/stale Task reference"
        );
        let context = self.load_context(reference)?;
        if context.data.get("task_pack").is_none() && context.data["format"] != FORMAT {
            ensure!(
                context.scope == task.scope() && !context.revision.is_empty(),
                "invalid opaque Task context provenance"
            );
            return Ok(());
        }
        let pack = self.task_pack(reference)?;
        // Immutable provenance only: a finalized worktree may already be disposed.
        ensure!(
            pack.scope == task.scope()
                && pack.task["id"] == serde_json::to_value(task.id)?
                && pack.task["project_id"] == serde_json::to_value(task.project_id)?
                && pack.task["goal_id"] == serde_json::to_value(task.goal_id)?
                && pack.repository.manifest_digest
                    == workflow::physical_manifest(&self.load_context(reference)?)?
                && !pack.repository.revision.is_empty(),
            "invalid historical Task provenance"
        );
        Ok(())
    }
    /// Compatibility selector accepts an owned Task, but observes the Goal's
    /// registered primary root without adopting or launching that Task.
    pub async fn publish_goal(
        &self,
        selector: &Scope,
        decisions: Vec<String>,
        metrics: BTreeMap<String, Option<u64>>,
    ) -> Result<PackRef> {
        if selector.task_id.is_some() {
            self.snapshot(selector)?;
        }
        let scope = Scope::goal(
            selector.project_id,
            selector.goal_id.context("requires Goal")?,
        );
        self.publish_goal_with_inputs(
            &scope,
            GoalInputs {
                decisions,
                metrics,
                ..Default::default()
            },
        )
        .await
    }
    pub async fn publish_goal_with_inputs(
        &self,
        scope: &Scope,
        inputs: GoalInputs,
    ) -> Result<PackRef> {
        text_list(&inputs.decisions)?;
        ensure!(
            inputs.artifacts.len() <= MAX_REFS
                && inputs.metrics.len() <= MAX_REFS
                && inputs
                    .metrics
                    .keys()
                    .all(|s| s.len() <= 128 && !s.trim().is_empty()),
            "invalid Goal pack metadata"
        );
        let (p, g) = self.goal_snapshot(scope)?;
        ensure!(
            g.dag.nodes.len() <= MAX_REFS && g.dag.edges.len() <= MAX_EVENTS,
            "Goal DAG exceeds pack limits"
        );
        let mut paths: Vec<String> = inputs.artifacts.iter().map(|a| a.path.clone()).collect();
        paths.sort();
        paths.dedup();
        let source = self
            .source()
            .goal_sources_with_files(scope, paths.clone())
            .await?;
        ensure!(
            [p.version, g.version] == [source.project_version, source.goal_version],
            "Goal changed during source capture"
        );
        let mut task_versions = vec![];
        let mut descriptors = vec![];
        let owned_tasks = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?
            .goal_pack_tasks(scope)?;
        for owned_task in owned_tasks {
            let id = owned_task.id;
            let (task, context) = {
                let store = self
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
                let task = store.task(id)?.context("missing owned Goal Task")?;
                let context = if task.context_version > 0 {
                    Some(
                        store
                            .context(&task.scope(), Some(task.context_version))?
                            .context("missing Task pack pointer")?,
                    )
                } else {
                    None
                };
                (task, context)
            };
            ensure!(
                task.project_id == p.id && task.goal_id == g.id,
                "foreign DAG Task"
            );
            let typed_context = context
                .as_ref()
                .is_some_and(|c| c.data.get("task_pack").is_some() || c.data["format"] == FORMAT);
            let context = context.map(|c| reference(&c)).transpose()?;
            if let Some(r) = &context {
                self.validate_task_reference(&task, r).await?;
            }
            task_versions.push((id, task.version));
            descriptors.push(TaskDescriptor {
                id,
                title: task.title,
                state: task.state,
                phase: task.phase,
                workflow: task.workflow,
                risk: task.risk,
                blockers: task.blockers,
                next_action: task.next_action,
                context,
                typed_context,
                historical: crate::state::task_terminal(task.state),
                source_validation_required: !crate::state::task_terminal(task.state),
            });
        }
        let next_work_candidates = goal_candidates(&g, &descriptors);
        let artifacts = inputs
            .artifacts
            .iter()
            .map(|a| {
                Ok(ArtifactRef {
                    scope: scope.clone(),
                    kind: a.kind,
                    path: a.path.clone(),
                    digest: source
                        .source_hashes
                        .get(&format!("project:{}", a.path))
                        .context("missing Goal artifact")?
                        .clone(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let authority_digest = digest(&(
            projection(&p)?,
            projection(&g)?,
            &descriptors,
            &inputs.decisions,
            &inputs.metrics,
            &artifacts,
        ))?;
        let pack = GoalPack {
            format: GOAL_FORMAT.into(),
            scope: scope.clone(),
            repository_identity: p.repository_identity,
            repository: GoalRepositoryRef::of(&source, paths.clone())?,
            artifacts,
            goal: projection(&g)?,
            tasks: descriptors,
            cross_task_decisions: inputs.decisions,
            next_work_candidates,
            metrics: inputs.metrics,
            metrics_origin: MetricOrigin::CallerSupplied,
            authority_digest,
        };
        bounded(&pack)?;
        let fresh = self.source().goal_sources_with_files(scope, paths).await?;
        ensure!(
            source.root_file_id == fresh.root_file_id
                && source.revision == fresh.revision
                && source.source_hashes == fresh.source_hashes
                && [source.project_version, source.goal_version]
                    == [fresh.project_version, fresh.goal_version],
            "Goal source changed before publication"
        );
        let mut store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
        let data = serde_json::to_value(&pack)?;
        let version = match store.context(scope, None)? {
            Some(c)
                if c.data == data
                    && c.revision == source.revision
                    && c.source_hashes == source.source_hashes =>
            {
                c.version
            }
            Some(c) => c.version.checked_add(1).context("context overflow")?,
            None => 1,
        };
        let context = ContextVersion {
            scope: scope.clone(),
            version,
            revision: source.revision,
            source_hashes: source.source_hashes,
            data,
        };
        store.publish_goal_context_pack(
            [source.project_version, source.goal_version],
            &context,
            &task_versions,
        )?;
        reference(&context)
    }
    pub fn goal_pack(&self, reference: &PackRef) -> Result<GoalPack> {
        let c = self.load_context(reference)?;
        let pack: GoalPack = serde_json::from_value(c.data)?;
        ensure!(
            pack.format == GOAL_FORMAT
                && pack.scope == reference.scope
                && pack.scope.task_id.is_none()
                && pack.repository.revision == c.revision,
            "invalid Goal pack envelope"
        );
        bounded(&pack)?;
        Ok(pack)
    }
    pub async fn validate_goal(&self, reference: &PackRef) -> Result<()> {
        let c = self.load_context(reference)?;
        let pack = self.goal_pack(reference)?;
        let (p, g) = self.goal_snapshot(&pack.scope)?;
        ensure!(
            g.context_version == reference.version
                && self
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Store poisoned"))?
                    .context(&reference.scope, None)?
                    .is_some_and(|c| c.version == reference.version)
                && projection(&g)? == pack.goal
                && p.repository_identity == pack.repository_identity,
            "stale/foreign Goal pack"
        );
        let owned_tasks = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?
            .goal_pack_tasks(&g.scope())?;
        ensure!(
            pack.tasks.iter().map(|t| t.id).collect::<Vec<_>>()
                == owned_tasks.iter().map(|t| t.id).collect::<Vec<_>>(),
            "stale Goal Task membership/order"
        );
        let mut task_versions = vec![];
        for d in &pack.tasks {
            let t = self
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("Store poisoned"))?
                .task(d.id)?
                .context("missing Task")?;
            ensure!(
                t.project_id == p.id
                    && t.goal_id == g.id
                    && t.title == d.title
                    && t.state == d.state
                    && t.phase == d.phase
                    && t.workflow == d.workflow
                    && t.risk == d.risk
                    && t.blockers == d.blockers
                    && t.next_action == d.next_action
                    && t.context_version == d.context.as_ref().map_or(0, |r| r.version)
                    && crate::state::task_terminal(t.state) == d.historical
                    && d.source_validation_required != d.historical,
                "stale Goal Task summary"
            );
            task_versions.push((t.id, t.version));
            let typed_context = if let Some(r) = &d.context {
                let context = self.load_context(r)?;
                context.data.get("task_pack").is_some() || context.data["format"] == FORMAT
            } else {
                false
            };
            ensure!(
                d.typed_context == typed_context,
                "Goal context typing differs from exact envelope"
            );
            if let Some(r) = &d.context {
                self.validate_task_reference(&t, r).await?;
            }
        }
        ensure!(
            goal_candidates(&g, &pack.tasks) == pack.next_work_candidates
                && digest(&(
                    projection(&p)?,
                    projection(&g)?,
                    &pack.tasks,
                    &pack.cross_task_decisions,
                    &pack.metrics,
                    &pack.artifacts
                ))? == pack.authority_digest,
            "stale Goal authority"
        );
        let source = self
            .source()
            .goal_sources_with_files(&pack.scope, pack.repository.additional_paths.clone())
            .await?;
        ensure!(
            GoalRepositoryRef::of(&source, pack.repository.additional_paths.clone())?
                == pack.repository
                && source.source_hashes == c.source_hashes
                && source.project_version == p.version
                && source.goal_version == g.version,
            "stale Goal repository sources"
        );
        for artifact in &pack.artifacts {
            ensure!(
                artifact.scope == pack.scope
                    && source
                        .source_hashes
                        .get(&format!("project:{}", artifact.path))
                        == Some(&artifact.digest),
                "invalid Goal artifact reference"
            );
        }
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
        ensure!(
            crate::project::registered_project(&store, p.id)?.version == p.version
                && store.goal(g.id)?.context("Goal disappeared")?.version == g.version,
            "Goal authority changed after source observation"
        );
        ensure!(
            store
                .goal_pack_tasks(&g.scope())?
                .iter()
                .map(|t| (t.id, t.version))
                .collect::<Vec<_>>()
                == task_versions,
            "Goal Task membership changed after source observation"
        );
        for (id, version) in task_versions {
            ensure!(
                store.task(id)?.context("Task disappeared")?.version == version,
                "Goal Task summary changed after source observation"
            );
        }
        Ok(())
    }
}
fn goal_candidates(g: &Goal, tasks: &[TaskDescriptor]) -> Vec<TaskId> {
    tasks
        .iter()
        .filter(|t| {
            g.state == GoalState::Running
                && g.dag.nodes.contains(&t.id)
                && t.state == TaskState::Created
                && t.blockers.is_empty()
                && g.dag
                    .edges
                    .iter()
                    .filter(|e| e.hard && e.dependent == t.id)
                    .all(|e| {
                        tasks.iter().any(|p| {
                            p.id == e.prerequisite
                                && matches!(p.state, TaskState::Completed | TaskState::Merged)
                        })
                    })
        })
        .map(|t| t.id)
        .collect()
}
fn own_history(cp: &Checkpoint) -> Value {
    json!({"scope":cp.scope,"session":cp.session,"revision":cp.authority.revision,"role":cp.role,"retained":cp.retained,"mandatory_goal_at_checkpoint":cp.mandatory_goal,"mandatory_rules_at_checkpoint":cp.mandatory_rules,"mandatory_task_at_checkpoint":cp.mandatory_task,"recent":cp.recent})
}
fn versions(p: &Project, g: &Goal, t: &Task) -> [u64; 3] {
    [p.version, g.version, t.version]
}
fn map_versions(map: &RepositoryMap) -> [u64; 3] {
    let f = map.freshness();
    [f.project_version, f.goal_version, f.task_version]
}
fn digest(value: &impl Serialize) -> Result<String> {
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(value)?)
    ))
}
fn bounded(value: &impl Serialize) -> Result<()> {
    ensure!(
        serde_json::to_vec(value)?.len() <= MAX_BYTES,
        "mandatory pack/checkpoint exceeds 1 MiB; narrow explicitly"
    );
    Ok(())
}
fn projection(value: &impl Serialize) -> Result<Value> {
    let mut v = serde_json::to_value(value)?;
    if let Some(obj) = v.as_object_mut() {
        for key in ["version", "context_version", "created_at", "updated_at"] {
            obj.remove(key);
        }
    }
    Ok(v)
}
fn authority(p: &Project, g: &Goal, t: &Task) -> Result<String> {
    digest(&(projection(p)?, projection(g)?, projection(t)?))
}
fn rules(map: &RepositoryMap) -> BTreeMap<String, String> {
    map.freshness()
        .source_hashes
        .iter()
        .filter(|(k, _)| k.starts_with("rule:"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}
fn artifact(scope: &Scope, request: &ArtifactRequest, map: &RepositoryMap) -> Result<ArtifactRef> {
    let key = format!("worktree:{}", request.path);
    let hash = map
        .freshness()
        .source_hashes
        .get(&key)
        .context("unindexed authoritative artifact")?;
    ensure!(
        hash.starts_with("sha256:") && map.files().contains_key(&request.path),
        "authoritative artifact missing or unsupported"
    );
    Ok(ArtifactRef {
        scope: scope.clone(),
        kind: request.kind,
        path: request.path.clone(),
        digest: hash.clone(),
    })
}
fn reference(c: &ContextVersion) -> Result<PackRef> {
    Ok(PackRef {
        scope: c.scope.clone(),
        version: c.version,
        digest: digest(c)?,
    })
}
fn text_list(list: &[String]) -> Result<()> {
    ensure!(
        list.len() <= MAX_EVENTS
            && list
                .iter()
                .all(|s| !s.trim().is_empty() && s.len() <= MAX_TEXT)
            && list.iter().map(String::len).sum::<usize>() <= MAX_BYTES,
        "invalid bounded fact list"
    );
    Ok(())
}
fn validate_inputs(inputs: &TaskInputs) -> Result<()> {
    ensure!(
        inputs.artifacts.len() <= MAX_REFS && inputs.additional_paths.len() <= MAX_REFS,
        "too many pack references"
    );
    ensure!(
        inputs.additional_paths.iter().all(|p| p.len() <= 4096)
            && inputs.artifacts.iter().all(|a| a.path.len() <= 4096),
        "pack path exceeds bounded metadata"
    );
    for list in [
        &inputs.decisions,
        &inputs.completed_work,
        &inputs.failures,
        &inputs.verification,
        &inputs.unresolved_findings,
    ] {
        text_list(list)?;
    }
    if let Some(s) = &inputs.impact_summary {
        text_list(std::slice::from_ref(s))?;
    }
    Ok(())
}

fn instruction_versions(p: &Project, g: &Goal, t: &Task) -> Result<BTreeMap<String, String>> {
    Ok(BTreeMap::from([
        (
            "repository:identity".into(),
            digest(&p.repository_identity)?,
        ),
        (
            "instruction:goal".into(),
            digest(&json!({"id":g.id,"objective":g.objective,
            "criteria":g.completion_criteria.iter().map(|c|json!({"id":c.id,"description":c.description})).collect::<Vec<_>>(),
            "constraints":g.constraints,"non_goals":g.non_goals,"source_refs":g.source_refs}))?,
        ),
        (
            "instruction:task".into(),
            digest(&json!({"id":t.id,"title":t.title,"criteria":t.acceptance_criteria}))?,
        ),
    ]))
}

pub(crate) fn head_digest(reference: Option<&CheckpointRef>) -> String {
    reference.map_or_else(|| "none".into(), |r| r.digest.clone())
}
