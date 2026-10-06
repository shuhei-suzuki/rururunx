//! Native-only exact row plans. SQL images remain nongrant; registration still
//! needs the genuine retained launch and the actual Native issuer after commit.
use super::*;
use crate::{
    execution::native::{NativePhaseBinding, NativePhaseSession, NativeSeed},
    state::managed_binding::{
        CurrentWorkflowSuccessor, ExactRowMutation, PhaseLaunchParts, phase_pair_columns,
        plan_current_phase, snapshot, validate_current_tx,
    },
};
use rusqlite::{params_from_iter, types::Value as SqlValue};
use std::sync::Arc;

const PAIR_BODY_BYTES: usize = 32 * 1024;

fn selected_database(connection: &rusqlite::Connection, launch: &PhaseLaunchParts) -> Result<()> {
    ensure!(
        launch
            .allocation()
            .facts()
            .state_path
            .to_str()
            .is_some_and(|path| connection.path() == Some(path)),
        "Native phase writer is not the selected owner's database"
    );
    Ok(())
}

/// Complete indexed image, never a persisted owner-to-authority conversion.
struct PairRow {
    table: &'static str,
    values: Vec<SqlValue>,
}
impl PairRow {
    fn read(tx: &Transaction<'_>, table: &'static str, key: &str) -> Result<Self> {
        let columns = phase_pair_columns(table).context("unsupported Native pair table")?;
        ensure!(
            matches!(
                table,
                "managed_phase_owners"
                    | "managed_phase_readiness"
                    | "records"
                    | "managed_phase_admissions"
            ),
            "invalid Native pair read"
        );
        let select = columns
            .iter()
            .map(|name| {
                if *name == "body" {
                    "CASE WHEN length(CAST(body AS BLOB))<=32768 THEN body END".to_owned()
                } else {
                    (*name).to_owned()
                }
            })
            .collect::<Vec<_>>();
        let values = tx.query_row(
            &format!(
                "SELECT {} FROM {table} WHERE {}=?1",
                select.join(","),
                columns[0]
            ),
            [key],
            |row| {
                (0..columns.len())
                    .map(|i| row.get(i))
                    .collect::<rusqlite::Result<Vec<SqlValue>>>()
            },
        )?;
        let image = Self { table, values };
        image.body()?;
        Ok(image)
    }
    fn column(&self, name: &str) -> Result<&SqlValue> {
        let index = phase_pair_columns(self.table)
            .context("Native columns absent")?
            .iter()
            .position(|column| *column == name)
            .context("Native column absent")?;
        self.values.get(index).context("incomplete Native image")
    }
    fn replace(&mut self, name: &str, value: SqlValue) -> Result<()> {
        let index = phase_pair_columns(self.table)
            .context("Native columns absent")?
            .iter()
            .position(|column| *column == name)
            .context("Native column absent")?;
        *self
            .values
            .get_mut(index)
            .context("incomplete Native image")? = value;
        Ok(())
    }
    fn body(&self) -> Result<Value> {
        let SqlValue::Text(raw) = self.column("body")? else {
            anyhow::bail!("bounded Native pair body absent")
        };
        Ok(crate::execution::strict_json::decode(
            raw.as_bytes(),
            crate::execution::strict_json::Limits {
                frame_bytes: PAIR_BODY_BYTES,
                depth: 12,
                nodes: 256,
                string_bytes: 4096,
                total_string_bytes: PAIR_BODY_BYTES,
                object_entries: 64,
                array_entries: 16,
            },
        )?)
    }
    fn set_body(&mut self, body: &Value) -> Result<()> {
        let raw = serde_json::to_string(body)?;
        ensure!(
            raw.len() <= PAIR_BODY_BYTES,
            "Native pair image exceeds finite profile"
        );
        self.replace("body", SqlValue::Text(raw))?;
        self.body()?;
        Ok(())
    }
    fn validate_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        let columns = phase_pair_columns(self.table).context("Native columns absent")?;
        let predicates = columns
            .iter()
            .enumerate()
            .map(|(i, name)| format!("{name} IS ?{}", i + 1))
            .collect::<Vec<_>>();
        let exact: bool = tx.query_row(
            &format!(
                "SELECT EXISTS(SELECT 1 FROM {} WHERE {})",
                self.table,
                predicates.join(" AND ")
            ),
            params_from_iter(&self.values),
            |r| r.get(0),
        )?;
        ensure!(exact, "Native pair complete image changed");
        Ok(())
    }
    fn update_tx(&self, tx: &Transaction<'_>, next: &Self) -> Result<()> {
        ensure!(self.table == next.table, "different Native pair update");
        let columns = phase_pair_columns(self.table).context("Native columns absent")?;
        let set = columns
            .iter()
            .enumerate()
            .map(|(i, name)| format!("{name}=?{}", i + 1))
            .collect::<Vec<_>>();
        let predicates = columns
            .iter()
            .enumerate()
            .map(|(i, name)| format!("{name} IS ?{}", columns.len() + i + 1))
            .collect::<Vec<_>>();
        let values = next.values.iter().chain(&self.values);
        ensure!(
            tx.execute(
                &format!(
                    "UPDATE {} SET {} WHERE {}",
                    self.table,
                    set.join(","),
                    predicates.join(" AND ")
                ),
                params_from_iter(values)
            )? == 1,
            "Native pair update CAS changed"
        );
        Ok(())
    }
    fn update_permission(&self, next: &Self) -> Result<ExactRowMutation> {
        ExactRowMutation::new(
            self.table,
            "UPDATE",
            Some(self.values.clone()),
            Some(next.values.clone()),
        )
    }
}

