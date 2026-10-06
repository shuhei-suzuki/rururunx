//! Command-only verification contracts, exact grants and qualified receipt acceptance.
use super::*;
use crate::execution::verification::{
    AdmittedProfile, CollectedCommand, ManagedVerificationActivation, VerificationClaim,
    VerificationCompletion, VerificationGrant, VerificationRun, hash, json_hash,
};
use crate::workflow::{Actor, AttemptState, Phase, PhaseInvocation, WorkflowSnapshot};

pub(in crate::state) fn install_schema(tx: &Transaction<'_>) -> Result<()> {
    tx.execute_batch(include_str!("verification.sql"))?;
    Ok(())
}
pub(in crate::state) fn validate_legacy_namespace(tx: &Transaction<'_>) -> Result<()> {
    let n:u64=tx.query_row("SELECT COUNT(*) FROM sqlite_schema WHERE name IN ('verification_profiles','workflow_verification_contracts','verification_runs','verification_commands','verification_profile_no_update','verification_profile_no_delete','verification_contract_no_update','verification_contract_no_delete','verification_run_identity','verification_run_no_delete','verification_command_identity','verification_command_no_delete')",[],|r|r.get(0))?;
    ensure!(n == 0, "legacy verification namespace is not empty");
    Ok(())
}
pub(in crate::state) fn install_contract(
    tx: &Transaction<'_>,
    activation: &ManagedVerificationActivation,
    record: &Record,
    task: &Task,
) -> Result<()> {
    ensure!(
        activation.record() == record.id
            && activation.scope() == &record.scope
            && record.scope == task.scope()
            && record.version == 0,
        "managed verification activation binding mismatch"
    );
    let epoch: u64 = tx.query_row(
        "SELECT epoch FROM runtime_epoch WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    ensure!(
        epoch > 0 && epoch == activation.epoch(),
        "managed verification activation epoch retired"
    );
    let (p, g, t) = scope_keys(&task.scope())?;
    let digest: Option<String> = tx
        .query_row(
            "SELECT digest FROM verification_profiles WHERE project_id=?1",
            [&p],
            |r| r.get(0),
        )
        .optional()?;
    tx.execute("INSERT INTO workflow_verification_contracts(workflow_id,project_id,goal_id,task_id,owner_epoch,profile_digest) VALUES(?1,?2,?3,?4,?5,?6)",params![record.id.to_string(),p,g,t,epoch,digest])?;
    Ok(())
}
pub(in crate::state) fn is_command_unit(connection: &Connection, id: UnitId) -> Result<bool> {
    Ok(connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM verification_runs WHERE unit_id=?1)",
        [id.to_string()],
        |r| r.get(0),
    )?)
}
pub(in crate::state) fn requires_verification(
    connection: &Connection,
    id: RecordId,
) -> Result<bool> {
    Ok(connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM workflow_verification_contracts WHERE workflow_id=?1)",
        [id.to_string()],
        |r| r.get(0),
    )?)
}
fn profile(connection: &Connection, workflow: RecordId) -> Result<AdmittedProfile> {
    let row:Option<(String,String)>=connection.query_row("SELECT p.digest,p.body FROM workflow_verification_contracts c JOIN verification_profiles p ON p.project_id=c.project_id AND p.digest=c.profile_digest WHERE c.workflow_id=?1 AND length(CAST(p.body AS BLOB))<=1048576",[workflow.to_string()],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    let (digest,body)=row.context("managed Tests profile pending; explicit admission before fresh Workflow activation required")?;
    ensure!(
        hash(body.as_bytes()) == digest,
        "verification profile body/digest mismatch"
    );
    let p: AdmittedProfile = decode(body)?;
    p.validate()?;
    Ok(p)
}
fn bounded<T: serde::de::DeserializeOwned>(
    connection: &Connection,
    table: &str,
    id: &str,
    maximum: usize,
) -> Result<T> {
    ensure!(
        matches!(
            table,
            "tasks" | "projects" | "goals" | "records" | "result_artifacts"
        ),
        "invalid verifier snapshot table"
    );
    let n: usize = connection.query_row(
        &format!("SELECT length(CAST(body AS BLOB)) FROM {table} WHERE id=?1"),
        [id],
        |r| r.get(0),
    )?;
    ensure!(n <= maximum, "verifier authority snapshot exceeds bound");
    read_tx(connection, table, id)?.context("verifier authority snapshot missing")
}
fn context_read(connection: &Connection, scope: &Scope, version: u64) -> Result<ContextVersion> {
    let (body, n): (String, usize) = connection.query_row(
        "SELECT body,length(CAST(body AS BLOB)) FROM context_versions WHERE project_id=?1 AND owner=?2 AND version=?3 AND length(CAST(body AS BLOB))<=8388608",
        params![scope.project_id.to_string(),context_owner(scope)?,version], |r| Ok((r.get(0)?,r.get(1)?)))?;
    ensure!(n <= 8 * 1024 * 1024, "verifier Context exceeds bound");
    let context: ContextVersion = decode(body)?;
    ensure!(
        context.scope == *scope && context.version == version,
        "verifier Context indexed ownership differs"
    );
    Ok(context)
}
fn record_claim(
    connection: &Connection,
    invocation: &PhaseInvocation,
) -> Result<(VerificationClaim, Record)> {
    ensure!(
        invocation.phase == Phase::Tests && invocation.sources.scope == invocation.task.scope(),
        "verification requires exact Tests Scope"
    );
    let task: Task = bounded(
        connection,
        "tasks",
        &invocation.task.id.to_string(),
        1024 * 1024,
    )?;
    let project: Project = bounded(
        connection,
        "projects",
        &task.project_id.to_string(),
        1024 * 1024,
    )?;
    let goal: Goal = bounded(connection, "goals", &task.goal_id.to_string(), 1024 * 1024)?;
    ensure!(
        serde_json::to_value(&task)? == serde_json::to_value(&invocation.task)?
            && serde_json::to_value(&project)? == serde_json::to_value(&invocation.project)?
            && project.state == ProjectState::Registered
            && matches!(
                goal.state,
                GoalState::Created | GoalState::Analyzing | GoalState::Running
            )
            && !task_terminal(task.state),
        "verification owners stale/inactive"
    );
    let mut rows = connection.prepare(
        "SELECT id FROM records WHERE task_id=?1 AND kind='workflow' ORDER BY rowid LIMIT 2",
    )?;
    let records = rows
        .query_map([task.id.to_string()], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    ensure!(records.len() == 1, "verification requires sole Workflow");
    let record: Record = bounded(connection, "records", &records[0], 8 * 1024 * 1024)?;
    ensure!(
        requires_verification(connection, record.id)?,
        "verification requires activated managed contract"
    );
    let w: WorkflowSnapshot = serde_json::from_value(record.data.clone())?;
    let index = w.active.context("verification active claim missing")?;
    let a = w
        .history
        .get(index)
        .context("verification active index invalid")?;
    let context = context_read(connection, &task.scope(), a.context_version)?;
    crate::workflow::validate_transition(&task, &record, Some(&record))?;
    crate::workflow::validate_context(&task, &record, &context)?;
    ensure!(
        record.scope == task.scope() && record.kind == RecordKind::Workflow,
        "verification Workflow indexed owner mismatch"
    );
    ensure!(
        a.phase == Phase::Tests
            && a.state == AttemptState::Evaluating
            && a.generation == w.generation
            && a.context_version == task.context_version
            && a.claimed_observations == a.observations.len()
            && serde_json::to_value(&context)? == serde_json::to_value(&invocation.context)?
            && serde_json::to_value(&a.observations)?
                == serde_json::to_value(&invocation.prior_observations)?
            && serde_json::to_value(w.completed.values().collect::<Vec<_>>())?
                == serde_json::to_value(&invocation.prerequisites)?,
        "verification requires exact Evaluating Context/prerequisites"
    );
    ensure!(
        !w.configured_phases.contains(&Phase::ImpactAnalysis),
        "STANDARD/STRICT impact producer integration pending"
    );
    ensure!(
        context.revision == invocation.sources.revision
            && w.sources.scope == task.scope()
            && w.sources.revision == invocation.sources.revision
            && w.sources.artifact == invocation.sources.artifact
            && w.sources.source_versions == invocation.sources.source_versions,
        "verification source frame mismatch"
    );
    let artifact = self_artifact_tx(
        connection,
        invocation
            .sources
            .artifact
            .context("verification Published input missing")?,
    )?;
    ensure!(
        artifact.state == ArtifactState::Published
            && artifact.scope == task.scope()
            && artifact.revision == context.revision
            && task.revision.as_ref() == Some(&artifact.revision)
            && artifact
                .dependencies
                .iter()
                .all(
                    |(k, v)| invocation.sources.source_versions.get(k) == Some(v)
                        && context.source_hashes.get(k) == Some(v)
                ),
        "verification retained input mismatch"
    );
    let claim = VerificationClaim {
        record: record.id,
        version: record.version,
        index,
        scope: task.scope(),
        generation: w.generation,
        task_version: task.version,
        task_digest: json_hash(&task)?,
        project_version: project.version,
        project_digest: json_hash(&project)?,
        goal_version: goal.version,
        goal_digest: json_hash(&goal)?,
        workflow_digest: json_hash(&record)?,
        workflow_updated_at: record.updated_at,
        context_version: context.version,
        context_digest: json_hash(&context)?,
        source_digest: json_hash(&invocation.sources)?,
        observations: a.observations.len(),
        artifact,
    };
    Ok((claim, record))
}
fn run_read(connection: &Connection, unit: UnitId) -> Result<VerificationRun> {
    let (body,scope,workflow,profile_digest,generation,epoch):(String,(String,String,String),String,String,u64,u64)=connection.query_row("SELECT body,project_id,goal_id,task_id,workflow_id,profile_digest,generation,owner_epoch FROM verification_runs WHERE unit_id=?1 AND length(CAST(body AS BLOB))<=262144",[unit.to_string()],|r|Ok((r.get(0)?,(r.get(1)?,r.get(2)?,r.get(3)?),r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?)))?;
    let mut run: VerificationRun = decode(body)?;
    let u = unit_tx(connection, unit)?;
    ensure!(
        run.unit == unit
            && scope == scope_keys(&run.scope)?
            && run.scope == u.scope
            && run.workflow.to_string() == workflow
            && run.profile_digest == profile_digest
            && u.generation == generation
            && u.owner_epoch == epoch
            && run.commands.len() <= 32,
        "verification run indexed ownership mismatch"
    );
    let current_epoch: u64 = connection.query_row(
        "SELECT epoch FROM runtime_epoch WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    let current_generation: u64 = connection.query_row(
        "SELECT generation FROM task_execution WHERE task_id=?1",
        [run.scope.task_id.context("Task required")?.to_string()],
        |r| r.get(0),
    )?;
    run.historical =
        epoch != current_epoch || generation != current_generation || u.state == UnitState::Retired;
    Ok(run)
}
fn checked(
    connection: &Connection,
    grant: &VerificationGrant,
    native: bool,
) -> Result<ExecutionUnit> {
    let claim = grant.claim();
    let current = unit_tx(connection, grant.unit().id)?;
    ensure!(
        current.scope == grant.unit().scope
            && current.kind == UnitKind::Verifier
            && current.provider == "verifier"
            && current.phase == Phase::Tests.key()
            && current.generation == grant.unit().generation
            && current.owner_epoch == grant.unit().owner_epoch
            && current.session_id.is_none()
            && current.worktree == grant.unit().worktree
            && current.artifact_id == Some(claim.artifact.id)
            && current.base_sha == claim.artifact.revision
            && current.profile_digest == grant.unit().profile_digest,
        "command verifier identity changed"
    );
    validate_authority(connection, &current.authority(), native, !native)?;
    let row: Record = read_tx(connection, "records", &claim.record.to_string())?
        .context("verification Workflow missing")?;
    let w: WorkflowSnapshot = serde_json::from_value(row.data.clone())?;
    let a = w
        .history
        .get(claim.index)
        .context("verification attempt missing")?;
    let task: Task = read_tx(
        connection,
        "tasks",
        &claim.scope.task_id.unwrap().to_string(),
    )?
    .context("verification Task missing")?;
    let project: Project = bounded(
        connection,
        "projects",
        &task.project_id.to_string(),
        1024 * 1024,
    )?;
    let goal: Goal = bounded(connection, "goals", &task.goal_id.to_string(), 1024 * 1024)?;
    let context: ContextVersion = decode(connection.query_row(
        "SELECT body FROM context_versions WHERE project_id=?1 AND owner=?2 AND version=?3",
        params![
            task.project_id.to_string(),
            context_owner(&task.scope())?,
            claim.context_version
        ],
        |r| r.get::<_, String>(0),
    )?)?;
    let stored: String = connection.query_row(
        "SELECT claim FROM verification_runs WHERE unit_id=?1 AND profile_digest=?2",
        params![current.id.to_string(), grant.profile_digest()],
        |r| r.get(0),
    )?;
    ensure!(
        stored == serde_json::to_string(claim)?
            && row.version == claim.version
            && json_hash(&row)? == claim.workflow_digest
            && w.active == Some(claim.index)
            && w.generation == claim.generation
            && a.phase == Phase::Tests
            && a.state == AttemptState::Evaluating
            && a.unit.as_ref() == Some(&ManagedUnitRef::from(&current))
            && a.session_id.is_none()
            && a.execution.is_none()
            && a.claimed_observations == claim.observations
            && a.observations.len() == claim.observations
            && task.version == claim.task_version
            && json_hash(&task)? == claim.task_digest
            && task.context_version == claim.context_version
            && project.version == claim.project_version
            && json_hash(&project)? == claim.project_digest
            && goal.version == claim.goal_version
            && json_hash(&goal)? == claim.goal_digest
            && json_hash(&context)? == claim.context_digest
            && serde_json::to_value(self_artifact_tx(connection, claim.artifact.id)?)?
                == serde_json::to_value(&claim.artifact)?,
        "verification exact successor claim retired"
    );
    Ok(current)
}
impl Store {
    pub(crate) fn admit_verification_profile(
        &mut self,
        project_id: ProjectId,
        expected: u64,
        epoch: u64,
        p: &AdmittedProfile,
    ) -> Result<String> {
        p.validate()?;
        let body = serde_json::to_string(p)?;
        ensure!(
            body.len() <= 1048576,
            "verification admitted profile encoding limit"
        );
        let digest = hash(body.as_bytes());
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let project: Project = read_tx(&tx, "projects", &project_id.to_string())?
            .context("verification Project missing")?;
        let actual: u64 = tx.query_row(
            "SELECT epoch FROM runtime_epoch WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        ensure!(
            project.version == expected
                && project.state == ProjectState::Registered
                && actual == epoch
                && epoch > 0,
            "verification activation Project/epoch mismatch"
        );
        let activated: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM workflow_verification_contracts WHERE project_id=?1)",
            [project_id.to_string()],
            |r| r.get(0),
        )?;
        ensure!(
            !activated,
            "verification catalog must be admitted before managed Workflow activation"
        );
        tx.execute(
            "INSERT INTO verification_profiles(project_id,digest,body) VALUES(?1,?2,?3)",
            params![project_id.to_string(), digest, body],
        )?;
        tx.commit()?;
        Ok(digest)
    }
    pub(crate) fn verification_claim(
        &self,
        invocation: &PhaseInvocation,
    ) -> Result<VerificationClaim> {
        Ok(record_claim(&self.connection, invocation)?.0)
    }
    pub(crate) fn verification_profile_for(&self, workflow: RecordId) -> Result<AdmittedProfile> {
        profile(&self.connection, workflow)
    }
    pub(crate) fn workflow_requires_verification(&self, workflow: RecordId) -> Result<bool> {
        requires_verification(&self.connection, workflow)
    }
    pub fn verification_run(&self, unit: UnitId) -> Result<VerificationRun> {
        run_read(&self.connection, unit)
    }
    pub(crate) fn validate_verification(
        &self,
        grant: &VerificationGrant,
        native: bool,
    ) -> Result<()> {
        checked(&self.connection, grant, native)?;
        Ok(())
    }
    pub(crate) fn reserve_verification(
        &mut self,
        invocation: &PhaseInvocation,
        origin: &VerificationClaim,
        mut unit: ExecutionUnit,
        p: &AdmittedProfile,
    ) -> Result<(VerificationGrant, Record)> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (mut claim, mut record) = record_claim(&tx, invocation)?;
        let source_advance = source_recovery::before_write(
            &tx,
            claim.scope.task_id.context("Task required")?,
            false,
        )?;
        ensure!(
            serde_json::to_value(&claim)? == serde_json::to_value(origin)?,
            "verification origin claim changed before reservation"
        );
        let admitted = profile(&tx, claim.record)?;
        ensure!(
            json_hash(&admitted)? == json_hash(p)?,
            "verification profile changed before admission"
        );
        let mut w: WorkflowSnapshot = serde_json::from_value(record.data.clone())?;
        let a = &w.history[claim.index];
        ensure!(
            a.unit.is_none() && a.session_id.is_none() && a.execution.is_none(),
            "fresh verification unit required; no namespace reuse"
        );
        let epoch: u64 = tx.query_row(
            "SELECT epoch FROM runtime_epoch WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        let generation: u64 = tx.query_row(
            "SELECT generation FROM task_execution WHERE task_id=?1",
            [unit.scope.task_id.unwrap().to_string()],
            |r| r.get(0),
        )?;
        ensure!(
            unit.kind == UnitKind::Verifier
                && unit.provider == "verifier"
                && unit.state == UnitState::Reserved
                && unit.version == 0
                && unit.generation == 0
                && unit.owner_epoch == epoch
                && epoch > 0
                && generation > 0
                && unit.scope == claim.scope
                && unit.phase == Phase::Tests.key()
                && unit.session_id.is_none()
                && unit.branch.is_none()
                && unit.base_sha == claim.artifact.revision
                && unit.artifact_id == Some(claim.artifact.id)
                && unit.work.is_none()
                && unit.native_effects_open
                && unit.result_finalization_open,
            "invalid command verifier admission"
        );
        unit.generation = generation;
        unit.version = 1;
        insert_unit(&tx, &unit)?;
        let project: Project =
            read_tx(&tx, "projects", &unit.scope.project_id.to_string())?.unwrap();
        let goal: Goal = read_tx(&tx, "goals", &unit.scope.goal_id.unwrap().to_string())?.unwrap();
        tx.execute("INSERT INTO execution_context(unit_id,project_version,goal_version,governing_digest) VALUES(?1,?2,?3,?4)",params![unit.id.to_string(),project.version,goal.version,governing_digest(&project,&goal)?])?;
        let before = record.clone();
        w.history[claim.index].unit = Some((&unit).into());
        record.data = serde_json::to_value(w)?;
        crate::workflow::validate_transition(&invocation.task, &record, Some(&before))?;
        record = put_record_tx(&tx, &record)?;
        claim.version = record.version;
        claim.workflow_digest = json_hash(&record)?;
        claim.workflow_updated_at = record.updated_at;
        let pd = json_hash(p)?;
        let run = VerificationRun {
            unit: unit.id,
            scope: unit.scope.clone(),
            workflow: record.id,
            artifact: claim.artifact.id,
            revision: unit.base_sha.clone(),
            profile_digest: pd.clone(),
            commands: vec![],
            work: WorkOutcome::Unknown,
            certifying: false,
            historical: false,
        };
        let (project, goal, task) = scope_keys(&unit.scope)?;
        tx.execute("INSERT INTO verification_runs(unit_id,workflow_id,project_id,goal_id,task_id,generation,owner_epoch,profile_digest,state,version,claim_digest,claim,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,'admitted',1,?9,?10,?11)",params![unit.id.to_string(),record.id.to_string(),project,goal,task,unit.generation,epoch,pd,json_hash(&claim)?,serde_json::to_string(&claim)?,serde_json::to_string(&run)?])?;
        append_event(
            &tx,
            &unit.scope,
            "verification.admitted",
            json!({"unit":unit.id,"workflow":record.id,"profile":pd}),
        )?;
        source_recovery::after_write(&tx, source_advance, false)?;
        tx.commit()?;
        Ok((VerificationGrant::admitted(claim, unit, pd), record))
    }
    pub(crate) fn reserve_verification_command(
        &mut self,
        grant: &VerificationGrant,
        index: usize,
        operation: OperationId,
        cwd: &std::path::Path,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = checked(&tx, grant, true)?;
        let p = profile(&tx, grant.claim().record)?;
        let command = p
            .proposal
            .commands
            .get(index)
            .context("verification command index missing")?;
        let n: u64 = tx.query_row(
            "SELECT COUNT(*) FROM verification_commands WHERE unit_id=?1",
            [unit.id.to_string()],
            |r| r.get(0),
        )?;
        ensure!(n==index as u64 && (index==0 || tx.query_row("SELECT EXISTS(SELECT 1 FROM verification_commands WHERE unit_id=?1 AND state='pending')",[unit.id.to_string()],|r|r.get::<_,bool>(0))?==false),"verification command sequencing mismatch");
        ensure!(
            cwd == std::fs::canonicalize(unit.worktree.join(&command.cwd))?
                && cwd.starts_with(&unit.worktree),
            "verification command cwd differs from admitted input"
        );
        let effect = ManagedEffect {
            id: operation,
            unit_id: unit.id,
            scope: unit.scope.clone(),
            kind: "verification_command".into(),
            idempotency_key: format!("verification-{operation}"),
            expected_target: cwd.to_string_lossy().into(),
            state: EffectState::Pending,
            receipt: BTreeMap::new(),
            version: 1,
        };
        let (p, g, t) = scope_keys(&unit.scope)?;
        tx.execute("INSERT INTO managed_effects(id,unit_id,project_id,goal_id,task_id,idempotency_key,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,'pending',1,?7)",params![operation.to_string(),unit.id.to_string(),p,g,t,effect.idempotency_key,serde_json::to_string(&effect)?])?;
        tx.execute("INSERT INTO verification_commands(unit_id,ordinal,operation_id,state) VALUES(?1,?2,?3,'pending')",params![unit.id.to_string(),index,operation.to_string()])?;
        unit.state = UnitState::Running;
        write_unit(&tx, &mut unit)?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn complete_verification_command(
        &mut self,
        grant: &VerificationGrant,
        index: usize,
        captured: &CollectedCommand,
    ) -> Result<()> {
        let o = captured.observation();
        let body = serde_json::to_string(o)?;
        ensure!(body.len() <= 65536, "verification command receipt bound");
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let unit = unit_tx(&tx, grant.unit().id)?;
        ensure!(
            unit.scope == grant.unit().scope
                && unit.generation == grant.unit().generation
                && unit.owner_epoch == grant.unit().owner_epoch
                && unit.session_id.is_none(),
            "historical command receipt owner changed"
        );
        let profile = profile(&tx, grant.claim().record)?;
        let c = profile
            .proposal
            .commands
            .get(index)
            .context("verification command index missing")?;
        ensure!(
            o.command == c.id
                && o.timeout_seconds == c.timeout_seconds
                && o.drain_seconds == c.drain_seconds
                && o.stdout.bytes <= c.stdout_bytes as u64
                && o.stderr.bytes <= c.stderr_bytes as u64
                && o.finished_at >= o.started_at
                && o.started_at >= 0,
            "verification collector receipt does not match admitted command"
        );
        captured.validate(c)?;
        let e = effect_tx(&tx, o.operation)?;
        ensure!(
            e.unit_id == unit.id && e.scope == unit.scope && e.kind == "verification_command",
            "verification command intent mismatch"
        );
        ensure!(tx.execute("UPDATE verification_commands SET state='terminal',body=?1 WHERE unit_id=?2 AND ordinal=?3 AND operation_id=?4 AND state='pending'",params![body,unit.id.to_string(),index,o.operation.to_string()])?==1,"verification command terminal replay/mismatch");
        let mut effect = e;
        let expected = effect.version;
        effect.version = expected.checked_add(1).context("effect version overflow")?;
        effect.state = if o.work_known {
            EffectState::Confirmed
        } else {
            EffectState::Unknown
        };
        effect.receipt = BTreeMap::from([(
            "result".into(),
            if o.certifying() {
                "passed"
            } else {
                "noncertifying"
            }
            .into(),
        )]);
        ensure!(
            tx.execute(
                "UPDATE managed_effects SET state=?1,version=?2,body=?3 WHERE id=?4 AND version=?5",
                params![
                    key(effect.state),
                    effect.version,
                    serde_json::to_string(&effect)?,
                    effect.id.to_string(),
                    expected
                ]
            )? == 1,
            "verification effect terminal CAS mismatch"
        );
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn retire_verification(
        &mut self,
        grant: &VerificationGrant,
    ) -> Result<ExecutionUnit> {
        let current = unit_tx(&self.connection, grant.unit().id)?;
        ensure!(
            current.scope == grant.unit().scope
                && current.generation == grant.unit().generation
                && current.owner_epoch == grant.unit().owner_epoch
                && is_command_unit(&self.connection, current.id)?,
            "verification retirement owner mismatch"
        );
        self.retire_execution(&current.authority(), false)
    }
    pub(crate) fn finish_verification(
        &mut self,
        grant: &VerificationGrant,
        commands: &[crate::execution::verification::CommandObservation],
        snapshot: &str,
    ) -> Result<VerificationRun> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = checked(&tx, grant, true)?;
        let p = profile(&tx, grant.claim().record)?;
        let mut query=tx.prepare("SELECT body FROM verification_commands WHERE unit_id=?1 AND state='terminal' ORDER BY ordinal LIMIT 33")?;
        let bodies = query
            .query_map([unit.id.to_string()], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(query);
        let persisted = bodies
            .into_iter()
            .map(decode::<crate::execution::verification::CommandObservation>)
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            serde_json::to_value(&persisted)? == serde_json::to_value(commands)?
                && commands.len() <= p.proposal.commands.len()
                && snapshot.len() == 64,
            "verification terminal observations mismatch"
        );
        let certifying =
            commands.len() == p.proposal.commands.len() && commands.iter().all(|o| o.certifying());
        let work = if commands.iter().any(|o| {
            o.work_known
                && (o.exit.is_some_and(|c| c != 0) || o.signal.is_some())
                && o.issue.is_none()
        }) {
            WorkOutcome::Failure
        } else if commands.len() == p.proposal.commands.len()
            && commands.iter().all(|o| o.work_known && o.exit == Some(0))
        {
            WorkOutcome::Success
        } else {
            WorkOutcome::Unknown
        };
        let mut run = run_read(&tx, unit.id)?;
        run.commands = commands.to_vec();
        run.work = work;
        run.certifying = certifying;
        ensure!(tx.execute("UPDATE verification_runs SET state='terminal',version=version+1,snapshot_digest=?1,body=?2 WHERE unit_id=?3 AND state='admitted'",params![snapshot,serde_json::to_string(&run)?,unit.id.to_string()])?==1,"verification run terminal CAS mismatch");
        unit.work = Some(work);
        unit.native_effects_open = false;
        unit.result_finalization_open = certifying;
        unit.state = if work == WorkOutcome::Unknown {
            UnitState::WorkUnknown
        } else {
            UnitState::WorkKnown
        };
        unit.disposition = Disposition::Completed;
        write_unit(&tx, &mut unit)?;
        tx.execute("INSERT INTO cleanup_jobs(unit_id,next_due,attempts,version) VALUES(?1,?2,0,1) ON CONFLICT(unit_id) DO NOTHING",params![unit.id.to_string(),now_ms()])?;
        append_event(
            &tx,
            &unit.scope,
            "verification.terminal",
            json!({"unit":unit.id,"work":work,"certifying":certifying,"commands":commands.len()}),
        )?;
        tx.commit()?;
        Ok(run)
    }
}

