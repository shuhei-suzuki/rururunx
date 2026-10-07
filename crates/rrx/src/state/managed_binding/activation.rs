//! Original installed composition activation. Parsing and hashes are nongrant
//! planning work; only the retained Driver transaction can publish the row.
use super::{
    ActivationRoster, canonical,
    permits::{ExactRowMutation, PrivatePermitManager},
};
use crate::{
    adapter::native::NativePhasePort,
    domain::{Record, RecordId, RecordKind, Scope, SessionRole, Task},
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Transaction, params, params_from_iter, types::Value as SqlValue};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativeContract {
    pub(super) workflow_id: RecordId,
    pub(super) project_id: crate::domain::ProjectId,
    pub(super) goal_id: crate::domain::GoalId,
    pub(super) task_id: crate::domain::TaskId,
    pub(super) owner_epoch: u64,
    pub(super) origin: uuid::Uuid,
    pub(super) profile_digest: String,
    pub(super) contract_state: String,
    pub(super) version: u64,
    pub(super) members: Vec<String>,
}

pub(crate) fn member_digest(role: SessionRole, port: &NativePhasePort) -> Result<String> {
    let adapter = port.selected_adapter()?;
    let declaration = adapter
        .compatibility
        .as_ref()
        .context("installed Native declaration absent")?;
    let bytes = canonical::encode(
        &json!({"role":role,"alias":port.alias(),"provider":port.provider(),
        "program":port.program(),"declaration_digest":declaration.digest(),"port_origin":port.origin_id()}),
        4096,
    )?;
    Ok(canonical::digest(b"rrx.native-roster-member/v1\0", &bytes))
}
pub(crate) fn roster_digest(members: &[String]) -> Result<String> {
    ensure!(
        !members.is_empty()
            && members.len() <= 9
            && members.windows(2).all(|p| p[0] < p[1])
            && members.iter().all(|m| m.len() == 64
                && m.bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))),
        "Native roster members invalid"
    );
    Ok(canonical::digest(
        b"rrx.native-roster/v1\0",
        &canonical::encode(&json!(members), 4096)?,
    ))
}

/// No Clone/Deserialize/Default and no SQL/DTO constructor. Keeps the original
/// issuer's identity alive without keeping Runtime alive across Source awaits.
pub(crate) struct NativeActivationPlan {
    roster: ActivationRoster,
    workflow: RecordId,
    scope: Scope,
    epoch: u64,
    instance: String,
    state_path: PathBuf,
    body: String,
    row: Vec<SqlValue>,
    insert: ExactRowMutation,
}
pub(crate) fn plan_native_activation(
    roster: ActivationRoster,
    record: &Record,
    task: &Task,
) -> Result<NativeActivationPlan> {
    ensure!(
        record.kind == RecordKind::Workflow && record.version == 0 && record.scope == task.scope(),
        "native activation requires original Workflow"
    );
    ensure!(
        task.id == roster.task()
            && task.executor == roster.executor()
            && task.reviewers == roster.reviewers(),
        "native activation original roster binding differs"
    );
    let owner = roster.owner();
    let epoch = owner.epoch();
    ensure!(epoch > 0, "native activation epoch absent");
    let instance = owner.instance_id().to_owned();
    let state_path = owner.state_path().to_path_buf();
    let contract = NativeContract {
        workflow_id: record.id,
        project_id: task.project_id,
        goal_id: task.goal_id,
        task_id: task.id,
        owner_epoch: epoch,
        origin: roster.installation(),
        profile_digest: roster.digest().to_owned(),
        contract_state: "composed".into(),
        version: 1,
        members: roster.members().to_vec(),
    };
    let body = String::from_utf8(canonical::encode(&serde_json::to_value(&contract)?, 4096)?)?;
    super::Body::<NativeContract>::decode(body.clone(), 4096)?;
    let row = vec![
        SqlValue::Text(record.id.to_string()),
        SqlValue::Text(task.project_id.to_string()),
        SqlValue::Text(task.goal_id.to_string()),
        SqlValue::Text(task.id.to_string()),
        SqlValue::Integer(i64::try_from(epoch)?),
        SqlValue::Text(roster.installation().to_string()),
        SqlValue::Text(roster.digest().to_owned()),
        SqlValue::Text("composed".into()),
        SqlValue::Integer(1),
        SqlValue::Text(body.clone()),
    ];
    let insert = ExactRowMutation::new(
        "workflow_native_contracts",
        "INSERT",
        None,
        Some(row.clone()),
    )?;
    Ok(NativeActivationPlan {
        roster,
        workflow: record.id,
        scope: task.scope(),
        epoch,
        instance,
        state_path,
        body,
        row,
        insert,
    })
}
pub(crate) enum ActivationCommit {
    Published,
    Deferred,
}
impl ActivationCommit {
    pub(in crate::state) fn require_published(self) -> Result<()> {
        ensure!(
            matches!(self, Self::Published),
            "nonactivation publication deferred"
        );
        Ok(())
    }
}
impl NativeActivationPlan {
    pub(crate) fn roster(&self) -> &ActivationRoster {
        &self.roster
    }
    pub(in crate::state) fn matches_input(&self, record: &Record, task: &Task) -> bool {
        self.workflow == record.id
            && self.scope == record.scope
            && self.scope == task.scope()
            && self.roster.task() == task.id
    }
    pub(in crate::state) fn validate_result(&self, tx: &Transaction<'_>) -> Result<()> {
        let exact:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM workflow_native_contracts WHERE workflow_id=?1 AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND owner_epoch=?5 AND origin=?6 AND profile_digest=?7 AND contract_state=?8 AND version=?9 AND body=?10)",params_from_iter(self.row.iter()),|r|r.get(0))?;
        ensure!(exact, "original native activation contract differs");
        Ok(())
    }
    pub(in crate::state) fn validate_rollback(&self, tx: &Transaction<'_>) -> Result<()> {
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM workflow_native_contracts WHERE workflow_id=?1)",
            [self.workflow.to_string()],
            |r| r.get(0),
        )?;
        ensure!(!exists, "native activation retains committed contract");
        Ok(())
    }
    pub(in crate::state) fn write_tx(
        &self,
        tx: &Transaction<'_>,
        permits: &PrivatePermitManager,
        record: &[SqlValue],
    ) -> Result<()> {
        ensure!(
            self.state_path
                .to_str()
                .is_some_and(|p| tx.path() == Some(p)),
            "native activation state path differs"
        );
        let current:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM runtime_epoch WHERE singleton=1 AND instance_id=?1 AND epoch=?2)",params![self.instance,self.epoch],|r|r.get(0))?;
        ensure!(current, "native activation epoch differs");
        let exact:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE id=?1 AND kind=?2 AND project_id=?3 AND goal_id IS ?4 AND task_id IS ?5 AND version=?6 AND body=?7)",params_from_iter(record.iter()),|r|r.get(0))?;
        ensure!(
            exact,
            "native activation original Workflow postimage differs"
        );
        self.validate_rollback(tx)?;
        permits.with_exact_permit(vec![self.insert.copy_for_transaction()?],||{
            ensure!(tx.execute("INSERT INTO workflow_native_contracts(workflow_id,project_id,goal_id,task_id,owner_epoch,origin,profile_digest,contract_state,version,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params_from_iter(self.row.iter()))?==1,"native activation insert absent");
            permits.ensure_consumed()
        })
    }
}
