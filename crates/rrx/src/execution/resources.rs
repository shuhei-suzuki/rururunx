//! Durable cooperative namespaces, not a host or security sandbox.
use super::{
    results::{durable_file, hex},
    *,
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceProfile {
    pub unit: UnitId,
    pub runtime: String,
    pub project: String,
    pub task: String,
    pub root: PathBuf,
    pub worktree: PathBuf,
    pub temp: PathBuf,
    pub output: PathBuf,
    pub cache: PathBuf,
    pub tool_bin: PathBuf,
    pub port_start: u16,
    pub port_end: u16,
    pub docker_project: String,
    pub docker_labels: BTreeMap<String, String>,
    pub real_tools: BTreeMap<String, PathBuf>,
    pub digest: String,
}
impl ResourceProfile {
    pub fn environment(&self, cookie: &str, ipc: &Path) -> Result<BTreeMap<String, String>> {
        let inherited = std::env::var_os("PATH").context("PATH is required")?;
        let mut paths = vec![self.tool_bin.clone()];
        paths.extend(std::env::split_paths(&inherited));
        let path = std::env::join_paths(paths)?
            .into_string()
            .map_err(|_| anyhow::anyhow!("PATH must be UTF-8"))?;
        let mut overlay = self.namespace_environment(cookie, ipc);
        overlay.insert("PATH".into(), path);
        // Preserve inherited config entries, including required hooks, without reading/copying their values.
        let count = std::env::var("GIT_CONFIG_COUNT")
            .ok()
            .map(|s| s.parse::<usize>())
            .transpose()?
            .unwrap_or(0);
        ensure!(count <= 128, "inherited Git config overlay exceeds bound");
        overlay.insert("GIT_CONFIG_COUNT".into(), (count + 3).to_string());
        for (i, (key, value)) in [
            ("gc.auto", "0"),
            ("maintenance.auto", "false"),
            ("core.fsmonitor", "false"),
        ]
        .into_iter()
        .enumerate()
        {
            overlay.insert(format!("GIT_CONFIG_KEY_{}", count + i), key.into());
            overlay.insert(format!("GIT_CONFIG_VALUE_{}", count + i), value.into());
        }
        Ok(overlay)
    }
    /// Non-secret owner-assigned resource values only. Auth, native settings and
    /// Git overlay entries stay inherited in the shim's actual native context.
    pub(crate) fn namespace_environment(
        &self,
        cookie: &str,
        ipc: &Path,
    ) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("RRX_UNIT_ID".into(), self.unit.to_string()),
            ("RRX_PROCESS_COOKIE".into(), cookie.into()),
            (
                "RRX_PROFILE".into(),
                self.root.join("profile.json").to_string_lossy().into(),
            ),
            ("RRX_RUNTIME_SOCKET".into(), ipc.to_string_lossy().into()),
            ("RRX_TMPDIR".into(), self.temp.to_string_lossy().into()),
            ("TMPDIR".into(), self.temp.to_string_lossy().into()),
            (
                "RRX_OUTPUT_DIR".into(),
                self.output.to_string_lossy().into(),
            ),
            ("RRX_PORT_START".into(), self.port_start.to_string()),
            ("RRX_PORT_END".into(), self.port_end.to_string()),
            ("RRX_DOCKER_PROJECT".into(), self.docker_project.clone()),
            ("COMPOSE_PROJECT_NAME".into(), self.docker_project.clone()),
            (
                "CARGO_TARGET_DIR".into(),
                self.output.join("cargo-target").to_string_lossy().into(),
            ),
            (
                "GRADLE_USER_HOME".into(),
                self.cache.join("gradle").to_string_lossy().into(),
            ),
            (
                "SCCACHE_DIR".into(),
                self.cache.join("sccache").to_string_lossy().into(),
            ),
        ])
    }
    pub(crate) fn validate(&self, owner: &RuntimeOwner, unit: &ExecutionUnit) -> Result<()> {
        ensure!(
            self.unit == unit.id
                && self.runtime == owner.instance
                && self.project == unit.scope.project_id.to_string()
                && self.task == unit.scope.task_id.context("Task missing")?.to_string()
                && self.digest == unit.profile_digest
                && self.root == owner.root.join("units").join(unit.id.to_string())
                && self.worktree == unit.worktree,
            "foreign resource profile"
        );
        let mut draft = self.clone();
        draft.digest.clear();
        ensure!(
            hex(&serde_json::to_vec(&draft)?) == self.digest,
            "resource profile changed"
        );
        for path in [&self.temp, &self.output, &self.cache, &self.tool_bin] {
            ensure!(
                path.parent() == Some(self.root.as_path()),
                "resource path outside unit namespace"
            );
            ensure!(
                path.canonicalize()? == *path,
                "resource path aliases another namespace"
            );
        }
        Ok(())
    }
}

pub(crate) fn resolve_program(name: &str) -> Result<PathBuf> {
    ensure!(!name.contains('/'), "program resolution requires a name");
    let found = std::env::split_paths(&std::env::var_os("PATH").context("PATH missing")?)
        .map(|p| p.join(name))
        .find(|p| p.is_file())
        .context("required native tool is not installed")?;
    let canonical = found.canonicalize()?;
    ensure!(canonical.is_file(), "native binary is not a regular file");
    Ok(canonical)
}

