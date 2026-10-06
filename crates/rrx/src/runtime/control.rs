//! Dedicated local controls. Public DTOs contain neither principal nor authority.
use super::{Runtime, goal::GoalPlan};
use crate::domain::{GoalId, GoalState, ProjectId, ProjectState, Scope, TaskId, TaskState};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf, time::Duration};
use tokio::net::UnixStream;
use uuid::Uuid;

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
    },
    GoalTasks {
        project: ProjectId,
        goal: GoalId,
        after: Option<TaskId>,
        maximum: usize,
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
    GoalFacts {
        goal: GoalId,
        version: u64,
        state: GoalState,
        task_count: usize,
        states: std::collections::BTreeMap<String, usize>,
        dispatch_available: bool,
        attention: UnavailableReason,
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
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskFacts {
    pub scope: Scope,
    pub version: u64,
    pub state: TaskState,
    pub phase: Option<String>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableReason {
    TaskDriverUnavailable,
    PlanningUnavailable,
    NativeBindingUnavailable,
    FreshBootstrapRecoveryUnavailable,
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
    /// Trusted controls publish only their specific typed decisions. A public
    /// request never supplies Driver, native input or result authority.
    pub async fn handle_control(
        &self,
        accepted: &UnixStream,
        request: ControlRequest,
    ) -> Result<ControlResponse> {
        let ingress = HumanIngress::from_connection(self, accepted)?;
        ingress.check(&request)?;
        if let ControlAction::ResolveProject { selector, cwd } = &request.action {
            ensure!(cwd.as_os_str().len() <= 4096, "routing CWD exceeds bound");
            let cwd = canonical_directory(cwd.clone()).await?;
            let snapshot = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .runtime_project_routes(
                    selector.as_deref(),
                    &cwd,
                    self.owner.instance_id(),
                    self.owner.epoch(),
                )?;
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
                    selector.as_deref(),
                    &cwd,
                    self.owner.instance_id(),
                    self.owner.epoch(),
                    &snapshot,
                )?;
            return Ok(ControlResponse::ProjectResolved {
                project: selected.id,
                version: selected.version,
                display_name: selected.name.clone(),
                state: selected.state,
            });
        }
        if let ControlAction::CreateGoal { plan, .. } = &request.action {
            ensure!(
                !self.stopping.load(std::sync::atomic::Ordering::SeqCst),
                "Runtime stopping; new Goal refused"
            );
            let validated = plan.clone().validate(&self.config)?;
            use sha2::{Digest, Sha256};
            let policy = serde_json::to_vec(&self.config)?;
            ensure!(policy.len() <= 1024 * 1024, "Runtime policy exceeds bound");
            let digest = format!("{:x}", Sha256::digest(&policy));
            let response = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .accept_runtime_goal(&ingress, &request, &validated, &digest)?;
            self.wake.notify_one();
            return Ok(response);
        }
        if let ControlAction::GoalStatus { project, goal } = &request.action {
            return self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .runtime_goal_facts(&ingress, *project, *goal);
        }
        if let ControlAction::GoalTasks {
            project,
            goal,
            after,
            maximum,
        } = &request.action
        {
            return self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .runtime_goal_task_page(&ingress, *project, *goal, *after, *maximum);
        }
        if matches!(&request.action, ControlAction::SetGoalLifecycle { .. }) {
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
            use sha2::{Digest, Sha256};
            let policy = serde_json::to_vec(&self.config)?;
            ensure!(policy.len() <= 1024 * 1024, "Runtime policy exceeds bound");
            let policy_digest = format!("{:x}", Sha256::digest(policy));
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
            self.shutdown().await?;
            return Ok(ControlResponse::RuntimeStopped {
                instance: self.owner.instance_id().into(),
                epoch: self.owner.epoch(),
            });
        }
        Ok(match &request.action {
            ControlAction::RuntimeStatus => ControlResponse::RuntimeMetadata {
                instance: self.owner.instance_id().into(),
                epoch: self.owner.epoch(),
                operational: false,
                service_running: self.service_running(),
                requested_global_sessions: self.config.scheduler.global_max_sessions,
                requested_tasks_per_project: self.config.scheduler.max_tasks_per_project,
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