/// A current observation is tied to a real Core-issued owner, not a Session
/// lookup. All decoding/comparison/planned encodings happen outside SharedStore.
struct NativeOwnerPlan {
    binding: NativePhaseBinding,
    current: CurrentWorkflowSuccessor,
    session: PairRow,
    owner: PairRow,
    readiness: PairRow,
}
fn plan_native_owner(
    runtime: &crate::execution::RuntimeOwner,
    phase: &Arc<NativePhaseSession>,
) -> Result<NativeOwnerPlan> {
    let binding = phase.binding_snapshot()?;
    ensure!(
        binding.is_live() && phase.launch_parts().is_retained(),
        "actual Native owner ended"
    );
    let current = plan_current_phase(runtime, phase.marker())?;
    let f = phase.allocation().facts();
    let unit = current.unit();
    ensure!(
        unit.session_id == Some(f.session_id)
            && unit.native_effects_open
            && unit.result_finalization_open
            && unit.work.is_none()
            && unit.disposition == Disposition::Active
            && matches!(
                unit.state,
                UnitState::DispatchPending | UnitState::Running | UnitState::WaitingQuota
            ),
        "actual Native owner Unit is not open"
    );
    let (session, owner, readiness) = snapshot(runtime, |tx| {
        validate_current_tx(tx, phase.marker(), &current)?;
        phase.marker().validate_driver_live_tx(tx)?;
        let session = PairRow::read(tx, "records", &f.session_id.to_string())?;
        let body = session.body()?;
        let record: Record = serde_json::from_value(body)?;
        let stored: Session = serde_json::from_value(record.data.clone())?;
        ensure!(
            record.id == RecordId(f.session_id.0)
                && record.kind == RecordKind::Session
                && record.scope == *f.scope
                && record.version == binding.record_version()
                && serde_json::to_value(&stored)? == record.data
                && serde_json::to_value(&stored)? == serde_json::to_value(binding.session())?
                && matches!(stored.state, SessionState::Starting | SessionState::Running),
            "actual Native Session projection changed"
        );
        let checks = [
            ("id", SqlValue::Text(record.id.to_string())),
            ("kind", SqlValue::Text("session".into())),
            ("project_id", SqlValue::Text(f.scope.project_id.to_string())),
            (
                "goal_id",
                SqlValue::Text(f.scope.goal_id.context("Native Goal absent")?.to_string()),
            ),
            (
                "task_id",
                SqlValue::Text(f.scope.task_id.context("Native Task absent")?.to_string()),
            ),
            ("version", SqlValue::Integer(i64::try_from(record.version)?)),
        ];
        for (name, value) in checks {
            ensure!(
                session.column(name)? == &value,
                "Native Session indexed image changed"
            );
        }
        let owner = PairRow::read(tx, "managed_phase_owners", &f.pair_id.to_string())?;
        let expected = json!({"owner_id":f.pair_id,"operation_id":f.operation_id,"scope":f.scope,
            "unit_id":f.unit_id,"owner_epoch":f.epoch,"execution_generation":f.generation,
            "allocated_session_id":f.session_id,"provider":f.provider,"alias":f.alias,"role":f.role,
            "worktree":f.path,"origin":f.origin_id,"native_invocation_id":f.invocation_id,"validated":true,"version":2});
        ensure!(
            owner.body()? == expected
                && owner.column("native_invocation_id")?
                    == &SqlValue::Text(f.invocation_id.to_string())
                && owner.column("validated")? == &SqlValue::Integer(1)
                && owner.column("version")? == &SqlValue::Integer(2),
            "actual registered Native owner image changed"
        );
        check_owner_indices(&owner, phase.launch_parts(), true)?;
        let readiness = PairRow::read(tx, "managed_phase_readiness", &f.operation_id.to_string())?;
        let r = readiness.body()?;
        let version = r["version"]
            .as_u64()
            .context("Native readiness version absent")?;
        let ended = r["start_ended"]
            .as_bool()
            .context("Native readiness end flag absent")?;
        ensure!(
            r == json!({"operation_id":f.operation_id,"origin":f.origin_id,"owner_epoch":f.epoch,
            "state":"registered","start_ended":ended,"known_terminal":false,"parking_version":null,"version":version})
                && ((version == 2 && !ended) || (version == 3 && ended))
                && readiness.column("operation_id")? == &SqlValue::Text(f.operation_id.to_string())
                && readiness.column("origin")? == &SqlValue::Text(f.origin_id.to_string())
                && readiness.column("owner_epoch")? == &SqlValue::Integer(i64::try_from(f.epoch)?)
                && readiness.column("state")? == &SqlValue::Text("registered".into())
                && readiness.column("start_ended")? == &SqlValue::Integer(i64::from(ended))
                && readiness.column("known_terminal")? == &SqlValue::Integer(0)
                && readiness.column("parking_version")? == &SqlValue::Null
                && readiness.column("version")? == &SqlValue::Integer(i64::try_from(version)?),
            "actual Native readiness image changed"
        );
        let invocation = native_results::invocation_tx(tx, f.invocation_id)?;
        ensure!(
            invocation.id == f.invocation_id
                && invocation.session_id == f.session_id
                && invocation.unit_id == f.unit_id
                && invocation.scope == *f.scope
                && invocation.generation == f.generation
                && invocation.owner_epoch == f.epoch
                && invocation.provider == f.provider,
            "actual Native invocation changed"
        );
        Ok((session, owner, readiness))
    })?;
    Ok(NativeOwnerPlan {
        binding,
        current,
        session,
        owner,
        readiness,
    })
}
impl NativeOwnerPlan {
    fn validate_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        ensure!(
            self.binding.is_live() && self.binding.owner().launch_parts().is_retained(),
            "actual Native owner revoked"
        );
        validate_current_tx(tx, self.binding.marker(), &self.current)?;
        self.binding.marker().validate_driver_live_tx(tx)?;
        validate_authority(tx, &self.current.unit().authority(), true, false)?;
        self.session.validate_tx(tx)?;
        self.owner.validate_tx(tx)?;
        self.readiness.validate_tx(tx)?;
        Ok(())
    }
}

