//! An attempt's owned, level-triggered lifetime is independent of its caller.
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch};

use super::{
    preparation::{Admission, Cause, Preparation},
    protocol::failure,
};
use crate::adapter::{AdapterResult, ErrorKind, SessionStatus};

#[derive(Clone, Debug)]
pub(super) enum Outcome {
    RestoredBeforeAdmission {
        cause: Cause,
        snapshot: SessionStatus,
    },
    CheckpointCommitted {
        input_version: u64,
        snapshot: SessionStatus,
    },
    FailedBeforeAdmission {
        cause: Cause,
        snapshot: SessionStatus,
    },
    Terminal {
        snapshot: SessionStatus,
    },
    Lost {
        cause: Cause,
        publication_result: Result<SessionStatus, Cause>,
    },
    RestoreUnpublished {
        cause: Cause,
        error: Cause,
    },
    FreshUnpublished {
        cause: Cause,
        error: Cause,
    },
}
impl Outcome {
    pub fn snapshot(&self) -> Option<&SessionStatus> {
        match self {
            Self::RestoredBeforeAdmission { snapshot, .. }
            | Self::CheckpointCommitted { snapshot, .. }
            | Self::FailedBeforeAdmission { snapshot, .. }
            | Self::Terminal { snapshot } => Some(snapshot),
            Self::Lost {
                publication_result: Ok(snapshot),
                ..
            } => Some(snapshot),
            _ => None,
        }
    }
    pub fn cause(&self) -> Option<&Cause> {
        match self {
            Self::RestoredBeforeAdmission { cause, .. }
            | Self::FailedBeforeAdmission { cause, .. }
            | Self::Lost { cause, .. }
            | Self::RestoreUnpublished { cause, .. }
            | Self::FreshUnpublished { cause, .. } => Some(cause),
            _ => None,
        }
    }
    pub fn error(&self) -> Option<crate::adapter::AdapterError> {
        match self {
            Self::RestoreUnpublished { error, .. }
            | Self::FreshUnpublished { error, .. }
            | Self::Lost {
                publication_result: Err(error),
                ..
            } => Some(error.error()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum Phase {
    Preparing,
    AwaitingTurnAck,
    Supervised,
    Finished(Arc<Outcome>),
}

pub(super) struct Control {
    pub preparation: Preparation,
    pub stop: mpsc::Sender<()>,
    phase: watch::Sender<Phase>,
    // A leaf publication channel; updating it while Store is held never takes
    // admission/registry. It cannot be stolen by a later attempt's shared watch.
    published: watch::Sender<Option<SessionStatus>>,
    task: Mutex<TaskOwner>,
    #[cfg(test)]
    pub cas_entered: std::sync::atomic::AtomicBool,
}
enum TaskOwner {
    Vacant,
    Installing,
    Running(tokio::task::JoinHandle<()>),
    Released,
}
impl Control {
    pub fn new(previous: Option<SessionStatus>) -> (Arc<Self>, mpsc::Receiver<()>) {
        let (stop, stopped) = mpsc::channel(1);
        let (phase, _) = watch::channel(Phase::Preparing);
        let (published, _) = watch::channel(previous);
        (
            Arc::new(Self {
                preparation: Preparation::new(),
                stop,
                phase,
                published,
                task: Mutex::new(TaskOwner::Vacant),
                #[cfg(test)]
                cas_entered: std::sync::atomic::AtomicBool::new(false),
            }),
            stopped,
        )
    }
    pub fn subscribe(&self) -> watch::Receiver<Phase> {
        self.phase.subscribe()
    }
    pub fn publish(&self, status: SessionStatus) {
        self.published.send_replace(Some(status));
    }
    pub fn published(&self) -> Option<SessionStatus> {
        self.published.borrow().clone()
    }
    pub fn awaiting_ack(&self) {
        self.phase.send_replace(Phase::AwaitingTurnAck);
    }
    pub fn supervised(&self) {
        self.phase.send_replace(Phase::Supervised);
    }
    pub fn finished(&self, outcome: Outcome) {
        self.phase.send_replace(Phase::Finished(Arc::new(outcome)));
    }
    pub fn spawn(
        self: &Arc<Self>,
        future: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> AdapterResult<()> {
        let runtime = tokio::runtime::Handle::try_current().map_err(|_| {
            self.preparation.failed(failure(
                ErrorKind::LaunchFailure,
                "native attempt requires an async runtime",
            ))
        })?;
        {
            let mut task = self.task.lock().map_err(|_| {
                self.preparation.failed(failure(
                    ErrorKind::StateFailure,
                    "native attempt task owner poisoned",
                ))
            })?;
            if !matches!(*task, TaskOwner::Vacant) {
                return Err(self.preparation.failed(failure(
                    ErrorKind::StateConflict,
                    "native attempt task already owned",
                )));
            }
            *task = TaskOwner::Installing;
        }
        // A closed runtime can synchronously drop the future inside spawn. Its
        // TaskGuard must release this exact owner without re-locking a mutex
        // held by spawn. Released also records completion-before-installation.
        let spawned = runtime.spawn(future);
        let mut task = self.task.lock().unwrap_or_else(|error| error.into_inner());
        if matches!(*task, TaskOwner::Installing) {
            *task = TaskOwner::Running(spawned);
        } else {
            drop(task);
            drop(spawned);
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn abort_owned_task(&self) {
        let handle = {
            let task = self.task.lock().unwrap();
            match &*task {
                TaskOwner::Running(task) => Some(task.abort_handle()),
                _ => None,
            }
        };
        if let Some(handle) = handle {
            handle.abort();
        }
    }
    pub fn release_task(&self) {
        // Releasing one's own JoinHandle never aborts it or touches a replacement.
        let previous = {
            let mut task = self.task.lock().unwrap_or_else(|error| error.into_inner());
            std::mem::replace(&mut *task, TaskOwner::Released)
        };
        if let TaskOwner::Running(task) = previous {
            drop(task);
        }
    }
    pub async fn wait_finished(&self) -> AdapterResult<Outcome> {
        let mut receiver = self.subscribe();
        loop {
            if let Phase::Finished(outcome) = receiver.borrow_and_update().clone() {
                return Ok(outcome.as_ref().clone());
            }
            receiver.changed().await.map_err(|_| {
                failure(
                    ErrorKind::SessionLost,
                    "native attempt completion unavailable",
                )
            })?;
        }
    }
    pub fn consumed(&self) -> bool {
        matches!(self.preparation.state(), Ok(Admission::Consumed))
    }
}

pub(super) struct CallerGuard {
    control: Arc<Control>,
    armed: bool,
}
impl CallerGuard {
    pub fn new(control: Arc<Control>) -> Self {
        Self {
            control,
            armed: true,
        }
    }
    pub fn disarm(&mut self) {
        self.armed = false;
    }
}
impl Drop for CallerGuard {
    fn drop(&mut self) {
        if self.armed {
            self.control.preparation.cancel();
        }
    }
}

/// Last-resort typed uncertainty after nested child/Reservation owners drop.
/// Normal paths publish their factual outcome before this guard releases a task.
pub(super) struct TaskGuard(pub Arc<Control>);
impl Drop for TaskGuard {
    fn drop(&mut self) {
        if !matches!(*self.0.phase.borrow(), Phase::Finished(_)) {
            let error = failure(
                ErrorKind::SessionLost,
                if std::thread::panicking() {
                    "owned preparation/supervision task panicked"
                } else {
                    "owned preparation/supervision task dropped without completion"
                },
            );
            let selected = self.0.preparation.failed(error);
            let cause = self.0.preparation.cause(&selected);
            let publication_result = match self.0.published() {
                Some(status) if status.session.state == crate::domain::SessionState::Lost => {
                    Ok(status)
                }
                _ => Err(Cause::Failed(
                    ErrorKind::SessionLost,
                    "task abandonment has no verified terminal publication".into(),
                )),
            };
            self.0.finished(Outcome::Lost {
                cause,
                publication_result,
            });
        }
        self.0.release_task();
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(super) enum TestPoint {
    BeforeStarting,
    BeforeInitialPersist,
    BeforeBootstrap,
    BeforeDispatch,
    BeforeNativeCleanup,
    BeforeCheckpointCommit,
    AfterCheckpointCommit,
    AfterCheckpointFinished,
    StopWaiting,
    AfterTerminalPublication,
}
#[cfg(test)]
#[derive(Default)]
pub(super) struct TestGates(Mutex<std::collections::HashMap<TestPoint, Arc<TestGate>>>);
#[cfg(test)]
pub(super) struct TestGate {
    reached: watch::Sender<bool>,
    released: watch::Sender<bool>,
}
#[cfg(test)]
impl TestGates {
    pub fn install(&self, point: TestPoint) -> Arc<TestGate> {
        let gate = Arc::new(TestGate {
            reached: watch::channel(false).0,
            released: watch::channel(false).0,
        });
        assert!(self.0.lock().unwrap().insert(point, gate.clone()).is_none());
        gate
    }
    pub async fn wait(&self, point: TestPoint) {
        let gate = self.0.lock().unwrap().remove(&point);
        if let Some(gate) = gate {
            gate.reached.send_replace(true);
            TestGate::level(&gate.released).await;
        }
    }
}
#[cfg(test)]
impl TestGate {
    async fn level(sender: &watch::Sender<bool>) {
        let mut receiver = sender.subscribe();
        loop {
            if *receiver.borrow_and_update() {
                return;
            }
            receiver.changed().await.unwrap();
        }
    }
    pub async fn reached(&self) {
        Self::level(&self.reached).await;
    }
    pub fn release(&self) {
        self.released.send_replace(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_runtime_releases_unpolled_owned_task_without_installation_deadlock() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let handle = runtime.handle().clone();
        drop(runtime);
        let (control, _) = Control::new(None);
        let owner = control.clone();
        let (sent, received) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            let _entered = handle.enter();
            let guard = TaskGuard(owner.clone());
            let result = owner.spawn(async move {
                let _guard = guard;
                std::future::pending::<()>().await;
            });
            sent.send(result).unwrap();
        });
        // This synthetic fixture owns no OS child. A deadlock mutant leaves only
        // this thread, so a bounded assertion can finish without orphaning native
        // or Git process ownership.
        let result = received.recv_timeout(std::time::Duration::from_secs(2));
        if result.is_ok() {
            worker.join().unwrap();
        }
        result
            .expect("closed scheduler cannot deadlock owned task installation")
            .unwrap();
        assert!(matches!(*control.task.lock().unwrap(), TaskOwner::Released));
        assert!(matches!(
            &*control.phase.borrow(),
            Phase::Finished(outcome) if matches!(outcome.as_ref(), Outcome::Lost { publication_result: Err(_), .. })
        ));
    }

    #[test]
    fn runtime_shutdown_and_missing_runtime_keep_owned_task_drop_uncertain() {
        let (missing, _) = Control::new(None);
        let guard = TaskGuard(missing.clone());
        let error = missing
            .spawn(async move {
                let _guard = guard;
            })
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::LaunchFailure);
        assert!(matches!(*missing.task.lock().unwrap(), TaskOwner::Released));
        assert!(matches!(
            &*missing.phase.borrow(),
            Phase::Finished(outcome) if matches!(outcome.as_ref(), Outcome::Lost { publication_result: Err(_), .. })
        ));

        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let (control, _) = Control::new(None);
        let guard = TaskGuard(control.clone());
        {
            let _entered = runtime.enter();
            control
                .spawn(async move {
                    let _guard = guard;
                    std::future::pending::<()>().await;
                })
                .unwrap();
        }
        assert!(matches!(
            *control.task.lock().unwrap(),
            TaskOwner::Running(_)
        ));
        drop(runtime);
        assert!(matches!(*control.task.lock().unwrap(), TaskOwner::Released));
        assert!(matches!(
            &*control.phase.borrow(),
            Phase::Finished(outcome) if matches!(outcome.as_ref(), Outcome::Lost { publication_result: Err(_), .. })
        ));
    }

    #[tokio::test]
    async fn finished_is_level_triggered_and_caller_drop_only_cancels_preparing() {
        let (control, mut stopped) = Control::new(None);
        let guard = CallerGuard::new(control.clone());
        control.preparation.consume(|| Ok(())).unwrap();
        control.awaiting_ack();
        drop(guard);
        assert!(control.consumed());
        assert!(
            stopped.try_recv().is_err(),
            "caller drop cannot inject an interrupt after consumption"
        );
        control.finished(Outcome::FreshUnpublished {
            cause: Cause::Failed(ErrorKind::StateFailure, "factual failure".into()),
            error: Cause::Failed(ErrorKind::StateFailure, "publication failed".into()),
        });
        // Subscription happens after completion; no edge notification is needed.
        for _ in 0..2 {
            let outcome =
                tokio::time::timeout(std::time::Duration::from_secs(1), control.wait_finished())
                    .await
                    .unwrap()
                    .unwrap();
            assert_eq!(outcome.error().unwrap().message, "publication failed");
            assert_eq!(outcome.cause().unwrap().error().message, "factual failure");
        }
    }

    #[tokio::test]
    async fn dropped_owned_task_is_uncertain_without_invented_restore() {
        let (control, _) = Control::new(None);
        let task = TaskGuard(control.clone());
        drop(task);
        let outcome = control.wait_finished().await.unwrap();
        assert!(matches!(
            outcome,
            Outcome::Lost {
                publication_result: Err(_),
                ..
            }
        ));
        assert!(
            outcome
                .cause()
                .unwrap()
                .error()
                .message
                .contains("dropped without completion")
        );
    }
}
