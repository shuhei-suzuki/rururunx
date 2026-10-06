use super::*;

impl Store {
    /// Journal a new native Session before spawn, without changing the Task version.
    pub(crate) fn register_execution_session(
        &mut self,
        authority: &ExecutionAuthority,
        session: &Session,
    ) -> Result<ExecutionUnit> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = validate_authority(&tx, authority, true, false)?;
        ensure!(
            true,
            "command-only verifier cannot register or update native Sessions"
        );
        ensure!(
            unit.state == UnitState::Preparing
                && unit.phase != WORKFLOW_SOURCE_BOOTSTRAP
                && valid_oid(&unit.base_sha)
                && unit.session_id.is_none()
                && session.scope == unit.scope
                && session.worktree == unit.worktree
                && session.state == SessionState::Starting
                && session.pid.is_none()
                && session.native_ref.is_none()
                && session.provider == unit.provider,
            "invalid new execution Session"
        );
        ensure!(
            matches!(
                (unit.kind, session.role),
                (UnitKind::Executor, SessionRole::Executor)
                    | (UnitKind::Reviewer, SessionRole::Reviewer)
                    | (UnitKind::Verifier, SessionRole::Consultant)
            ),
            "Session/unit role mismatch"
        );
        let mut record = Record::new(
            unit.scope.clone(),
            RecordKind::Session,
            serde_json::to_value(session)?,
        );
        record.id = RecordId(session.id.0);
        write_record_tx(&tx, &record)?;
        let (p, g, t) = scope_keys(&unit.scope)?;
        tx.execute("INSERT INTO session_units(session_id,unit_id,project_id,goal_id,task_id,dispatch_state) VALUES(?1,?2,?3,?4,?5,'pending')",params![session.id.to_string(),unit.id.to_string(),p,g,t])?;
        unit.session_id = Some(session.id);
        unit.state = UnitState::DispatchPending;
        write_unit(&tx, &mut unit)?;
        append_event(
            &tx,
            &unit.scope,
            "execution.session_intent",
            json!({"unit":unit.id,"session":session.id}),
        )?;
        tx.commit()?;
        Ok(unit)
    }
    pub fn session_execution_unit(&self, id: SessionId) -> Result<Option<ExecutionUnit>> {
        session_unit_tx(&self.connection, id)
    }
    pub(crate) fn update_execution_session(
        &mut self,
        authority: &ExecutionAuthority,
        session: &Session,
        expected: u64,
    ) -> Result<(ExecutionUnit, u64)> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = validate_authority(&tx, authority, true, false)?;
        ensure!(
            !verification::is_command_unit(&tx, unit.id)?,
            "command-only verifier cannot register or update native Sessions"
        );
        ensure!(
            unit.session_id == Some(session.id) && session.state == SessionState::Running,
            "invalid native acknowledgement"
        );
        let record = checked_session_record(&tx, &unit, session, expected)?;
        ensure!(
            matches!(
                unit.state,
                UnitState::DispatchPending | UnitState::Running | UnitState::WaitingQuota
            ),
            "Session acknowledgement state mismatch"
        );
        let next = write_record_tx(&tx, &record)?;
        tx.execute("UPDATE session_units SET dispatch_state='acknowledged' WHERE session_id=?1 AND unit_id=?2",params![session.id.to_string(),unit.id.to_string()])?;
        unit.state = UnitState::Running;
        write_unit(&tx, &mut unit)?;
        tx.commit()?;
        Ok((unit, next.version))
    }
    /// Terminal transport observations may describe a retired unit, never revive it.
    pub(crate) fn close_execution_session(
        &mut self,
        unit_id: UnitId,
        session: &Session,
        expected: u64,
    ) -> Result<u64> {
        ensure!(
            session_terminal(session.state) || session.state == SessionState::Lost,
            "not a conservative Session closure"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let unit = unit_tx(&tx, unit_id)?;
        ensure!(
            !unit.native_effects_open,
            "close logical authority before transport closure"
        );
        let record = checked_session_record(&tx, &unit, session, expected)?;
        let next = write_record_tx(&tx, &record)?;
        tx.execute(
            "UPDATE session_units SET dispatch_state=?1 WHERE session_id=?2 AND unit_id=?3",
            params![
                if session.state == SessionState::Lost {
                    "unknown"
                } else {
                    "terminal"
                },
                session.id.to_string(),
                unit.id.to_string()
            ],
        )?;
        tx.commit()?;
        Ok(next.version)
    }
}
fn session_unit_tx(connection: &Connection, id: SessionId) -> Result<Option<ExecutionUnit>> {
    let unit: Option<String> = connection
        .query_row(
            "SELECT unit_id FROM session_units WHERE session_id=?1",
            [id.to_string()],
            |r| r.get(0),
        )
        .optional()?;
    let Some(unit) = unit else { return Ok(None) };
    let unit = unit_tx(connection, unit.parse()?)?;
    ensure!(
        unit.session_id == Some(id),
        "Session unit identity mismatch"
    );
    let mut cols = scoped_columns(&unit.scope);
    cols.extend([("session_id", json!(id)), ("unit_id", json!(unit.id))]);
    check_indexed(connection, "session_units", &cols)?;
    Ok(Some(unit))
}
fn checked_session_record(
    tx: &Transaction<'_>,
    unit: &ExecutionUnit,
    session: &Session,
    expected: u64,
) -> Result<Record> {
    ensure!(
        unit.session_id == Some(session.id)
            && session.scope == unit.scope
            && session.worktree == unit.worktree
            && session.provider == unit.provider,
        "Session identity changed"
    );
    let bound = session_unit_tx(tx, session.id)?.context("Session lacks unit binding")?;
    ensure!(bound.id == unit.id, "foreign Session unit");
    let mut record: Record =
        read_tx(tx, "records", &session.id.to_string())?.context("Session record missing")?;
    ensure!(
        record.id.0 == session.id.0
            && record.scope == unit.scope
            && record.kind == RecordKind::Session
            && record.version == expected,
        "Session record CAS/scope mismatch"
    );
    let old: Session = serde_json::from_value(record.data.clone())?;
    ensure!(
        old.id == session.id
            && old.scope == session.scope
            && old.provider == session.provider
            && old.agent == session.agent
            && old.role == session.role
            && old.worktree == session.worktree
            && old.model == session.model
            && old.effort == session.effort
            && !session_terminal(old.state),
        "Session immutable identity/terminal changed"
    );
    if let Some(native) = &old.native_ref {
        ensure!(
            session.native_ref.as_ref() == Some(native),
            "native Session identity changed"
        );
    }
    record.data = serde_json::to_value(session)?;
    Ok(record)
}

