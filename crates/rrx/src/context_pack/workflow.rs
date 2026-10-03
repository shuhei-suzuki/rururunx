//! Source-only provider for the Engine's coordinated phase publication.
use super::*;
use crate::workflow::{ContextBudget, Phase, SourceSnapshot, WorkflowFuture, WorkflowSources};
use std::{collections::VecDeque, sync::Mutex};
const PHASE_FORMAT: &str = "rrx.phase-pack.v1";
const CAPTURE_CACHE: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhasePackArtifact {
    pub format: String,
    pub phase: Phase,
    pub budget: ContextBudget,
    pub pack: TaskPack,
    /// Capture-time guards; historical artifacts do not claim current DB versions.
    pub authority_versions: [u64; 3],
    pub scope: Scope,
    pub revision: String,
    pub source_versions: BTreeMap<String, String>,
    pub payload_digest: String,
    pub estimated_bytes: usize,
    pub estimated_tokens: usize,
    pub mandatory_bytes: usize,
    pub optional_bytes: usize,
    pub estimate_method: String,
    pub measured_tokens: Option<u64>,
}
fn payload_digest(payload: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(payload.as_bytes()))
}
fn source_key(source: &SourceSnapshot) -> Result<String> {
    digest(source)
}
/// Per-provider scoped inputs and a bounded cache of exact captures. Engine alone
/// assigns ContextVersion and attempt identities; this service never publishes.
pub struct WorkflowPackSources {
    packs: ContextPacks,
    inputs: Mutex<BTreeMap<TaskId, (Scope, TaskInputs)>>,
    captured: Mutex<VecDeque<(String, Value)>>,
}
impl WorkflowPackSources {
    pub fn new(packs: ContextPacks) -> Self {
        Self {
            packs,
            inputs: Mutex::new(BTreeMap::new()),
            captured: Mutex::new(VecDeque::new()),
        }
    }
    pub fn set_inputs(&self, scope: &Scope, inputs: TaskInputs) -> Result<()> {
        self.packs.snapshot(scope)?;
        validate_inputs(&inputs)?;
        let mut configured = self
            .inputs
            .lock()
            .map_err(|_| anyhow::anyhow!("pack inputs poisoned"))?;
        ensure!(
            configured.len() < MAX_REFS || configured.contains_key(&scope.task_id.unwrap()),
            "provider scoped-input limit reached"
        );
        configured.insert(scope.task_id.unwrap(), (scope.clone(), inputs));
        Ok(())
    }
    async fn capture_owned(
        &self,
        project: Project,
        task: Task,
        phase: Phase,
        budget: ContextBudget,
    ) -> Result<SourceSnapshot> {
        ensure!(budget.discretionary_tokens > 0, "empty phase source budget");
        let (p, g, current) = self.packs.snapshot(&task.scope())?;
        ensure!(
            p.id == project.id
                && p.version == project.version
                && current.version == task.version
                && p.root == project.root
                && p.repository_identity == project.repository_identity
                && current.worktree == task.worktree
                && current.branch == task.branch
                && instruction_versions(&p, &g, &current)?
                    == instruction_versions(&project, &g, &task)?,
            "phase capture ownership/instructions changed"
        );
        let configured = self
            .inputs
            .lock()
            .map_err(|_| anyhow::anyhow!("pack inputs poisoned"))?
            .get(&task.id)
            .map(|(scope, inputs)| {
                ensure!(*scope == task.scope(), "foreign provider inputs");
                Ok(inputs.clone())
            })
            .transpose()?;
        let inputs = match configured {
            Some(inputs) => inputs,
            None => self.restored_inputs(&current)?,
        };
        let draft = self.packs.draft_task(&task.scope(), inputs).await?;
        let header = format!(
            "{}\n",
            serde_json::to_string(
                &json!({"kind":"phase_context_pack","phase":phase,"budget":budget,"body":draft.pack})
            )?
        );
        let request = SelectionRequest {
            task_text: format!("{} {}", task.title, task.acceptance_criteria.join(" ")),
            ..Default::default()
        };
        let available = budget.discretionary_tokens;
        let outcome = self
            .packs
            .source()
            .select_for_workflow(
                &draft.map,
                &request,
                Budget {
                    bytes: available,
                    estimated_tokens: available,
                },
            )
            .await?;
        let SelectionOutcome::Ready { slice } = outcome else {
            let SelectionOutcome::NeedsBudget { evidence } = outcome else {
                unreachable!()
            };
            anyhow::bail!(
                "phase pack NeedsBudget: {} required estimated bytes, budget {}",
                header.len() + evidence.required_bytes,
                budget.discretionary_tokens
            );
        };
        let payload = format!("{header}{}", slice.payload());
        let mandatory_bytes = header
            .len()
            .checked_add(slice.evidence().required_bytes)
            .context("mandatory phase byte count overflow")?;
        let optional_bytes = payload
            .len()
            .checked_sub(mandatory_bytes)
            .context("invalid mandatory phase byte count")?;
        ensure!(
            optional_bytes <= budget.discretionary_tokens,
            "optional phase source exceeds selected budget"
        );
        ensure_phase_payload(&payload)?;
        self.packs.audit_preparation(&draft.pack,&draft.map,None,json!({"ready":true,"phase":phase,"budget":budget,"estimated_bytes":payload.len(),"estimated_tokens":payload.len(),"mandatory_bytes":mandatory_bytes,"optional_bytes":optional_bytes,"estimate_method":"utf8_bytes_v1","measured_tokens":null,"rules_supplied_by_engine":true}))?;
        let source = SourceSnapshot {
            scope: task.scope(),
            revision: draft.pack.repository.revision.clone(),
            source_versions: draft.source_versions(),
            payload,
        };
        let artifact = PhasePackArtifact {
            format: PHASE_FORMAT.into(),
            phase,
            budget,
            pack: draft.pack,
            authority_versions: draft.versions,
            scope: source.scope.clone(),
            revision: source.revision.clone(),
            source_versions: source.source_versions.clone(),
            payload_digest: payload_digest(&source.payload),
            estimated_bytes: source.payload.len(),
            estimated_tokens: source.payload.len(),
            mandatory_bytes,
            optional_bytes,
            estimate_method: "utf8_bytes_v1".into(),
            measured_tokens: None,
        };
        bounded(&artifact)?;
        let value = serde_json::to_value(artifact)?;
        let key = source_key(&source)?;
        let mut cache = self
            .captured
            .lock()
            .map_err(|_| anyhow::anyhow!("capture cache poisoned"))?;
        cache.retain(|(existing, _)| existing != &key);
        cache.push_back((key, value));
        while cache.len() > CAPTURE_CACHE {
            cache.pop_front();
        }
        Ok(source)
    }
    // A restart must recover durable scoped facts/references, rather than treating
    // absence of an in-memory configuration as an instruction to discard them.
    fn restored_inputs(&self, task: &Task) -> Result<TaskInputs> {
        if task.context_version == 0 {
            return Ok(TaskInputs::default());
        }
        let context = {
            let store = self
                .packs
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
            let context = store
                .context(&task.scope(), None)?
                .context("missing current Task context")?;
            ensure!(
                context.version == task.context_version,
                "Task context pointer changed"
            );
            context
        };
        if context.data.get("task_pack").is_none() && context.data["format"] != FORMAT {
            return Ok(TaskInputs::default());
        }
        let pack = self.packs.task_pack(&reference(&context)?)?;
        Ok(TaskInputs {
            artifacts: pack
                .artifacts
                .into_iter()
                .map(|a| ArtifactRequest {
                    kind: a.kind,
                    path: a.path,
                })
                .collect(),
            additional_paths: pack.repository.additional_paths,
            promoted_consultation: pack.promoted_consultation,
            decisions: pack.decisions,
            completed_work: pack.completed_work,
            failures: pack.failures,
            verification: pack.verification,
            unresolved_findings: pack.unresolved_findings,
            impact_summary: pack.impact_summary,
            // Always resolve the current own checkpoint at capture, including new
            // facts appended since the immutable prior phase artifact.
            checkpoint: None,
        })
    }
}
pub(crate) fn ensure_phase_payload(payload: &str) -> Result<()> {
    ensure!(
        payload.len() <= MAX_BYTES,
        "rendered phase input exceeds absolute 1 MiB cap; mandatory facts retained, explicit blocked budget"
    );
    Ok(())
}
impl WorkflowSources for WorkflowPackSources {
    fn capture(
        &self,
        project: Project,
        task: Task,
        phase: Phase,
        budget: ContextBudget,
    ) -> WorkflowFuture<'_, SourceSnapshot> {
        Box::pin(self.capture_owned(project, task, phase, budget))
    }
    fn pack_artifact(&self, source: &SourceSnapshot) -> Result<Option<Value>> {
        let key = source_key(source)?;
        let cache = self
            .captured
            .lock()
            .map_err(|_| anyhow::anyhow!("capture cache poisoned"))?;
        let artifact = cache
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.clone())
            .context("exact captured phase pack unavailable; recapture")?;
        Ok(Some(artifact))
    }
}
pub(crate) fn validate_capture(
    value: &Value,
    source: &SourceSnapshot,
    phase: Phase,
    budget: &ContextBudget,
) -> Result<()> {
    let a: PhasePackArtifact = serde_json::from_value(value.clone())?;
    bounded(&a)?;
    ensure_phase_payload(&source.payload)?;
    ensure!(
        a.mandatory_bytes.checked_add(a.optional_bytes) == Some(source.payload.len())
            && a.optional_bytes <= budget.discretionary_tokens,
        "invalid mandatory/optional phase accounting"
    );
    ensure!(
        a.format == PHASE_FORMAT
            && a.phase == phase
            && &a.budget == budget
            && a.scope == source.scope
            && a.pack.scope == source.scope
            && a.revision == source.revision
            && a.pack.repository.revision == source.revision
            && a.source_versions == source.source_versions
            && a.payload_digest == payload_digest(&source.payload)
            && a.estimated_bytes == source.payload.len()
            && a.estimated_tokens == source.payload.len()
            && a.estimate_method == "utf8_bytes_v1"
            && a.measured_tokens.is_none()
            && a.source_versions.get("checkpoint:head")
                == Some(&head_digest(a.pack.checkpoint.as_ref())),
        "foreign/malformed phase pack capture"
    );
    Ok(())
}
/// Verify durable metadata without re-adopting or reading a finalized worktree.
pub(crate) fn context_artifact(context: &ContextVersion) -> Result<PhasePackArtifact> {
    let a: PhasePackArtifact = serde_json::from_value(context.data["task_pack"].clone())?;
    bounded(&a)?;
    let offset = context.data["source_payload_offset"]
        .as_u64()
        .context("phase source offset missing")? as usize;
    let payload = context.data["payload"]
        .as_str()
        .context("phase payload missing")?;
    ensure_phase_payload(payload)?;
    let source_payload = payload
        .get(offset..)
        .context("invalid phase source boundary")?;
    let phase: Phase = serde_json::from_value(context.data["phase"].clone())?;
    let budget: ContextBudget = serde_json::from_value(context.data["budget"].clone())?;
    let source = SourceSnapshot {
        scope: context.scope.clone(),
        revision: context.revision.clone(),
        source_versions: a.source_versions.clone(),
        payload: source_payload.into(),
    };
    validate_capture(&serde_json::to_value(&a)?, &source, phase, &budget)?;
    ensure!(
        a.source_versions
            .iter()
            .all(|(k, v)| context.source_hashes.get(k) == Some(v)),
        "phase source metadata differs from published authority"
    );
    Ok(a)
}
pub(crate) fn physical_manifest(context: &ContextVersion) -> Result<String> {
    if context.data.get("task_pack").is_some() {
        let a = context_artifact(context)?;
        let physical = a
            .source_versions
            .into_iter()
            .filter(|(k, _)| {
                !k.starts_with("instruction:")
                    && !k.starts_with("repository:")
                    && k != "checkpoint:head"
            })
            .collect::<BTreeMap<_, _>>();
        digest(&physical)
    } else {
        digest(&context.source_hashes)
    }
}
