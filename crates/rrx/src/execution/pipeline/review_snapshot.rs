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
    let mut inventory = Vec::new();
    let mut names: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
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
                && !name.split('/').any(is_dotgit),
            "review_snapshot_unsupported"
        );
        let mut fields = std::str::from_utf8(header)?.split_whitespace();
        let mode = fields.next().context("review_snapshot_unsupported")?;
        ensure!(
            matches!(mode, "100644" | "100755") && fields.next() == Some("blob"),
            "review_snapshot_unsupported"
        );
        let oid = fields.next().context("review_snapshot_unsupported")?;
        let mut parent = PathBuf::new();
        for part in name.split('/') {
            names
                .entry(parent.clone())
                .or_default()
                .push(normalize_name(part));
            parent.push(part);
        }
        inventory.push((name.to_owned(), mode.to_owned(), oid.to_owned()));
    }
    for children in names.values() {
        reject_bare(children)?;
    }
    let oids = inventory
        .iter()
        .map(|(_, _, oid)| oid.clone())
        .collect::<Vec<_>>();
    let blobs = git.blobs(&oids).await?;
    for ((name, mode, _), bytes) in inventory.into_iter().zip(blobs) {
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

fn reject_bare(children: &[String]) -> Result<()> {
    ensure!(
        !(children.iter().any(|c| c == "head")
            && (children.iter().any(|c| c == "commondir")
                || ["objects", "refs"]
                    .iter()
                    .all(|name| children.iter().any(|c| c == name)))),
        "review_snapshot_unsupported"
    );
    Ok(())
}

fn is_dotgit(name: &str) -> bool {
    normalize_name(name) == ".git"
}

fn normalize_name(name: &str) -> String {
    // Git's next_hfs_char ignores these code points during is_hfs_dotgit.
    let visible: String = name
        .chars()
        .filter(|c| !matches!(c, '\u{200c}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{206a}'..='\u{206f}' | '\u{feff}'))
        .collect();
    super::git::fold_name(&visible)
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
    let expected = git
        .text(&["rev-parse", &format!("{commit}^{{tree}}")])
        .await?;
    let actual = digest(path, expected.len()).context("review_snapshot_changed")?;
    ensure!(actual == expected, "review_snapshot_changed");
    Ok(actual)
}
fn digest(root: &Path, oid_len: usize) -> Result<String> {
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
        let mut names = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            let name = path
                .file_name()
                .context("review_snapshot_path")?
                .to_string_lossy();
            ensure!(!is_dotgit(&name), "review_snapshot_unsupported");
            names.push(normalize_name(&name));
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
        reject_bare(&names)?;
        index += 1;
    }
    let mut children: BTreeMap<PathBuf, Vec<(Vec<u8>, String, String)>> = BTreeMap::new();
    let mut total = 0;
    for (path, mode) in files {
        let bytes = read_regular(&path, 4 * 1024 * 1024)?;
        total += bytes.len();
        ensure!(total <= 64 * 1024 * 1024, "review_snapshot_limit");
        let oid = object_id("blob", &bytes, oid_len)?;
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
        let oid = object_id("tree", &bytes, oid_len)?;
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

fn object_id(kind: &str, bytes: &[u8], oid_len: usize) -> Result<String> {
    let header = format!("{kind} {}\0", bytes.len());
    match oid_len {
        40 => {
            use sha1::Digest;
            use std::fmt::Write;
            let mut hash = sha1::Sha1::new();
            hash.update(header.as_bytes());
            hash.update(bytes);
            let mut oid = String::with_capacity(40);
            for byte in hash.finalize() {
                write!(oid, "{byte:02x}")?;
            }
            Ok(oid)
        }
        64 => {
            use sha2::Digest;
            let mut hash = sha2::Sha256::new();
            hash.update(header.as_bytes());
            hash.update(bytes);
            Ok(format!("{:x}", hash.finalize()))
        }
        _ => anyhow::bail!("review_snapshot_unsupported"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Split bare layouts are refused at every disk depth with normalized names.
    #[test]
    fn b7_snapshot_commondir_disk() {
        for prefix in ["", "nested/deeper/"] {
            for (head, common) in [
                ("HEAD", "commondir"),
                ("Head", "COMMONDIR"),
                ("HE\u{200c}AD", "commondir"),
            ] {
                let dir = tempfile::tempdir().unwrap();
                let root = dir.path().join(prefix);
                std::fs::create_dir_all(root.join("c/objects")).unwrap();
                std::fs::create_dir(root.join("c/refs")).unwrap();
                for (name, content) in [
                    (head, "ref: refs/heads/x\n"),
                    (common, "c\n"),
                    ("c/config", "[core]\nfsmonitor=untrusted\n"),
                ] {
                    let path = root.join(name);
                    std::fs::write(&path, content).unwrap();
                    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o444)).unwrap();
                }
                seal(dir.path()).unwrap();
                assert_eq!(
                    digest(dir.path(), 40).unwrap_err().to_string(),
                    "review_snapshot_unsupported"
                );
                remove(dir.path()).unwrap();
            }
        }
        assert!(reject_bare(&["head".into(), "ordinary".into()]).is_ok());
        assert!(reject_bare(&["commondir".into(), "ordinary".into()]).is_ok());
    }

    /// HFS-ignorable aliases of .git are refused on disk at every depth.
    #[test]
    fn b7_snapshot_hfs_disk() {
        for prefix in ["", "nested/deeper/"] {
            for ignored in [
                '\u{200c}', '\u{200d}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}',
                '\u{202d}', '\u{202e}', '\u{206a}', '\u{206b}', '\u{206c}', '\u{206d}', '\u{206e}',
                '\u{206f}', '\u{feff}',
            ] {
                let dir = tempfile::tempdir().unwrap();
                std::fs::create_dir_all(dir.path().join(prefix).join(format!(".g{ignored}it")))
                    .unwrap();
                seal(dir.path()).unwrap();
                assert_eq!(
                    digest(dir.path(), 40).unwrap_err().to_string(),
                    "review_snapshot_unsupported"
                );
                remove(dir.path()).unwrap();
            }
        }
        assert!(!is_dotgit(".g\u{200c}it-safe"));
        assert!(!is_dotgit("ordinary\u{200c}"));
    }

    #[cfg(target_os = "macos")]
    async fn fixture_blob(git: &PipelineGit, bytes: &[u8]) -> String {
        let oid = git
            .invoke(&["hash-object", "-w", "--stdin"], bytes, None, false, 4096)
            .await
            .unwrap();
        std::str::from_utf8(&oid).unwrap().trim().into()
    }

    #[cfg(target_os = "macos")]
    async fn fixture_tree(git: &PipelineGit, entries: &str) -> String {
        let oid = git
            .invoke(&["mktree"], entries.as_bytes(), None, false, 4096)
            .await
            .unwrap();
        std::str::from_utf8(&oid).unwrap().trim().into()
    }

    #[cfg(target_os = "macos")]
    async fn assert_tree_refused(git: &PipelineGit, root: &Path, mut tree: String, nested: bool) {
        if nested {
            tree = fixture_tree(git, &format!("040000 tree {tree}\tnested\n")).await;
        }
        let commit = git
            .invoke(&["commit-tree", &tree], b"fixture\n", None, true, 4096)
            .await
            .unwrap();
        let work = root.join("work");
        let path = snapshot_path(&work, "refused", 2, 1).unwrap();
        let error = rebuild(
            git,
            &work,
            &path,
            std::str::from_utf8(&commit).unwrap().trim(),
        )
        .await
        .unwrap_err();
        assert_eq!(error.to_string(), "review_snapshot_unsupported");
        // Tree preflight must refuse before materializing any untrusted entry.
        assert_eq!(std::fs::read_dir(&path).unwrap().count(), 0);
        remove(&path).unwrap();
    }

    /// Tree preflight refuses split bare repositories before any blob is materialized.
    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn b7_snapshot_commondir_tree() {
        for nested in [false, true] {
            for (head_name, common_name) in [
                ("HEAD", "commondir"),
                ("Head", "COMMONDIR"),
                ("HE\u{200c}AD", "commondir"),
            ] {
                let dir = tempfile::tempdir().unwrap();
                let root = dir.path().canonicalize().unwrap();
                let project = root.join("project");
                std::fs::create_dir(&project).unwrap();
                let git = PipelineGit::new(&project).unwrap();
                git.run(&["init"]).await.unwrap();
                let head = fixture_blob(&git, b"ref: refs/heads/x\n").await;
                let common = fixture_blob(&git, b"c\n").await;
                let config = fixture_blob(&git, b"[core]\nfsmonitor=untrusted\n").await;
                let child = fixture_tree(&git, &format!("100644 blob {head}\tentry\n")).await;
                let c = fixture_tree(&git, &format!("040000 tree {child}\tobjects\n040000 tree {child}\trefs\n100644 blob {config}\tconfig\n")).await;
                let tree = fixture_tree(&git, &format!("100644 blob {head}\t{head_name}\n100644 blob {common}\t{common_name}\n040000 tree {c}\tc\n")).await;
                assert_tree_refused(&git, &root, tree, nested).await;
            }
        }
    }

    /// Tree preflight refuses HFS .git aliases before any blob is materialized.
    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn b7_snapshot_hfs_tree() {
        for nested in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().canonicalize().unwrap();
            let project = root.join("project");
            std::fs::create_dir(&project).unwrap();
            let git = PipelineGit::new(&project).unwrap();
            git.run(&["init"]).await.unwrap();
            let blob = fixture_blob(&git, b"safe\n").await;
            let child = fixture_tree(&git, &format!("100644 blob {blob}\tconfig\n")).await;
            let tree = fixture_tree(&git, &format!("040000 tree {child}\t.g\u{200c}it\n")).await;
            assert_tree_refused(&git, &root, tree, nested).await;
        }
    }

    /// Every directory is checked for case-folded implicit bare-repository names.
    #[test]
    fn b7_bare_snapshot_names() {
        for prefix in ["", "nested/deeper/"] {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().join(prefix);
            std::fs::create_dir_all(root.join("OBJECTſ")).unwrap();
            std::fs::create_dir(root.join("REFſ")).unwrap();
            std::fs::write(root.join("Head"), b"ref: refs/heads/main\n").unwrap();
            std::fs::write(root.join("config"), b"[core]\nfsmonitor=untrusted\n").unwrap();
            // seal also seals the ancestors, matching a materialized snapshot.
            seal(dir.path()).unwrap();
            for name in ["Head", "config"] {
                std::fs::set_permissions(root.join(name), std::fs::Permissions::from_mode(0o444))
                    .unwrap();
            }
            let error = digest(dir.path(), 40).unwrap_err();
            assert_eq!(error.to_string(), "review_snapshot_unsupported");
            remove(dir.path()).unwrap();
        }
        assert!(reject_bare(&["head".into(), "objects".into(), "ordinary".into()]).is_ok());
    }

    /// Object headers and both repository hash formats match known Git object IDs.
    #[test]
    fn b7_object_formats() {
        assert_eq!(
            object_id("blob", b"", 40).unwrap(),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
        assert_eq!(
            object_id("tree", b"", 40).unwrap(),
            "4b825dc642cb6eb9a060e54bf8d69288fbee4904"
        );
        assert_eq!(
            object_id("blob", b"", 64).unwrap(),
            "473a0f4c3be8a93681a267e3b1e9a7dcda1185436fe141f7749120a303721813"
        );
        assert_eq!(
            object_id("tree", b"", 64).unwrap(),
            "6ef19b41225c5369f1c104d45d8d85efa9b057b53b14b4b9b939dd74decc5321"
        );
    }

    /// Snapshot plumbing spawns are bounded independently of the tree entry count.
    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn b7_snapshot_batch_spawns() {
        for format in ["sha1", "sha256"] {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().canonicalize().unwrap();
            let project = root.join("project");
            std::fs::create_dir(&project).unwrap();
            let git = PipelineGit::new(&project).unwrap();
            git.run(&["init", &format!("--object-format={format}")])
                .await
                .unwrap();
            let blob = git
                .invoke(
                    &["hash-object", "-w", "--stdin"],
                    b"safe\n",
                    None,
                    false,
                    4096,
                )
                .await
                .unwrap();
            let blob = std::str::from_utf8(&blob).unwrap().trim();
            let mut input = Vec::new();
            for i in 0..2048 {
                input.extend_from_slice(format!("100644 blob {blob}\tf{i:04}\n").as_bytes());
            }
            let tree = git
                .invoke(&["mktree"], &input, None, false, 4096)
                .await
                .unwrap();
            let tree = std::str::from_utf8(&tree).unwrap().trim();
            // Include subtrees and executable mode in the independent Git comparison.
            let tree_input =
                format!("100755 blob {blob}\texecutable\n040000 tree {tree}\tnested\n");
            let tree = git
                .invoke(&["mktree"], tree_input.as_bytes(), None, false, 4096)
                .await
                .unwrap();
            let tree = std::str::from_utf8(&tree).unwrap().trim();
            let commit = git
                .invoke(&["commit-tree", tree], b"fixture\n", None, true, 4096)
                .await
                .unwrap();
            let commit = std::str::from_utf8(&commit).unwrap().trim();
            let work = root.join("work");
            let path = snapshot_path(&work, "batch", 2, 1).unwrap();
            let before = git.spawn_count();
            assert_eq!(rebuild(&git, &work, &path, commit).await.unwrap(), tree);
            assert!(
                git.spawn_count() - before <= 10,
                "per-entry Git spawn regression"
            );
            let before = git.spawn_count();
            assert_eq!(verify(&git, &work, &path, commit).await.unwrap(), tree);
            assert_eq!(git.spawn_count() - before, 1);
            remove(&path).unwrap();
        }
    }

    /// Materialization rejects implicit bare repositories before launching a Reviewer.
    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn b7_snapshot_bare_preflight() {
        for nested in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().canonicalize().unwrap();
            let project = root.join("project");
            std::fs::create_dir(&project).unwrap();
            let git = PipelineGit::new(&project).unwrap();
            git.run(&["init"]).await.unwrap();
            let blob = git
                .invoke(
                    &["hash-object", "-w", "--stdin"],
                    b"safe\n",
                    None,
                    false,
                    4096,
                )
                .await
                .unwrap();
            let blob = std::str::from_utf8(&blob).unwrap().trim();
            let child = git
                .invoke(
                    &["mktree"],
                    format!("100644 blob {blob}\tentry\n").as_bytes(),
                    None,
                    false,
                    4096,
                )
                .await
                .unwrap();
            let child = std::str::from_utf8(&child).unwrap().trim();
            let input = format!(
                "100644 blob {blob}\tHead\n040000 tree {child}\tOBJECTſ\n040000 tree {child}\tREFſ\n100644 blob {blob}\tconfig\n"
            );
            let mut tree = git
                .invoke(&["mktree"], input.as_bytes(), None, false, 4096)
                .await
                .unwrap();
            if nested {
                let input = format!(
                    "040000 tree {}\tnested\n",
                    std::str::from_utf8(&tree).unwrap().trim()
                );
                tree = git
                    .invoke(&["mktree"], input.as_bytes(), None, false, 4096)
                    .await
                    .unwrap();
            }
            let commit = git
                .invoke(
                    &["commit-tree", std::str::from_utf8(&tree).unwrap().trim()],
                    b"fixture\n",
                    None,
                    true,
                    4096,
                )
                .await
                .unwrap();
            let work = root.join("work");
            let path = snapshot_path(&work, "bare", 2, 1).unwrap();
            let error = rebuild(
                &git,
                &work,
                &path,
                std::str::from_utf8(&commit).unwrap().trim(),
            )
            .await
            .unwrap_err();
            assert_eq!(error.to_string(), "review_snapshot_unsupported");
            remove(&path).unwrap();
        }
    }
}
