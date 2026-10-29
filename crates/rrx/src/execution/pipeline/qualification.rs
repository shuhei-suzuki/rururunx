use super::argv::{CLAUDE_FLAGS, CODEX_FLAGS};
use crate::{
    config::{PipelineCli, PipelineConfig},
    execution::RuntimeOwner,
    state::pipeline::PipelineQualification,
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{File, OpenOptions},
    io::Read,
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualifiedCli {
    pub realpath: PathBuf,
    pub sha256: String,
}
#[cfg(test)]
thread_local! { static HASH_COUNT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }

const PROBE_BOUND: Duration = Duration::from_secs(5);
const PROBE_OUTPUT_LIMIT: usize = 4 * 1024;
const HELP_OUTPUT_LIMIT: usize = 64 * 1024;
// Five Claude probes plus bounded image/config work; cleanup retains custody
// even after observation expires and must finish before returning a refusal.
const CLI_BOUND: Duration = Duration::from_secs(35);
fn regular_file(path: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NONBLOCK.bits() as i32)
        .open(path)?;
    ensure!(
        file.metadata()?.is_file(),
        "qualification input is not a regular file"
    );
    Ok(file)
}
#[cfg(all(test, target_os = "macos"))]
fn executable(path: &Path) -> Result<QualifiedCli> {
    executable_until(path, Instant::now() + PROBE_BOUND)
}
fn executable_until(path: &Path, deadline: Instant) -> Result<QualifiedCli> {
    let realpath = path.canonicalize()?;
    let mut file = regular_file(&realpath)?;
    ensure!(
        file.metadata()?.permissions().mode() & 0o111 != 0,
        "CLI is not executable"
    );
    let mut magic = [0; 4];
    file.read_exact(&mut magic)?;
    ensure!(
        matches!(
            magic,
            [0xcf, 0xfa, 0xed, 0xfe]
                | [0xfe, 0xed, 0xfa, 0xcf]
                | [0xce, 0xfa, 0xed, 0xfe]
                | [0xfe, 0xed, 0xfa, 0xce]
                | [0xca, 0xfe, 0xba, 0xbe]
                | [0xbe, 0xba, 0xfe, 0xca]
                | [0xca, 0xfe, 0xba, 0xbf]
                | [0xbf, 0xba, 0xfe, 0xca]
        ),
        "CLI must be a native Mach-O binary"
    );
    #[cfg(test)]
    HASH_COUNT.with(|n| n.set(n.get() + 1));
    let mut hash = Sha256::new();
    hash.update(magic);
    let mut buffer = [0; 16384];
    loop {
        ensure!(
            Instant::now() < deadline,
            "qualification input deadline exceeded"
        );
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(QualifiedCli {
        realpath,
        sha256: format!("{:x}", hash.finalize()),
    })
}

/// Help lines may have aliases, metavariables and descriptions; match only option tokens.
fn required_flags(help: &str, required: &[&str]) -> bool {
    let flags: BTreeSet<&str> = help
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            line.starts_with('-').then_some(line)
        })
        .flat_map(|line| {
            line.split_whitespace().take_while(|word| {
                word.starts_with('-') || word.starts_with('<') || word.starts_with('[')
            })
        })
        .filter_map(|word| word.starts_with('-').then_some(word.trim_end_matches(',')))
        .collect();
    required.iter().all(|flag| flags.contains(flag))
}

fn reports_parser_error(out: &[u8], err: &[u8]) -> bool {
    [out, err].iter().any(|bytes| {
        String::from_utf8_lossy(bytes).lines().any(|line| {
            let line = line.trim_start().to_ascii_lowercase();
            line.starts_with("error:")
                || line.starts_with("error ")
                || line.starts_with("invalid value")
        })
    })
}

