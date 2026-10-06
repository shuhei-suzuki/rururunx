//! Prescribed preparatory changes belonging to the same live Sources worker.
//! A plan is not a native owner, prepared input or result certificate.
use super::*;
use crate::execution::{
    CleanupOutcome, Disposition, ExecutionUnit, UnitKind, UnitState, WORKFLOW_SOURCE_BOOTSTRAP,
};
use crate::state::managed_binding::ExactRowMutation;

pub(crate) struct DriverPreparationAdvance {
    ticket: DriverReadTicket,
    unit: ExecutionUnit,
    unit_body: String,
    task: Task,
    task_body: String,
    next: Row,
    body: String,
    initial: bool,
    governing: String,
}
impl DriverReadTicket {
    /// Only the actual AttemptManager draft reaches this private entry. The
    /// initial lane is deliberately narrower than replacement/resume admission.
    pub(crate) fn plan_first_preparation(
        self,
        mut unit: ExecutionUnit,
    ) -> Result<DriverPreparationAdvance> {
        let task = self.task();
        ensure!(
            self.preparation.is_none()
                && self.source.is_none()
                && !self.scope.has_input_history()
                && self.row.pins.generation == 0
                && task.worktree.is_none()
                && task.branch.is_none()
                && task.revision.is_none(),
            "Driver first preparation requires pristine original input"
        );
        ensure!(
            unit.scope == task.scope()
                && unit.owner_epoch == self.owner.epoch()
                && unit.kind == UnitKind::Executor
                && unit.generation == 0
                && unit.version == 0
                && unit.phase == WORKFLOW_SOURCE_BOOTSTRAP
                && unit.state == UnitState::Reserved
                && unit.base_sha.is_empty()
                && matches!(unit.provider.as_str(), "claude" | "codex")
                && unit.native_effects_open
                && unit.result_finalization_open
                && unit.work.is_none()
                && unit.cleanup == CleanupOutcome::Unknown
                && unit.disposition == Disposition::Active
                && unit.session_id.is_none()
                && unit.artifact_id.is_none()
                && unit.wait_reason.is_none()
                && unit.capacity_retry_at.is_none()
                && unit.cookie.len() == 36
                && uuid::Uuid::parse_str(&unit.cookie).is_ok()
                && unit.profile_digest.len() == 64
                && unit.profile_digest.bytes().all(|c| c.is_ascii_hexdigit())
                && unit.created_at == unit.updated_at
                && unit.worktree.is_absolute()
                && unit
                    .branch
                    .as_ref()
                    .is_some_and(|b| b == &format!("rrx/{}/{}", task.id, unit.id)),
            "invalid actual first preparation draft"
        );
        unit.version = 1;
        unit.generation = 1;
        let mut next_task = task.clone();
        next_task.worktree = Some(unit.worktree.clone());
        next_task.branch = unit.branch.clone();
        bump(&mut next_task.version)?;
        next_task.updated_at = unit.updated_at;
        self.preparation_plan(unit, next_task, true)
    }
    /// No arbitrary transition DTO is accepted: only the two prescribed source
    /// preparation edges before first Workflow/input publication exist here.
    pub(crate) fn plan_preparing(self) -> Result<DriverPreparationAdvance> {
        let (old, _) = self
            .preparation
            .as_ref()
            .context("Driver preparation missing")?;
        ensure!(
            old.state == UnitState::Reserved,
            "Driver Unit is not Reserved"
        );
        let mut unit = old.clone();
        unit.state = UnitState::Preparing;
        bump(&mut unit.version)?;
        unit.updated_at = now_ms();
        let task = self.task().clone();
        self.preparation_plan(unit, task, false)
    }
    pub(crate) fn plan_preparation_base(self, base: &str) -> Result<DriverPreparationAdvance> {
        ensure!(
            crate::execution::valid_oid(base),
            "base must be exact commit OID"
        );
        let (old, _) = self
            .preparation
            .as_ref()
            .context("Driver preparation missing")?;
        ensure!(
            old.state == UnitState::Preparing && old.base_sha.is_empty(),
            "Driver base already bound"
        );
        let mut unit = old.clone();
        unit.base_sha = base.into();
        bump(&mut unit.version)?;
        unit.updated_at = now_ms();
        let task = self.task().clone();
        self.preparation_plan(unit, task, false)
    }
    fn preparation_plan(
        self,
        unit: ExecutionUnit,
        task: Task,
        initial: bool,
    ) -> Result<DriverPreparationAdvance> {
        ensure!(
            self.source.is_none()
                && !self.scope.has_input_history()
                && self.row.marker.is_none()
                && unit.phase == WORKFLOW_SOURCE_BOOTSTRAP
                && unit.session_id.is_none()
                && unit.artifact_id.is_none()
                && unit.work.is_none()
                && unit.native_effects_open
                && unit.result_finalization_open
                && unit.owner_epoch == self.owner.epoch(),
            "Driver preparation is no longer pre-input"
        );
        if let Some((old, _)) = &self.preparation {
            ensure!(
                old.id == unit.id && old.scope == unit.scope && old.generation == unit.generation,
                "Driver preparation identity changed"
            );
        }
        let unit_body = serde_json::to_string(&unit)?;
        let task_body = serde_json::to_string(&task)?;
        ensure!(
            unit_body.len() <= 16 * 1024 && task_body.len() <= 1024 * 1024,
            "preparation body exceeds bound"
        );
        let mut next = self.row.clone();
        bump(&mut next.version)?;
        next.pins.task = pin(task.version, &task)?;
        next.pins.generation = unit.generation;
        next.pins.preparation = Some(PreparationPin {
            unit: unit.id,
            version: unit.version,
            digest: hash(&unit)?,
        });
        let body = serde_json::to_string(&next)?;
        ensure!(body.len() <= 128 * 1024, "preparation Driver exceeds bound");
        let (project, goal) = self.scope.governing_owners();
        let governing = crate::state::execution::governing_digest(project, goal)?;
        Ok(DriverPreparationAdvance {
            ticket: self,
            unit,
            unit_body,
            task,
            task_body,
            next,
            body,
            initial,
            governing,
        })
    }
}
impl DriverPreparationAdvance {
    fn validate_result(&self, tx: &Transaction<'_>) -> Result<()> {
        self.ticket
            .scope
            .validate_projection(tx, &self.task, &self.task_body, None)?;
        super::claim::validate_prerequisite_rows(tx, &self.task, &self.ticket.prerequisites)?;
        self.ticket.validate_source_tx(tx)?;
        let exact: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM execution_units u JOIN task_execution t ON t.active_unit=u.id AND t.task_id=u.task_id AND t.project_id=u.project_id AND t.goal_id=u.goal_id AND t.generation=u.generation WHERE u.id=?1 AND u.project_id=?2 AND u.goal_id=?3 AND u.task_id=?4 AND u.kind='executor' AND u.generation=?5 AND u.owner_epoch=?6 AND u.version=?7 AND u.native_effects_open=1 AND u.result_finalization_open=1 AND u.worktree=?8 AND u.branch IS ?9 AND u.body=?10)",params![self.unit.id.to_string(),self.task.project_id.to_string(),self.task.goal_id.to_string(),self.task.id.to_string(),self.unit.generation,self.unit.owner_epoch,self.unit.version,self.unit.worktree.to_str().context("Unit path not UTF-8")?,self.unit.branch,self.unit_body],|r|r.get(0))?;
        ensure!(exact, "planned preparation Unit/generation differs");
        let context: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM execution_context WHERE unit_id=?1 AND project_version=?2 AND goal_version=?3 AND governing_digest=?4)",params![self.unit.id.to_string(),self.next.pins.project.version,self.next.pins.goal.version,self.governing],|r|r.get(0))?;
        ensure!(context, "planned preparation governing versions differ");
        Ok(())
    }
}
impl Store {
    pub(crate) fn apply_driver_preparation(
        &mut self,
        plan: &DriverPreparationAdvance,
    ) -> Result<ExecutionUnit> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        plan.ticket.validate_current_tx(&tx)?;
        // Reuse all current lifecycle/accepted-definition/activity checks, while
        // the old genuine association and full pins are still current.
        super::validate(&tx, plan.task.id)?;
        if plan.initial {
            let history: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM execution_units WHERE task_id=?1) OR EXISTS(SELECT 1 FROM task_execution WHERE task_id=?1) OR EXISTS(SELECT 1 FROM records WHERE task_id=?1 AND kind='session') OR EXISTS(SELECT 1 FROM managed_phase_operations WHERE task_id=?1)",[plan.task.id.to_string()],|r|r.get(0))?;
            ensure!(!history, "first preparation has retained history");
            let project: Project = read_tx(&tx, "projects", &plan.task.project_id.to_string())?
                .context("Project missing")?;
            ensure!(
                plan.unit.worktree.parent() == Some(project.worktree_root.as_path())
                    && plan.unit.branch.as_ref() != Some(&project.base_branch),
                "invalid preparation namespace"
            );
            crate::state::execution::insert_unit(&tx, &plan.unit)?;
            tx.execute("INSERT INTO execution_context(unit_id,project_version,goal_version,governing_digest) VALUES(?1,?2,?3,?4)",params![plan.unit.id.to_string(),plan.next.pins.project.version,plan.next.pins.goal.version,plan.governing])?;
            tx.execute("INSERT INTO task_execution(task_id,project_id,goal_id,generation,active_unit) VALUES(?1,?2,?3,1,?4)",params![plan.task.id.to_string(),plan.task.project_id.to_string(),plan.task.goal_id.to_string(),plan.unit.id.to_string()])?;
            let mut input = plan.task.clone();
            input.version = plan.ticket.task().version;
            let written = put_task_tx_at(&tx, &input, plan.task.updated_at)?;
            ensure!(
                serde_json::to_string(&written)? == plan.task_body,
                "preparation Task write differs from prescribed plan"
            );
        } else {
            let (old, body) = plan
                .ticket
                .preparation
                .as_ref()
                .context("original preparation missing")?;
            ensure!(tx.execute("UPDATE execution_units SET version=?1,body=?2 WHERE id=?3 AND version=?4 AND body=?5",params![plan.unit.version,plan.unit_body,old.id.to_string(),old.version,body])? == 1,"preparation Unit CAS changed");
        }
        plan.validate_result(&tx)?;
        let mutation = ExactRowMutation::new(
            "task_drivers",
            "UPDATE",
            Some(super::marker::image(&plan.ticket.row, &plan.ticket.body)?),
            Some(super::marker::image(&plan.next, &plan.body)?),
        )?;
        self.binding_permits.with_exact_permit(vec![mutation],||{
            ensure!(tx.execute("UPDATE task_drivers SET version=?1,body=?2 WHERE task_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='driving' AND body=?7",params![plan.next.version,plan.body,plan.task.id.to_string(),plan.next.id.to_string(),plan.next.epoch,plan.ticket.row.version,plan.ticket.body])?==1,"preparation Driver CAS changed");
            self.binding_permits.ensure_consumed()
        })?;
        append_event(
            &tx,
            &plan.unit.scope,
            "rrx.private.runtime.driver_preparation_advanced",
            json!({"driver":plan.next.id,"version":plan.next.version,"unit":plan.unit.id,"unit_version":plan.unit.version,"initial":plan.initial}),
        )?;
        tx.commit()?;
        self.publish_driver_preparation(plan)?;
        Ok(plan.unit.clone())
    }
    /// Same owned plan reconciliation, never current-row authority recapture.
    pub(crate) fn publish_driver_preparation(
        &mut self,
        plan: &DriverPreparationAdvance,
    ) -> Result<()> {
        ensure!(
            self.connection.is_autocommit(),
            "preparation publication precedes commit"
        );
        let tx = self.connection.transaction()?;
        plan.validate_result(&tx)?;
        let exact: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM task_drivers WHERE task_id=?1 AND goal_id=?2 AND project_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='driving' AND body=?7)",params![plan.task.id.to_string(),plan.task.goal_id.to_string(),plan.task.project_id.to_string(),plan.next.id.to_string(),plan.next.epoch,plan.next.version,plan.body],|r|r.get(0))?;
        ensure!(exact, "committed preparation Driver differs");
        tx.commit()?;
        plan.ticket.association.publish_exact(&DriverPublication {
            task: plan.task.id,
            id: plan.next.id,
            epoch: plan.next.epoch,
            before_version: plan.ticket.row.version,
            before_body: plan.ticket.body.clone(),
            after_version: plan.next.version,
            after_body: plan.body.clone(),
        })
    }
}