pub(crate) struct NativeProjectionPlan {
    before: NativeOwnerPlan,
    after: PairRow,
    session: Session,
    unit: ExecutionUnit,
    unit_raw: String,
}
fn plan_phase_projection(
    runtime: &crate::execution::RuntimeOwner,
    phase: &Arc<NativePhaseSession>,
    session: Session,
) -> Result<NativeProjectionPlan> {
    let before = plan_native_owner(runtime, phase)?;
    let old = before.binding.session();
    ensure!(
        session.id == old.id
            && session.scope == old.scope
            && session.agent == old.agent
            && session.provider == old.provider
            && session.role == old.role
            && session.worktree == old.worktree
            && session.model == old.model
            && session.effort == old.effort
            && session.started_at == old.started_at
            && session.recovery == old.recovery
            && session.state == SessionState::Running
            && session.native_ref.as_ref().is_some_and(|s| !s.is_empty()
                && s.len() <= 512
                && !s.chars().any(char::is_control))
            && old
                .native_ref
                .as_ref()
                .is_none_or(|s| session.native_ref.as_ref() == Some(s)),
        "Native factual acknowledgement changes immutable Session"
    );
    let mut record: Record = serde_json::from_value(before.session.body()?)?;
    record.version = record
        .version
        .checked_add(1)
        .context("Native Session version exhausted")?;
    record.updated_at = now_ms();
    record.data = serde_json::to_value(&session)?;
    let mut after = PairRow {
        table: before.session.table,
        values: before.session.values.clone(),
    };
    after.replace("version", SqlValue::Integer(i64::try_from(record.version)?))?;
    after.set_body(&serde_json::to_value(&record)?)?;
    let mut unit = before.current.unit().clone();
    unit.version = unit
        .version
        .checked_add(1)
        .context("Native Unit version exhausted")?;
    unit.updated_at = record.updated_at;
    // A telemetry wait is derived from quota facts. Session ACK cannot silently
    // clear an active native quota wait or erase its reason.
    if unit.state != UnitState::WaitingQuota {
        unit.state = UnitState::Running;
    }
    let unit_raw = serde_json::to_string(&unit)?;
    ensure!(
        unit_raw.len() <= 16 * 1024,
        "Native Unit projection exceeds finite profile"
    );
    Ok(NativeProjectionPlan {
        before,
        after,
        session,
        unit,
        unit_raw,
    })
}

