//! First Executor reservation and SAME preparation adoption, before any marker.
use super::*;
use crate::{
    adapter::native::NativePhasePort,
    execution::workflow_source::InitialExecutorFrame,
    workflow::{Actor, AttemptState, Phase, PhaseAttempt, WorkflowSnapshot},
};

pub(super) struct ExecutorInput {
    selected: Arc<NativePhasePort>,
}

// Prescribed DTO projection only, not authority. The plan producer additionally
// requires the actual Sources namespace completion, live ticket and vtable.
fn adopted_unit(
    old: &ExecutionUnit,
    phase: Phase,
    provider: &str,
    at: i64,
) -> Result<ExecutionUnit> {
    ensure!(
        phase.actor() == Actor::Executor
            && old.kind == UnitKind::Executor
            && old.phase == WORKFLOW_SOURCE_BOOTSTRAP
            && old.generation == 1
            && old.state == UnitState::Preparing
            && old.disposition == Disposition::Active
            && old.native_effects_open
            && old.result_finalization_open
            && old.work.is_none()
            && old.session_id.is_none()
            && old.artifact_id.is_none()
            && old.wait_reason.is_none()
            && old.capacity_retry_at.is_none()
            && crate::execution::valid_oid(&old.base_sha),
        "first Executor requires pristine same-provider preparation"
    );
    let mut unit = old.clone();
    unit.phase = phase.key().into();
    bump(&mut unit.version)?;
    unit.updated_at = at;
    Ok(unit)
}
impl DriverReadTicket {
    pub(crate) fn plan_first_executor(
        self,
        completion: InitialExecutorFrame,
        selected: Arc<NativePhasePort>,
        context: ContextVersion,
        at: i64,
    ) -> Result<Arc<DriverPreparationAdvance>> {
        ensure!(
            self.row.marker.is_none() && self.source.is_none() && self.namespace.is_some(),
            "first Executor requires original pre-marker lane"
        );
        let frame = completion.into_frame();
        frame.validate(&self.owner)?;
        self.preparation_matches(frame.unit())?;
        let (old_record, old_context) = self.scope.workflow_input()?;
        let old_record = old_record.clone();
        let mut task = self.task().clone();
        let mut w: WorkflowSnapshot = serde_json::from_value(old_record.data.clone())?;
        gates::initial_only(&w)?;
        frame.validate_context(&task, &old_record, old_context)?;
        let phase = crate::workflow::next_phase(&w).context("first Executor phase missing")?;
        ensure!(
            w.active.is_none()
                && w.held_reason.is_none()
                && w.history.iter().all(|a| a.state == AttemptState::Succeeded)
                && w.history.len() == w.completed.len()
                && w.configured_phases
                    .iter()
                    .find(|p| p.actor() == Actor::Executor)
                    == Some(&phase)
                && w.configured_phases
                    .iter()
                    .take_while(|p| **p != phase)
                    .all(|p| w.completed.contains_key(p))
                && selected.alias() == task.executor
                && std::ptr::eq(selected.owner(), self.owner.as_ref()),
            "first Executor history or actual selected vtable differs"
        );
        let unit = adopted_unit(frame.unit(), phase, selected.provider(), at)?;
        ensure!(
            context.version
                == old_context
                    .version
                    .checked_add(1)
                    .context("Context overflow")?,
            "first Executor Context must be consecutive"
        );
        let index = w.history.len();
        task.phase = Some(phase.key().into());
        task.state = phase.task_state();
        task.context_version = context.version;
        task.revision = Some(context.revision.clone());
        w.context_version = context.version;
        w.context_fresh = true;
        w.sources.payload = context.data["payload"]
            .as_str()
            .context("first Executor payload missing")?
            .into();
        let budget = serde_json::from_value(context.data["budget"].clone())?;
        w.history.push(PhaseAttempt {
            phase,
            generation: w.generation,
            context_version: context.version,
            budget,
            state: AttemptState::Running,
            session_id: None,
            execution: None,
            unit: Some((&unit).into()),
            native_wait: None,
            next_due: None,
            dispatch_started: false,
            observations: vec![],
            claimed_observations: 0,
            agent: Some(task.executor.clone()),
            started_at: at,
            completed_at: None,
            detail: None,
        });
        w.active = Some(index);
        let mut record = old_record;
        record.data = serde_json::to_value(w)?;
        frame.validate_context(&task, &record, &context)?;
        crate::workflow::validate_transition(&task, &record, Some(self.scope.workflow_input()?.0))?;
        let before = record.clone();
        bump(&mut task.version)?;
        task.updated_at = at;
        bump(&mut record.version)?;
        record.updated_at = at;
        let task_body = serde_json::to_string(&task)?;
        let record_body = serde_json::to_string(&record)?;
        let context_body = serde_json::to_string(&context)?;
        let unit_body = serde_json::to_string(&unit)?;
        ensure!(
            task_body.len() <= 1024 * 1024
                && record_body.len() <= 8 * 1024 * 1024
                && context_body.len() <= 8 * 1024 * 1024
                && unit_body.len() <= 16 * 1024,
            "first Executor body exceeds bound"
        );
        let mut next = self.row.clone();
        bump(&mut next.version)?;
        next.pins.task = pin(task.version, &task)?;
        next.pins.workflow = Some((record.id, pin(record.version, &record)?));
        next.pins.context = Some(pin(context.version, &context)?);
        next.pins.preparation = Some(PreparationPin {
            unit: unit.id,
            version: unit.version,
            digest: hash(&unit)?,
        });
        let body = serde_json::to_string(&next)?;
        ensure!(
            body.len() <= 128 * 1024,
            "first Executor Driver exceeds bound"
        );
        let (p, g) = self.scope.governing_owners();
        let governing = crate::state::execution::governing_digest(p, g)?;
        ensure!(
            frame.governing() == governing,
            "first Executor governing changed"
        );
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
                gate: None,
                executor: Some(ExecutorInput { selected }),
                fresh_context: true,
                frame,
                record_before: before,
                record,
                record_body,
                context,
                context_body,
            }),
            reconcile_ready: std::sync::atomic::AtomicBool::new(false),
        });
        plan.ticket.association.retain_preparation(&plan)?;
        Ok(plan)
    }
}
impl ExecutorInput {
    pub(super) fn write_tx(
        &self,
        plan: &DriverPreparationAdvance,
        tx: &Transaction<'_>,
    ) -> Result<()> {
        let (old, raw) = plan
            .ticket
            .preparation
            .as_ref()
            .context("original preparation missing")?;
        ensure!(
            self.selected.alias() == plan.task.executor
                && self.selected.provider() == old.provider
                && std::ptr::eq(self.selected.owner(), plan.ticket.owner.as_ref()),
            "actual Executor selection changed"
        );
        // Settled registered Git is the sole allowed effect history. Session,
        // native quota/operation and unknown/pending helper history cannot be adopted.
        let forbidden:bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM managed_effects WHERE unit_id=?1 AND (state NOT IN ('confirmed','resolved') OR json_extract(body,'$.kind') IS NOT 'git_helper')) OR EXISTS(SELECT 1 FROM quota_leases WHERE unit_id=?1) OR EXISTS(SELECT 1 FROM quota_waiters WHERE unit_id=?1) OR EXISTS(SELECT 1 FROM records WHERE task_id=?2 AND kind='session') OR EXISTS(SELECT 1 FROM managed_phase_operations WHERE task_id=?2)",params![old.id.to_string(),plan.task.id.to_string()],|r|r.get(0))?;
        ensure!(
            !forbidden,
            "first Executor has native or unresolved effect history"
        );
        ensure!(tx.execute("UPDATE execution_units SET version=?1,body=?2 WHERE id=?3 AND version=?4 AND body=?5",params![plan.unit.version,plan.unit_body,old.id.to_string(),old.version,raw])? == 1,
            "first Executor original Unit CAS changed");
        Ok(())
    }
}
impl DriverPreparationAdvance {
    pub(crate) fn validate_adoption_images(
        &self,
        owner: &Arc<crate::execution::RuntimeOwner>,
        original: &ExecutionUnit,
        adopted: &ExecutionUnit,
    ) -> Result<()> {
        ensure!(
            self.input.as_ref().is_some_and(|i| i.executor.is_some())
                && Arc::ptr_eq(owner, &self.ticket.owner)
                && self
                    .ticket
                    .preparation
                    .as_ref()
                    .is_some_and(
                        |(_, raw)| serde_json::to_string(original).is_ok_and(|b| &b == raw)
                    )
                && serde_json::to_string(adopted)? == self.unit_body,
            "adoption cache images differ from saved actual plan"
        );
        Ok(())
    }
    pub(in crate::state) fn executor_write(
        &self,
    ) -> Result<(Task, Record, &ContextVersion, u64, u64)> {
        let input = self
            .input
            .as_ref()
            .context("first Executor input missing")?;
        ensure!(
            input.executor.is_some() && input.gate.is_none(),
            "not a first Executor plan"
        );
        let mut task = self.task.clone();
        task.version = self.ticket.task().version;
        task.updated_at = self.ticket.task().updated_at;
        Ok((
            task,
            input.record_before.clone(),
            &input.context,
            self.next.pins.project.version,
            self.next.pins.goal.version,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Pure prescribed-image controls. No Store/Driver/native proof is seeded.
    fn preparation() -> ExecutionUnit {
        ExecutionUnit {
            id: crate::execution::UnitId::new(),
            scope: Scope {
                project_id: ProjectId::new(),
                goal_id: Some(GoalId::new()),
                task_id: Some(TaskId::new()),
            },
            kind: UnitKind::Executor,
            generation: 1,
            owner_epoch: 3,
            version: 7,
            phase: WORKFLOW_SOURCE_BOOTSTRAP.into(),
            provider: "codex".into(),
            state: UnitState::Preparing,
            native_effects_open: true,
            result_finalization_open: true,
            work: None,
            cleanup: CleanupOutcome::Unknown,
            disposition: Disposition::Active,
            worktree: "/isolated/nongrant-preparation".into(),
            branch: Some("rrx/nongrant".into()),
            base_sha: "a".repeat(40),
            profile_digest: "b".repeat(64),
            cookie: Uuid::new_v4().to_string(),
            session_id: None,
            artifact_id: None,
            wait_reason: None,
            capacity_retry_at: None,
            created_at: 10,
            updated_at: 20,
        }
    }
    #[test]
    fn first_executor_projection_preserves_all_original_identity_and_no_dispatch() {
        let old = preparation();
        let after = adopted_unit(&old, Phase::Implement, "codex", 30).unwrap();
        let mut expected = old.clone();
        expected.phase = "implement".into();
        expected.version = 8;
        expected.updated_at = 30;
        assert_eq!(
            serde_json::to_value(&after).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
        assert_eq!(after.id, old.id);
        assert!(after.session_id.is_none() && after.work.is_none() && after.artifact_id.is_none());
        assert_eq!(after.state, UnitState::Preparing);
    }
    #[test]
    fn first_executor_projection_refuses_native_or_revoked_preparation() {
        let old = preparation();
        assert!(
            adopted_unit(&old, Phase::Implement, "claude", 30).is_err(),
            "another provider cannot adopt the original preparation"
        );
        assert!(adopted_unit(&old, Phase::RequirementsReview, "codex", 30).is_err());
        let mut variants = Vec::new();
        let mut v = old.clone();
        v.kind = UnitKind::Reviewer;
        variants.push(v);
        let mut v = old.clone();
        v.phase = "implement".into();
        variants.push(v);
        let mut v = old.clone();
        v.generation = 2;
        variants.push(v);
        let mut v = old.clone();
        v.state = UnitState::Running;
        variants.push(v);
        let mut v = old.clone();
        v.disposition = Disposition::Lost;
        variants.push(v);
        let mut v = old.clone();
        v.native_effects_open = false;
        variants.push(v);
        let mut v = old.clone();
        v.result_finalization_open = false;
        variants.push(v);
        let mut v = old.clone();
        v.work = Some(crate::execution::WorkOutcome::Unknown);
        variants.push(v);
        let mut v = old.clone();
        v.session_id = Some(SessionId::new());
        variants.push(v);
        let mut v = old.clone();
        v.artifact_id = Some(crate::execution::ArtifactId::new());
        variants.push(v);
        let mut v = old.clone();
        v.capacity_retry_at = Some(40);
        variants.push(v);
        let mut v = old;
        v.base_sha.clear();
        variants.push(v);
        for variant in variants {
            assert!(
                adopted_unit(&variant, Phase::Implement, "codex", 30).is_err(),
                "native/revoked preparation cannot be projected: {:?}",
                variant
            );
        }
    }
    #[test]
    fn first_executor_projection_rejects_version_overflow_without_original_mutation() {
        let mut old = preparation();
        old.version = u64::MAX;
        let before = serde_json::to_string(&old).unwrap();
        assert!(adopted_unit(&old, Phase::Implement, "codex", 30).is_err());
        assert_eq!(serde_json::to_string(&old).unwrap(), before);
    }
}
