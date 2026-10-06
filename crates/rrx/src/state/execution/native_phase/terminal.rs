//! Actual saved-terminal plans. Current rows alone cannot issue owned success.
use super::*;
use crate::execution::{
    native::NativeTerminal,
    native_result::{AcquisitionStatus, InvocationState, NativeResultReceipt, ReceiptAuthority},
};
use rusqlite::{Connection, OpenFlags};
use std::time::Duration;

struct UnitImage {
    value: ExecutionUnit,
    raw: String,
}
impl UnitImage {
    fn read(tx: &Transaction<'_>, phase: &NativePhaseSession) -> Result<Self> {
        let f = phase.allocation().facts();
        let raw: Option<String> = tx.query_row(
            "SELECT CASE WHEN length(CAST(body AS BLOB))<=16384 THEN body END FROM execution_units WHERE id=?1",
            [f.unit_id.to_string()], |r| r.get(0),
        )?;
        let raw = raw.context("bounded terminal Unit absent")?;
        let value: ExecutionUnit = serde_json::from_value(crate::execution::strict_json::decode(
            raw.as_bytes(),
            crate::execution::strict_json::Limits {
                frame_bytes: 16 * 1024,
                depth: 8,
                nodes: 256,
                string_bytes: 4096,
                total_string_bytes: 16 * 1024,
                object_entries: 64,
                array_entries: 64,
            },
        )?)?;
        let original = phase.allocation().unit_snapshot();
        ensure!(
            value.id == f.unit_id
                && value.scope == *f.scope
                && value.kind == original.kind
                && value.phase == original.phase
                && value.generation == f.generation
                && value.owner_epoch == f.epoch
                && value.provider == f.provider
                && value.worktree == f.path
                && value.branch == original.branch
                && value.base_sha == original.base_sha
                && value.profile_digest == original.profile_digest
                && value.cookie == original.cookie
                && value.created_at == original.created_at
                && value.session_id == Some(f.session_id),
            "terminal Unit differs from original allocated core"
        );
        let image = Self { value, raw };
        image.validate_tx(tx)?;
        Ok(image)
    }
    fn encoded(value: ExecutionUnit) -> Result<Self> {
        let raw = serde_json::to_string(&value)?;
        ensure!(raw.len() <= 16 * 1024, "terminal Unit encoded limit");
        Ok(Self { value, raw })
    }
    fn validate_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        let u = &self.value;
        let (p, g, t) = scope_keys(&u.scope)?;
        let exact: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM execution_units WHERE id=?1 AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND kind=?5 AND generation=?6 AND owner_epoch=?7 AND version=?8 AND native_effects_open=?9 AND result_finalization_open=?10 AND worktree=?11 AND branch IS ?12 AND body=?13)",
            params![u.id.to_string(),p,g,t,key(u.kind),u.generation,u.owner_epoch,u.version,u.native_effects_open,u.result_finalization_open,u.worktree.to_str().context("terminal path is not UTF-8")?,u.branch,self.raw], |r|r.get(0),
        )?;
        ensure!(exact, "terminal full Unit CAS changed");
        Ok(())
    }
    fn update_tx(&self, tx: &Transaction<'_>, next: &Self) -> Result<()> {
        ensure!(tx.execute(
            "UPDATE execution_units SET version=?1,native_effects_open=?2,result_finalization_open=?3,body=?4 WHERE id=?5 AND version=?6 AND body=?7",
            params![next.value.version,next.value.native_effects_open,next.value.result_finalization_open,next.raw,self.value.id.to_string(),self.value.version,self.raw],
        )? == 1, "terminal Unit update changed");
        Ok(())
    }
}

/// No normal currency is required to read factual closure of the SAME actor's
/// old Unit. This separate query-only snapshot grants neither start nor input.
fn terminal_snapshot<R>(
    runtime: &crate::execution::RuntimeOwner,
    read: impl FnOnce(&Transaction<'_>) -> Result<R>,
) -> Result<R> {
    let mut connection = Connection::open_with_flags(
        runtime.state_path(),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.busy_timeout(Duration::from_millis(250))?;
    connection.pragma_update(None, "query_only", true)?;
    let tx = connection.transaction()?;
    ensure!(
        tx.pragma_query_value(None, "application_id", |r| r.get::<_, i64>(0))?
            == crate::state::APPLICATION_ID
            && tx.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))?
                == crate::state::SCHEMA_VERSION,
        "terminal snapshot contract unavailable"
    );
    let instance: String = tx.query_row(
        "SELECT instance_id FROM runtime_epoch WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    ensure!(
        instance == runtime.instance_id(),
        "terminal snapshot instance changed"
    );
    let result = read(&tx)?;
    tx.commit()?;
    Ok(result)
}

