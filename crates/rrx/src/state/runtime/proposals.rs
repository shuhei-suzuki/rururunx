//! Human-ingress inert proposals in the existing protected layout. No authority.
use super::{super::*, goals::owner_current};
use crate::runtime::control::{
    ControlAction, ControlRequest, ControlResponse, HumanIngress, UnavailableReason,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProposalObservation {
    kind: String,
    request_id: uuid::Uuid,
    // Exact proposed Goal version, independent of goal_observations.version.
    // Later observer revisions never refresh this pin or grant acceptance.
    goal_version: u64,
    objective_sha256: String,
}
fn objective_digest(objective: &str) -> String {
    format!("{:x}", Sha256::digest(objective.as_bytes()))
}
impl Store {
    pub(crate) fn propose_runtime_goal(
        &mut self,
        ingress: &HumanIngress,
        request: &ControlRequest,
    ) -> Result<ControlResponse> {
        let ControlAction::ProposeGoal {
            project,
            expected_project,
            objective,
        } = &request.action
        else {
            bail!("Goal proposal action required");
        };
        ensure!(
            !objective.trim().is_empty() && objective.len() <= 16 * 1024,
            "Goal proposal objective empty/over budget"
        );
        let (instance, epoch, uid) = ingress.identity();
        ensure!(
            request.instance == instance && request.epoch == epoch && !request.request_id.is_nil(),
            "proposal control identity differs"
        );
        let encoded = serde_json::to_vec(request)?;
        ensure!(
            encoded.len() <= 1024 * 1024,
            "proposal request exceeds bound"
        );
        let request_hash = format!("{:x}", Sha256::digest(encoded));
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        owner_current(&tx, ingress)?;
        let previous: Option<(String, String, u32, String)> = tx.query_row(
            "SELECT instance,action_sha256,ingress_uid,body FROM runtime_control_acks WHERE request_id=?1 AND length(CAST(body AS BLOB))<=65536",
            [request.request_id.to_string()], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        ).optional()?;
        if let Some((saved_instance, saved_hash, saved_uid, body)) = previous {
            ensure!(
                saved_instance == instance && saved_hash == request_hash && saved_uid == uid,
                "proposal request ID reused with different action"
            );
            let response = decode(body)?;
            tx.commit()?;
            return Ok(response);
        }
        let size: usize = tx.query_row(
            "SELECT length(CAST(body AS BLOB)) FROM projects WHERE id=?1",
            [project.to_string()],
            |r| r.get(0),
        )?;
        ensure!(size <= 1024 * 1024, "Project proposal body exceeds bound");
        let p: Project =
            read_tx(&tx, "projects", &project.to_string())?.context("unknown proposal Project")?;
        let indexed: (String, u64) = tx.query_row(
            "SELECT root,version FROM projects WHERE id=?1",
            [project.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        ensure!(
            p.id == *project
                && p.state == ProjectState::Registered
                && indexed == (p.root.to_string_lossy().into_owned(), p.version)
                && p.version == *expected_project,
            "Project proposal currency changed"
        );
        let mut goal = Goal::new(*project, objective.clone(), vec![]);
        goal.state = GoalState::Analyzing;
        goal.version = 1;
        let body = serde_json::to_string(&goal)?;
        ensure!(body.len() <= 65536, "inert Goal body exceeds bound");
        tx.execute(
            "INSERT INTO goals(id,project_id,version,body) VALUES(?1,?2,1,?3)",
            params![goal.id.to_string(), project.to_string(), body],
        )?;
        let observation = ProposalObservation {
            kind: "inert_goal_proposal".into(),
            request_id: request.request_id,
            goal_version: goal.version,
            objective_sha256: objective_digest(objective),
        };
        tx.execute(
            "INSERT INTO goal_observations(goal_id,project_id,version,body) VALUES(?1,?2,1,?3)",
            params![
                goal.id.to_string(),
                project.to_string(),
                serde_json::to_string(&observation)?
            ],
        )?;
        let response = ControlResponse::GoalProposed {
            goal: goal.id,
            version: goal.version,
            state: goal.state,
        };
        tx.execute("INSERT INTO runtime_control_acks(request_id,instance,action_sha256,ingress_uid,accepted_epoch,body) VALUES(?1,?2,?3,?4,?5,?6)",
            params![request.request_id.to_string(), instance, request_hash, uid, epoch, serde_json::to_string(&response)?])?;
        append_event(
            &tx,
            &goal.scope(),
            "rrx.private.runtime.goal_proposed",
            json!({"request":request.request_id,"epoch":epoch,"accepted":false}),
        )?;
        tx.commit()?;
        Ok(response)
    }
}

pub(super) fn proposal_facts(
    tx: &Transaction<'_>,
    project: ProjectId,
    id: GoalId,
) -> Result<ControlResponse> {
    let (version, bytes): (u64, usize) = tx.query_row(
        "SELECT version,length(CAST(body AS BLOB)) FROM goals WHERE id=?1 AND project_id=?2",
        params![id.to_string(), project.to_string()],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    ensure!(bytes <= 65536, "proposal Goal body exceeds bound");
    let goal: Goal = read_tx(tx, "goals", &id.to_string())?.context("unknown proposal")?;
    ensure!(
        goal.id == id
            && goal.project_id == project
            && goal.version == version
            && version == 1
            && goal.state == GoalState::Analyzing
            && goal.title == goal.objective
            && !goal.objective.trim().is_empty()
            && goal.objective.len() <= 16 * 1024
            && goal.completion_criteria.is_empty()
            && goal.dag.nodes.is_empty()
            && goal.dag.edges.is_empty()
            && goal.constraints.is_empty()
            && goal.non_goals.is_empty()
            && goal.source_refs.is_empty()
            && goal.followups.is_empty()
            && goal.blockers.is_empty()
            && goal.context_version == 0,
        "inert proposal body/index differs"
    );
    let observation_body: String = tx.query_row(
        "SELECT body FROM goal_observations WHERE goal_id=?1 AND project_id=?2 AND version=1 AND length(CAST(body AS BLOB))<=65536",
        params![id.to_string(), project.to_string()], |r| r.get(0),
    )?;
    let observation: ProposalObservation = decode(observation_body)?;
    ensure!(
        observation.kind == "inert_goal_proposal"
            && observation.goal_version == version
            && observation.objective_sha256 == objective_digest(&goal.objective)
            && !observation.request_id.is_nil(),
        "proposal observation differs"
    );
    let body: String = tx.query_row(
        "SELECT body FROM runtime_control_acks WHERE request_id=?1 AND length(CAST(body AS BLOB))<=65536",
        [observation.request_id.to_string()], |r| r.get(0),
    )?;
    let acknowledged: ControlResponse = decode(body)?;
    ensure!(
        matches!(acknowledged, ControlResponse::GoalProposed { goal: acknowledged_id,
        version: acknowledged_version, state: GoalState::Analyzing }
        if acknowledged_id == id && acknowledged_version == version),
        "proposal acknowledgement differs"
    );
    let tasks: u64 = tx.query_row(
        "SELECT count(*) FROM tasks WHERE goal_id=?1",
        [id.to_string()],
        |r| r.get(0),
    )?;
    ensure!(tasks == 0, "inert proposal has Tasks");
    Ok(ControlResponse::GoalProposalFacts {
        goal: id,
        version,
        state: goal.state,
        objective: goal.objective,
        accepted: false,
        task_count: 0,
        dispatch_available: false,
        attention: UnavailableReason::PlanningUnavailable,
    })
}