fn codex_config(home: &Path, deadline: Instant) -> Result<()> {
    ensure!(
        home.is_absolute() && home.is_dir(),
        "dedicated CODEX_HOME must be an existing absolute directory"
    );
    let default = PathBuf::from(std::env::var_os("HOME").context("missing HOME")?).join(".codex");
    ensure!(
        home.canonicalize()? != default.canonicalize().unwrap_or(default),
        "default CODEX_HOME is not dedicated"
    );
    let path = home.join("config.toml");
    if !path.exists() {
        return Ok(());
    }
    let mut bytes = Vec::new();
    regular_file(&path)?.take(16385).read_to_end(&mut bytes)?;
    ensure!(
        Instant::now() < deadline,
        "qualification input deadline exceeded"
    );
    ensure!(bytes.len() <= 16384, "CODEX_HOME config exceeds bound");
    let table: toml::Table = toml::from_str(std::str::from_utf8(&bytes)?)?;
    // The contract does not enumerate a positive allowlist. Only these inert
    // scalar defaults are accepted; executable templates override both.
    for (key, value) in table {
        ensure!(
            matches!(key.as_str(), "model" | "model_reasoning_effort") && value.is_str(),
            "CODEX_HOME config key refused"
        );
    }
    Ok(())
}

pub(super) fn environment(
    config: &PipelineConfig,
    tmp: &Path,
    cookie: &str,
    cli: &PipelineCli,
    codex: bool,
    reviewer: bool,
) -> Result<Vec<(String, std::ffi::OsString)>> {
    let mut env = Vec::new();
    for key in ["HOME", "USER", "LOGNAME"] {
        env.push((
            key.into(),
            std::env::var_os(key).with_context(|| format!("missing {key}"))?,
        ));
    }
    let mut paths = config.path_prefix.clone();
    paths.extend(["/usr/bin", "/bin", "/usr/sbin", "/sbin"].map(PathBuf::from));
    env.push(("PATH".into(), std::env::join_paths(paths)?));
    for (key, value) in [
        ("LANG", "en_US.UTF-8"),
        ("TERM", "dumb"),
        ("NO_COLOR", "1"),
        ("RRX_PROCESS_COOKIE", cookie),
    ] {
        env.push((key.into(), value.into()));
    }
    env.push(("TMPDIR".into(), tmp.as_os_str().into()));
    if codex {
        env.push((
            "CODEX_HOME".into(),
            cli.codex_home
                .as_ref()
                .context("CODEX_HOME missing")?
                .as_os_str()
                .into(),
        ));
    }
    if reviewer {
        for (key, value) in [
            ("GIT_CONFIG_COUNT", "1"),
            ("GIT_CONFIG_KEY_0", "safe.bareRepository"),
            ("GIT_CONFIG_VALUE_0", "explicit"),
            ("GIT_CONFIG_NOSYSTEM", "1"),
        ] {
            env.push((key.into(), value.into()));
        }
    }
    Ok(env)
}

async fn qualify_cli(
    config: &PipelineConfig,
    subject: &str,
    cli: &PipelineCli,
    tmp: &Path,
) -> PipelineQualification {
    qualify_cli_with_bound(config, subject, cli, tmp, CLI_BOUND).await
}

