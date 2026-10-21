//! Fenced result staging and deterministic planning; no I/O occurs in these transactions.
use super::*;
use crate::execution::{
    pipeline::{
        ReviewResult, ReviewSeverity, RoundDecision,
        confirm::{ConfirmResult, FilteredReview, Label},
        review as wire,
        rounds::{self, FixInput},
    },
    verification::AdmittedProfile,
};
use std::collections::BTreeSet;

pub(crate) struct RoundContext {
    pub run: PipelineRun,
    pub step: PipelineStep,
    pub project: Project,
    pub task: Value,
    pub scopes: Vec<String>,
    pub threshold: ReviewSeverity,
    pub max: u32,
    pub profile: AdmittedProfile,
    pub history: Vec<PipelineStep>,
}

fn context(
    tx: &Transaction<'_>,
    owner: &RuntimeOwner,
    cursor: &PipelineCursor,
) -> Result<RoundContext> {
    pipeline_owner_current(tx, owner)?;
    let (run, step) = cursor_rows(tx, owner, cursor, true)?;
    require_last_step(tx, &step)?;
    check_wait(tx, owner, cursor.wait_version)?;
    ensure!(
        run.state == "running"
            && run.kind == "task"
            && step.round == run.round
            && (step.step_base == run.head_commit
                || (matches!(step.kind.as_str(), "implement" | "fix")
                    && step.state == "adopted"
                    && step.result_commit.as_ref() == Some(&run.head_commit))),
        "round_context_changed"
    );
    let project =
        read_tx(tx, "projects", &run.project_id.to_string())?.context("round Project missing")?;
    let task: Task = read_tx(
        tx,
        "tasks",
        &run.task_id.context("round Task missing")?.to_string(),
    )?
    .context("round Task missing")?;
    ensure!(
        task.goal_id == run.goal_id && task.project_id == run.project_id,
        "round Task changed"
    );
    let policy: String = tx.query_row(
        "SELECT body FROM goal_execution_profiles WHERE goal_id=?1",
        [run.goal_id.to_string()],
        |r| r.get(0),
    )?;
    let policy: Value = serde_json::from_str(&policy)?;
    let threshold = serde_json::from_value(policy["blocking_severity"].clone())?;
    let max: u32 = serde_json::from_value(policy["max_fix_rounds"].clone())?;
    ensure!(max <= 2 && run.round <= max, "round_policy_invalid");
    let goal = pipeline_goal(tx, &run, true)?;
    let scopes = executor_scopes(&policy, &goal.dag.nodes, task.id)?;
    let data =
        json!({"title":task.title,"acceptance_criteria":task.acceptance_criteria,"scope":scopes});
    let mut query = tx.prepare("SELECT * FROM pipeline_steps WHERE run_id=?1 AND (state='adopted' OR (seq=?2 AND attempt=?3)) ORDER BY seq,attempt")?;
    let history = query
        .query_map(params![run.id, step.seq, step.attempt], step_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(RoundContext {
        profile: admitted_profile(tx, run.goal_id).context("tests_profile_changed")?,
        run,
        step,
        project,
        task: data,
        scopes,
        threshold,
        max,
        history,
    })
}

fn validated(step: &PipelineStep) -> bool {
    step.state == "adopted"
        || (step.state == "exited" && step.body["round_result_validated"] == true)
}
fn current_tests(ctx: &RoundContext) -> Result<Vec<&PipelineStep>> {
    let tests: Vec<_> = ctx
        .history
        .iter()
        .filter(|s| {
            s.kind == "test"
                && s.round == ctx.run.round
                && s.step_base == ctx.run.head_commit
                && validated(s)
        })
        .collect();
    for (index, step) in tests.iter().enumerate() {
        ensure!(
            step.body["command_index"].as_u64() == Some(index as u64),
            "round_tests_incomplete"
        );
        let command = ctx
            .profile
            .proposal
            .commands
            .get(index)
            .context("round_tests_incomplete")?;
        ensure!(
            step.body["command_id"] == command.id
                && step.body["adoption"]["command_id"] == command.id,
            "round_tests_incomplete"
        );
        let pass = test_pass(step)?;
        ensure!(
            step.body["adoption"]["result"] == if pass { "pass" } else { "fail" },
            "round_tests_incomplete"
        );
    }
    Ok(tests)
}
pub(crate) fn source_review(
    ctx: &RoundContext,
) -> Result<(&PipelineStep, ReviewResult, FilteredReview)> {
    let review = if ctx.step.kind == "review" {
        &ctx.step
    } else {
        let seq = ctx.step.body["review_seq"]
            .as_u64()
            .context("confirm_review_missing")?;
        ctx.history
            .iter()
            .find(|s| s.seq as u64 == seq && s.kind == "review" && s.state == "adopted")
            .context("confirm_review_missing")?
    };
    ensure!(
        validated(review)
            && review.round == ctx.run.round
            && review.step_base == ctx.run.head_commit,
        "confirm_review_changed"
    );
    let mut result: ReviewResult =
        serde_json::from_value(review.body["adoption"]["verdict"].clone())?;
    result.validate(ctx.threshold)?;
    let filtered: FilteredReview =
        serde_json::from_value(review.body["filtering"].clone()).context("round_filter_missing")?;
    validate_filter(&result, ctx.threshold, &filtered)?;
    Ok((review, result, filtered))
}

fn validate_filter(
    review: &ReviewResult,
    threshold: ReviewSeverity,
    filtered: &FilteredReview,
) -> Result<()> {
    let expected: BTreeSet<_> = review
        .findings
        .iter()
        .filter(|f| rounds::blocking(f, threshold))
        .map(|f| f.id.as_str())
        .collect();
    let ids: Vec<_> = filtered
        .blocking
        .iter()
        .map(|f| f.id.as_str())
        .chain(filtered.invalid.iter().map(|f| f.id.as_str()))
        .collect();
    ensure!(
        ids.len() == expected.len() && ids.iter().copied().collect::<BTreeSet<_>>() == expected,
        "round_filter_changed"
    );
    for f in &filtered.blocking {
        let original = review
            .findings
            .iter()
            .find(|v| v.id == f.id)
            .context("round_filter_changed")?;
        ensure!(
            serde_json::to_value(f)? == serde_json::to_value(original)?,
            "round_filter_changed"
        );
    }
    let non: Vec<_> = review
        .findings
        .iter()
        .filter(|f| !rounds::blocking(f, threshold))
        .map(|f| &f.id)
        .collect();
    ensure!(
        non == filtered.non_blocking.iter().collect::<Vec<_>>(),
        "round_filter_changed"
    );
    Ok(())
}

pub(crate) fn round_action(ctx: &RoundContext) -> Result<RoundDecision> {
    ensure!(validated(&ctx.step), "round_result_incomplete");
    let tests = current_tests(ctx)?;
    if ctx.step.kind == "test" {
        ensure!(
            tests.last().is_some_and(|s| s.seq == ctx.step.seq),
            "round_tests_incomplete"
        );
        if !test_pass(&ctx.step)? {
            return rounds::decide(false, None, 0, 0, ctx.run.round, ctx.max);
        }
        ensure!(
            tests.iter().all(|s| test_pass(s).unwrap_or(false)),
            "round_tests_incomplete"
        );
        if tests.len() < ctx.profile.proposal.commands.len() {
            return Ok(RoundDecision::Test(tests.len()));
        }
        return Ok(RoundDecision::Review);
    }
    ensure!(
        tests.len() == ctx.profile.proposal.commands.len()
            && !tests.is_empty()
            && tests.iter().all(|s| test_pass(s).unwrap_or(false)),
        "round_tests_incomplete"
    );
    let (_, verdict, filtered) = source_review(ctx)?;
    if ctx.step.kind == "review" && !filtered.blocking.is_empty() {
        return Ok(RoundDecision::Confirm);
    }
    let (confirmed, unclear) = if ctx.step.kind == "confirm" {
        let result: ConfirmResult =
            serde_json::from_value(ctx.step.body["adoption"]["confirmation"].clone())?;
        result.validate(&filtered.blocking)?;
        (
            result
                .labels
                .iter()
                .filter(|l| l.label == Label::Confirmed)
                .count(),
            result
                .labels
                .iter()
                .filter(|l| l.label == Label::Unclear)
                .count(),
        )
    } else {
        ensure!(ctx.step.kind == "review", "round_result_kind");
        (0, 0)
    };
    rounds::decide(
        true,
        Some(&verdict.verdict),
        confirmed,
        unclear,
        ctx.run.round,
        ctx.max,
    )
}

pub(crate) fn round_fix_input(ctx: &RoundContext) -> Result<FixInput> {
    ensure!(
        round_action(ctx)? == RoundDecision::Fix,
        "fix_not_authorized"
    );
    if ctx.step.kind == "test" {
        let e = &ctx.step.body["exit"];
        // A bounded summary of stored observations; raw output and logs never enter this input.
        Ok(FixInput::TestFailure {
            command_id: ctx.step.body["command_id"]
                .as_str()
                .context("test command missing")?
                .into(),
            exit_code: e["exit_code"].as_i64(),
            signal: e["signal"].as_i64(),
            timed_out: e["timed_out"].as_bool().context("test timeout missing")?,
            overflow: e["overflow"].as_bool().context("test overflow missing")?,
        })
    } else {
        let (_, _, filtered) = source_review(ctx)?;
        let result: ConfirmResult =
            serde_json::from_value(ctx.step.body["adoption"]["confirmation"].clone())?;
        result.validate(&filtered.blocking)?;
        Ok(FixInput::Findings {
            findings: filtered
                .blocking
                .into_iter()
                .filter(|f| {
                    result
                        .labels
                        .iter()
                        .any(|l| l.id == f.id && l.label == Label::Confirmed)
                })
                .collect(),
        })
    }
}

fn nonce_check(tx: &Transaction<'_>, step: &PipelineStep, key: &str) -> Result<()> {
    let nonce = step.body[key].as_str().context("review_nonce_invalid")?;
    wire::validate_nonce(nonce)?;
    let reused: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM pipeline_steps WHERE (lower(json_extract(body,'$.confirm_nonce'))=lower(?1) OR lower(json_extract(body,'$.fix_nonce'))=lower(?1) OR lower(json_extract(body,'$.review_nonce'))=lower(?1)) AND NOT(run_id=?2 AND seq=?3 AND attempt=?4))", params![nonce,step.run_id,step.seq,step.attempt], |r| r.get(0))?;
    ensure!(!reused, "review_nonce_reused");
    Ok(())
}