/// Retained only by the actual Native actor. It contains the SAME captured
/// terminal and original current plan; exact committed replay cannot remint it.
pub(crate) struct NativeTerminalPlan {
    phase: Arc<NativePhaseSession>,
    terminal: Arc<NativeTerminal>,
    normal: Option<NativeOwnerPlan>,
    unit: UnitImage,
    unit_after: UnitImage,
    invocation: InvocationImage,
    invocation_after: InvocationImage,
    session: PairRow,
    session_after: PairRow,
    closed_session: Session,
    owner: PairRow,
    readiness: PairRow,
    readiness_after: PairRow,
    admission: Option<PairRow>,
    admission_after: Option<PairRow>,
    receipt: NativeResultReceipt,
    receipt_raw: String,
    session_version: u64,
    input_effect: Option<(OperationId, String, u64, EffectState)>,
}

/// Only a known transaction/confirmation of the SAME retained plan creates this.
/// It is consumed by the actual actor, not serialized or used as review approval.
pub(crate) struct NativeTerminalCommit {
    terminal: Arc<NativeTerminal>,
    phase: Arc<NativePhaseSession>,
    unit: ExecutionUnit,
    receipt: NativeResultReceipt,
    session: Session,
    version: u64,
    owned_success: bool,
}
impl NativeTerminalCommit {
    pub(crate) fn into_parts(
        self,
    ) -> (
        Arc<NativeTerminal>,
        Arc<NativePhaseSession>,
        ExecutionUnit,
        NativeResultReceipt,
        Session,
        u64,
        bool,
    ) {
        (
            self.terminal,
            self.phase,
            self.unit,
            self.receipt,
            self.session,
            self.version,
            self.owned_success,
        )
    }
}

