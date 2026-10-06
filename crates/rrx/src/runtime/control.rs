//! Dedicated local controls. Public DTOs contain neither principal nor authority.
use super::{Runtime, goal::GoalPlan};
use crate::domain::{GoalId, ProjectId, Scope};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
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
    /// Interim metadata only; operational status requires the schema9 Store consumer.
    RuntimeMetadata {
        instance: String,
        epoch: u64,
        operational: bool,
        requested_global_sessions: usize,
        requested_tasks_per_project: usize,
    },
    Unavailable {
        request_id: Uuid,
        reason: UnavailableReason,
    },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableReason {
    RuntimeSchemaPending,
}

/// Constructed solely from the actual accepted socket, never Deserialize/Clone/public constructor.
struct HumanIngress {
    uid: u32,
    instance: String,
    epoch: u64,
}
impl HumanIngress {
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
    /// No effect or accepted definition is published by this initial ingress milestone.
    /// Schema9/Driver integration must replace the explicit unavailable dispatch.
    pub async fn handle_control(
        &self,
        accepted: &UnixStream,
        request: ControlRequest,
    ) -> Result<ControlResponse> {
        let ingress = HumanIngress::from_connection(self, accepted)?;
        ingress.check(&request)?;
        if let ControlAction::CreateGoal { plan, .. } = &request.action {
            plan.clone().validate(&self.config)?;
        }
        Ok(match request.action {
            ControlAction::RuntimeStatus => ControlResponse::RuntimeMetadata {
                instance: self.owner.instance_id().into(),
                epoch: self.owner.epoch(),
                operational: false,
                requested_global_sessions: self.config.scheduler.global_max_sessions,
                requested_tasks_per_project: self.config.scheduler.max_tasks_per_project,
            },
            _ => ControlResponse::Unavailable {
                request_id: request.request_id,
                reason: UnavailableReason::RuntimeSchemaPending,
            },
        })
    }
}