pub struct ResourceManager {
    owner: Arc<RuntimeOwner>,
}
impl ResourceManager {
    pub fn new(owner: Arc<RuntimeOwner>) -> Self {
        Self { owner }
    }
    pub(crate) fn draft(
        &self,
        id: UnitId,
        scope: &crate::domain::Scope,
        worktree: &Path,
    ) -> Result<ResourceProfile> {
        let units = {
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .execution_units(None)?
        };
        let used = units
            .into_iter()
            .map(|u| {
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .execution_leases(u.id)
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .filter(|l| l.kind == ResourceKind::Ports && l.state != LeaseState::Released)
            .filter_map(|l| Some((l.port_start?, l.port_end?)))
            .collect::<Vec<_>>();
        let start = (20000..=60000u16)
            .step_by(32)
            .find(|s| used.iter().all(|(a, b)| s + 31 < *a || *s > *b))
            .context("port resource pool exhausted; waiting_resource")?;
        let root = self.owner.root.join("units").join(id.to_string());
        let task = scope.task_id.context("Task missing")?.to_string();
        let mut tools = BTreeMap::new();
        for name in [
            "git", "docker", "gradle", "bazel", "bazelisk", "sccache", "tmux", "ssh",
        ] {
            if let Ok(path) = resolve_program(name) {
                tools.insert(name.into(), path);
            }
        }
        ensure!(tools.contains_key("git"), "Git is required");
        let labels = BTreeMap::from([
            ("org.rururunx.runtime".into(), self.owner.instance.clone()),
            ("org.rururunx.project".into(), scope.project_id.to_string()),
            ("org.rururunx.task".into(), task.clone()),
            ("org.rururunx.unit".into(), id.to_string()),
        ]);
        let mut profile = ResourceProfile {
            unit: id,
            runtime: self.owner.instance.clone(),
            project: scope.project_id.to_string(),
            task,
            root: root.clone(),
            worktree: worktree.into(),
            temp: root.join("tmp"),
            output: root.join("output"),
            cache: root.join("cache"),
            tool_bin: root.join("tool-bin"),
            port_start: start,
            port_end: start + 31,
            docker_project: format!("rrx-{}", id.to_string().replace('-', "")),
            docker_labels: labels,
            real_tools: tools,
            digest: String::new(),
        };
        profile.digest = hex(&serde_json::to_vec(&profile)?);
        Ok(profile)
    }
    pub(crate) fn reserve(&self, unit: &ExecutionUnit, profile: &ResourceProfile) -> Result<()> {
        let mut leases = Vec::new();
        for (kind, path) in [
            (ResourceKind::Worktree, &profile.worktree),
            (ResourceKind::Temp, &profile.temp),
            (ResourceKind::Output, &profile.output),
        ] {
            leases.push(ResourceLease {
                id: LeaseId::new(),
                unit_id: unit.id,
                scope: unit.scope.clone(),
                kind,
                namespace: self.owner.instance.clone(),
                value: path.to_string_lossy().into(),
                port_start: None,
                port_end: None,
                state: LeaseState::Reserved,
                version: 1,
            });
        }
        leases.push(ResourceLease {
            id: LeaseId::new(),
            unit_id: unit.id,
            scope: unit.scope.clone(),
            kind: ResourceKind::Ports,
            namespace: "host".into(),
            value: format!("{}-{}", profile.port_start, profile.port_end),
            port_start: Some(profile.port_start),
            port_end: Some(profile.port_end),
            state: LeaseState::Reserved,
            version: 1,
        });
        leases.push(ResourceLease {
            id: LeaseId::new(),
            unit_id: unit.id,
            scope: unit.scope.clone(),
            kind: ResourceKind::Docker,
            namespace: self.owner.instance.clone(),
            value: profile.docker_project.clone(),
            port_start: None,
            port_end: None,
            state: LeaseState::Reserved,
            version: 1,
        });
        leases.push(ResourceLease {
            id: LeaseId::new(),
            unit_id: unit.id,
            scope: unit.scope.clone(),
            kind: ResourceKind::ToolSocket,
            namespace: self.owner.instance.clone(),
            value: profile.root.join("tmux.sock").to_string_lossy().into(),
            port_start: None,
            port_end: None,
            state: LeaseState::Reserved,
            version: 1,
        });
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .reserve_execution_leases(&unit.authority(), &leases)
    }
    pub(crate) fn materialize(&self, profile: &ResourceProfile) -> Result<()> {
        std::fs::create_dir_all(profile.root.parent().context("unit root parent missing")?)?;
        ensure!(
            !profile.root.exists() && !profile.root.symlink_metadata().is_ok(),
            "unit namespace is never reused"
        );
        std::fs::create_dir(&profile.root)?;
        for path in [
            &profile.temp,
            &profile.output,
            &profile.cache,
            &profile.tool_bin,
        ] {
            std::fs::create_dir(path)?;
        }
        durable_file(
            &profile.root.join("profile.json"),
            &serde_json::to_vec(profile)?,
        )?;
        let helper = std::env::current_exe()?.canonicalize()?;
        for name in profile.real_tools.keys() {
            symlink(&helper, profile.tool_bin.join(name))?;
        }
        Ok(())
    }
    pub fn profile(&self, unit: &ExecutionUnit) -> Result<ResourceProfile> {
        let path = self
            .owner
            .root
            .join("units")
            .join(unit.id.to_string())
            .join("profile.json");
        let bytes = std::fs::read(path)?;
        ensure!(bytes.len() <= 64 * 1024, "profile bound exceeded");
        let profile: ResourceProfile = serde_json::from_slice(&bytes)?;
        profile.validate(&self.owner, unit)?;
        Ok(profile)
    }
}
