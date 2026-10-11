//! Actual retained Sources/Engine dispatcher. Control futures never own its job.
use super::{Runtime, driver::WorkerLifetime};
use crate::{
    domain::TaskId,
    state::{InitialDriverPlan, PendingDriverClaim, managed_binding::InstalledDriverComposition},
};
use anyhow::{Result, ensure};
use std::{sync::Arc, time::Duration};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum SkipReason {
    CurrentTaskBounded,
    BoundedReread,
    Composition,
    ValidateFor,
    Plan,
    /// The Project stores a Task limit other than the fixed MVP value.
    ProjectLimitUnsupported,
}
pub(super) enum AdmitOutcome {
    Claimed,
    Skipped(SkipReason),
}
impl Runtime {
    /// Root's real installed composition must exist BEFORE planning/claiming.
    /// This private entry is not a public caller-supplied availability switch.
    pub(super) fn admit_task_driver(
        self: &Arc<Self>,
        _admission: &tokio::sync::MutexGuard<'_, ()>,
        key: &crate::state::CandidateKey,
        task: TaskId,
        composition: InstalledDriverComposition,
    ) -> Result<AdmitOutcome> {
        ensure!(
            self.service_running(),
            "Runtime is not accepting Task Drivers"
        );
        let current = match self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .current_task_bounded(key)
        {
            Ok(task) => task,
            Err(_) => return Ok(AdmitOutcome::Skipped(SkipReason::BoundedReread)),
        };
        if current.id != task || composition.validate_for(&self.owner, &current).is_err() {
            return Ok(AdmitOutcome::Skipped(SkipReason::ValidateFor));
        }
        let plan: InitialDriverPlan = match crate::state::plan_initial_driver(
            self.owner.clone(),
            task,
            self.config.scheduler.global_max_sessions,
        ) {
            Ok(plan) => plan,
            Err(error) if error.is::<crate::state::ProjectLimitUnsupported>() => {
                return Ok(AdmitOutcome::Skipped(SkipReason::ProjectLimitUnsupported));
            }
            Err(_) => return Ok(AdmitOutcome::Skipped(SkipReason::Plan)),
        };
        let pending = self._drivers.reserve_pending(&plan)?;
        // The known commit and actual spawn/custody installation have NO await.
        // Cancellation at an async boundary cannot strand an unowned created job.
        let claim = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .claim_initial_driver(plan, composition)?;
        pending.bind_claim(&claim)?;
        let runtime = Arc::downgrade(self);
        pending.spawn(move |lifetime,ack| Box::pin(async move {
            #[cfg(test)]
            let issue87_spawned=std::time::Instant::now();
            {
                let runtime=runtime.upgrade().ok_or_else(||anyhow::anyhow!("Runtime ended before Driver startup"))?;
                let _admission=tokio::select! {
                    biased;
                    ()=lifetime.cancelled()=>return Err(anyhow::anyhow!("Driver stopped before startup acknowledgement")),
                    admission=runtime.control_admission.lock()=>admission,
                };
                ensure!(runtime.service_running(),"Runtime stopped before Driver startup");
                let store=runtime.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?;
                store.validate_pending_driver(&claim)?;
                lifetime.activate(ack,&claim)?;
                // Runtime and Store guards end BEFORE Source/helper awaits.
            }
            #[cfg(test)]
            let issue87_task=claim.task().id;
            #[cfg(test)]
            if issue87_spawned.elapsed()>Duration::from_millis(500) { crate::issue87_trace::mark(&format!("driver {issue87_task} startup admission waited {:?}",issue87_spawned.elapsed())); }
            let result=drive(claim,lifetime).await;
            #[cfg(test)]
            crate::issue87_trace::mark(&format!("driver {issue87_task} drive returned after {:?} ok={}",issue87_spawned.elapsed(),result.is_ok()));
            #[cfg(test)]
            if let Err(error)=&result { eprintln!("actual retained Driver outcome: {error:#}"); }
            result
        }))?;
        self.wake.notify_one();
        Ok(AdmitOutcome::Claimed)
    }
}
async fn drive(claim: PendingDriverClaim, lifetime: WorkerLifetime) -> Result<()> {
    let task = claim.task().id;
    let sources = claim.composition().sources().clone();
    let engine = claim.composition().engine().clone();
    let provider = claim.composition().provider().to_owned();
    let ticket = crate::state::read_driver_ticket(claim.owner().clone(), lifetime.association()?)?;
    {
        let mut store = claim
            .owner()
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        store.validate_driver_read(&ticket)?;
    }
    // Every concrete reservation/transition below must carry the worker's exact
    // private pre-marker lane. Root's composition cannot be issued while that
    // coupling/marker/binder is absent; these existing calls cannot grant it.
    tokio::select! {
        biased;
        ()=lifetime.cancelled()=>return Err(anyhow::anyhow!("Task Driver cancelled before Source preparation")),
        result=sources.prepare_driven(task,&provider,&lifetime)=>{result?;}
    }
    #[cfg(test)]
    crate::issue87_trace::mark(&format!("driver {task} prepared"));
    tokio::select! {
        biased;
        ()=lifetime.cancelled()=>return Err(anyhow::anyhow!("Task Driver cancelled before Workflow initialization")),
        result=engine.initialize_driven(task,claim.composition(),&lifetime)=>{result?;}
    }
    loop {
        if lifetime.revoked() {
            return Err(anyhow::anyhow!("Task Driver revoked"));
        }
        let result = tokio::select! {
            biased;
            ()=lifetime.cancelled()=>return Err(anyhow::anyhow!("Task Driver cancelled")),
            result=engine.step_driven_initial(task,claim.composition(),&lifetime)=>result?,
        };
        if matches!(result, crate::workflow::StepResult::Finished) {
            return Ok(());
        }
        // A bounded wake interval is a poll hint only. Durable Workflow/Unit
        // authority is rechecked by the real Engine, never by a result label.
        tokio::select! {
            ()=lifetime.cancelled()=>return Err(anyhow::anyhow!("Task Driver cancelled while waiting")),
            ()=tokio::time::sleep(Duration::from_millis(100))=>{},
        }
    }
}