async fn qualify_cli_with_bound(
    config: &PipelineConfig,
    subject: &str,
    cli: &PipelineCli,
    tmp: &Path,
    bound: Duration,
) -> PipelineQualification {
    let mut row = PipelineQualification {
        subject: subject.into(),
        state: "refused".into(),
        executable_sha256: None,
        version_text: None,
        body: json!({}),
    };
    row.body["config_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(config).expect("serializable pipeline config"))
    ));
    let deadline = Instant::now() + bound;
    let result = async {
        row.body["stage"] = json!("executable");
        let pinned = executable_until(&cli.executable, deadline)?;
        row.executable_sha256 = Some(pinned.sha256.clone());
        row.body["realpath"] = json!(pinned.realpath);
        let codex = subject == "codex";
        row.body["stage"] = json!("config");
        if codex {
            codex_config(
                cli.codex_home.as_ref().context("CODEX_HOME missing")?,
                deadline,
            )?;
        }
        let cookie = uuid::Uuid::new_v4().simple().to_string();
        row.body["stage"] = json!("environment");
        let env = environment(config, tmp, &cookie, cli, codex, !codex)?;
        let probe = |args: &[&str], output_limit: usize| {
            let mut command = tokio::process::Command::new(&pinned.realpath);
            command
                .args(args)
                .env_clear()
                .envs(env.clone())
                .current_dir(tmp);
            crate::execution::process::capture_probe_until(
                command,
                PROBE_BOUND,
                deadline.into(),
                output_limit,
            )
        };
        row.body["stage"] = json!("version_probe");
        let (receipt, out, _) = probe(&["--version"], PROBE_OUTPUT_LIMIT).await?;
        row.body["stage"] = json!("version_check");
        let version = std::str::from_utf8(&out)?.trim();
        // Never persist arbitrary CLI output; retain only the exact approved pin.
        ensure!(
            receipt.status.success() && version == cli.version,
            "CLI version does not equal pin"
        );
        row.version_text = Some(version.into());
        row.body["stage"] = json!("help_probe");
        let (receipt, out, _) = probe(
            if codex {
                &["exec", "--help"]
            } else {
                &["--help"]
            },
            HELP_OUTPUT_LIMIT,
        )
        .await?;
        row.body["stage"] = json!("help_check");
        ensure!(
            receipt.status.success()
                && required_flags(
                    std::str::from_utf8(&out)?,
                    if codex { CODEX_FLAGS } else { CLAUDE_FLAGS }
                ),
            "CLI required flag missing"
        );
        if !codex {
            // Parser-only qualification: this never supplies -p or a prompt.
            row.body["stage"] = json!("settings_probe");
            let (invalid, _, err) =
                probe(&["--setting-sources", "bogus", "--help"], HELP_OUTPUT_LIMIT).await?;
            let rejects_invalid = !invalid.status.success() || reports_parser_error(&[], &err);
            row.body["reason"] = json!("setting_sources_validation_unproven");
            ensure!(
                rejects_invalid,
                "setting sources value validation not demonstrated"
            );
            row.body.as_object_mut().unwrap().remove("reason");
            let (receipt, out, err) =
                probe(&["--setting-sources", "", "--help"], HELP_OUTPUT_LIMIT).await?;
            row.body["stage"] = json!("settings_check");
            ensure!(
                receipt.status.success() && !reports_parser_error(&out, &err),
                "CLI empty setting sources refused"
            );
        }
        row.body["stage"] = json!("login_probe");
        let (receipt, _, _) = probe(
            if codex {
                &["login", "status"]
            } else {
                &["--setting-sources", "", "auth", "status"]
            },
            PROBE_OUTPUT_LIMIT,
        )
        .await?;
        row.body["stage"] = json!("login_check");
        ensure!(receipt.status.success(), "CLI login status refused");
        row.body["stage"] = json!("image_recheck");
        let final_image = executable_until(&cli.executable, deadline)?;
        ensure!(
            final_image.realpath == pinned.realpath && final_image.sha256 == pinned.sha256,
            "qualified executable changed"
        );
        row.body["stage"] = json!("done");
        Ok::<_, anyhow::Error>(())
    }
    .await;
    match result {
        Ok(()) => row.state = "qualified".into(),
        // Paths, credentials and tool output are not copied into durable errors.
        Err(_) => {
            if row.body.get("reason").is_none() {
                row.body["reason"] = json!("cli_qualification_failed");
            }
        }
    }
    row
}

pub(crate) struct QualificationRecord(PipelineQualification);
impl QualificationRecord {
    pub(crate) fn row(&self) -> &PipelineQualification {
        &self.0
    }
}

pub struct CliQualifier;

