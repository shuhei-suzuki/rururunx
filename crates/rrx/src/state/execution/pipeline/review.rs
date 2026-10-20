//! Reviewer context and evidence use the same owner/CAS/Goal checks as other steps.
use super::*;
use crate::execution::pipeline::{ReviewResult, ReviewSeverity};

impl Store {
    pub(crate) fn pipeline_review_context(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
    ) -> Result<(Project, Value, ReviewSeverity)> {
        let tx = self.connection.transaction()?;
        pipeline_owner_current(&tx, owner)?;
        let (run, step) = cursor_rows(&tx, owner, cursor, true)?;
        require_last_step(&tx, &step)?;
        ensure!(
            run.state == "running"
                && step.kind == "review"
                && step.provider.as_deref() == Some("claude")
                && step.step_base == run.head_commit,
            "review_context_changed"
        );
        let project: Project = read_tx(&tx, "projects", &run.project_id.to_string())?
            .context("review Project missing")?;
        let task: Task = read_tx(
            &tx,
            "tasks",
            &run.task_id.context("review Task missing")?.to_string(),
        )?
        .context("review Task missing")?;
        ensure!(
            task.project_id == run.project_id && task.goal_id == run.goal_id,
            "review Task scope changed"
        );
        let policy: String = tx.query_row(
            "SELECT body FROM goal_execution_profiles WHERE goal_id=?1",
            [run.goal_id.to_string()],
            |r| r.get(0),
        )?;
        let policy: Value = serde_json::from_str(&policy)?;
        let threshold: ReviewSeverity = serde_json::from_value(policy["blocking_severity"].clone())
            .context("review policy missing")?;
        let goal = pipeline_goal(&tx, &run, true)?;
        let scopes = executor_scopes(&policy, &goal.dag.nodes, task.id)?;
        let data = json!({"title":task.title,"acceptance_criteria":task.acceptance_criteria,"scope":scopes});
        tx.commit()?;
        Ok((project, data, threshold))
    }

    pub(crate) fn pipeline_review_preflight(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        tree: &str,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (run, mut step) = cursor_rows(&tx, owner, cursor, true)?;
        require_last_step(&tx, &step)?;
        ensure!(
            run.state == "running"
                && step.kind == "review"
                && step.state == "planned"
                && step.step_base == run.head_commit,
            "review_preflight_changed"
        );
        ensure!(
            matches!(tree.len(), 40 | 64) && tree.bytes().all(|b| b.is_ascii_hexdigit()),
            "review_snapshot_changed"
        );
        let nonce = step.body["review_nonce"]
            .as_str()
            .context("review_nonce_invalid")?;
        let reused: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM pipeline_steps WHERE kind='review' AND json_extract(body,'$.review_nonce')=?1 AND NOT(run_id=?2 AND seq=?3 AND attempt=?4))", params![nonce,step.run_id,step.seq,step.attempt], |r| r.get(0))?;
        ensure!(!reused, "review_nonce_reused");
        step.body["snapshot_tree"] = json!(tree);
        write_step(&tx, owner, &step)?;
        tx.commit()?;
        Ok(())
    }
}

pub(super) fn review_adoption(
    step: &PipelineStep,
    evidence: &Value,
    threshold: ReviewSeverity,
) -> Result<()> {
    ensure!(
        step.body["exit"]["code"] == 0 && step.body["provider_outcome"] == "success",
        "review_agent_failed"
    );
    ensure!(
        evidence["snapshot_tree"].is_string()
            && evidence["snapshot_tree"] == step.body["snapshot_tree"],
        "review_snapshot_changed"
    );
    let mut verdict: ReviewResult =
        serde_json::from_value(evidence["verdict"].clone()).context("verdict_schema")?;
    verdict.validate(threshold)?;
    ensure!(
        serde_json::to_value(verdict)? == evidence["verdict"],
        "verdict_schema"
    );
    Ok(())
}
