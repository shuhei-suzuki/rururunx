//! Dispatcher join facts do not grant Native work or release an operation.
use super::*;
/// Produced only after the matching actual join observation's DB transaction.
/// It is not a Native/input/settlement certificate.
pub(crate) struct DriverExitPublication {
    identity: (TaskId, Uuid, u64),
    initial_closed: bool,
}
impl DriverExitPublication {
    pub(crate) fn identity(&self) -> (TaskId, Uuid, u64) {
        self.identity
    }
    pub(crate) fn initial_closed(&self) -> bool {
        self.initial_closed
    }
}
impl Store {
    pub(crate) fn record_driver_exit(
        &mut self,
        exit: &crate::runtime::driver::DriverExit,
    ) -> Result<DriverExitPublication> {
        let (task, id, epoch) = exit.identity();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (project,goal,body):(String,String,String)=tx.query_row(
            "SELECT project_id,goal_id,CASE WHEN length(CAST(body AS BLOB))<=131072 THEN body END FROM task_drivers WHERE task_id=?1 AND id=?2 AND owner_epoch=?3",
            params![task.to_string(),id.to_string(),epoch],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)),
        )?;
        let row: Row = decode(body)?;
        ensure!(
            row.id == id
                && row.epoch == epoch
                && row.pins.scope.task_id == Some(task)
                && row.pins.scope.project_id.to_string() == project
                && row.pins.scope.goal_id.map(|v| v.to_string()).as_deref() == Some(goal.as_str()),
            "Driver exit indexed identity differs"
        );
        let initial_closed = if let Some(plan) = exit.never_activated() {
            ensure!(
                plan.identity().0 == id && plan.identity().1 == epoch && plan.task() == task,
                "never-activated actual claim identity differs"
            );
            plan.close_unactivated_tx(&tx, &self.binding_permits)?;
            true
        } else {
            false
        };
        // Exact dispatcher identity makes retry idempotent. Only the sealed
        // never-activated route can invalidate its own original initial claim;
        // no Unit/Session or original P/G/T/Workflow/source pin is changed.
        let prior:bool=tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM audit WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind='rrx.private.runtime.driver_exited' AND json_extract(data,'$.driver')=?4 AND json_extract(data,'$.epoch')=?5)",
            params![project,goal,task.to_string(),id.to_string(),epoch],|r|r.get(0),
        )?;
        if !prior {
            append_event(
                &tx,
                &row.pins.scope,
                "rrx.private.runtime.driver_exited",
                json!({"driver":id,"epoch":epoch,"dispatcher_outcome":exit.label(),"native_settlement":false}),
            )?;
        }
        tx.commit()?;
        Ok(DriverExitPublication {
            identity: (task, id, epoch),
            initial_closed,
        })
    }
}
