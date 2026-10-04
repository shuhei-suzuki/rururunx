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
    Finished(Outcome),
}

pub(super) struct Control {
    pub preparation: Preparation,
    pub stop: mpsc::Sender<()>,
    phase: watch::Sender<Phase>,
    // A leaf publication channel; updating it while Store is held never takes
    // admission/registry. It cannot be stolen by a later attempt's shared watch.
    published: watch::Sender<Option<SessionStatus>>,
    task: Mutex<Option<tokio::task::JoinHandle<()>>>,
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
                task: Mutex::new(None),
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
        self.phase.send_replace(Phase::Finished(outcome));
    }
    pub fn spawn(
        self: &Arc<Self>,
        future: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> AdapterResult<()> {
        // Holding only the private handle mutex prevents completion-before-
        // installation. No Store/registry/admission lock or await is involved.
        let mut task = self.task.lock().map_err(|_| {
            self.preparation.failed(failure(
                ErrorKind::StateFailure,
                "native attempt task owner poisoned",
            ))
        })?;
        if task.is_some() {
            return Err(self.preparation.failed(failure(
                ErrorKind::StateConflict,
                "native attempt task already owned",
            )));
        }
        *task = Some(tokio::spawn(future));
        Ok(())
    }
    pub fn release_task(&self) {
        // Releasing one's own JoinHandle never aborts it or touches a replacement.
        match self.task.lock() {
            Ok(mut task) => {
                task.take();
            }
            Err(poisoned) => {
                poisoned.into_inner().take();
            }
        }
    }
    pub async fn wait_finished(&self) -> AdapterResult<Outcome> {
        let mut receiver = self.subscribe();
        loop {
            if let Phase::Finished(outcome) = receiver.borrow_and_update().clone() {
                return Ok(outcome);
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
mod tests {
    use super::*;

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
