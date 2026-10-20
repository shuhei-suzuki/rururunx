//! Pure transactional pipeline storage. No process, Git or filesystem operations.
use super::*;
use crate::execution::RuntimeOwner;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
mod review;

pub const PIPELINE_TABLES: &[&str] = &[
    "goal_execution_profiles",
    "pipeline_runs",
    "pipeline_steps",
    "pipeline_recoveries",
    "pipeline_ref_journal",
    "pipeline_qualifications",
    "pipeline_goal_completions",
    "pipeline_limit_wait",
    "pipeline_test_cleanup",
];

#[cfg(all(test, target_os = "macos"))]
thread_local! { static WAIT_CLOCK: std::cell::Cell<Option<i64>> = const { std::cell::Cell::new(None) }; }
fn pipeline_now_ms() -> i64 {
    #[cfg(all(test, target_os = "macos"))]
    if let Some(now) = WAIT_CLOCK.with(|clock| clock.get()) {
        return now;
    }
    now_ms()
}
#[cfg(all(test, target_os = "macos"))]
fn with_pipeline_clock<T>(now: i64, run: impl FnOnce() -> T) -> T {
    struct Reset(Option<i64>);
    impl Drop for Reset {
        fn drop(&mut self) {
            WAIT_CLOCK.with(|clock| clock.set(self.0));
        }
    }
    let _reset = Reset(WAIT_CLOCK.with(|clock| clock.replace(Some(now))));
    run()
}