/// A finite unconsumed registration plan. No public ID/DTO can construct this.
/// The retained launch is rechecked at commit; this value is not a Native owner.
pub(crate) struct NativeRegistrationPlan {
    launch: Arc<PhaseLaunchParts>,
    current: CurrentWorkflowSuccessor,
    owner_before: PairRow,
    owner_after: PairRow,
    readiness_before: PairRow,
    readiness_after: PairRow,
    session: Session,
    record: Record,
    record_raw: String,
    unit_after: ExecutionUnit,
    unit_raw: String,
    invocation: crate::execution::native_result::NativeInvocation,
    invocation_raw: String,
}

fn original_owner(tx: &Transaction<'_>, launch: &PhaseLaunchParts) -> Result<PairRow> {
    let f = launch.allocation().facts();
    let owner = PairRow::read(tx, "managed_phase_owners", &f.pair_id.to_string())?;
    let expected = json!({"owner_id":f.pair_id,"operation_id":f.operation_id,
        "scope":f.scope,"unit_id":f.unit_id,"owner_epoch":f.epoch,
        "execution_generation":f.generation,"allocated_session_id":f.session_id,
        "provider":f.provider,"alias":f.alias,"role":f.role,"worktree":f.path,
        "origin":f.origin_id,"native_invocation_id":null,"validated":false,"version":1});
    ensure!(
        owner.body()? == expected,
        "original unregistered Native owner body changed"
    );
    check_owner_indices(&owner, launch, false)?;
    Ok(owner)
}
fn check_owner_indices(owner: &PairRow, launch: &PhaseLaunchParts, registered: bool) -> Result<()> {
    let f = launch.allocation().facts();
    let checks = [
        ("owner_id", SqlValue::Text(f.pair_id.to_string())),
        ("operation_id", SqlValue::Text(f.operation_id.to_string())),
        ("project_id", SqlValue::Text(f.scope.project_id.to_string())),
        (
            "goal_id",
            SqlValue::Text(f.scope.goal_id.context("Native Goal absent")?.to_string()),
        ),
        (
            "task_id",
            SqlValue::Text(f.scope.task_id.context("Native Task absent")?.to_string()),
        ),
        ("unit_id", SqlValue::Text(f.unit_id.to_string())),
        ("owner_epoch", SqlValue::Integer(i64::try_from(f.epoch)?)),
        (
            "execution_generation",
            SqlValue::Integer(i64::try_from(f.generation)?),
        ),
        (
            "allocated_session_id",
            SqlValue::Text(f.session_id.to_string()),
        ),
        ("provider", SqlValue::Text(f.provider.to_owned())),
        ("alias", SqlValue::Text(f.alias.to_owned())),
        (
            "role",
            SqlValue::Text(
                serde_json::to_value(f.role)?
                    .as_str()
                    .context("Native role absent")?
                    .to_owned(),
            ),
        ),
        (
            "worktree",
            SqlValue::Text(f.path.to_str().context("Native path not UTF-8")?.to_owned()),
        ),
        ("origin", SqlValue::Text(f.origin_id.to_string())),
        (
            "native_invocation_id",
            if registered {
                SqlValue::Text(f.invocation_id.to_string())
            } else {
                SqlValue::Null
            },
        ),
        (
            "validated",
            SqlValue::Integer(if registered { 1 } else { 0 }),
        ),
        ("version", SqlValue::Integer(if registered { 2 } else { 1 })),
    ];
    for (column, value) in checks {
        ensure!(
            owner.column(column)? == &value,
            "original Native owner index changed"
        );
    }
    Ok(())
}
fn initial_readiness(tx: &Transaction<'_>, launch: &PhaseLaunchParts) -> Result<PairRow> {
    let f = launch.allocation().facts();
    let row = PairRow::read(tx, "managed_phase_readiness", &f.operation_id.to_string())?;
    let expected = json!({"operation_id":f.operation_id,"origin":f.origin_id,"owner_epoch":f.epoch,
        "state":"allocated","start_ended":false,"known_terminal":false,"parking_version":null,"version":1});
    ensure!(
        row.body()? == expected
            && row.column("operation_id")? == &SqlValue::Text(f.operation_id.to_string())
            && row.column("origin")? == &SqlValue::Text(f.origin_id.to_string())
            && row.column("owner_epoch")? == &SqlValue::Integer(i64::try_from(f.epoch)?)
            && row.column("state")? == &SqlValue::Text("allocated".into())
            && row.column("start_ended")? == &SqlValue::Integer(0)
            && row.column("known_terminal")? == &SqlValue::Integer(0)
            && row.column("parking_version")? == &SqlValue::Null
            && row.column("version")? == &SqlValue::Integer(1),
        "original Native readiness changed"
    );
    Ok(row)
}
fn no_registration(tx: &Transaction<'_>, launch: &PhaseLaunchParts) -> Result<()> {
    let f = launch.allocation().facts();
    let absent: bool = tx.query_row("SELECT NOT EXISTS(SELECT 1 FROM records WHERE id=?1) AND NOT EXISTS(SELECT 1 FROM session_units WHERE session_id=?1 OR unit_id=?2) AND NOT EXISTS(SELECT 1 FROM native_invocations WHERE session_id=?1 OR unit_id=?2 OR id=?3) AND NOT EXISTS(SELECT 1 FROM managed_phase_admissions WHERE pair_id=?4 OR operation_id=?5)",
        params![f.session_id.to_string(),f.unit_id.to_string(),f.invocation_id.to_string(),f.pair_id.to_string(),f.operation_id.to_string()], |r| r.get(0))?;
    ensure!(absent, "Native phase was already registered or consumed");
    Ok(())
}
fn registration_unit(unit: &ExecutionUnit, launch: &PhaseLaunchParts) -> Result<()> {
    let f = launch.allocation().facts();
    ensure!(
        unit.id == f.unit_id
            && unit.scope == *f.scope
            && unit.owner_epoch == f.epoch
            && unit.generation == f.generation
            && unit.provider == f.provider
            && unit.worktree == f.path
            && unit.profile_digest == f.profile_digest
            && unit.artifact_id == f.artifact
            && unit.base_sha == f.input.revision
            && unit.phase != WORKFLOW_SOURCE_BOOTSTRAP
            && unit.state == UnitState::Preparing
            && unit.session_id.is_none()
            && unit.native_effects_open
            && unit.result_finalization_open
            && unit.work.is_none()
            && unit.disposition == Disposition::Active,
        "Native phase Unit is not eligible for first registration"
    );
    ensure!(
        matches!(
            (unit.kind, f.role),
            (UnitKind::Executor, SessionRole::Executor)
                | (UnitKind::Reviewer, SessionRole::Reviewer)
        ),
        "Native phase role changed"
    );
    Ok(())
}

