use super::{PipelineGit, adoption::read_regular, launcher::hash};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

/// Only a bound, private task worktree may be reset by the pipeline.
pub struct TaskWorkspace {
    pub(crate) git: PipelineGit,
    pub(crate) branch: String,
    pub(crate) base: String,
    project: PathBuf,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ControlImage {
    config: String,
    files: BTreeMap<String, String>,
}
impl TaskWorkspace {
    pub fn bound(
        project: &Path,
        work_root: &Path,
        work: &Path,
        branch: &str,
        base: &str,
    ) -> Result<Self> {
        ensure!(
            work_root.canonicalize()? == work_root && project.canonicalize()? == project,
            "worktree_drift"
        );
        ensure!(
            work.starts_with(work_root)
                && work != work_root
                && !work.starts_with(project)
                && !project.starts_with(work_root),
            "worktree_drift"
        );
        ensure!(
            branch.starts_with("rrx/")
                && branch.split('/').all(|p| !p.is_empty()
                    && p.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))),
            "invalid task branch"
        );
        ensure!(
            matches!(base.len(), 40 | 64) && base.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid step base"
        );
        Ok(Self {
            git: PipelineGit::new(work)?,
            branch: format!("refs/heads/{branch}"),
            base: base.into(),
            project: project.into(),
        })
    }
    pub(crate) async fn identity(&self) -> Result<()> {
        let project_git = PipelineGit::new(&self.project)?;
        let common = project_git
            .text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
            .await?;
        ensure!(
            self.git
                .text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
                .await?
                == common,
            "git_control_changed"
        );
        ensure!(
            self.git.text(&["rev-parse", "--show-toplevel"]).await?
                == self.git.root().to_str().context("work path encoding")?,
            "worktree_drift"
        );
        let listing = project_git
            .run(&["worktree", "list", "--porcelain", "-z"])
            .await?;
        let expected = format!("worktree {}", self.git.root().display());
        ensure!(
            listing
                .split(|b| *b == 0)
                .any(|field| field == expected.as_bytes()),
            "worktree registration missing"
        );
        ensure!(
            self.git.text(&["symbolic-ref", "HEAD"]).await? == self.branch,
            "agent_commit"
        );
        Ok(())
    }
    pub(crate) async fn ref_value(&self) -> Result<String> {
        self.git.require_direct_ref(&self.branch).await?;
        self.git
            .text(&["rev-parse", "--verify", &self.branch])
            .await
    }
    pub(crate) async fn controls(&self) -> Result<ControlImage> {
        self.identity().await?;
        let mut files = BTreeMap::new();
        inventory(&self.git.root().join(".git"), ".git", &mut files)?;
        let common = self
            .git
            .text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
            .await?;
        for name in ["config", "config.worktree", "hooks", "info"] {
            inventory(&Path::new(&common).join(name), name, &mut files)?;
        }
        let dir = self.git.text(&["rev-parse", "--absolute-git-dir"]).await?;
        inventory(
            &Path::new(&dir).join("config.worktree"),
            "task/config.worktree",
            &mut files,
        )?;
        let config = hash(
            &self
                .git
                .run(&["config", "--includes", "--list", "--show-origin", "-z"])
                .await?,
        );
        Ok(ControlImage { config, files })
    }
    pub(crate) async fn clean_at(&self, commit: &str) -> Result<()> {
        self.identity().await?;
        self.git.reject_drivers(commit).await?;
        ensure!(
            self.ref_value().await? == commit
                && self.git.text(&["rev-parse", "HEAD"]).await? == commit,
            "worktree_drift"
        );
        self.git.reject_unsafe_index().await?;
        self.git.reject_ignored_protected().await?;
        ensure!(
            self.git
                .run(&[
                    "status",
                    "--porcelain=v1",
                    "-z",
                    "--untracked-files=all",
                    "--ignore-submodules=all",
                ])
                .await?
                .is_empty(),
            "worktree_drift"
        );
        Ok(())
    }
    /// The caller has confirmed death and holds control_admission; no unowned path is removed.
    pub(crate) async fn initialize(&self, expected: &ControlImage) -> Result<()> {
        ensure!(&self.controls().await? == expected, "git_control_changed");
        self.git.reject_drivers(&self.base).await?;
        self.git.run(&["reset", "--hard", &self.base]).await?;
        self.git.run(&["clean", "-ffdx"]).await?;
        self.clean_at(&self.base).await?;
        ensure!(&self.controls().await? == expected, "git_control_changed");
        Ok(())
    }
}
fn inventory(path: &Path, key: &str, files: &mut BTreeMap<String, String>) -> Result<()> {
    ensure!(files.len() < 4096, "Git controls exceed bound");
    // Control filenames are untrusted too; persist only their path digests.
    let key_hash = hash(key.as_bytes());
    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            files.insert(key_hash, "absent".into());
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };
    ensure!(!meta.file_type().is_symlink(), "git_control_changed");
    if meta.is_dir() {
        files.insert(key_hash, format!("dir:{}", meta.permissions().mode()));
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            inventory(
                &entry.path(),
                &format!(
                    "{key}/{}",
                    entry
                        .file_name()
                        .to_str()
                        .context("control filename encoding")?
                ),
                files,
            )?;
        }
    } else {
        let bytes = read_regular(path, 4 * 1024 * 1024)?;
        files.insert(
            key_hash,
            format!("{}:{}", meta.permissions().mode(), hash(&bytes)),
        );
    }
    Ok(())
}