pub(in crate::state) fn accept_tx(
    tx: &Transaction<'_>,
    proof: &VerificationCompletion,
    task: &Task,
    next: &Record,
    previous: &Record,
    context: &ContextVersion,
) -> Result<()> {
    let grant = proof.grant();
    let claim = grant.claim();
    let mut unit = unit_tx(tx, grant.unit().id)?;
    validate_authority(tx, &unit.authority(), false, true)?;
    let before: WorkflowSnapshot = serde_json::from_value(previous.data.clone())?;
    let after: WorkflowSnapshot = serde_json::from_value(next.data.clone())?;
    let old = before
        .history
        .get(claim.index)
        .context("verification acceptance index missing")?;
    let new = after
        .history
        .get(claim.index)
        .context("verification acceptance history missing")?;
    let run = run_read(tx, unit.id)?;
    let old_task: Task = read_tx(tx, "tasks", &task.id.to_string())?.unwrap();
    let old_context: ContextVersion = decode(tx.query_row(
        "SELECT body FROM context_versions WHERE project_id=?1 AND owner=?2 AND version=?3",
        params![
            task.project_id.to_string(),
            context_owner(&task.scope())?,
            claim.context_version
        ],
        |r| r.get::<_, String>(0),
    )?)?;
    let artifact = self_artifact_tx(tx, claim.artifact.id)?;
    ensure!(
        previous.id == claim.record
            && previous.version
                == claim
                    .version
                    .checked_add(1)
                    .context("verification observation version overflow")?
            && before.active == Some(claim.index)
            && before.generation == claim.generation
            && after.generation == claim.generation
            && after.active.is_none()
            && old.phase == Phase::Tests
            && old.phase.actor() == Actor::EvidencePort
            && old.state == AttemptState::Evaluating
            && new.state == AttemptState::Succeeded
            && old.unit.as_ref() == Some(&ManagedUnitRef::from(&unit))
            && old.session_id.is_none()
            && old.execution.is_none()
            && unit.kind == UnitKind::Verifier
            && unit.provider == "verifier"
            && unit.session_id.is_none()
            && unit.scope == claim.scope
            && unit.generation == grant.unit().generation
            && unit.owner_epoch == grant.unit().owner_epoch
            && unit.work == Some(WorkOutcome::Success)
            && !unit.native_effects_open,
        "verification accepted claim/terminal mismatch"
    );
    ensure!(
        old_task.version == claim.task_version
            && old_task.context_version == claim.context_version
            && json_hash(&old_context)? == claim.context_digest
            && serde_json::to_value(&artifact)? == serde_json::to_value(&claim.artifact)?
            && run.certifying
            && !run.historical
            && json_hash(&run)? == proof.digest()
            && json_hash(proof.run())? == proof.digest(),
        "verification input/receipt proof changed before acceptance"
    );
    let mut origin_record = previous.clone();
    let mut origin = before.clone();
    ensure!(
        origin.history[claim.index].observations.len()
            == claim
                .observations
                .checked_add(1)
                .context("observation count overflow")?,
        "verification observation count changed"
    );
    origin.history[claim.index].observations.pop();
    origin_record.data = serde_json::to_value(origin)?;
    origin_record.version = claim.version;
    origin_record.updated_at = claim.workflow_updated_at;
    let project: Project = bounded(tx, "projects", &task.project_id.to_string(), 1024 * 1024)?;
    let goal: Goal = bounded(tx, "goals", &task.goal_id.to_string(), 1024 * 1024)?;
    ensure!(
        json_hash(&origin_record)? == claim.workflow_digest
            && json_hash(&old_task)? == claim.task_digest
            && json_hash(&project)? == claim.project_digest
            && project.version == claim.project_version
            && json_hash(&goal)? == claim.goal_digest
            && goal.version == claim.goal_version,
        "verification whole authority changed beyond audited observation"
    );
    let evidence = after
        .completed
        .get(&Phase::Tests)
        .context("verification evidence missing")?;
    let marker = format!("rrx-verification:{}:{}", unit.id, proof.digest());
    ensure!(evidence.artifacts==vec![marker] && evidence.scope==unit.scope && evidence.phase==Phase::Tests && evidence.revision==artifact.revision && evidence.session_id.is_none() && evidence.review_approved.is_none() && evidence.context_version==claim.context_version && evidence.source_versions==before.sources.source_versions && evidence.dependencies==before.sources.source_versions && old.observations.len()==claim.observations+1 && old.claimed_observations==claim.observations && old.observations.last().is_some_and(|o|matches!(&o.outcome,Some(crate::workflow::GateOutcome::Passed(e)) if serde_json::to_value(e).ok()==serde_json::to_value(evidence).ok())) && task.revision.as_ref()==Some(&artifact.revision) && context.revision==artifact.revision && after.sources.artifact==Some(artifact.id) && after.sources.revision==artifact.revision && artifact.dependencies.iter().all(|(k,v)|context.source_hashes.get(k)==Some(v) && after.sources.source_versions.get(k)==Some(v)),"verification required coverage/observation/dependency mismatch");
    let p = profile(tx, claim.record)?;
    ensure!(
        run.profile_digest == json_hash(&p)?
            && run.commands.len() == p.proposal.commands.len()
            && run
                .commands
                .iter()
                .zip(&p.proposal.commands)
                .all(|(o, c)| o.command == c.id && o.certifying()),
        "verification activated plan coverage changed"
    );
    ensure!(tx.execute("UPDATE verification_runs SET state='accepted',version=version+1 WHERE unit_id=?1 AND state='terminal' AND body=?2",params![unit.id.to_string(),serde_json::to_string(&run)?])?==1,"verification accepted receipt CAS mismatch");
    unit.result_finalization_open = false;
    write_unit(tx, &mut unit)?;
    append_event(
        tx,
        &unit.scope,
        "verification.accepted",
        json!({"unit":unit.id,"workflow":next.id,"sha":artifact.revision,"receipt":proof.digest()}),
    )?;
    Ok(())
}