fn registration_attempt(
    current: &CurrentWorkflowSuccessor,
    launch: &PhaseLaunchParts,
) -> Result<()> {
    let original: crate::workflow::WorkflowSnapshot = serde_json::from_value(
        launch
            .marker()
            .original_plan()
            .workflow_after()
            .0
            .data
            .clone(),
    )?;
    let current: crate::workflow::WorkflowSnapshot =
        serde_json::from_value(current.workflow().data.clone())?;
    let index = original
        .active
        .context("original Native phase attempt absent")?;
    let before = original
        .history
        .get(index)
        .context("original Native phase history absent")?;
    let active = current
        .history
        .get(index)
        .context("current Native phase history absent")?;
    ensure!(
        current.active == Some(index)
            && current.generation == original.generation
            && current.context_version == original.context_version
            && !current.finished
            && current.terminal_decision.is_none()
            && active.generation == before.generation
            && active.context_version == before.context_version
            && active.phase == before.phase
            && active.state == crate::workflow::AttemptState::Running
            && active.dispatch_started
            && active.session_id.is_none()
            && active.execution.is_none()
            && active.completed_at.is_none(),
        "Native registration active attempt changed"
    );
    Ok(())
}

/// Exact current planning, hashing and complete encoded output preparation are
/// outside SharedStore. The same original actual launch is mandatory at commit.
pub(crate) fn plan_phase_registration(
    owner: &crate::execution::RuntimeOwner,
    launch: Arc<PhaseLaunchParts>,
    session: Session,
    seed: &NativeSeed,
) -> Result<NativeRegistrationPlan> {
    ensure!(
        launch.is_retained() && Arc::ptr_eq(launch.marker().allocation(), launch.allocation()),
        "Native registration lost original retained launch"
    );
    let f = launch.allocation().facts();
    ensure!(
        session.id == f.session_id
            && session.scope == *f.scope
            && session.agent == f.alias
            && session.provider == f.provider
            && session.role == f.role
            && session.worktree == f.path
            && session.model.as_deref() == f.model
            && session.effort.as_deref() == f.effort
            && session.pid.is_none()
            && session.native_ref.is_none()
            && session.state == SessionState::Starting
            && seed.id() == f.invocation_id
            && crate::execution::phase::encode_input(seed.input())? == f.input_bytes,
        "Native registration seed or Session differs from original selected allocation"
    );
    let current = plan_current_phase(owner, launch.marker())?;
    registration_attempt(&current, &launch)?;
    let (owner_before, readiness_before, artifact_version) = snapshot(owner, |tx| {
        validate_current_tx(tx, launch.marker(), &current)?;
        launch.marker().validate_driver_live_tx(tx)?;
        registration_unit(current.unit(), &launch)?;
        no_registration(tx, &launch)?;
        let artifact_version = current
            .unit()
            .artifact_id
            .map(|id| self_artifact_tx(tx, id).map(|a| a.version))
            .transpose()?;
        Ok((
            original_owner(tx, &launch)?,
            initial_readiness(tx, &launch)?,
            artifact_version,
        ))
    })?;
    let mut owner_after = PairRow {
        table: owner_before.table,
        values: owner_before.values.clone(),
    };
    let mut body = owner_before.body()?;
    body["native_invocation_id"] = json!(f.invocation_id);
    body["validated"] = json!(true);
    body["version"] = json!(2);
    owner_after.replace(
        "native_invocation_id",
        SqlValue::Text(f.invocation_id.to_string()),
    )?;
    owner_after.replace("validated", SqlValue::Integer(1))?;
    owner_after.replace("version", SqlValue::Integer(2))?;
    owner_after.set_body(&body)?;
    let mut readiness_after = PairRow {
        table: readiness_before.table,
        values: readiness_before.values.clone(),
    };
    let mut body = readiness_before.body()?;
    body["state"] = json!("registered");
    body["version"] = json!(2);
    readiness_after.replace("state", SqlValue::Text("registered".into()))?;
    readiness_after.replace("version", SqlValue::Integer(2))?;
    readiness_after.set_body(&body)?;
    let at = now_ms();
    let record = Record {
        id: RecordId(session.id.0),
        scope: session.scope.clone(),
        kind: RecordKind::Session,
        version: 1,
        data: serde_json::to_value(&session)?,
        created_at: at,
        updated_at: at,
    };
    let record_raw = serde_json::to_string(&record)?;
    ensure!(
        record_raw.len() <= PAIR_BODY_BYTES,
        "Native Session registration exceeds profile"
    );
    let mut unit_after = current.unit().clone();
    unit_after.version = unit_after
        .version
        .checked_add(1)
        .context("Native Unit version exhausted")?;
    unit_after.updated_at = at;
    unit_after.session_id = Some(session.id);
    unit_after.state = UnitState::DispatchPending;
    let unit_raw = serde_json::to_string(&unit_after)?;
    ensure!(
        unit_raw.len() <= 16 * 1024,
        "Native Unit registration exceeds profile"
    );
    let context = launch.marker().original_plan().context().0;
    let context_hash = crate::execution::native_result::digest(&serde_json::to_vec(context)?);
    let invocation = seed.invocation(&unit_after, &session, Some(context_hash), artifact_version);
    invocation.validate()?;
    let invocation_raw = serde_json::to_string(&invocation)?;
    ensure!(
        invocation_raw.len() <= crate::execution::native_result::INVOCATION_BYTES,
        "Native registration invocation exceeds profile"
    );
    Ok(NativeRegistrationPlan {
        launch,
        current,
        owner_before,
        owner_after,
        readiness_before,
        readiness_after,
        session,
        record,
        record_raw,
        unit_after,
        unit_raw,
        invocation,
        invocation_raw,
    })
}

