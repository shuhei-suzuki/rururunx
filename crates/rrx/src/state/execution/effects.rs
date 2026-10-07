use super::*;

pub(super) fn fence_epoch_effects(tx: &Transaction<'_>) -> Result<()> {
    let mut statement =
        tx.prepare("SELECT id FROM managed_effects WHERE state='pending' ORDER BY rowid")?;
    let ids = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(statement);
    for id in ids {
        let mut effect = effect_tx(tx, id.parse()?)?;
        let unit = unit_tx(tx, effect.unit_id)?;
        ensure!(effect.scope == unit.scope, "effect recovery scope mismatch");
        let prior = effect.version;
        effect.version = prior.checked_add(1).context("effect version overflow")?;
        effect.state = EffectState::Unknown;
        effect.receipt = BTreeMap::from([("transport".into(), "runtime_epoch_lost".into())]);
        ensure!(tx.execute("UPDATE managed_effects SET state='unknown',version=?1,body=?2 WHERE id=?3 AND version=?4 AND state='pending'",
            params![effect.version,serde_json::to_string(&effect)?,effect.id.to_string(),prior])?==1,"effect recovery CAS mismatch");
        append_event(
            tx,
            &unit.scope,
            "execution.effect_epoch_unknown",
            json!({"unit":unit.id,"operation":effect.id}),
        )?;
    }
    Ok(())
}

