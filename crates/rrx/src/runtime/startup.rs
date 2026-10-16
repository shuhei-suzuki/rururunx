//! S custody and startup observations. Journal receipts are never admission authority.
mod path;
mod probe;
use crate::{
    config::Config,
    execution::{RuntimeOwner, claude_wire::DecoderChecked, process::RetainedRawProcess},
    state::startup::StartupIntent,
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupState {
    NotStarted,
    Pending,
    Refused,
    Unknown,
    Stopped,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeState {
    Pending,
    Passed,
    Refused,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupRefusal {
    Installation,
    PrivatePath,
    ProgramIdentity,
    Journal,
    CodexSchemaUnavailable,
    CodexHandshakeUnavailable,
    Protocol,
    Stopped,
    Budget,
    Deadline,
    UnitUnavailable,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartupStep {
    pub step: String,
    pub state: ProbeState,
    pub reason: Option<StartupRefusal>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartupAlias {
    pub alias: String,
    pub provider: String,
    pub steps: Vec<StartupStep>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartupStatus {
    pub state: StartupState,
    pub reason: Option<StartupRefusal>,
    pub aliases: Vec<StartupAlias>,
    pub old_epoch_unknown: u64,
}
struct TaskGuard(Arc<Startup>);
impl Drop for TaskGuard {
    fn drop(&mut self) {
        if let Ok(mut s) = self.0.status.lock()
            && s.state == StartupState::Pending
        {
            s.state = StartupState::Unknown;
        }
    }
}
struct ChildCustody {
    raw: Mutex<RetainedRawProcess>,
}
impl ChildCustody {
    fn revoke(&self) {
        if let Ok(mut raw) = self.raw.lock() {
            raw.close_stdin();
            raw.hygiene();
        }
    }
}
impl Drop for ChildCustody {
    fn drop(&mut self) {
        self.revoke();
    }
}
/// The Runtime registers this slot before releasing start's short admission lock.
/// Tasks own this custody, never the Runtime itself, across I/O and caller Drop.
pub(super) struct Startup {
    owner: Arc<RuntimeOwner>,
    cancelled: AtomicBool,
    status: Mutex<StartupStatus>,
    children: Mutex<Vec<Arc<ChildCustody>>>,
    task: tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
}
pub(super) type StartupInstallation = (
    uuid::Uuid,
    Arc<DecoderChecked>,
    Vec<(String, String, std::path::PathBuf)>,
);
struct AliasPlan {
    alias: String,
    provider: String,
    program: probe::Program,
    directory: path::AliasPath,
    intents: Vec<(String, StartupIntent, Value)>,
    model: Option<String>,
    effort: Option<String>,
}
impl Startup {
    pub(super) fn new(owner: Arc<RuntimeOwner>) -> Arc<Self> {
        Arc::new(Self {
            owner,
            cancelled: AtomicBool::new(false),
            status: Mutex::new(StartupStatus {
                state: StartupState::NotStarted,
                reason: None,
                aliases: Vec::new(),
                old_epoch_unknown: 0,
            }),
            children: Mutex::default(),
            task: tokio::sync::Mutex::new(None),
        })
    }
    pub(super) async fn register(
        self: &Arc<Self>,
        config: Config,
        installation: Option<StartupInstallation>,
    ) -> tokio::sync::oneshot::Sender<()> {
        let end = Instant::now() + Duration::from_secs(600);
        let (release, ready) = tokio::sync::oneshot::channel();
        self.status.lock().unwrap().state = StartupState::Pending;
        let custody = self.clone();
        *self.task.lock().await = Some(tokio::spawn(async move {
            let _guard = TaskGuard(custody.clone());
            if ready.await.is_err() {
                return;
            }
            // Time is fixed at custody registration, not refreshed by messages.
            custody.run(config, installation, end).await;
        }));
        release
    }
    fn fail(&self, reason: StartupRefusal) {
        if let Ok(mut s) = self.status.lock() {
            s.state = if reason == StartupRefusal::Journal {
                StartupState::Unknown
            } else if reason == StartupRefusal::Stopped {
                StartupState::Stopped
            } else {
                StartupState::Refused
            };
            s.reason = Some(reason);
        }
    }
    pub(super) fn is_revoked(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
    pub(super) fn revoke(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        if let Ok(children) = self.children.lock() {
            for child in children.iter() {
                child.revoke();
            }
        }
    }
    pub(super) fn snapshot(&self) -> Result<StartupStatus> {
        let mut s = self
            .status
            .lock()
            .map_err(|_| anyhow::anyhow!("startup status poisoned"))?
            .clone();
        s.old_epoch_unknown = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .old_startup_unknown(&self.owner)?;
        if self.is_revoked() && s.state == StartupState::Pending {
            s.state = StartupState::Stopped;
        }
        Ok(s)
    }
    /// U3 deliberately has no producer of the all-alias pass. U4 must supply
    /// fresh Schema and Handshake evidence before this boundary can open.
    pub(super) fn require_qualified(&self) -> Result<()> {
        anyhow::bail!("startup Codex Schema/Handshake unavailable (U4)")
    }
    pub(super) async fn shutdown(&self) -> Result<()> {
        self.revoke();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let mut slot = self.task.lock().await;
            let joined = slot.as_ref().is_none_or(|h| h.is_finished());
            if joined
                && let Some(handle) = slot.take()
                && handle.await.is_err()
            {
                let mut s = self
                    .status
                    .lock()
                    .map_err(|_| anyhow::anyhow!("startup status poisoned"))?;
                s.state = StartupState::Unknown;
            }
            drop(slot);
            let reaped = self
                .children
                .lock()
                .map_err(|_| anyhow::anyhow!("startup custody poisoned"))?
                .iter()
                .all(|child| {
                    let Ok(mut raw) = child.raw.lock() else {
                        return false;
                    };
                    raw.hygiene();
                    raw.leader_reaped() || !raw.has_child() || raw.reap().ok().flatten().is_some()
                });
            if joined && reaped {
                return Ok(());
            }
            ensure!(
                Instant::now() < deadline,
                super::stop::ShutdownPending::at(super::stop::ShutdownSite::StartupProbes)
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
    async fn run(
        self: &Arc<Self>,
        config: Config,
        installation: Option<StartupInstallation>,
        end: Instant,
    ) {
        let paths = match path::Paths::new(self.owner.clone()) {
            Ok(p) => p,
            Err(_) => {
                self.fail(StartupRefusal::PrivatePath);
                return;
            }
        };
        let Some((installation, checked, ports)) = installation else {
            self.fail(StartupRefusal::Installation);
            return;
        };
        if ports.len() > 32 {
            self.fail(StartupRefusal::Budget);
            return;
        }
        let mut plans = Vec::new();
        for (alias, provider, program) in ports {
            let directory = match paths.alias(installation, &alias) {
                Ok(p) => p,
                Err(_) => {
                    self.fail(StartupRefusal::PrivatePath);
                    return;
                }
            };
            let program = match probe::Program::read(&program) {
                Ok(p) => p,
                Err(_) => {
                    self.fail(StartupRefusal::ProgramIdentity);
                    return;
                }
            };
            let plan = (|| -> Result<AliasPlan> {
                let agent = config.agents.get(&alias).context("startup alias absent")?;
                let steps = if provider == "codex" {
                    vec!["version", "schema", "handshake"]
                } else {
                    vec!["version", "handshake"]
                };
                let declaration =
                    crate::execution::native_result::digest(&serde_json::to_vec(agent)?);
                let body = json!({"instance":self.owner.instance_id(),"provider":provider,"declaration":declaration,"model":agent.model,"effort":agent.effort,"program":program});
                let mut intents = Vec::new();
                for step in steps {
                    let intent = self
                        .owner
                        .store
                        .lock()
                        .map_err(|_| anyhow::anyhow!("state poisoned"))?
                        .startup_intent(&self.owner, installation, &alias, step, &body)?;
                    intents.push((step.into(), intent, body.clone()));
                }
                Ok(AliasPlan {
                    alias,
                    provider,
                    program,
                    directory,
                    intents,
                    model: agent.model.clone(),
                    effort: agent.effort.clone(),
                })
            })();
            match plan {
                Ok(p) => plans.push(p),
                Err(_) => {
                    self.fail(StartupRefusal::Journal);
                    return;
                }
            }
        }
        self.status.lock().unwrap().aliases = plans
            .iter()
            .map(|p| StartupAlias {
                alias: p.alias.clone(),
                provider: p.provider.clone(),
                steps: p
                    .intents
                    .iter()
                    .map(|(step, _, _)| StartupStep {
                        step: step.clone(),
                        state: ProbeState::Pending,
                        reason: None,
                    })
                    .collect(),
            })
            .collect();
        let mut aggregate = 0usize;
        let mut halted = None;
        for (index, plan) in plans.iter().enumerate() {
            let mut prior_pass = true;
            for (i, (step, intent, body)) in plan.intents.iter().enumerate() {
                let observation = if self.is_revoked() {
                    probe::Observation::not_spawned(StartupRefusal::Stopped)
                } else if let Some(reason) = halted {
                    probe::Observation::not_spawned(reason)
                } else if step == "schema" {
                    probe::Observation::not_spawned(StartupRefusal::CodexSchemaUnavailable)
                } else if plan.provider == "codex" && step == "handshake" {
                    probe::Observation::not_spawned(StartupRefusal::CodexHandshakeUnavailable)
                } else if !prior_pass {
                    probe::Observation::not_spawned(StartupRefusal::Protocol)
                } else {
                    probe::run(self, plan, step, &checked, end, &mut aggregate).await
                };
                if matches!(
                    observation.reason,
                    Some(StartupRefusal::Budget | StartupRefusal::Deadline)
                ) {
                    halted = observation.reason;
                }
                prior_pass &= observation.pass;
                let mut settled = body.clone();
                settled["observation"] = serde_json::to_value(&observation).unwrap();
                let saved = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))
                    .and_then(|mut store| {
                        store.settle_startup(&self.owner, intent, observation.known, &settled)
                    })
                    .is_ok();
                let mut status = self.status.lock().unwrap();
                status.aliases[index].steps[i] = StartupStep {
                    step: step.clone(),
                    state: if !saved || !observation.known {
                        ProbeState::Unknown
                    } else if observation.pass {
                        ProbeState::Passed
                    } else {
                        ProbeState::Refused
                    },
                    reason: if saved {
                        observation.reason
                    } else {
                        Some(StartupRefusal::Journal)
                    },
                };
                if !saved {
                    status.state = StartupState::Unknown;
                    status.reason = Some(StartupRefusal::Journal);
                    return;
                }
            }
        }
        if let Ok(mut status) = self.status.lock() {
            let unknown = status
                .aliases
                .iter()
                .flat_map(|a| &a.steps)
                .find(|s| s.state == ProbeState::Unknown)
                .map(|s| s.reason);
            if let Some(reason) = unknown {
                status.state = StartupState::Unknown;
                status.reason = reason;
                return;
            }
        }
        // U3 is an intentionally closed intermediate unit, including a
        // configuration containing only Claude aliases. No journal replay gate.
        self.fail(if self.is_revoked() {
            StartupRefusal::Stopped
        } else {
            StartupRefusal::UnitUnavailable
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// S-4 classifies a retained unjoined S task as StartupProbes pending.
    #[tokio::test]
    async fn unjoined_startup_task_is_not_shutdown_completion() {
        let dir = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        let startup = Startup::new(owner);
        let (release, wait) = tokio::sync::oneshot::channel::<()>();
        *startup.task.lock().await = Some(tokio::spawn(async move {
            let _ = wait.await;
        }));
        let error = startup.shutdown().await.unwrap_err();
        assert_eq!(
            super::super::stop::pending_site(&error),
            Some(super::super::stop::ShutdownSite::StartupProbes)
        );
        release.send(()).unwrap();
        startup.shutdown().await.unwrap();
    }
    /// S-4 shutdown requires a genuine retained leader reap in addition to task join.
    #[tokio::test]
    async fn startup_stop_reaps_the_retained_leader_before_completion() {
        let dir = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        owner
            .store
            .lock()
            .unwrap()
            .startup_intent(
                &owner,
                uuid::Uuid::new_v4(),
                "custody",
                "version",
                &json!({"instance":owner.instance_id()}),
            )
            .unwrap();
        let startup = Startup::new(owner);
        let custody = Arc::new(ChildCustody {
            raw: Mutex::new(RetainedRawProcess::default()),
        });
        startup.children.lock().unwrap().push(custody.clone());
        let mut command = tokio::process::Command::new("/bin/sleep");
        command.arg("30").process_group(0).kill_on_drop(true);
        let child = rrx_spawn_hygiene::spawn_tokio(command).unwrap();
        custody.raw.lock().unwrap().adopt(child);
        custody.raw.lock().unwrap().qualify().unwrap();
        startup.shutdown().await.unwrap();
        assert!(
            custody.raw.lock().unwrap().leader_reaped(),
            "S-4: shutdown completed without known leader reap"
        );
    }
    /// S-4 task panic leaves pending intents for epoch fencing and never publishes a pass.
    #[tokio::test]
    async fn startup_task_panic_keeps_intent_and_marks_unknown() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join("state.db");
        let owner = RuntimeOwner::open(&state).unwrap();
        let startup = Startup::new(owner.clone());
        owner
            .store
            .lock()
            .unwrap()
            .startup_intent(
                &owner,
                uuid::Uuid::new_v4(),
                "worker",
                "version",
                &json!({"instance":owner.instance_id()}),
            )
            .unwrap();
        startup.status.lock().unwrap().state = StartupState::Pending;
        let retained = startup.clone();
        let handle = tokio::spawn(async move {
            let _guard = TaskGuard(retained);
            panic!("synthetic task panic");
        });
        assert!(handle.await.is_err());
        assert_eq!(startup.snapshot().unwrap().state, StartupState::Unknown);
        assert!(startup.require_qualified().is_err());
        let read = crate::state::current_test_writer(&state).unwrap();
        assert_eq!(
            read.query_row("SELECT state FROM runtime_startup_probes", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "pending"
        );
        drop(read);
        drop(startup);
        drop(owner);
        let next = RuntimeOwner::open(&state).unwrap();
        assert_eq!(
            next.store
                .lock()
                .unwrap()
                .old_startup_unknown(&next)
                .unwrap(),
            1
        );
    }
}
