//! Dedicated local controls. Public DTOs contain neither principal nor authority.
use super::{Runtime, goal::GoalPlan};
use crate::domain::{GoalId, GoalState, ProjectId, ProjectState, Scope, TaskId, TaskState};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf, time::Duration};
use tokio::net::UnixStream;
use uuid::Uuid;
mod read;
mod recorded;
pub use read::*;
pub use recorded::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalControl {
    Pause,
    Resume,
    Cancel,
    Fail,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum ControlAction {
    ResolveProject {
        selector: Option<String>,
        cwd: PathBuf,
    },
    CreateGoal {
        project: ProjectId,
        expected_project: u64,
        plan: GoalPlan,
    },
    ProposeGoal {
        project: ProjectId,
        expected_project: u64,
        objective: String,
    },
    GoalStatus {
        project: ProjectId,
        goal: GoalId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        view: Option<GoalReadView>,
    },
    GoalTasks {
        project: ProjectId,
        goal: GoalId,
        after: Option<TaskId>,
        maximum: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        view: Option<GoalReadView>,
    },
    SetGoalLifecycle {
        project: ProjectId,
        goal: GoalId,
        expected_goal: u64,
        target: GoalControl,
        reason: String,
    },
    TaskRetry {
        scope: Scope,
        expected_task: u64,
        reason: String,
    },
    TaskCancel {
        scope: Scope,
        expected_task: u64,
        reason: String,
    },
    RuntimeStatus,
    RuntimeStop,
    /// S3 D10: exact-root match over every row, Removed included.
    ProjectLookupRoot {
        root: PathBuf,
    },
    /// S3 D1: registers a new root; `path` is client-canonicalized.
    ProjectRegister {
        path: PathBuf,
        options: ProjectOptions,
    },
    /// S3 D1/D6: updates or reactivates a row at the observed version.
    ProjectUpdate {
        project: ProjectId,
        expected_project: u64,
        options: ProjectOptions,
    },
    /// S3 D6: soft removal at the observed version.
    ProjectRemove {
        project: ProjectId,
        expected_project: u64,
    },
    /// S3 D8: every row, each Registered one reconciled and bounded.
    ProjectList {
        all: bool,
    },
    /// S3 D8: one Project, resolved read-only and reconciled alone.
    ProjectStatus {
        selector: Option<String>,
        cwd: PathBuf,
    },
    /// S4 D2: one page of a Project's Goals, by stable Goal ID.
    ProjectGoals {
        project: ProjectId,
        after: Option<GoalId>,
        maximum: usize,
    },
    /// S4 D4: one page of a Task's Review/Approval metadata.
    TaskReview {
        scope: Scope,
        after: Option<ReviewCursor>,
        maximum: usize,
    },
    /// S4 D5: one page of a Task's own Sessions, by stable Session ID.
    TaskSessions {
        scope: Scope,
        after: Option<crate::domain::SessionId>,
        maximum: usize,
    },
    /// S4 D6: one page of the derived Goal+Task attention queue.
    AttentionQueue {
        project: Option<ProjectId>,
        after: Option<AttentionCursor>,
        maximum: usize,
    },
    /// S4 D7: one page of audit summaries, by audit sequence.
    Events {
        scope: Scope,
        after: Option<i64>,
        maximum: usize,
    },
    /// S4 D8: typed unavailable.
    Routing {
        scope: Scope,
    },
    /// S4 D8: typed unavailable.
    Metrics {
        scope: Scope,
    },
}
/// S3 D1: `project add` options. Source references (`config_ref`, rules,
/// `worktree_root`) keep their source-relative meaning and are sent as given.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectOptions {
    pub name: Option<String>,
    pub base_branch: Option<String>,
    pub config_ref: Option<PathBuf>,
    pub rule_refs: Vec<PathBuf>,
    pub environment_refs: Vec<String>,
    pub worktree_root: Option<PathBuf>,
    pub max_tasks: Option<usize>,
    pub clear_config: bool,
    pub clear_rules: bool,
    pub clear_environment: bool,
}
impl From<ProjectOptions> for crate::project::AddProject {
    fn from(o: ProjectOptions) -> Self {
        Self {
            name: o.name,
            base_branch: o.base_branch,
            config_ref: o.config_ref,
            rule_refs: o.rule_refs,
            environment_refs: o.environment_refs,
            worktree_root: o.worktree_root,
            max_tasks: o.max_tasks,
            clear_config: o.clear_config,
            clear_rules: o.clear_rules,
            clear_environment: o.clear_environment,
        }
    }
}
/// S3 D10: what an exact-root lookup found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectLookupFound {
    pub id: ProjectId,
    pub version: u64,
    pub state: ProjectState,
}
/// S3 D8: how one row's reconcile went in this read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconcileMark {
    /// Checked by this request's own preflight, and the reported row is the
    /// version that was checked (or the `Blocked` row this check wrote).
    Checked,
    /// Not checked at the reported version (Sol 6079932858 M2): the row
    /// changed, or appeared, after this request's check.
    Changed,
    /// Another request's preflight for this Project was in flight; not
    /// waited on, the current row is reported.
    SkippedInFlight,
    /// This request's own preflight did not finish within its deadline.
    Unavailable,
    /// Not Registered, so not reconciled.
    NotRegistered,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRow {
    pub project: crate::domain::Project,
    pub reconcile: ReconcileMark,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlRequest {
    pub request_id: Uuid,
    pub instance: String,
    pub epoch: u64,
    pub action: ControlAction,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case", deny_unknown_fields)]
pub enum ControlResponse {
    /// Sanitized refusal; rejected raw frames and internal errors are never echoed.
    Rejected {
        request_id: Option<Uuid>,
    },
    /// S3 D10.
    ProjectLookup {
        found: Option<ProjectLookupFound>,
    },
    /// S3 D1: the row after a register, update or remove.
    ProjectSaved {
        project: crate::domain::Project,
        changed: bool,
    },
    /// S3 D8.
    ProjectRows {
        rows: Vec<ProjectRow>,
    },
    /// S3 D1/D8: the non-secret status projection of one Project.
    ProjectStatusFacts {
        status: crate::project::ProjectStatusView,
        reconcile: ReconcileMark,
    },
    /// S3: a registry or Store validation refused the operation, with its
    /// bounded message (same-UID peer, its own paths). Nothing was written.
    ProjectRefused {
        reason: String,
    },
    /// Routing metadata only; actual source identity is checked before effects.
    ProjectResolved {
        project: ProjectId,
        version: u64,
        display_name: String,
        state: ProjectState,
    },
    GoalAccepted {
        goal: GoalId,
        version: u64,
        task_count: usize,
    },
    /// Persisted inert objective; no accepted definition, evaluator or Task exists.
    GoalProposed {
        goal: GoalId,
        version: u64,
        state: GoalState,
    },
    GoalProposalFacts {
        goal: GoalId,
        version: u64,
        state: GoalState,
        objective: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        recorded: Option<RecordedProposalStatus>,
        accepted: bool,
        task_count: usize,
        dispatch_available: bool,
        attention: UnavailableReason,
        /// Present only with `ProjectLimitUnsupported`: the stored limit.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        project_limit_stored: Option<usize>,
    },
    GoalFacts {
        goal: GoalId,
        version: u64,
        state: GoalState,
        task_count: usize,
        states: std::collections::BTreeMap<String, usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        recorded: Option<RecordedGoalStatus>,
        dispatch_available: bool,
        attention: UnavailableReason,
        /// Present only with `ProjectLimitUnsupported`: the stored limit.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        project_limit_stored: Option<usize>,
    },
    GoalLifecycleChanged {
        goal: GoalId,
        version: u64,
        state: GoalState,
    },
    GoalTaskPage {
        goal: GoalId,
        version: u64,
        tasks: Vec<TaskFacts>,
        next: Option<TaskId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        recorded: Option<RecordedGoalTaskPage>,
    },
    /// S4 D2.
    ProjectGoalPage {
        project: ProjectId,
        project_version: u64,
        goals: Vec<GoalView>,
        next: Option<GoalId>,
    },
    /// S4 D4: metadata only; `decision` stays unqualified.
    TaskReviewPage {
        task: Scope,
        task_version: u64,
        items: Vec<ReviewRecordView>,
        next: Option<ReviewCursor>,
        decision: ReviewDecisionView,
    },
    /// S4 D5.
    TaskSessionPage {
        task: Scope,
        task_version: u64,
        sessions: Vec<crate::project::SessionView>,
        next: Option<crate::domain::SessionId>,
    },
    /// S4 D6: advisory, never authority.
    AttentionPage {
        project: Option<ProjectId>,
        items: Vec<AttentionItem>,
        next: Option<AttentionCursor>,
    },
    /// S4 D7.
    EventPage {
        scope: Scope,
        events: Vec<EventView>,
        next: Option<i64>,
    },
    /// Control-service facts only; native Driver admission remains unavailable.
    RuntimeMetadata {
        instance: String,
        epoch: u64,
        operational: bool,
        service_running: bool,
        requested_global_sessions: usize,
        requested_tasks_per_project: usize,
    },
    Unavailable {
        request_id: Uuid,
        reason: UnavailableReason,
    },
    RuntimeStopped {
        instance: String,
        epoch: u64,
    },
    /// Shutdown is held by owned work at `site`; the service is not stopped.
    RuntimeStopPending {
        instance: String,
        epoch: u64,
        site: super::stop::ShutdownSite,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskFacts {
    pub scope: Scope,
    pub version: u64,
    pub state: TaskState,
    pub phase: Option<String>,
    /// Read-only Workflow wait derived from durable rows; never a grant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_wait: Option<WorkflowWait>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowWaitKind {
    EvidenceIntegrationUnavailable,
    GateFailed,
    GateUnknown,
    NextPhaseUnavailable,
    Held,
}
/// One fixed kind and one of the fixed detail constants (<=128 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowWait {
    pub kind: WorkflowWaitKind,
    pub detail: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableReason {
    TaskDriverUnavailable,
    PlanningUnavailable,
    NativeBindingUnavailable,
    NativeContinuationUnavailable,
    FreshBootstrapRecoveryUnavailable,
    /// The Project stores a Task limit other than 1 (R4.5); see
    /// `project_limit_stored`. Repaired by `project add --max-tasks 1`.
    ProjectLimitUnsupported,
    /// S3 D5: the bounded Git/filesystem preflight did not finish; nothing
    /// was written and the work is not claimed to have ended.
    ProjectPreflightUnavailable,
    /// S3 D5: another request's preflight for this Project has not joined.
    ProjectPreflightInFlight,
    /// S3 D6: the row, version, owner epoch or Runtime state changed since
    /// the observed version; nothing was written.
    ProjectCurrencyChanged,
    /// S3 D3: a `path`, `cwd` or `root` that is not absolute.
    ProjectPathNotAbsolute,
    /// S4 D2–D7: a cursor not present in the exact selected scope/view.
    ReadCursorInvalid,
    /// S4 D3: a non-elidable projected item cannot fit one response.
    ReadProjectionTooLarge,
    /// S4 D4: stored Review/Approval rows are not a qualified decision.
    ReviewDecisionUnqualified,
    /// S4 D8: stored Usage is not a qualified metric.
    MetricsUnqualified,
    /// S4 D8: no qualified routing projection exists.
    RoutingUnavailable,
    /// S4 D6: the Goal lifecycle resume predicate (accepted policy,
    /// registered Project, Runtime not stopping) does not hold now.
    GoalResumeHeld,
}

// Bounds the response wait; a timed-out filesystem worker is not claimed stopped.
// It performs no native executable or Store writes and cannot publish routing later.
async fn canonical_directory(path: PathBuf) -> Result<PathBuf> {
    let result = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::task::spawn_blocking(move || {
            let path = path.canonicalize()?;
            ensure!(path.is_dir(), "routing CWD must be a directory");
            Ok::<_, anyhow::Error>(path)
        }),
    )
    .await???;
    Ok(result)
}

/// Constructed solely from the actual accepted socket, never Deserialize/Clone/public constructor.
pub(crate) struct HumanIngress {
    uid: u32,
    instance: String,
    epoch: u64,
}
impl HumanIngress {
    pub(crate) fn identity(&self) -> (&str, u64, u32) {
        (&self.instance, self.epoch, self.uid)
    }
    fn from_connection(runtime: &Runtime, stream: &UnixStream) -> Result<Self> {
        let uid = stream.peer_cred()?.uid();
        ensure!(
            uid == rustix::process::getuid().as_raw(),
            "foreign control peer UID"
        );
        Ok(Self {
            uid,
            instance: runtime.owner.instance_id().into(),
            epoch: runtime.owner.epoch(),
        })
    }
    fn check(&self, request: &ControlRequest) -> Result<()> {
        ensure!(
            self.uid == rustix::process::getuid().as_raw()
                && self.instance == request.instance
                && self.epoch == request.epoch,
            "control instance/epoch changed"
        );
        ensure!(!request.request_id.is_nil(), "control request ID missing");
        Ok(())
    }
}
impl Runtime {
    /// Read-only Project routing by selector or CWD: the same route checks
    /// `ResolveProject` answers with, shared by S3's `ProjectStatus` (D8).
    pub(super) async fn resolve_project_route(
        &self,
        selector: Option<&str>,
        cwd: &std::path::Path,
    ) -> Result<crate::domain::Project> {
        ensure!(cwd.as_os_str().len() <= 4096, "routing CWD exceeds bound");
        let cwd = canonical_directory(cwd.to_path_buf()).await?;
        let snapshot = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .runtime_project_routes(selector, &cwd, self.owner.instance_id(), self.owner.epoch())?;
        let mut matches = BTreeSet::new();
        for project in &snapshot.projects {
            if selector.is_some() {
                matches.insert(project.id);
                continue;
            }
            if project.state == ProjectState::Removed {
                continue;
            }
            // Canonicalize physical routing boundaries only off the Store lock.
            if cwd.starts_with(&project.root)
                && !cwd.starts_with(&project.worktree_root)
                && !cwd.starts_with(project.root.join(".git"))
            {
                let root = canonical_directory(project.root.clone()).await?;
                ensure!(root == project.root, "registered Project root moved");
                matches.insert(project.id);
            }
            for task in snapshot.tasks.iter().filter(|t| t.project_id == project.id) {
                let path = task
                    .worktree
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("registered Task worktree missing"))?;
                if cwd.starts_with(path) {
                    let actual = canonical_directory(path.clone()).await?;
                    ensure!(actual == *path, "registered Task worktree moved");
                    matches.insert(project.id);
                }
            }
        }
        ensure!(
            matches.len() == 1,
            "unknown or ambiguous registered Project routing"
        );
        let selected = snapshot
            .projects
            .iter()
            .find(|p| matches.contains(&p.id))
            .ok_or_else(|| anyhow::anyhow!("Project routing missing"))?;
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .recheck_runtime_project_routes(
                selector,
                &cwd,
                self.owner.instance_id(),
                self.owner.epoch(),
                &snapshot,
            )?;
        Ok(selected.clone())
    }
    /// The accepted-policy digest a Goal is accepted and resumed under.
    fn policy_digest(&self) -> Result<String> {
        use sha2::{Digest, Sha256};
        let policy = serde_json::to_vec(&self.config)?;
        ensure!(policy.len() <= 1024 * 1024, "Runtime policy exceeds bound");
        Ok(format!("{:x}", Sha256::digest(&policy)))
    }
    /// S4 (D1–D9): the read-only surface; `None` for every other action.
    /// Each reader holds only the short Store lock and performs no Git,
    /// process or native I/O.
    fn handle_read(
        &self,
        ingress: &HumanIngress,
        request: &ControlRequest,
    ) -> Result<Option<ControlResponse>> {
        let store = || {
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))
        };
        let answer = match &request.action {
            ControlAction::ProjectGoals {
                project,
                after,
                maximum,
            } => store()?.runtime_project_goals(ingress, *project, *after, *maximum)?,
            ControlAction::TaskReview {
                scope,
                after,
                maximum,
            } => store()?.runtime_task_review(ingress, scope, *after, *maximum)?,
            ControlAction::TaskSessions {
                scope,
                after,
                maximum,
            } => store()?.runtime_task_sessions(ingress, scope, *after, *maximum)?,
            ControlAction::AttentionQueue {
                project,
                after,
                maximum,
            } => {
                let policy = self.policy_digest()?;
                let resume = crate::state::ResumeInputs {
                    policy_sha256: &policy,
                    stopping: self.stopping.load(std::sync::atomic::Ordering::SeqCst),
                };
                store()?.runtime_attention_queue(ingress, *project, *after, *maximum, &resume)?
            }
            ControlAction::Events {
                scope,
                after,
                maximum,
            } => store()?.runtime_events(ingress, scope, *after, *maximum)?,
            ControlAction::Routing { scope } => store()?.runtime_unqualified_read(
                ingress,
                scope,
                UnavailableReason::RoutingUnavailable,
            )?,
            ControlAction::Metrics { scope } => store()?.runtime_unqualified_read(
                ingress,
                scope,
                UnavailableReason::MetricsUnqualified,
            )?,
            _ => return Ok(None),
        };
        Ok(Some(answer.unwrap_or_else(|reason| {
            ControlResponse::Unavailable {
                request_id: request.request_id,
                reason,
            }
        })))
    }
    /// Trusted controls publish only their specific typed decisions. A public
    /// request never supplies Driver, native input or result authority.
    pub(crate) async fn handle_control(
        &self,
        accepted: &UnixStream,
        request: ControlRequest,
    ) -> Result<ControlResponse> {
        let ingress = HumanIngress::from_connection(self, accepted)?;
        ingress.check(&request)?;
        if let ControlAction::ResolveProject { selector, cwd } = &request.action {
            let selected = self.resolve_project_route(selector.as_deref(), cwd).await?;
            return Ok(ControlResponse::ProjectResolved {
                project: selected.id,
                version: selected.version,
                display_name: selected.name.clone(),
                state: selected.state,
            });
        }
        if let Some(response) = self.handle_project(&request).await? {
            return Ok(response);
        }
        if let ControlAction::CreateGoal { plan, .. } = &request.action {
            ensure!(
                !self.stopping.load(std::sync::atomic::Ordering::SeqCst),
                "Runtime stopping; new Goal refused"
            );
            let validated = plan.clone().validate(&self.config)?;
            let digest = self.policy_digest()?;
            // Same admission boundary as shutdown, retained through actual publication.
            let _admission = self.control_admission.lock().await;
            ensure!(
                !self.stopping.load(std::sync::atomic::Ordering::SeqCst),
                "Runtime stopping; new Goal refused"
            );
            #[cfg(test)]
            {
                let pause = self.goal_admission_pause.lock().unwrap().take();
                if let Some(pause) = pause {
                    let _ = pause.reached.send(());
                    pause
                        .release
                        .await
                        .map_err(|_| anyhow::anyhow!("test admission observer closed"))?;
                }
            }
            let response = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .accept_runtime_goal(&ingress, &request, &validated, &digest)?;
            self.wake.notify_one();
            return Ok(response);
        }
        if matches!(&request.action, ControlAction::ProposeGoal { .. }) {
            let _admission = self.control_admission.lock().await;
            ensure!(
                !self.stopping.load(std::sync::atomic::Ordering::SeqCst),
                "Runtime stopping; new proposal refused"
            );
            return self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .propose_runtime_goal(&ingress, &request);
        }
        if let ControlAction::GoalStatus {
            project,
            goal,
            view,
        } = &request.action
        {
            return self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .runtime_goal_facts(&ingress, *project, *goal, *view);
        }
        if let ControlAction::GoalTasks {
            project,
            goal,
            after,
            maximum,
            view,
        } = &request.action
        {
            return self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .runtime_goal_task_page(&ingress, *project, *goal, *after, *maximum, *view);
        }
        if let Some(response) = self.handle_read(&ingress, &request)? {
            return Ok(response);
        }
        if matches!(&request.action, ControlAction::SetGoalLifecycle { .. }) {
            let _admission = self.control_admission.lock().await;
            if matches!(
                &request.action,
                ControlAction::SetGoalLifecycle {
                    target: GoalControl::Resume,
                    ..
                }
            ) {
                ensure!(
                    !self.stopping.load(std::sync::atomic::Ordering::SeqCst),
                    "Runtime stopping; Goal resume refused"
                );
            }
            let policy_digest = self.policy_digest()?;
            let response = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .set_runtime_goal_lifecycle(&ingress, &request, &policy_digest)?;
            self.wake.notify_one();
            return Ok(response);
        }
        if matches!(&request.action, ControlAction::RuntimeStop) {
            // Held owned work is a typed pending answer; every other error,
            // poison included, stays a sanitized refusal. Neither is cleanup.
            return match self.shutdown().await {
                Ok(()) => Ok(ControlResponse::RuntimeStopped {
                    instance: self.owner.instance_id().into(),
                    epoch: self.owner.epoch(),
                }),
                Err(error) => match super::stop::pending_site(&error) {
                    Some(site) => Ok(ControlResponse::RuntimeStopPending {
                        instance: self.owner.instance_id().into(),
                        epoch: self.owner.epoch(),
                        site,
                    }),
                    None => Err(error),
                },
            };
        }
        Ok(match &request.action {
            ControlAction::RuntimeStatus => ControlResponse::RuntimeMetadata {
                instance: self.owner.instance_id().into(),
                epoch: self.owner.epoch(),
                operational: false,
                service_running: self.service_running(),
                requested_global_sessions: self.config.scheduler.global_max_sessions,
                requested_tasks_per_project: crate::config::MVP_PROJECT_TASKS,
            },
            _ => ControlResponse::Unavailable {
                request_id: request.request_id,
                reason: match &request.action {
                    ControlAction::ProposeGoal { .. } => UnavailableReason::PlanningUnavailable,
                    _ => UnavailableReason::TaskDriverUnavailable,
                },
            },
        })
    }
}
