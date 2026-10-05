//! Best-effort historical backlog. Work, artifacts and phase progress are never
//! changed by a reclamation observation. No scope-empty/death guarantee.
use super::*;
use anyhow::{Context, Result};
use rrx_process_tracker::{Cookie, Limits, Termination};
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};

/// The worker holds the owner only during a sweep. Dropping it aborts its
/// scheduler; an OS worker carries neither the owner lock nor SQLite state.
pub struct CleanupWorker {
    task: tokio::task::JoinHandle<()>,
}
impl CleanupWorker {
    pub fn start(owner: &Arc<RuntimeOwner>) -> Result<Self> {
        let handle = tokio::runtime::Handle::try_current().context("cleanup needs Tokio")?;
        let owner = Arc::downgrade(owner);
        let task = handle.spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                let Some(owner) = owner.upgrade() else {
                    break;
                };
                // Failed/abandoned claims retain their bounded retry deadline.
                // A background cleanup error never transitions a Task.
                let _ = CleanupService::new(owner).sweep_once().await;
            }
        });
        Ok(Self { task })
    }
    pub async fn shutdown(mut self) {
        self.task.abort();
        let _ = (&mut self.task).await;
    }
}

impl Drop for CleanupWorker {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub struct CleanupService {
    owner: Arc<RuntimeOwner>,
    gate: tokio::sync::Mutex<()>,
}
impl CleanupService {
    pub fn new(owner: Arc<RuntimeOwner>) -> Self {
        Self {
            owner,
            gate: tokio::sync::Mutex::new(()),
        }
    }
    /// At most four already-closed units per call. Finalization is never retired
    /// merely to permit this sweep. Docker/filesystem release remains pending.
    pub async fn sweep_once(&self) -> Result<Vec<CleanupObservation>> {
        let _gate = self.gate.lock().await;
        let ids = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .due_execution_cleanup(crate::domain::now_ms(), 4)?;
        let mut observations = Vec::new();
        for id in ids {
            let claim = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .claim_execution_cleanup(id, self.owner.epoch, crate::domain::now_ms())?;
            let Some(claim) = claim else {
                continue;
            };
            // Only the opaque cookie is sent to the blocking OS worker. No owner
            // lock, SQLite connection, environment dump or Task mutation travels.
            let cookie = claim.unit.cookie.clone();
            let scanned = tokio::task::spawn_blocking(move || {
                Cookie::new(&cookie)
                    .and_then(|cookie| rrx_process_tracker::discover(&cookie, Limits::default()))
            })
            .await
            .unwrap_or_else(|_| Err(std::io::Error::other("cleanup discovery worker ended")));
            let mut observation = CleanupObservation {
                actions: Vec::new(),
                unit_id: id,
                at: crate::domain::now_ms(),
                outcome: CleanupOutcome::Unknown,
                coverage: BTreeMap::from([
                    (
                        "cookie".into(),
                        "observation only; not complete membership".into(),
                    ),
                    ("docker".into(), "not yet reconciled".into()),
                    ("paths_ports".into(), "retained/quarantined".into()),
                ]),
                remaining: Vec::new(),
                errors: Vec::new(),
            };
            match scanned {
                Err(_) => observation
                    .errors
                    .push("cookie_discovery_unavailable".into()),
                Ok(discovery) => {
                    observation.coverage.insert(
                        "cookie_scan".into(),
                        format!(
                            "inspected={},same_user={},limited={},identity_unavailable={},environment_unavailable={},handle_unavailable={},identity_changed={}",
                            discovery.coverage.inspected,
                            discovery.coverage.same_user,
                            discovery.coverage.limited(),
                            discovery.coverage.identity_unavailable,
                            discovery.coverage.environment_unavailable,
                            discovery.coverage.handle_unavailable,
                            discovery.coverage.identity_changed
                        ),
                    );
                    if discovery.processes.len() > 960 {
                        observation
                            .errors
                            .push("cookie_action_limit_exceeded".into());
                    }
                    observation.coverage.insert(
                        "cookie_actions".into(),
                        format!(
                            "matches={},max=960,poll_budget_ms=500",
                            discovery.processes.len()
                        ),
                    );
                    let started = Instant::now();
                    for process in discovery.processes.into_iter().take(960) {
                        let identity = process.identity();
                        let target = format!(
                            "pid:{}@{}:{}",
                            identity.pid, identity.birth.0, identity.birth.1
                        );
                        if started.elapsed() >= Duration::from_millis(500) {
                            record_action(
                                &mut observation,
                                CleanupAction {
                                    target,
                                    action: CleanupActionKind::ProcessTerminate,
                                    outcome: CleanupActionOutcome::Skipped,
                                    confirmation: CleanupConfirmation::NotAttempted,
                                },
                            );
                            continue;
                        }
                        let sent = process.terminate();
                        let mut observed = if sent == Termination::AlreadyExited {
                            CleanupConfirmation::Exited
                        } else {
                            exit_confirmation(process.exited())
                        };
                        if sent == Termination::Sent {
                            for _ in 0..5 {
                                if observed != CleanupConfirmation::StillObserved {
                                    break;
                                }
                                tokio::time::sleep(Duration::from_millis(20)).await;
                                observed = exit_confirmation(process.exited());
                            }
                        }
                        record_action(
                            &mut observation,
                            CleanupAction {
                                target,
                                action: CleanupActionKind::ProcessTerminate,
                                outcome: termination_outcome(sent),
                                confirmation: observed,
                            },
                        );
                    }
                    if !observation.remaining.is_empty() {
                        observation.outcome = CleanupOutcome::Leftovers;
                    }
                }
            }
            observation.at = crate::domain::now_ms();
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .finish_execution_cleanup(&claim, &observation)?;
            observations.push(observation);
        }
        Ok(observations)
    }
}

fn termination_outcome(termination: Termination) -> CleanupActionOutcome {
    match termination {
        Termination::Sent => CleanupActionOutcome::Sent,
        Termination::AlreadyExited => CleanupActionOutcome::AlreadyExited,
        Termination::Unsupported => CleanupActionOutcome::Unsupported,
        Termination::Denied => CleanupActionOutcome::Denied,
        Termination::Unknown => CleanupActionOutcome::Unknown,
    }
}
fn exit_confirmation(result: std::io::Result<Option<bool>>) -> CleanupConfirmation {
    match result {
        Ok(Some(true)) => CleanupConfirmation::Exited,
        Ok(Some(false)) => CleanupConfirmation::StillObserved,
        Ok(None) => CleanupConfirmation::Unavailable,
        Err(_) => CleanupConfirmation::Error,
    }
}
fn record_action(observation: &mut CleanupObservation, action: CleanupAction) {
    let error = match action.outcome {
        CleanupActionOutcome::Unsupported => Some("cookie_terminate_unsupported"),
        CleanupActionOutcome::Denied => Some("cookie_terminate_denied"),
        CleanupActionOutcome::Unknown => Some("cookie_terminate_unknown"),
        CleanupActionOutcome::Skipped => Some("cookie_action_budget_exceeded"),
        _ => None,
    };
    for category in [
        error,
        match action.confirmation {
            CleanupConfirmation::Unavailable => Some("cookie_exit_unavailable"),
            CleanupConfirmation::Error => Some("cookie_exit_error"),
            _ => None,
        },
    ]
    .into_iter()
    .flatten()
    {
        if !observation.errors.iter().any(|s| s == category) {
            observation.errors.push(category.into());
        }
    }
    if !matches!(
        action.confirmation,
        CleanupConfirmation::Exited | CleanupConfirmation::Absent
    ) {
        observation.remaining.push(action.target.clone());
    }
    observation.actions.push(action);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::Task,
        execution::{attempts::AttemptManager, results},
    };
    use std::process::{Child, Command, Stdio};

