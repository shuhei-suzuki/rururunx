use super::argv::{CLAUDE_FLAGS, CODEX_FLAGS};
use crate::{
    config::{PipelineCli, PipelineConfig},
    execution::{RuntimeOwner, process::capture_probe},
    state::pipeline::PipelineQualification,
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io::Read,
    os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualifiedCli {
    pub realpath: PathBuf,
    pub sha256: String,
}
impl QualifiedCli {
    /// Rechecked outside Store transactions immediately before any future launch.
    pub fn recheck(&self, configured: &PipelineCli) -> Result<()> {
        let current = executable(&configured.executable)?;
        ensure!(
            current.realpath == self.realpath && current.sha256 == self.sha256,
            "qualified executable changed"
        );
        Ok(())
    }
}
fn executable(path: &Path) -> Result<QualifiedCli> {
    let realpath = path.canonicalize()?;
    let mut file = File::open(&realpath)?;
    ensure!(file.metadata()?.is_file(), "CLI executable is not a file");
    let mut hash = Sha256::new();
    let mut buffer = [0; 16384];
    loop {
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

fn codex_config(home: &Path) -> Result<()> {
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
    File::open(path)?.take(16385).read_to_end(&mut bytes)?;
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

fn environment(
    config: &PipelineConfig,
    tmp: &Path,
    cookie: &str,
    cli: &PipelineCli,
    codex: bool,
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
    Ok(env)
}

async fn qualify_cli(
    config: &PipelineConfig,
    subject: &str,
    cli: &PipelineCli,
    tmp: &Path,
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
    let result = async {
        row.body["stage"] = json!("executable");
        let pinned = executable(&cli.executable)?;
        row.executable_sha256 = Some(pinned.sha256.clone());
        row.body["realpath"] = json!(pinned.realpath);
        let codex = subject == "codex";
        row.body["stage"] = json!("config");
        if codex {
            codex_config(cli.codex_home.as_ref().context("CODEX_HOME missing")?)?;
        }
        let cookie = uuid::Uuid::new_v4().simple().to_string();
        row.body["stage"] = json!("environment");
        let env = environment(config, tmp, &cookie, cli, codex)?;
        let probe = |args: &[&str]| {
            let mut command = tokio::process::Command::new(&pinned.realpath);
            command
                .args(args)
                .env_clear()
                .envs(env.clone())
                .current_dir(tmp);
            capture_probe(command)
        };
        row.body["stage"] = json!("version_probe");
        let (receipt, out, _) = probe(&["--version"]).await?;
        row.body["stage"] = json!("version_check");
        let version = std::str::from_utf8(&out)?.trim();
        // Never persist arbitrary CLI output; retain only the exact approved pin.
        ensure!(
            receipt.status.success() && version == cli.version,
            "CLI version does not equal pin"
        );
        row.version_text = Some(version.into());
        row.body["stage"] = json!("help_probe");
        let (receipt, out, _) = probe(if codex {
            &["exec", "--help"]
        } else {
            &["--help"]
        })
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
            let (receipt, _, _) = probe(&["--setting-sources", "", "--help"]).await?;
            row.body["stage"] = json!("settings_check");
            ensure!(
                receipt.status.success(),
                "CLI empty setting sources refused"
            );
        }
        row.body["stage"] = json!("login_probe");
        let (receipt, _, _) = probe(if codex {
            &["login", "status"]
        } else {
            &["auth", "status"]
        })
        .await?;
        row.body["stage"] = json!("login_check");
        ensure!(receipt.status.success(), "CLI login status refused");
        row.body["stage"] = json!("image_recheck");
        pinned.recheck(cli)?;
        row.body["stage"] = json!("done");
        Ok::<_, anyhow::Error>(())
    }
    .await;
    match result {
        Ok(()) => row.state = "qualified".into(),
        // Paths, credentials and tool output are not copied into durable errors.
        Err(_) => row.body["reason"] = json!("cli_qualification_failed"),
    }
    row
}

pub struct CliQualifier;
impl CliQualifier {
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
            .pipeline_record_qualification(owner, &platform)?;
        ensure!(platform.state == "qualified", "pipeline platform refused");
        let root = &config.work_root;
        if !root.exists() {
            std::fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(root)?;
        }
        let metadata = std::fs::symlink_metadata(root)?;
        ensure!(
            metadata.is_dir()
                && metadata.permissions().mode() & 0o777 == 0o700
                && metadata.uid() == rustix::process::getuid().as_raw(),
            "work_root must be owned directory mode 0700"
        );
        let root = root.canonicalize()?;
        ensure!(
            root == config.work_root,
            "work_root must be canonical without symlink aliases"
        );
        ensure!(
            !root.starts_with(owner.state_path().parent().context("state parent absent")?)
                && !owner.state_path().starts_with(&root),
            "work_root overlaps state"
        );
        let projects = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .projects()?;
        for project in projects {
            let project = project.root.canonicalize()?;
            ensure!(
                !root.starts_with(&project) && !project.starts_with(&root),
                "work_root overlaps Project"
            );
        }
        for dir in &config.path_prefix {
            let m = dir.metadata()?;
            ensure!(
                m.is_dir() && m.permissions().mode() & 0o002 == 0,
                "world-writable pipeline PATH directory"
            );
        }
        let temp_root = root.join("t");
        std::fs::create_dir_all(&temp_root)?;
        let tmp = tempfile::Builder::new()
            .prefix("q")
            .rand_bytes(4)
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
                .pipeline_record_qualification(owner, &row)?;
        }
        ensure!(all, "pipeline startup qualification refused");
        Ok(())
    }

    /// No work may be admitted using another epoch, pin or executable image.
    pub fn current(owner: &RuntimeOwner, config: &PipelineConfig) -> Result<()> {
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
                if subject == "codex" {
                    codex_config(cli.codex_home.as_ref().context("CODEX_HOME missing")?)?;
                }
                QualifiedCli {
                    realpath: serde_json::from_value(row.body["realpath"].clone())?,
                    sha256: row
                        .executable_sha256
                        .clone()
                        .context("pipeline Q hash missing")?,
                }
                .recheck(cli)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
