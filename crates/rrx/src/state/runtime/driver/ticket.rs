//! A retained worker's exact coherent read, not a row-derived live credential.
use super::*;
use crate::{
    execution::RuntimeOwner, runtime::driver::DriverAssociation, state::managed_binding::ScopePlan,
};

/// Deliberately non-Clone and non-Deserialize. Only an actual live worker's
/// private association can mint this ticket; scalar IDs/body copies cannot.
pub(crate) struct DriverReadTicket {
    pub(super) owner: Arc<RuntimeOwner>,
    pub(super) association: DriverAssociation,
    pub(super) scope: ScopePlan,
    pub(super) row: Row,
    pub(super) body: String,
    pub(super) preparation: Option<(crate::execution::ExecutionUnit, String)>,
    pub(super) source: Option<(Uuid, u64, String)>,
    pub(super) prerequisites: super::claim::PrerequisiteRows,
    pub(super) namespace: Option<super::NamespaceSnapshot>,
}

pub(crate) fn read_driver_ticket(
    owner: Arc<RuntimeOwner>,
    association: DriverAssociation,
) -> Result<DriverReadTicket> {
    ensure!(
        association.owner_matches(&owner),
        "Driver association belongs to another owner"
    );
    crate::state::managed_binding::snapshot(&owner, |tx| {
        let (id, epoch, version, body) = association.binding()?;
        ensure!(
            association.validates(id, epoch, version, &body),
            "actual retained Driver is not current"
        );
        ensure!(body.len() <= 128 * 1024, "Driver metadata exceeds bound");
        let row: Row = decode(body.clone())?;
        let task = association.task();
        ensure!(
            row.id == id
                && row.epoch == epoch
                && epoch == owner.epoch()
                && row.version == version
                && row.state == "driving"
                && row.marker.is_none()
                && row.pins.scope.task_id == Some(task),
            "Driver cached/indexed identity differs"
        );
        let scope = crate::state::managed_binding::read_scope(tx, &owner, &row.pins.scope)?;
        let exact: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM task_drivers WHERE task_id=?1 AND goal_id=?2 AND project_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='driving' AND body=?7)",
            params![task.to_string(), scope.task().goal_id.to_string(), scope.task().project_id.to_string(), id.to_string(), epoch, version, body],
            |r| r.get(0),
        )?;
        ensure!(exact, "Driver row differs from actual worker binding");
        let pins = super::snapshot(tx, task)?;
        ensure!(
            serde_json::to_value(&pins)? == serde_json::to_value(&row.pins)?,
            "Driver original full authority changed"
        );
        let preparation = preparation_anchor(tx, scope.task(), pins.generation)?;
        let source = crate::state::execution::source_recovery::driver_anchor(tx, task)?;
        let prerequisites = super::claim::read_prerequisites(tx, scope.task())?;
        let namespace = if preparation.is_none() && !scope.has_input_history() {
            Some(super::NamespaceSnapshot::read(tx, scope.task().project_id)?)
        } else {
            None
        };
        ensure!(
            association.validates(id, epoch, version, &body),
            "Driver revoked during coherent planning"
        );
        Ok(DriverReadTicket {
            owner: owner.clone(),
            association,
            scope,
            row,
            body,
            preparation,
            source,
            prerequisites,
            namespace,
        })
    })
}

impl DriverReadTicket {
    /// Collision evidence only, added under the same original ticket before
    /// the writer mutex. It cannot refresh governing/input/Driver authority.
    pub(crate) fn with_initial_namespace(mut self) -> Result<Self> {
        ensure!(
            !self.scope.has_input_history(),
            "initial input already exists"
        );
        self.namespace = Some(crate::state::managed_binding::snapshot(
            &self.owner,
            |tx| {
                self.validate_current_tx(tx)?;
                super::NamespaceSnapshot::read(tx, self.task().project_id)
            },
        )?);
        Ok(self)
    }
    /// Finite collision evidence under the unchanged original ticket; this
    /// does not refresh Task, input, governing or Driver authority.
    pub(crate) fn with_gate_namespace(mut self) -> Result<Self> {
        self.namespace = Some(crate::state::managed_binding::snapshot(
            &self.owner,
            |tx| {
                self.validate_current_tx(tx)?;
                super::NamespaceSnapshot::read(tx, self.task().project_id)
            },
        )?);
        Ok(self)
    }
    pub(crate) fn matches_input_view(
        &self,
        task: &Task,
        record: &Record,
        context: &ContextVersion,
    ) -> Result<()> {
        let (w, c) = self.scope.workflow_input()?;
        ensure!(
            serde_json::to_value(task)? == serde_json::to_value(self.task())?
                && serde_json::to_value(record)? == serde_json::to_value(w)?
                && context == c,
            "Driver Engine input view differs from captured original"
        );
        Ok(())
    }
    pub(crate) fn task(&self) -> &Task {
        self.scope.task()
    }
    pub(crate) fn preparation_matches(&self, unit: &crate::execution::ExecutionUnit) -> Result<()> {
        let (original, body) = self
            .preparation
            .as_ref()
            .context("Driver has no preparation Unit")?;
        ensure!(
            original.id == unit.id && *body == serde_json::to_string(unit)?,
            "helper Driver/Unit snapshot differs"
        );
        Ok(())
    }