    struct OwnedChild(Child);
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    fn child(cookie: &str, marker: &std::path::Path) -> OwnedChild {
        OwnedChild(Command::new("/usr/bin/python3")
            .args(["-c", "import os,time,pathlib; pathlib.Path(os.environ['RRX_FIXTURE_READY']).write_text('ready'); time.sleep(30)"])
            .env("RRX_PROCESS_COOKIE", cookie).env("RRX_FIXTURE_READY", marker)
            .stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap())
    }
    #[tokio::test]
    async fn historical_cookie_sweep_preserves_retained_result_and_live_sibling() {
        let (dir, owner, task) = results::tests::fixture().await;
        let attempts = AttemptManager::new(owner.clone());
        let (first, _) = attempts
            .prepare(task.id, "codex", "Implement", None)
            .await
            .unwrap();
        let first = owner
            .store
            .lock()
            .unwrap()
            .finish_execution(
                &first.authority(),
                WorkOutcome::Success,
                Disposition::Completed,
            )
            .unwrap();
        let result = results::ResultStore::new(owner.clone());
        let artifact = result
            .capture(
                &first.authority(),
                &first.base_sha,
                BTreeMap::from([("code".into(), first.base_sha.clone())]),
            )
            .await
            .unwrap();
        let marker = dir.path().join("first-ready");
        let mut first_child = child(&first.cookie, &marker);
        let mut sibling = Task::new(
            task.project_id,
            task.goal_id,
            "live sibling".into(),
            "claude".into(),
        );
        owner.store.lock().unwrap().put_task(&mut sibling).unwrap();
        let (second, _) = attempts
            .prepare(sibling.id, "claude", "Implement", None)
            .await
            .unwrap();
        let second_marker = dir.path().join("second-ready");
        let mut second_child = child(&second.cookie, &second_marker);
        tokio::time::timeout(Duration::from_secs(10), async {
            while !marker.exists() || !second_marker.exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        let cleanup = CleanupService::new(owner.clone());
        assert!(cleanup.sweep_once().await.unwrap().is_empty());
        assert!(first_child.0.try_wait().unwrap().is_none());
        let task_version = owner
            .store
            .lock()
            .unwrap()
            .task(task.id)
            .unwrap()
            .unwrap()
            .version;
        let published = result
            .publish(&first.authority(), &artifact, task_version)
            .await
            .unwrap();
        let before_task = owner.store.lock().unwrap().task(task.id).unwrap().unwrap();
        let observation = cleanup.sweep_once().await.unwrap();
        assert_eq!(observation.len(), 1);
        assert_eq!(observation[0].unit_id, first.id);
        assert_ne!(observation[0].outcome, CleanupOutcome::Reclaimed);
        #[cfg(target_os = "macos")]
        {
            assert_eq!(observation[0].outcome, CleanupOutcome::Leftovers);
            let action = observation[0]
                .actions
                .iter()
                .find(|a| {
                    a.target
                        .starts_with(&format!("pid:{}@", first_child.0.id()))
                })
                .unwrap();
            assert_eq!(action.outcome, CleanupActionOutcome::Unsupported);
            assert_eq!(action.confirmation, CleanupConfirmation::Unavailable);
            assert!(
                observation[0]
                    .errors
                    .iter()
                    .any(|s| s == "cookie_terminate_unsupported")
            );
            assert!(
                observation[0]
                    .errors
                    .iter()
                    .any(|s| s == "cookie_exit_unavailable")
            );
            assert!(first_child.0.try_wait().unwrap().is_none());
        }
        #[cfg(target_os = "linux")]
        {
            tokio::time::timeout(Duration::from_secs(10), async {
                while first_child.0.try_wait().unwrap().is_none() {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await
            .unwrap();
        }
        assert!(second_child.0.try_wait().unwrap().is_none());
        {
            let store = owner.store.lock().unwrap();
            assert_eq!(
                store.execution_unit(first.id).unwrap().work,
                Some(WorkOutcome::Success)
            );
            let persisted = store.execution_cleanup_observations(first.id, 1).unwrap();
            assert_eq!(
                serde_json::to_value(&persisted[0]).unwrap(),
                serde_json::to_value(&observation[0]).unwrap()
            );
            assert_eq!(
                serde_json::to_value(store.result_artifact(published.id).unwrap()).unwrap(),
                serde_json::to_value(published).unwrap()
            );
            assert_eq!(
                serde_json::to_value(store.task(task.id).unwrap().unwrap()).unwrap(),
                serde_json::to_value(before_task).unwrap()
            );
            assert!(
                store
                    .validate_execution(&second.authority(), true, false)
                    .is_ok()
            );
            assert!(
                store
                    .execution_leases(first.id)
                    .unwrap()
                    .iter()
                    .all(|l| l.state == LeaseState::Quarantined)
            );
            assert!(
                store
                    .execution_leases(second.id)
                    .unwrap()
                    .iter()
                    .all(|l| l.state == LeaseState::Reserved)
            );
        }
        result.verify(&artifact).await.unwrap();
        assert!(cleanup.sweep_once().await.unwrap().is_empty());
    }
    #[tokio::test]
    async fn cleanup_worker_does_not_retain_an_idle_owner_lock() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join("state.db");
        let owner = RuntimeOwner::open(&state).unwrap();
        let weak = Arc::downgrade(&owner);
        let epoch = owner.epoch();
        let worker = CleanupWorker::start(&owner).unwrap();
        drop(owner);
        assert!(weak.upgrade().is_none());
        let successor = RuntimeOwner::open(&state).unwrap();
        assert!(successor.epoch() > epoch);
        worker.shutdown().await;
    }
    #[test]
    fn cookie_receipts_distinguish_actions_and_uncertain_exit_without_raw_errors() {
        let mut observation = CleanupObservation {
            unit_id: UnitId::new(),
            at: 1,
            outcome: CleanupOutcome::Unknown,
            coverage: BTreeMap::new(),
            remaining: Vec::new(),
            errors: Vec::new(),
            actions: Vec::new(),
        };
        for (index, (sent, result, expected)) in [
            (
                Termination::Sent,
                Ok(Some(true)),
                CleanupConfirmation::Exited,
            ),
            (
                Termination::Denied,
                Ok(Some(false)),
                CleanupConfirmation::StillObserved,
            ),
            (
                Termination::Unsupported,
                Ok(None),
                CleanupConfirmation::Unavailable,
            ),
            (
                Termination::Unknown,
                Err(std::io::Error::other(
                    "fixture raw diagnostic must not persist",
                )),
                CleanupConfirmation::Error,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            record_action(
                &mut observation,
                CleanupAction {
                    target: format!("pid:{}@1:0", index + 1),
                    action: CleanupActionKind::ProcessTerminate,
                    outcome: termination_outcome(sent),
                    confirmation: exit_confirmation(result),
                },
            );
            assert_eq!(observation.actions[index].confirmation, expected);
        }
        assert_eq!(
            observation.remaining,
            vec!["pid:2@1:0", "pid:3@1:0", "pid:4@1:0"]
        );
        for category in [
            "cookie_terminate_denied",
            "cookie_terminate_unsupported",
            "cookie_terminate_unknown",
            "cookie_exit_unavailable",
            "cookie_exit_error",
        ] {
            assert!(observation.errors.iter().any(|s| s == category));
        }
        let serialized = serde_json::to_string(&observation).unwrap();
        assert!(!serialized.contains("raw diagnostic"));
        let restored: CleanupObservation = serde_json::from_str(&serialized).unwrap();
        assert_eq!(restored.actions[1].outcome, CleanupActionOutcome::Denied);
        assert_eq!(restored.actions[3].confirmation, CleanupConfirmation::Error);
        let mut legacy = serde_json::to_value(restored).unwrap();
        legacy.as_object_mut().unwrap().remove("actions");
        assert!(
            serde_json::from_value::<CleanupObservation>(legacy)
                .unwrap()
                .actions
                .is_empty()
        );
    }
}