impl Store {
    pub(crate) fn reserve_execution_helper(
        &mut self,
        authority: &ExecutionAuthority,
        id: OperationId,
        native: bool,
        path: &std::path::Path,
        kind: &str,
    ) -> Result<()> {
        self.reserve_execution_helper_pinned(authority, id, native, path, kind, None)
    }
    pub(crate) fn reserve_execution_helper_pinned(
        &mut self,
        authority: &ExecutionAuthority,
        id: OperationId,
        native: bool,
        path: &std::path::Path,
        kind: &str,
        driver: Option<&DriverReadTicket>,
    ) -> Result<()> {
        ensure!(path.is_absolute(), "helper path must be absolute");
        ensure!(
            matches!(kind, "git_helper" | "native_version" | "docker_probe"),
            "unsupported unit helper kind"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(ticket) = driver {
            ticket.validate_current_tx(&tx)?;
        }
        let unit = validate_authority(&tx, authority, native, !native)?;
        ensure!(
            !verification::is_command_unit(&tx, unit.id)? || kind == "git_helper",
            "command-only verifier only permits scoped snapshot Git helpers"
        );
        ensure!(
            unit.phase != WORKFLOW_SOURCE_BOOTSTRAP || kind == "git_helper",
            "source preparation only permits registered Git helpers"
        );
        if let Some(ticket) = driver {
            ticket.preparation_matches(&unit)?;
        }
        let effect = ManagedEffect {
            id,
            unit_id: authority.unit_id,
            scope: authority.scope.clone(),
            kind: kind.into(),
            idempotency_key: format!("helper-{id}"),
            expected_target: path.to_string_lossy().into(),
            state: EffectState::Pending,
            receipt: BTreeMap::new(),
            version: 1,
        };
        let (p, g, t) = scope_keys(&effect.scope)?;
        tx.execute("INSERT INTO managed_effects(id,unit_id,project_id,goal_id,task_id,idempotency_key,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,'pending',1,?7)",
            params![id.to_string(),effect.unit_id.to_string(),p,g,t,effect.idempotency_key,serde_json::to_string(&effect)?])?;
        tx.commit()?;
        Ok(())
    }
    /// G2 for a settled phase: the same `git_helper` row shape as the generic
    /// reservation, with the protected reader instead of generic authority.
    pub(crate) fn reserve_settled_helper(
        &mut self,
        currency: &crate::state::managed_binding::SettledCurrency,
        id: OperationId,
        path: &std::path::Path,
    ) -> Result<()> {
        ensure!(path.is_absolute(), "helper path must be absolute");
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        crate::state::managed_binding::validate_settled_tx(&tx, currency)?;
        let unit = currency.settled().settlement().unit();
        let effect = ManagedEffect {
            id,
            unit_id: unit.id,
            scope: unit.scope.clone(),
            kind: "git_helper".into(),
            idempotency_key: format!("helper-{id}"),
            expected_target: path.to_string_lossy().into(),
            state: EffectState::Pending,
            receipt: BTreeMap::new(),
            version: 1,
        };
        let (p, g, t) = scope_keys(&effect.scope)?;
        tx.execute("INSERT INTO managed_effects(id,unit_id,project_id,goal_id,task_id,idempotency_key,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,'pending',1,?7)",
            params![id.to_string(),effect.unit_id.to_string(),p,g,t,effect.idempotency_key,serde_json::to_string(&effect)?])?;
        tx.commit()?;
        Ok(())
    }
    /// G1/G3 for a settled phase: read-only protected currency of the pinned
    /// SAME Unit, run before spawn and on every fence tick.
    pub(crate) fn validate_settled_helper(
        &mut self,
        currency: &crate::state::managed_binding::SettledCurrency,
        pinned: &ExecutionUnit,
    ) -> Result<()> {
        let unit = currency.settled().settlement().unit();
        ensure!(
            pinned.id == unit.id
                && pinned.scope == unit.scope
                && pinned.generation == unit.generation
                && pinned.owner_epoch == unit.owner_epoch
                && pinned.session_id == unit.session_id,
            "settled helper pinned Unit differs"
        );
        let tx = self.connection.transaction()?;
        crate::state::managed_binding::validate_settled_tx(&tx, currency)?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn reserve_managed_effect(
        &mut self,
        authority: &ExecutionAuthority,
        effect: &ManagedEffect,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        reserve_effect_tx(&tx, authority, effect)?;
        tx.commit()?;
        Ok(())
    }
    pub fn managed_effect(&self, id: OperationId) -> Result<ManagedEffect> {
        effect_tx(&self.connection, id)
    }
    pub fn managed_effects(&self, unit: UnitId) -> Result<Vec<ManagedEffect>> {
        let mut s = self
            .connection
            .prepare("SELECT id FROM managed_effects WHERE unit_id=?1 ORDER BY rowid")?;
        let ids = s
            .query_map([unit.to_string()], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        ids.into_iter()
            .map(|id| {
                let e = self.managed_effect(id.parse()?)?;
                ensure!(e.unit_id == unit, "foreign effect");
                Ok(e)
            })
            .collect()
    }
    pub(crate) fn reconcile_managed_effect(
        &mut self,
        id: OperationId,
        expected: u64,
        state: EffectState,
        receipt: BTreeMap<String, String>,
    ) -> Result<()> {
        self.reconcile_managed_effect_pinned(id, expected, state, receipt, None)
    }
    pub(crate) fn reconcile_managed_effect_pinned(
        &mut self,
        id: OperationId,
        expected: u64,
        state: EffectState,
        receipt: BTreeMap<String, String>,
        driver: Option<&DriverReadTicket>,
    ) -> Result<()> {
        ensure!(
            receipt.len() <= 16
                && receipt.iter().all(|(k, v)| k.len() <= 64
                    && v.len() <= 256
                    && !v.chars().any(char::is_control)),
            "invalid effect safe receipt"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(ticket) = driver {
            ticket.validate_current_tx(&tx)?;
        }
        let mut effect = effect_tx(&tx, id)?;
        ensure!(
            !effect.kind.starts_with("native_phase_"),
            "private Native phase kind requires its original producer"
        );
        ensure!(
            effect.version == expected && effect.state != EffectState::Resolved,
            "effect CAS/terminal conflict"
        );
        ensure!(
            state != EffectState::Pending,
            "effect intent cannot be replayed"
        );
        if let Some(ticket) = driver {
            let unit = unit_tx(&tx, effect.unit_id)?;
            ticket.preparation_matches(&unit)?;
            ensure!(
                effect.scope == ticket.task().scope() && effect.kind == "git_helper",
                "Driver helper receipt differs"
            );
        }
        effect.state = state;
        effect.receipt = receipt;
        effect.version += 1;
        let changed = tx.execute(
            "UPDATE managed_effects SET state=?1,version=?2,body=?3 WHERE id=?4 AND version=?5",
            params![
                key(state),
                effect.version,
                serde_json::to_string(&effect)?,
                id.to_string(),
                expected
            ],
        )?;
        ensure!(changed == 1, "effect CAS conflict");
        append_event(
            &tx,
            &effect.scope,
            "execution.effect_reconciled",
            json!({"unit":effect.unit_id,"operation":id,"state":state}),
        )?;
        tx.commit()?;
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn release_execution_lease(
        &mut self,
        id: LeaseId,
        unit_id: UnitId,
        expected: u64,
        confirmed: bool,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut lease = lease_tx(&tx, id)?;
        ensure!(
            lease.id == id && lease.unit_id == unit_id && lease.version == expected,
            "lease identity/CAS conflict"
        );
        let unit = unit_tx(&tx, unit_id)?;
        ensure!(
            !unit.native_effects_open,
            "cannot release active execution resources"
        );
        if matches!(
            lease.kind,
            ResourceKind::Worktree | ResourceKind::Temp | ResourceKind::Output
        ) {
            ensure!(
                !unit.result_finalization_open,
                "result capture still depends on paths"
            );
        }
        lease.state = if confirmed {
            LeaseState::Released
        } else {
            LeaseState::Quarantined
        };
        lease.version += 1;
        tx.execute(
            "UPDATE resource_leases SET state=?1,version=?2,body=?3 WHERE id=?4 AND version=?5",
            params![
                key(lease.state),
                lease.version,
                serde_json::to_string(&lease)?,
                id.to_string(),
                expected
            ],
        )?;
        append_event(
            &tx,
            &unit.scope,
            "execution.resource_disposition",
            json!({"unit":unit_id,"lease":id,"state":lease.state}),
        )?;
        tx.commit()?;
        Ok(())
    }
}

pub(super) fn reserve_effect_tx(
    tx: &Transaction<'_>,
    authority: &ExecutionAuthority,
    effect: &ManagedEffect,
) -> Result<()> {
    ensure!(
        !effect.kind.starts_with("native_phase_"),
        "private Native phase kind requires its original producer"
    );
    ensure!(
        effect.unit_id == authority.unit_id
            && effect.scope == authority.scope
            && effect.state == EffectState::Pending
            && effect.version == 1
            && effect.receipt.is_empty()
            && !effect.expected_target.is_empty()
            && effect.expected_target.len() <= 4096
            && effect.idempotency_key.len() <= 256
            && !effect.idempotency_key.is_empty(),
        "invalid effect intent"
    );
    let unit = validate_authority(tx, authority, true, false)?;
    ensure!(
        !verification::is_command_unit(tx, unit.id)?,
        "command-only verifier cannot issue generic or delegated effects"
    );
    ensure!(
        unit.phase != WORKFLOW_SOURCE_BOOTSTRAP,
        "source preparation cannot issue native or delegated effects"
    );
    if matches!(effect.kind.as_str(), "publish" | "merge" | "deploy") {
        let ambiguous:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM managed_effects WHERE project_id=?1 AND state IN ('pending','unknown') AND json_extract(body,'$.kind')=?2 AND json_extract(body,'$.expected_target')=?3)",params![effect.scope.project_id.to_string(),effect.kind,effect.expected_target],|r|r.get(0))?;
        ensure!(
            !ambiguous,
            "unknown external target outcome requires reconciliation before this phase"
        );
    }
    let (p, g, t) = scope_keys(&effect.scope)?;
    tx.execute("INSERT INTO managed_effects(id,unit_id,project_id,goal_id,task_id,idempotency_key,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,'pending',1,?7)",
            params![effect.id.to_string(),effect.unit_id.to_string(),p,g,t,effect.idempotency_key,serde_json::to_string(effect)?])?;
    append_event(
        tx,
        &effect.scope,
        "execution.effect_intent",
        json!({"unit":effect.unit_id,"operation":effect.id,"kind":effect.kind}),
    )?;
    Ok(())
}
