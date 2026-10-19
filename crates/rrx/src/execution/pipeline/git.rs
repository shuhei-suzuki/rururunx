//! Filter-free, bounded Git plumbing; never inherits repository routing or credentials.
use crate::execution::process::{OwnedProcess, bounded_read};
use anyhow::{Context, Result, ensure};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::io::AsyncWriteExt;

pub(crate) const DIFF_BYTES: usize = 8 * 1024 * 1024;
const OUTPUT_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const BLOB_ARGS: &[&str] = &["hash-object", "-w", "--no-filters", "--stdin"];
const OPTIONS: &[&str] = &[
    "core.hooksPath=/dev/null",
    "core.fsmonitor=false",
    "core.untrackedCache=false",
    "gc.auto=0",
    "maintenance.auto=false",
    "commit.gpgsign=false",
    "core.autocrlf=false",
    "diff.ignoreSubmodules=all",
    "submodule.recurse=false",
    "core.attributesFile=/dev/null",
    "core.excludesFile=/dev/null",
];

#[derive(Clone)]
pub struct PipelineGit {
    root: PathBuf,
    home: OsString,
}
impl PipelineGit {
    pub fn new(root: &Path) -> Result<Self> {
        ensure!(root.canonicalize()? == root, "Git root aliases");
        Ok(Self {
            root: root.into(),
            home: std::env::var_os("HOME").context("HOME missing")?,
        })
    }
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
    fn command(
        &self,
        args: &[&str],
        index: Option<&Path>,
        identity: bool,
    ) -> tokio::process::Command {
        let mut command = tokio::process::Command::new("/usr/bin/git");
        command.arg("--no-replace-objects");
        command
            .current_dir(&self.root)
            .env_clear()
            .env("HOME", &self.home)
            .env("PATH", "/usr/bin:/bin")
            .env("LANG", "en_US.UTF-8")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_NO_LAZY_FETCH", "1")
            .env("GIT_ATTR_NOSYSTEM", "1");
        if let Some(index) = index {
            command.env("GIT_INDEX_FILE", index);
        }
        if identity {
            for key in ["GIT_AUTHOR_NAME", "GIT_COMMITTER_NAME"] {
                command.env(key, "rrx");
            }
            for key in ["GIT_AUTHOR_EMAIL", "GIT_COMMITTER_EMAIL"] {
                command.env(key, "rrx@localhost");
            }
        }
        for option in OPTIONS {
            command.args(["-c", option]);
        }
        command
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }
    pub(crate) async fn run(&self, args: &[&str]) -> Result<Vec<u8>> {
        self.invoke(args, &[], None, false, OUTPUT_BYTES).await
    }
    pub(crate) async fn text(&self, args: &[&str]) -> Result<String> {
        Ok(String::from_utf8(self.run(args).await?)?
            .trim_end_matches('\n')
            .into())
    }
    pub(crate) async fn invoke(
        &self,
        args: &[&str],
        input: &[u8],
        index: Option<&Path>,
        identity: bool,
        limit: usize,
    ) -> Result<Vec<u8>> {
        let (out, status) = self.execute(args, input, index, identity, limit).await?;
        ensure!(status.success(), "Git command refused");
        Ok(out)
    }
    async fn execute(
        &self,
        args: &[&str],
        input: &[u8],
        index: Option<&Path>,
        identity: bool,
        limit: usize,
    ) -> Result<(Vec<u8>, std::process::ExitStatus)> {
        let mut child = OwnedProcess::spawn(self.command(args, index, identity))?;
        let mut stdin = child.child.stdin.take().context("Git stdin missing")?;
        let stdout = child.child.stdout.take().context("Git stdout missing")?;
        let stderr = child.child.stderr.take().context("Git stderr missing")?;
        // Keep the unreaped child owned through timeout, overflow and input errors.
        let observed = tokio::time::timeout(Duration::from_secs(10), async {
            let write = async {
                stdin.write_all(input).await?;
                stdin.shutdown().await?;
                drop(stdin);
                Ok::<_, anyhow::Error>(())
            };
            let (_, out, err, ()) = tokio::try_join!(
                write,
                bounded_read(stdout, limit),
                bounded_read(stderr, OUTPUT_BYTES),
                child.exited()
            )?;
            ensure!(
                out.len().saturating_add(err.len()) <= limit,
                "Git combined output exceeds bound"
            );
            Ok::<_, anyhow::Error>(out)
        })
        .await;
        let receipt = child.stop_and_reap_probe().await?;
        ensure!(
            receipt.group_error.is_none(),
            "Git group cleanup unconfirmed"
        );
        let out = observed.context("Git timeout")??;
        Ok((out, receipt.status))
    }
    pub(crate) async fn diff(&self, base: &str, commit: &str) -> Result<Vec<u8>> {
        let result = self
            .invoke(
                &[
                    "diff-tree",
                    "-p",
                    "--text",
                    "--no-ext-diff",
                    "--no-textconv",
                    base,
                    commit,
                    "--",
                ],
                &[],
                None,
                false,
                DIFF_BYTES,
            )
            .await;
        match result {
            Err(error)
                if error.chain().any(|e| {
                    matches!(
                        e.to_string().as_str(),
                        "command output exceeded its bound" | "Git combined output exceeds bound"
                    )
                }) =>
            {
                Err(error.context("diff_too_large"))
            }
            other => other,
        }
    }
    pub(crate) async fn local_config(&self) -> Result<Vec<u8>> {
        read_local_config(|args| self.run(args)).await
    }
    pub(crate) async fn reject_drivers(&self, tree: &str) -> Result<()> {
        self.reject_tree_drivers(tree).await?;
        scan_attributes(&self.root)
    }
    /// The Agent's index can hide paths from status even when the base has no gitlinks.
    pub(crate) async fn reject_unsafe_index(&self) -> Result<()> {
        let entries = self.run(&["ls-files", "--stage", "-z"]).await?;
        reject_index_entries(&entries)?;
        let flags = self.run(&["ls-files", "-v", "-z"]).await?;
        reject_index_flags(&flags)
    }
    /// Ignored instruction/control paths must not survive into the next step.
    pub(crate) async fn reject_ignored_protected(&self) -> Result<()> {
        let status = self
            .run(&[
                "status",
                "--porcelain=v1",
                "-z",
                "--ignored=matching",
                "--untracked-files=all",
                "--ignore-submodules=all",
            ])
            .await?;
        super::adoption::reject_ignored_entries(&status)
    }
    /// Project working files are not inputs to materializing the selected tree.
    pub(crate) async fn reject_tree_drivers(&self, tree: &str) -> Result<()> {
        let config = self.local_config().await?;
        for item in config.split(|b| *b == 0) {
            let key = item.split(|b| *b == b'\n').next().unwrap_or_default();
            let key = String::from_utf8_lossy(key).to_ascii_lowercase();
            ensure!(!driver_key(&key), "git_driver_declared");
        }
        // No checkout/status is allowed before inspecting every tree attribute blob.
        let entries = self.run(&["ls-tree", "-r", "-z", tree, "--"]).await?;
        for entry in entries.split(|b| *b == 0).filter(|s| !s.is_empty()) {
            let (header, path) = split_entry(entry)?;
            ensure!(!header.starts_with(b"160000 "), "git_driver_declared");
            if path
                .rsplit(|b| *b == b'/')
                .next()
                .is_some_and(|name| fold_name(&String::from_utf8_lossy(name)) == ".gitattributes")
            {
                let oid = std::str::from_utf8(header)?
                    .split_whitespace()
                    .nth(2)
                    .context("tree oid missing")?;
                reject_attributes(&self.run(&["cat-file", "blob", oid]).await?)?;
            }
        }
        let common = self
            .text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
            .await?;
        let info = Path::new(&common).join("info/attributes");
        if info.try_exists()? {
            reject_attributes(&super::adoption::read_regular(&info, OUTPUT_BYTES)?)?;
        }
        Ok(())
    }
    pub(crate) async fn require_direct_ref(&self, name: &str) -> Result<()> {
        // Exit 1 proves a direct or absent ref; dangling symrefs also exit 0.
        let (_, status) = self
            .execute(
                &["symbolic-ref", "--quiet", "--no-recurse", name],
                &[],
                None,
                false,
                OUTPUT_BYTES,
            )
            .await?;
        ensure!(status.code() == Some(1), "symbolic_pipeline_ref");
        Ok(())
    }
}
/// Compare control names with ASCII casing and Unicode upper-then-lower folding.
pub(super) fn fold_name(name: &str) -> String {
    name.to_uppercase().to_lowercase()
}
fn reject_index_entries(entries: &[u8]) -> Result<()> {
    for entry in entries.split(|b| *b == 0).filter(|s| !s.is_empty()) {
        ensure!(!entry.starts_with(b"160000 "), "git_driver_declared");
    }
    Ok(())
}
fn reject_index_flags(entries: &[u8]) -> Result<()> {
    for entry in entries.split(|b| *b == 0).filter(|s| !s.is_empty()) {
        let tag = entry[0];
        ensure!(
            !tag.is_ascii_lowercase() && tag != b'S',
            "git_driver_declared"
        );
    }
    Ok(())
}
async fn read_local_config<F, Fut>(mut read: F) -> Result<Vec<u8>>
where
    F: FnMut(&'static [&'static str]) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<u8>>>,
{
    // Git accepts only one scope switch; includes and origins belong to each scope.
    let mut bytes = read(&[
        "config",
        "--includes",
        "--list",
        "--show-origin",
        "-z",
        "--local",
    ])
    .await?;
    let enabled = read(&[
        "config",
        "--includes",
        "--local",
        "--type=bool",
        "--default=false",
        "--get",
        "extensions.worktreeConfig",
    ])
    .await?;
    ensure!(
        matches!(enabled.as_slice(), b"true\n" | b"false\n"),
        "invalid worktreeConfig value"
    );
    // Without the extension Git ignores config.worktree and may refuse --worktree.
    if enabled == b"true\n" {
        let path = read(&[
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "config.worktree",
        ])
        .await?;
        let path = Path::new(
            std::str::from_utf8(&path)?
                .strip_suffix('\n')
                .context("Git path missing newline")?,
        );
        ensure!(path.is_absolute(), "worktree config path is not absolute");
        match std::fs::symlink_metadata(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(bytes),
            Ok(metadata) => ensure!(metadata.is_file(), "worktree config is not a regular file"),
            Err(error) => return Err(error.into()),
        }
        bytes.extend(
            read(&[
                "config",
                "--includes",
                "--list",
                "--show-origin",
                "-z",
                "--worktree",
            ])
            .await?,
        );
    }
    Ok(bytes)
}
fn driver_key(key: &str) -> bool {
    key.starts_with("filter.")
        || key == "diff.external"
        || key == "core.attributesfile"
        || (key.starts_with("diff.") && (key.ends_with(".textconv") || key.ends_with(".command")))
        || (key.starts_with("merge.") && key.ends_with(".driver"))
}
fn reject_attributes(bytes: &[u8]) -> Result<()> {
    for line in bytes.split(|b| *b == b'\n') {
        if line
            .iter()
            .find(|b| !b.is_ascii_whitespace())
            .is_none_or(|b| *b == b'#')
        {
            continue;
        }
        for word in line.split(|b| b.is_ascii_whitespace()).skip(1) {
            let word = word
                .strip_prefix(b"-")
                .or_else(|| word.strip_prefix(b"!"))
                .unwrap_or(word);
            let key = word.split(|b| *b == b'=').next().unwrap_or_default();
            ensure!(
                ![
                    b"filter".as_slice(),
                    b"diff",
                    b"merge",
                    b"working-tree-encoding",
                    b"binary",
                    b"ident"
                ]
                .contains(&key),
                "git_driver_declared"
            );
        }
    }
    Ok(())
}
fn scan_attributes(root: &Path) -> Result<()> {
    // Bounded traversal includes ignored and untracked attributes, without following links.
    let mut pending = vec![root.to_owned()];
    let mut count = 0;
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            count += 1;
            ensure!(count <= 100_000, "attribute inventory exceeds bound");
            if entry.file_name() == ".git" {
                continue;
            }
            let meta = std::fs::symlink_metadata(entry.path())?;
            if fold_name(&entry.file_name().to_string_lossy()) == ".gitattributes" {
                reject_attributes(&super::adoption::read_regular(&entry.path(), OUTPUT_BYTES)?)?;
            }
            if meta.is_dir() {
                pending.push(entry.path());
            }
        }
    }
    Ok(())
}
pub(crate) fn split_entry(entry: &[u8]) -> Result<(&[u8], &[u8])> {
    let at = entry
        .iter()
        .position(|b| *b == b'\t')
        .context("malformed tree entry")?;
    Ok((&entry[..at], &entry[at + 1..]))
}

#[cfg(test)]
mod tests;