/// §7.2: a catalog that no longer verifies neither plans nor runs a test; the step is held.
fn hold_profile_changed(
    tx: &Transaction<'_>,
    owner: &RuntimeOwner,
    cursor: &PipelineCursor,
) -> Result<()> {
    let (mut run, mut step) = cursor_rows(tx, owner, cursor, true)?;
    match step.state.as_str() {
        "planned" | "exited" => {
            step.state = "held".into();
            step.body["reason"] = json!("tests_profile_changed");
            if step.cookie.is_none() {
                step.body["death_confirmed"] = json!(true);
            }
            write_step(tx, owner, &step)?;
            if step.kind == "test" {
                record_test_cleanup(tx, &run.id)?;
            }
            held(tx, owner, &mut run, &step, "tests_profile_changed")
        }
        "adopted" if matches!(step.kind.as_str(), "implement" | "fix") => {
            // Planning the re-test holds it at insertion; its placeholder command never runs.
            let retest = PipelineStep {
                run_id: run.id.clone(),
                seq: step.seq + 1,
                attempt: 1,
                kind: "test".into(),
                provider: None,
                owner_epoch: owner.epoch,
                round: run.round,
                state: "planned".into(),
                version: 0,
                cookie: None,
                pid: None,
                birth: None,
                step_base: run.head_commit.clone(),
                input_sha256: crate::execution::verification::hash(b""),
                result_commit: None,
                result_tree: None,
                body: json!({"command_index":0,"command_id":"tests_profile_changed"}),
            };
            insert_step(tx, owner, &retest)
        }
        _ => bail!("round_context_changed"),
    }
}

