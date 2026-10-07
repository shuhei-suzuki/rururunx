//! Accepted definitions originate at the actual local control ingress, not public Goal DTOs.
use super::super::*;
use super::recorded::{self, GOAL_TASK_PAGE_BYTES};
use crate::runtime::{
    control::{
        ControlAction, ControlRequest, ControlResponse, CriterionEvaluation, GoalReadView,
        HumanIngress, RecordedGoalTaskPage, RunnableAdmission,
    },
    goal::{CriterionEvaluator, ValidatedGoalPlan},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub(super) fn owner_current(tx: &Transaction<'_>, ingress: &HumanIngress) -> Result<()> {
    let (instance, epoch, _) = ingress.identity();
    let actual: (String, u64) = tx.query_row(
        "SELECT instance_id,epoch FROM runtime_epoch WHERE singleton=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    ensure!(
        actual == (instance.to_owned(), epoch) && epoch > 0,
        "control owner retired"
    );
    Ok(())
}
impl Store {
    pub(crate) fn accept_runtime_goal(
        &mut self,
        ingress: &HumanIngress,
        request: &ControlRequest,
        validated: &ValidatedGoalPlan,
        policy_sha256: &str,
    ) -> Result<ControlResponse> {
        let ControlAction::CreateGoal {
            project,
            expected_project,
            ..
        } = &request.action
        else {
            bail!("accepted Goal requires create action")
        };
        let (instance, epoch, uid) = ingress.identity();
        ensure!(
            request.instance == instance && request.epoch == epoch && !request.request_id.is_nil(),
            "control identity differs"
        );
        ensure!(
            policy_sha256.len() == 64 && policy_sha256.bytes().all(|b| b.is_ascii_hexdigit()),
            "policy digest invalid"
        );
        let encoded = serde_json::to_vec(request)?;
        ensure!(
            encoded.len() <= 1024 * 1024,
            "control request exceeds bound"
        );
        let action_sha256 = format!("{:x}", Sha256::digest(&encoded));
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        owner_current(&tx, ingress)?;
        let old:Option<(String,String,u32,String)>=tx.query_row("SELECT instance,action_sha256,ingress_uid,body FROM runtime_control_acks WHERE request_id=?1 AND length(CAST(body AS BLOB))<=65536",[request.request_id.to_string()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
        if let Some((saved_instance, digest, saved_uid, body)) = old {
            ensure!(
                saved_instance == instance && digest == action_sha256 && saved_uid == uid,
                "control request ID reused with different action"
            );
            let response = decode(body)?;
            tx.commit()?;
            return Ok(response);
        }
        let p_size: usize = tx.query_row(
            "SELECT length(CAST(body AS BLOB)) FROM projects WHERE id=?1",
            [project.to_string()],
            |r| r.get(0),
        )?;
        ensure!(p_size <= 1024 * 1024, "Project control body exceeds bound");
        let p: Project =
            read_tx(&tx, "projects", &project.to_string())?.context("unknown Project")?;
        let indexed: (String, u64) = tx.query_row(
            "SELECT root,version FROM projects WHERE id=?1",
            [project.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        ensure!(
            p.id == *project
                && indexed == (p.root.to_string_lossy().into_owned(), p.version)
                && p.version == *expected_project
                && p.state == ProjectState::Registered,
            "Project control currency changed"
        );
        let plan = validated.plan();
        // The validated definition is immutable content. Native evaluation never consumes its booleans as evidence.
        let d = &plan.definition;
        ensure!(
            d.canonical_digest()? == validated.definition_sha256(),
            "validated definition changed"
        );
        let criteria = d
            .criteria
            .iter()
            .map(|c| CompletionCriterion {
                id: c.id.clone(),
                description: c.description.clone(),
                evaluator: c.evaluator.clone(),
                evidence: None,
                satisfied: false,
            })
            .collect();
        ensure!(
            d.criteria
                .iter()
                .all(|c| c.evaluator != CriterionEvaluator::Unverified),
            "unverified accepted evaluator"
        );
        let mut goal = Goal::new(*project, d.objective.clone(), criteria);
        goal.title = d.title.clone();
        goal.constraints = d.constraints.clone();
        goal.non_goals = d.non_goals.clone();
        goal.source_refs = d.source_refs.clone();
        goal.state = GoalState::Running;
        goal.version = 1;
        let mut tasks = Vec::new();
        let mut keys = BTreeMap::new();
        for definition in &plan.tasks {
            let mut task = Task::new(
                *project,
                goal.id,
                definition.title.clone(),
                definition.executor.clone(),
            );
            task.acceptance_criteria = definition.acceptance_criteria.clone();
            task.reviewers = definition.reviewers.clone();
            task.workflow = definition.workflow;
            task.risk = definition.risk;
            keys.insert(definition.key.clone(), task.id);
            goal.dag.nodes.push(task.id);
            tasks.push(task);
        }
        for edge in &plan.dependencies {
            goal.dag.edges.push(Dependency {
                prerequisite: *keys
                    .get(&edge.prerequisite)
                    .context("missing prerequisite")?,
                dependent: *keys.get(&edge.dependent).context("missing dependent")?,
                hard: edge.hard,
            });
        }
        goal.dag.hard_order()?;
        let body = serde_json::to_string(&goal)?;
        ensure!(body.len() <= 4 * 1024 * 1024, "Goal body exceeds bound");
        // Goal graph references are resolved by the same transaction's Task insertions before commit.
        tx.execute(
            "INSERT INTO goals(id,project_id,version,body) VALUES(?1,?2,1,?3)",
            params![goal.id.to_string(), project.to_string(), body],
        )?;
        let mut sequence: u64 = tx.query_row(
            "SELECT sequence FROM scheduler_clock WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        let known_project: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM scheduler_projects WHERE project_id=?1)",
            [project.to_string()],
            |r| r.get(0),
        )?;
        if !known_project {
            tx.execute(
                "INSERT INTO scheduler_projects(project_id,rotation) VALUES(?1,0)",
                [project.to_string()],
            )?;
        }
        tx.execute(
            "INSERT INTO scheduler_goals(goal_id,project_id,rotation) VALUES(?1,?2,0)",
            params![goal.id.to_string(), project.to_string()],
        )?;
        for task in tasks {
            let task = put_task_tx(&tx, &task)?;
            sequence = sequence
                .checked_add(1)
                .filter(|v| *v <= i64::MAX as u64)
                .context("queue sequence exhausted")?;
            tx.execute("INSERT INTO scheduler_tasks(task_id,goal_id,project_id,queue_sequence) VALUES(?1,?2,?3,?4)",params![task.id.to_string(),goal.id.to_string(),project.to_string(),sequence])?;
        }
        validate_goal_references(&tx, &goal)?;
        tx.execute(
            "UPDATE scheduler_clock SET sequence=?1 WHERE singleton=1",
            [sequence],
        )?;
        tx.execute("INSERT INTO goal_authority(goal_id,project_id,definition_sha256,accepted_epoch,ingress_uid,policy_sha256,version) VALUES(?1,?2,?3,?4,?5,?6,1)",params![goal.id.to_string(),project.to_string(),validated.definition_sha256(),epoch,uid,policy_sha256])?;
        let response = ControlResponse::GoalAccepted {
            goal: goal.id,
            version: goal.version,
            task_count: goal.dag.nodes.len(),
        };
        let response_body = serde_json::to_string(&response)?;
        ensure!(response_body.len() <= 65536, "control response bound");
        tx.execute("INSERT INTO runtime_control_acks(request_id,instance,action_sha256,ingress_uid,accepted_epoch,body) VALUES(?1,?2,?3,?4,?5,?6)",params![request.request_id.to_string(),instance,action_sha256,uid,epoch,response_body])?;
        append_event(
            &tx,
            &goal.scope(),
            "rrx.private.runtime.goal_accepted",
            json!({"request":request.request_id,"definition":validated.definition_sha256(),"tasks":goal.dag.nodes.len(),"epoch":epoch}),
        )?;
        tx.commit()?;
        Ok(response)
    }
}

fn current_goal(tx: &Transaction<'_>, project: ProjectId, id: GoalId) -> Result<Goal> {
    let size: usize = tx.query_row(
        "SELECT length(CAST(body AS BLOB)) FROM goals WHERE id=?1 AND project_id=?2",
        params![id.to_string(), project.to_string()],
        |r| r.get(0),
    )?;
    ensure!(size <= 4 * 1024 * 1024, "Goal body exceeds Runtime bound");
    let goal: Goal = read_tx(tx, "goals", &id.to_string())?.context("unknown Goal")?;
    let version: u64 = tx.query_row(
        "SELECT version FROM goals WHERE id=?1 AND project_id=?2",
        params![id.to_string(), project.to_string()],
        |r| r.get(0),
    )?;
    ensure!(
        goal.id == id && goal.project_id == project && goal.version == version,
        "Goal body/index identity differs"
    );
    let accepted: String = tx.query_row(
        "SELECT definition_sha256 FROM goal_authority WHERE goal_id=?1 AND project_id=?2",
        params![id.to_string(), project.to_string()],
        |r| r.get(0),
    )?;
    let definition = crate::runtime::goal::GoalDefinition {
        title: goal.title.clone(),
        objective: goal.objective.clone(),
        criteria: goal
            .completion_criteria
            .iter()
            .map(|c| crate::runtime::goal::CriterionDefinition {
                id: c.id.clone(),
                description: c.description.clone(),
                evaluator: c.evaluator.clone(),
            })
            .collect(),
        constraints: goal.constraints.clone(),
        non_goals: goal.non_goals.clone(),
        source_refs: goal.source_refs.clone(),
    };
    ensure!(
        definition.canonical_digest()? == accepted,
        "accepted Goal definition differs"
    );
    Ok(goal)
}
fn scoped_tasks(tx: &Transaction<'_>, goal: &Goal) -> Result<Vec<Task>> {
    let (count,bytes):(usize,usize)=tx.query_row("SELECT count(*),COALESCE(sum(length(CAST(body AS BLOB))),0) FROM tasks WHERE goal_id=?1 AND project_id=?2",params![goal.id.to_string(),goal.project_id.to_string()],|r|Ok((r.get(0)?,r.get(1)?)))?;
    ensure!(
        count <= 4096 && bytes <= 32 * 1024 * 1024,
        "Runtime Task inventory exceeds bound"
    );
    let mut s = tx.prepare(
        "SELECT id,version,body FROM tasks WHERE goal_id=?1 AND project_id=?2 ORDER BY id",
    )?;
    let raw = s
        .query_map(
            params![goal.id.to_string(), goal.project_id.to_string()],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, u64>(1)?,
                    r.get::<_, String>(2)?,
                ))
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut tasks = Vec::with_capacity(raw.len());
    for (id, version, body) in raw {
        ensure!(body.len() <= 1024 * 1024, "Runtime Task body exceeds bound");
        let task: Task = decode(body)?;
        ensure!(
            task.id.to_string() == id
                && task.version == version
                && task.project_id == goal.project_id
                && task.goal_id == goal.id,
            "Runtime Task body/index identity differs"
        );
        tasks.push(task);
    }
    let ids = tasks
        .iter()
        .map(|t| t.id)
        .collect::<std::collections::BTreeSet<_>>();
    ensure!(
        ids == goal.dag.nodes.iter().copied().collect() && ids.len() == goal.dag.nodes.len(),
        "accepted Goal Task inventory differs"
    );
    goal.dag.hard_order()?;
    Ok(tasks)
}
impl Store {
    pub(crate) fn runtime_goal_facts(
        &self,
        ingress: &HumanIngress,
        project: ProjectId,
        id: GoalId,
        view: Option<GoalReadView>,
    ) -> Result<ControlResponse> {
        let tx = self.connection.unchecked_transaction()?;
        owner_current(&tx, ingress)?;
        let accepted: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM goal_authority WHERE goal_id=?1 AND project_id=?2)",
            params![id.to_string(), project.to_string()],
            |r| r.get(0),
        )?;
        if !accepted {
            let mut response = super::proposals::proposal_facts(&tx, project, id)?;
            if view.is_some() {
                recorded::enrich_proposal(&mut response, project)?;
            }
            tx.commit()?;
            return Ok(response);
        }
        let goal = current_goal(&tx, project, id)?;
        let tasks = scoped_tasks(&tx, &goal)?;
        let mut states = BTreeMap::new();
        for task in &tasks {
            let observation = super::waiting::observe(&tx, task, &goal);
            let state = super::waiting::effective_state(task, observation.as_ref());
            *states
                .entry(
                    serde_json::to_value(state)?
                        .as_str()
                        .context("Task state encoding")?
                        .to_owned(),
                )
                .or_insert(0) += 1;
        }
        let initial_driver:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM scheduler_tasks s JOIN task_drivers d ON d.task_id=s.task_id AND d.goal_id=s.goal_id AND d.project_id=s.project_id WHERE s.goal_id=?1 AND s.project_id=?2 AND s.attention IS NULL AND d.owner_epoch=?3 AND d.state='driving')",params![id.to_string(),project.to_string(),ingress.identity().1],|r|r.get(0))?;
        let mut response = ControlResponse::GoalFacts {
            goal: id,
            version: goal.version,
            state: goal.state,
            task_count: tasks.len(),
            states,
            recorded: None,
            dispatch_available: false,
            attention: if initial_driver {
                crate::runtime::control::UnavailableReason::NativeContinuationUnavailable
            } else {
                crate::runtime::control::UnavailableReason::NativeBindingUnavailable
            },
        };
        if view.is_some() {
            recorded::enrich_status(&mut response, &goal, project)?;
        }
        tx.commit()?;
        Ok(response)
    }
    pub(crate) fn set_runtime_goal_lifecycle(
        &mut self,
        ingress: &HumanIngress,
        request: &ControlRequest,
        policy_sha256: &str,
    ) -> Result<ControlResponse> {
        use crate::runtime::control::{GoalControl, UnavailableReason};
        let ControlAction::SetGoalLifecycle {
            project,
            goal: id,
            expected_goal,
            target,
            reason,
        } = &request.action
        else {
            bail!("Goal lifecycle action required")
        };
        ensure!(
            !reason.trim().is_empty() && reason.len() <= 16 * 1024,
            "Goal lifecycle reason invalid"
        );
        let (instance, epoch, uid) = ingress.identity();
        ensure!(
            request.instance == instance && request.epoch == epoch && !request.request_id.is_nil(),
            "control identity differs"
        );
        let encoded = serde_json::to_vec(request)?;
        ensure!(
            encoded.len() <= 1024 * 1024,
            "control request exceeds bound"
        );
        let request_hash = format!("{:x}", Sha256::digest(encoded));
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        owner_current(&tx, ingress)?;
        let old:Option<(String,String,u32,String)>=tx.query_row("SELECT instance,action_sha256,ingress_uid,body FROM runtime_control_acks WHERE request_id=?1 AND length(CAST(body AS BLOB))<=65536",[request.request_id.to_string()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
        if let Some((old_instance, old_hash, old_uid, body)) = old {
            ensure!(
                old_instance == instance && old_hash == request_hash && old_uid == uid,
                "control request ID reused with different action"
            );
            let response = decode(body)?;
            tx.commit()?;
            return Ok(response);
        }
        let mut goal = current_goal(&tx, *project, *id)?;
        ensure!(goal.version == *expected_goal, "Goal lifecycle CAS differs");
        let tasks = scoped_tasks(&tx, &goal)?;
        let next = match target {
            GoalControl::Pause => {
                ensure!(!goal_terminal(goal.state), "terminal Goal cannot pause");
                GoalState::Paused
            }
            GoalControl::Cancel => {
                ensure!(!goal_terminal(goal.state), "terminal Goal cannot cancel");
                GoalState::Cancelled
            }
            GoalControl::Fail => {
                ensure!(!goal_terminal(goal.state), "terminal Goal cannot fail");
                GoalState::Failed
            }
            GoalControl::Resume => {
                ensure!(
                    matches!(
                        goal.state,
                        GoalState::Paused | GoalState::Blocked | GoalState::WaitingHuman
                    ),
                    "Goal not resumable"
                );
                let saved_policy: String = tx.query_row(
                    "SELECT policy_sha256 FROM goal_authority WHERE goal_id=?1 AND project_id=?2",
                    params![id.to_string(), project.to_string()],
                    |r| r.get(0),
                )?;
                ensure!(
                    saved_policy == policy_sha256,
                    "Goal accepted policy changed; resume remains held"
                );
                let history:u64=tx.query_row("SELECT (SELECT count(*) FROM execution_units WHERE goal_id=?1 AND project_id=?2)+(SELECT count(*) FROM records WHERE goal_id=?1 AND project_id=?2 AND kind IN ('workflow','session','worktree_lock'))+(SELECT count(*) FROM context_versions WHERE project_id=?2 AND goal_id=?1)+(SELECT count(*) FROM source_recoveries WHERE goal_id=?1 AND project_id=?2)",params![id.to_string(),project.to_string()],|r|r.get(0))?;
                // Only a genuinely never-prepared graph can resume here. A row,
                // retired Unit or old Session is not a recovery capability.
                if history > 0 {
                    return Ok(ControlResponse::Unavailable {
                        request_id: request.request_id,
                        reason: UnavailableReason::FreshBootstrapRecoveryUnavailable,
                    });
                }
                ensure_project_registered(&tx, *project)?;
                GoalState::Running
            }
        };
        // Bound the complete fence inventory BEFORE mutating any row. The helper
        // remains the existing logical cancellation producer; no process death is asserted.
        let (units,bytes):(usize,usize)=tx.query_row("SELECT count(*),COALESCE(sum(length(CAST(body AS BLOB))),0) FROM execution_units WHERE goal_id=?1 AND project_id=?2 AND (native_effects_open=1 OR result_finalization_open=1)",params![id.to_string(),project.to_string()],|r|Ok((r.get(0)?,r.get(1)?)))?;
        ensure!(
            units <= 8192 && bytes <= 32 * 1024 * 1024,
            "Goal fence inventory exceeds bound"
        );
        for task in tasks.iter().filter(|_| next != GoalState::Running) {
            let generation:Option<u64>=tx.query_row("SELECT generation FROM task_execution WHERE task_id=?1 AND active_unit IS NOT NULL",[task.id.to_string()],|r|r.get(0)).optional()?;
            ensure!(
                generation.is_none_or(|g| g < i64::MAX as u64),
                "Task generation exhausted; scoped lifecycle fence unavailable"
            );
            super::super::execution::fence_task_tx(&tx, &task.scope())?;
            let current:Option<(String,u64,u64,String)>=tx.query_row("SELECT id,owner_epoch,version,body FROM task_drivers WHERE task_id=?1 AND state='driving'",[task.id.to_string()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
            if let Some((id, owner_epoch, version, body)) = current {
                super::driver::invalidate_tx(
                    &self.binding_permits,
                    &tx,
                    task.id,
                    &id,
                    owner_epoch,
                    version,
                    &body,
                )?;
            }
        }
        let previous = goal.version;
        goal.state = next;
        bump(&mut goal.version)?;
        goal.updated_at = now_ms();
        let body = serde_json::to_string(&goal)?;
        ensure!(
            body.len() <= 4 * 1024 * 1024,
            "Goal lifecycle body exceeds bound"
        );
        ensure!(
            tx.execute(
                "UPDATE goals SET version=?1,body=?2 WHERE id=?3 AND project_id=?4 AND version=?5",
                params![
                    goal.version,
                    body,
                    id.to_string(),
                    project.to_string(),
                    previous
                ]
            )? == 1,
            "Goal lifecycle write CAS differs"
        );
        let response = ControlResponse::GoalLifecycleChanged {
            goal: *id,
            version: goal.version,
            state: goal.state,
        };
        let response_body = serde_json::to_string(&response)?;
        tx.execute("INSERT INTO runtime_control_acks(request_id,instance,action_sha256,ingress_uid,accepted_epoch,body) VALUES(?1,?2,?3,?4,?5,?6)",params![request.request_id.to_string(),instance,request_hash,uid,epoch,response_body])?;
        append_event(
            &tx,
            &goal.scope(),
            "rrx.private.runtime.goal_lifecycle",
            json!({"request":request.request_id,"state":goal.state,"version":goal.version,"reason":reason,"cleanup":"best_effort_pending"}),
        )?;
        tx.commit()?;
        Ok(response)
    }
}

impl Store {
    pub(crate) fn runtime_goal_task_page(
        &self,
        ingress: &HumanIngress,
        project: ProjectId,
        id: GoalId,
        after: Option<TaskId>,
        maximum: usize,
        view: Option<GoalReadView>,
    ) -> Result<ControlResponse> {
        use crate::runtime::control::TaskFacts;
        ensure!(
            (1..=128).contains(&maximum),
            "Task page maximum must be 1..128"
        );
        let tx = self.connection.unchecked_transaction()?;
        owner_current(&tx, ingress)?;
        let goal = current_goal(&tx, project, id)?;
        // Complete bounded inventory validation precedes any response. IDs and
        // graph membership, not titles or JSON principal labels, define scope.
        let all = scoped_tasks(&tx, &goal)?;
        ensure!(
            after.is_none_or(|a| all.iter().any(|t| t.id == a)),
            "Task cursor is foreign or no longer present"
        );
        let eligible = all
            .iter()
            .filter(|t| after.is_none_or(|a| t.id > a))
            .collect::<Vec<_>>();
        let lookup = all.iter().map(|t| (t.id, t)).collect::<BTreeMap<_, _>>();
        let mut facts = Vec::new();
        let mut enriched = view.map(|view| RecordedGoalTaskPage {
            view,
            project,
            dag: recorded::dag(&goal),
            nodes: Vec::new(),
            criterion_evaluation: CriterionEvaluation::Unavailable,
            runnable_admission: RunnableAdmission::Unknown,
        });
        let mut next = None;
        // Borrow candidates so packing never clones accumulated page strings/details.
        #[derive(serde::Serialize)]
        struct Candidate<'a> {
            outcome: &'static str,
            goal: GoalId,
            version: u64,
            tasks: &'a [TaskFacts],
            next: Option<TaskId>,
            #[serde(skip_serializing_if = "Option::is_none")]
            recorded: &'a Option<RecordedGoalTaskPage>,
        }
        for task in eligible.iter().take(maximum) {
            ensure!(
                task.phase.as_ref().is_none_or(|p| p.len() <= 128),
                "Task phase exceeds status bound"
            );
            facts.push(TaskFacts {
                scope: task.scope(),
                version: task.version,
                state: super::waiting::effective_state(
                    task,
                    super::waiting::observe(&tx, task, &goal).as_ref(),
                ),
                phase: task.phase.clone(),
            });
            if let Some(page) = &mut enriched {
                page.nodes.push(recorded::node(task, &goal, &lookup)?);
            }
            let more = eligible.len() > facts.len();
            let candidate = Candidate {
                outcome: "goal_task_page",
                goal: id,
                version: goal.version,
                tasks: &facts,
                next: more.then_some(task.id),
                recorded: &enriched,
            };
            if !recorded::fits(&candidate, GOAL_TASK_PAGE_BYTES) {
                facts.pop();
                if let Some(page) = &mut enriched {
                    page.nodes.pop();
                }
                ensure!(!facts.is_empty(), "one Task status exceeds response budget");
                next = facts.last().and_then(|f| f.scope.task_id);
                break;
            }
            next = more.then_some(task.id);
        }
        let response = ControlResponse::GoalTaskPage {
            goal: id,
            version: goal.version,
            tasks: facts,
            next,
            recorded: enriched,
        };
        ensure!(
            recorded::fits(&response, GOAL_TASK_PAGE_BYTES),
            "Task response budget exceeded"
        );
        tx.commit()?;
        Ok(response)
    }
}
