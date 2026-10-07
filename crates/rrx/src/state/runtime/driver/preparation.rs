//! Prescribed preparatory changes belonging to the same live Sources worker.
//! A plan is not a native owner, prepared input or result certificate.
use super::*;
use crate::execution::{
    CleanupOutcome, Disposition, ExecutionUnit, UnitKind, UnitState, WORKFLOW_SOURCE_BOOTSTRAP,
};
use crate::state::managed_binding::ExactRowMutation;
#[path = "gates.rs"]
mod gates;
pub(crate) use gates::InitialGateEdge;
#[path = "executor.rs"]
mod executor;

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
    input: Option<InitialInput>,
    reconcile_ready: std::sync::atomic::AtomicBool,
}
struct InitialInput {
    activation: Option<crate::state::managed_binding::NativeActivationPlan>,
    records: Option<ExactRowMutation>,
    record_image: Vec<rusqlite::types::Value>,
    gate: Option<gates::GateInput>,
    executor: Option<executor::ExecutorInput>,
    fresh_context: bool,
    frame: crate::execution::workflow_source::InitialInputFrame,
    record_before: Record,
    record: Record,
    record_body: String,
    context: ContextVersion,
    context_body: String,
}
impl DriverReadTicket {
    /// Prescribed FIRST Workflow/Context publication from the real prepared
    /// Sources slot. Retained before any SQL just like other preparation edges.
    pub(crate) fn plan_initial_input(
        self,
        frame: crate::execution::workflow_source::InitialInputFrame,
        unit: ExecutionUnit,
        input: &Task,
        record: &Record,
        context: &ContextVersion,
        activation: crate::state::managed_binding::NativeActivationPlan,
    ) -> Result<Arc<DriverPreparationAdvance>> {
        ensure!(
            !self.scope.has_input_history()
                && self.source.is_none()
                && self.row.marker.is_none()
                && self.namespace.is_some()
                && self.row.pins.generation == unit.generation
                && unit.generation == 1
                && input.context_version == 1
                && context.version == 1
                && record.version == 0
                && record.kind == RecordKind::Workflow
                && record.scope == input.scope()
                && context.scope == input.scope(),
            "Driver initial input is not pristine"
        );
        self.preparation_matches(&unit)?;
        ensure!(
            serde_json::to_value(frame.unit())? == serde_json::to_value(&unit)?,
            "initial frame/Driver selected Unit differs"
        );
        frame.validate(&self.owner)?;
        let (project, goal) = self.scope.governing_owners();
        let governing = crate::state::execution::governing_digest(project, goal)?;
        ensure!(
            frame.governing() == governing,
            "initial governing frame changed"
        );
        let mut expected = self.task().clone();
        expected.workflow = input.workflow;
        expected.context_version = 1;
        expected.revision = Some(unit.base_sha.clone());
        ensure!(
            serde_json::to_value(&expected)? == serde_json::to_value(input)?,
            "initial Task projection changes unrelated authority"
        );
        let w: crate::workflow::WorkflowSnapshot = serde_json::from_value(record.data.clone())?;
        ensure!(
            w.generation == 1
                && w.context_version == 1
                && w.context_fresh
                && w.active.is_none()
                && w.history.is_empty()
                && w.completed.is_empty()
                && w.escalations.is_empty()
                && w.retries.is_empty()
                && w.invalidations.is_empty()
                && w.finalizations.is_empty()
                && w.terminal_decision.is_none()
                && !w.finished
                && w.held_reason.is_none()
                && w.risk == input.risk
                && input.workflow >= self.task().workflow,
            "initial Workflow is not an empty first generation"
        );
        crate::workflow::validate_context(input, record, context)?;
        let mut task = input.clone();
        bump(&mut task.version)?;
        task.updated_at = record.created_at;
        let mut next_record = record.clone();
        bump(&mut next_record.version)?;
        next_record.updated_at = record.created_at;
        let task_body = serde_json::to_string(&task)?;
        let record_body = serde_json::to_string(&next_record)?;
        let context_body = serde_json::to_string(context)?;
        ensure!(
            task_body.len() <= 1024 * 1024
                && record_body.len() <= 8 * 1024 * 1024
                && context_body.len() <= 8 * 1024 * 1024,
            "initial input body exceeds bounds"
        );
        ensure!(
            activation.matches_input(record, input) && activation.roster().task() == self.task().id,
            "activation/input ticket differs"
        );
        let record_image = crate::state::managed_binding::record_image(&next_record, &record_body)?;
        let unit_body = serde_json::to_string(&unit)?;
        let mut next = self.row.clone();
        bump(&mut next.version)?;
        next.pins.task = pin(task.version, &task)?;
        next.pins.workflow = Some((next_record.id, pin(next_record.version, &next_record)?));
        next.pins.context = Some(pin(context.version, context)?);
        let body = serde_json::to_string(&next)?;
        ensure!(body.len() <= 128 * 1024, "initial Driver exceeds bound");
        let plan = Arc::new(DriverPreparationAdvance {
            ticket: self,
            unit,
            unit_body,
            task,
            task_body,
            next,
            body,
            initial: false,
            governing,
            input: Some(InitialInput {
                activation: Some(activation),
                records: None,
                record_image,
                gate: None,
                executor: None,
                fresh_context: true,
                frame,
                record_before: record.clone(),
                record: next_record,
                record_body,
                context: context.clone(),
                context_body,
            }),
            reconcile_ready: std::sync::atomic::AtomicBool::new(false),
        });
        plan.ticket.association.retain_preparation(&plan)?;
        Ok(plan)
    }
    /// Only the actual AttemptManager draft reaches this private entry. The
    /// initial lane is deliberately narrower than replacement/resume admission.
    pub(crate) fn plan_first_preparation(
        self,
        mut unit: ExecutionUnit,
    ) -> Result<Arc<DriverPreparationAdvance>> {
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
    pub(crate) fn plan_preparing(self) -> Result<Arc<DriverPreparationAdvance>> {
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
    pub(crate) fn plan_preparation_base(self, base: &str) -> Result<Arc<DriverPreparationAdvance>> {
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
    ) -> Result<Arc<DriverPreparationAdvance>> {
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
        let plan = Arc::new(DriverPreparationAdvance {
            ticket: self,
            unit,
            unit_body,
            task,
            task_body,
            next,
            body,
            initial,
            governing,
            input: None,
            reconcile_ready: std::sync::atomic::AtomicBool::new(false),
        });
        plan.ticket.association.retain_preparation(&plan)?;
        Ok(plan)
    }
}
// Entered while the actual Store is excluded. Unwind/Err permits observation
// of this same attempted plan; a not-yet-started producer is never rolled back
// underneath its live caller merely because original rows still exist.
pub(in crate::state) struct Applying<'a>(&'a Arc<DriverPreparationAdvance>);
impl Drop for Applying<'_> {
    fn drop(&mut self) {
        self.0.allow_reconciliation();
    }
}
impl DriverPreparationAdvance {
    pub(crate) fn activation_roster(
        &self,
    ) -> Result<&crate::state::managed_binding::ActivationRoster> {
        Ok(self
            .input
            .as_ref()
            .and_then(|i| i.activation.as_ref())
            .context("not original activation input")?
            .roster())
    }
    pub(crate) fn planned_binding(&self) -> (Uuid, u64, u64, &str) {
        (self.next.id, self.next.epoch, self.next.version, &self.body)
    }
    pub(in crate::state) fn write_native_activation_tx(
        &self,
        tx: &Transaction<'_>,
        permits: &crate::state::managed_binding::PrivatePermitManager,
    ) -> Result<()> {
        let input = self.input.as_ref().context("activation input absent")?;
        input
            .activation
            .as_ref()
            .context("native activation plan absent")?
            .write_tx(tx, permits, &input.record_image)
    }
    pub(in crate::state) fn write_input_record_tx(
        &self,
        tx: &Transaction<'_>,
        permits: &crate::state::managed_binding::PrivatePermitManager,
        record: &Record,
    ) -> Result<Record> {
        let input = self.input.as_ref().context("input absent")?;
        let write = || {
            guard_record_tx(tx, record)?;
            write_record_tx_at(tx, record, self.input_timestamp())
        };
        if let Some(records) = &input.records {
            permits.with_exact_permit(vec![records.copy_for_transaction()?], || {
                let next = write()?;
                permits.ensure_consumed()?;
                Ok(next)
            })
        } else {
            write()
        }
    }
    pub(in crate::state) fn begin_input(self: &Arc<Self>) -> Result<Applying<'_>> {
        ensure!(
            self.input.is_some() && self.is_retained()?,
            "initial input plan lost custody"
        );
        Ok(Applying(self))
    }
    pub(in crate::state) fn validate_input_before_tx(
        &self,
        tx: &Transaction<'_>,
        task: &Task,
        record: &Record,
        context: Option<&ContextVersion>,
    ) -> Result<()> {
        let input = self.input.as_ref().context("not an initial input plan")?;
        input.frame.validate(&self.ticket.owner)?;
        self.ticket.validate_current_tx(tx)?;
        let mut projected = self.task.clone();
        projected.version = self.ticket.task().version;
        projected.updated_at = self.ticket.task().updated_at;
        ensure!(
            serde_json::to_value(task)? == serde_json::to_value(&projected)?
                && serde_json::to_value(record)? == serde_json::to_value(&input.record_before)?
                && context.map(serde_json::to_string).transpose()?
                    == input.fresh_context.then(|| input.context_body.clone()),
            "initial input write differs from retained plan"
        );
        if let Some(gate) = &input.gate {
            gate.validate_before_tx(tx)?;
        }
        let outstanding:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM managed_effects WHERE task_id=?1 AND state IN ('pending','unknown'))",[task.id.to_string()],|r|r.get(0))?;
        ensure!(!outstanding, "initial input has unresolved helper effects");
        Ok(())
    }
    pub(in crate::state) fn write_input_unit_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        if let Some(executor) = self.input.as_ref().and_then(|i| i.executor.as_ref()) {
            executor.write_tx(self, tx)?;
        }
        Ok(())
    }
    pub(in crate::state) fn input_observed_before(&self) -> Option<&Record> {
        self.input
            .as_ref()
            .and_then(|i| i.gate.as_ref())
            .and_then(|g| g.observed_before.as_ref())
    }
    pub(in crate::state) fn input_observation(&self) -> Option<&crate::workflow::GateObservation> {
        self.input
            .as_ref()
            .and_then(|i| i.gate.as_ref())
            .and_then(|g| g.observation.as_ref())
    }
    pub(in crate::state) fn input_timestamp(&self) -> i64 {
        self.task.updated_at
    }
    pub(in crate::state) fn input_namespace(&self) -> Result<&NamespaceSnapshot> {
        self.ticket
            .namespace
            .as_ref()
            .context("initial namespace missing")
    }
    pub(in crate::state) fn finish_input_tx(
        &self,
        tx: &Transaction<'_>,
        permits: &crate::state::managed_binding::PrivatePermitManager,
        task: &Task,
        record: &Record,
    ) -> Result<()> {
        let input = self.input.as_ref().context("not initial input")?;
        ensure!(
            serde_json::to_string(task)? == self.task_body
                && serde_json::to_string(record)? == input.record_body,
            "actual initial write differs from prescribed image"
        );
        self.validate_result(tx)?;
        ensure!(
            self.ticket.association.validates(
                self.ticket.row.id,
                self.ticket.row.epoch,
                self.ticket.row.version,
                &self.ticket.body
            ),
            "initial Driver revoked before publication"
        );
        self.write_driver_tx(tx, permits)?;
        if let Some(observation) = self.input_observation() {
            append_event(
                tx,
                &task.scope(),
                "workflow.gate_observed",
                json!({"workflow":record.id,"attempt":self.input.as_ref().and_then(|i|i.gate.as_ref()).map(|g|g.index),"observation":observation}),
            )?;
        }
        append_event(
            tx,
            &task.scope(),
            "rrx.private.runtime.driver_initial_input_advanced",
            json!({"driver":self.next.id,"version":self.next.version,
                "workflow":record.id,"context_version":input.context.version,"unit":self.unit.id}),
        )?;
        Ok(())
    }
    fn write_driver_tx(
        &self,
        tx: &Transaction<'_>,
        permits: &crate::state::managed_binding::PrivatePermitManager,
    ) -> Result<()> {
        let mutation = ExactRowMutation::new(
            "task_drivers",
            "UPDATE",
            Some(super::marker::image(&self.ticket.row, &self.ticket.body)?),
            Some(super::marker::image(&self.next, &self.body)?),
        )?;
        permits.with_exact_permit(vec![mutation], || {
            ensure!(tx.execute("UPDATE task_drivers SET version=?1,body=?2 WHERE task_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='driving' AND body=?7",params![self.next.version,self.body,self.task.id.to_string(),self.next.id.to_string(),self.next.epoch,self.ticket.row.version,self.ticket.body])?==1,"preparation Driver CAS changed");
            permits.ensure_consumed()
        })
    }
    pub(crate) fn allow_reconciliation(&self) {
        self.reconcile_ready
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
    pub(crate) fn reconciliation_ready(&self) -> bool {
        self.reconcile_ready
            .load(std::sync::atomic::Ordering::SeqCst)
    }
    pub(crate) fn belongs_to(
        &self,
        association: &crate::runtime::driver::DriverAssociation,
    ) -> bool {
        self.ticket.association.same_association(association)
    }
    pub(crate) fn original_binding(&self) -> (Uuid, u64, u64, &str) {
        (
            self.ticket.row.id,
            self.ticket.row.epoch,
            self.ticket.row.version,
            &self.ticket.body,
        )
    }
    pub(crate) fn is_retained(self: &Arc<Self>) -> Result<bool> {
        self.ticket.association.preparation_retained(self)
    }
    fn validate_rollback(&self, tx: &Transaction<'_>) -> Result<()> {
        if let Some(activation) = self.input.as_ref().and_then(|i| i.activation.as_ref()) {
            activation.validate_rollback(tx)?;
        }
        // No liveness is granted: this proves only that THIS atomic write-plan
        // did not commit. Other effects/leases/history are never released.
        self.ticket.scope.validate_current(tx)?;
        self.ticket.validate_selection_tx(tx)?;
        self.ticket.validate_source_tx(tx)?;
        if let Some(input) = &self.input {
            // A saved adoption whose cache already advanced cannot be relabeled
            // as an original rollback, even if unrelated SQL has changed again.
            input.frame.validate(&self.ticket.owner)?;
        }
        let task = self.ticket.task();
        let exact: bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM task_drivers WHERE task_id=?1 AND project_id=?2 AND goal_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='driving' AND body=?7)",params![task.id.to_string(),task.project_id.to_string(),task.goal_id.to_string(),self.ticket.row.id.to_string(),self.ticket.row.epoch,self.ticket.row.version,self.ticket.body],|r|r.get(0))?;
        ensure!(exact, "preparation rollback original Driver differs");
        if self.initial {
            let exists: bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM execution_units WHERE task_id=?1) OR EXISTS(SELECT 1 FROM execution_context WHERE unit_id=?2)",params![task.id.to_string(),self.unit.id.to_string()],|r|r.get(0))?;
            ensure!(!exists, "preparation rollback retains new Unit history");
        }
        let (_, _, version, body) = self.ticket.association.binding()?;
        ensure!(
            version == self.ticket.row.version && body == self.ticket.body,
            "preparation rollback cache differs"
        );
        Ok(())
    }
    fn validate_result(&self, tx: &Transaction<'_>) -> Result<()> {
        if let Some(activation) = self.input.as_ref().and_then(|i| i.activation.as_ref()) {
            activation.validate_result(tx)?;
        }
        if let Some(input) = &self.input {
            if input.executor.is_some() {
                input
                    .frame
                    .validate_adoption(&self.ticket.owner, &self.unit)?;
            } else {
                input.frame.validate(&self.ticket.owner)?;
            }
            if input.gate.is_some() || input.executor.is_some() {
                self.ticket.scope.validate_gate_projection(
                    tx,
                    &self.task,
                    &self.task_body,
                    (&input.record, &input.record_body),
                    (&input.context, &input.context_body),
                    input.fresh_context,
                )?;
            } else {
                self.ticket.scope.validate_input_projection(
                    tx,
                    &self.task,
                    &self.task_body,
                    (&input.record, &input.record_body),
                    (&input.context, &input.context_body),
                )?;
            }
        } else {
            self.ticket
                .scope
                .validate_projection(tx, &self.task, &self.task_body, None)?;
        }
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
        plan: &Arc<DriverPreparationAdvance>,
    ) -> Result<ExecutionUnit> {
        ensure!(
            plan.input.is_none(),
            "initial input requires atomic Workflow publication"
        );
        ensure!(
            plan.is_retained()?,
            "Driver preparation is not retained by its actual slot"
        );
        let _applying = Applying(plan);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        plan.ticket.validate_current_tx(&tx)?;
        let outstanding: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM managed_effects WHERE task_id=?1 AND state IN ('pending','unknown'))",[plan.task.id.to_string()],|r|r.get(0))?;
        ensure!(
            !outstanding,
            "Driver advance has outstanding/unknown helper effects"
        );
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
            let namespace = plan
                .ticket
                .namespace
                .as_ref()
                .context("Driver initial namespace snapshot missing")?;
            let written =
                put_task_tx_at_with_namespace(&tx, &input, plan.task.updated_at, Some(namespace))?;
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
        plan.write_driver_tx(&tx, &self.binding_permits)?;
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
        plan: &Arc<DriverPreparationAdvance>,
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
        ensure!(
            plan.is_retained()?,
            "preparation publication lost actual custody"
        );
        plan.ticket.association.publish_exact(&DriverPublication {
            task: plan.task.id,
            id: plan.next.id,
            epoch: plan.next.epoch,
            before_version: plan.ticket.row.version,
            before_body: plan.ticket.body.clone(),
            after_version: plan.next.version,
            after_body: plan.body.clone(),
        })?;
        if let Some(input) = &plan.input
            && input.executor.is_some()
        {
            input.frame.publish_adoption(&plan.unit, plan)?;
        }
        plan.ticket.association.retire_preparation(plan)
    }
    /// Actual service consumes the SAME retained pre-SQL plan. Missing/foreign
    /// images remain held; it never reapplies SQL or reconstructs a plan.
    pub(crate) fn reconcile_driver_preparation(
        &mut self,
        plan: &Arc<DriverPreparationAdvance>,
    ) -> Result<bool> {
        ensure!(
            self.connection.is_autocommit(),
            "preparation reconcile inside writer"
        );
        if !plan.is_retained()? {
            return Ok(true);
        }
        if !plan.reconciliation_ready() {
            return Ok(false);
        }
        if self.publish_driver_preparation(plan).is_ok() {
            return Ok(true);
        }
        let tx = self.connection.transaction()?;
        if plan.validate_rollback(&tx).is_err() {
            return Ok(false);
        }
        tx.commit()?;
        plan.ticket.association.retire_preparation(plan)?;
        Ok(true)
    }
}
