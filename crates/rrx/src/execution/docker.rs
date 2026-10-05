//! Explicit context/engine qualification. No Docker authentication data is read
//! by rrx; the installed CLI retains its normal inherited authentication context.
use super::*;
use anyhow::{Context, Result, ensure};
use std::{collections::BTreeMap, path::Path, sync::Arc, time::Duration};

const VERSION_FORMAT: &str =
    "{{json .Client.Version}}\n{{json .Server.Version}}\n{{json .Server.APIVersion}}";
const ENGINE_FORMAT: &str = "{{json .ID}}";
#[derive(Clone)]
pub(crate) struct DockerTarget {
    pub context: String,
    pub engine: String,
    pub client: String,
    pub server: String,
    pub api: String,
}
impl DockerTarget {
    fn receipt(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("transport".into(), "unix".into()),
            ("context".into(), self.context.clone()),
            ("engine".into(), self.engine.clone()),
            ("client".into(), self.client.clone()),
            ("server".into(), self.server.clone()),
            ("api".into(), self.api.clone()),
        ])
    }
    pub fn recorded(effect: &ManagedEffect) -> Result<Self> {
        ensure!(
            effect.kind == "docker_target" && effect.state == EffectState::Confirmed,
            "Docker target is unqualified"
        );
        let get = |key: &str| {
            effect
                .receipt
                .get(key)
                .cloned()
                .with_context(|| format!("Docker target {key} missing"))
        };
        let target = Self {
            context: get("context")?,
            engine: get("engine")?,
            client: get("client")?,
            server: get("server")?,
            api: get("api")?,
        };
        ensure!(get("transport")? == "unix", "unsupported Docker transport");
        ensure!(
            context_name(&target.context)
                && target.engine.len() == 64
                && target.engine.bytes().all(|b| b.is_ascii_hexdigit()),
            "Docker target metadata invalid"
        );
        validate_versions(&target.client, &target.server, &target.api)?;
        Ok(target)
    }
}
fn context_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}
fn validate_versions(client: &str, server: &str, api: &str) -> Result<()> {
    // Finite protocol profile, not an assertion that every patch was executed.
    ensure!(
        [client, server].iter().all(|v| v.len() <= 64
            && v.starts_with("28.")
            && v.bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'+' | b'-' | b'_')))
            && matches!(api, "1.48" | "1.49" | "1.50" | "1.51"),
        "unsupported Docker version profile (requires client/server 28, API 1.48-1.51)"
    );
    Ok(())
}
fn versions(bytes: &[u8]) -> Result<(String, String, String)> {
    ensure!(bytes.len() <= 1024, "Docker version response exceeds bound");
    let lines = std::str::from_utf8(bytes)?.lines().collect::<Vec<_>>();
    ensure!(lines.len() == 3, "Docker version response malformed");
    let values = lines
        .into_iter()
        .map(serde_json::from_str::<String>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    validate_versions(&values[0], &values[1], &values[2])?;
    Ok((values[0].clone(), values[1].clone(), values[2].clone()))
}
fn engine_digest(bytes: &[u8]) -> Result<String> {
    ensure!(bytes.len() <= 1024, "Docker engine identity exceeds bound");
    let id: String = serde_json::from_slice(bytes)?;
    ensure!(
        !id.is_empty() && id.len() <= 256 && !id.chars().any(char::is_control),
        "Docker engine identity unavailable"
    );
    Ok(results::hex(id.as_bytes()))
}
fn args(context: &str, command: &[&str]) -> Vec<String> {
    ["--context", context]
        .into_iter()
        .chain(command.iter().copied())
        .map(str::to_owned)
        .collect()
}
pub(crate) async fn qualify_native(
    owner: Arc<RuntimeOwner>,
    unit: &ExecutionUnit,
) -> Result<DockerTarget> {
    qualify_inner(owner, unit, None).await
}
async fn qualify_inner(
    owner: Arc<RuntimeOwner>,
    unit: &ExecutionUnit,
    fixture: Option<&Path>,
) -> Result<DockerTarget> {
    // A DOCKER_HOST-only endpoint cannot be reconstructed by context name.
    // Refuse this finite profile before tool grant instead of silently using a
    // different daemon. No host/TLS/auth file is copied into the ledger.
    ensure!(
        std::env::var_os("DOCKER_HOST").is_none(),
        "managed Docker requires a named context, not DOCKER_HOST"
    );
    let existing = owner
        .store
        .lock()
        .map_err(|_| anyhow::anyhow!("state poisoned"))?
        .managed_effects(unit.id)?
        .into_iter()
        .find(|e| e.kind == "docker_target");
    let previous = existing.as_ref().map(DockerTarget::recorded).transpose()?;
    let intent = if previous.is_none() {
        let id = OperationId::new();
        let mut store = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        let current = store.execution_unit(unit.id)?;
        ensure!(
            current.scope == unit.scope
                && current.owner_epoch == unit.owner_epoch
                && current.generation == unit.generation
                && current.session_id == unit.session_id,
            "Docker qualifier identity retired"
        );
        store.reserve_managed_effect(
            &current.authority(),
            &ManagedEffect {
                id,
                unit_id: unit.id,
                scope: unit.scope.clone(),
                kind: "docker_target".into(),
                idempotency_key: format!("docker-target-{}", unit.id),
                expected_target: format!("docker-target-{}", unit.id),
                state: EffectState::Pending,
                receipt: BTreeMap::new(),
                version: 1,
            },
        )?;
        Some(id)
    } else {
        None
    };
    let mut guard = intent.map(|id| owner::HelperGuard::new(owner.clone(), id));
    let io = git_io::UnitGit::new(owner.clone(), unit, true)?;
    let probe = |a: Vec<String>| {
        let io = &io;
        async move {
            match fixture {
                Some(p) => io.probe_docker_for(&a, p).await,
                None => io.probe_docker(&a).await,
            }
        }
    };
    let observed = tokio::time::timeout(Duration::from_secs(5), async {
        let context = if let Some(previous) = &previous {
            previous.context.clone()
        } else {
            let bytes = probe(vec!["context".into(), "show".into()]).await?;
            let name = std::str::from_utf8(&bytes)?.trim().to_owned();
            ensure!(context_name(&name), "unsupported Docker context name");
            name
        };
        let transport = probe(vec![
            "context".into(),
            "inspect".into(),
            "--format".into(),
            "{{index (split .Endpoints.docker.Host \"://\") 0}}".into(),
            context.clone(),
        ])
        .await?;
        ensure!(
            std::str::from_utf8(&transport)?.trim() == "unix",
            "managed Docker supports only local named Unix-socket contexts"
        );
        let (client, server, api) =
            versions(&probe(args(&context, &["version", "--format", VERSION_FORMAT])).await?)?;
        let engine =
            engine_digest(&probe(args(&context, &["info", "--format", ENGINE_FORMAT])).await?)?;
        if let Some(previous) = &previous {
            ensure!(previous.engine == engine, "Docker engine identity changed");
        }
        Ok::<_, anyhow::Error>(DockerTarget {
            context,
            engine,
            client,
            server,
            api,
        })
    })
    .await
    .context("Docker qualification deadline exceeded")?;
    if let Some(id) = intent {
        owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .reconcile_managed_effect(
                id,
                1,
                if observed.is_ok() {
                    EffectState::Confirmed
                } else {
                    EffectState::Unknown
                },
                observed
                    .as_ref()
                    .map_or_else(|_| BTreeMap::new(), DockerTarget::receipt),
            )?;
        if let Some(guard) = &mut guard {
            guard.disarm();
        }
    }
    observed
}

pub(crate) struct DockerCleanup {
    pub coverage: String,
    pub remaining: Vec<String>,
    pub errors: Vec<String>,
    pub actions: Vec<CleanupAction>,
}
pub(crate) async fn cleanup(
    owner: Arc<RuntimeOwner>,
    claim: &crate::state::CleanupClaim,
) -> DockerCleanup {
    cleanup_for(owner, claim, None).await
}
async fn cleanup_for(
    owner: Arc<RuntimeOwner>,
    claim: &crate::state::CleanupClaim,
    fixture: Option<&Path>,
) -> DockerCleanup {
    let mut report = DockerCleanup {
        coverage: "observed managed containers; no complete delegation proof".into(),
        remaining: vec![],
        errors: vec![],
        actions: vec![],
    };
    let observed = tokio::time::timeout(
        Duration::from_secs(5),
        cleanup_inner(owner, claim, fixture, &mut report),
    )
    .await;
    if !matches!(observed, Ok(Ok(()))) {
        report.coverage = "unavailable or incomplete; no full release confirmation".into();
        report
            .errors
            .push("docker_reconciliation_incomplete".into());
    }
    report.remaining.sort();
    report.remaining.dedup();
    report.errors.sort();
    report.errors.dedup();
    report
}
async fn cleanup_inner(
    owner: Arc<RuntimeOwner>,
    claim: &crate::state::CleanupClaim,
    fixture: Option<&Path>,
    report: &mut DockerCleanup,
) -> Result<()> {
    let effects = owner
        .store
        .lock()
        .map_err(|_| anyhow::anyhow!("state poisoned"))?
        .managed_effects(claim.unit.id)?;
    let creations = effects
        .iter()
        .filter(|e| e.kind == "docker_create")
        .collect::<Vec<_>>();
    if creations.is_empty() {
        report.coverage = "no managed creation recorded; raw Docker delegation uncovered".into();
        return Ok(());
    }
    let target = DockerTarget::recorded(
        effects
            .iter()
            .find(|e| e.kind == "docker_target")
            .context("Docker target missing")?,
    )?;
    let profile = resources::ResourceManager::new(owner.clone()).profile(&claim.unit)?;
    let program = fixture
        .or_else(|| profile.real_tools.get("docker").map(|p| p.as_path()))
        .context("recorded Docker CLI unavailable")?;
    let io = CleanupDocker {
        owner: &owner,
        claim,
        program,
        target: &target,
    };
    io.check_engine().await?;
    let ids = io.inventory("container", &profile.docker_labels).await?;
    ensure!(ids.len() <= 32, "Docker inventory exceeds action bound");
    for id in &ids {
        let selected = io.inspect(id).await;
        let selected = match selected {
            Ok(selected) => selected,
            Err(_) => {
                report.remaining.push(format!("container:{id}"));
                report.errors.push("docker_inspection_unavailable".into());
                continue;
            }
        };
        let own = creations.iter().any(|effect| {
            effect.id.to_string() == selected.operation && effect.expected_target == selected.name
        });
        if !profile
            .docker_labels
            .iter()
            .all(|(k, v)| selected.labels.get(k) == Some(v))
        {
            report.errors.push("docker_label_filter_mismatch".into());
            continue;
        }
        if !own {
            report.remaining.push(format!("container:{id}"));
            report
                .errors
                .push("docker_identity_unregistered_or_changed".into());
            continue;
        }
        if selected.running {
            io.check_engine().await?;
            let killed = io
                .run(
                    "container-kill",
                    &args(&target.context, &["container", "kill", id]),
                )
                .await;
            report.actions.push(CleanupAction {
                target: format!("container:{id}"),
                action: CleanupActionKind::DockerKill,
                outcome: if killed.is_ok() {
                    CleanupActionOutcome::Sent
                } else {
                    CleanupActionOutcome::Unknown
                },
                confirmation: CleanupConfirmation::NotAttempted,
            });
        }
        // Recheck the exact label/name/operation tuple before non-forced removal.
        let stopped = match io.inspect(id).await {
            Ok(stopped) => stopped,
            Err(_) => {
                report.remaining.push(format!("container:{id}"));
                report
                    .errors
                    .push("docker_post_stop_inspection_unavailable".into());
                continue;
            }
        };
        if stopped.running
            || stopped.operation != selected.operation
            || stopped.name != selected.name
            || stopped.labels != selected.labels
        {
            report.remaining.push(format!("container:{id}"));
            report.errors.push("docker_still_running_or_changed".into());
            continue;
        }
        if let Some(action) = report.actions.last_mut().filter(|a| {
            a.target == format!("container:{id}") && a.action == CleanupActionKind::DockerKill
        }) {
            action.confirmation = CleanupConfirmation::Exited;
        }
        io.check_engine().await?;
        let removed = io
            .run(
                "container-remove",
                &args(&target.context, &["container", "rm", id]),
            )
            .await;
        report.actions.push(CleanupAction {
            target: format!("container:{id}"),
            action: CleanupActionKind::DockerRemove,
            outcome: if removed.is_ok() {
                CleanupActionOutcome::Confirmed
            } else {
                CleanupActionOutcome::Unknown
            },
            confirmation: CleanupConfirmation::NotAttempted,
        });
    }
    io.check_engine().await?;
    let final_ids = io.inventory("container", &profile.docker_labels).await?;
    for action in &mut report.actions {
        if action.action == CleanupActionKind::DockerRemove {
            let id = action
                .target
                .strip_prefix("container:")
                .context("cleanup target malformed")?;
            action.confirmation = if final_ids.iter().any(|seen| seen == id) {
                CleanupConfirmation::StillObserved
            } else {
                CleanupConfirmation::Absent
            };
        }
    }
    for id in final_ids {
        match io.inspect(&id).await {
            Ok(identity) if identity.labels == profile.docker_labels => {
                report.remaining.push(format!("container:{id}"));
            }
            Ok(_) => report.errors.push("docker_label_filter_mismatch".into()),
            Err(_) => report
                .errors
                .push("docker_final_inspection_unavailable".into()),
        }
    }
    // This finite profile creates containers with bind mounts. It does not
    // create managed networks/volumes; matching unexpected resources are
    // reported instead of deleting shared/attached or unregistered targets.
    for kind in ["network", "volume"] {
        for id in io.inventory(kind, &profile.docker_labels).await? {
            match io.resource_labels(kind, &id).await {
                Ok(labels) if labels == profile.docker_labels => {
                    report.remaining.push(format!("{kind}:{id}"));
                }
                Ok(_) => report.errors.push("docker_label_filter_mismatch".into()),
                Err(_) => report
                    .errors
                    .push("docker_resource_inspection_unavailable".into()),
            }
        }
    }
    report.remaining.sort();
    report.remaining.dedup();
    report.errors.sort();
    report.errors.dedup();
    ensure!(
        report.remaining.len() <= 96 && report.errors.len() <= 16,
        "Docker receipt exceeds bound"
    );
    Ok(())
}
struct ContainerIdentity {
    name: String,
    operation: String,
    labels: BTreeMap<String, String>,
    running: bool,
}
struct CleanupDocker<'a> {
    owner: &'a Arc<RuntimeOwner>,
    claim: &'a crate::state::CleanupClaim,
    program: &'a Path,
    target: &'a DockerTarget,
}
impl CleanupDocker<'_> {
    async fn run(&self, target: &str, args: &[String]) -> Result<Vec<u8>> {
        let operation = OperationId::new();
        let mut command = tokio::process::Command::new(self.program);
        command
            .args(args)
            .current_dir(&self.owner.root)
            .env("DOCKER_API_VERSION", "1.48")
            .env("RRX_PROCESS_COOKIE", operation.to_string())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        // Separate cookie prevents another historical sweep from treating this
        // Runtime helper as a native survivor. Registration precedes spawn.
        let mut guard = owner::HelperGuard::new(self.owner.clone(), operation);
        let child = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            store.reserve_cleanup_helper(self.claim, operation, target)?;
            process::OwnedProcess::spawn(&mut command)?
        };
        let captured = process::capture_child(child);
        tokio::pin!(captured);
        let mut fence = tokio::time::interval(Duration::from_millis(50));
        let observed = loop {
            tokio::select! {
                result=&mut captured=>break result,
                _=fence.tick()=>{self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.validate_cleanup_claim(self.claim)?;}
            }
        };
        let receipt = observed.as_ref().map_or_else(
            |_| BTreeMap::new(),
            |o| {
                BTreeMap::from([
                    (
                        "exit".into(),
                        o.receipt
                            .status
                            .code()
                            .map_or_else(|| "signal".into(), |n| n.to_string()),
                    ),
                    (
                        "group_cleanup".into(),
                        if o.receipt.group_error.is_none() {
                            "requested"
                        } else {
                            "unknown"
                        }
                        .into(),
                    ),
                ])
            },
        );
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .reconcile_managed_effect(
                operation,
                1,
                if observed.is_ok() {
                    EffectState::Confirmed
                } else {
                    EffectState::Unknown
                },
                receipt,
            )?;
        guard.disarm();
        let observed = observed?;
        ensure!(
            observed.receipt.status.success() && observed.stdout.len() <= 64 * 1024,
            "Docker cleanup command incomplete"
        );
        Ok(observed.stdout)
    }
    async fn check_engine(&self) -> Result<()> {
        let transport = self
            .run(
                "context-transport",
                &[
                    "context".into(),
                    "inspect".into(),
                    "--format".into(),
                    "{{index (split .Endpoints.docker.Host \"://\") 0}}".into(),
                    self.target.context.clone(),
                ],
            )
            .await?;
        ensure!(
            std::str::from_utf8(&transport)?.trim() == "unix",
            "Docker cleanup context transport changed"
        );
        let (client, server, api) = versions(
            &self
                .run(
                    "engine-version",
                    &args(
                        &self.target.context,
                        &["version", "--format", VERSION_FORMAT],
                    ),
                )
                .await?,
        )?;
        validate_versions(&client, &server, &api)?;
        let observed = engine_digest(
            &self
                .run(
                    "engine-identity",
                    &args(&self.target.context, &["info", "--format", ENGINE_FORMAT]),
                )
                .await?,
        )?;
        ensure!(
            observed == self.target.engine,
            "Docker cleanup engine changed"
        );
        Ok(())
    }
    async fn inventory(
        &self,
        kind: &str,
        labels: &BTreeMap<String, String>,
    ) -> Result<Vec<String>> {
        let mut a = args(&self.target.context, &[kind, "ls", "--quiet"]);
        if kind == "container" {
            a.extend(["--all".into(), "--no-trunc".into()]);
        }
        if kind == "network" {
            a.push("--no-trunc".into());
        }
        for (key, value) in labels {
            a.extend(["--filter".into(), format!("label={key}={value}")]);
        }
        let bytes = self.run(&format!("{kind}-inventory"), &a).await?;
        let ids = std::str::from_utf8(&bytes)?
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        ensure!(
            ids.len() <= 32
                && ids.iter().all(|id| match kind {
                    "container" | "network" =>
                        id.len() == 64 && id.bytes().all(|b| b.is_ascii_hexdigit()),
                    "volume" =>
                        !id.is_empty()
                            && id.len() <= 128
                            && id
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric()
                                    || matches!(b, b'_' | b'-' | b'.')),
                    _ => false,
                }),
            "Docker inventory identities malformed or exceeded bound"
        );
        Ok(ids)
    }
    async fn inspect(&self, id: &str) -> Result<ContainerIdentity> {
        let mut format = vec![
            "{{json .Id}}".to_owned(),
            "{{json .Name}}".into(),
            "{{json .State.Running}}".into(),
        ];
        for key in [
            "org.rururunx.runtime",
            "org.rururunx.project",
            "org.rururunx.task",
            "org.rururunx.unit",
            "org.rururunx.operation",
        ] {
            format.push(format!("{{{{json (index .Config.Labels {key:?})}}}}"));
        }
        let bytes = self
            .run(
                "container-inspect",
                &args(
                    &self.target.context,
                    &["container", "inspect", "--format", &format.join("\n"), id],
                ),
            )
            .await?;
        let lines = std::str::from_utf8(&bytes)?.lines().collect::<Vec<_>>();
        ensure!(lines.len() == 8, "Docker inspection malformed");
        let observed: String = serde_json::from_str(lines[0])?;
        ensure!(observed == id, "Docker inspected ID changed");
        let name: String = serde_json::from_str(lines[1])?;
        let name = name
            .strip_prefix('/')
            .context("Docker container name malformed")?
            .to_owned();
        ensure!(
            name.len() <= 256 && context_name(&name),
            "Docker container name exceeds profile"
        );
        let running: bool = serde_json::from_str(lines[2])?;
        let keys = [
            "org.rururunx.runtime",
            "org.rururunx.project",
            "org.rururunx.task",
            "org.rururunx.unit",
        ];
        let labels = keys
            .into_iter()
            .zip(&lines[3..7])
            .map(|(key, line)| Ok((key.to_owned(), serde_json::from_str::<String>(line)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let operation: String = serde_json::from_str(lines[7])?;
        operation.parse::<OperationId>()?;
        Ok(ContainerIdentity {
            name,
            operation,
            labels,
            running,
        })
    }
    async fn resource_labels(&self, kind: &str, id: &str) -> Result<BTreeMap<String, String>> {
        ensure!(
            matches!(kind, "network" | "volume"),
            "unsupported Docker resource"
        );
        let keys = [
            "org.rururunx.runtime",
            "org.rururunx.project",
            "org.rururunx.task",
            "org.rururunx.unit",
        ];
        // Emit only the namespace labels, never a resource's full inspect data.
        let format = keys
            .iter()
            .map(|key| format!("{{{{json (index .Labels {key:?})}}}}"))
            .collect::<Vec<_>>()
            .join("\n");
        let bytes = self
            .run(
                &format!("{kind}-inspect"),
                &args(
                    &self.target.context,
                    &[kind, "inspect", "--format", &format, id],
                ),
            )
            .await?;
        let lines = std::str::from_utf8(&bytes)?.lines().collect::<Vec<_>>();
        ensure!(
            lines.len() == keys.len(),
            "Docker resource labels malformed"
        );
        keys.into_iter()
            .zip(lines)
            .map(|(key, value)| Ok((key.into(), serde_json::from_str::<String>(value)?)))
            .collect()
    }
}

#[cfg(test)]
mod tests;
