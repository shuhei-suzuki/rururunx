//! Actual retained Sources/Engine dispatcher. Control futures never own its job.
use super::{Runtime, driver::WorkerLifetime};
use crate::{
    domain::TaskId,
    state::{InitialDriverPlan, PendingDriverClaim, managed_binding::InstalledDriverComposition},
};
use anyhow::{Result, ensure};
use std::{sync::Arc, time::Duration};

impl Runtime {
    /// Root's real installed composition must exist BEFORE planning/claiming.
    /// This private entry is not a public caller-supplied availability switch.
    pub(crate) async fn admit_task_driver(
        self: &Arc<Self>,
        task: TaskId,
        composition: InstalledDriverComposition,
    ) -> Result<()> {
        let _admission = self.control_admission.lock().await;
        ensure!(
            self.service_running(),
            "Runtime is not accepting Task Drivers"
        );
        let current = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .task(task)?
            .ok_or_else(|| anyhow::anyhow!("Task missing"))?;
        ensure!(current.id == task, "Driver Task body identity differs");
        composition.validate_for(&self.owner, &current)?;
        let plan: InitialDriverPlan = crate::state::plan_initial_driver(
            self.owner.clone(),
            task,
            self.config.scheduler.global_max_sessions,
            self.config.scheduler.max_tasks_per_project,
        )?;
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
            drive(claim,lifetime).await
        }))?;
        self.wake.notify_one();
        Ok(())
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
    tokio::select! {
        biased;
        ()=lifetime.cancelled()=>return Err(anyhow::anyhow!("Task Driver cancelled before Workflow initialization")),
        result=engine.initialize_driven(task,&sources,&lifetime)=>{result?;}
    }
    loop {
        if lifetime.revoked() {
            return Err(anyhow::anyhow!("Task Driver revoked"));
        }
        let result = tokio::select! {
            biased;
            ()=lifetime.cancelled()=>return Err(anyhow::anyhow!("Task Driver cancelled")),
            result=engine.step_driven_initial(task,&sources,&lifetime)=>result?,
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
