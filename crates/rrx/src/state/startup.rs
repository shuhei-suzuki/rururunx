//! Nongrant startup journal. Only fresh observations can qualify the memory gate.
use super::*;
use crate::execution::RuntimeOwner;

pub(in crate::state) fn install_schema(tx: &Transaction<'_>) -> Result<()> {
    tx.execute_batch(include_str!("startup.sql"))?;
    Ok(())
}
pub(in crate::state) fn fence_epoch(tx: &Transaction<'_>, epoch: u64) -> Result<()> {
    tx.execute("UPDATE runtime_startup_probes SET state='unknown',version=version+1 WHERE state='pending' AND owner_epoch<?1", [epoch])?;
    tx.execute(
        "DELETE FROM runtime_startup_probes WHERE state='confirmed' AND owner_epoch<=?1",
        [epoch.saturating_sub(64)],
    )?;
    Ok(())
}

/// Exact committed pending image; no Deserialize, Clone or authority conversion.
pub(crate) struct StartupIntent {
    epoch: u64,
    installation: String,
    alias: String,
    step: String,
    body: String,
    version: u64,
}
fn current(tx: &Connection, owner: &RuntimeOwner) -> Result<()> {
    let identity: (String, u64) = tx.query_row(
        "SELECT instance_id,epoch FROM runtime_epoch WHERE singleton=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    ensure!(
        identity == (owner.instance_id().into(), owner.epoch()),
        "startup owner fenced"
    );
    Ok(())
}
impl Store {
    pub(crate) fn startup_intent(
        &mut self,
        owner: &RuntimeOwner,
        installation: uuid::Uuid,
        alias: &str,
        step: &str,
        body: &Value,
    ) -> Result<StartupIntent> {
        let body = serde_json::to_string(body)?;
        ensure!(
            body.len() <= 4096 && !alias.is_empty(),
            "startup intent over bound"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, owner)?;
        let (total, uncertain): (u64,u64) = tx.query_row("SELECT count(*),coalesce(sum(state IN ('pending','unknown')),0) FROM runtime_startup_probes", [], |r| Ok((r.get(0)?,r.get(1)?)))?;
        ensure!(
            total < 8192 && uncertain < 4096,
            "startup journal capacity exhausted"
        );
        let (rows, aliases, exists): (u64,u64,bool) = tx.query_row("SELECT count(*),count(DISTINCT alias),coalesce(max(alias=?3),0) FROM runtime_startup_probes WHERE owner_epoch=?1 AND installation=?2", params![owner.epoch(),installation.to_string(),alias], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
        let epoch_rows: u64 = tx.query_row(
            "SELECT count(*) FROM runtime_startup_probes WHERE owner_epoch=?1",
            [owner.epoch()],
            |r| r.get(0),
        )?;
        ensure!(
            rows < 96 && epoch_rows < 96 && (exists || aliases < 32),
            "startup epoch budget exhausted"
        );
        tx.execute("INSERT INTO runtime_startup_probes(owner_epoch,installation,alias,step,state,version,body) VALUES(?1,?2,?3,?4,'pending',1,?5)",params![owner.epoch(),installation.to_string(),alias,step,body])?;
        tx.commit()?;
        Ok(StartupIntent {
            epoch: owner.epoch(),
            installation: installation.to_string(),
            alias: alias.into(),
            step: step.into(),
            body,
            version: 1,
        })
    }
    pub(crate) fn settle_startup(
        &mut self,
        owner: &RuntimeOwner,
        intent: &StartupIntent,
        known: bool,
        body: &Value,
    ) -> Result<()> {
        let body = serde_json::to_string(body)?;
        ensure!(
            body.len() <= 4096 && intent.epoch == owner.epoch(),
            "startup settlement over bound or foreign epoch"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, owner)?;
        let changed = tx.execute("UPDATE runtime_startup_probes SET state=?1,version=version+1,body=?2 WHERE owner_epoch=?3 AND installation=?4 AND alias=?5 AND step=?6 AND state='pending' AND version=?7 AND body=?8",params![if known {"confirmed"} else {"unknown"},body,intent.epoch,intent.installation,intent.alias,intent.step,intent.version,intent.body])?;
        ensure!(changed == 1, "startup settlement exact CAS differs");
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn old_startup_unknown(&self, owner: &RuntimeOwner) -> Result<u64> {
        current(&self.connection, owner)?;
        Ok(self.connection.query_row(
            "SELECT count(*) FROM runtime_startup_probes WHERE owner_epoch<?1 AND state='unknown'",
            [owner.epoch()],
            |r| r.get(0),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// MP-1 distinguishes historical manifests and refuses old writers after migration.
    #[test]
    fn v12_migration_checks_historical_and_current_layouts_atomically() {
        for version in [0, 9, 10, 11, 12] {
            let connection = if version == 0 {
                Connection::open_in_memory().unwrap()
            } else if version == 9 {
                let c = Connection::open_in_memory().unwrap();
                for sql in [
                    include_str!("schema.sql"),
                    include_str!("execution.sql"),
                    include_str!("execution/native_results.sql"),
                    include_str!("execution/source_recovery.sql"),
                    include_str!("execution/verification.sql"),
                    include_str!("runtime/schema.sql"),
                ] {
                    c.execute_batch(sql).unwrap();
                }
                // Historical input layout only, never an execution success seed.
                c.execute(
                    "INSERT INTO runtime_epoch(singleton,instance_id,epoch) VALUES(1,?1,0)",
                    [uuid::Uuid::new_v4().to_string()],
                )
                .unwrap();
                for table in execution::M11_TABLES
                    .iter()
                    .filter(|t| !managed_binding::TABLES.contains(t))
                {
                    for action in ["INSERT", "UPDATE", "DELETE"] {
                        c.execute_batch(&format!("CREATE TRIGGER writer_{table}_{action} BEFORE {action} ON {table} WHEN rrx_writer_contract_version()<>9 BEGIN SELECT RAISE(ABORT,'incompatible rrx writer contract'); END;")).unwrap();
                    }
                }
                c.pragma_update(None, "application_id", APPLICATION_ID)
                    .unwrap();
                c.pragma_update(None, "user_version", 9).unwrap();
                c
            } else {
                let mut s = Store::memory().unwrap();
                if version < 12 {
                    let tx = s.connection.transaction().unwrap();
                    tx.execute_batch("DROP TABLE runtime_startup_probes")
                        .unwrap();
                    if version == 10 {
                        tx.execute_batch("DROP INDEX goals_by_project").unwrap();
                    }
                    execution::install_writer_guards_at(&tx, version).unwrap();
                    tx.pragma_update(None, "user_version", version).unwrap();
                    tx.commit().unwrap();
                }
                s.connection
            };
            let store =
                Store::initialize(connection).unwrap_or_else(|e| panic!("MP-1 v{version}: {e:#}"));
            assert_eq!(store.schema_version().unwrap(), 12);
            managed_binding::validate_current_layout(&store.connection).unwrap();
            assert_eq!(
                store
                    .connection
                    .query_row("SELECT count(*) FROM runtime_startup_probes", [], |r| r
                        .get::<_, u64>(0))
                    .unwrap(),
                0
            );
            // An old contract function cannot write any current table, including journal.
            store
                .connection
                .create_scalar_function(
                    "rrx_writer_contract_version",
                    0,
                    rusqlite::functions::FunctionFlags::SQLITE_UTF8,
                    |_| Ok(11),
                )
                .unwrap();
            assert!(
                store
                    .connection
                    .execute("INSERT INTO runtime_startup_probes(owner_epoch,installation,alias,step,state,version,body) VALUES(1,'old','worker','version','pending',1,'{}')", [])
                    .is_err(),
                "MP-1: old writer bypassed startup guard"
            );
            assert!(
                store
                    .connection
                    .execute("UPDATE runtime_epoch SET epoch=epoch", [])
                    .is_err(),
                "MP-1: old writer bypassed existing guard"
            );
        }
    }
}

#[cfg(test)]
mod journal_tests {
    use super::*;
    fn intent(
        owner: &RuntimeOwner,
        id: uuid::Uuid,
        alias: &str,
        step: &str,
    ) -> Result<StartupIntent> {
        owner.store.lock().unwrap().startup_intent(
            owner,
            id,
            alias,
            step,
            &json!({"instance":owner.instance_id(),"provider":"codex"}),
        )
    }
    /// AC-10/MP-1 exact body/version CAS and current owner epoch precede every settlement.
    #[test]
    fn journal_cas_and_epoch_fence_never_reconstruct_a_pass() {
        let temp = tempfile::tempdir().unwrap();
        let state = temp.path().join("state.db");
        let owner = RuntimeOwner::open(&state).unwrap();
        let id = uuid::Uuid::new_v4();
        let saved = intent(&owner, id, "a", "version").unwrap();
        assert!(intent(&owner, id, "a", "version").is_err());
        let c = current_test_writer(&state).unwrap();
        for (sql, undo) in [
            (
                "UPDATE runtime_startup_probes SET body=body||' '",
                "UPDATE runtime_startup_probes SET body=?1",
            ),
            (
                "UPDATE runtime_startup_probes SET version=version+1",
                "UPDATE runtime_startup_probes SET version=version-1 WHERE body=?1",
            ),
        ] {
            c.execute(sql, []).unwrap();
            assert!(
                owner
                    .store
                    .lock()
                    .unwrap()
                    .settle_startup(&owner, &saved, true, &json!({"known":true}))
                    .is_err(),
                "AC-10: changed pending image settled"
            );
            c.execute(undo, [&saved.body]).unwrap();
        }
        owner
            .store
            .lock()
            .unwrap()
            .settle_startup(
                &owner,
                &saved,
                true,
                &json!({"known":true,"protocol":false}),
            )
            .unwrap();
        assert!(
            owner
                .store
                .lock()
                .unwrap()
                .settle_startup(&owner, &saved, true, &json!({}))
                .is_err(),
            "AC-10: settlement replayed"
        );
        intent(&owner, id, "b", "version").unwrap();
        owner.store.lock().unwrap().begin_execution_epoch().unwrap();
        assert!(
            intent(&owner, id, "c", "version").is_err(),
            "AC-10: stale owner intent accepted"
        );
        assert!(
            owner
                .store
                .lock()
                .unwrap()
                .settle_startup(&owner, &saved, true, &json!({}))
                .is_err()
        );
        drop(c);
        drop(owner);
        let next = RuntimeOwner::open(&state).unwrap();
        assert_eq!(
            next.store
                .lock()
                .unwrap()
                .old_startup_unknown(&next)
                .unwrap(),
            1,
            "MP-1: pending not fenced unknown"
        );
    }
    /// MP-1 independently bounds alias cardinality, epoch rows and complete serialized bodies.
    #[test]
    fn journal_alias_row_and_body_budgets_are_independent() {
        for all_steps in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let owner = RuntimeOwner::open(&temp.path().join("state.db")).unwrap();
            let id = uuid::Uuid::new_v4();
            for i in 0..32 {
                for step in if all_steps {
                    vec!["version", "schema", "handshake"]
                } else {
                    vec!["version"]
                } {
                    intent(&owner, id, &format!("alias-{i}"), step).unwrap();
                }
            }
            assert!(
                intent(&owner, id, "extra", "version").is_err(),
                "MP-1: alias/row budget widened"
            );
            if all_steps {
                assert!(
                    intent(&owner, uuid::Uuid::new_v4(), "extra", "version").is_err(),
                    "MP-1: installation split widened epoch budget"
                );
            }
        }
        let temp = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::open(&temp.path().join("state.db")).unwrap();
        let id = uuid::Uuid::new_v4();
        let saved = owner
            .store
            .lock()
            .unwrap()
            .startup_intent(&owner, id, "a", "version", &json!("x".repeat(4094)))
            .unwrap();
        assert!(
            owner
                .store
                .lock()
                .unwrap()
                .startup_intent(&owner, id, "b", "version", &json!("x".repeat(4095)))
                .is_err(),
            "MP-1: intent body bound widened"
        );
        assert!(
            owner
                .store
                .lock()
                .unwrap()
                .settle_startup(&owner, &saved, true, &json!("x".repeat(4095)))
                .is_err(),
            "MP-1: settlement body bound widened"
        );
        owner
            .store
            .lock()
            .unwrap()
            .settle_startup(&owner, &saved, true, &json!("x".repeat(4094)))
            .unwrap();
    }
    /// MP-1 reserves pending capacity, preserves unknown rows, and prunes only old confirmed rows.
    #[test]
    fn journal_total_uncertain_and_64_epoch_retention_use_real_intents() {
        for uncertain_cap in [true, false] {
            let temp = tempfile::tempdir().unwrap();
            let state = temp.path().join("state.db");
            let mut owner = RuntimeOwner::open(&state).unwrap();
            let mut count = 0usize;
            // Fill uncertain capacity exactly, or reserve 2049 unknown then
            // 6143 confirmed within 64 epochs to hit total with one epoch slot free.
            let uncertain = if uncertain_cap { 4096 } else { 2049 };
            for target in [uncertain, if uncertain_cap { 4096 } else { 8192 }] {
                while count < target {
                    let id = uuid::Uuid::new_v4();
                    for i in 0..96.min(target - count) {
                        let saved = intent(
                            &owner,
                            id,
                            &format!("alias-{}", i / 3),
                            ["version", "schema", "handshake"][i % 3],
                        )
                        .unwrap();
                        if count >= uncertain {
                            owner
                                .store
                                .lock()
                                .unwrap()
                                .settle_startup(&owner, &saved, true, &json!({"known":true}))
                                .unwrap();
                        }
                        count += 1;
                    }
                    if count < target {
                        drop(owner);
                        owner = RuntimeOwner::open(&state).unwrap();
                    }
                }
                if !uncertain_cap && count == uncertain {
                    drop(owner);
                    owner = RuntimeOwner::open(&state).unwrap();
                }
            }
            let error = intent(&owner, uuid::Uuid::new_v4(), "extra", "version")
                .err()
                .expect("MP-1: journal cap widened");
            assert!(
                error.to_string().contains("capacity exhausted"),
                "MP-1: wrong refusal boundary {error:#}"
            );
            let c = current_test_writer(&state).unwrap();
            let unknown: u64 = c
                .query_row(
                    "SELECT count(*) FROM runtime_startup_probes WHERE state='unknown'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(unknown > 0);
            drop(c);
            drop(owner);
            owner = RuntimeOwner::open(&state).unwrap();
            assert!(
                owner
                    .store
                    .lock()
                    .unwrap()
                    .old_startup_unknown(&owner)
                    .unwrap()
                    >= unknown,
                "MP-1: unknown history pruned"
            );
            if !uncertain_cap {
                // The oldest confirmed epoch ages out on the next fence.
                let id = uuid::Uuid::new_v4();
                intent(&owner, id, "after-prune", "version")
                    .expect("MP-1: confirmed history not pruned at 64 epochs");
            } else {
                assert!(
                    intent(&owner, uuid::Uuid::new_v4(), "after-restart", "version").is_err(),
                    "MP-1: restart cleared uncertain capacity"
                );
            }
        }
    }
}
