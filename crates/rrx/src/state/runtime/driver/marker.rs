//! Final Driver/Source freeze is planned only from the actual allocation-derived
//! Root marker. These plans do not publish a marker, Native owner or input.
use super::*;
use crate::state::{
    execution::source_recovery::{SourceMarkerAdvance, plan_source_marker_advance},
    managed_binding::{ExactRowMutation, ManagedMarkerPlan},
};
use rusqlite::types::Value as SqlValue;

pub(crate) struct DriverMarkerAdvance {
    ticket: DriverReadTicket,
    next: Row,
    body: String,
    task: Task,
    task_body: String,
    workflow: Record,
    workflow_body: String,
    source: Option<SourceMarkerAdvance>,
}
/// Created only by checking a completed planned mutation on the same Store
/// after commit. Not Clone/Deserialize and never an input/Native credential.
pub(crate) struct DriverPublication {
    pub(super) task: TaskId,
    pub(super) id: Uuid,
    pub(super) epoch: u64,
    pub(super) before_version: u64,
    pub(super) before_body: String,
    pub(super) after_version: u64,
    pub(super) after_body: String,
}
impl DriverPublication {
    pub(crate) fn before(&self) -> (TaskId, Uuid, u64, u64, &str) {
        (
            self.task,
            self.id,
            self.epoch,
            self.before_version,
            &self.before_body,
        )
    }
    pub(crate) fn after(&self) -> (u64, &str) {
        (self.after_version, &self.after_body)
    }
}
pub(super) fn image(row: &Row, body: &str) -> Result<Vec<SqlValue>> {
    let scope = &row.pins.scope;
    Ok(vec![
        scope
            .task_id
            .context("Driver Task missing")?
            .to_string()
            .into(),
        scope
            .goal_id
            .context("Driver Goal missing")?
            .to_string()
            .into(),
        scope.project_id.to_string().into(),
        row.id.to_string().into(),
        i64::try_from(row.epoch)?.into(),
        i64::try_from(row.version)?.into(),
        row.state.clone().into(),
        body.to_owned().into(),
    ])
}
impl DriverReadTicket {
    pub(crate) fn plan_marker_advance(
        self,
        marker: &ManagedMarkerPlan,
    ) -> Result<DriverMarkerAdvance> {
        self.scope.matches_marker(marker)?;
        ensure!(self.row.marker.is_none(), "Driver already marker-bound");
        let unit = self
            .preparation
            .as_ref()
            .context("marker Driver needs actual preparation")?;
        ensure!(
            unit.1 == marker.unit().1 && unit.0.id == marker.unit().0.id,
            "Driver/marker preparation differs"
        );
        let source = self
            .source
            .as_ref()
            .map(|original| plan_source_marker_advance(original, marker))
            .transpose()?;
        let (task, task_body) = marker.task_after();
        let (workflow, workflow_body) = marker.workflow_after();
        let mut next = self.row.clone();
        next.version = next
            .version
            .checked_add(1)
            .filter(|v| *v <= i64::MAX as u64)
            .context("Driver version exhausted")?;
        next.pins.task = pin(task.version, task)?;
        next.pins.workflow = Some((workflow.id, pin(workflow.version, workflow)?));
        if let Some(source) = &source {
            let (id, version, digest) = source.resulting_pin();
            next.pins.source = Some(SourcePin {
                id,
                version,
                digest: digest.into(),
            });
        }
        next.marker = Some(MarkerAnchor {
            operation: marker.operation(),
            digest: marker.marker_digest().into(),
        });
        let body = serde_json::to_string(&next)?;
        ensure!(
            body.len() <= 128 * 1024,
            "marker Driver exceeds metadata bound"
        );
        Ok(DriverMarkerAdvance {
            ticket: self,
            next,
            body,
            task: task.clone(),
            task_body: task_body.into(),
            workflow: workflow.clone(),
            workflow_body: workflow_body.into(),
            source,
        })
    }
}
impl DriverMarkerAdvance {
    pub(in crate::state) fn exact_mutations(&self) -> Result<Vec<ExactRowMutation>> {
        let mut mutations = vec![ExactRowMutation::new(
            "task_drivers",
            "UPDATE",
            Some(image(&self.ticket.row, &self.ticket.body)?),
            Some(image(&self.next, &self.body)?),
        )?];
        if let Some(source) = &self.source {
            mutations.push(source.mutation()?);
        }
        Ok(mutations)
    }
    /// Required BEFORE Root's Task/Workflow marker writes, in their same TX.
    pub(crate) fn validate_current_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        self.ticket.validate_current_tx(tx)
    }
    /// Required AFTER the fixed Task/W writes. Root supplies ONE exact permit
    /// batch covering every planned row; no nested manager or generic refresh.
    pub(crate) fn freeze_marker_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        self.ticket.validate_ancillary_tx(tx)?;
        self.ticket.scope.validate_projection(
            tx,
            &self.task,
            &self.task_body,
            Some((&self.workflow, &self.workflow_body)),
        )?;
        if let Some(source) = &self.source {
            source.write_tx(tx)?;
        }
        ensure!(tx.execute("UPDATE task_drivers SET version=?1,body=?2 WHERE task_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='driving' AND body=?7",params![self.next.version,self.body,self.task.id.to_string(),self.next.id.to_string(),self.next.epoch,self.ticket.row.version,self.ticket.body])?==1,"Driver marker freeze CAS changed");
        Ok(())
    }
}
impl Store {
    /// Known-commit publication while SharedStore remains excluded. A failed
    /// check leaves this exact owned plan with its caller, never adopts newer rows.
    pub(crate) fn publish_driver_marker(&mut self, plan: &DriverMarkerAdvance) -> Result<()> {
        ensure!(
            self.connection.is_autocommit(),
            "Driver publication precedes commit"
        );
        let tx = self.connection.transaction()?;
        plan.ticket.scope.validate_projection(
            &tx,
            &plan.task,
            &plan.task_body,
            Some((&plan.workflow, &plan.workflow_body)),
        )?;
        plan.ticket.validate_selection_tx(&tx)?;
        if let Some(source) = &plan.source {
            source.validate_result_tx(&tx)?;
        } else {
            plan.ticket.validate_source_tx(&tx)?;
        }
        let exact:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM task_drivers WHERE task_id=?1 AND id=?2 AND owner_epoch=?3 AND version=?4 AND state='driving' AND body=?5)",params![plan.task.id.to_string(),plan.next.id.to_string(),plan.next.epoch,plan.next.version,plan.body],|r|r.get(0))?;
        ensure!(exact, "planned committed Driver row differs");
        tx.commit()?;
        let publication = DriverPublication {
            task: plan.task.id,
            id: plan.next.id,
            epoch: plan.next.epoch,
            before_version: plan.ticket.row.version,
            before_body: plan.ticket.body.clone(),
            after_version: plan.next.version,
            after_body: plan.body.clone(),
        };
        plan.ticket.association.publish_exact(&publication)
    }
}
