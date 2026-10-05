//! Typed bounded reads; DTOs returned here do not carry native producer authority.
use super::*;
use crate::execution::native_result::{
    INVOCATION_BYTES, NativeInvocation, NativeResultReceipt, RECEIPT_BYTES,
};

pub(in crate::state) fn install_schema(tx: &Transaction<'_>) -> Result<()> {
    tx.execute_batch(include_str!("native_results.sql"))?;
    Ok(())
}
pub(in crate::state) fn validate_legacy_namespace(tx: &Transaction<'_>) -> Result<()> {
    let count: usize = tx.query_row(
        "SELECT COUNT(*) FROM sqlite_schema WHERE name IN ('native_invocations','native_results','native_invocation_identity','native_invocation_no_delete','native_result_no_update','native_result_no_delete')",
        [], |r| r.get(0))?;
    ensure!(count == 0, "legacy native receipt namespace is not empty");
    Ok(())
}
fn bounded_body(
    connection: &Connection,
    table: &str,
    id: &str,
    maximum: usize,
) -> Result<Option<String>> {
    let n: Option<usize> = connection
        .query_row(
            &format!("SELECT length(CAST(body AS BLOB)) FROM {table} WHERE id=?1"),
            [id],
            |r| r.get(0),
        )
        .optional()?;
    let Some(n) = n else { return Ok(None) };
    ensure!(n <= maximum, "native content body exceeds bound");
    Ok(Some(connection.query_row(
        &format!("SELECT body FROM {table} WHERE id=?1"),
        [id],
        |r| r.get(0),
    )?))
}
pub(super) fn invocation_tx(
    connection: &Connection,
    id: NativeInvocationId,
) -> Result<NativeInvocation> {
    let invocation: NativeInvocation = decode(
        bounded_body(
            connection,
            "native_invocations",
            &id.to_string(),
            INVOCATION_BYTES,
        )?
        .context("native invocation missing")?,
    )?;
    invocation.validate()?;
    ensure!(invocation.id == id, "native invocation identity mismatch");
    let mut columns = scoped_columns(&invocation.scope);
    columns.extend([
        ("id", json!(id)),
        ("unit_id", json!(invocation.unit_id)),
        ("session_id", json!(invocation.session_id)),
        ("generation", json!(invocation.generation)),
        ("owner_epoch", json!(invocation.owner_epoch)),
        ("provider", json!(invocation.provider)),
        ("state", json!(invocation.state)),
        ("version", json!(invocation.version)),
        ("input_operation", json!(invocation.input_operation)),
        ("native_thread", json!(invocation.native_thread)),
        ("native_turn", json!(invocation.native_turn)),
    ]);
    check_indexed(connection, "native_invocations", &columns)?;
    let unit = unit_tx(connection, invocation.unit_id)?;
    ensure!(
        unit.scope == invocation.scope
            && unit.generation == invocation.generation
            && unit.owner_epoch == invocation.owner_epoch
            && unit.session_id == Some(invocation.session_id)
            && unit.provider == invocation.provider,
        "native invocation Unit mismatch"
    );
    let matched: bool=connection.query_row("SELECT EXISTS(SELECT 1 FROM session_units WHERE session_id=?1 AND unit_id=?2 AND project_id=?3 AND goal_id=?4 AND task_id=?5)",
        params![invocation.session_id.to_string(),invocation.unit_id.to_string(),invocation.scope.project_id.to_string(),invocation.scope.goal_id.unwrap().to_string(),invocation.scope.task_id.unwrap().to_string()],|r|r.get(0))?;
    ensure!(matched, "native invocation Session mismatch");
    Ok(invocation)
}
impl Store {
    pub fn native_session_result(&self, session: SessionId) -> Result<Option<NativeResultReceipt>> {
        let id: Option<String> = self
            .connection
            .query_row(
                "SELECT id FROM native_results WHERE session_id=?1",
                [session.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        id.map(|id| self.native_result(id.parse()?)).transpose()
    }
    pub fn native_session_invocation(&self, session: SessionId) -> Result<NativeInvocation> {
        let id: String = self.connection.query_row(
            "SELECT id FROM native_invocations WHERE session_id=?1",
            [session.to_string()],
            |r| r.get(0),
        )?;
        invocation_tx(&self.connection, id.parse()?)
    }
    pub fn native_invocation(&self, id: NativeInvocationId) -> Result<NativeInvocation> {
        invocation_tx(&self.connection, id)
    }
    pub fn native_result(&self, id: NativeResultId) -> Result<NativeResultReceipt> {
        let receipt: NativeResultReceipt = decode(
            bounded_body(
                &self.connection,
                "native_results",
                &id.to_string(),
                RECEIPT_BYTES,
            )?
            .context("native result missing")?,
        )?;
        receipt.validate()?;
        ensure!(receipt.id == id, "native receipt identity mismatch");
        let mut columns = scoped_columns(&receipt.scope);
        columns.extend([
            ("id", json!(id)),
            ("invocation_id", json!(receipt.invocation_id)),
            ("unit_id", json!(receipt.unit_id)),
            ("session_id", json!(receipt.session_id)),
            ("generation", json!(receipt.generation)),
            ("owner_epoch", json!(receipt.owner_epoch)),
            ("provider", json!(receipt.provider)),
            ("acquisition", json!(receipt.acquisition)),
            ("authority", json!(receipt.authority)),
            ("version", json!(receipt.version)),
        ]);
        check_indexed(&self.connection, "native_results", &columns)?;
        let invocation = invocation_tx(&self.connection, receipt.invocation_id)?;
        ensure!(
            receipt.unit_id == invocation.unit_id
                && receipt.session_id == invocation.session_id
                && receipt.scope == invocation.scope
                && receipt.generation == invocation.generation
                && receipt.owner_epoch == invocation.owner_epoch
                && receipt.provider == invocation.provider
                && receipt.native_thread == invocation.native_thread
                && receipt.native_turn == invocation.native_turn,
            "native result binding mismatch"
        );
        Ok(receipt)
    }
}

fn write_invocation(
    tx: &Transaction<'_>,
    invocation: &NativeInvocation,
    expected: Option<u64>,
) -> Result<()> {
    invocation.validate()?;
    let body = serde_json::to_string(invocation)?;
    if let Some(expected) = expected {
        ensure!(tx.execute("UPDATE native_invocations SET state=?1,version=?2,input_operation=?3,native_thread=?4,native_turn=?5,body=?6 WHERE id=?7 AND version=?8",
            params![key(invocation.state),invocation.version,invocation.input_operation.map(|i|i.to_string()),invocation.native_thread,invocation.native_turn,body,invocation.id.to_string(),expected])? == 1, "native invocation CAS mismatch");
    } else {
        let (p, g, t) = scope_keys(&invocation.scope)?;
        tx.execute("INSERT INTO native_invocations(id,unit_id,session_id,project_id,goal_id,task_id,generation,owner_epoch,provider,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![invocation.id.to_string(),invocation.unit_id.to_string(),invocation.session_id.to_string(),p,g,t,invocation.generation,invocation.owner_epoch,invocation.provider,key(invocation.state),invocation.version,body])?;
    }
    Ok(())
}
fn context_body(tx: &Connection, scope: &Scope, version: u64) -> Result<String> {
    let owner = context_owner(scope)?;
    let bytes: usize = tx.query_row("SELECT length(CAST(body AS BLOB)) FROM context_versions WHERE project_id=?1 AND owner=?2 AND version=?3",params![scope.project_id.to_string(),owner,version],|r|r.get(0))?;
    ensure!(bytes <= 2 * 1024 * 1024, "native Context encoded limit");
    Ok(tx.query_row(
        "SELECT body FROM context_versions WHERE project_id=?1 AND owner=?2 AND version=?3",
        params![scope.project_id.to_string(), owner, version],
        |r| r.get(0),
    )?)
}
fn bound_context(
    tx: &Connection,
    unit: &ExecutionUnit,
    input: &crate::adapter::PreparedInput,
) -> Result<Option<String>> {
    let task: Task = read_tx(
        tx,
        "tasks",
        &unit
            .scope
            .task_id
            .context("native Task missing")?
            .to_string(),
    )?
    .context("native Task missing")?;
    ensure!(
        input.scope == unit.scope && input.revision == unit.base_sha,
        "native input semantic binding mismatch"
    );
    if task.context_version == 0 {
        ensure!(
            input.source_versions.is_empty(),
            "standalone input cannot claim a durable source frame"
        );
        return Ok(None);
    }
    ensure!(
        task.context_version == input.version,
        "native Context version changed"
    );
    let body = context_body(tx, &unit.scope, input.version)?;
    let context: ContextVersion = decode(body)?;
    ensure!(
        context.scope == unit.scope
            && context.version == input.version
            && context.revision == input.revision
            && context.source_hashes == input.source_versions
            && serde_json::to_string(&context.data)? == input.payload,
        "native Context content changed"
    );
    Ok(Some(crate::execution::native_result::digest(
        &serde_json::to_vec(&context)?,
    )))
}
impl Store {
    pub(crate) fn validate_native_input(
        &self,
        authority: &ExecutionAuthority,
        input: &crate::adapter::PreparedInput,
    ) -> Result<()> {
        let unit = validate_authority(&self.connection, authority, true, false)?;
        bound_context(&self.connection, &unit, input)?;
        Ok(())
    }
    /// The private Native seed is constructed only at the actual launch path.
    pub(crate) fn register_native_session(
        &mut self,
        authority: &ExecutionAuthority,
        session: &Session,
        seed: &crate::execution::native::NativeSeed,
    ) -> Result<ExecutionUnit> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let before = validate_authority(&tx, authority, true, false)?;
        let context_hash = bound_context(&tx, &before, seed.input())?;
        let artifact_version = before
            .artifact_id
            .map(|id| self_artifact_tx(&tx, id).map(|a| a.version))
            .transpose()?;
        let unit = sessions::register_session_tx(&tx, authority, session)?;
        let invocation = seed.invocation(&unit, session, context_hash, artifact_version);
        write_invocation(&tx, &invocation, None)?;
        append_event(
            &tx,
            &unit.scope,
            "execution.native_invocation",
            json!({"unit":unit.id,"session":session.id,"invocation":invocation.id}),
        )?;
        tx.commit()?;
        Ok(unit)
    }
    pub(crate) fn reserve_native_input(
        &mut self,
        authority: &ExecutionAuthority,
        id: NativeInvocationId,
        effect: &ManagedEffect,
        frame_sha256: &str,
    ) -> Result<()> {
        use crate::execution::native_result::InvocationState;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let unit = validate_authority(&tx, authority, true, false)?;
        let mut invocation = invocation_tx(&tx, id)?;
        ensure!(
            invocation.unit_id == unit.id
                && unit.session_id == Some(invocation.session_id)
                && invocation.state == InvocationState::NotDispatched
                && effect.kind == "native_input"
                && effect.expected_target
                    == format!("session/{}/sha256/{frame_sha256}", invocation.session_id),
            "native input invocation changed"
        );
        // Input source/context bytes were frozen in the same launch transaction;
        // immutable Context rows are checked again before stdin can receive bytes.
        if let Some(version) = invocation.context_version {
            let body = context_body(&tx, &invocation.scope, version)?;
            let context: ContextVersion = decode(body)?;
            ensure!(
                invocation.context_sha256.as_deref()
                    == Some(
                        crate::execution::native_result::digest(&serde_json::to_vec(&context)?)
                            .as_str()
                    ),
                "native Context digest changed"
            );
        }
        effects::reserve_effect_tx(&tx, authority, effect)?;
        let expected = invocation.version;
        invocation.version += 1;
        invocation.state = InvocationState::InputPending;
        invocation.input_operation = Some(effect.id);
        invocation.frame_sha256 = Some(frame_sha256.into());
        write_invocation(&tx, &invocation, Some(expected))?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn ack_native_invocation(
        &mut self,
        authority: &ExecutionAuthority,
        id: NativeInvocationId,
        thread: &str,
        turn: Option<&str>,
    ) -> Result<()> {
        use crate::execution::native_result::InvocationState;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let unit = validate_authority(&tx, authority, true, false)?;
        let mut invocation = invocation_tx(&tx, id)?;
        let record: Record = read_tx(&tx, "records", &invocation.session_id.to_string())?
            .context("native Session missing")?;
        let session: Session = serde_json::from_value(record.data)?;
        ensure!(
            unit.id == invocation.unit_id
                && invocation.state == InvocationState::InputPending
                && session.native_ref.as_deref() == Some(thread)
                && session.state == SessionState::Running
                && (invocation.provider != "codex" || turn.is_some()),
            "native input acknowledgement mismatch"
        );
        let expected = invocation.version;
        invocation.version += 1;
        invocation.native_thread = Some(thread.into());
        invocation.native_turn = turn.map(str::to_owned);
        invocation.state = InvocationState::Acknowledged;
        write_invocation(&tx, &invocation, Some(expected))?;
        tx.commit()?;
        Ok(())
    }
    /// One private actor receipt transaction closes logical work, Session and leases.
    /// A stale actor may append historical content, but cannot revive any authority.
    pub(crate) fn finish_native_result(
        &mut self,
        terminal: &crate::execution::native::NativeTerminal,
    ) -> Result<(ExecutionUnit, NativeResultReceipt, Session)> {
        use crate::execution::native_result::{InvocationState, ReceiptAuthority};
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut receipt = terminal.receipt().clone();
        let mut invocation = invocation_tx(&tx, receipt.invocation_id)?;
        ensure!(
            receipt.unit_id == invocation.unit_id
                && receipt.session_id == invocation.session_id
                && receipt.scope == invocation.scope
                && receipt.generation == invocation.generation
                && receipt.owner_epoch == invocation.owner_epoch
                && receipt.provider == invocation.provider
                && receipt.native_thread == invocation.native_thread
                && receipt.native_turn == invocation.native_turn,
            "native terminal semantic binding mismatch"
        );
        let current = unit_tx(&tx, invocation.unit_id)?;
        let owned = current.native_effects_open
            && current.work.is_none()
            && validate_authority(&tx, &current.authority(), false, true).is_ok();
        receipt.authority = if owned {
            ReceiptAuthority::OwnedTerminal
        } else {
            ReceiptAuthority::HistoricalDraft
        };
        if owned
            && receipt.acquisition == crate::execution::native_result::AcquisitionStatus::Complete
        {
            ensure!(
                invocation.state == InvocationState::Acknowledged,
                "complete native answer lacks owned acknowledgement"
            );
            let effect = effect_tx(
                &tx,
                invocation.input_operation.context("native input missing")?,
            )?;
            ensure!(
                effect.unit_id == invocation.unit_id
                    && effect.scope == invocation.scope
                    && effect.kind == "native_input"
                    && effect.state == EffectState::Confirmed
                    && effect.expected_target
                        == format!(
                            "session/{}/sha256/{}",
                            invocation.session_id,
                            invocation
                                .frame_sha256
                                .as_deref()
                                .context("native input hash missing")?
                        ),
                "native answer lacks exact confirmed input effect"
            );
        }
        receipt.validate()?;
        let prior: Option<String> = tx
            .query_row(
                "SELECT body FROM native_results WHERE invocation_id=?1",
                [invocation.id.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(prior) = prior {
            let previous: NativeResultReceipt = decode(prior.clone())?;
            previous.validate()?;
            receipt.authority = previous.authority;
            ensure!(
                prior == serde_json::to_string(&receipt)?,
                "changed native result replay refused"
            );
            let record: Record = read_tx(&tx, "records", &invocation.session_id.to_string())?
                .context("native Session missing")?;
            let session: Session = serde_json::from_value(record.data)?;
            tx.commit()?;
            return Ok((current, previous, session));
        }
        ensure!(
            invocation.state != InvocationState::Closed,
            "native invocation already closed without matching receipt"
        );
        let (p, g, t) = scope_keys(&receipt.scope)?;
        tx.execute("INSERT INTO native_results(id,invocation_id,unit_id,session_id,project_id,goal_id,task_id,generation,owner_epoch,provider,acquisition,authority,version,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,1,?13)",params![receipt.id.to_string(),invocation.id.to_string(),receipt.unit_id.to_string(),receipt.session_id.to_string(),p,g,t,receipt.generation,receipt.owner_epoch,receipt.provider,key(receipt.acquisition),key(receipt.authority),serde_json::to_string(&receipt)?])?;
        let expected = invocation.version;
        invocation.version += 1;
        invocation.state = InvocationState::Closed;
        write_invocation(&tx, &invocation, Some(expected))?;
        let unit = if owned {
            finish_execution_tx(
                &tx,
                &current.authority(),
                receipt.observed_work,
                receipt.disposition,
                terminal.failure(),
            )?
        } else {
            current
        };
        let record: Record = read_tx(&tx, "records", &invocation.session_id.to_string())?
            .context("native Session missing")?;
        ensure!(
            record.kind == RecordKind::Session && record.scope == unit.scope,
            "native terminal Session changed"
        );
        let mut session: Session = serde_json::from_value(record.data.clone())?;
        ensure!(
            (!owned || record.version == terminal.session_version())
                && session.id == terminal.session().id
                && session.role == terminal.session().role
                && session.worktree == terminal.session().worktree
                && session.model == terminal.session().model
                && session.effort == terminal.session().effort
                && session.agent == terminal.session().agent
                && session.native_ref == terminal.session().native_ref,
            "native terminal Session identity mismatch"
        );
        if !session_terminal(session.state) && session.state != SessionState::Lost {
            session.state = if unit.disposition == Disposition::Cancelled {
                SessionState::Stopped
            } else if unit.work == Some(WorkOutcome::Unknown) {
                SessionState::Lost
            } else if unit.work == Some(WorkOutcome::Failure) {
                SessionState::Failed
            } else {
                SessionState::Exited
            };
            sessions::close_session_tx(&tx, unit.id, &session, record.version)?;
        }
        append_event(
            &tx,
            &unit.scope,
            "execution.native_result",
            json!({"unit":unit.id,"receipt":receipt.id,"acquisition":receipt.acquisition,"authority":receipt.authority,"answer_sha256":receipt.answer_sha256}),
        )?;
        tx.commit()?;
        Ok((unit, receipt, session))
    }
}
