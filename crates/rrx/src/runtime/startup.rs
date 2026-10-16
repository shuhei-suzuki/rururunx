//! S custody and startup observations. Journal receipts are never admission authority.
mod path;
mod probe;
use crate::{
    config::Config,
    execution::{RuntimeOwner, claude_wire::DecoderChecked, process::RetainedRawProcess},
    state::startup::StartupIntent,
};
use anyhow::{Result, ensure};
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
    Passed,
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
    Configuration,
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
    cancelled: Arc<AtomicBool>,
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
            cancelled: Arc::new(AtomicBool::new(false)),
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
    fn intent(
        &self,
        installation: uuid::Uuid,
        alias: &str,
        step: &str,
        body: &Value,
    ) -> std::result::Result<StartupIntent, StartupRefusal> {
        probe::Observation::validate_body(body).map_err(|_| StartupRefusal::Configuration)?;
        self.owner
            .store
            .lock()
            .map_err(|_| StartupRefusal::Journal)?
            .startup_intent(&self.owner, installation, alias, step, body)
            .map_err(|_| StartupRefusal::Journal)
    }
    /// Planning has not spawned anything: settle every committed original by CAS.
    fn refuse_plans(&self, plans: &[AliasPlan], reason: StartupRefusal) {
        let mut saved = true;
        for plan in plans {
            for (_, intent, body) in &plan.intents {
                let mut settled = body.clone();
                settled["observation"] =
                    serde_json::to_value(probe::Observation::not_spawned(reason)).unwrap();
                saved &= self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))
                    .and_then(|mut store| store.settle_startup(&self.owner, intent, true, &settled))
                    .is_ok();
            }
        }
        self.fail(if saved {
            reason
        } else {
            StartupRefusal::Journal
        });
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
    /// S never substitutes for a fresh Preparing actor's P observations.
    pub(super) fn require_qualified(&self) -> Result<()> {
        ensure!(
            !self.is_revoked() && self.snapshot()?.state == StartupState::Passed,
            "startup all-alias qualification unavailable"
        );
        anyhow::bail!(FreshPreparationUnavailable)
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
                    self.refuse_plans(&plans, StartupRefusal::PrivatePath);
                    return;
                }
            };
            let program = match probe::Program::read(&program, self.cancelled.clone(), end).await {
                Ok(p) => p,
                Err(reason) => {
                    self.refuse_plans(&plans, reason);
                    return;
                }
            };
            let plan = (|| -> std::result::Result<(), StartupRefusal> {
                let agent = config
                    .agents
                    .get(&alias)
                    .ok_or(StartupRefusal::Configuration)?;
                let steps = if provider == "codex" {
                    vec!["version", "schema", "handshake"]
                } else {
                    vec!["version", "handshake"]
                };
                let declaration = crate::execution::native_result::digest(
                    &serde_json::to_vec(agent).map_err(|_| StartupRefusal::Configuration)?,
                );
                let body = json!({"instance":self.owner.instance_id(),"provider":provider,"declaration":declaration,"model":agent.model,"effort":agent.effort,"program":program});
                plans.push(AliasPlan {
                    alias,
                    provider,
                    program,
                    directory,
                    intents: Vec::new(),
                    model: agent.model.clone(),
                    effort: agent.effort.clone(),
                });
                let plan = plans.last_mut().unwrap();
                for step in steps {
                    let intent = self.intent(installation, &plan.alias, step, &body)?;
                    plan.intents.push((step.into(), intent, body.clone()));
                }
                Ok(())
            })();
            if let Err(reason) = plan {
                self.refuse_plans(&plans, reason);
                return;
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
            let mut contract = None;
            for (i, (step, intent, body)) in plan.intents.iter().enumerate() {
                let observation = if self.is_revoked() {
                    probe::Observation::not_spawned(StartupRefusal::Stopped)
                } else if let Some(reason) = halted {
                    probe::Observation::not_spawned(reason)
                } else if !prior_pass {
                    probe::Observation::not_spawned(StartupRefusal::Protocol)
                } else {
                    probe::run(
                        self,
                        plan,
                        step,
                        &checked,
                        end,
                        &mut aggregate,
                        &mut contract,
                    )
                    .await
                };
                if matches!(
                    observation.reason,
                    Some(
                        StartupRefusal::Budget | StartupRefusal::Deadline | StartupRefusal::Stopped
                    )
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
        let passed = self
            .status
            .lock()
            .unwrap()
            .aliases
            .iter()
            .all(|a| a.steps.iter().all(|s| s.state == ProbeState::Passed));
        if !passed || self.is_revoked() {
            self.fail(if self.is_revoked() {
                StartupRefusal::Stopped
            } else {
                StartupRefusal::Protocol
            });
            return;
        }
        for plan in &plans {
            if plan.directory.cleanup().is_err() {
                self.fail(StartupRefusal::PrivatePath);
                return;
            }
        }
        if paths.cleanup().is_err() {
            self.fail(StartupRefusal::PrivatePath);
            return;
        }
        let mut status = self.status.lock().unwrap();
        if self.is_revoked() {
            status.state = StartupState::Stopped;
            status.reason = Some(StartupRefusal::Stopped);
        } else {
            status.state = StartupState::Passed;
            status.reason = None;
        }
    }
}

#[derive(Debug)]
pub(super) struct FreshPreparationUnavailable;
impl std::fmt::Display for FreshPreparationUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fresh Codex preparation unavailable (U5)")
    }
}
impl std::error::Error for FreshPreparationUnavailable {}