/// Every round boundary reads its context here: a catalog mismatch is held and committed
/// in this transaction before the error returns, so nothing later runs or is adopted.
fn checked_context<'c>(
    tx: Transaction<'c>,
    owner: &RuntimeOwner,
    cursor: &PipelineCursor,
) -> Result<(Transaction<'c>, RoundContext)> {
    match context(&tx, owner, cursor) {
        Ok(ctx) => Ok((tx, ctx)),
        Err(error) if error.to_string() == "tests_profile_changed" => {
            hold_profile_changed(&tx, owner, cursor)?;
            tx.commit()?;
            Err(error)
        }
        Err(error) => Err(error),
    }
}

impl Store {
    pub(crate) fn pipeline_fix_input(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
    ) -> Result<FixInput> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (tx, mut ctx) = checked_context(tx, owner, cursor)?;
        ensure!(
            ctx.step.kind == "fix" && ctx.step.state == "planned",
            "fix_context_changed"
        );
        let planned = ctx.step.clone();
        ctx.step = ctx
            .history
            .iter()
            .find(|s| s.seq + 1 == planned.seq && s.state == "adopted")
            .context("fix_source_missing")?
            .clone();
        ensure!(
            ctx.step.round + 1 == ctx.run.round,
            "invalid pipeline round"
        );
        ctx.run.round = ctx.step.round;
        let input = round_fix_input(&ctx)?;
        ensure!(
            planned.body["fix_input"] == serde_json::to_value(&input)?
                && planned.body["fix_input_sha256"]
                    == crate::execution::verification::hash(&serde_json::to_vec(&input)?)
                && planned.body["confirmed_ids"] == json!(input.confirmed_ids()),
            "fix_input_changed"
        );
        nonce_check(&tx, &planned, "fix_nonce")?;
        let prompt = rounds::fix_prompt(
            planned.body["fix_nonce"]
                .as_str()
                .context("review_nonce_invalid")?,
            &input,
        )?;
        ensure!(
            crate::execution::verification::hash(&prompt) == planned.input_sha256,
            "fix_input_changed"
        );
        tx.commit()?;
        Ok(input)
    }

    /// A catalog mismatch holds the step in this transaction before the error returns.
    pub(crate) fn pipeline_round_context(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
    ) -> Result<RoundContext> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (tx, ctx) = checked_context(tx, owner, cursor)?;
        tx.commit()?;
        Ok(ctx)
    }
    pub(crate) fn pipeline_record_filter(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        filtered: &FilteredReview,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (tx, mut ctx) = checked_context(tx, owner, cursor)?;
        ensure!(
            ctx.step.kind == "review"
                && ctx.step.state == "exited"
                && ctx.step.body["round_result_validated"] == true,
            "round_filter_state"
        );
        let result: ReviewResult =
            serde_json::from_value(ctx.step.body["adoption"]["verdict"].clone())?;
        validate_filter(&result, ctx.threshold, filtered)?;
        ensure!(ctx.step.body["filtering"].is_null(), "round_filter_exists");
        ctx.step.body["filtering"] = serde_json::to_value(filtered)?;
        ensure!(step_body_fits(&ctx.step.body)?, "confirm_storage_limit");
        write_step(&tx, owner, &ctx.step)?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn pipeline_confirm_preflight(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        tree: &str,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (tx, mut ctx) = checked_context(tx, owner, cursor)?;
        ensure!(
            ctx.step.kind == "confirm" && ctx.step.state == "planned" && valid_oid(tree),
            "confirm_preflight_changed"
        );
        source_review(&ctx)?;
        nonce_check(&tx, &ctx.step, "confirm_nonce")?;
        ctx.step.body["confirm_nonce_verified"] = ctx.step.body["confirm_nonce"].clone();
        ctx.step.body["snapshot_tree"] = json!(tree);
        write_step(&tx, owner, &ctx.step)?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn pipeline_round_nonce_check(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        key: &str,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (tx, ctx) = checked_context(tx, owner, cursor)?;
        nonce_check(&tx, &ctx.step, key)?;
        tx.commit()?;
        Ok(())
    }

    /// Adopt-and-plan or exited-to-held is one transaction with a freshly computed decision.
    pub(crate) fn pipeline_apply_round(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        next: Option<&PipelineStep>,
    ) -> Result<RoundDecision> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (tx, mut ctx) = checked_context(tx, owner, cursor)?;
        let action = round_action(&ctx)?;
        if let RoundDecision::Hold(reason) = action {
            ensure!(
                next.is_none() && ctx.step.state == "exited",
                "round_hold_requires_exited"
            );
            ctx.step.state = "held".into();
            ctx.step.body["reason"] = json!(reason);
            write_step(&tx, owner, &ctx.step)?;
            if ctx.step.kind == "test" {
                record_test_cleanup(&tx, &ctx.run.id)?;
            }
            held(&tx, owner, &mut ctx.run, &ctx.step, reason)?;
        } else {
            let next = next.context("round_next_missing")?;
            validate_next(&tx, &ctx, &action, next)?;
            if ctx.step.state == "exited" {
                ctx.step.state = "adopted".into();
                write_step(&tx, owner, &ctx.step)?;
            }
            if action == RoundDecision::Fix {
                ctx.run.round += 1;
                write_run(&tx, owner, &ctx.run)?;
            }
            insert_step(&tx, owner, next)?;
        }
        tx.commit()?;
        Ok(action)
    }
}