fn project_root_for_exclusion(project: &crate::domain::Project) -> Result<Option<PathBuf>> {
    use crate::domain::ProjectState;
    if project.state == ProjectState::Removed {
        return Ok(None);
    }
    match project.root.canonicalize() {
        Ok(root) => Ok(Some(root)),
        Err(error)
            if project.state == ProjectState::Blocked
                && error.kind() == std::io::ErrorKind::NotFound =>
        {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

impl CliQualifier {
    pub(in crate::execution) fn record_quiescence(
        owner: &RuntimeOwner,
        proof: crate::execution::owner::ModeQuiescence,
    ) -> Result<()> {
        let row = QualificationRecord(PipelineQualification {
            subject: "mode_quiescence".into(),
            state: "qualified".into(),
            executable_sha256: None,
            version_text: None,
            body: proof.into_body(),
        });
        owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_record_qualification(owner, &row)
    }
    /// Run Q once per owner epoch, with no Store/control mutex across a probe.
    pub async fn qualify(owner: &RuntimeOwner, config: &PipelineConfig) -> Result<()> {
        config.validate()?;
        let platform = PipelineQualification {
            subject: "platform".into(),
            state: if cfg!(target_os = "macos") {
                "qualified"
            } else {
                "refused"
            }
            .into(),
            executable_sha256: None,
            version_text: None,
            body: json!({"os":std::env::consts::OS}),
        };
        owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_record_qualification(owner, &QualificationRecord(platform.clone()))?;
        ensure!(platform.state == "qualified", "pipeline platform refused");
        let root = &config.work_root;
        if !root.exists() {
            std::fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(root)?;
        }
        Self::check_layout(owner, config)?;
        let temp_root = root.join("t");
        super::ensure_private_directory(&temp_root)?;
        let tmp = tempfile::Builder::new()
            .prefix("q")
            .rand_bytes(4)
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir_in(temp_root)?;
        ensure!(
            tmp.path().as_os_str().len() <= 60,
            "pipeline TMPDIR exceeds bound"
        );
        Self::record_clis(owner, config, tmp.path()).await?;
        Self::current(owner, config)
    }

    async fn record_clis(owner: &RuntimeOwner, config: &PipelineConfig, tmp: &Path) -> Result<()> {
        let mut all = true;
        for (subject, cli) in [
            ("codex", &config.clis.codex),
            ("claude", &config.clis.claude),
        ] {
            let row = qualify_cli(config, subject, cli, tmp).await;
            all &= row.state == "qualified";
            owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_record_qualification(owner, &QualificationRecord(row))?;
        }
        ensure!(all, "pipeline startup qualification refused");
        Ok(())
    }

    fn check_layout(owner: &RuntimeOwner, config: &PipelineConfig) -> Result<()> {
        let root = &config.work_root;
        private_directory(root).context("work_root")?;
        let home = config
            .clis
            .codex
            .codex_home
            .as_ref()
            .context("CODEX_HOME missing")?;
        private_directory(home).context("CODEX_HOME")?;
        for (left, right) in [
            (root.as_path(), home.as_path()),
            (
                root.as_path(),
                owner.state_path().parent().context("state needs parent")?,
            ),
            (
                home.as_path(),
                owner.state_path().parent().context("state needs parent")?,
            ),
        ] {
            ensure!(
                !overlaps(left, right),
                "pipeline private directories overlap"
            );
        }
        let projects = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .projects()?;
        for project in projects {
            if let Some(root) = project_root_for_exclusion(&project)? {
                config.check_project_root(&root)?;
            }
        }
        Self::check_launch_paths(owner, config)?;
        trusted_path_prefix(config)?;
        Ok(())
    }

    fn check_launch_paths(owner: &RuntimeOwner, config: &PipelineConfig) -> Result<()> {
        let mut excluded = vec![
            config.work_root.clone(),
            config
                .clis
                .codex
                .codex_home
                .clone()
                .context("CODEX_HOME missing")?,
            owner
                .state_path()
                .parent()
                .context("state needs parent")?
                .to_path_buf(),
        ];
        let projects = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .projects()?;
        for project in projects {
            if let Some(root) = project_root_for_exclusion(&project)? {
                excluded.push(root);
            }
        }
        for path in config.path_prefix.iter().chain([
            &config.clis.codex.executable,
            &config.clis.claude.executable,
        ]) {
            let realpath = path.canonicalize()?;
            for root in &excluded {
                ensure!(
                    !overlaps(path, root) && !overlaps(&realpath, root),
                    "pipeline launch path overlaps writable or state directory"
                );
            }
        }
        Ok(())
    }

    /// Idle currency checks read only immutable current-epoch rows and the config digest.
    pub fn current(owner: &RuntimeOwner, config: &PipelineConfig) -> Result<()> {
        ensure!(
            owner.execution_mode() == crate::config::ExecutionMode::CliPipeline,
            "pipeline requires CliPipeline owner"
        );
        let rows = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_qualifications(owner)?;
        for subject in ["platform", "mode_quiescence", "codex", "claude"] {
            let row = rows
                .iter()
                .find(|row| row.subject == subject)
                .context("pipeline Q missing")?;
            ensure!(row.state == "qualified", "pipeline Q refused");
            if let Some(cli) = match subject {
                "codex" => Some(&config.clis.codex),
                "claude" => Some(&config.clis.claude),
                _ => None,
            } {
                ensure!(
                    row.body["config_sha256"]
                        == json!(format!("{:x}", Sha256::digest(serde_json::to_vec(config)?))),
                    "pipeline Q configuration changed"
                );
                ensure!(
                    row.version_text.as_deref() == Some(&cli.version),
                    "pipeline Q pin changed"
                );
            }
        }
        Ok(())
    }

    /// B4 must call this outside the Store transaction, between tx1 and spawn.
    pub fn before_launch(
        owner: &RuntimeOwner,
        config: &PipelineConfig,
    ) -> Result<[QualifiedCli; 2]> {
        Self::current(owner, config)?;
        let deadline = Instant::now() + PROBE_BOUND;
        Self::check_layout(owner, config)?;
        codex_config(
            config
                .clis
                .codex
                .codex_home
                .as_ref()
                .context("CODEX_HOME missing")?,
            deadline,
        )?;
        let rows = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_qualifications(owner)?;
        let mut images = Vec::new();
        for (subject, cli) in [
            ("codex", &config.clis.codex),
            ("claude", &config.clis.claude),
        ] {
            let row = rows
                .iter()
                .find(|r| r.subject == subject)
                .context("pipeline Q missing")?;
            let image = executable_until(&cli.executable, deadline)?;
            ensure!(
                Some(&image.sha256) == row.executable_sha256.as_ref()
                    && serde_json::to_value(&image.realpath)? == row.body["realpath"],
                "qualified executable changed"
            );
            images.push(image);
        }
        images
            .try_into()
            .map_err(|_| anyhow::anyhow!("qualified CLI pair missing"))
    }
}

fn overlaps(a: &Path, b: &Path) -> bool {
    a.starts_with(b) || b.starts_with(a)
}
fn private_directory(path: &Path) -> Result<()> {
    let m = std::fs::symlink_metadata(path)?;
    ensure!(m.is_dir(), "pipeline private directory must be a directory");
    ensure!(
        m.permissions().mode() & 0o777 == 0o700,
        "pipeline private directory requires mode 0700"
    );
    ensure!(
        m.uid() == rustix::process::getuid().as_raw(),
        "pipeline private directory has wrong owner"
    );
    ensure!(
        path.canonicalize()? == path,
        "pipeline private directory must be canonical"
    );
    Ok(())
}
fn trusted_path_prefix(config: &PipelineConfig) -> Result<()> {
    for configured in &config.path_prefix {
        let dir = configured.canonicalize()?;
        ensure!(
            &dir == configured,
            "pipeline PATH directory must be canonical"
        );
        for parent in dir.ancestors() {
            let m = parent.metadata()?;
            ensure!(
                m.is_dir()
                    && m.permissions().mode() & 0o022 == 0
                    && [0, rustix::process::getuid().as_raw()].contains(&m.uid()),
                "untrusted pipeline PATH directory or parent"
            );
        }
    }
    Ok(())
}

#[cfg(all(test, target_os = "macos"))]
mod tests;