#[cfg(test)]
mod tests {
    use super::*;
    /// Planning refusal settles all committed unspawned originals without consuming unknown capacity on restart.
    #[tokio::test]
    async fn planning_refusal_settles_earlier_aliases_before_restart() {
        use std::os::unix::fs::PermissionsExt;
        for failure in [
            "unreadable",
            "model",
            "undefined-alias",
            "oversized",
            "directory",
            "private-path",
            "partial-journal",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let state = dir.path().join("state.db");
            let owner = RuntimeOwner::open(&state).unwrap();
            let startup = Startup::new(owner.clone());
            let first = dir.path().join("first");
            let second = dir.path().join("second");
            std::fs::write(&first, b"never spawned").unwrap();
            std::fs::write(&second, b"never spawned").unwrap();
            let mut config = Config::default();
            for alias in ["a", "b"] {
                config
                    .agents
                    .insert(alias.into(), crate::config::AgentConfig::default());
            }
            let reason = match failure {
                "unreadable" => {
                    std::fs::set_permissions(&second, std::fs::Permissions::from_mode(0o111))
                        .unwrap();
                    StartupRefusal::ProgramIdentity
                }
                "model" => {
                    config.agents.get_mut("b").unwrap().model = Some("m".repeat(4096));
                    StartupRefusal::Configuration
                }
                "undefined-alias" => {
                    config.agents.remove("b");
                    StartupRefusal::Configuration
                }
                "oversized" => {
                    std::fs::File::create(&second)
                        .unwrap()
                        .set_len(512 * 1024 * 1024 + 1)
                        .unwrap();
                    StartupRefusal::ProgramIdentity
                }
                "directory" => {
                    std::fs::remove_file(&second).unwrap();
                    std::fs::create_dir(&second).unwrap();
                    StartupRefusal::ProgramIdentity
                }
                "private-path" => StartupRefusal::PrivatePath,
                "partial-journal" => StartupRefusal::Journal,
                _ => unreachable!(),
            };
            let checked = Arc::new(crate::execution::claude_wire::decoder_selfcheck().unwrap());
            let installation = uuid::Uuid::new_v4();
            let second_alias = if failure == "private-path" { "a" } else { "b" };
            if failure == "partial-journal" {
                // An independently settled original refuses the second step
                // while the first step of this alias has already committed.
                let existing = startup
                    .intent(installation, "b", "handshake", &json!({}))
                    .unwrap();
                owner
                    .store
                    .lock()
                    .unwrap()
                    .settle_startup(&owner, &existing, true, &json!({}))
                    .unwrap();
            }
            startup
                .run(
                    config,
                    Some((
                        installation,
                        checked,
                        vec![
                            ("a".into(), "claude".into(), first),
                            (second_alias.into(), "claude".into(), second),
                        ],
                    )),
                    Instant::now() + Duration::from_secs(10),
                )
                .await;
            let status = startup.snapshot().unwrap();
            assert_eq!(status.reason, Some(reason), "{failure}");
            assert_eq!(
                status.state,
                if reason == StartupRefusal::Journal {
                    StartupState::Unknown
                } else {
                    StartupState::Refused
                },
                "{failure}"
            );
            let wire = serde_json::to_value(&status).unwrap();
            let decoded: StartupStatus = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(decoded.state, status.state);
            assert_eq!(decoded.reason, status.reason);
            if reason == StartupRefusal::Configuration {
                assert_eq!(wire["state"], "refused");
                assert_eq!(wire["reason"], "configuration");
            }
            assert!(
                startup.children.lock().unwrap().is_empty(),
                "planning refusal spawned a child"
            );
            let read = crate::state::current_test_writer(&state).unwrap();
            let (pending, confirmed, attempted): (u64, u64, u64) = read.query_row(
                "SELECT coalesce(sum(state='pending'),0),coalesce(sum(state='confirmed'),0),coalesce(sum(json_extract(body,'$.observation.attempted')=1),0) FROM runtime_startup_probes",
                [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)),
            ).unwrap();
            assert_eq!(pending, 0, "unspawned intent remained pending: {failure}");
            assert!(
                confirmed >= 2,
                "earlier alias intents were not observed: {failure}"
            );
            assert_eq!(attempted, 0);
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
                0,
                "unspawned intent fenced unknown: {failure}"
            );
        }
    }
    /// A planning refusal cannot claim a known result when its committed original fails exact CAS.
    #[tokio::test]
    async fn planning_refusal_cas_miss_is_unknown_journal() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join("state.db");
        let owner = RuntimeOwner::open(&state).unwrap();
        let startup = Startup::new(owner.clone());
        let installation = uuid::Uuid::new_v4();
        let paths = path::Paths::new(owner).unwrap();
        let program = dir.path().join("program");
        std::fs::write(&program, b"never spawned").unwrap();
        let body = json!({"instance":startup.owner.instance_id()});
        let intent = startup.intent(installation, "a", "version", &body).unwrap();
        let plan = AliasPlan {
            alias: "a".into(),
            provider: "claude".into(),
            program: probe::Program::read(
                &program,
                startup.cancelled.clone(),
                Instant::now() + Duration::from_secs(10),
            )
            .await
            .unwrap(),
            directory: paths.alias(installation, "a").unwrap(),
            intents: vec![("version".into(), intent, body.clone())],
            model: None,
            effort: None,
        };
        let writer = crate::state::current_test_writer(&state).unwrap();
        assert_eq!(writer.execute("UPDATE runtime_startup_probes SET version=version+1 WHERE alias='a' AND step='version'", []).unwrap(), 1);
        startup.refuse_plans(&[plan], StartupRefusal::Configuration);
        let status = startup.snapshot().unwrap();
        assert_eq!(status.state, StartupState::Unknown);
        assert_eq!(status.reason, Some(StartupRefusal::Journal));
        let row: (String, u64, String) = writer
            .query_row(
                "SELECT state,version,body FROM runtime_startup_probes",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            row,
            ("pending".into(), 2, serde_json::to_string(&body).unwrap())
        );
        assert!(startup.children.lock().unwrap().is_empty());
    }
    /// Intent admission reserves the largest settlement at exactly 4096 bytes and refuses the next byte without a write.
    #[test]
    fn intent_reserves_complete_settlement_at_the_byte_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join("state.db");
        let owner = RuntimeOwner::open(&state).unwrap();
        let startup = Startup::new(owner.clone());
        let mut observation = serde_json::to_value(probe::Observation::not_spawned(
            StartupRefusal::CodexHandshakeUnavailable,
        ))
        .unwrap();
        observation["known"] = json!(false);
        observation["exit"] = json!(i32::MIN);
        observation["bytes"] = json!(usize::MAX);
        observation["version"] = json!("v".repeat(96));
        let mut settled = json!({"model":"", "observation":observation});
        let remaining = 4096 - serde_json::to_vec(&settled).unwrap().len();
        settled["model"] = json!("m".repeat(remaining));
        assert_eq!(serde_json::to_vec(&settled).unwrap().len(), 4096);
        let mut body = settled.clone();
        body.as_object_mut().unwrap().remove("observation");
        let id = uuid::Uuid::new_v4();
        let intent = startup.intent(id, "a", "version", &body).unwrap();
        let mut over = body.clone();
        over["model"] = json!("m".repeat(remaining + 1));
        assert_eq!(
            startup.intent(id, "b", "version", &over).err(),
            Some(StartupRefusal::Configuration),
            "intent accepted without settlement headroom"
        );
        owner
            .store
            .lock()
            .unwrap()
            .settle_startup(&owner, &intent, false, &settled)
            .unwrap();
        let read = crate::state::current_test_writer(&state).unwrap();
        assert_eq!(
            read.query_row("SELECT count(*) FROM runtime_startup_probes", [], |r| r
                .get::<_, u64>(0))
                .unwrap(),
            1
        );
        let stored: String = read
            .query_row("SELECT body FROM runtime_startup_probes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(stored.len(), 4096);
        assert_eq!(serde_json::from_str::<Value>(&stored).unwrap(), settled);
    }
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
