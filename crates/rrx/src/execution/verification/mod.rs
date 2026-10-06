//! Command verification is a private producer, not a public Passed/Session label.
mod collector;
mod plan;
use super::*;
use crate::{
    domain::*,
    workflow::{Evidence, GateOutcome, Phase, PhaseInvocation},
};
use anyhow::{Context, Result, ensure};
pub(crate) use collector::CollectedCommand;
pub use collector::{CaptureIssue, CommandObservation, StreamEvidence};
pub(crate) use plan::{AdmittedProfile, hash, json_hash};
pub use plan::{Applicability, Category, TestCommand, TestsProfile};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, process::Stdio, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VerificationClaim {
    pub record: RecordId,
    pub version: u64,
    pub index: usize,
    pub scope: Scope,
    pub generation: u64,
    pub task_version: u64,
    pub task_digest: String,
    pub project_version: u64,
    pub project_digest: String,
    pub goal_version: u64,
    pub goal_digest: String,
    pub workflow_digest: String,
    pub workflow_updated_at: i64,
    pub attempt_detail: Option<String>,
    pub context_version: u64,
    pub context_digest: String,
    pub source_digest: String,
    pub observations: usize,
    pub artifact: ResultArtifact,
}
/// Only actual managed Workflow activation can request the corresponding SQL marker.
pub(crate) struct ManagedVerificationActivation {
    record: RecordId,
    scope: Scope,
    epoch: u64,
}
impl ManagedVerificationActivation {
    pub(crate) fn record(&self) -> RecordId {
        self.record
    }
    pub(crate) fn scope(&self) -> &Scope {
        &self.scope
    }
    pub(crate) fn epoch(&self) -> u64 {
        self.epoch
    }
    pub(crate) fn from_owner(owner: &RuntimeOwner, record: &Record) -> Result<Self> {
        ensure!(
            record.kind == RecordKind::Workflow && record.version == 0,
            "verification activation needs a fresh managed Workflow"
        );
        let store = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        let task = store
            .task(record.scope.task_id.context("activation Task missing")?)?
            .context("activation Task missing")?;
        ensure!(
            task.scope() == record.scope && !crate::state::task_terminal(task.state),
            "activation Scope/lifecycle changed"
        );
        Ok(Self {
            record: record.id,
            scope: record.scope.clone(),
            epoch: owner.epoch,
        })
    }
}
/// Durable DTO; inspection cannot turn it into a grant or accepted phase proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationRun {
    pub unit: UnitId,
    pub scope: Scope,
    pub workflow: RecordId,
    pub artifact: ArtifactId,
    pub revision: String,
    pub profile_digest: String,
    pub commands: Vec<CommandObservation>,
    pub work: WorkOutcome,
    pub certifying: bool,
    pub historical: bool,
}
pub(crate) struct VerificationGrant {
    claim: VerificationClaim,
    unit: ExecutionUnit,
    profile_digest: String,
}
impl VerificationGrant {
    pub(crate) fn admitted(
        claim: VerificationClaim,
        unit: ExecutionUnit,
        profile_digest: String,
    ) -> Self {
        Self {
            claim,
            unit,
            profile_digest,
        }
    }
    pub(crate) fn claim(&self) -> &VerificationClaim {
        &self.claim
    }
    pub(crate) fn unit(&self) -> &ExecutionUnit {
        &self.unit
    }
    pub(crate) fn profile_digest(&self) -> &str {
        &self.profile_digest
    }
}
/// Actual producer observation, minted off Store/SQL locks after snapshot
/// inspection. DTO paths never stand in for this canonical ancestry observation.
pub(crate) struct CommandCwdObservation {
    unit: UnitId,
    profile_digest: String,
    command: TestCommand,
    input_root: std::path::PathBuf,
    canonical: std::path::PathBuf,
}
impl CommandCwdObservation {
    fn observe(
        grant: &VerificationGrant,
        command: &TestCommand,
        input: &std::path::Path,
    ) -> Result<Self> {
        ensure!(
            input == grant.unit.worktree,
            "verification input differs from owned Unit"
        );
        let canonical = std::fs::canonicalize(input.join(&command.cwd))?;
        ensure!(
            canonical.starts_with(input) && canonical.is_dir(),
            "verification cwd escapes readonly input"
        );
        Ok(Self {
            unit: grant.unit.id,
            profile_digest: grant.profile_digest.clone(),
            command: command.clone(),
            input_root: input.into(),
            canonical,
        })
    }
    pub(crate) fn bind(
        &self,
        grant: &VerificationGrant,
        command: &TestCommand,
    ) -> Result<&std::path::Path> {
        ensure!(
            self.unit == grant.unit.id
                && self.profile_digest == grant.profile_digest
                && self.command == *command
                && self.input_root == grant.unit.worktree
                && self.canonical.starts_with(&self.input_root),
            "verification cwd observation differs from exact grant/plan"
        );
        Ok(&self.canonical)
    }
}
/// Fields stay private: only real collection and snapshot inspection mint this.
pub(crate) struct VerificationCompletion {
    grant: VerificationGrant,
    run: VerificationRun,
    run_digest: String,
}
impl VerificationCompletion {
    pub(crate) fn grant(&self) -> &VerificationGrant {
        &self.grant
    }
    pub(crate) fn run(&self) -> &VerificationRun {
        &self.run
    }
    pub(crate) fn digest(&self) -> &str {
        &self.run_digest
    }
}
pub(crate) struct ManagedVerificationResult {
    pub successor: Record,
    pub outcome: GateOutcome,
    pub completion: Option<VerificationCompletion>,
}
pub struct ManagedVerifier {
    owner: Arc<RuntimeOwner>,
    sources: Arc<workflow_source::ManagedWorkflowSources>,
    #[cfg(test)]
    before_commands: std::sync::Mutex<Option<VerificationHook>>,
}
#[cfg(test)]
type VerificationHook = Box<dyn FnOnce(UnitId) + Send>;
impl ManagedVerifier {
    pub fn new(
        owner: Arc<RuntimeOwner>,
        sources: Arc<workflow_source::ManagedWorkflowSources>,
    ) -> Result<Self> {
        ensure!(
            sources.belongs_to(&owner),
            "verifier/source Runtime mismatch"
        );
        Ok(Self {
            owner,
            sources,
            #[cfg(test)]
            before_commands: std::sync::Mutex::new(None),
        })
    }
    #[cfg(test)]
    pub(crate) fn before_commands(&self, hook: VerificationHook) {
        *self.before_commands.lock().unwrap() = Some(hook);
    }
    pub(crate) fn belongs_to(&self, owner: &Arc<RuntimeOwner>) -> bool {
        Arc::ptr_eq(&self.owner, owner)
    }
    /// Trusted operator integration explicitly admits this catalog before Workflow
    /// activation. Task text/config is never passed through this method implicitly.
    pub fn admit_tests(
        &self,
        project: ProjectId,
        expected_version: u64,
        profile: TestsProfile,
    ) -> Result<String> {
        let admitted = AdmittedProfile::admit(profile)?;
        let registered = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .project(project)?
            .context("verification Project missing")?;
        for c in &admitted.proposal.commands {
            ensure!(
                !c.program.starts_with(&self.owner.root)
                    && !c.program.starts_with(&registered.root)
                    && !c.program.starts_with(&registered.worktree_root),
                "verification executable must be outside managed writable namespaces"
            );
        }
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .admit_verification_profile(project, expected_version, self.owner.epoch, &admitted)
    }
    pub fn inspect_stream(
        &self,
        unit: UnitId,
        index: usize,
        stderr: bool,
        maximum: usize,
    ) -> Result<Vec<u8>> {
        let run = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .verification_run(unit)?;
        let observation = run
            .commands
            .get(index)
            .context("verification command missing")?;
        collector::inspect(
            &collector::evidence_root(&self.owner.root, unit, index),
            if stderr { "stderr" } else { "stdout" },
            if stderr {
                &observation.stderr
            } else {
                &observation.stdout
            },
            maximum,
        )
    }
    pub(crate) async fn evaluate(
        &self,
        invocation: PhaseInvocation,
    ) -> Result<ManagedVerificationResult> {
        use crate::workflow::WorkflowSources;
        ensure!(
            invocation.phase == Phase::Tests,
            "only actual Tests verifier is implemented"
        );
        // Claim inspection precedes source helper effects; source producer then
        // checks committed frame/governing rules, never a live rule fallback.
        let claim = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .verification_claim(&invocation)?;
        let admitted = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .verification_profile_for(claim.record)?;
        admitted.validate()?;
        for c in &admitted.proposal.commands {
            admitted.recheck(c)?;
        }
        let observed = self
            .sources
            .capture(
                invocation.project.clone(),
                invocation.task.clone(),
                invocation.phase,
                invocation.budget.clone(),
            )
            .await?;
        ensure!(
            serde_json::to_value(&observed)? == serde_json::to_value(&invocation.sources)?,
            "verification managed source differs from invocation"
        );
        let id = UnitId::new();
        let manager = resources::ResourceManager::new(self.owner.clone());
        let path = self
            .owner
            .root
            .join("units")
            .join(id.to_string())
            .join("source");
        let profile = manager.draft(id, &claim.scope, &path)?;
        let at = now_ms();
        let draft = ExecutionUnit {
            id,
            scope: claim.scope.clone(),
            kind: UnitKind::Verifier,
            generation: 0,
            owner_epoch: self.owner.epoch,
            version: 0,
            phase: Phase::Tests.key().into(),
            provider: "verifier".into(),
            state: UnitState::Reserved,
            native_effects_open: true,
            result_finalization_open: true,
            work: None,
            cleanup: CleanupOutcome::Unknown,
            disposition: Disposition::Active,
            worktree: path,
            branch: None,
            base_sha: claim.artifact.revision.clone(),
            profile_digest: profile.digest.clone(),
            cookie: uuid::Uuid::new_v4().to_string(),
            session_id: None,
            artifact_id: Some(claim.artifact.id),
            wait_reason: None,
            capacity_retry_at: None,
            created_at: at,
            updated_at: at,
        };
        let (grant, successor) = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .reserve_verification(&invocation, &claim, draft, &admitted)?;
        // From this point every return carries the authorized successor version.
        let result = self.run(&grant, &admitted, &manager, &profile).await;
        match result {
            Ok((run, snapshot)) => {
                if !run.certifying {
                    return Ok(ManagedVerificationResult {
                        successor,
                        outcome: if run.work == WorkOutcome::Failure {
                            GateOutcome::Failed(
                                "verification command failed; inspect protected receipts".into(),
                            )
                        } else {
                            GateOutcome::Waiting(
                                "verification evidence incomplete; fresh safe retry required"
                                    .into(),
                            )
                        },
                        completion: None,
                    });
                }
                let inspection = async {
                    snapshot.verify().await?;
                    results::ResultStore::new(self.owner.clone())
                        .verify(&claim.artifact)
                        .await?;
                    for (i, o) in run.commands.iter().enumerate() {
                        for (name, reference) in [("stdout", &o.stdout), ("stderr", &o.stderr)] {
                            let _ = collector::inspect(
                                &collector::evidence_root(&self.owner.root, id, i),
                                name,
                                reference,
                                plan::STREAM_BYTES,
                            )?;
                        }
                    }
                    let store = self
                        .owner
                        .store
                        .lock()
                        .map_err(|_| anyhow::anyhow!("state poisoned"))?;
                    store.validate_verification(&grant, false)?;
                    ensure!(
                        json_hash(&store.verification_run(id)?)? == json_hash(&run)?,
                        "verification receipt changed before proof"
                    );
                    drop(store);
                    Ok::<_, anyhow::Error>(())
                }
                .await;
                if inspection.is_err() {
                    let _ = self
                        .owner
                        .store
                        .lock()
                        .map_err(|_| anyhow::anyhow!("state poisoned"))?
                        .retire_verification(&grant);
                    return Ok(ManagedVerificationResult {
                        successor,
                        outcome: GateOutcome::Waiting(
                            "verification retained input/evidence changed; no acceptance proof"
                                .into(),
                        ),
                        completion: None,
                    });
                }
                let run_digest = json_hash(&run)?;
                let evidence = Evidence {
                    scope: claim.scope.clone(),
                    phase: Phase::Tests,
                    revision: run.revision.clone(),
                    source_versions: invocation.sources.source_versions.clone(),
                    artifacts: vec![format!("rrx-verification:{id}:{run_digest}")],
                    dependencies: invocation.sources.source_versions,
                    review_approved: None,
                    session_id: None,
                    context_version: claim.context_version,
                };
                Ok(ManagedVerificationResult {
                    successor,
                    outcome: GateOutcome::Passed(evidence),
                    completion: Some(VerificationCompletion {
                        grant,
                        run,
                        run_digest,
                    }),
                })
            }
            Err(error) => {
                // Fixture diagnostics distinguish real admission/source failures
                // from a rendezvous timeout; production retains protected intents
                // and its conservative stable Waiting response.
                #[cfg(test)]
                eprintln!("verification fixture operation failure: {error:#}");
                let _ = &error;
                let _ = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .retire_verification(&grant);
                Ok(ManagedVerificationResult{successor,outcome:GateOutcome::Waiting("verification operation incomplete; inspect durable intent and retry in a fresh namespace".into()),completion:None})
            }
        }
    }
    async fn run(
        &self,
        grant: &VerificationGrant,
        admitted: &AdmittedProfile,
        manager: &resources::ResourceManager,
        profile: &resources::ResourceProfile,
    ) -> Result<(VerificationRun, results::ResultSnapshot)> {
        let mut preparation = owner::PreparationGuard::new(self.owner.clone(), grant.unit());
        manager.reserve(grant.unit(), profile)?;
        manager.materialize(profile)?;
        let snapshot = results::ResultStore::new(self.owner.clone())
            .snapshot(grant.unit())
            .await?;
        preparation.disarm();
        #[cfg(test)]
        if let Some(hook) = self.before_commands.lock().unwrap().take() {
            hook(grant.unit.id);
        }
        let mut commands = Vec::new();
        let mut bytes = 0u64;
        for (index, c) in admitted.proposal.commands.iter().enumerate() {
            snapshot.verify().await?;
            admitted.recheck(c)?;
            let cwd_observation = CommandCwdObservation::observe(grant, c, snapshot.source())?;
            let cwd = cwd_observation.bind(grant, c)?;
            let operation = OperationId::new();
            let mut command = tokio::process::Command::new(&c.program);
            command
                .args(&c.args)
                .current_dir(cwd)
                .envs(profile.environment(&grant.unit.cookie, self.owner.ipc_path())?)
                .env_remove("RRX_GIT_GATE_TOKEN")
                .env("PYTHONDONTWRITEBYTECODE", "1")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            for variable in [
                "GIT_DIR",
                "GIT_COMMON_DIR",
                "GIT_WORK_TREE",
                "GIT_INDEX_FILE",
                "GIT_OBJECT_DIRECTORY",
                "GIT_ALTERNATE_OBJECT_DIRECTORIES",
                "GIT_NAMESPACE",
                "GIT_REPLACE_REF_BASE",
                "GIT_SHALLOW_FILE",
                "GIT_GRAFT_FILE",
            ] {
                command.env_remove(variable);
            }
            let child = {
                let mut store = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?;
                store.reserve_verification_command(grant, index, operation, &cwd_observation)?;
                match process::OwnedProcess::spawn(&mut command) {
                    Ok(child) => child,
                    Err(e) => {
                        store.reconcile_managed_effect(
                            operation,
                            1,
                            EffectState::Unknown,
                            BTreeMap::new(),
                        )?;
                        return Err(e);
                    }
                }
            };
            let mut guard = owner::HelperGuard::new(self.owner.clone(), operation);
            let capture = collector::collect(child, c, operation, || {
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .validate_verification(grant, true)
            })
            .await?;
            bytes = bytes
                .checked_add(capture.stdout.len() as u64)
                .and_then(|n| n.checked_add(capture.stderr.len() as u64))
                .context("verification output accounting overflow")?;
            ensure!(
                bytes <= plan::RUN_BYTES as u64,
                "verification run output exceeded bound"
            );
            let root = collector::evidence_root(&self.owner.root, grant.unit.id, index);
            collector::retain(&root, "stdout", &capture.stdout)?;
            collector::retain(&root, "stderr", &capture.stderr)?;
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .complete_verification_command(grant, index, &capture)?;
            guard.disarm();
            let pass = capture.observation().certifying();
            commands.push(capture.observation().clone());
            if !pass {
                break;
            }
        }
        snapshot.verify().await?;
        let run = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .finish_verification(grant, &commands, snapshot.manifest_sha256())?;
        Ok((run, snapshot))
    }
}
