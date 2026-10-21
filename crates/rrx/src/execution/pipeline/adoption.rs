use super::{SecretScanner, TaskWorkspace, launcher::hash, workspace::ControlImage};
use crate::{execution::verification::StreamEvidence, state::pipeline::PipelineStep};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Component, Path},
};

const FILE_BYTES: usize = 1024 * 1024;
const TOTAL_BYTES: usize = 8 * 1024 * 1024;
const FILES: usize = 200;
pub struct AdoptionChecker;
pub(crate) struct Candidate {
    pub commit: String,
    pub tree: String,
    pub evidence: Value,
}
struct Change {
    path: String,
    mode: &'static str,
    bytes: Option<Vec<u8>>,
}
impl AdoptionChecker {
    pub(crate) async fn build(
        work: &TaskWorkspace,
        step: &PipelineStep,
        scope: &[String],
        attempt: &Path,
    ) -> Result<Candidate> {
        ensure!(
            step.body["death_confirmed"] == true && step.body["exit"]["issue"].is_null(),
            "exit_uncertified"
        );
        ensure!(
            step.body["exit"]["code"] == 0 && step.body["provider_outcome"] == "success",
            "agent_failed"
        );
        work.identity().await?;
        ensure!(
            work.ref_value().await? == step.step_base
                && work.git.text(&["rev-parse", "HEAD"]).await? == step.step_base,
            "agent_commit"
        );
        let before: ControlImage = serde_json::from_value(step.body["executor_controls"].clone())
            .context("git_control_changed")?;
        ensure!(work.controls().await? == before, "git_control_changed");
        work.git.reject_drivers(&step.step_base).await?;
        evidence(step, attempt)?;
        work.git.reject_unsafe_index().await?;
        work.git.reject_ignored_protected().await?;
        let status = work
            .git
            .run(&[
                "status",
                "--porcelain=v2",
                "-z",
                "--untracked-files=all",
                "--no-renames",
                "--ignore-submodules=all",
            ])
            .await?;
        let changes = changes(work.git.root(), &status, scope)?;
        let index = attempt.join(format!("index-{}", uuid::Uuid::new_v4().simple()));
        let built = async {
            work.git
                .invoke(
                    &["read-tree", &step.step_base],
                    &[],
                    Some(&index),
                    false,
                    TOTAL_BYTES / 2,
                )
                .await?;
            for change in &changes {
                let oid = if let Some(bytes) = &change.bytes {
                    String::from_utf8(
                        work.git
                            .invoke(
                                super::git::BLOB_ARGS,
                                bytes,
                                Some(&index),
                                false,
                                TOTAL_BYTES / 2,
                            )
                            .await?,
                    )?
                    .trim()
                    .to_owned()
                } else {
                    "0".repeat(step.step_base.len())
                };
                let row = format!("{} {oid}\t{}\0", change.mode, change.path);
                work.git
                    .invoke(
                        &["update-index", "-z", "--index-info"],
                        row.as_bytes(),
                        Some(&index),
                        false,
                        TOTAL_BYTES / 2,
                    )
                    .await?;
            }
            let tree = String::from_utf8(
                work.git
                    .invoke(&["write-tree"], &[], Some(&index), false, TOTAL_BYTES / 2)
                    .await?,
            )?
            .trim()
            .to_owned();
            let base_tree = work
                .git
                .text(&["rev-parse", &format!("{}^{{tree}}", step.step_base)])
                .await?;
            ensure!(tree != base_tree, "no_change");
            let message = format!(
                "rrx Executor result\n\nRrx-Run: {}\nRrx-Step: {}-{}\n",
                step.run_id, step.seq, step.attempt
            );
            let commit = String::from_utf8(
                work.git
                    .invoke(
                        &["commit-tree", &tree, "-p", &step.step_base],
                        message.as_bytes(),
                        Some(&index),
                        true,
                        TOTAL_BYTES / 2,
                    )
                    .await?,
            )?
            .trim()
            .to_owned();
            let diff = work.git.diff(&step.step_base, &commit).await?;
            SecretScanner::additions(&diff).context("diff.patch")?;
            write_artifact(&attempt.join("diff.patch"), &diff)?;
            // Recheck controls after object construction; only held bytes entered blobs.
            ensure!(
                work.controls().await? == before && work.ref_value().await? == step.step_base,
                "git_control_changed"
            );
            Ok(Candidate {
                commit,
                tree,
                evidence: json!({"diff_sha256":hash(&diff), "diff_bytes":diff.len()}),
            })
        }
        .await;
        if index.try_exists()? {
            std::fs::remove_file(&index)?;
        }
        built
    }
    pub(crate) async fn finish(work: &TaskWorkspace, step: &PipelineStep) -> Result<()> {
        ensure!(
            step.body["death_confirmed"] == true && step.body["exit"]["issue"].is_null(),
            "exit_uncertified"
        );
        ensure!(step.state == "committed", "recorded commit missing");
        let commit = step
            .result_commit
            .as_deref()
            .context("recorded commit missing")?;
        let tree = step
            .result_tree
            .as_deref()
            .context("recorded tree missing")?;
        work.identity().await?;
        let before: ControlImage = serde_json::from_value(step.body["executor_controls"].clone())
            .context("git_control_changed")?;
        ensure!(work.controls().await? == before, "git_control_changed");
        work.git.reject_drivers(commit).await?;
        // The recorded SHA and tree are identity; trailers and parent alone never identify rrx work.
        let object = work.git.text(&["cat-file", "commit", commit]).await?;
        let headers = object
            .split_once("\n\n")
            .context("invalid commit object")?
            .0;
        ensure!(
            headers
                .lines()
                .filter_map(|l| l.strip_prefix("tree "))
                .collect::<Vec<_>>()
                == [tree]
                && headers
                    .lines()
                    .filter_map(|l| l.strip_prefix("parent "))
                    .collect::<Vec<_>>()
                    == [step.step_base.as_str()],
            "recorded_commit_changed"
        );
        let current = work.ref_value().await?;
        ensure!(current == step.step_base || current == commit, "ref_moved");
        if current == step.step_base {
            work.git
                .run(&[
                    "update-ref",
                    "--no-deref",
                    &work.branch,
                    commit,
                    &step.step_base,
                ])
                .await?;
        }
        work.git.run(&["read-tree", commit]).await?;
        work.clean_at(commit).await?;
        Ok(())
    }
}
pub(crate) fn read_regular(path: &Path, maximum: usize) -> Result<Vec<u8>> {
    let meta = std::fs::symlink_metadata(path)?;
    ensure!(
        meta.is_file()
            && !meta.file_type().is_symlink()
            && meta.len() <= maximum as u64
            && meta.nlink() == 1,
        "nonregular or oversized file"
    );
    let mut f = OpenOptions::new()
        .read(true)
        .custom_flags(
            rustix::fs::OFlags::NOFOLLOW.bits() as i32 | rustix::fs::OFlags::NONBLOCK.bits() as i32,
        )
        .open(path)?;
    let opened = f.metadata()?;
    ensure!(
        opened.is_file() && opened.dev() == meta.dev() && opened.ino() == meta.ino(),
        "file changed"
    );
    let mut bytes = Vec::new();
    Read::by_ref(&mut f)
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= maximum, "file exceeds bound");
    Ok(bytes)
}
fn retained(attempt: &Path, name: &str, receipt: &Value, maximum: usize) -> Result<Vec<u8>> {
    let receipt: StreamEvidence =
        serde_json::from_value(receipt.clone()).context("output_evidence_changed")?;
    ensure!(
        receipt.complete && receipt.bytes <= maximum as u64,
        "output_evidence_changed"
    );
    let bytes = read_regular(&attempt.join(name), maximum)?;
    ensure!(
        receipt.bytes == bytes.len() as u64 && receipt.sha256 == hash(&bytes),
        "output_evidence_changed"
    );
    super::review::scan_all_raw(&bytes).with_context(|| name.to_owned())?;
    Ok(bytes)
}
pub(crate) fn evidence(step: &PipelineStep, attempt: &Path) -> Result<()> {
    ensure!(
        attempt.canonicalize()? == attempt,
        "output_evidence_changed"
    );
    retained(
        attempt,
        "stdout.jsonl",
        &step.body["exit"]["stdout"],
        16 * 1024 * 1024,
    )?;
    retained(
        attempt,
        "stderr.log",
        &step.body["exit"]["stderr"],
        FILE_BYTES,
    )?;
    retained(attempt, "last.md", &step.body["exit"]["last"], 256 * 1024)?;
    Ok(())
}
pub(crate) fn diff_evidence(step: &PipelineStep, attempt: &Path) -> Result<()> {
    let path = attempt.join("diff.patch");
    let bytes = read_regular(&path, super::git::DIFF_BYTES).context("output_evidence_changed")?;
    ensure!(
        step.body["commit_evidence"]["diff_bytes"].as_u64() == Some(bytes.len() as u64)
            && step.body["commit_evidence"]["diff_sha256"].as_str() == Some(hash(&bytes).as_str())
            && std::fs::symlink_metadata(path)?.permissions().mode() & 0o777 == 0o400,
        "output_evidence_changed"
    );
    Ok(())
}
pub(crate) fn write_artifact(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut artifact = PendingArtifact::new(path)?;
    artifact.pending.as_file_mut().write_all(bytes)?;
    artifact.publish(path)
}
struct PendingArtifact {
    pending: tempfile::NamedTempFile,
}
impl PendingArtifact {
    fn new(path: &Path) -> Result<Self> {
        // Before committed is recorded, a missing/incomplete artifact is regenerated.
        // Refuse aliases even though rename itself would not follow the destination.
        match std::fs::symlink_metadata(path) {
            Ok(_) => {
                read_regular(path, super::git::DIFF_BYTES)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let parent = path.parent().context("artifact parent missing")?;
        Ok(Self {
            pending: tempfile::Builder::new()
                .prefix("diff-pending-")
                .tempfile_in(parent)?,
        })
    }
    fn publish(mut self, path: &Path) -> Result<()> {
        let file = self.pending.as_file_mut();
        file.sync_all()?;
        file.set_permissions(std::fs::Permissions::from_mode(0o400))?;
        file.sync_all()?;
        self.pending.persist(path)?;
        std::fs::File::open(path.parent().context("artifact parent missing")?)?.sync_all()?;
        Ok(())
    }
}
fn changes(root: &Path, status: &[u8], scope: &[String]) -> Result<Vec<Change>> {
    let mut changes = Vec::new();
    let mut total = 0;
    let mut paths = std::collections::BTreeSet::new();
    for row in status.split(|b| *b == 0).filter(|s| !s.is_empty()) {
        let row = std::str::from_utf8(row).context("invalid path encoding")?;
        let (path, deleted) = if let Some(path) = row.strip_prefix("? ") {
            (path, false)
        } else {
            let parts: Vec<_> = row.splitn(9, ' ').collect();
            ensure!(
                parts.len() == 9 && parts[0] == "1" && parts[2] == "N...",
                "unsupported status entry"
            );
            ensure!(
                ![parts[3], parts[4], parts[5]]
                    .iter()
                    .any(|m| matches!(*m, "120000" | "160000")),
                "unsupported file mode"
            );
            (parts[8], parts[1].contains('D'))
        };
        ensure!(
            Path::new(path)
                .components()
                .all(|c| matches!(c, Component::Normal(_)))
                && !path.contains(['\n', '\r', '\t'])
                && paths.insert(path),
            "invalid changed path"
        );
        ensure!(
            scope.iter().any(|glob| glob_match(glob, path)),
            "scope_violation"
        );
        // Ignore rules decide which paths status can see, so no scope may change them.
        ensure!(
            super::git::fold_name(path.rsplit('/').next().unwrap_or(path)) != ".gitignore",
            "protected_path"
        );
        ensure!(
            !protected(path) || scope.iter().any(|glob| explicit(glob, path)),
            "protected_path"
        );
        ensure!(changes.len() < FILES, "too_many_files");
        let full = root.join(path);
        // Parent aliases must not redirect a permitted path outside the task worktree.
        let mut parent = full.parent().context("missing parent")?;
        while matches!(std::fs::symlink_metadata(parent), Err(e) if e.kind() == std::io::ErrorKind::NotFound)
        {
            ensure!(parent != root, "missing worktree root");
            parent = parent.parent().context("missing parent")?;
        }
        ensure!(parent.canonicalize()? == parent, "path_alias");
        let (mode, bytes) = if deleted {
            ensure!(
                std::fs::symlink_metadata(&full)
                    .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound),
                "deletion_changed"
            );
            ("0", None)
        } else {
            let meta = std::fs::symlink_metadata(&full)?;
            ensure!(
                meta.is_file() && !meta.file_type().is_symlink(),
                "unsupported_file"
            );
            let bytes = read_regular(&full, FILE_BYTES)?;
            ensure!(!bytes.contains(&0), "binary_file");
            total += bytes.len();
            ensure!(total <= TOTAL_BYTES, "files_too_large");
            (
                if meta.permissions().mode() & 0o100 != 0 {
                    "100755"
                } else {
                    "100644"
                },
                Some(bytes),
            )
        };
        changes.push(Change {
            path: path.into(),
            mode,
            bytes,
        });
    }
    Ok(changes)
}
pub(super) fn reject_ignored_entries(status: &[u8]) -> Result<()> {
    for row in status.split(|b| *b == 0) {
        if let Some(path) = row.strip_prefix(b"!! ") {
            ensure!(
                !path
                    .split(|b| *b == b'/')
                    .any(|component| { std::str::from_utf8(component).is_ok_and(protected) }),
                "protected_path"
            );
        }
    }
    Ok(())
}
fn protected(path: &str) -> bool {
    path.split('/').any(|p| {
        let p = super::git::fold_name(p);
        p.starts_with(".git")
            || matches!(
                p.as_str(),
                ".claude"
                    | ".codex"
                    | ".mcp.json"
                    | "agents.md"
                    | "agents.override.md"
                    | "claude.md"
                    | "claude.local.md"
            )
    })
}
fn explicit(glob: &str, path: &str) -> bool {
    glob_match(glob, path)
        && path
            .split('/')
            .filter(|p| protected(p))
            .all(|part| glob.split('/').any(|g| g == part))
}
/// Whole-path component matching; unsupported bracket/escape syntax refuses safely.
pub(crate) fn glob_match(glob: &str, path: &str) -> bool {
    if glob.is_empty()
        || glob.len() > 512
        || glob.starts_with('/')
        || glob.split('/').any(|p| p == ".." || p.is_empty())
        || glob.contains(['[', ']', '\\'])
    {
        return false;
    }
    fn component(pattern: &[u8], text: &[u8]) -> bool {
        let (mut p, mut t, mut star, mut retry) = (0, 0, None, 0);
        while t < text.len() {
            if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == text[t]) {
                p += 1;
                t += 1;
            } else if p < pattern.len() && pattern[p] == b'*' {
                star = Some(p);
                p += 1;
                retry = t;
            } else if let Some(s) = star {
                retry += 1;
                t = retry;
                p = s + 1;
            } else {
                return false;
            }
        }
        while p < pattern.len() && pattern[p] == b'*' {
            p += 1;
        }
        p == pattern.len()
    }
    let g: Vec<_> = glob.split('/').collect();
    let p: Vec<_> = path.split('/').collect();
    let mut current = vec![false; p.len() + 1];
    current[0] = true;
    for part in g {
        let mut next = vec![false; p.len() + 1];
        for i in 0..=p.len() {
            if part == "**" {
                next[i] = current[i] || (i > 0 && next[i - 1]);
            } else if i > 0 {
                next[i] = current[i - 1] && component(part.as_bytes(), p[i - 1].as_bytes());
            }
        }
        current = next;
    }
    current[p.len()]
}

#[cfg(test)]
mod tests;
