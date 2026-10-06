//! Retained control-service event loop. It publishes routing/readiness attention,
//! never a native phase, prepared owner, success or reconstructed Driver.
use super::Runtime;
use anyhow::{Context, Result, ensure};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
struct Running(Arc<AtomicBool>);
impl Drop for Running {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

impl Runtime {
    fn observe_task_drivers(&self) -> Result<usize> {
        let pending = self._drivers.observe_finished()?;
        for exit in self._drivers.pending_exits()? {
            let publication = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .record_driver_exit(&exit)?;
            self._drivers.acknowledge_exit(&exit, &publication)?;
        }
        Ok(pending)
    }
    /// Start exactly once on the existing owner. Missing native binding stays a
    /// named durable hold; the service does not instantiate another owner epoch.
    pub async fn start(self: &Arc<Self>) -> Result<()> {
        let _admission = self.control_admission.lock().await;
        ensure!(
            !self.stopping.load(Ordering::SeqCst),
            "Runtime already stopping"
        );
        let mut slot = self.supervisor.lock().await;
        ensure!(
            slot.is_none() && !self.started.swap(true, Ordering::SeqCst),
            "Runtime already started"
        );
        let retained = Arc::downgrade(self);
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);
        *slot = Some(tokio::spawn(async move {
            let _running = Running(running);
            let mut sequence = 0;
            let mut backoff = 0;
            loop {
                let Some(runtime) = retained.upgrade() else {
                    return Ok(());
                };
                if runtime.stopping.load(Ordering::SeqCst) {
                    return Ok(());
                }
                let (next, more) = runtime
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .reconcile_runtime_attention(
                        runtime.owner.instance_id(),
                        runtime.owner.epoch(),
                        sequence,
                    )?;
                sequence = next;
                let pending = runtime.phases.reconcile_pending()?;
                let driver_pending = runtime.observe_task_drivers()?;
                let delay = super::phase_supervisor::PhaseSupervisor::delay(
                    pending || driver_pending > 0,
                    &mut backoff,
                );
                let wake = runtime.wake.clone();
                drop(runtime);
                if more {
                    tokio::task::yield_now().await;
                    continue;
                }
                // A wake is only a hint. Every sweep reloads durable rows; full,
                // lost or duplicated hints cannot ratify a launch or claim.
                tokio::select! {
                    ()=wake.notified()=>{},
                    ()=tokio::time::sleep(delay)=>{},
                }
                sequence = 0;
            }
        }));
        Ok(())
    }
    /// Cooperative shutdown retains the JoinHandle on timeout/caller Drop.
    /// No child/native work is owned by this initial control-service loop.
    pub async fn shutdown(&self) -> Result<()> {
        let _admission =
            tokio::time::timeout(Duration::from_secs(5), self.control_admission.lock())
                .await
                .context("Runtime control admission shutdown remains pending")?;
        self.stopping.store(true, Ordering::SeqCst);
        self._drivers.stop_all();
        self.phases.close_unmarked();
        self.wake.notify_one();
        let mut slot = self.supervisor.lock().await;
        if let Some(handle) = slot.as_mut() {
            let completed = tokio::time::timeout(Duration::from_secs(5), handle)
                .await
                .context("Runtime control loop shutdown remains pending")?;
            // Completion (including Err/panic) consumes the JoinHandle result.
            // Take it before propagating that error; polling it again is invalid.
            // Timeout/caller Drop still leaves the pending handle in its owner.
            slot.take();
            completed??;
        }
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        while self.observe_task_drivers()? > 0 {
            ensure!(
                tokio::time::Instant::now() < deadline,
                "Task Driver shutdown remains pending with owned handles"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        Ok(())
    }
}