pub(crate) fn plan_phase_terminal(
    runtime: &crate::execution::RuntimeOwner,
    phase: &Arc<NativePhaseSession>,
    terminal: Arc<NativeTerminal>,
) -> Result<Arc<NativeTerminalPlan>> {
    let f = phase.allocation().facts();
    let observed = terminal.receipt();
    observed.validate()?;
    let binding = phase.binding_snapshot()?;
    ensure!(
        observed.invocation_id == f.invocation_id
            && observed.unit_id == f.unit_id
            && observed.session_id == f.session_id
            && observed.scope == *f.scope
            && observed.generation == f.generation
            && observed.owner_epoch == f.epoch
            && observed.provider == f.provider
            && terminal.session().id == f.session_id
            && terminal.session().scope == *f.scope
            && serde_json::to_value(terminal.session())?
                == serde_json::to_value(binding.session())?
            && terminal.session_version() == binding.record_version(),
        "saved terminal differs from actual phase owner"
    );
    // Storage faults are not evidence that owned known work became historical.
    // A known terminal needs original current currency even after its Core has
    // ended; revocation prevents new effects but cannot erase that observation.
    // Only actual closed/retired/foreign-epoch Units, or non-success transport
    // observations, use the separate nongrant closure lane.
    let eligible = terminal_snapshot(runtime, |tx| {
        let unit = UnitImage::read(tx, phase)?;
        let epoch: u64 = tx.query_row(
            "SELECT epoch FROM runtime_epoch WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        Ok(epoch == f.epoch
            && unit.value.native_effects_open
            && unit.value.result_finalization_open
            && unit.value.work.is_none()
            && unit.value.disposition == Disposition::Active
            && matches!(
                unit.value.state,
                UnitState::DispatchPending | UnitState::Running | UnitState::WaitingQuota
            ))
    })?;
    let normal = if eligible
        && observed.observed_work != WorkOutcome::Unknown
        && observed.disposition == Disposition::Completed
    {
        Some(plan_owner_currency(runtime, phase, true)?)
    } else {
        None
    };
    let (unit, invocation, session, owner, readiness, admission, input_effect) = terminal_snapshot(
        runtime,
        |tx| {
            selected_database(tx, phase.launch_parts())?;
            let unit = UnitImage::read(tx, phase)?;
            let invocation = InvocationImage::read(tx, f.invocation_id)?;
            let n = &invocation.value;
            ensure!(
                n.id == observed.invocation_id
                    && n.unit_id == observed.unit_id
                    && n.session_id == observed.session_id
                    && n.scope == observed.scope
                    && n.generation == observed.generation
                    && n.owner_epoch == observed.owner_epoch
                    && n.provider == observed.provider
                    && n.native_thread
                        .as_ref()
                        .is_none_or(|v| observed.native_thread.as_ref() == Some(v))
                    && n.native_turn
                        .as_ref()
                        .is_none_or(|v| observed.native_turn.as_ref() == Some(v))
                    && n.state != InvocationState::Closed,
                "actual terminal invocation identity changed"
            );
            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM native_results WHERE invocation_id=?1)",
                [f.invocation_id.to_string()],
                |r| r.get(0),
            )?;
            ensure!(
                !exists,
                "terminal requires its retained original commit plan"
            );
            let session = PairRow::read(tx, "records", &f.session_id.to_string())?;
            let record: Record = serde_json::from_value(session.body()?)?;
            let s: Session = serde_json::from_value(record.data.clone())?;
            ensure!(
                record.id == RecordId(f.session_id.0)
                    && record.kind == RecordKind::Session
                    && record.scope == *f.scope
                    && record.version >= binding.record_version()
                    && s.id == f.session_id
                    && s.scope == *f.scope
                    && s.agent == f.alias
                    && s.provider == f.provider
                    && s.role == f.role
                    && s.worktree == f.path
                    && s.model.as_deref() == f.model
                    && s.effort.as_deref() == f.effort
                    && s.started_at == terminal.session().started_at
                    && s.pid == terminal.session().pid
                    && s.native_ref == terminal.session().native_ref
                    && s.recovery == terminal.session().recovery
                    && (record.version == binding.record_version()
                        || session_terminal(s.state)
                        || s.state == SessionState::Lost),
                "latest terminal Session immutable identity changed"
            );
            let mut indices = scoped_columns(&record.scope);
            indices.extend([
                ("id", json!(record.id)),
                ("kind", json!(record.kind)),
                ("version", json!(record.version)),
            ]);
            check_indexed(tx, "records", &indices)?;
            let owner = PairRow::read(tx, "managed_phase_owners", &f.pair_id.to_string())?;
            check_owner_indices(&owner, phase.launch_parts(), true)?;
            ensure!(
                owner.body()?
                    == json!({"owner_id":f.pair_id,"operation_id":f.operation_id,"scope":f.scope,
            "unit_id":f.unit_id,"owner_epoch":f.epoch,"execution_generation":f.generation,
            "allocated_session_id":f.session_id,"provider":f.provider,"alias":f.alias,"role":f.role,
            "worktree":f.path,"origin":f.origin_id,"native_invocation_id":f.invocation_id,"validated":true,"version":2}),
                "terminal registered owner changed"
            );
            let readiness =
                PairRow::read(tx, "managed_phase_readiness", &f.operation_id.to_string())?;
            let body = readiness.body()?;
            let version = body["version"]
                .as_u64()
                .context("terminal readiness version absent")?;
            let ended = body["start_ended"]
                .as_bool()
                .context("terminal start ended flag absent")?;
            ensure!(
                body == json!({"operation_id":f.operation_id,"origin":f.origin_id,"owner_epoch":f.epoch,
            "state":"registered","start_ended":ended,"known_terminal":false,"parking_version":null,"version":version})
                    && ((version == 2 && !ended) || (version == 3 && ended)),
                "terminal readiness changed"
            );
            for (name, value) in [
                ("operation_id", SqlValue::Text(f.operation_id.to_string())),
                ("origin", SqlValue::Text(f.origin_id.to_string())),
                ("owner_epoch", SqlValue::Integer(i64::try_from(f.epoch)?)),
                ("state", SqlValue::Text("registered".into())),
                ("start_ended", SqlValue::Integer(i64::from(ended))),
                ("known_terminal", SqlValue::Integer(0)),
                ("parking_version", SqlValue::Null),
                ("version", SqlValue::Integer(i64::try_from(version)?)),
            ] {
                ensure!(
                    readiness.column(name)? == &value,
                    "terminal readiness index changed"
                );
            }
            let (admission, input_effect) = if let Some(input) = binding.consumed() {
                ensure!(
                    input.belongs_to(phase)
                        && n.input_operation == Some(input.effect())
                        && n.frame_sha256.as_deref() == Some(input.frame_sha256()),
                    "terminal consumed pair differs"
                );
                let actual_ack = input.acknowledgement()?;
                ensure!(
                    actual_ack.as_ref().map(|v| v.0.as_str()) == observed.native_thread.as_deref()
                        && actual_ack.as_ref().and_then(|v| v.1.as_deref())
                            == observed.native_turn.as_deref(),
                    "terminal does not retain the actual consumed acknowledgement"
                );
                let a = PairRow::read(tx, "managed_phase_admissions", &f.pair_id.to_string())?;
                let body = a.body()?;
                let confirmed = body["confirmed"]
                    .as_bool()
                    .context("terminal confirmed flag absent")?;
                let version = if confirmed { 2 } else { 1 };
                ensure!(
                    body == json!({"pair_id":f.pair_id,"operation_id":f.operation_id,"owner_id":f.pair_id,
                "native_invocation_id":f.invocation_id,"input_effect_id":input.effect(),"frame_sha256":input.frame_sha256(),
                "native_thread":if confirmed {n.native_thread.as_deref()}else{None},"native_turn":if confirmed {n.native_turn.as_deref()}else{None},
                "confirmed":confirmed,"uncertain":false,"settled":false,"version":version}),
                    "terminal admission differs"
                );
                for (name, value) in [
                    ("pair_id", SqlValue::Text(f.pair_id.to_string())),
                    ("operation_id", SqlValue::Text(f.operation_id.to_string())),
                    ("owner_id", SqlValue::Text(f.pair_id.to_string())),
                    (
                        "native_invocation_id",
                        SqlValue::Text(f.invocation_id.to_string()),
                    ),
                    (
                        "input_effect_id",
                        SqlValue::Text(input.effect().to_string()),
                    ),
                    ("frame_sha256", SqlValue::Text(input.frame_sha256().into())),
                    (
                        "native_thread",
                        if confirmed {
                            n.native_thread
                                .clone()
                                .map(SqlValue::Text)
                                .unwrap_or(SqlValue::Null)
                        } else {
                            SqlValue::Null
                        },
                    ),
                    (
                        "native_turn",
                        if confirmed {
                            n.native_turn
                                .clone()
                                .map(SqlValue::Text)
                                .unwrap_or(SqlValue::Null)
                        } else {
                            SqlValue::Null
                        },
                    ),
                    ("confirmed", SqlValue::Integer(i64::from(confirmed))),
                    ("uncertain", SqlValue::Integer(0)),
                    ("settled", SqlValue::Integer(0)),
                    ("version", SqlValue::Integer(version)),
                ] {
                    ensure!(
                        a.column(name)? == &value,
                        "terminal admission index differs"
                    );
                }
                let raw:Option<String>=tx.query_row("SELECT CASE WHEN length(CAST(body AS BLOB))<=8192 THEN body END FROM managed_effects WHERE id=?1",[input.effect().to_string()],|r|r.get(0))?;
                let raw = raw.context("terminal effect encoded limit")?;
                let effect = effect_tx(tx, input.effect())?;
                ensure!(
                    effect.unit_id == f.unit_id
                        && effect.scope == *f.scope
                        && effect.kind == "native_input"
                        && effect.expected_target
                            == format!("session/{}/sha256/{}", f.session_id, input.frame_sha256()),
                    "terminal input effect changed"
                );
                if normal.is_some() && observed.acquisition == AcquisitionStatus::Complete {
                    ensure!(
                        confirmed
                            && n.state == InvocationState::Acknowledged
                            && effect.state == EffectState::Confirmed
                            && input.acknowledgement()?
                                == Some((
                                    n.native_thread
                                        .clone()
                                        .context("owned terminal thread absent")?,
                                    n.native_turn.clone()
                                )),
                        "complete owned terminal lacks actual confirmed acknowledgement"
                    );
                }
                (
                    Some(a),
                    Some((input.effect(), raw, effect.version, effect.state)),
                )
            } else {
                ensure!(
                    n.input_operation.is_none() && n.frame_sha256.is_none(),
                    "terminal lost actual input ownership"
                );
                let present:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM managed_phase_admissions WHERE pair_id=?1 OR operation_id=?2 OR native_invocation_id=?3)",
                params![f.pair_id.to_string(),f.operation_id.to_string(),f.invocation_id.to_string()],|r|r.get(0))?;
                ensure!(!present, "terminal admission has no actual consumed proof");
                ensure!(
                    observed.acquisition != AcquisitionStatus::Complete,
                    "unconsumed terminal cannot claim complete owned answer"
                );
                (None, None)
            };
            if let Some(normal) = &normal {
                normal.validate_terminal_tx(tx)?;
            }
            Ok((
                unit,
                invocation,
                session,
                owner,
                readiness,
                admission,
                input_effect,
            ))
        },
    )?;
    let owned = normal.is_some();
    let mut receipt = observed.clone();
    receipt.authority = if owned {
        ReceiptAuthority::OwnedTerminal
    } else {
        ReceiptAuthority::HistoricalDraft
    };
    receipt.validate()?;
    let receipt_raw = serde_json::to_string(&receipt)?;
    let mut after = unit.value.clone();
    if after.native_effects_open || after.work.is_none() {
        if after.work.is_none() {
            after.work = Some(if owned {
                receipt.observed_work
            } else {
                WorkOutcome::Unknown
            });
            after.disposition = if owned
                || (receipt.observed_work == WorkOutcome::Unknown
                    && receipt.disposition != Disposition::Completed)
            {
                receipt.disposition
            } else {
                Disposition::Lost
            };
            after.state = if after.work == Some(WorkOutcome::Unknown) {
                UnitState::WorkUnknown
            } else {
                UnitState::WorkKnown
            };
        }
        after.native_effects_open = false;
        if !owned || after.disposition != Disposition::Completed {
            after.result_finalization_open = false;
        }
        after.wait_reason = match after.disposition {
            Disposition::QuotaInterrupted => Some(WaitReason::Quota),
            Disposition::CapacityInterrupted => Some(WaitReason::Capacity),
            _ => None,
        };
        after.capacity_retry_at = (after.disposition == Disposition::CapacityInterrupted)
            .then(|| now_ms().saturating_add(60_000));
        after.version = after
            .version
            .checked_add(1)
            .context("terminal Unit version exhausted")?;
        after.updated_at = now_ms();
    }
    let unit_after = UnitImage::encoded(after)?;
    let mut n = invocation.value.clone();
    // This is only factual projection of the retained actual ACK. An unconfirmed
    // input remains historical and cannot become an OwnedPhaseSettlement.
    n.native_thread = receipt.native_thread.clone();
    n.native_turn = receipt.native_turn.clone();
    n.state = InvocationState::Closed;
    n.version = n
        .version
        .checked_add(1)
        .context("terminal invocation version exhausted")?;
    let invocation_after = InvocationImage::encoded(n)?;
    let mut record: Record = serde_json::from_value(session.body()?)?;
    let mut closed_session: Session = serde_json::from_value(record.data.clone())?;
    if closed_session.native_ref.is_none() {
        closed_session.native_ref = receipt.native_thread.clone();
    }
    let mut session_after = PairRow {
        table: session.table,
        values: session.values.clone(),
    };
    if !session_terminal(closed_session.state) && closed_session.state != SessionState::Lost {
        closed_session.state = if unit_after.value.disposition == Disposition::Cancelled {
            SessionState::Stopped
        } else if unit_after.value.work == Some(WorkOutcome::Failure) {
            SessionState::Failed
        } else if unit_after.value.work == Some(WorkOutcome::Unknown) {
            SessionState::Lost
        } else {
            SessionState::Exited
        };
    }
    if serde_json::to_value(&closed_session)? != record.data {
        record.version = record
            .version
            .checked_add(1)
            .context("terminal Session version exhausted")?;
        record.updated_at = now_ms();
        record.data = serde_json::to_value(&closed_session)?;
        session_after.replace("version", SqlValue::Integer(i64::try_from(record.version)?))?;
        session_after.set_body(&serde_json::to_value(&record)?)?;
    }
    let mut readiness_after = PairRow {
        table: readiness.table,
        values: readiness.values.clone(),
    };
    let mut body = readiness_after.body()?;
    let version = body["version"]
        .as_u64()
        .context("readiness version absent")?
        .checked_add(1)
        .context("readiness version exhausted")?;
    body["state"] = json!("closed");
    body["start_ended"] = json!(true);
    body["known_terminal"] = json!(true);
    body["version"] = json!(version);
    readiness_after.replace("state", SqlValue::Text("closed".into()))?;
    readiness_after.replace("start_ended", SqlValue::Integer(1))?;
    readiness_after.replace("known_terminal", SqlValue::Integer(1))?;
    readiness_after.replace("version", SqlValue::Integer(i64::try_from(version)?))?;
    readiness_after.set_body(&body)?;
    let admission_after = admission
        .as_ref()
        .map(|a| -> Result<PairRow> {
            let mut next = PairRow {
                table: a.table,
                values: a.values.clone(),
            };
            let mut body = next.body()?;
            let version = body["version"]
                .as_u64()
                .context("admission version absent")?
                .checked_add(1)
                .context("admission version exhausted")?;
            let settled = body["confirmed"] == true
                && input_effect
                    .as_ref()
                    .is_some_and(|e| e.3 == EffectState::Confirmed);
            body["settled"] = json!(settled);
            body["uncertain"] = json!(!settled);
            body["version"] = json!(version);
            next.replace("settled", SqlValue::Integer(i64::from(settled)))?;
            next.replace("uncertain", SqlValue::Integer(i64::from(!settled)))?;
            next.replace("version", SqlValue::Integer(i64::try_from(version)?))?;
            next.set_body(&body)?;
            Ok(next)
        })
        .transpose()?;
    Ok(Arc::new(NativeTerminalPlan {
        phase: phase.clone(),
        terminal,
        normal,
        unit,
        unit_after,
        invocation,
        invocation_after,
        session,
        session_after,
        closed_session,
        owner,
        readiness,
        readiness_after,
        admission,
        admission_after,
        receipt,
        receipt_raw,
        session_version: record.version,
        input_effect,
    }))
}