pub(super) fn fence_epoch_sessions(tx: &Transaction<'_>) -> Result<()> {
    let mut statement = tx.prepare("SELECT session_id FROM session_units ORDER BY rowid")?;
    let ids = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(statement);
    for id in ids {
        let id: SessionId = id.parse()?;
        let unit = session_unit_tx(tx, id)?.context("managed Session unit missing")?;
        let record: Record =
            read_tx(tx, "records", &id.to_string())?.context("managed Session record missing")?;
        let mut session: Session = serde_json::from_value(record.data.clone())?;
        ensure!(
            record.id.0 == id.0
                && record.kind == RecordKind::Session
                && record.scope == unit.scope
                && session.id == id
                && session.scope == unit.scope
                && session.provider == unit.provider
                && session.worktree == unit.worktree,
            "managed Session recovery identity mismatch"
        );
        if session_terminal(session.state) {
            continue;
        }
        session.state = SessionState::Lost;
        let record = checked_session_record(tx, &unit, &session, record.version)?;
        write_record_tx(tx, &record)?;
        tx.execute(
            "UPDATE session_units SET dispatch_state='unknown' WHERE session_id=?1 AND unit_id=?2",
            params![id.to_string(), unit.id.to_string()],
        )?;
        append_event(
            tx,
            &unit.scope,
            "execution.session_epoch_lost",
            json!({"unit":unit.id,"session":id}),
        )?;
    }
    Ok(())
}

/// Unknown legacy dispatch remains a hold; logical managed retirement can proceed.
pub(in crate::state) fn logically_retired_session(
    tx: &Connection,
    session: &Session,
) -> Result<bool> {
    let Some(unit) = session_unit_tx(tx, session.id)? else {
        return Ok(false);
    };
    ensure!(
        unit.scope == session.scope && unit.worktree == session.worktree,
        "managed Session binding mismatch"
    );
    Ok(
        !unit.native_effects_open
            && !unit.result_finalization_open
            && unit.kind != UnitKind::Legacy,
    )
}
