//! Detached, filter-checked inputs and fixed Seatbelt policy for admitted commands.
use super::{PipelineGit, ensure_private_directory};
use crate::{
    config::PipelineConfig,
    execution::{
        RuntimeOwner,
        verification::{AdmittedProfile, TestCommand},
    },
    state::pipeline::{PipelineRun, PipelineStep},
};
use anyhow::{Context, Result, ensure};
use std::path::{Component, Path, PathBuf};

pub struct PipelineTestRunner;
pub(crate) struct PreparedTest {
    pub profile: AdmittedProfile,
    pub command: TestCommand,
    pub checkout: PathBuf,
    pub forbidden: Vec<PathBuf>,
}

impl PipelineTestRunner {
    pub(crate) fn checkout(config: &PipelineConfig, step: &PipelineStep) -> Result<PathBuf> {
        ensure!(
            !step.run_id.is_empty()
                && step
                    .run_id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b)),
            "invalid test run component"
        );
        Ok(config
            .work_root
            .join("co")
            .join(format!("{}-{}-{}", step.run_id, step.seq, step.attempt)))
    }

    pub(crate) async fn prepare(
        owner: &RuntimeOwner,
        config: &PipelineConfig,
        run: &PipelineRun,
        step: &PipelineStep,
        profile: AdmittedProfile,
    ) -> Result<PreparedTest> {
        ensure!(cfg!(target_os = "macos"), "pipeline tests require macOS");
        ensure!(
            step.kind == "test"
                && step.state == "planned"
                && step.cookie.is_none()
                && step.step_base == run.head_commit,
            "test checkout requires unlaunched selected head"
        );
        let command = selected(&profile, step)?.clone();
        let projects = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .projects()?;
        let project = projects
            .iter()
            .find(|p| p.id == run.project_id)
            .context("test Project missing")?;
        let git = PipelineGit::new(&project.root)?;
        git.reject_tree_drivers(&step.step_base).await?;
        let common = PathBuf::from(
            git.text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
                .await?,
        );
        let checkout = Self::checkout(config, step)?;
        let task_worktree = run
            .worktree
            .as_deref()
            .map(|p| Path::new(p).canonicalize())
            .transpose()?;
        if let Some(task) = &task_worktree {
            ensure!(
                !checkout.starts_with(task) && !task.starts_with(&checkout),
                "test checkout overlaps Task worktree"
            );
        }
        ensure_private_directory(&config.work_root.join("co"))?;
        // A leftover is removable only when it is this repository's registered worktree.
        if std::fs::symlink_metadata(&checkout).is_ok() {
            registered(&git, &checkout).await?;
            if clean(&git, &checkout, &step.step_base).await.is_err() {
                git.run(&[
                    "worktree",
                    "remove",
                    "--force",
                    checkout.to_str().context("checkout encoding")?,
                ])
                .await?;
            }
        }
        if !checkout.try_exists()? {
            // Inspect conditional worktree config before checkout can run any filters.
            git.run(&[
                "worktree",
                "add",
                "--detach",
                "--no-checkout",
                checkout.to_str().context("checkout encoding")?,
                &step.step_base,
            ])
            .await?;
            let co = PipelineGit::new(&checkout)?;
            co.reject_tree_drivers(&step.step_base).await?;
            co.run(&["reset", "--hard", &step.step_base]).await?;
        }
        clean(&git, &checkout, &step.step_base).await?;
        let cwd = checkout.join(&command.cwd).canonicalize()?;
        ensure!(
            cwd.starts_with(&checkout) && cwd.is_dir(),
            "test cwd escaped checkout"
        );
        let mut forbidden = vec![
            common.canonicalize()?,
            owner.root.clone(),
            owner
                .state_path()
                .parent()
                .context("state parent missing")?
                .to_path_buf(),
        ];
        forbidden.extend(task_worktree);
        for p in projects {
            forbidden.extend([p.root, p.worktree_root]);
        }
        forbidden.extend(config.test_protected_paths()?);
        Ok(PreparedTest {
            profile,
            command,
            checkout,
            forbidden,
        })
    }

    /// Called after the final command is adopted, under control_admission.
    pub(crate) async fn remove(project: &Path, checkout: &Path) -> Result<()> {
        let git = PipelineGit::new(project)?;
        let listing = git.run(&["worktree", "list", "--porcelain", "-z"]).await?;
        let field = format!("worktree {}", checkout.display());
        let owned = listing.split(|b| *b == 0).any(|b| b == field.as_bytes());
        let exists = match std::fs::symlink_metadata(checkout) {
            Ok(_) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(e) => return Err(e.into()),
        };
        ensure!(owned || !exists, "test cleanup path is not registered");
        if !owned {
            return Ok(());
        }
        if exists {
            registered(&git, checkout).await?;
        }
        git.run(&[
            "worktree",
            "remove",
            "--force",
            checkout.to_str().context("checkout encoding")?,
        ])
        .await?;
        Ok(())
    }
}