    /// Exact byte/index CAS only. Complete bodies/hashes were decoded outside
    /// SharedStore. No current-row recapture or authority advance occurs here.
    pub(crate) fn validate_current_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        self.scope.validate_current(tx)?;
        self.validate_ancillary_tx(tx)
    }
    pub(super) fn validate_ancillary_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        ensure!(
            self.association.owner_matches(&self.owner)
                && self.association.validates(
                    self.row.id,
                    self.row.epoch,
                    self.row.version,
                    &self.body
                ),
            "retained Driver ticket revoked or replaced"
        );
        let task = self.task();
        let exact: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM task_drivers WHERE task_id=?1 AND goal_id=?2 AND project_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='driving' AND body=?7)",
            params![task.id.to_string(), task.goal_id.to_string(), task.project_id.to_string(), self.row.id.to_string(), self.row.epoch, self.row.version, self.body],
            |r| r.get(0),
        )?;
        ensure!(exact, "Driver ticket durable row changed");
        self.validate_selection_tx(tx)?;
        self.validate_source_tx(tx)
    }
    pub(super) fn validate_selection_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        super::claim::validate_prerequisite_rows(tx, self.task(), &self.prerequisites)?;
        let task = self.task();
        let generation: Option<(u64, String)> = tx.query_row(
            "SELECT generation,active_unit FROM task_execution WHERE task_id=?1 AND goal_id=?2 AND project_id=?3",
            params![task.id.to_string(), task.goal_id.to_string(), task.project_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).optional()?;
        let expected = self
            .preparation
            .as_ref()
            .map(|(u, _)| (u.generation, u.id.to_string()));
        ensure!(
            generation == expected,
            "Driver original generation/active Unit changed"
        );
        if let Some((unit, body)) = &self.preparation {
            let exact: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM execution_units WHERE id=?1 AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND kind='executor' AND generation=?5 AND owner_epoch=?6 AND version=?7 AND native_effects_open=?8 AND result_finalization_open=?9 AND worktree=?10 AND branch IS ?11 AND body=?12)",
                params![unit.id.to_string(), task.project_id.to_string(), task.goal_id.to_string(), task.id.to_string(), unit.generation, unit.owner_epoch, unit.version, unit.native_effects_open, unit.result_finalization_open, unit.worktree.to_str().context("Driver Unit path is not UTF-8")?, unit.branch, body],
                |r| r.get(0),
            )?;
            ensure!(exact, "Driver original preparation Unit changed");
        }
        Ok(())
    }
    pub(super) fn validate_source_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        let task = self.task();
        let source: Option<(String, u64, Option<String>)> = tx.query_row(
            "SELECT id,version,CASE WHEN length(CAST(body AS BLOB))<=131072 THEN body END FROM source_recoveries WHERE task_id=?1 AND goal_id=?2 AND project_id=?3 AND owner_epoch=?4 AND state='installed'",
            params![task.id.to_string(), task.goal_id.to_string(), task.project_id.to_string(), self.row.epoch],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        ).optional()?;
        let expected = self
            .source
            .as_ref()
            .map(|(id, version, body)| (id.to_string(), *version, Some(body.clone())));
        ensure!(
            source == expected,
            "Driver original Source7 binding changed"
        );
        // Absence is checked independently of installed-state filtering: an
        // unexpected Preparing/invalid or foreign indexed row cannot disappear.
        let count: u64 = tx.query_row(
            "SELECT count(*) FROM source_recoveries WHERE task_id=?1",
            [task.id.to_string()],
            |r| r.get(0),
        )?;
        ensure!(
            count == u64::from(self.source.is_some()),
            "Driver Source7 inventory changed"
        );
        Ok(())
    }
}

impl Store {
    pub(crate) fn validate_driver_read(&mut self, ticket: &DriverReadTicket) -> Result<()> {
        let tx = self.connection.transaction()?;
        ticket.validate_current_tx(&tx)?;
        tx.commit()?;
        Ok(())
    }
}
