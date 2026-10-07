//! Same-observation bookkeeping after normal currency revocation. This plan
//! cannot reserve helpers, admit input or issue any Native completion proof.
use super::*;

const UNIT_BYTES: usize = 16 * 1024;
const UNIT_COLUMNS: &str = "id,project_id,goal_id,task_id,kind,generation,owner_epoch,version,native_effects_open,result_finalization_open,worktree,branch,body";
const UNIT_CAS: &str = "SELECT EXISTS(SELECT 1 FROM execution_units WHERE id IS ?1 AND project_id IS ?2 AND goal_id IS ?3 AND task_id IS ?4 AND kind IS ?5 AND generation IS ?6 AND owner_epoch IS ?7 AND version IS ?8 AND native_effects_open IS ?9 AND result_finalization_open IS ?10 AND worktree IS ?11 AND branch IS ?12 AND body IS ?13)";

/// A bounded complete physical Unit preimage, without cleanup overlay or a
/// current Task-generation join. Neither flags nor rows are Native authority.
pub(in crate::state::execution::native_phase) struct LatestUnitImage {
    values: Vec<SqlValue>,
}
pub(in crate::state::execution::native_phase) struct RetiredUnitImage {
    values: Vec<SqlValue>,
    before: u64,
}
fn validate_indexed(tx: &Transaction<'_>, values: &[SqlValue]) -> Result<()> {
    let predicate = UNIT_COLUMNS
        .split(',')
        .enumerate()
        .map(|(i, n)| format!("u.{n} IS ?{}", i + 1))
        .collect::<Vec<_>>()
        .join(" AND ");
    ensure!(tx.query_row(&format!("SELECT EXISTS(SELECT 1 FROM execution_units u JOIN task_execution t ON t.task_id=u.task_id WHERE {predicate} AND t.project_id=u.project_id AND t.goal_id=u.goal_id AND t.generation=u.generation)"), params_from_iter(values), |r| r.get::<_, bool>(0))?, "non-success indexed Unit/generation changed");
    Ok(())
}
impl RetiredUnitImage {
    pub(in crate::state::execution::native_phase) fn validate_indexed_tx(
        &self,
        tx: &Transaction<'_>,
    ) -> Result<()> {
        validate_indexed(tx, &self.values)
    }
    pub(in crate::state::execution::native_phase) fn unit_id(&self) -> &str {
        match &self.values[0] {
            SqlValue::Text(id) => id,
            _ => unreachable!("sealed Unit ID is text"),
        }
    }
    pub(in crate::state::execution::native_phase) fn versions(&self) -> (u64, u64) {
        match self.values[7] {
            SqlValue::Integer(v) => (self.before, v as u64),
            _ => unreachable!("sealed Unit version is integer"),
        }
    }
}
// Fixed physical column vocabulary only; no caller SQL or grant is accepted.
fn unit_projection() -> Vec<String> {
    UNIT_COLUMNS.split(',').enumerate().map(|(index, column)| {
            if (5..=9).contains(&index) {
                format!("CASE WHEN typeof({column})='integer' THEN {column} END")
            } else {
                let limit = if index == 12 { UNIT_BYTES } else { 4096 };
                // Only branch is nullable. An invalid non-NULL branch must
                // not masquerade as its absent physical preimage.
                let invalid = if index == 11 { " WHEN branch IS NULL THEN NULL ELSE X'00'" } else { "" };
                format!("CASE WHEN typeof({column})='text' AND length(CAST({column} AS BLOB))<={limit} THEN {column}{invalid} END")
            }
    }).collect()
}
impl LatestUnitImage {
    pub(in crate::state::execution::native_phase) fn validate_indexed_tx(
        &self,
        tx: &Transaction<'_>,
    ) -> Result<()> {
        validate_indexed(tx, &self.values)
    }
    pub(in crate::state::execution::native_phase) fn plan_retired(
        &self,
        launch: &PhaseLaunchParts,
        at: i64,
    ) -> Result<RetiredUnitImage> {
        self.validate_original(launch.allocation().unit_snapshot())?;
        let SqlValue::Text(raw) = &self.values[12] else {
            anyhow::bail!("non-success Unit body absent")
        };
        let body = crate::execution::strict_json::decode(
            raw.as_bytes(),
            crate::execution::strict_json::Limits {
                frame_bytes: UNIT_BYTES,
                depth: 16,
                nodes: 512,
                string_bytes: UNIT_BYTES,
                total_string_bytes: UNIT_BYTES,
                object_entries: 128,
                array_entries: 64,
            },
        )?;
        let mut unit: ExecutionUnit = serde_json::from_value(body)?;
        registration_unit(&unit, launch)?;
        ensure!(
            matches!(
                unit.wait_reason,
                None | Some(WaitReason::Quota | WaitReason::Capacity)
            ),
            "non-success Unit wait reason differs"
        );
        let before = unit.version;
        unit.version = unit
            .version
            .checked_add(1)
            .filter(|v| *v <= i64::MAX as u64)
            .context("non-success Unit version exhausted")?;
        unit.state = UnitState::Retired;
        unit.native_effects_open = false;
        unit.result_finalization_open = false;
        unit.wait_reason = None;
        unit.disposition = Disposition::Refused;
        unit.updated_at = at;
        let raw = serde_json::to_string(&unit)?;
        ensure!(
            raw.len() <= UNIT_BYTES,
            "non-success retired Unit body exceeds bound"
        );
        let mut values = self.values.clone();
        values[7] = SqlValue::Integer(i64::try_from(unit.version)?);
        values[8] = SqlValue::Integer(0);
        values[9] = SqlValue::Integer(0);
        values[12] = SqlValue::Text(raw);
        LatestUnitImage {
            values: values.clone(),
        }
        .validate_original(launch.allocation().unit_snapshot())?;
        ensure!(
            unit.version > before,
            "non-success retired Unit version not increasing"
        );
        Ok(RetiredUnitImage { values, before })
    }
    pub(in crate::state::execution::native_phase) fn write_retired_tx(
        &self,
        tx: &Transaction<'_>,
        after: &RetiredUnitImage,
    ) -> Result<()> {
        let set = UNIT_COLUMNS
            .split(',')
            .enumerate()
            .map(|(i, n)| format!("{n}=?{}", i + 1))
            .collect::<Vec<_>>()
            .join(",");
        let predicate = UNIT_COLUMNS
            .split(',')
            .enumerate()
            .map(|(i, n)| format!("{n} IS ?{}", i + 14))
            .collect::<Vec<_>>()
            .join(" AND ");
        ensure!(
            tx.execute(
                &format!("UPDATE execution_units SET {set} WHERE {predicate}"),
                params_from_iter(after.values.iter().chain(&self.values))
            )? == 1,
            "non-success complete Unit CAS changed"
        );
        Ok(())
    }
    pub(in crate::state::execution::native_phase) fn read(
        tx: &Transaction<'_>,
        original: &ExecutionUnit,
    ) -> Result<Self> {
        // First qualify types and all copied byte lengths before text copying.
        let columns = unit_projection().join(",");
        let mut statement = tx.prepare(&format!(
            "SELECT {columns} FROM execution_units WHERE id=?1"
        ))?;
        let image = statement.query_row([original.id.to_string()], |row| {
            let mut bytes = 13usize * 8;
            for index in 0..13 {
                match row.get_ref(index)? {
                    ValueRef::Text(raw) => {
                        let limit = if index == 12 { UNIT_BYTES } else { 4096 };
                        if raw.len() > limit {
                            return Err(rusqlite::Error::InvalidQuery);
                        }
                        bytes = bytes
                            .checked_add(raw.len())
                            .ok_or(rusqlite::Error::InvalidQuery)?;
                    }
                    ValueRef::Integer(_) if (5..=9).contains(&index) => {
                        bytes = bytes.checked_add(8).ok_or(rusqlite::Error::InvalidQuery)?;
                    }
                    ValueRef::Null if index == 11 => {}
                    _ => return Err(rusqlite::Error::InvalidQuery),
                }
            }
            if bytes > UNIT_BYTES + 8 * 4096 + 13 * 8 + 5 * 8 {
                return Err(rusqlite::Error::InvalidQuery);
            }
            Ok(Self {
                values: (0..13)
                    .map(|index| row.get(index))
                    .collect::<rusqlite::Result<_>>()?,
            })
        })?;
        drop(statement);
        image.validate_original(original)?;
        Ok(image)
    }
    pub(in crate::state::execution::native_phase) fn validate_original(
        &self,
        original: &ExecutionUnit,
    ) -> Result<()> {
        ensure!(self.values.len() == 13, "incomplete closure Unit image");
        for (index, value) in self.values.iter().enumerate() {
            match value {
                SqlValue::Text(raw) if !(5..=9).contains(&index) => {
                    ensure!(
                        raw.len() <= if index == 12 { UNIT_BYTES } else { 4096 },
                        "closure Unit copied column exceeds bound"
                    );
                }
                SqlValue::Integer(_) if (5..=9).contains(&index) => {}
                SqlValue::Null if index == 11 => {}
                _ => anyhow::bail!("closure Unit copied column type differs"),
            }
        }
        let SqlValue::Text(raw) = &self.values[12] else {
            anyhow::bail!("closure Unit body absent")
        };
        let body = crate::execution::strict_json::decode(
            raw.as_bytes(),
            crate::execution::strict_json::Limits {
                frame_bytes: UNIT_BYTES,
                depth: 16,
                nodes: 512,
                string_bytes: UNIT_BYTES,
                total_string_bytes: UNIT_BYTES,
                object_entries: 128,
                array_entries: 64,
            },
        )?;
        let unit: ExecutionUnit = serde_json::from_value(body)?;
        let (project, goal, task) = scope_keys(&unit.scope)?;
        let expected = vec![
            SqlValue::Text(unit.id.to_string()),
            SqlValue::Text(project),
            SqlValue::Text(goal),
            SqlValue::Text(task),
            SqlValue::Text(key(unit.kind)),
            SqlValue::Integer(i64::try_from(unit.generation)?),
            SqlValue::Integer(i64::try_from(unit.owner_epoch)?),
            SqlValue::Integer(i64::try_from(unit.version)?),
            SqlValue::Integer(i64::from(unit.native_effects_open)),
            SqlValue::Integer(i64::from(unit.result_finalization_open)),
            SqlValue::Text(
                unit.worktree
                    .to_str()
                    .context("closure Unit path not UTF-8")?
                    .into(),
            ),
            unit.branch.clone().map_or(SqlValue::Null, SqlValue::Text),
            SqlValue::Text(raw.clone()),
        ];
        ensure!(
            self.values == expected,
            "closure Unit full indexed/body image differs"
        );
        ensure!(
            unit.id == original.id
                && unit.scope == original.scope
                && unit.kind == original.kind
                && unit.generation == original.generation
                && unit.owner_epoch == original.owner_epoch
                && unit.phase == original.phase
                && unit.provider == original.provider
                && unit.worktree == original.worktree
                && unit.branch == original.branch
                && unit.base_sha == original.base_sha
                && unit.profile_digest == original.profile_digest
                && unit.cookie == original.cookie
                && unit.created_at == original.created_at
                && unit.version >= original.version
                && unit.version > 0
                && (unit.kind != UnitKind::Reviewer || unit.artifact_id == original.artifact_id),
            "closure Unit immutable original identity changed"
        );
        Ok(())
    }
    pub(in crate::state::execution::native_phase) fn validate_tx(
        &self,
        tx: &Transaction<'_>,
    ) -> Result<()> {
        ensure!(
            tx.query_row(UNIT_CAS, params_from_iter(&self.values), |row| row
                .get::<_, bool>(0))?,
            "latest complete closure Unit CAS changed; original observation held"
        );
        Ok(())
    }
}