impl Store {
    pub(crate) fn plan_native_phase_projection(
        runtime: &crate::execution::RuntimeOwner,
        phase: &Arc<NativePhaseSession>,
        session: Session,
    ) -> Result<NativeProjectionPlan> {
        plan_phase_projection(runtime, phase, session)
    }
    /// Actual factual Session ACK is separate from input/turn acknowledgement.
    /// A Running Session does not itself prove that native_input was consumed.
    pub(crate) fn project_phase_session(
        &mut self,
        plan: NativeProjectionPlan,
    ) -> Result<(ExecutionUnit, Session, u64)> {
        selected_database(&self.connection, plan.before.binding.owner().launch_parts())?;
        let permission = plan.before.session.update_permission(&plan.after)?;
        self.binding_permits.with_exact_permit(vec![permission], || {
            let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            plan.before.validate_tx(&tx)?;
            plan.before.session.update_tx(&tx,&plan.after)?;
            ensure!(tx.execute("UPDATE session_units SET dispatch_state='acknowledged' WHERE session_id=?1 AND unit_id=?2",params![plan.session.id.to_string(),plan.unit.id.to_string()])?==1,"Native Session dispatch binding changed");
            ensure!(tx.execute("UPDATE execution_units SET version=?1,body=?2 WHERE id=?3 AND version=?4 AND body=?5",params![plan.unit.version,plan.unit_raw,plan.unit.id.to_string(),plan.before.current.unit().version,plan.before.current.unit_raw()])?==1,"Native Session ACK Unit CAS changed");
            let SqlValue::Integer(version)=plan.after.column("version")? else { anyhow::bail!("Native Session ACK version absent") };
            let version=u64::try_from(*version)?;
            append_event(&tx,&plan.session.scope,"session.saved",json!({"id":plan.session.id,"version":version,"evidence":{"state":plan.session.state,"agent":plan.session.agent,"provider":plan.session.provider,"role":plan.session.role,"native_ref":plan.session.native_ref}}))?;
            self.binding_permits.ensure_consumed()?;
            tx.commit()?;
            Ok((plan.unit,plan.session,version))
        })
    }
    pub(crate) fn plan_native_phase_registration(
        owner: &crate::execution::RuntimeOwner,
        launch: Arc<PhaseLaunchParts>,
        session: Session,
        seed: &NativeSeed,
    ) -> Result<NativeRegistrationPlan> {
        plan_phase_registration(owner, launch, session, seed)
    }
    /// Rechecks a real launch before any Native-owned preparation helper. This
    /// check grants no generic caller permission and never reconstructs an owner.
    pub(crate) fn validate_phase_preparation(
        &mut self,
        launch: &PhaseLaunchParts,
        current: &CurrentWorkflowSuccessor,
    ) -> Result<ExecutionUnit> {
        ensure!(launch.is_retained(), "Native preparation retention ended");
        selected_database(&self.connection, launch)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        validate_current_tx(&tx, launch.marker(), current)?;
        launch.marker().validate_driver_live_tx(&tx)?;
        registration_unit(current.unit(), launch)?;
        validate_authority(&tx, &current.unit().authority(), true, false)?;
        original_owner(&tx, launch)?;
        initial_readiness(&tx, launch)?;
        no_registration(&tx, launch)?;
        tx.commit()?;
        Ok(current.unit().clone())
    }
    /// One exact permission/Immediate transaction journals actual Session and
    /// invocation and validates the SAME allocated owner before Native spawn.
    /// The caller issues its private owner only after this known commit succeeds.
    pub(crate) fn register_phase_session(
        &mut self,
        plan: NativeRegistrationPlan,
    ) -> Result<(ExecutionUnit, Session, u64)> {
        ensure!(
            plan.launch.is_retained(),
            "Native registration retention ended"
        );
        selected_database(&self.connection, &plan.launch)?;
        let mutations = vec![
            plan.owner_before.update_permission(&plan.owner_after)?,
            plan.readiness_before
                .update_permission(&plan.readiness_after)?,
            ExactRowMutation::new(
                "records",
                "INSERT",
                None,
                Some(vec![
                    SqlValue::Text(plan.record.id.to_string()),
                    SqlValue::Text("session".into()),
                    SqlValue::Text(plan.record.scope.project_id.to_string()),
                    SqlValue::Text(
                        plan.record
                            .scope
                            .goal_id
                            .context("Native Goal absent")?
                            .to_string(),
                    ),
                    SqlValue::Text(
                        plan.record
                            .scope
                            .task_id
                            .context("Native Task absent")?
                            .to_string(),
                    ),
                    SqlValue::Integer(1),
                    SqlValue::Text(plan.record_raw.clone()),
                ]),
            )?,
        ];
        self.binding_permits.with_exact_permit(mutations, || {
            let tx = self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            validate_current_tx(&tx,plan.launch.marker(),&plan.current)?;
            plan.launch.marker().validate_driver_live_tx(&tx)?;
            registration_unit(plan.current.unit(),&plan.launch)?;
            validate_authority(&tx,&plan.current.unit().authority(),true,false)?;
            no_registration(&tx,&plan.launch)?;
            plan.owner_before.validate_tx(&tx)?; plan.readiness_before.validate_tx(&tx)?;
            let (p,g,t) = scope_keys(&plan.session.scope)?;
            tx.execute("INSERT INTO records(id,kind,project_id,goal_id,task_id,version,body) VALUES(?1,'session',?2,?3,?4,1,?5)",params![plan.session.id.to_string(),p,g,t,plan.record_raw])?;
            tx.execute("INSERT INTO session_units(session_id,unit_id,project_id,goal_id,task_id,dispatch_state) VALUES(?1,?2,?3,?4,?5,'pending')",params![plan.session.id.to_string(),plan.unit_after.id.to_string(),p,g,t])?;
            ensure!(tx.execute("UPDATE execution_units SET version=?1,body=?2 WHERE id=?3 AND version=?4 AND body=?5",params![plan.unit_after.version,plan.unit_raw,plan.unit_after.id.to_string(),plan.current.unit().version,plan.current.unit_raw()])?==1,"Native registration Unit full CAS changed");
            let i=&plan.invocation;
            tx.execute("INSERT INTO native_invocations(id,unit_id,session_id,project_id,goal_id,task_id,generation,owner_epoch,provider,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,'not_dispatched',1,?10)",params![i.id.to_string(),i.unit_id.to_string(),i.session_id.to_string(),p,g,t,i.generation,i.owner_epoch,i.provider,plan.invocation_raw])?;
            plan.owner_before.update_tx(&tx,&plan.owner_after)?;
            plan.readiness_before.update_tx(&tx,&plan.readiness_after)?;
            append_event(&tx,&plan.session.scope,"execution.session_intent",json!({"unit":plan.unit_after.id,"session":plan.session.id}))?;
            append_event(&tx,&plan.session.scope,"execution.native_invocation",json!({"unit":plan.unit_after.id,"session":plan.session.id,"invocation":i.id}))?;
            self.binding_permits.ensure_consumed()?;
            tx.commit()?;
            Ok((plan.unit_after,plan.session,1))
        })
    }
}