pub(crate) fn selected<'a>(
    profile: &'a AdmittedProfile,
    step: &PipelineStep,
) -> Result<&'a TestCommand> {
    let index = step.body["command_index"]
        .as_u64()
        .context("test command index missing")?;
    let command = profile
        .proposal
        .commands
        .get(usize::try_from(index)?)
        .context("test command index outside profile")?;
    ensure!(
        step.body["command_id"].as_str() == Some(&command.id),
        "test command identity changed"
    );
    Ok(command)
}

async fn registered(project: &PipelineGit, checkout: &Path) -> Result<()> {
    ensure!(
        checkout.canonicalize()? == checkout,
        "test checkout aliases"
    );
    let listing = project
        .run(&["worktree", "list", "--porcelain", "-z"])
        .await?;
    let field = format!("worktree {}", checkout.display());
    ensure!(
        listing.split(|b| *b == 0).any(|b| b == field.as_bytes()),
        "test checkout registration missing"
    );
    let co = PipelineGit::new(checkout)?;
    let common = PathBuf::from(
        project
            .text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
            .await?,
    )
    .canonicalize()?;
    let gitfile = checkout.join(".git");
    ensure!(
        std::fs::symlink_metadata(&gitfile)?.file_type().is_file(),
        "test gitfile is not regular"
    );
    let contents = std::fs::read_to_string(&gitfile)?;
    let gitdir = PathBuf::from(
        contents
            .trim_end()
            .strip_prefix("gitdir: ")
            .context("test gitfile malformed")?,
    );
    let expected = common
        .join("worktrees")
        .join(checkout.file_name().context("checkout name missing")?);
    ensure!(
        gitdir == expected && gitdir.canonicalize()? == expected,
        "test gitfile changed"
    );
    ensure!(
        co.text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
            .await?
            == project
                .text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
                .await?,
        "test checkout common dir changed"
    );
    ensure!(
        co.text(&["rev-parse", "--show-toplevel"]).await?
            == checkout.to_str().context("checkout encoding")?,
        "test checkout root changed"
    );
    Ok(())
}
async fn clean(project: &PipelineGit, checkout: &Path, commit: &str) -> Result<()> {
    registered(project, checkout).await?;
    let co = PipelineGit::new(checkout)?;
    co.reject_drivers(commit).await?;
    co.reject_unsafe_index().await?;
    co.reject_ignored_protected().await?;
    ensure!(
        co.text(&["rev-parse", "HEAD"]).await? == commit,
        "test checkout head changed"
    );
    co.require_direct_ref("HEAD").await?;
    ensure!(
        co.run(&[
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignore-submodules=all"
        ])
        .await?
        .is_empty(),
        "test checkout dirty"
    );
    Ok(())
}

fn absolute_literal(path: &Path) -> Result<&str> {
    let text = path.to_str().context("Seatbelt path encoding")?;
    ensure!(
        path.is_absolute()
            && path != Path::new("/")
            && path
                .components()
                .all(|c| matches!(c, Component::RootDir | Component::Normal(_)))
            && text
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"/_-.".contains(&b)),
        "invalid Seatbelt path literal"
    );
    ensure!(
        path.canonicalize()? == path && path.is_dir(),
        "Seatbelt directory aliases"
    );
    Ok(text)
}

pub(crate) fn seatbelt(
    checkout: &Path,
    tmp: &Path,
    caches: &[PathBuf],
    work_root: &Path,
    forbidden: &[PathBuf],
) -> Result<String> {
    let home = std::env::var_os("HOME").context("HOME missing")?;
    let home = Path::new(&home).canonicalize()?;
    let forbidden = forbidden
        .iter()
        .map(|p| p.canonicalize())
        .collect::<std::io::Result<Vec<_>>>()?;
    let work_root = work_root.canonicalize()?;
    let mut grants = vec![checkout, tmp];
    grants.extend(caches.iter().map(PathBuf::as_path));
    let mut policy = String::from(
        "(version 1)\n(deny default)\n(allow process-fork)\n(allow process-exec*)\n(allow process-info* (target self))\n(allow signal (target same-sandbox))\n(allow sysctl-read)\n(allow file-read*)\n(deny network*)\n(allow file-write-data (literal \"/dev/null\"))\n",
    );
    for (index, path) in grants.into_iter().enumerate() {
        let literal = absolute_literal(path)?;
        ensure!(
            !home.starts_with(path)
                && forbidden
                    .iter()
                    .all(|p| !p.starts_with(path) && !path.starts_with(p)),
            "Seatbelt grant overlaps protected directory"
        );
        if index >= 2 {
            ensure!(
                !path.starts_with(&work_root) && !work_root.starts_with(path),
                "Seatbelt cache overlaps work_root"
            );
        } else {
            ensure!(
                path.starts_with(&work_root) && path != work_root,
                "Seatbelt grant outside work_root"
            );
        }
        policy.push_str(&format!("(allow file-write* (subpath \"{literal}\"))\n"));
    }
    Ok(policy)
}

#[cfg(all(test, target_os = "macos"))]
mod hardening;
#[cfg(test)]
pub(crate) mod tests;