pub(in crate::state) fn install_schema(tx: &Transaction<'_>) -> Result<()> {
    tx.execute_batch(include_str!("pipeline.sql"))?;
    let collisions = [
        "goal_id=NEW.goal_id",
        "id=NEW.id OR (kind='task' AND NEW.kind='task' AND task_id=NEW.task_id) OR (kind='goal' AND NEW.kind='goal' AND goal_id=NEW.goal_id) OR worktree=NEW.worktree OR branch=NEW.branch",
        "(run_id=NEW.run_id AND seq=NEW.seq AND attempt=NEW.attempt) OR (run_id=NEW.run_id AND state IN ('planned','launching','running','exited','committed') AND NEW.state IN ('planned','launching','running','exited','committed'))",
        "id=NEW.id OR request_id=NEW.request_id",
        "id=NEW.id OR (kind='goal_create' AND NEW.kind='goal_create' AND goal_id=NEW.goal_id) OR (kind='task_release' AND NEW.kind='task_release' AND run_id=NEW.run_id) OR (state IN ('pending','held') AND NEW.state IN ('pending','held') AND ref_name=NEW.ref_name)",
        "owner_epoch=NEW.owner_epoch AND subject=NEW.subject",
        "goal_id=NEW.goal_id OR goal_check_run=NEW.goal_check_run",
        "singleton=NEW.singleton",
        "run_id=NEW.run_id AND seq=NEW.seq AND attempt=NEW.attempt",
    ];
    for (table, collision) in PIPELINE_TABLES.iter().zip(collisions) {
        // BEFORE INSERT sees an unspecified rowid for implicit allocation (-1 in
        // bundled SQLite, pinned by b2_fix2_implicit_rowid). Check positivity AFTER
        // allocation so that this sentinel can never collide with a stored row.
        tx.execute_batch(&format!(
            "CREATE TRIGGER {table}_no_replace BEFORE INSERT ON {table} WHEN EXISTS(SELECT 1 FROM {table} WHERE ({collision}) OR rowid=NEW.rowid) BEGIN SELECT RAISE(ABORT,'pipeline row cannot be replaced'); END;
             CREATE TRIGGER {table}_positive_rowid AFTER INSERT ON {table} WHEN NEW.rowid<=0 BEGIN SELECT RAISE(ABORT,'pipeline rowid must be positive'); END;
             CREATE TRIGGER {table}_no_delete BEFORE DELETE ON {table} BEGIN SELECT RAISE(ABORT,'pipeline history retained'); END;"
        ))?;
        for action in ["INSERT", "UPDATE", "DELETE"] {
            tx.execute_batch(&format!("CREATE TRIGGER writer_{table}_{action} BEFORE {action} ON {table} WHEN rrx_writer_contract_version()<>12 BEGIN SELECT RAISE(ABORT,'incompatible rrx writer contract'); END;"))?;
        }
        if *table == "pipeline_test_cleanup" {
            continue; // Run ownership fences its independent maintenance writes below.
        }
        if matches!(
            *table,
            "pipeline_runs" | "pipeline_steps" | "pipeline_ref_journal" | "pipeline_limit_wait"
        ) {
            for action in ["INSERT", "UPDATE"] {
                tx.execute_batch(&format!("CREATE TRIGGER {table}_epoch_{action} BEFORE {action} ON {table} WHEN NEW.owner_epoch IS NOT (SELECT epoch FROM runtime_epoch WHERE singleton=1) BEGIN SELECT RAISE(ABORT,'pipeline owner retired'); END;"))?;
            }
        } else {
            let epoch = match *table {
                "goal_execution_profiles" | "pipeline_recoveries" => "accepted_epoch",
                "pipeline_goal_completions" => "completed_epoch",
                _ => "owner_epoch",
            };
            tx.execute_batch(&format!("CREATE TRIGGER {table}_epoch_INSERT BEFORE INSERT ON {table} WHEN NEW.{epoch} IS NOT (SELECT epoch FROM runtime_epoch WHERE singleton=1) BEGIN SELECT RAISE(ABORT,'pipeline record owner retired'); END;
                CREATE TRIGGER {table}_no_update BEFORE UPDATE ON {table} BEGIN SELECT RAISE(ABORT,'pipeline record immutable'); END;"))?;
        }
        if matches!(*table, "pipeline_steps" | "pipeline_recoveries") {
            for action in ["INSERT", "UPDATE"] {
                tx.execute_batch(&format!("CREATE TRIGGER {table}_run_epoch_{action} BEFORE {action} ON {table} WHEN (SELECT owner_epoch FROM pipeline_runs WHERE id=NEW.run_id) IS NOT (SELECT epoch FROM runtime_epoch WHERE singleton=1) BEGIN SELECT RAISE(ABORT,'pipeline run owner retired'); END;"))?;
            }
        }
    }
    for action in ["INSERT", "UPDATE"] {
        tx.execute_batch(&format!("CREATE TRIGGER cleanup_writer_{action} BEFORE {action} ON pipeline_test_cleanup WHEN rrx_writer_contract_version()<>12 OR (SELECT owner_epoch FROM pipeline_runs WHERE id=NEW.run_id) IS NOT (SELECT epoch FROM runtime_epoch WHERE singleton=1) BEGIN SELECT RAISE(ABORT,'test cleanup owner retired'); END;"))?;
    }
    Ok(())
}

/// Same instance/epoch check as state/runtime/goals.rs::owner_current.
fn pipeline_owner_current(tx: &Transaction<'_>, owner: &RuntimeOwner) -> Result<()> {
    let actual: (String, u64) = tx.query_row(
        "SELECT instance_id,epoch FROM runtime_epoch WHERE singleton=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    ensure!(
        actual == (owner.instance.clone(), owner.epoch) && owner.epoch > 0,
        "pipeline owner retired"
    );
    Ok(())
}

fn pipeline_startup_current(tx: &Transaction<'_>, owner: &RuntimeOwner) -> Result<()> {
    ensure!(
        owner.execution_mode() == crate::config::ExecutionMode::CliPipeline,
        "pipeline requires CliPipeline owner"
    );
    {
        for subject in ["platform", "mode_quiescence", "codex", "claude"] {
            let qualified: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM pipeline_qualifications WHERE owner_epoch=?1 AND subject=?2 AND state='qualified')", params![owner.epoch, subject], |r| r.get(0))?;
            ensure!(qualified, "pipeline startup Q is not current");
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PipelineRun {
    pub id: String,
    pub kind: String,
    pub task_id: Option<TaskId>,
    pub goal_id: GoalId,
    pub project_id: ProjectId,
    pub owner_epoch: u64,
    pub version: u64,
    pub state: String,
    pub round: u32,
    pub base_commit: String,
    pub head_commit: String,
    pub worktree: Option<String>,
    pub branch: Option<String>,
    pub body: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PipelineStep {
    pub run_id: String,
    pub seq: u32,
    pub attempt: u32,
    pub kind: String,
    pub provider: Option<String>,
    pub owner_epoch: u64,
    pub round: u32,
    pub state: String,
    pub version: u64,
    pub cookie: Option<String>,
    pub pid: Option<u32>,
    pub birth: Option<String>,
    pub step_base: String,
    pub input_sha256: String,
    pub result_commit: Option<String>,
    pub result_tree: Option<String>,
    pub body: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PipelineJournal {
    pub id: String,
    pub goal_id: GoalId,
    pub project_id: ProjectId,
    pub run_id: Option<String>,
    pub kind: String,
    pub ref_name: String,
    pub old_commit: String,
    pub new_commit: String,
    pub owner_epoch: u64,
    pub state: String,
    pub version: u64,
    pub body: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PipelineLimitWait {
    pub owner_epoch: u64,
    pub version: u64,
    pub state: String,
    pub not_before: i64,
    pub backoff_ms: i64,
    pub probe_run: Option<String>,
    pub probe_seq: Option<u32>,
    pub probe_attempt: Option<u32>,
    pub body: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PipelineCleanup {
    pub run_id: String,
    pub seq: u32,
    pub attempt: u32,
    pub failures: u32,
    pub not_before: i64,
    pub failure_class: Option<String>,
    pub path: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PipelineView {
    pub pending_cleanup: Vec<PipelineCleanup>,
    pub runs: Vec<PipelineRun>,
    pub steps: Vec<PipelineStep>,
    pub journals: Vec<PipelineJournal>,
    pub provider_wait: Option<PipelineLimitWait>,
    pub native_exhaustion: Vec<Value>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PipelineQuiescenceVerdict {
    Busy,
    CookieDiscoveryRequired,
    Quiescent,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PipelineModeInventory {
    pub to_cli_pipeline: PipelineQuiescenceVerdict,
    pub to_managed_native: PipelineQuiescenceVerdict,
    pub native_drivers: u64,
    pub native_units: u64,
    pub native_phases: u64,
    pub native_leases: u64,
    pub native_effects: u64,
    /// Caller must discover these cookies with full coverage before changing modes.
    pub native_cookies: Vec<String>,
    pub native_exhaustion: Vec<Value>,
    pub pipeline_runs: u64,
    pub pipeline_journals: u64,
    pub pipeline_processes: u64,
    pub pipeline_cookies: Vec<String>,
    pub provider_wait: Option<PipelineLimitWait>,
}

fn stamped(body: &Value, version: u64, state: &str, epoch: u64) -> Result<String> {
    let mut body = body
        .as_object()
        .context("pipeline body must be an object")?
        .clone();
    body.insert("version".into(), json!(version));
    body.insert("state".into(), json!(state));
    body.insert("epoch".into(), json!(epoch));
    Ok(serde_json::to_string(&body)?)
}
fn cas(changed: usize, table: &str, id: &str, expected: u64) -> Result<()> {
    if changed != 1 {
        bail!(StateGuardError::SnapshotChanged {
            table: table.into(),
            id: id.into(),
            expected
        });
    }
    Ok(())
}
fn next_version(version: u64) -> Result<u64> {
    version
        .checked_add(1)
        .filter(|v| *v <= i64::MAX as u64)
        .context("pipeline version exhausted")
}

fn run_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<PipelineRun> {
    Ok(PipelineRun {
        id: r.get(0)?,
        kind: r.get(1)?,
        task_id: r
            .get::<_, Option<String>>(2)?
            .map(|s| s.parse())
            .transpose()
            .map_err(sql_decode)?,
        goal_id: r.get::<_, String>(3)?.parse().map_err(sql_decode)?,
        project_id: r.get::<_, String>(4)?.parse().map_err(sql_decode)?,
        owner_epoch: r.get(5)?,
        version: r.get(6)?,
        state: r.get(7)?,
        round: r.get(8)?,
        base_commit: r.get(9)?,
        head_commit: r.get(10)?,
        worktree: r.get(11)?,
        branch: r.get(12)?,
        body: serde_json::from_str(&r.get::<_, String>(13)?).map_err(sql_decode)?,
    })
}
fn sql_decode(e: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
}
const RUN_COLUMNS: &str = "id,kind,task_id,goal_id,project_id,owner_epoch,version,state,round,base_commit,head_commit,worktree,branch,body";
const STEP_COLUMNS: &str = "run_id,seq,attempt,kind,provider,owner_epoch,round,state,version,cookie,pid,birth,step_base,input_sha256,result_commit,result_tree,body";
const JOURNAL_COLUMNS: &str = "id,goal_id,project_id,run_id,kind,ref_name,old_commit,new_commit,owner_epoch,state,version,body";
fn step_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<PipelineStep> {
    Ok(PipelineStep {
        run_id: r.get(0)?,
        seq: r.get(1)?,
        attempt: r.get(2)?,
        kind: r.get(3)?,
        provider: r.get(4)?,
        owner_epoch: r.get(5)?,
        round: r.get(6)?,
        state: r.get(7)?,
        version: r.get(8)?,
        cookie: r.get(9)?,
        pid: r.get(10)?,
        birth: r.get(11)?,
        step_base: r.get(12)?,
        input_sha256: r.get(13)?,
        result_commit: r.get(14)?,
        result_tree: r.get(15)?,
        body: serde_json::from_str(&r.get::<_, String>(16)?).map_err(sql_decode)?,
    })
}
fn journal_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<PipelineJournal> {
    Ok(PipelineJournal {
        id: r.get(0)?,
        goal_id: r.get::<_, String>(1)?.parse().map_err(sql_decode)?,
        project_id: r.get::<_, String>(2)?.parse().map_err(sql_decode)?,
        run_id: r.get(3)?,
        kind: r.get(4)?,
        ref_name: r.get(5)?,
        old_commit: r.get(6)?,
        new_commit: r.get(7)?,
        owner_epoch: r.get(8)?,
        state: r.get(9)?,
        version: r.get(10)?,
        body: serde_json::from_str(&r.get::<_, String>(11)?).map_err(sql_decode)?,
    })
}
fn read_run(c: &Connection, id: &str) -> Result<PipelineRun> {
    Ok(c.query_row(
        &format!("SELECT {RUN_COLUMNS} FROM pipeline_runs WHERE id=?1"),
        [id],
        run_row,
    )?)
}
fn read_step(c: &Connection, run: &str, seq: u32, attempt: u32) -> Result<PipelineStep> {
    Ok(c.query_row(
        &format!(
            "SELECT {STEP_COLUMNS} FROM pipeline_steps WHERE run_id=?1 AND seq=?2 AND attempt=?3"
        ),
        params![run, seq, attempt],
        step_row,
    )?)
}
fn read_wait(c: &Connection) -> Result<Option<PipelineLimitWait>> {
    Ok(c.query_row("SELECT owner_epoch,version,state,not_before,backoff_ms,probe_run,probe_seq,probe_attempt,body FROM pipeline_limit_wait WHERE singleton=1",[],|r|Ok(PipelineLimitWait{owner_epoch:r.get(0)?,version:r.get(1)?,state:r.get(2)?,not_before:r.get(3)?,backoff_ms:r.get(4)?,probe_run:r.get(5)?,probe_seq:r.get(6)?,probe_attempt:r.get(7)?,body:serde_json::from_str(&r.get::<_,String>(8)?).map_err(sql_decode)?})).optional()?)
}
fn pipeline_goal(tx: &Transaction<'_>, run: &PipelineRun, advancing: bool) -> Result<Goal> {
    let mode: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM goal_execution_profiles WHERE goal_id=?1 AND project_id=?2)",
        params![run.goal_id.to_string(), run.project_id.to_string()],
        |r| r.get(0),
    )?;
    ensure!(mode, "ExecutionModeMismatch");
    let goal = runtime::goals::current_goal(tx, run.project_id, run.goal_id)?;
    ensure!(
        goal.project_id == run.project_id,
        "pipeline Goal scope changed"
    );
    if advancing {
        ensure!(
            goal.state == GoalState::Running,
            "pipeline Goal is not Running"
        );
    }
    Ok(goal)
}
fn current_run(
    tx: &Transaction<'_>,
    owner: &RuntimeOwner,
    id: &str,
    expected: u64,
    advancing: bool,
) -> Result<PipelineRun> {
    let run = read_run(tx, id)?;
    cas(
        usize::from(run.version == expected),
        "pipeline_runs",
        id,
        expected,
    )?;
    ensure!(run.owner_epoch == owner.epoch, "pipeline run owner retired");
    pipeline_goal(tx, &run, advancing)?;
    Ok(run)
}
fn write_run(tx: &Transaction<'_>, owner: &RuntimeOwner, run: &PipelineRun) -> Result<()> {
    let version = next_version(run.version)?;
    let body = stamped(&run.body, version, &run.state, owner.epoch)?;
    cas(tx.execute("UPDATE pipeline_runs SET owner_epoch=?1,version=?2,state=?3,round=?4,head_commit=?5,body=?6 WHERE id=?7 AND version=?8",params![owner.epoch,version,run.state,run.round,run.head_commit,body,run.id,run.version])?,"pipeline_runs",&run.id,run.version)
}
fn insert_step(tx: &Transaction<'_>, owner: &RuntimeOwner, step: &PipelineStep) -> Result<()> {
    ensure!(
        step.version == 0 && step.state == "planned",
        "new step requires absent version"
    );
    if step.kind == "review" {
        crate::execution::pipeline::review::scan_strings(&step.body).context("review.body")?;
    }
    let body = stamped(&step.body, 1, "planned", owner.epoch)?;
    tx.execute("INSERT INTO pipeline_steps(run_id,seq,attempt,kind,provider,owner_epoch,round,state,version,cookie,pid,birth,step_base,input_sha256,result_commit,result_tree,body) VALUES(?1,?2,?3,?4,?5,?6,?7,'planned',1,NULL,NULL,NULL,?8,?9,NULL,NULL,?10)",params![step.run_id,step.seq,step.attempt,step.kind,step.provider,owner.epoch,step.round,step.step_base,step.input_sha256,body])?;
    if step.kind == "test" {
        let mut run = read_run(tx, &step.run_id)?;
        let mut stored = read_step(tx, &step.run_id, step.seq, step.attempt)?;
        check_test_profile(tx, owner, &mut run, &mut stored)?;
    }
    Ok(())
}
fn write_step(tx: &Transaction<'_>, owner: &RuntimeOwner, step: &PipelineStep) -> Result<()> {
    if step.kind == "review" {
        crate::execution::pipeline::review::scan_strings(&step.body).context("review.body")?;
    }
    let version = next_version(step.version)?;
    let body = stamped(&step.body, version, &step.state, owner.epoch)?;
    cas(tx.execute("UPDATE pipeline_steps SET owner_epoch=?1,version=?2,state=?3,cookie=?4,pid=?5,birth=?6,result_commit=?7,result_tree=?8,body=?9 WHERE run_id=?10 AND seq=?11 AND attempt=?12 AND version=?13",params![owner.epoch,version,step.state,step.cookie,step.pid,step.birth,step.result_commit,step.result_tree,body,step.run_id,step.seq,step.attempt,step.version])?,"pipeline_steps",&format!("{}/{}/{}",step.run_id,step.seq,step.attempt),step.version)
}
fn write_journal(
    tx: &Transaction<'_>,
    owner: &RuntimeOwner,
    journal: &PipelineJournal,
) -> Result<()> {
    cas(tx.execute("UPDATE pipeline_ref_journal SET owner_epoch=?1,version=?2,state=?3,body=?4 WHERE id=?5 AND version=?6",params![owner.epoch,next_version(journal.version)?,journal.state,serde_json::to_string(&journal.body)?,journal.id,journal.version])?,"pipeline_ref_journal",&journal.id,journal.version)
}

fn native_exhaustion(c: &Connection) -> Result<Vec<Value>> {
    let mut q = c.prepare(
        "SELECT DISTINCT provider,account_key FROM quota_windows ORDER BY provider,account_key",
    )?;
    let accounts = q
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut out = Vec::new();
    for (provider, account) in accounts {
        for observation in super::quotas::windows(c, &provider, &account)? {
            if observation.status == crate::execution::QuotaStatus::Exhausted {
                out.push(json!({"provider":provider,"bucket":observation.bucket,"resets_at":observation.resets_at}));
            }
        }
    }
    Ok(out)
}

/// Independently observed row versions; a missing wait row is expected absence.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineCursor {
    pub run_id: String,
    pub run_version: u64,
    pub seq: u32,
    pub attempt: u32,
    pub step_version: u64,
    pub wait_version: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PipelineTakeover {
    pub runs: Vec<PipelineRun>,
    pub steps: Vec<PipelineStep>,
    pub journals: Vec<PipelineJournal>,
    pub limit_wait: Option<PipelineLimitWait>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineProfile {
    pub policy_sha256: String,
    pub clis_sha256: String,
    pub tests_profile_digest: String,
    pub body: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineQualification {
    pub subject: String,
    pub state: String,
    pub executable_sha256: Option<String>,
    pub version_text: Option<String>,
    pub body: Value,
}

impl PipelineProfile {
    /// Canonical JSON binds execution policy, admitted tests and pinned CLIs.
    pub fn policy_digest(&self) -> Result<String> {
        use sha2::{Digest, Sha256};
        ensure!(
            self.body.is_object(),
            "pipeline execution policy must be an object"
        );
        ensure!(
            self.body["max_fix_rounds"].as_u64().is_some_and(|n| n <= 2),
            "pipeline fix policy missing or invalid"
        );
        pipeline_max_limit_waits(&self.body)?;
        ensure!(
            self.clis_sha256.len() == 64
                && self
                    .clis_sha256
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
            "pipeline CLI pin digest invalid"
        );
        Ok(format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&json!({
                "execution": self.body, "tests_profile_digest": self.tests_profile_digest,
                "clis_sha256": self.clis_sha256
            }))?)
        ))
    }
}
fn pipeline_max_limit_waits(body: &Value) -> Result<u64> {
    let max = match body.get("max_limit_waits") {
        None => 3,
        Some(value) => value.as_u64().context("pipeline limit policy invalid")?,
    };
    ensure!(max <= 6, "pipeline limit policy exceeds bound");
    Ok(max)
}

fn check_wait(
    tx: &Transaction<'_>,
    owner: &RuntimeOwner,
    expected: Option<u64>,
) -> Result<Option<PipelineLimitWait>> {
    let wait = read_wait(tx)?;
    cas(
        usize::from(wait.as_ref().map(|w| w.version) == expected),
        "pipeline_limit_wait",
        "1",
        expected.unwrap_or(0),
    )?;
    if let Some(w) = &wait {
        ensure!(w.owner_epoch == owner.epoch, "pipeline wait owner retired");
    }
    Ok(wait)
}
fn write_wait(tx: &Transaction<'_>, owner: &RuntimeOwner, wait: &PipelineLimitWait) -> Result<()> {
    let version = next_version(wait.version)?;
    let body = stamped(&wait.body, version, &wait.state, owner.epoch)?;
    if wait.version == 0 {
        tx.execute("INSERT INTO pipeline_limit_wait(singleton,owner_epoch,version,state,not_before,backoff_ms,probe_run,probe_seq,probe_attempt,body) VALUES(1,?1,1,?2,?3,?4,?5,?6,?7,?8)",params![owner.epoch,wait.state,wait.not_before,wait.backoff_ms,wait.probe_run,wait.probe_seq,wait.probe_attempt,body])?;
    } else {
        cas(tx.execute("UPDATE pipeline_limit_wait SET owner_epoch=?1,version=?2,state=?3,not_before=?4,backoff_ms=?5,probe_run=?6,probe_seq=?7,probe_attempt=?8,body=?9 WHERE singleton=1 AND version=?10",params![owner.epoch,version,wait.state,wait.not_before,wait.backoff_ms,wait.probe_run,wait.probe_seq,wait.probe_attempt,body,wait.version])?,"pipeline_limit_wait","1",wait.version)?;
    }
    Ok(())
}
fn settle_wait(
    tx: &Transaction<'_>,
    owner: &RuntimeOwner,
    wait: Option<PipelineLimitWait>,
    step: &PipelineStep,
) -> Result<()> {
    let probe = wait.as_ref().is_some_and(|w| {
        w.state == "probing"
            && w.probe_run.as_deref() == Some(&step.run_id)
            && w.probe_seq == Some(step.seq)
            && w.probe_attempt == Some(step.attempt)
    });
    let outcome = step.body.get("provider_outcome").and_then(Value::as_str);
    let clear = probe
        && !matches!(step.state.as_str(), "planned" | "launching" | "running")
        && outcome == Some("success");
    let extend = (step.state == "exited" && outcome == Some("limit"))
        || (probe && !matches!(step.state.as_str(), "planned" | "launching" | "running"));
    if !clear && !extend {
        return Ok(());
    }
    let mut w = wait.unwrap_or(PipelineLimitWait {
        owner_epoch: owner.epoch,
        version: 0,
        state: "waiting".into(),
        not_before: 1,
        backoff_ms: 60000,
        probe_run: None,
        probe_seq: None,
        probe_attempt: None,
        body: json!({}),
    });
    w.body = json!({"run":step.run_id,"seq":step.seq,"attempt":step.attempt,"outcome":outcome});
    if clear {
        w.state = "clear".into();
        w.backoff_ms = 60000;
    } else {
        w.state = "waiting".into();
        w.not_before = w.not_before.max(
            pipeline_now_ms()
                .checked_add(w.backoff_ms)
                .context("pipeline wait deadline overflow")?,
        );
        w.backoff_ms = (w.backoff_ms * 2).min(1800000);
    }
    w.probe_run = None;
    w.probe_seq = None;
    w.probe_attempt = None;
    write_wait(tx, owner, &w)
}
fn require_last_step(c: &Connection, step: &PipelineStep) -> Result<()> {
    let later: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM pipeline_steps WHERE run_id=?1 AND (seq,attempt)>(?2,?3))",
        params![step.run_id, step.seq, step.attempt],
        |r| r.get(0),
    )?;
    ensure!(!later, "pipeline cursor is not last step");
    Ok(())
}
fn cursor_rows(
    tx: &Transaction<'_>,
    owner: &RuntimeOwner,
    cursor: &PipelineCursor,
    advancing: bool,
) -> Result<(PipelineRun, PipelineStep)> {
    let run = current_run(tx, owner, &cursor.run_id, cursor.run_version, advancing)?;
    let step = read_step(tx, &cursor.run_id, cursor.seq, cursor.attempt)?;
    cas(
        usize::from(step.version == cursor.step_version),
        "pipeline_steps",
        &cursor.run_id,
        cursor.step_version,
    )?;
    ensure!(
        step.owner_epoch == owner.epoch,
        "pipeline step owner retired"
    );
    Ok((run, step))
}
fn held(
    tx: &Transaction<'_>,
    owner: &RuntimeOwner,
    run: &mut PipelineRun,
    step: &PipelineStep,
    reason: &str,
) -> Result<()> {
    ensure!(
        matches!(step.state.as_str(), "held" | "unknown" | "rejected"),
        "held run requires a recoverable step"
    );
    run.state = "held".into();
    run.body["held"] = json!({"reason":reason,"seq":step.seq,"attempt":step.attempt});
    write_run(tx, owner, run)?;
    append_event(
        tx,
        &Scope {
            project_id: run.project_id,
            goal_id: Some(run.goal_id),
            task_id: run.task_id,
        },
        "rrx.private.pipeline.run_held",
        json!({"run":run.id,"reason":reason,"seq":step.seq,"attempt":step.attempt}),
    )
}
fn journal_step_matches(j: &PipelineJournal, run: &PipelineRun, step: &PipelineStep) -> bool {
    run.state == "running"
        && step.state == "running"
        && step.run_id == run.id
        && ((j.kind == "task_release"
            && j.run_id.as_deref() == Some(&run.id)
            && step.kind == "release")
            || (j.kind == "goal_create"
                && j.body["creator_run_id"].as_str() == Some(&run.id)
                && step.kind == "prepare"))
}
fn insert_journal(tx: &Transaction<'_>, owner: &RuntimeOwner, j: &PipelineJournal) -> Result<()> {
    ensure!(
        j.version == 0 && j.state == "pending",
        "new journal requires absent version"
    );
    ensure!(
        j.ref_name == format!("refs/heads/rrx/goal/{}", j.goal_id),
        "pipeline journal ref mismatch"
    );
    ensure!(
        valid_oid(&j.old_commit) && valid_oid(&j.new_commit),
        "pipeline journal OID invalid"
    );
    ensure!(
        j.kind != "goal_create" || j.old_commit == "0".repeat(j.new_commit.len()),
        "pipeline Goal create old is not zero"
    );
    tx.execute("INSERT INTO pipeline_ref_journal(id,goal_id,project_id,run_id,kind,ref_name,old_commit,new_commit,owner_epoch,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,'pending',1,?10)",params![j.id,j.goal_id.to_string(),j.project_id.to_string(),j.run_id,j.kind,j.ref_name,j.old_commit,j.new_commit,owner.epoch,serde_json::to_string(&j.body)?])?;
    Ok(())
}
fn takeover_rows(c: &Connection) -> Result<PipelineTakeover> {
    let mut q = c.prepare(&format!(
        "SELECT {RUN_COLUMNS} FROM pipeline_runs WHERE state IN ('running','held') ORDER BY id"
    ))?;
    let runs = q
        .query_map([], run_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut steps = Vec::new();
    for run in &runs {
        steps.push(c.query_row(&format!("SELECT {STEP_COLUMNS} FROM pipeline_steps WHERE run_id=?1 ORDER BY seq DESC,attempt DESC LIMIT 1"),[&run.id],step_row)?);
    }
    let mut q=c.prepare(&format!("SELECT {JOURNAL_COLUMNS} FROM pipeline_ref_journal WHERE state IN ('pending','held') ORDER BY id"))?;
    let journals = q
        .query_map([], journal_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(PipelineTakeover {
        runs,
        steps,
        journals,
        limit_wait: read_wait(c)?,
    })
}

impl Store {
    /// Preserve the existing ingress and accepted-definition transaction.
    pub fn pipeline_accept_goal(
        &mut self,
        ingress: &crate::runtime::control::HumanIngress,
        request: &crate::runtime::control::ControlRequest,
        validated: &crate::runtime::goal::ValidatedGoalPlan,
        profile: &PipelineProfile,
    ) -> Result<crate::runtime::control::ControlResponse> {
        self.accept_runtime_goal_with_pipeline(
            ingress,
            request,
            validated,
            &profile.policy_sha256,
            Some(profile),
        )
    }
    /// Coherent read-only inventory; cookie discovery remains the caller's responsibility.
    pub fn pipeline_mode_inventory(&self) -> Result<PipelineModeInventory> {
        self.pipeline_mode_inventory_selected(None)
    }
    /// Startup inspects the opposite mode only; own-mode recovery is independent.
    pub fn pipeline_mode_inventory_for(
        &self,
        mode: crate::config::ExecutionMode,
    ) -> Result<PipelineModeInventory> {
        self.pipeline_mode_inventory_selected(Some(mode))
    }
    fn pipeline_mode_inventory_selected(
        &self,
        mode: Option<crate::config::ExecutionMode>,
    ) -> Result<PipelineModeInventory> {
        let read_native = mode != Some(crate::config::ExecutionMode::ManagedNative);
        let read_pipeline = mode != Some(crate::config::ExecutionMode::CliPipeline);
        let tx = self.connection.unchecked_transaction()?;
        let count = |sql: &str| -> Result<u64> { Ok(tx.query_row(sql, [], |r| r.get(0))?) };
        let native_count = |sql| if read_native { count(sql) } else { Ok(0) };
        let pipeline_count = |sql| if read_pipeline { count(sql) } else { Ok(0) };
        let cookies = |sql: &str| -> Result<Vec<String>> {
            let mut q = tx.prepare(sql)?;
            let rows = q
                .query_map([], |r| r.get(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            ensure!(rows.len() <= 4096, "mode cookie inventory exceeds bound");
            Ok(rows)
        };
        // Cleanup identifies a Unit by id; its cookie is in the validated body,
        // not an indexed column. Missing Units or unusable cookies cannot prove
        // quiescence, even when all indexed open flags have already been closed.
        let native_cookies = if read_native {
            let mut q =
                tx.prepare("SELECT unit_id FROM cleanup_jobs ORDER BY unit_id LIMIT 4097")?;
            let ids = q
                .query_map([], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            ensure!(ids.len() <= 4096, "mode cookie inventory exceeds bound");
            ids.into_iter()
                .map(|id| {
                    let unit = unit_tx(&tx, id.parse()?)?;
                    Uuid::parse_str(&unit.cookie).context("native cleanup cookie unavailable")?;
                    Ok(unit.cookie)
                })
                .collect::<Result<Vec<_>>>()?
        } else {
            Vec::new()
        };
        let mut out = PipelineModeInventory {
            to_cli_pipeline: PipelineQuiescenceVerdict::Busy,
            to_managed_native: PipelineQuiescenceVerdict::Busy,
            native_drivers: native_count(
                "SELECT count(*) FROM task_drivers WHERE state='driving'",
            )?,
            native_units: native_count(
                "SELECT count(*) FROM execution_units WHERE native_effects_open=1 OR result_finalization_open=1",
            )?,
            native_phases: native_count(
                "SELECT count(*) FROM managed_phase_operations WHERE phase_open=1",
            )?,
            native_leases: native_count("SELECT count(*) FROM quota_leases WHERE active=1")?,
            native_effects: native_count(
                "SELECT count(*) FROM managed_effects WHERE state IN ('pending','unknown')",
            )?,
            native_cookies,
            native_exhaustion: if read_native {
                native_exhaustion(&tx)?
            } else {
                Vec::new()
            },
            pipeline_runs: pipeline_count(
                "SELECT count(*) FROM pipeline_runs WHERE state IN ('running','held')",
            )?,
            pipeline_journals: pipeline_count(
                "SELECT count(*) FROM pipeline_ref_journal WHERE state IN ('pending','held')",
            )?,
            pipeline_processes: pipeline_count(
                "SELECT count(*) FROM pipeline_steps WHERE kind NOT IN ('prepare','release') AND (state IN ('launching','running','unknown') OR (state='held' AND json_extract(body,'$.death_confirmed') IS NOT 1))",
            )?,
            // Terminal death-confirmed steps cannot own a live process: only exit,
            // never-launched intent or audited recovery can establish this fact,
            // and triggers prevent rewriting it later (including takeover).
            pipeline_cookies: if read_pipeline {
                cookies(
                    "SELECT cookie FROM pipeline_steps WHERE cookie IS NOT NULL AND NOT (state IN ('exited','committed','adopted','rejected','limited','interrupted','held','abandoned') AND json_extract(body,'$.death_confirmed') IS 1) ORDER BY run_id,seq,attempt LIMIT 4097",
                )?
            } else {
                Vec::new()
            },
            provider_wait: if read_pipeline { read_wait(&tx)? } else { None },
        };
        let verdict = |busy: bool, cookies: &[String]| {
            if busy {
                PipelineQuiescenceVerdict::Busy
            } else if !cookies.is_empty() {
                PipelineQuiescenceVerdict::CookieDiscoveryRequired
            } else {
                PipelineQuiescenceVerdict::Quiescent
            }
        };
        out.to_cli_pipeline = verdict(
            out.native_drivers
                + out.native_units
                + out.native_phases
                + out.native_leases
                + out.native_effects
                > 0
                || !out.native_exhaustion.is_empty(),
            &out.native_cookies,
        );
        out.to_managed_native = verdict(
            out.pipeline_runs + out.pipeline_journals + out.pipeline_processes > 0
                || out
                    .provider_wait
                    .as_ref()
                    .is_some_and(|w| w.state != "clear"),
            &out.pipeline_cookies,
        );
        tx.commit()?;
        Ok(out)
    }

    pub fn pipeline_takeover_view(&self) -> Result<PipelineTakeover> {
        let tx = self.connection.unchecked_transaction()?;
        let out = takeover_rows(&tx)?;
        tx.commit()?;
        Ok(out)
    }
    /// Compare the complete observed inventory, then fence every retained row atomically.
    pub fn pipeline_take_over(
        &mut self,
        owner: &RuntimeOwner,
        expected: &PipelineTakeover,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        ensure!(
            takeover_rows(&tx)? == *expected,
            "pipeline takeover inventory changed"
        );
        for run in &expected.runs {
            ensure!(
                run.owner_epoch < owner.epoch,
                "pipeline takeover requires older owner"
            );
            pipeline_goal(&tx, run, false)?;
            write_run(&tx, owner, run)?;
        }
        for step in &expected.steps {
            write_step(&tx, owner, step)?;
        }
        for journal in &expected.journals {
            write_journal(&tx, owner, journal)?;
        }
        if let Some(wait) = &expected.limit_wait {
            write_wait(&tx, owner, wait)?;
        }
        for run in &expected.runs {
            append_event(
                &tx,
                &Scope {
                    project_id: run.project_id,
                    goal_id: Some(run.goal_id),
                    task_id: run.task_id,
                },
                "rrx.private.pipeline.taken_over",
                json!({"run":run.id,"epoch":owner.epoch}),
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Claim an absent run and first step under current Goal and Project facts.
    pub fn pipeline_claim_run(
        &mut self,
        owner: &RuntimeOwner,
        run: &PipelineRun,
        expected_goal: u64,
        first: &PipelineStep,
        create: Option<&PipelineJournal>,
    ) -> Result<PipelineRun> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        pipeline_startup_current(&tx, owner)?;
        ensure!(
            run.version == 0
                && run.state == "running"
                && first.run_id == run.id
                && first.seq == 1
                && first.attempt == 1,
            "invalid pipeline claim"
        );
        let goal = pipeline_goal(&tx, run, true)?;
        cas(
            usize::from(goal.version == expected_goal),
            "goals",
            &run.goal_id.to_string(),
            expected_goal,
        )?;
        ensure_project_registered(&tx, run.project_id)?;
        let occupied:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM pipeline_runs WHERE project_id=?1 AND state IN ('running','held'))",[run.project_id.to_string()],|r|r.get(0))?;
        ensure!(!occupied, "pipeline Project slot occupied");
        if let Some(task) = run.task_id {
            let create_state: Option<String> = tx.query_row("SELECT state FROM pipeline_ref_journal WHERE goal_id=?1 AND kind='goal_create'", [run.goal_id.to_string()], |r| r.get(0)).optional()?;
            let has_create = create_state.is_some();
            ensure!(
                !has_create || create_state.as_deref() == Some("done"),
                "pipeline Goal create journal is not done"
            );
            ensure!(
                has_create != create.is_some(),
                "first pipeline Task requires exactly one Goal create journal"
            );
            let current: Task =
                read_tx(&tx, "tasks", &task.to_string())?.context("pipeline Task missing")?;
            ensure!(
                current.project_id == run.project_id
                    && current.goal_id == run.goal_id
                    && goal.dag.nodes.contains(&task),
                "pipeline Task scope mismatch"
            );
            ensure!(first.kind == "prepare", "Task run must begin with prepare");
            for edge in goal
                .dag
                .edges
                .iter()
                .filter(|e| e.hard && e.dependent == task)
            {
                let complete:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM pipeline_runs WHERE task_id=?1 AND goal_id=?2 AND project_id=?3 AND state='completed')",params![edge.prerequisite.to_string(),goal.id.to_string(),goal.project_id.to_string()],|r|r.get(0))?;
                ensure!(complete, "pipeline hard prerequisite incomplete");
            }
        } else {
            ensure!(
                run.kind == "goal" && first.kind == "test",
                "Goal run must begin with test"
            );
            let last_release: String = tx.query_row("SELECT j.new_commit FROM pipeline_ref_journal j JOIN pipeline_runs r ON r.id=j.run_id WHERE j.goal_id=?1 AND j.kind='task_release' AND j.state='done' ORDER BY r.rowid DESC LIMIT 1", [goal.id.to_string()], |r| r.get(0))?;
            ensure!(
                run.base_commit == last_release,
                "pipeline Goal base differs from last release"
            );
            for task in &goal.dag.nodes {
                let complete:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM pipeline_runs WHERE task_id=?1 AND goal_id=?2 AND project_id=?3 AND state='completed')",params![task.to_string(),goal.id.to_string(),goal.project_id.to_string()],|r|r.get(0))?;
                ensure!(complete, "pipeline Goal has incomplete Tasks");
            }
        }
        let mut body = run.body.clone();
        body["id"] = json!(run.id);
        let body = stamped(&body, 1, "running", owner.epoch)?;
        tx.execute("INSERT INTO pipeline_runs(id,kind,task_id,goal_id,project_id,owner_epoch,version,state,round,base_commit,head_commit,worktree,branch,body) VALUES(?1,?2,?3,?4,?5,?6,1,'running',?7,?8,?9,?10,?11,?12)",params![run.id,run.kind,run.task_id.map(|t|t.to_string()),run.goal_id.to_string(),run.project_id.to_string(),owner.epoch,run.round,run.base_commit,run.head_commit,run.worktree,run.branch,body])?;
        insert_step(&tx, owner, first)?;
        if let Some(j) = create {
            ensure!(
                j.kind == "goal_create"
                    && j.goal_id == run.goal_id
                    && j.project_id == run.project_id
                    && j.run_id.is_none(),
                "Goal create journal mismatch"
            );
            let mut j = j.clone();
            j.body
                .as_object_mut()
                .context("pipeline Goal create body must be an object")?
                .insert("creator_run_id".into(), json!(run.id));
            insert_journal(&tx, owner, &j)?;
        }
        let out = read_run(&tx, &run.id)?;
        tx.commit()?;
        Ok(out)
    }

    pub(crate) fn pipeline_record_qualification(
        &mut self,
        owner: &RuntimeOwner,
        record: &crate::execution::pipeline::QualificationRecord,
    ) -> Result<()> {
        let qualification = record.row();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        tx.execute("INSERT INTO pipeline_qualifications(owner_epoch,subject,state,executable_sha256,version_text,body) VALUES(?1,?2,?3,?4,?5,?6)",params![owner.epoch,qualification.subject,qualification.state,qualification.executable_sha256,qualification.version_text,serde_json::to_string(&qualification.body)?])?;
        tx.commit()?;
        Ok(())
    }

    /// Only the current owner's immutable Q observations are returned.
    pub fn pipeline_qualifications(
        &self,
        owner: &RuntimeOwner,
    ) -> Result<Vec<PipelineQualification>> {
        let tx = self.connection.unchecked_transaction()?;
        pipeline_owner_current(&tx, owner)?;
        let rows = {
            let mut query = tx.prepare("SELECT subject,state,executable_sha256,version_text,body FROM pipeline_qualifications WHERE owner_epoch=?1 ORDER BY subject")?;
            query
                .query_map([owner.epoch], |r| {
                    Ok(PipelineQualification {
                        subject: r.get(0)?,
                        state: r.get(1)?,
                        executable_sha256: r.get(2)?,
                        version_text: r.get(3)?,
                        body: serde_json::from_str(&r.get::<_, String>(4)?).map_err(sql_decode)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        tx.commit()?;
        Ok(rows)
    }

    pub fn pipeline_goal_view(&self, goal: GoalId) -> Result<PipelineView> {
        self.pipeline_view(goal, None)
    }
    pub fn pipeline_task_view(&self, goal: GoalId, task: TaskId) -> Result<PipelineView> {
        self.pipeline_view(goal, Some(task))
    }
    fn pipeline_view(&self, goal: GoalId, task: Option<TaskId>) -> Result<PipelineView> {
        let tx = self.connection.unchecked_transaction()?;
        let mut q=tx.prepare(&format!("SELECT {RUN_COLUMNS} FROM pipeline_runs WHERE goal_id=?1 AND (?2 IS NULL OR task_id=?2) ORDER BY id"))?;
        let runs = q
            .query_map(
                params![goal.to_string(), task.map(|t| t.to_string())],
                run_row,
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(q);
        let mut steps = Vec::new();
        for run in &runs {
            let mut q = tx.prepare(&format!(
                "SELECT {STEP_COLUMNS} FROM pipeline_steps WHERE run_id=?1 ORDER BY seq,attempt"
            ))?;
            steps.extend(
                q.query_map([&run.id], step_row)?
                    .collect::<rusqlite::Result<Vec<_>>>()?,
            );
        }
        let mut q = tx.prepare(&format!(
            "SELECT {JOURNAL_COLUMNS} FROM pipeline_ref_journal WHERE goal_id=?1 ORDER BY id"
        ))?;
        let journals = q
            .query_map([goal.to_string()], journal_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(q);
        let mut pending_cleanup = Vec::new();
        for run in &runs {
            pending_cleanup.extend(cleanup_rows(&tx, &run.id)?);
        }
        let view = PipelineView {
            pending_cleanup,
            runs,
            steps,
            journals,
            provider_wait: read_wait(&tx)?,
            native_exhaustion: native_exhaustion(&tx)?,
        };
        tx.commit()?;
        Ok(view)
    }
}

impl Store {
    pub fn pipeline_release_begin(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        journal: &PipelineJournal,
        approval: &Value,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (run, mut step) = cursor_rows(&tx, owner, cursor, true)?;
        ensure!(
            run.kind == "task"
                && run.state == "running"
                && step.kind == "release"
                && step.state == "planned",
            "pipeline release state changed"
        );
        ensure!(
            journal.kind == "task_release"
                && journal.run_id.as_deref() == Some(&run.id)
                && journal.goal_id == run.goal_id
                && journal.project_id == run.project_id
                && journal.old_commit == run.base_commit
                && journal.new_commit == run.head_commit,
            "pipeline release journal mismatch"
        );
        ensure!(approval.is_object(), "pipeline approval evidence missing");
        insert_journal(&tx, owner, journal)?;
        step.state = "running".into();
        step.body["approval"] = approval.clone();
        write_step(&tx, owner, &step)?;
        tx.commit()?;
        Ok(())
    }

    /// Records the observed ref only; it never performs update-ref.
    pub fn pipeline_release_settle(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        journal_id: &str,
        expected_journal: u64,
        observed_commit: &str,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (mut run, mut step) = cursor_rows(&tx, owner, cursor, false)?;
        ensure!(
            run.state != "abandoned",
            "pipeline abandoned run cannot settle a journal"
        );
        let mut j = tx.query_row(
            &format!("SELECT {JOURNAL_COLUMNS} FROM pipeline_ref_journal WHERE id=?1"),
            [journal_id],
            journal_row,
        )?;
        cas(
            usize::from(j.version == expected_journal),
            "pipeline_ref_journal",
            journal_id,
            expected_journal,
        )?;
        ensure!(
            j.owner_epoch == owner.epoch
                && j.goal_id == run.goal_id
                && j.project_id == run.project_id
                && matches!(j.state.as_str(), "pending" | "held"),
            "pipeline journal owner/scope/state changed"
        );
        ensure!(
            journal_step_matches(&j, &run, &step),
            "pipeline journal step mismatch"
        );
        let goal = pipeline_goal(&tx, &run, false)?;
        if observed_commit == j.new_commit {
            // A held run requires an operator recovery record before completion.
            ensure!(
                run.state == "running",
                "held pipeline release requires recovery"
            );
            j.state = "done".into();
            write_journal(&tx, owner, &j)?;
            if j.kind == "task_release" {
                ensure!(
                    j.run_id.as_deref() == Some(&run.id)
                        && step.kind == "release"
                        && step.state == "running",
                    "pipeline release step mismatch"
                );
                step.state = "adopted".into();
                write_step(&tx, owner, &step)?;
                run.state = "completed".into();
                write_run(&tx, owner, &run)?;
            }
        } else if observed_commit == j.old_commit {
            if goal.state == GoalState::Cancelled {
                j.state = "aborted".into();
                write_journal(&tx, owner, &j)?;
                step.state = "interrupted".into();
                write_step(&tx, owner, &step)?;
            } else if goal.state == GoalState::Running && j.state == "held" {
                j.state = "pending".into();
                write_journal(&tx, owner, &j)?;
            }
        } else {
            ensure!(
                run.state == "running",
                "held pipeline journal requires recovery"
            );
            j.state = "held".into();
            write_journal(&tx, owner, &j)?;
            step.state = "held".into();
            step.body["reason"] = json!("ref_unexpected");
            write_step(&tx, owner, &step)?;
            held(&tx, owner, &mut run, &step, "ref_unexpected")?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Cancel records observations only; live or unknown work keeps the run held.
    pub fn pipeline_cancel_run(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        death: Option<&PipelineDeathEvidence>,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (mut run, mut step) = cursor_rows(&tx, owner, cursor, false)?;
        let goal = pipeline_goal(&tx, &run, false)?;
        ensure!(
            goal.state == GoalState::Cancelled && matches!(run.state.as_str(), "running" | "held"),
            "pipeline cancel requires Cancelled Goal"
        );
        let wait = check_wait(&tx, owner, cursor.wait_version)?;
        let mut q = tx.prepare(&format!("SELECT {JOURNAL_COLUMNS} FROM pipeline_ref_journal WHERE goal_id=?1 AND kind='goal_create' AND state IN ('pending','held')"))?;
        let creates = q
            .query_map([run.goal_id.to_string()], journal_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(q);
        for mut j in creates {
            // begin_launch persists intent in the same write that enters running;
            // all attempts retain it, including interrupted/recovered attempts.
            let prepared: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM pipeline_steps WHERE run_id=?1 AND kind='prepare' AND (state IN ('running','adopted','interrupted') OR json_type(body,'$.intent') IS NOT NULL))",
                [&run.id], |r| r.get(0),
            )?;
            if j.body["creator_run_id"].as_str() == Some(&run.id) && !prepared {
                j.state = "aborted".into();
                write_journal(&tx, owner, &j)?;
            }
        }
        let open_create: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM pipeline_ref_journal WHERE goal_id=?1 AND kind='goal_create' AND state IN ('pending','held'))",
            [run.goal_id.to_string()], |r| r.get(0),
        )?;
        if open_create && step.kind == "prepare" {
            if step.state == "running" {
                // Leave the running prepare usable by observation-only settlement.
                tx.commit()?;
                return Ok(());
            }
            if step.state == "interrupted" {
                // Keep the terminal attempt intact; create a recoverable cancellation target.
                ensure!(step.attempt < 16, "pipeline attempts exhausted");
                step.attempt += 1;
                step.state = "planned".into();
                step.version = 0;
                step.body = json!({});
                insert_step(&tx, owner, &step)?;
                step.version = 1;
            }
            if step.state == "planned" {
                // An earlier attempt may have applied the ref. Recovery must observe it.
                step.state = "held".into();
                step.body["reason"] = json!("cancel_pending");
                write_step(&tx, owner, &step)?;
            }
        }
        match step.state.as_str() {
            "planned" => {
                step.state = "abandoned".into();
                if !matches!(step.kind.as_str(), "prepare" | "release") {
                    step.body["death_confirmed"] = json!(true);
                }
                write_step(&tx, owner, &step)?;
            }
            "exited" | "committed" => {
                step.state = if step.attempt == 16 {
                    "held"
                } else {
                    "interrupted"
                }
                .into();
                step.body["death_confirmed"] = json!(true);
                if step.attempt == 16 {
                    step.body["reason"] = json!("attempt_limit");
                }
                write_step(&tx, owner, &step)?;
            }
            "running" if matches!(step.kind.as_str(), "prepare" | "release") => {
                step.state = "interrupted".into();
                write_step(&tx, owner, &step)?;
            }
            "running" => {
                if let Some(death) = death {
                    death.validate(step.birth.is_some())?;
                    step.state = "exited".into();
                    step.body["death_confirmed"] = json!(true);
                    step.body["death_evidence"] = serde_json::to_value(death)?;
                    write_step(&tx, owner, &step)?;
                    step.version = next_version(step.version)?;
                    step.state = if step.attempt == 16 {
                        "held"
                    } else {
                        "interrupted"
                    }
                    .into();
                    step.body["death_confirmed"] = json!(true);
                    if step.attempt == 16 {
                        step.body["reason"] = json!("attempt_limit");
                    }
                } else {
                    step.state = "held".into();
                    step.body["death_confirmed"] = json!(false);
                }
                write_step(&tx, owner, &step)?;
            }
            "launching" => {
                step.state = "unknown".into();
                step.body["death_confirmed"] = json!(false);
                write_step(&tx, owner, &step)?;
            }
            _ => {}
        }
        settle_wait(&tx, owner, wait, &step)?;
        record_test_cleanup(&tx, &run.id)?;
        let unsafe_step:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM pipeline_steps WHERE run_id=?1 AND (state IN ('planned','launching','running','exited','committed','unknown') OR (state='held' AND kind NOT IN ('prepare','release') AND json_extract(body,'$.death_confirmed') IS NOT 1)))",[&run.id],|r|r.get(0))?;
        let open_journal:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM pipeline_ref_journal WHERE (run_id=?1 OR (goal_id=?2 AND kind='goal_create')) AND state IN ('pending','held'))",params![run.id,run.goal_id.to_string()],|r|r.get(0))?;
        if unsafe_step || open_journal {
            ensure!(
                matches!(step.state.as_str(), "held" | "unknown" | "rejected"),
                "pipeline cancel requires release journal settlement first"
            );
            held(&tx, owner, &mut run, &step, "cancel_pending")?;
        } else {
            run.state = "abandoned".into();
            write_run(&tx, owner, &run)?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Completes a Goal with stored passing command results in its check run.
    pub fn pipeline_complete_goal(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        expected_goal: u64,
        last_command: &str,
    ) -> Result<()> {
        use sha2::{Digest, Sha256};
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (mut run, mut step) = cursor_rows(&tx, owner, cursor, true)?;
        certifying_exit(&step)?;
        ensure!(
            run.kind == "goal"
                && run.state == "running"
                && step.kind == "test"
                && step.state == "exited"
                && step.body.pointer("/exit/exit_code").and_then(Value::as_i64) == Some(0),
            "pipeline Goal check has not passed"
        );
        require_last_step(&tx, &step)?;
        ensure!(
            step.body["command_id"].as_str() == Some(last_command),
            "pipeline final command differs from plan"
        );
        let mut goal = pipeline_goal(&tx, &run, true)?;
        cas(
            usize::from(goal.version == expected_goal),
            "goals",
            &goal.id.to_string(),
            expected_goal,
        )?;
        let Some(profile) = check_test_profile(&tx, owner, &mut run, &mut step)? else {
            tx.commit()?;
            return Ok(());
        };
        ensure!(test_pass(&step)?, "pipeline Goal final test failed");
        let mut q=tx.prepare(&format!("SELECT {STEP_COLUMNS} FROM pipeline_steps WHERE run_id=?1 AND kind='test' AND state='adopted' ORDER BY seq,attempt"))?;
        let passed = q
            .query_map([&run.id], step_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(q);
        let mut ids = std::collections::BTreeSet::new();
        for passed in passed {
            ensure!(test_pass(&passed)?, "pipeline Goal prior test failed");
            ensure!(
                passed.step_base == run.head_commit
                    && passed
                        .body
                        .pointer("/exit/exit_code")
                        .and_then(Value::as_i64)
                        == Some(0),
                "pipeline Goal command evidence differs"
            );
            ensure!(
                passed.body["command_id"].as_str()
                    == passed
                        .body
                        .pointer("/adoption/command_id")
                        .and_then(Value::as_str),
                "pipeline adopted command differs from plan"
            );
            ids.insert(
                passed
                    .body
                    .pointer("/adoption/command_id")
                    .and_then(Value::as_str)
                    .context("pipeline test command ID missing")?
                    .to_owned(),
            );
        }
        ensure!(
            step.step_base == run.head_commit && ids.insert(last_command.into()),
            "pipeline final command duplicated"
        );
        let required = profile
            .proposal
            .commands
            .iter()
            .map(|c| c.id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        ensure!(ids == required, "pipeline Goal commands incomplete");
        let mut runs = Vec::new();
        for task in &goal.dag.nodes {
            let (id,head):(String,String)=tx.query_row("SELECT id,head_commit FROM pipeline_runs WHERE task_id=?1 AND goal_id=?2 AND project_id=?3 AND state='completed'",params![task.to_string(),goal.id.to_string(),goal.project_id.to_string()],|r|Ok((r.get(0)?,r.get(1)?)))?;
            runs.push((id, head));
        }
        runs.push((run.id.clone(), run.head_commit.clone()));
        runs.sort();
        let runs_sha256 = format!("{:x}", Sha256::digest(serde_json::to_vec(&runs)?));
        step.state = "adopted".into();
        step.body["adoption"] = json!({"command_id":last_command,"result":"pass"});
        write_step(&tx, owner, &step)?;
        record_test_cleanup(&tx, &run.id)?;
        tx.execute("INSERT INTO pipeline_goal_completions(goal_id,project_id,goal_branch_head,goal_check_run,runs_sha256,completed_epoch,body) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![goal.id.to_string(),goal.project_id.to_string(),run.head_commit,run.id,runs_sha256,owner.epoch,serde_json::to_string(&json!({"runs":runs}))?])?;
        run.state = "completed".into();
        write_run(&tx, owner, &run)?;
        goal.state = GoalState::Completed;
        goal.version = next_version(goal.version)?;
        goal.updated_at = now_ms();
        cas(
            tx.execute(
                "UPDATE goals SET version=?1,body=?2 WHERE id=?3 AND version=?4",
                params![
                    goal.version,
                    serde_json::to_string(&goal)?,
                    goal.id.to_string(),
                    expected_goal
                ],
            )?,
            "goals",
            &goal.id.to_string(),
            expected_goal,
        )?;
        append_event(
            &tx,
            &goal.scope(),
            "rrx.private.pipeline.goal_completed",
            json!({"run":run.id,"head":run.head_commit,"runs_sha256":runs_sha256}),
        )?;
        tx.commit()?;
        Ok(())
    }
}

fn admitted_profile(
    c: &Connection,
    goal: GoalId,
) -> Result<crate::execution::verification::AdmittedProfile> {
    let (digest,body):(String,String)=c.query_row("SELECT p.digest,p.body FROM goal_execution_profiles g JOIN verification_profiles p ON p.project_id=g.project_id AND p.digest=g.tests_profile_digest WHERE g.goal_id=?1 AND length(CAST(p.body AS BLOB))<=1048576",[goal.to_string()],|r|Ok((r.get(0)?,r.get(1)?)))?;
    decode_tests_profile(&digest, &body)
}

fn decode_tests_profile(
    digest: &str,
    body: &str,
) -> Result<crate::execution::verification::AdmittedProfile> {
    use crate::execution::verification::{AdmittedProfile, hash};
    ensure!(
        hash(body.as_bytes()) == digest,
        "pipeline TestsProfile body/digest mismatch"
    );
    let profile: AdmittedProfile = serde_json::from_str(body)?;
    profile.validate()?;
    Ok(profile)
}

fn check_test_profile(
    tx: &Transaction<'_>,
    owner: &RuntimeOwner,
    run: &mut PipelineRun,
    step: &mut PipelineStep,
) -> Result<Option<crate::execution::verification::AdmittedProfile>> {
    let result = admitted_profile(tx, run.goal_id).and_then(|profile| {
        crate::execution::pipeline::tests_runner::selected(&profile, step)?;
        ensure!(step.step_base == run.head_commit, "test commit changed");
        Ok(profile)
    });
    match result {
        Ok(profile) => Ok(Some(profile)),
        Err(_) => {
            step.state = "held".into();
            step.body["reason"] = json!("tests_profile_changed");
            if step.cookie.is_none() {
                step.body["death_confirmed"] = json!(true);
            }
            write_step(tx, owner, step)?;
            held(tx, owner, run, step, "tests_profile_changed")?;
            Ok(None)
        }
    }
}

/// A bounded but incomplete capture is a known failure only after timeout or overflow.
pub(crate) fn test_pass(step: &PipelineStep) -> Result<bool> {
    use crate::execution::verification::StreamEvidence;
    certifying_exit(step)?;
    let exit = &step.body["exit"];
    let out: StreamEvidence = serde_json::from_value(exit["stdout"].clone())?;
    let err: StreamEvidence = serde_json::from_value(exit["stderr"].clone())?;
    let timeout = exit["timed_out"]
        .as_bool()
        .context("test timeout observation missing")?;
    let overflow = exit["overflow"]
        .as_bool()
        .context("test overflow observation missing")?;
    ensure!(
        exit["exit_code"].as_i64().is_some() || exit["signal"].as_i64().is_some(),
        "test exit observation missing"
    );
    ensure!(
        (out.complete && err.complete) || timeout || overflow,
        "test stream observation incomplete"
    );
    Ok(exit["exit_code"] == 0 && !timeout && !overflow)
}

// Receipts are cumulative, including done rows, so later adoption cannot requeue or
// overwrite an earlier attempt. The caller's adoption/abandon transaction owns this write.
fn record_test_cleanup(tx: &Transaction<'_>, id: &str) -> Result<()> {
    tx.execute("INSERT INTO pipeline_test_cleanup(run_id,seq,attempt) SELECT s.run_id,s.seq,s.attempt FROM pipeline_steps s WHERE s.run_id=?1 AND s.kind='test' AND json_extract(s.body,'$.death_confirmed') IS 1 AND NOT EXISTS(SELECT 1 FROM pipeline_test_cleanup c WHERE (c.run_id,c.seq,c.attempt)=(s.run_id,s.seq,s.attempt))", [id])?;
    Ok(())
}

fn cleanup_rows(c: &Connection, id: &str) -> Result<Vec<PipelineCleanup>> {
    let mut q = c.prepare("SELECT run_id,seq,attempt,failures,not_before,failure_class,path FROM pipeline_test_cleanup WHERE run_id=?1 AND state='pending' ORDER BY seq,attempt")?;
    Ok(q.query_map([id], |r| {
        Ok(PipelineCleanup {
            run_id: r.get(0)?,
            seq: r.get(1)?,
            attempt: r.get(2)?,
            failures: r.get(3)?,
            not_before: r.get(4)?,
            failure_class: r.get(5)?,
            path: r.get(6)?,
        })
    })?
    .collect::<rusqlite::Result<Vec<_>>>()?)
}

impl Store {
    /// Due maintenance is fenced by run ownership, independently of cursor versions.
    pub(crate) fn pipeline_test_cleanup(
        &mut self,
        owner: &RuntimeOwner,
        id: &str,
    ) -> Result<Vec<(PipelineCleanup, PipelineStep, PathBuf)>> {
        let tx = self.connection.transaction()?;
        pipeline_owner_current(&tx, owner)?;
        let run = read_run(&tx, id)?;
        ensure!(
            run.owner_epoch == owner.epoch,
            "pipeline cleanup owner retired"
        );
        let live: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM pipeline_steps WHERE run_id=?1 AND state IN ('launching','running'))", [id], |r| r.get(0))?;
        if live {
            return Ok(Vec::new());
        }
        let pending = cleanup_rows(&tx, id)?;
        let mut rows = Vec::new();
        for obligation in pending {
            if obligation.not_before > pipeline_now_ms() {
                continue;
            }
            let step = read_step(&tx, id, obligation.seq, obligation.attempt)?;
            ensure!(
                step.kind == "test" && step.body["death_confirmed"] == true,
                "test cleanup death unconfirmed"
            );
            let project = tx.query_row(
                "SELECT body FROM projects WHERE id=?1",
                [run.project_id.to_string()],
                |r| r.get::<_, String>(0),
            )?;
            let project: crate::domain::Project = serde_json::from_str(&project)?;
            rows.push((obligation, step, project.root));
        }
        tx.commit()?;
        Ok(rows)
    }

    pub(crate) fn pipeline_test_cleanup_runs(&self, owner: &RuntimeOwner) -> Result<Vec<String>> {
        let mut q = self.connection.prepare("SELECT DISTINCT r.id FROM pipeline_runs r JOIN pipeline_test_cleanup c ON c.run_id=r.id WHERE r.owner_epoch=?1 AND c.state='pending' ORDER BY r.id")?;
        Ok(q.query_map([owner.epoch], |r| r.get(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Metadata-only fallback when a sweep cannot read a step or Project inventory.
    pub(crate) fn pipeline_test_cleanup_pending(
        &self,
        owner: &RuntimeOwner,
        id: &str,
    ) -> Result<Vec<PipelineCleanup>> {
        let tx = self.connection.unchecked_transaction()?;
        pipeline_owner_current(&tx, owner)?;
        ensure!(
            read_run(&tx, id)?.owner_epoch == owner.epoch,
            "pipeline cleanup owner retired"
        );
        let live: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM pipeline_steps WHERE run_id=?1 AND state IN ('launching','running'))", [id], |r| r.get(0))?;
        if live {
            return Ok(vec![]);
        }
        let now = pipeline_now_ms();
        let pending = cleanup_rows(&tx, id)?
            .into_iter()
            .filter(|c| c.not_before <= now)
            .collect();
        tx.commit()?;
        Ok(pending)
    }

    /// Record only a fixed class and derived path; no process or file contents.
    pub(crate) fn pipeline_test_cleanup_result(
        &mut self,
        owner: &RuntimeOwner,
        obligation: &PipelineCleanup,
        failure: Option<(&str, &std::path::Path)>,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let run = read_run(&tx, &obligation.run_id)?;
        ensure!(
            run.owner_epoch == owner.epoch,
            "pipeline cleanup owner retired"
        );
        let (state, failures, not_before, class, path) = match failure {
            Some((class, path)) => {
                ensure!(
                    matches!(class, "checkout_remove" | "tmpdir_remove" | "cleanup_sweep"),
                    "invalid cleanup class"
                );
                let failures = obligation.failures.saturating_add(1).min(32);
                let backoff = (1000i64 << failures.min(8)).min(300_000);
                (
                    "pending",
                    failures,
                    pipeline_now_ms().saturating_add(backoff),
                    Some(class),
                    Some(path.to_string_lossy().into_owned()),
                )
            }
            None => ("done", obligation.failures, 0, None, None),
        };
        ensure!(tx.execute("UPDATE pipeline_test_cleanup SET state=?4,failures=?5,not_before=?6,failure_class=?7,path=?8 WHERE run_id=?1 AND seq=?2 AND attempt=?3 AND state='pending' AND failures=?9 AND not_before=?10", params![obligation.run_id,obligation.seq,obligation.attempt,state,failures,not_before,class,path,obligation.failures,obligation.not_before])? == 1, "test cleanup changed");
        tx.commit()?;
        Ok(())
    }

    /// Active runs, journals and limit waits remain with the full takeover protocol.
    pub(crate) fn pipeline_take_over_test_cleanup(&mut self, owner: &RuntimeOwner) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let mut q = tx.prepare(&format!("SELECT {RUN_COLUMNS} FROM pipeline_runs r WHERE state IN ('completed','abandoned') AND EXISTS(SELECT 1 FROM pipeline_test_cleanup c WHERE c.run_id=r.id AND c.state='pending') ORDER BY id"))?;
        let runs = q
            .query_map([], run_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(q);
        for run in runs {
            if run.owner_epoch == owner.epoch {
                continue;
            }
            ensure!(
                run.owner_epoch < owner.epoch,
                "pipeline cleanup owner is newer"
            );
            let mut q = tx.prepare(&format!(
                "SELECT {STEP_COLUMNS} FROM pipeline_steps WHERE run_id=?1 ORDER BY seq,attempt"
            ))?;
            let steps = q
                .query_map([&run.id], step_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            drop(q);
            ensure!(
                !steps
                    .iter()
                    .any(|s| matches!(s.state.as_str(), "launching" | "running")),
                "terminal cleanup has live step"
            );
            pipeline_goal(&tx, &run, false)?;
            write_run(&tx, owner, &run)?;
            for step in steps {
                write_step(&tx, owner, &step)?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Reads and checks the pinned catalog at the planning/launch boundary, holding on corruption.
    pub(crate) fn pipeline_test_context(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
    ) -> Result<Option<crate::execution::verification::AdmittedProfile>> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (mut run, mut step) = cursor_rows(&tx, owner, cursor, true)?;
        require_last_step(&tx, &step)?;
        check_wait(&tx, owner, cursor.wait_version)?;
        ensure!(
            run.state == "running"
                && step.kind == "test"
                && matches!(step.state.as_str(), "planned" | "exited"),
            "test context state changed"
        );
        let profile = check_test_profile(&tx, owner, &mut run, &mut step)?;
        tx.commit()?;
        Ok(profile)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineRecoveryRequest {
    pub request_id: Uuid,
    pub cursor: PipelineCursor,
    pub then_action: String,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineRecoveryEvidence {
    pub discovery_complete: bool,
    pub matching_processes: u32,
    pub workspace_verified: bool,
    pub journal: Option<PipelineRecoveryJournal>,
    pub body: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineRecoveryJournal {
    pub id: String,
    pub version: u64,
    pub observed_commit: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PipelineRecoveryResponse {
    pub request_id: Uuid,
    pub run_id: String,
    pub run_version: u64,
    pub step_version: u64,
    pub next_attempt: Option<u32>,
    pub state: String,
    pub release_applied: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PipelineRecoveryPreflight {
    Replayed(PipelineRecoveryResponse),
    Pending {
        run: Box<PipelineRun>,
        step: Box<PipelineStep>,
        journal: Option<Box<PipelineJournal>>,
    },
}
fn recovery_identity(
    ingress: &crate::runtime::control::HumanIngress,
    owner: &RuntimeOwner,
    request: &PipelineRecoveryRequest,
) -> Result<(u32, String)> {
    use sha2::{Digest, Sha256};
    let (instance, epoch, uid) = ingress.identity();
    ensure!(
        instance == owner.instance
            && epoch == owner.epoch
            && uid == rustix::process::getuid().as_raw(),
        "pipeline ingress owner differs"
    );
    ensure!(
        !request.request_id.is_nil()
            && !request.reason.trim().is_empty()
            && request.reason.len() <= 16384
            && matches!(request.then_action.as_str(), "retry" | "abandon_run"),
        "invalid pipeline recovery request"
    );
    let encoded = serde_json::to_vec(&(instance, epoch, request))?;
    ensure!(encoded.len() <= 65536, "pipeline recovery exceeds bound");
    Ok((uid, format!("{:x}", Sha256::digest(encoded))))
}
fn recovery_ack(
    tx: &Transaction<'_>,
    owner: &RuntimeOwner,
    request: &PipelineRecoveryRequest,
    uid: u32,
    digest: &str,
) -> Result<Option<PipelineRecoveryResponse>> {
    let saved:Option<(String,String,u32,String)>=tx.query_row("SELECT instance,action_sha256,ingress_uid,body FROM runtime_control_acks WHERE request_id=?1",[request.request_id.to_string()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
    if let Some((instance, hash, saved_uid, body)) = saved {
        ensure!(
            instance == owner.instance && hash == digest && saved_uid == uid,
            "pipeline request ID reused with different action"
        );
        ensure!(body.len() <= 65536, "pipeline ack exceeds bound");
        return Ok(Some(serde_json::from_str(&body)?));
    }
    Ok(None)
}
fn recovery_rows(
    tx: &Transaction<'_>,
    owner: &RuntimeOwner,
    request: &PipelineRecoveryRequest,
) -> Result<(PipelineRun, PipelineStep, Option<PipelineJournal>)> {
    let retry = request.then_action == "retry";
    let (run, step) = cursor_rows(tx, owner, &request.cursor, retry)?;
    ensure!(
        run.state == "held" && matches!(step.state.as_str(), "unknown" | "held" | "rejected"),
        "pipeline recovery needs held run and recoverable step"
    );
    let later: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM pipeline_steps WHERE run_id=?1 AND (seq,attempt)>(?2,?3))",
        params![run.id, step.seq, step.attempt],
        |r| r.get(0),
    )?;
    ensure!(!later, "pipeline recovery target is not last step");
    if retry {
        ensure!(step.attempt < 16, "pipeline attempts exhausted");
    }
    let journal=tx.query_row(&format!("SELECT {JOURNAL_COLUMNS} FROM pipeline_ref_journal WHERE (run_id=?1 OR (goal_id=?2 AND kind='goal_create')) AND state IN ('pending','held')"),params![run.id,run.goal_id.to_string()],journal_row).optional()?;
    ensure!(
        !retry
            || journal.as_ref().is_none_or(|j| {
                j.kind == "goal_create" && j.body["creator_run_id"].as_str() == Some(&run.id)
            }),
        "pipeline retry requires journal settlement via abandon_run"
    );
    Ok((run, step, journal))
}

impl Store {
    /// Use before side effects, and again after taking the control boundary.
    pub fn pipeline_recovery_preflight(
        &self,
        owner: &RuntimeOwner,
        ingress: &crate::runtime::control::HumanIngress,
        request: &PipelineRecoveryRequest,
    ) -> Result<PipelineRecoveryPreflight> {
        let (uid, digest) = recovery_identity(ingress, owner, request)?;
        let tx = self.connection.unchecked_transaction()?;
        pipeline_owner_current(&tx, owner)?;
        let out = if let Some(response) = recovery_ack(&tx, owner, request, uid, &digest)? {
            PipelineRecoveryPreflight::Replayed(response)
        } else {
            let (run, step, journal) = recovery_rows(&tx, owner, request)?;
            PipelineRecoveryPreflight::Pending {
                run: Box::new(run),
                step: Box::new(step),
                journal: journal.map(Box::new),
            }
        };
        tx.commit()?;
        Ok(out)
    }

    /// Ack replay precedes version checks; the ordered writes commit exactly once.
    pub fn pipeline_recover(
        &mut self,
        owner: &RuntimeOwner,
        ingress: &crate::runtime::control::HumanIngress,
        request: &PipelineRecoveryRequest,
        evidence: &PipelineRecoveryEvidence,
    ) -> Result<PipelineRecoveryResponse> {
        let (uid, digest) = recovery_identity(ingress, owner, request)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        if let Some(response) = recovery_ack(&tx, owner, request, uid, &digest)? {
            tx.commit()?;
            return Ok(response);
        }
        let (mut run, mut step, journal) = recovery_rows(&tx, owner, request)?;
        if step.cookie.is_some() {
            ensure!(
                evidence.discovery_complete && evidence.matching_processes == 0,
                "pipeline recovery process discovery incomplete or alive"
            );
        }
        let retry = request.then_action == "retry";
        if retry {
            ensure!(
                evidence.workspace_verified,
                "pipeline recovery workspace not verified"
            );
            if step.kind == "test" {
                let profile = admitted_profile(&tx, run.goal_id)?;
                crate::execution::pipeline::tests_runner::selected(&profile, &step)?;
            }
        }
        let mut settled = None;
        let release_applied = if !retry {
            match (journal, evidence.journal.as_ref()) {
                (Some(mut j), Some(observed)) => {
                    ensure!(
                        j.owner_epoch == owner.epoch && j.id == observed.id,
                        "pipeline recovery journal identity changed"
                    );
                    cas(
                        usize::from(j.version == observed.version),
                        "pipeline_ref_journal",
                        &j.id,
                        observed.version,
                    )?;
                    let applied = if observed.observed_commit == j.new_commit {
                        true
                    } else if observed.observed_commit == j.old_commit {
                        false
                    } else {
                        bail!("ref_unexpected");
                    };
                    ensure!(
                        j.kind != "goal_create"
                            || applied
                            || pipeline_goal(&tx, &run, false)?.state == GoalState::Cancelled,
                        "Running Goal create must be settled before abandonment"
                    );
                    j.state = if applied { "done" } else { "aborted" }.into();
                    let release_applied = applied && j.kind == "task_release";
                    settled = Some(j);
                    release_applied
                }
                (None, None) => false,
                _ => bail!("pipeline recovery journal inventory changed"),
            }
        } else {
            false
        };
        let state = if retry {
            "running"
        } else if release_applied {
            "completed"
        } else {
            "abandoned"
        };
        let response = PipelineRecoveryResponse {
            request_id: request.request_id,
            run_id: run.id.clone(),
            run_version: next_version(run.version)?,
            step_version: next_version(step.version)?,
            next_attempt: retry.then_some(step.attempt + 1),
            state: state.into(),
            release_applied,
        };
        let wait = check_wait(&tx, owner, request.cursor.wait_version)?;
        tx.execute("INSERT INTO runtime_control_acks(request_id,instance,action_sha256,ingress_uid,accepted_epoch,body) VALUES(?1,?2,?3,?4,?5,?6)",params![request.request_id.to_string(),owner.instance,digest,uid,owner.epoch,serde_json::to_string(&response)?])?;
        tx.execute("INSERT INTO pipeline_recoveries(id,run_id,seq,attempt,request_id,ingress_uid,accepted_epoch,then_action,consumed_version,body) VALUES(?1,?2,?3,?4,?1,?5,?6,?7,?8,?9)",params![request.request_id.to_string(),run.id,step.seq,step.attempt,uid,owner.epoch,request.then_action,response.run_version,serde_json::to_string(&json!({"reason":request.reason,"discovery_complete":evidence.discovery_complete,"matching_processes":evidence.matching_processes,"workspace_verified":evidence.workspace_verified,"journal":evidence.journal,"release_applied":release_applied,"evidence":evidence.body}))?])?;
        if let Some(journal) = settled {
            write_journal(&tx, owner, &journal)?;
        }
        step.state = "abandoned".into();
        step.body["death_confirmed"] = json!(true);
        write_step(&tx, owner, &step)?;
        settle_wait(&tx, owner, wait, &step)?;
        record_test_cleanup(&tx, &run.id)?;
        run.state = state.into();
        run.body["held"] = Value::Null;
        write_run(&tx, owner, &run)?;
        if retry {
            let mut next = step.clone();
            next.attempt += 1;
            next.version = 0;
            next.state = "planned".into();
            next.cookie = None;
            next.pid = None;
            next.birth = None;
            next.result_commit = None;
            next.result_tree = None;
            next.body = json!({"recovery":request.request_id,"command_id":step.body.get("command_id"),"command_index":step.body.get("command_index")});
            insert_step(&tx, owner, &next)?;
        }
        append_event(
            &tx,
            &Scope {
                project_id: run.project_id,
                goal_id: Some(run.goal_id),
                task_id: run.task_id,
            },
            "rrx.private.pipeline.step_abandoned",
            json!({"run":run.id,"seq":step.seq,"attempt":step.attempt,"request":request.request_id,"then":request.then_action}),
        )?;
        tx.commit()?;
        Ok(response)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineDeathEvidence {
    pub launch_checked: bool,
    pub leader_birth_matched: bool,
    pub leader_exit_observed: bool,
    pub group_kill_delivered: bool,
    pub leader_reaped: bool,
    pub group_empty: bool,
    pub discovery_complete: bool,
    pub matching_processes: u32,
    #[serde(default)]
    pub discovery_attempts: u32,
}
impl PipelineDeathEvidence {
    fn validate(&self, has_birth: bool) -> Result<()> {
        ensure!(
            self.launch_checked
                && (!has_birth || self.leader_birth_matched)
                && self.leader_exit_observed
                && self.group_kill_delivered
                && self.leader_reaped
                && self.group_empty
                && self.discovery_complete
                && self.matching_processes == 0,
            "pipeline process death unconfirmed"
        );
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PipelineLaunch {
    Started,
    Waiting,
    Held,
}

/// Exit observations and their final classification share one commit.
pub struct PipelineExit<'a> {
    pub outcome: Option<&'a str>,
    pub evidence: &'a Value,
    pub issue: Option<&'a str>,
}

fn certifying_exit(step: &PipelineStep) -> Result<()> {
    ensure!(
        step.body["death_confirmed"] == true,
        "pipeline adoption death unconfirmed"
    );
    ensure!(
        step.body["exit"].is_object() && step.body["exit"]["issue"].is_null(),
        "pipeline adoption exit issue"
    );
    Ok(())
}

impl Store {
    /// Operator-defined scopes are read from the immutable accepted policy, never Task.version.
    pub(crate) fn pipeline_executor_context(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
    ) -> Result<(Project, Vec<String>, Option<Value>)> {
        let tx = self.connection.transaction()?;
        pipeline_owner_current(&tx, owner)?;
        let (run, step) = cursor_rows(&tx, owner, cursor, true)?;
        require_last_step(&tx, &step)?;
        let project: Project = read_tx(&tx, "projects", &run.project_id.to_string())?
            .context("pipeline Project missing")?;
        let policy: String = tx.query_row(
            "SELECT body FROM goal_execution_profiles WHERE goal_id=?1",
            [run.goal_id.to_string()],
            |r| r.get(0),
        )?;
        let policy: Value = serde_json::from_str(&policy)?;
        let goal = pipeline_goal(&tx, &run, true)?;
        let scopes = executor_scopes(
            &policy,
            &goal.dag.nodes,
            run.task_id.context("Executor requires Task")?,
        )?;
        let controls: Option<String> = tx.query_row("SELECT json_extract(body,'$.adoption.controls') FROM pipeline_steps WHERE run_id=?1 AND kind='prepare' AND state='adopted'", [&run.id], |r| r.get(0)).optional()?.flatten();
        let controls = controls.map(|s| serde_json::from_str(&s)).transpose()?;
        tx.commit()?;
        Ok((project, scopes, controls))
    }

    /// Pre-spawn control inventory is fenced by the same owner, Goal and versions as launch.
    pub(crate) fn pipeline_executor_preflight(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        controls: &Value,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (run, mut step) = cursor_rows(&tx, owner, cursor, true)?;
        require_last_step(&tx, &step)?;
        ensure!(
            run.state == "running"
                && step.state == "planned"
                && matches!(step.kind.as_str(), "implement" | "fix"),
            "Executor preflight state changed"
        );
        ensure!(controls.is_object(), "Executor controls missing");
        step.body["executor_controls"] = controls.clone();
        write_step(&tx, owner, &step)?;
        tx.commit()?;
        Ok(())
    }

    /// Read exactly the caller's launch snapshot without extending its authority.
    pub fn pipeline_launch_snapshot(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
    ) -> Result<(PipelineRun, PipelineStep)> {
        let tx = self.connection.transaction()?;
        pipeline_owner_current(&tx, owner)?;
        let rows = cursor_rows(&tx, owner, cursor, true)?;
        check_wait(&tx, owner, cursor.wait_version)?;
        require_last_step(&tx, &rows.1)?;
        tx.commit()?;
        Ok(rows)
    }

    /// The intent transaction only; the caller retains the execution boundary.
    pub fn pipeline_begin_launch(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        cookie: Option<&str>,
        launch: Option<&rrx_process_tracker::Launch>,
        intent: &Value,
    ) -> Result<PipelineLaunch> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        pipeline_startup_current(&tx, owner)?;
        let (mut run, mut step) = cursor_rows(&tx, owner, cursor, true)?;
        ensure!(
            run.state == "running" && step.state == "planned",
            "pipeline launch state changed"
        );
        let wait = check_wait(&tx, owner, cursor.wait_version)?;
        if step.kind == "test" && check_test_profile(&tx, owner, &mut run, &mut step)?.is_none() {
            tx.commit()?;
            return Ok(PipelineLaunch::Held);
        }
        let internal = matches!(step.kind.as_str(), "prepare" | "release");
        if !native_exhaustion(&tx)?.is_empty() {
            step.state = "held".into();
            step.body["death_confirmed"] = json!(true);
            step.body["reason"] = json!("native_quota_exhausted");
            write_step(&tx, owner, &step)?;
            held(&tx, owner, &mut run, &step, "native_quota_exhausted")?;
            tx.commit()?;
            return Ok(PipelineLaunch::Held);
        }
        if !internal {
            let busy:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM pipeline_steps WHERE kind NOT IN ('prepare','release') AND (state IN ('launching','running','unknown') OR (state='held' AND json_extract(body,'$.death_confirmed') IS NOT 1)))",[],|r|r.get(0))?;
            if busy {
                tx.commit()?;
                return Ok(PipelineLaunch::Waiting);
            }
            ensure!(
                cookie.is_some_and(|c| (16..=128).contains(&c.len())),
                "pipeline launch cookie missing"
            );
            if let Some(provider) = &step.provider {
                let qualified:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM pipeline_qualifications WHERE owner_epoch=?1 AND subject=?2 AND state='qualified')",params![owner.epoch,provider],|r|r.get(0))?;
                ensure!(qualified, "pipeline CLI is not qualified in this epoch");
                if let Some(mut wait) = wait {
                    match wait.state.as_str() {
                        "waiting" if pipeline_now_ms() >= wait.not_before => {
                            wait.state = "probing".into();
                            wait.probe_run = Some(run.id.clone());
                            wait.probe_seq = Some(step.seq);
                            wait.probe_attempt = Some(step.attempt);
                            write_wait(&tx, owner, &wait)?;
                        }
                        "clear" => {}
                        _ => {
                            tx.commit()?;
                            return Ok(PipelineLaunch::Waiting);
                        }
                    }
                }
            }
        } else {
            ensure!(
                cookie.is_none(),
                "internal pipeline step cannot have a cookie"
            );
        }
        step.state = if internal { "running" } else { "launching" }.into();
        step.cookie = cookie.map(str::to_owned);
        step.body["intent"] = intent.clone();
        if !internal {
            let launch = launch.context("pipeline launch mark missing")?;
            ensure!(launch.wall.1 < 1_000_000, "invalid launch microseconds");
            step.body["launch"] =
                json!({"wall":format!("{}:{}",launch.wall.0,launch.wall.1),"boot":launch.boot});
        }
        write_step(&tx, owner, &step)?;
        tx.commit()?;
        Ok(PipelineLaunch::Started)
    }

    pub fn pipeline_record_running(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        pid: u32,
        birth: &str,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (_, mut step) = cursor_rows(&tx, owner, cursor, false)?;
        ensure!(
            step.state == "launching" && pid > 1 && !birth.is_empty(),
            "invalid pipeline running receipt"
        );
        step.state = "running".into();
        step.pid = Some(pid);
        step.birth = Some(birth.into());
        step.body["pgid"] = json!(pid);
        write_step(&tx, owner, &step)?;
        tx.commit()?;
        Ok(())
    }

    pub fn pipeline_record_launch_unknown(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        reason: &str,
    ) -> Result<()> {
        self.pipeline_observe_stop(owner, cursor, "unknown", reason, None)
    }
    pub fn pipeline_hold(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        reason: &str,
        death: Option<&PipelineDeathEvidence>,
    ) -> Result<()> {
        self.pipeline_observe_stop(owner, cursor, "held", reason, death)
    }
    pub fn pipeline_reject(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        reason: &str,
    ) -> Result<()> {
        self.pipeline_observe_stop(owner, cursor, "rejected", reason, None)
    }
    pub fn pipeline_record_interrupted(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        death: &PipelineDeathEvidence,
    ) -> Result<()> {
        self.pipeline_observe_stop(owner, cursor, "interrupted", "interrupted", Some(death))
    }
    /// An internal prepare has no process; absence is recorded without inventing death evidence.
    pub(crate) fn pipeline_prepare_absent(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
    ) -> Result<()> {
        let (_, step) = self.pipeline_launch_snapshot(owner, cursor)?;
        ensure!(
            step.kind == "prepare" && step.state == "running",
            "prepare absence state changed"
        );
        self.pipeline_observe_stop(owner, cursor, "interrupted", "prepare_absent", None)
    }
    fn pipeline_observe_stop(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        state: &str,
        reason: &str,
        death: Option<&PipelineDeathEvidence>,
    ) -> Result<()> {
        self.pipeline_observe_stop_detail(owner, cursor, state, reason, death, None)
    }
    pub(crate) fn pipeline_hold_secret(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        evidence: &Value,
    ) -> Result<()> {
        self.pipeline_observe_stop_detail(
            owner,
            cursor,
            "held",
            "secret_detected",
            None,
            Some(evidence),
        )
    }
    fn pipeline_observe_stop_detail(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        state: &str,
        reason: &str,
        death: Option<&PipelineDeathEvidence>,
        detail: Option<&Value>,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (mut run, mut step) = cursor_rows(&tx, owner, cursor, false)?;
        ensure!(
            matches!(
                step.state.as_str(),
                "planned" | "launching" | "running" | "exited" | "committed"
            ),
            "pipeline stop source is held or terminal"
        );
        ensure!(!reason.trim().is_empty(), "pipeline stop reason missing");
        if let Some(death) = death {
            death.validate(step.birth.is_some())?;
        }
        let internal = matches!(step.kind.as_str(), "prepare" | "release");
        let confirmed = internal
            || step.state == "planned"
            || matches!(step.state.as_str(), "exited" | "committed")
            || death.is_some();
        if state == "interrupted" {
            ensure!(confirmed, "pipeline interruption needs death proof");
        }
        if !internal && death.is_some() && matches!(step.state.as_str(), "launching" | "running") {
            step.state = "exited".into();
            step.body["death_confirmed"] = json!(true);
            step.body["death_evidence"] = serde_json::to_value(death)?;
            write_step(&tx, owner, &step)?;
            step.version = next_version(step.version)?;
        }
        let at_limit = state == "interrupted" && step.attempt == 16;
        step.state = if at_limit { "held" } else { state }.into();
        if !internal {
            step.body["death_confirmed"] = json!(confirmed);
        }
        step.body["reason"] = json!(if at_limit { "attempt_limit" } else { reason });
        if let Some(detail) = detail {
            step.body["secret"] = detail.clone();
        }
        let wait = check_wait(&tx, owner, cursor.wait_version)?;
        write_step(&tx, owner, &step)?;
        settle_wait(&tx, owner, wait, &step)?;
        if matches!(step.state.as_str(), "held" | "unknown" | "rejected") {
            held(
                &tx,
                owner,
                &mut run,
                &step,
                if at_limit { "attempt_limit" } else { reason },
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn pipeline_record_exit(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        death: &PipelineDeathEvidence,
        outcome: Option<&str>,
        evidence: &Value,
    ) -> Result<()> {
        self.pipeline_finish_step(
            owner,
            cursor,
            death,
            PipelineExit {
                outcome,
                evidence,
                issue: None,
            },
        )
    }

    pub fn pipeline_finish_step(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        death: &PipelineDeathEvidence,
        exit: PipelineExit<'_>,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (mut run, mut step) = cursor_rows(&tx, owner, cursor, false)?;
        death.validate(step.birth.is_some())?;
        ensure!(run.state == "running", "pipeline exit run is not Running");
        ensure!(
            matches!(step.state.as_str(), "running" | "launching"),
            "pipeline exit state changed"
        );
        if step.provider.is_some() {
            ensure!(
                matches!(exit.outcome, Some("success" | "limit" | "other")),
                "Agent exit requires classified provider outcome"
            );
        } else {
            ensure!(
                exit.outcome.is_none(),
                "non-Agent exit cannot classify provider"
            );
        }
        if let Some(reason) = exit.issue {
            ensure!(!reason.trim().is_empty(), "pipeline exit issue missing");
            ensure!(
                step.provider.is_none() || exit.outcome == Some("other"),
                "failed exit cannot classify provider success or limit"
            );
        }
        let wait = check_wait(&tx, owner, cursor.wait_version)?;
        step.state = "exited".into();
        step.body["death_confirmed"] = json!(true);
        step.body["death_evidence"] = serde_json::to_value(death)?;
        step.body["exit"] = exit.evidence.clone();
        if let Some(outcome) = exit.outcome {
            step.body["provider_outcome"] = json!(outcome);
        }
        write_step(&tx, owner, &step)?;
        settle_wait(&tx, owner, wait, &step)?;
        if let Some(reason) = exit.issue {
            step.version = next_version(step.version)?;
            let at_limit = reason == "interrupted" && step.attempt == 16;
            step.state = if at_limit {
                "held"
            } else if reason == "interrupted" {
                "interrupted"
            } else if (step.kind == "review" && reason == "secret_detected")
                || (step.kind == "test" && reason != "record_failed")
            {
                "held"
            } else {
                "rejected"
            }
            .into();
            let reason = if at_limit { "attempt_limit" } else { reason };
            step.body["reason"] = json!(reason);
            if step.kind == "review" && reason == "secret_detected" {
                step.body["secret"] = exit.evidence["secret"].clone();
            }
            write_step(&tx, owner, &step)?;
            if step.state != "interrupted" {
                held(&tx, owner, &mut run, &step, reason)?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn pipeline_record_limited(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (mut run, mut step) = cursor_rows(&tx, owner, cursor, true)?;
        certifying_exit(&step)?;
        ensure!(
            run.state == "running"
                && step.state == "exited"
                && matches!(
                    step.kind.as_str(),
                    "implement" | "fix" | "review" | "confirm"
                )
                && step.body["provider_outcome"] == "limit",
            "pipeline limit requires known limit exit"
        );
        let profile: String = tx.query_row(
            "SELECT body FROM goal_execution_profiles WHERE goal_id=?1",
            [run.goal_id.to_string()],
            |r| r.get(0),
        )?;
        let profile: Value = serde_json::from_str(&profile)?;
        let max = pipeline_max_limit_waits(&profile)?;
        ensure!(max <= 6, "invalid pipeline wait policy");
        let waits = run
            .body
            .pointer("/limit/waits")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .checked_add(1)
            .context("pipeline limit count overflow")?;
        run.body["limit"] = json!({"waits":waits});
        if step.attempt == 16 || waits > max {
            let reason = if step.attempt == 16 {
                "attempt_limit"
            } else {
                "limit_waits_exhausted"
            };
            step.state = "held".into();
            step.body["reason"] = json!(reason);
            write_step(&tx, owner, &step)?;
            held(&tx, owner, &mut run, &step, reason)?;
        } else {
            step.state = "limited".into();
            write_step(&tx, owner, &step)?;
            write_run(&tx, owner, &run)?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn pipeline_record_commit(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        commit: &str,
        tree: &str,
        evidence: &Value,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (run, mut step) = cursor_rows(&tx, owner, cursor, true)?;
        certifying_exit(&step)?;
        ensure!(
            run.state == "running"
                && step.state == "exited"
                && matches!(step.kind.as_str(), "implement" | "fix"),
            "pipeline commit state changed"
        );
        ensure!(
            valid_oid(commit) && valid_oid(tree),
            "invalid pipeline commit/tree"
        );
        step.state = "committed".into();
        step.result_commit = Some(commit.into());
        step.result_tree = Some(tree.into());
        step.body["commit_evidence"] = evidence.clone();
        write_step(&tx, owner, &step)?;
        tx.commit()?;
        Ok(())
    }

    pub fn pipeline_adopt(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        evidence: &Value,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (mut run, mut step) = cursor_rows(&tx, owner, cursor, true)?;
        ensure!(run.state == "running", "pipeline run is held");
        require_last_step(&tx, &step)?;
        match step.kind.as_str() {
            "implement" | "fix" => ensure!(
                step.state == "committed",
                "pipeline adoption needs recorded commit"
            ),
            "test" | "review" | "confirm" => ensure!(
                step.state == "exited",
                "pipeline adoption needs exited step"
            ),
            "prepare" => {
                let done: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM pipeline_ref_journal WHERE goal_id=?1 AND kind='goal_create' AND state='done')", [run.goal_id.to_string()], |r| r.get(0))?;
                ensure!(
                    step.state == "running" && done,
                    "pipeline prepare requires done Goal create"
                );
            }
            _ => bail!("pipeline release is adopted only by journal settlement"),
        }
        if !matches!(step.kind.as_str(), "prepare" | "release") {
            certifying_exit(&step)?;
        }
        if step.kind == "test" {
            let Some(_) = check_test_profile(&tx, owner, &mut run, &mut step)? else {
                tx.commit()?;
                return Ok(());
            };
            let pass = test_pass(&step)?;
            ensure!(
                evidence["result"].as_str() == Some(if pass { "pass" } else { "fail" }),
                "test result classification changed"
            );
            ensure!(
                step.body["command_id"].as_str().is_some()
                    && step.body["command_id"].as_str() == evidence["command_id"].as_str(),
                "pipeline adoption command differs from plan"
            );
            record_test_cleanup(&tx, &run.id)?;
        }
        if step.kind == "review" {
            let policy: String = tx.query_row(
                "SELECT body FROM goal_execution_profiles WHERE goal_id=?1",
                [run.goal_id.to_string()],
                |r| r.get(0),
            )?;
            let policy: Value = serde_json::from_str(&policy)?;
            review::review_adoption(
                &step,
                evidence,
                serde_json::from_value(policy["blocking_severity"].clone())?,
            )?;
        }
        step.state = "adopted".into();
        step.body["adoption"] = evidence.clone();
        write_step(&tx, owner, &step)?;
        if matches!(step.kind.as_str(), "implement" | "fix") {
            run.head_commit = step
                .result_commit
                .clone()
                .context("pipeline commit missing")?;
            write_run(&tx, owner, &run)?;
        }
        tx.commit()?;
        Ok(())
    }

    /// New attempts require caller's workspace verification; no old attempt is restarted.
    pub fn pipeline_plan_next(
        &mut self,
        owner: &RuntimeOwner,
        cursor: &PipelineCursor,
        next: &PipelineStep,
        workspace_verified: bool,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let (mut run, previous) = cursor_rows(&tx, owner, cursor, true)?;
        ensure!(
            run.state == "running" && next.run_id == run.id,
            "pipeline planning scope changed"
        );
        if matches!(previous.state.as_str(), "interrupted" | "limited") {
            ensure!(
                workspace_verified
                    && previous.attempt < 16
                    && next.seq == previous.seq
                    && next.attempt == previous.attempt + 1
                    && next.kind == previous.kind
                    && next.provider == previous.provider
                    && next.round == previous.round
                    && next.step_base == previous.step_base
                    && next.body["command_id"] == previous.body["command_id"],
                "invalid pipeline retry attempt"
            );
        } else {
            ensure!(
                previous.state == "adopted" && next.seq == previous.seq + 1 && next.attempt == 1,
                "invalid next pipeline sequence"
            );
            ensure!(
                next.round == run.round || (next.kind == "fix" && next.round == run.round + 1),
                "invalid pipeline round"
            );
            if next.round != run.round {
                let body: String = tx.query_row(
                    "SELECT body FROM goal_execution_profiles WHERE goal_id=?1",
                    [run.goal_id.to_string()],
                    |r| r.get(0),
                )?;
                let profile: Value = serde_json::from_str(&body)?;
                let max = profile["max_fix_rounds"]
                    .as_u64()
                    .context("pipeline fix policy missing")?;
                ensure!(
                    u64::from(next.round) <= max && max <= 2,
                    "pipeline fix rounds exhausted"
                );
                run.round = next.round;
                write_run(&tx, owner, &run)?;
            }
        }
        insert_step(&tx, owner, next)?;
        tx.commit()?;
        Ok(())
    }

    pub fn pipeline_settle_limit_wait(
        &mut self,
        owner: &RuntimeOwner,
        expected: u64,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pipeline_owner_current(&tx, owner)?;
        let wait = check_wait(&tx, owner, Some(expected))?.context("pipeline wait missing")?;
        ensure!(wait.state == "probing", "pipeline wait is not probing");
        let step = read_step(
            &tx,
            wait.probe_run.as_deref().context("probe run missing")?,
            wait.probe_seq.context("probe seq missing")?,
            wait.probe_attempt.context("probe attempt missing")?,
        )?;
        ensure!(
            step.owner_epoch == owner.epoch,
            "pipeline probe owner retired"
        );
        let run = read_run(&tx, &step.run_id)?;
        ensure!(
            run.owner_epoch == owner.epoch,
            "pipeline probe run owner retired"
        );
        pipeline_goal(&tx, &run, false)?;
        ensure!(
            !matches!(step.state.as_str(), "planned" | "launching" | "running"),
            "pipeline probe still active"
        );
        settle_wait(&tx, owner, Some(wait), &step)?;
        tx.commit()?;
        Ok(())
    }
}

fn executor_scopes(policy: &Value, tasks: &[TaskId], task: TaskId) -> Result<Vec<String>> {
    let scopes = policy["task_scopes"]
        .as_object()
        .context("task_scope_unresolved")?;
    // A single Task and a single operator key are unambiguous. Multiple keys require
    // a durable plan-key binding, which existing Goal acceptance does not retain.
    ensure!(
        tasks == [task] && scopes.len() == 1,
        "task_scope_unresolved"
    );
    let scope = scopes.values().next().context("task_scope_unresolved")?;
    let scope: Vec<String> =
        serde_json::from_value(scope.clone()).context("task_scope_unresolved")?;
    ensure!(
        !scope.is_empty()
            && scope.len() <= 64
            && scope.iter().all(|s| !s.is_empty()
                && s.len() <= 512
                && !s.starts_with('/')
                && !s.split('/').any(|p| p == "..")),
        "task_scope_unresolved"
    );
    Ok(scope)
}

#[cfg(test)]
mod tests;
