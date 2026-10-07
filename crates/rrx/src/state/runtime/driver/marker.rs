//! Final Driver/Source freeze is planned only from the actual allocation-derived
//! Root marker. These plans do not publish a marker, Native owner or input.
use super::*;
use crate::state::{
    execution::source_recovery::{SourceMarkerAdvance, plan_source_marker_advance},
    managed_binding::{ExactRowMutation, ManagedMarkerPlan},
};
use rusqlite::types::Value as SqlValue;

pub(crate) struct DriverMarkerAdvance {
    ticket: Arc<DriverReadTicket>,
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
        self: &Arc<Self>,
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
            ticket: self.clone(),
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
    /// Original-object identity only, no currency or Native grant. Root's
    /// accepted Source origin must link to the SAME retained producer ticket.
    pub(crate) fn matches_source_ticket(&self, ticket: &Arc<DriverReadTicket>) -> bool {
        Arc::ptr_eq(&self.ticket, ticket)
    }
    /// Borrow only this OriginalMarker-owned advance. This checks actual Driver
    /// liveness and immutable post Driver/Source rows; Root separately validates
    /// the original-derived Workflow successor, current Unit/pair and lifecycle
    /// in the SAME transaction. Factual Workflow links never refresh these pins.
    pub(crate) fn validate_live_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        ensure!(
            self.ticket.association.owner_matches(&self.ticket.owner)
                && self.ticket.association.validates(
                    self.next.id,
                    self.next.epoch,
                    self.next.version,
                    &self.body
                ),
            "original marker Driver no longer owns a live worker"
        );
        let current: bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM task_drivers d JOIN runtime_epoch e ON e.singleton=1 AND e.epoch=d.owner_epoch WHERE d.task_id=?1 AND d.project_id=?2 AND d.goal_id=?3 AND d.id=?4 AND d.owner_epoch=?5 AND d.version=?6 AND d.state='driving' AND d.body=?7 AND e.instance_id=?8)", params![self.task.id.to_string(),self.task.project_id.to_string(),self.task.goal_id.to_string(),self.next.id.to_string(),self.next.epoch,self.next.version,self.body,self.ticket.owner.instance_id()],|r|r.get(0))?;
        ensure!(current, "original marker Driver post binding changed");
        if let Some(source) = &self.source {
            source.validate_result_tx(tx)?;
        } else {
            self.ticket.validate_source_tx(tx)?;
        }
        Ok(())
    }
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