impl NativeTerminalPlan {
    pub(crate) fn belongs_to(
        &self,
        phase: &NativePhaseSession,
        terminal: &Arc<NativeTerminal>,
    ) -> bool {
        std::ptr::eq(self.phase.as_ref(), phase) && Arc::ptr_eq(&self.terminal, terminal)
    }
    pub(crate) fn confirmed_absent(
        &self,
        runtime: &crate::execution::RuntimeOwner,
    ) -> Result<bool> {
        terminal_snapshot(runtime, |tx| {
            selected_database(tx, self.phase.launch_parts())?;
            let present: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM native_results WHERE invocation_id=?1)",
                [self.receipt.invocation_id.to_string()],
                |r| r.get(0),
            )?;
            Ok(!present)
        })
    }
    fn committed_tx(&self, tx: &Transaction<'_>) -> Result<bool> {
        let prior:Option<String>=tx.query_row("SELECT CASE WHEN length(CAST(body AS BLOB))<=?2 THEN body END FROM native_results WHERE invocation_id=?1",
            params![self.receipt.invocation_id.to_string(),crate::execution::native_result::RECEIPT_BYTES],|r|r.get(0)).optional()?.flatten();
        let Some(prior) = prior else { return Ok(false) };
        ensure!(
            prior == self.receipt_raw,
            "saved terminal committed receipt differs"
        );
        let mut indices = scoped_columns(&self.receipt.scope);
        indices.extend([
            ("id", json!(self.receipt.id)),
            ("invocation_id", json!(self.receipt.invocation_id)),
            ("unit_id", json!(self.receipt.unit_id)),
            ("session_id", json!(self.receipt.session_id)),
            ("generation", json!(self.receipt.generation)),
            ("owner_epoch", json!(self.receipt.owner_epoch)),
            ("provider", json!(self.receipt.provider)),
            ("acquisition", json!(self.receipt.acquisition)),
            ("authority", json!(self.receipt.authority)),
            ("version", json!(1)),
        ]);
        check_indexed(tx, "native_results", &indices)?;
        self.unit_after.validate_tx(tx)?;
        self.invocation_after.validate_tx(tx)?;
        self.session_after.validate_tx(tx)?;
        self.owner.validate_tx(tx)?;
        self.readiness_after.validate_tx(tx)?;
        if let Some(a) = &self.admission_after {
            a.validate_tx(tx)?;
        }
        Ok(true)
    }
    fn commit(&self) -> NativeTerminalCommit {
        NativeTerminalCommit {
            terminal: self.terminal.clone(),
            phase: self.phase.clone(),
            unit: self.unit_after.value.clone(),
            receipt: self.receipt.clone(),
            session: self.closed_session.clone(),
            version: self.session_version,
            owned_success: self.normal.is_some()
                && self.receipt.authority == ReceiptAuthority::OwnedTerminal
                && self.receipt.observed_work == WorkOutcome::Success
                && self.receipt.disposition == Disposition::Completed
                && self.receipt.acquisition == AcquisitionStatus::Complete,
        }
    }
}
impl Store {
    pub(crate) fn plan_native_phase_terminal(
        runtime: &crate::execution::RuntimeOwner,
        phase: &Arc<NativePhaseSession>,
        terminal: Arc<NativeTerminal>,
    ) -> Result<Arc<NativeTerminalPlan>> {
        plan_phase_terminal(runtime, phase, terminal)
    }
    pub(crate) fn finish_phase_terminal(
        &mut self,
        plan: &NativeTerminalPlan,
    ) -> Result<NativeTerminalCommit> {
        selected_database(&self.connection, plan.phase.launch_parts())?;
        let mut permissions = vec![plan.readiness.update_permission(&plan.readiness_after)?];
        if plan.session.values != plan.session_after.values {
            permissions.push(plan.session.update_permission(&plan.session_after)?);
        }
        if let (Some(old), Some(next)) = (&plan.admission, &plan.admission_after) {
            permissions.push(old.update_permission(next)?);
        }
        self.binding_permits.with_exact_permit(permissions, || {
        let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        // Confirmation uses the SAME retained original pre-transaction plan and
        // all its exact post-images. It does not decode rows into owned proof.
        if plan.committed_tx(&tx)? {tx.commit()?;return Ok(plan.commit());}
        plan.unit.validate_tx(&tx)?;plan.invocation.validate_tx(&tx)?;
        plan.session.validate_tx(&tx)?;plan.owner.validate_tx(&tx)?;plan.readiness.validate_tx(&tx)?;
        if let Some(normal)=&plan.normal {normal.validate_terminal_tx(&tx)?;}
        if let Some(a)=&plan.admission {a.validate_tx(&tx)?;}
        if let Some((id,raw,version,_))=&plan.input_effect {
            let exact:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM managed_effects WHERE id=?1 AND version=?2 AND body=?3)",params![id.to_string(),version,raw],|r|r.get(0))?;
            ensure!(exact,"terminal input transport changed");
            let effect=effect_tx(&tx,*id)?;
            ensure!(effect.unit_id==plan.unit.value.id && effect.scope==plan.unit.value.scope,"terminal effect binding changed");
        }
        let r=&plan.receipt;let (p,g,t)=scope_keys(&r.scope)?;
        tx.execute("INSERT INTO native_results(id,invocation_id,unit_id,session_id,project_id,goal_id,task_id,generation,owner_epoch,provider,acquisition,authority,version,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,1,?13)",
            params![r.id.to_string(),r.invocation_id.to_string(),r.unit_id.to_string(),r.session_id.to_string(),p,g,t,r.generation,r.owner_epoch,r.provider,key(r.acquisition),key(r.authority),plan.receipt_raw])?;
        plan.invocation.update_tx(&tx,&plan.invocation_after)?;
        if plan.unit.raw!=plan.unit_after.raw {plan.unit.update_tx(&tx,&plan.unit_after)?;}
        if plan.session.values!=plan.session_after.values {plan.session.update_tx(&tx,&plan.session_after)?;}
        tx.execute("UPDATE session_units SET dispatch_state=?1 WHERE session_id=?2 AND unit_id=?3",
            params![if plan.closed_session.state==SessionState::Lost {"unknown"}else{"terminal"},r.session_id.to_string(),r.unit_id.to_string()])?;
        plan.readiness.update_tx(&tx,&plan.readiness_after)?;
        if let (Some(old),Some(next))=(&plan.admission,&plan.admission_after) {old.update_tx(&tx,next)?;}
        quotas::release_quota_tx(&tx,r.unit_id)?;
        tx.execute("DELETE FROM quota_waiters WHERE unit_id=?1",[r.unit_id.to_string()])?;
        tx.execute("INSERT INTO cleanup_jobs(unit_id,next_due,attempts,version) VALUES(?1,?2,0,1) ON CONFLICT(unit_id) DO NOTHING",params![r.unit_id.to_string(),now_ms()])?;
        append_event(&tx,&r.scope,"execution.native_result",json!({"unit":r.unit_id,"receipt":r.id,"acquisition":r.acquisition,"authority":r.authority,"answer_sha256":r.answer_sha256}))?;
        self.binding_permits.ensure_consumed()?;
        tx.commit()?;
        Ok(plan.commit())
        })
    }
}