fn validate_next(
    tx: &Transaction<'_>,
    ctx: &RoundContext,
    action: &RoundDecision,
    next: &PipelineStep,
) -> Result<()> {
    let kind = match action {
        RoundDecision::Test(_) => "test",
        RoundDecision::Review => "review",
        RoundDecision::Confirm => "confirm",
        RoundDecision::Fix => "fix",
        RoundDecision::Release => "release",
        RoundDecision::Hold(_) => bail!("round_next_unexpected"),
    };
    ensure!(
        next.kind == kind
            && next.run_id == ctx.run.id
            && next.seq == ctx.step.seq + 1
            && next.attempt == 1
            && next.step_base == ctx.run.head_commit
            && next.round == ctx.run.round + u32::from(kind == "fix"),
        "round_next_changed"
    );
    reserved_fields(next)?;
    if kind == "fix" {
        let input = round_fix_input(ctx)?;
        ensure!(
            next.body["fix_input"] == serde_json::to_value(&input)?
                && next.body["fix_input_sha256"]
                    == crate::execution::verification::hash(&serde_json::to_vec(&input)?)
                && next.body["confirmed_ids"] == json!(input.confirmed_ids()),
            "fix_input_changed"
        );
        nonce_check(tx, next, "fix_nonce")?;
        let prompt = rounds::fix_prompt(
            next.body["fix_nonce"]
                .as_str()
                .context("review_nonce_invalid")?,
            &input,
        )?;
        ensure!(
            next.input_sha256 == crate::execution::verification::hash(&prompt),
            "fix_input_changed"
        );
    }
    if kind == "confirm" {
        ensure!(
            next.body["review_seq"] == ctx.step.seq,
            "confirm_review_changed"
        );
        nonce_check(tx, next, "confirm_nonce")?;
    }
    if let RoundDecision::Test(index) = action {
        ensure!(
            next.body["command_index"] == *index
                && next.body["command_id"] == ctx.profile.proposal.commands[*index].id,
            "round_tests_incomplete"
        );
    }
    Ok(())
}