/// The successful closure's Driver re-anchor: the SAME frozen post-marker row
/// to a marker-free row whose pins are exactly what the generic snapshot
/// recomputes from the planned postimages. Built only from the SAME
/// `DriverMarkerAdvance`; not Clone/Deserialize and never a credential.
pub(crate) struct DriverClosureAdvance {
    ticket: Arc<DriverReadTicket>,
    task: TaskId,
    old: Row,
    old_body: String,
    next: Row,
    body: String,
}
impl DriverMarkerAdvance {
    /// `unit_after` is the decoded closure postimage as the generic reader
    /// sees it (its `cleanup` field carries the current read overlay).
    pub(crate) fn plan_success_closure(
        &self,
        task_after: &Task,
        workflow_after: &Record,
        context_after: &ContextVersion,
        unit_after: &crate::execution::ExecutionUnit,
    ) -> Result<DriverClosureAdvance> {
        ensure!(
            self.source.is_none() && self.next.pins.source.is_none(),
            "Source7-anchored success closure unsupported"
        );
        ensure!(
            self.next.marker.is_some()
                && task_after.id == self.task.id
                && workflow_after.id == self.workflow.id
                && context_after.scope == task_after.scope()
                && context_after.version == task_after.context_version
                && unit_after.scope == task_after.scope()
                && self
                    .next
                    .pins
                    .preparation
                    .as_ref()
                    .is_some_and(|p| p.unit == unit_after.id),
            "success closure Driver postimages differ from the marker advance"
        );
        let mut next = self.next.clone();
        next.version = next
            .version
            .checked_add(1)
            .filter(|v| *v <= i64::MAX as u64)
            .context("Driver version exhausted")?;
        next.marker = None;
        next.pins.task = pin(task_after.version, task_after)?;
        next.pins.workflow = Some((
            workflow_after.id,
            pin(workflow_after.version, workflow_after)?,
        ));
        next.pins.context = Some(pin(context_after.version, context_after)?);
        next.pins.preparation = Some(PreparationPin {
            unit: unit_after.id,
            version: unit_after.version,
            digest: hash(unit_after)?,
        });
        next.pins.source = None;
        let body = serde_json::to_string(&next)?;
        ensure!(
            body.len() <= 128 * 1024,
            "closure Driver exceeds metadata bound"
        );
        Ok(DriverClosureAdvance {
            ticket: self.ticket.clone(),
            task: self.task.id,
            old: self.next.clone(),
            old_body: self.body.clone(),
            next,
            body,
        })
    }
}
impl DriverClosureAdvance {
    pub(in crate::state) fn exact_mutations(&self) -> Result<Vec<ExactRowMutation>> {
        Ok(vec![ExactRowMutation::new(
            "task_drivers",
            "UPDATE",
            Some(image(&self.old, &self.old_body)?),
            Some(image(&self.next, &self.body)?),
        )?])
    }
    /// The SAME frozen post-marker row is current and its worker is live.
    pub(crate) fn validate_current_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        self.validate_row_tx(tx, &self.old, &self.old_body)
    }
    /// The planned marker-free row is current (postimage branch).
    pub(crate) fn validate_closed_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        self.validate_row_tx(tx, &self.next, &self.body)
    }
    fn validate_row_tx(&self, tx: &Transaction<'_>, row: &Row, body: &str) -> Result<()> {
        let exact: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM task_drivers d JOIN runtime_epoch e ON e.singleton=1 AND e.epoch=d.owner_epoch WHERE d.task_id=?1 AND d.id=?2 AND d.owner_epoch=?3 AND d.version=?4 AND d.state='driving' AND d.body=?5 AND e.instance_id=?6)", params![self.task.to_string(),row.id.to_string(),row.epoch,row.version,body,self.ticket.owner.instance_id()],|r|r.get(0))?;
        ensure!(exact, "success closure Driver row differs");
        Ok(())
    }
    pub(in crate::state) fn write_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        ensure!(tx.execute("UPDATE task_drivers SET version=?1,body=?2 WHERE task_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='driving' AND body=?7",params![self.next.version,self.body,self.task.to_string(),self.next.id.to_string(),self.next.epoch,self.old.version,self.old_body])?==1,"success closure Driver CAS changed");
        Ok(())
    }
    /// The SAME association owns the frozen row; nongrant.
    pub(crate) fn association_live(&self) -> bool {
        self.ticket.association.owner_matches(&self.ticket.owner)
            && self.ticket.association.validates(
                self.old.id,
                self.old.epoch,
                self.old.version,
                &self.old_body,
            )
    }
    /// The SAME live association, whose cache holds the frozen row before
    /// publication or the closed row after it; an ended or revoked worker is
    /// never current. Nongrant.
    pub(crate) fn association_current(&self) -> bool {
        self.ticket.association.owner_matches(&self.ticket.owner)
            && (self.ticket.association.validates(
                self.old.id,
                self.old.epoch,
                self.old.version,
                &self.old_body,
            ) || self.ticket.association.validates(
                self.next.id,
                self.next.epoch,
                self.next.version,
                &self.body,
            ))
    }
    pub(crate) fn version_after(&self) -> u64 {
        self.next.version
    }
}
impl Store {
    /// Known-commit publication of the closure's marker-free Driver row; the
    /// SAME association's idempotent `publish_exact`. Caller holds the
    /// control admission (a `SuccessAdmission`).
    pub(crate) fn publish_driver_closure(&mut self, plan: &DriverClosureAdvance) -> Result<()> {
        ensure!(
            self.connection.is_autocommit(),
            "Driver publication precedes commit"
        );
        let tx = self.connection.transaction()?;
        plan.validate_closed_tx(&tx)?;
        tx.commit()?;
        let publication = DriverPublication {
            task: plan.task,
            id: plan.next.id,
            epoch: plan.next.epoch,
            before_version: plan.old.version,
            before_body: plan.old_body.clone(),
            after_version: plan.next.version,
            after_body: plan.body.clone(),
        };
        plan.ticket.association.publish_exact(&publication)
    }
}