/// Constructed only from a retained authentic settlement plus a coherent read.
/// The latest Unit never replaces the original actor/helper/effect plan.
pub(crate) struct NativeVersionClosurePlan {
    settlement: Arc<NativeHelperSettlementPlan>,
    unit: LatestUnitImage,
}
impl NativeVersionClosurePlan {
    fn validate_original(&self) -> Result<()> {
        validate_settlement_original(&self.settlement)
    }
}
fn validate_settlement_original(settlement: &NativeHelperSettlementPlan) -> Result<()> {
    ensure!(
        settlement.observation.matches_plan(&settlement.original),
        "nongrant closure lacks SAME authentic observation"
    );
    let actor = settlement.original.actor();
    actor.validate_original()?;
    actor.launch().validate_preparation_original()
}

impl Store {
    /// Hashing/receipt encoding and this bounded snapshot precede SharedStore.
    /// A revoked actor is allowed to record its work, never to issue a grant.
    pub(crate) fn plan_phase_version_closure(
        runtime: &crate::execution::RuntimeOwner,
        settlement: Arc<NativeHelperSettlementPlan>,
    ) -> Result<Arc<NativeVersionClosurePlan>> {
        validate_settlement_original(&settlement)?;
        let launch = settlement.original.actor().launch();
        ensure!(
            std::ptr::eq(
                runtime,
                launch
                    .allocation()
                    .selected_port()
                    .selected_adapter()?
                    .owner
                    .as_ref()
            ),
            "closure snapshot uses another selected owner"
        );
        let unit = snapshot(runtime, |tx| {
            let budget = InventoryBudget::new(tx)?;
            budget.finish((|| {
                selected_database(tx, launch)?;
                validate_settlement_original(&settlement)?;
                let unit = LatestUnitImage::read(tx, launch.allocation().unit_snapshot())?;
                let current = Inventory::read(tx, launch.allocation().facts().unit_id)?;
                ensure!(
                    current == *settlement.original.pending || current == *settlement.after,
                    "closure original complete inventory changed"
                );
                Ok(unit)
            })())
        })?;
        Ok(Arc::new(NativeVersionClosurePlan { settlement, unit }))
    }
    /// No normal authority or permission is returned or reopened. A changed
    /// Unit/effect image refuses the write and retains SAME observation Held.
    pub(crate) fn close_phase_version_observation(
        &mut self,
        closure: &Arc<NativeVersionClosurePlan>,
    ) -> Result<NativeHelperSettlementCommit> {
        closure.validate_original()?;
        let settlement = &closure.settlement;
        let plan = &settlement.original;
        selected_database(&self.connection, plan.actor().launch())?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        {
            let budget = InventoryBudget::new(&tx)?;
            budget.finish((|| {
                closure.validate_original()?;
                closure.unit.validate_tx(&tx)?;
                let current = Inventory::read(&tx, plan.actor().launch().allocation().facts().unit_id)?;
                if current == *settlement.after { return Ok(()); }
                ensure!(current == *plan.pending, "closure original full inventory CAS changed");
                let mut values = settlement.effect.values();
                values.extend(plan.effect.values());
                ensure!(tx.execute("UPDATE managed_effects SET id=?1,unit_id=?2,project_id=?3,goal_id=?4,task_id=?5,idempotency_key=?6,state=?7,body=?8,version=?9 WHERE id IS ?10 AND unit_id IS ?11 AND project_id IS ?12 AND goal_id IS ?13 AND task_id IS ?14 AND idempotency_key IS ?15 AND state IS ?16 AND body IS ?17 AND version IS ?18",
                    params_from_iter(values))? == 1, "closure exact original effect CAS conflict");
                Ok(())
            })())?;
        }
        tx.commit()?;
        Ok(NativeHelperSettlementCommit {
            original: closure.settlement.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Pure DTO/copied-value controls only. No persisted Unit, Driver, owner,
    // accepted Goal or Native actor/observation/capability is constructed.
    fn original() -> ExecutionUnit {
        ExecutionUnit {
            id: UnitId::new(),
            scope: Scope {
                project_id: ProjectId::new(),
                goal_id: Some(GoalId::new()),
                task_id: Some(TaskId::new()),
            },
            kind: UnitKind::Executor,
            generation: 1,
            owner_epoch: 1,
            version: 1,
            phase: "Implement".into(),
            provider: "codex".into(),
            state: UnitState::Reserved,
            native_effects_open: true,
            result_finalization_open: true,
            work: None,
            cleanup: CleanupOutcome::Unknown,
            disposition: Disposition::Active,
            worktree: PathBuf::from("/tmp/nongrant-closure"),
            branch: Some("rrx/test".into()),
            base_sha: "a".repeat(40),
            profile_digest: "b".repeat(64),
            cookie: uuid::Uuid::new_v4().to_string(),
            session_id: None,
            artifact_id: None,
            wait_reason: None,
            capacity_retry_at: None,
            created_at: 1,
            updated_at: 1,
        }
    }
    fn image(unit: &ExecutionUnit) -> LatestUnitImage {
        let (project, goal, task) = scope_keys(&unit.scope).unwrap();
        LatestUnitImage {
            values: vec![
                SqlValue::Text(unit.id.to_string()),
                SqlValue::Text(project),
                SqlValue::Text(goal),
                SqlValue::Text(task),
                SqlValue::Text(key(unit.kind)),
                SqlValue::Integer(unit.generation as i64),
                SqlValue::Integer(unit.owner_epoch as i64),
                SqlValue::Integer(unit.version as i64),
                SqlValue::Integer(i64::from(unit.native_effects_open)),
                SqlValue::Integer(i64::from(unit.result_finalization_open)),
                SqlValue::Text(unit.worktree.to_str().unwrap().into()),
                unit.branch.clone().map_or(SqlValue::Null, SqlValue::Text),
                SqlValue::Text(serde_json::to_string(unit).unwrap()),
            ],
        }
    }
    #[test]
    fn nongrant_closure_latest_retirement_preserves_closed_flags() {
        let original = original();
        let mut retired = original.clone();
        retired.version += 1;
        retired.native_effects_open = false;
        retired.result_finalization_open = false;
        retired.state = UnitState::Retired;
        let image = image(&retired);
        let before = image.values.clone();
        image.validate_original(&original).unwrap();
        assert_eq!(image.values, before);
        assert_eq!(image.values[8], SqlValue::Integer(0));
        assert_eq!(image.values[9], SqlValue::Integer(0));
    }
    #[test]
    fn nongrant_closure_full_image_rejects_every_index_alias() {
        let original = original();
        image(&original).validate_original(&original).unwrap();
        for index in 0..13 {
            let mut copied = image(&original);
            copied.values[index] = match &copied.values[index] {
                SqlValue::Text(value) => SqlValue::Text(format!("{value}x")),
                SqlValue::Integer(value) => SqlValue::Integer(value + 1),
                SqlValue::Null => SqlValue::Text("foreign".into()),
                _ => unreachable!(),
            };
            assert!(
                copied.validate_original(&original).is_err(),
                "accepted changed column {index}"
            );
        }
    }
    #[test]
    fn nongrant_closure_rejects_replaced_original_identity_and_stale_version() {
        let original = original();
        let mut changed = original.clone();
        changed.cookie.push('x');
        assert!(image(&changed).validate_original(&original).is_err());
        changed = original.clone();
        changed.generation += 1;
        assert!(image(&changed).validate_original(&original).is_err());
        changed = original.clone();
        changed.owner_epoch += 1;
        assert!(image(&changed).validate_original(&original).is_err());
        changed = original.clone();
        changed.version = 0;
        assert!(image(&changed).validate_original(&original).is_err());
        let mut reviewer = original.clone();
        reviewer.kind = UnitKind::Reviewer;
        changed = reviewer.clone();
        changed.artifact_id = Some(ArtifactId::new());
        assert!(image(&changed).validate_original(&reviewer).is_err());
    }
    #[test]
    fn nongrant_closure_bounded_body_rejects_oversize_and_duplicate_keys() {
        let original = original();
        let mut copied = image(&original);
        let raw = serde_json::to_string(&original).unwrap();
        copied.values[12] = SqlValue::Text(format!("{raw}{}", " ".repeat(16 * 1024 - raw.len())));
        copied.validate_original(&original).unwrap();
        copied.values[12] =
            SqlValue::Text(format!("{raw}{}", " ".repeat(16 * 1024 + 1 - raw.len())));
        assert!(copied.validate_original(&original).is_err());
        let mut copied = image(&original);
        copied.values[10] = SqlValue::Text("x".repeat(4097));
        assert!(copied.validate_original(&original).is_err());
        let mut copied = image(&original);
        let SqlValue::Text(body) = &copied.values[12] else {
            unreachable!()
        };
        copied.values[12] = SqlValue::Text(format!("{{\"id\":\"{}\",{}", original.id, &body[1..]));
        assert!(copied.validate_original(&original).is_err());
    }
    #[test]
    fn nongrant_closure_sql_nullable_projection_distinguishes_invalid_values() {
        // Readonly scalar SQL only. This executes the production expression,
        // without an execution_units table, Unit CAS or actor-chain fixture.
        let connection = Connection::open_in_memory().unwrap();
        let projection = unit_projection();
        assert_eq!(projection.len(), 13);
        let sql = format!(
            "WITH scalar_branch(branch) AS (SELECT ?1) SELECT {} FROM scalar_branch",
            projection[11]
        );
        for (input, expected) in [
            (SqlValue::Null, SqlValue::Null),
            (
                SqlValue::Text("branch".into()),
                SqlValue::Text("branch".into()),
            ),
            (
                SqlValue::Text("x".repeat(4096)),
                SqlValue::Text("x".repeat(4096)),
            ),
            (SqlValue::Text("x".repeat(4097)), SqlValue::Blob(vec![0])),
            (SqlValue::Integer(1), SqlValue::Blob(vec![0])),
            (SqlValue::Blob(vec![1]), SqlValue::Blob(vec![0])),
        ] {
            let actual: SqlValue = connection
                .query_row(&sql, params![input], |row| row.get(0))
                .unwrap();
            assert_eq!(
                actual, expected,
                "invalid non-NULL projection must remain distinguishable from NULL"
            );
        }
    }
}