pub(super) fn confirm_adoption(
    tx: &Transaction<'_>,
    run: &PipelineRun,
    step: &PipelineStep,
    evidence: &Value,
) -> Result<()> {
    ensure!(
        step.body["exit"]["code"] == 0 && step.body["provider_outcome"] == "success",
        "confirm_agent_failed"
    );
    ensure!(
        evidence["snapshot_tree"].is_string()
            && evidence["snapshot_tree"] == step.body["snapshot_tree"],
        "review_snapshot_changed"
    );
    ensure!(
        evidence["result_sha256"]
            .as_str()
            .is_some_and(|s| s.len() == 64)
            && evidence["result_sha256"] == step.body["exit"]["last"]["sha256"],
        "confirm_output_changed"
    );
    let seq = step.body["review_seq"]
        .as_u64()
        .context("confirm_review_missing")?;
    let review = tx.query_row("SELECT * FROM pipeline_steps WHERE run_id=?1 AND seq=?2 AND kind='review' AND state='adopted'", params![run.id,seq], step_row)?;
    ensure!(
        review.round == run.round && review.step_base == step.step_base,
        "confirm_review_changed"
    );
    let filtered: FilteredReview = serde_json::from_value(review.body["filtering"].clone())?;
    let result: ConfirmResult =
        serde_json::from_value(evidence["confirmation"].clone()).context("confirm_schema")?;
    result.validate(&filtered.blocking)
}
