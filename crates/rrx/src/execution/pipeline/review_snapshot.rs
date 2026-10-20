//! A snapshot contains only independently materialized commit blobs, never a Task checkout.
use super::{PipelineGit, adoption::read_regular, ensure_private_directory};
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeMap,
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, PermissionsExt},
    },
    path::{Component, Path, PathBuf},
};

pub(crate) fn snapshot_path(root: &Path, run: &str, seq: u32, attempt: u32) -> Result<PathBuf> {
    ensure!(
        !run.is_empty()
            && run
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b)),
        "review_snapshot_path"
    );
    Ok(root.join("ro").join(format!("{run}-{seq}-{attempt}")))
}

pub(crate) async fn rebuild(
    git: &PipelineGit,
    root: &Path,
    path: &Path,
    commit: &str,
) -> Result<String> {
    ensure!(
        matches!(commit.len(), 40 | 64) && commit.bytes().all(|b| b.is_ascii_hexdigit()),
        "review_snapshot_changed"
    );
    ensure_private_directory(root)?;
    ensure!(root.canonicalize()? == root, "review_snapshot_changed");
    let parent = root.join("ro");
    ensure_private_directory(&parent)?;
    ensure!(
        parent.canonicalize()? == parent && path.parent() == Some(parent.as_path()),
        "review_snapshot_changed"
    );
    remove(path)?;
    git.reject_tree_drivers(commit).await?;
    ensure_private_directory(path)?;
    let entries = git.run(&["ls-tree", "-r", "-z", commit, "--"]).await?;
    let mut total = 0;
    for (index, entry) in entries
        .split(|b| *b == 0)
        .filter(|e| !e.is_empty())
        .enumerate()
    {
        ensure!(index < 100_000, "review_snapshot_limit");
        let (header, name) = super::git::split_entry(entry)?;
        let name = std::str::from_utf8(name)?;
        ensure!(
            !name.is_empty()
                && Path::new(name)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_)))
                && !name.split('/').any(|n| n.eq_ignore_ascii_case(".git")),
            "review_snapshot_unsupported"
        );
        let mut fields = std::str::from_utf8(header)?.split_whitespace();
        let mode = fields.next().context("review_snapshot_unsupported")?;
        ensure!(
            matches!(mode, "100644" | "100755") && fields.next() == Some("blob"),
            "review_snapshot_unsupported"
        );
        let oid = fields.next().context("review_snapshot_unsupported")?;
        let bytes = git.run(&["cat-file", "blob", oid]).await?;
        total += bytes.len();
        ensure!(total <= 64 * 1024 * 1024, "review_snapshot_limit");
        let file = path.join(name);
        std::fs::create_dir_all(file.parent().context("review_snapshot_path")?)?;
        super::adoption::write_artifact(&file, &bytes)?;
        std::fs::set_permissions(
            file,
            std::fs::Permissions::from_mode(if mode == "100755" { 0o555 } else { 0o444 }),
        )?;
    }
    seal(path)?;
    verify(git, root, path, commit).await
}

fn seal(path: &Path) -> Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if std::fs::symlink_metadata(entry.path())?.is_dir() {
            seal(&entry.path())?;
        }
    }
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o555))?;
    Ok(())
}
pub(crate) fn remove(path: &Path) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
        Ok(m) if !m.is_dir() => {
            std::fs::remove_file(path)?;
            return Ok(());
        }
        Ok(_) => {}
    }
    // Never follow symlinks or chmod their referents during cleanup.
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    for entry in std::fs::read_dir(path)? {
        remove(&entry?.path())?;
    }
    std::fs::remove_dir(path)?;
    Ok(())
}

pub(crate) async fn verify(
    git: &PipelineGit,
    root: &Path,
    path: &Path,
    commit: &str,
) -> Result<String> {
    ensure!(
        root.canonicalize()? == root
            && path.canonicalize()? == path
            && path.parent() == Some(root.join("ro").as_path()),
        "review_snapshot_changed"
    );
    let actual = digest(git, path).await.context("review_snapshot_changed")?;
    let expected = git
        .text(&["rev-parse", &format!("{commit}^{{tree}}")])
        .await?;
    ensure!(actual == expected, "review_snapshot_changed");
    Ok(actual)
}
async fn digest(git: &PipelineGit, root: &Path) -> Result<String> {
    let mut dirs = vec![root.to_path_buf()];
    let mut files = Vec::new();
    let mut index = 0;
    while index < dirs.len() {
        let dir = &dirs[index];
        let meta = std::fs::symlink_metadata(dir)?;
        ensure!(
            meta.is_dir()
                && meta.uid() == rustix::process::getuid().as_raw()
                && meta.permissions().mode() & 0o222 == 0,
            "review_snapshot_changed"
        );
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            let meta = std::fs::symlink_metadata(&path)?;
            ensure!(
                meta.permissions().mode() & 0o222 == 0
                    && meta.uid() == rustix::process::getuid().as_raw(),
                "review_snapshot_changed"
            );
            if meta.is_dir() {
                dirs.push(path);
            } else {
                ensure!(
                    meta.is_file() && meta.nlink() == 1,
                    "review_snapshot_changed"
                );
                files.push((
                    path,
                    if meta.permissions().mode() & 0o100 != 0 {
                        "100755"
                    } else {
                        "100644"
                    },
                ));
            }
            ensure!(dirs.len() + files.len() <= 100_000, "review_snapshot_limit");
        }
        index += 1;
    }
    let mut children: BTreeMap<PathBuf, Vec<(Vec<u8>, String, String)>> = BTreeMap::new();
    let mut total = 0;
    for (path, mode) in files {
        let bytes = read_regular(&path, 4 * 1024 * 1024)?;
        total += bytes.len();
        ensure!(total <= 64 * 1024 * 1024, "review_snapshot_limit");
        let oid = String::from_utf8(
            git.invoke(
                &["hash-object", "--no-filters", "--stdin"],
                &bytes,
                None,
                false,
                4096,
            )
            .await?,
        )?
        .trim()
        .to_owned();
        children
            .entry(path.parent().context("review_snapshot_path")?.into())
            .or_default()
            .push((
                path.file_name()
                    .context("review_snapshot_path")?
                    .as_bytes()
                    .to_vec(),
                mode.into(),
                oid,
            ));
    }
    for dir in dirs.into_iter().rev() {
        let mut rows = children.remove(&dir).unwrap_or_default();
        rows.sort_by_key(|(name, mode, _)| {
            let mut key = name.clone();
            if mode == "40000" {
                key.push(b'/');
            }
            key
        });
        let mut bytes = Vec::new();
        for (name, mode, oid) in rows {
            bytes.extend_from_slice(mode.as_bytes());
            bytes.push(b' ');
            bytes.extend(name);
            bytes.push(0);
            for pair in oid.as_bytes().chunks_exact(2) {
                bytes.push(u8::from_str_radix(std::str::from_utf8(pair)?, 16)?);
            }
        }
        let oid = String::from_utf8(
            git.invoke(
                &["hash-object", "-t", "tree", "--stdin"],
                &bytes,
                None,
                false,
                4096,
            )
            .await?,
        )?
        .trim()
        .to_owned();
        if dir == root {
            return Ok(oid);
        }
        children
            .entry(dir.parent().context("review_snapshot_path")?.into())
            .or_default()
            .push((
                dir.file_name()
                    .context("review_snapshot_path")?
                    .as_bytes()
                    .to_vec(),
                "40000".into(),
                oid,
            ));
    }
    anyhow::bail!("review_snapshot_changed")
}
