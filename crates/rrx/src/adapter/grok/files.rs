//! Scoped ACP text filesystem. Path strings never authorize a followed symlink.
use super::{AdapterResult, ErrorKind, failure};
use crate::domain::Project;
use rustix::fs::{Mode, OFlags, open, openat};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    os::unix::fs::MetadataExt,
    path::{Component, Path, PathBuf},
};

const FILE_LIMIT: usize = 1_048_576;
const INVENTORY_LIMIT: usize = 16_777_216;
const FILES_LIMIT: usize = 4096;

pub(super) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn io_error(error: impl std::fmt::Display) -> crate::adapter::AdapterError {
    failure(
        ErrorKind::InvalidInput,
        format!("scoped text filesystem: {error}"),
    )
}
fn identity(file: &File) -> AdapterResult<(u64, u64)> {
    let m = file.metadata().map_err(io_error)?;
    Ok((m.dev(), m.ino()))
}
fn open_dir(path: &Path) -> AdapterResult<File> {
    open(
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map(File::from)
    .map_err(io_error)
}
fn regular(file: &File) -> AdapterResult<()> {
    let m = file.metadata().map_err(io_error)?;
    if !m.is_file() || m.nlink() != 1 || m.len() > FILE_LIMIT as u64 {
        return Err(failure(
            ErrorKind::InvalidInput,
            "text file must be bounded regular single-link file",
        ));
    }
    Ok(())
}
fn text(file: &mut File) -> AdapterResult<String> {
    regular(file)?;
    let mut bytes = vec![];
    file.take((FILE_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() > FILE_LIMIT || bytes.contains(&0) {
        return Err(failure(
            ErrorKind::InvalidInput,
            "binary/oversized text file unsupported",
        ));
    }
    String::from_utf8(bytes).map_err(io_error)
}

fn protected_name(relative: &Path) -> AdapterResult<PathBuf> {
    let name = relative.to_str().filter(|s| s.is_ascii()).ok_or_else(|| {
        failure(
            ErrorKind::InvalidConfiguration,
            "Task rule/config paths must be ASCII",
        )
    })?;
    Ok(PathBuf::from(name.to_ascii_lowercase()))
}
pub(super) struct ScopedFiles {
    root: PathBuf,
    pinned: File,
    protected: BTreeSet<PathBuf>,
    protected_inodes: BTreeSet<(u64, u64)>,
    /// Existing writes require an unchanged successful prior read.
    reads: BTreeMap<PathBuf, String>,
    pub writes: BTreeMap<PathBuf, String>,
}
impl ScopedFiles {
    pub fn new(root: PathBuf, project: &Project) -> AdapterResult<Self> {
        if root.canonicalize().map_err(io_error)? != root {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "Task root must be canonical",
            ));
        }
        let pinned = open_dir(&root)?;
        let mut protected = BTreeSet::new();
        let mut protected_inodes = BTreeSet::new();
        for path in project.rule_refs.iter().chain(project.config_ref.iter()) {
            if !path.is_absolute() {
                return Err(failure(
                    ErrorKind::InvalidConfiguration,
                    "Project rule/config refs must be absolute",
                ));
            }
            let canonical = path.canonicalize().map_err(io_error)?;
            if path.as_os_str() != canonical.as_os_str() {
                return Err(failure(
                    ErrorKind::InvalidConfiguration,
                    "Project rule/config refs must be canonical without aliases",
                ));
            }
            if !canonical.metadata().map_err(io_error)?.is_file() {
                return Err(failure(
                    ErrorKind::InvalidConfiguration,
                    "Project rule/config refs must name files",
                ));
            }
            if let Ok(relative) = canonical.strip_prefix(&project.root) {
                protected.insert(protected_name(relative)?);
                if let Ok(metadata) = root.join(relative).metadata() {
                    protected_inodes.insert((metadata.dev(), metadata.ino()));
                }
            }
            if let Ok(relative) = canonical.strip_prefix(&root) {
                protected.insert(protected_name(relative)?);
            }
            let metadata = canonical.metadata().map_err(io_error)?;
            protected_inodes.insert((metadata.dev(), metadata.ino()));
        }
        Ok(Self {
            root,
            pinned,
            protected,
            protected_inodes,
            reads: BTreeMap::new(),
            writes: BTreeMap::new(),
        })
    }
    fn relative(&self, path: &str) -> AdapterResult<PathBuf> {
        let candidate = Path::new(path);
        let relative = if candidate.is_absolute() {
            candidate.strip_prefix(&self.root).map_err(io_error)?
        } else {
            candidate
        };
        if relative.as_os_str().is_empty()
            || relative.components().any(|component| match component {
                Component::Normal(name) => name.to_str().is_none_or(|s| {
                    !s.is_ascii()
                        || s.starts_with('.')
                        || matches!(
                            s.to_ascii_lowercase().as_str(),
                            "agents.md" | "claude.md" | "grok.md"
                        )
                }),
                _ => true,
            })
            || self.protected.contains(&PathBuf::from(
                relative.to_string_lossy().to_ascii_lowercase(),
            ))
        {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "foreign, aliased or protected file path",
            ));
        }
        Ok(relative.to_path_buf())
    }
    fn parent(&self, relative: &Path) -> AdapterResult<(File, PathBuf, String)> {
        if identity(&open_dir(&self.root)?)? != identity(&self.pinned)? {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "Task root was replaced",
            ));
        }
        let parent_path = relative.parent().unwrap_or(Path::new(""));
        let mut fd = self.pinned.try_clone().map_err(io_error)?;
        for component in parent_path.components() {
            let Component::Normal(name) = component else {
                return Err(failure(ErrorKind::OwnershipMismatch, "invalid parent path"));
            };
            fd = File::from(
                openat(
                    &fd,
                    name,
                    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                    Mode::empty(),
                )
                .map_err(io_error)?,
            );
        }
        let name = relative
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| failure(ErrorKind::InvalidInput, "invalid basename"))?
            .to_owned();
        Ok((fd, parent_path.to_path_buf(), name))
    }
    fn verify_parent(&self, relative: &Path, parent: &File) -> AdapterResult<()> {
        let (current, _, _) = self.parent(relative)?;
        if identity(&current)? != identity(parent)? {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "file parent was replaced",
            ));
        }
        Ok(())
    }
    fn check_protected_inode(&self, file: &File) -> AdapterResult<()> {
        if self.protected_inodes.contains(&identity(file)?) {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "configured rule/config inode is protected",
            ));
        }
        Ok(())
    }
    pub fn read(&mut self, params: &Value) -> AdapterResult<Value> {
        if params.get("line").is_some() || params.get("limit").is_some() {
            return Err(failure(ErrorKind::InvalidInput, "ranged reads unsupported"));
        }
        let relative = self.relative(
            params["path"]
                .as_str()
                .ok_or_else(|| failure(ErrorKind::InvalidInput, "missing read path"))?,
        )?;
        let (parent, _, name) = self.parent(&relative)?;
        let mut file = File::from(
            openat(
                &parent,
                name.as_str(),
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(io_error)?,
        );
        self.check_protected_inode(&file)?;
        let content = text(&mut file)?;
        self.verify_parent(&relative, &parent)?;
        self.reads.insert(relative, digest(content.as_bytes()));
        Ok(json!({"content":content}))
    }
    /// Return factual outcome separately so caller audits actual effects before policy rechecks.
    pub fn write(&mut self, params: &Value) -> AdapterResult<Value> {
        let relative = self.relative(
            params["path"]
                .as_str()
                .ok_or_else(|| failure(ErrorKind::InvalidInput, "missing write path"))?,
        )?;
        let content = params["content"]
            .as_str()
            .ok_or_else(|| failure(ErrorKind::InvalidInput, "missing text content"))?;
        if content.len() > FILE_LIMIT || content.contains('\0') {
            return Err(failure(
                ErrorKind::InvalidInput,
                "binary/oversized write unsupported",
            ));
        }
        let (parent, _, name) = self.parent(&relative)?;
        let existing = openat(
            &parent,
            name.as_str(),
            OFlags::RDWR | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        );
        let mut file = match existing {
            Ok(fd) => {
                let mut file = File::from(fd);
                self.check_protected_inode(&file)?;
                let prior = text(&mut file)?;
                if self.reads.get(&relative) != Some(&digest(prior.as_bytes())) {
                    return Err(failure(
                        ErrorKind::StateConflict,
                        "existing write requires unchanged prior ACP read",
                    ));
                }
                let check = File::from(
                    openat(
                        &parent,
                        name.as_str(),
                        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                        Mode::empty(),
                    )
                    .map_err(io_error)?,
                );
                if identity(&check)? != identity(&file)? {
                    return Err(failure(
                        ErrorKind::OwnershipMismatch,
                        "target replaced before write",
                    ));
                }
                file
            }
            Err(rustix::io::Errno::NOENT) => File::from(
                openat(
                    &parent,
                    name.as_str(),
                    OFlags::WRONLY
                        | OFlags::CREATE
                        | OFlags::EXCL
                        | OFlags::NOFOLLOW
                        | OFlags::NONBLOCK
                        | OFlags::CLOEXEC,
                    Mode::from_raw_mode(0o600),
                )
                .map_err(io_error)?,
            ),
            Err(error) => return Err(io_error(error)),
        };
        regular(&file)?;
        self.verify_parent(&relative, &parent)?;
        // Effects may be partial on I/O failure: they remain in the reconciliation journal.
        let result = (|| {
            file.seek(SeekFrom::Start(0)).map_err(io_error)?;
            file.write_all(content.as_bytes()).map_err(io_error)?;
            file.set_len(content.len() as u64).map_err(io_error)?;
            file.sync_data().map_err(io_error)
        })();
        self.writes
            .insert(relative.clone(), fingerprint(&file, content.as_bytes())?);
        result?;
        self.verify_parent(&relative, &parent)?;
        let current = File::from(
            openat(
                &parent,
                name.as_str(),
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(io_error)?,
        );
        if identity(&current)? != identity(&file)? {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "target replaced after write",
            ));
        }
        self.reads.insert(relative, digest(content.as_bytes()));
        Ok(json!({}))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Inventory(pub BTreeMap<PathBuf, String>);
fn fingerprint(file: &File, bytes: &[u8]) -> AdapterResult<String> {
    let m = file.metadata().map_err(io_error)?;
    Ok(format!(
        "file:{}:{}:{}:{}:{}",
        m.dev(),
        m.ino(),
        m.mode(),
        m.nlink(),
        digest(bytes)
    ))
}
impl Inventory {
    pub fn capture(root: &Path) -> AdapterResult<Self> {
        fn walk(
            directory: &File,
            prefix: &Path,
            values: &mut BTreeMap<PathBuf, String>,
            total: &mut usize,
            depth: usize,
        ) -> AdapterResult<()> {
            use std::os::unix::ffi::OsStringExt;
            if depth > 32 {
                return Err(failure(
                    ErrorKind::InvalidInput,
                    "inventory directory depth exceeds budget",
                ));
            }
            let mut entries = rustix::fs::Dir::read_from(directory)
                .map_err(io_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(io_error)?;
            entries.sort_by_key(|entry| entry.file_name().to_bytes().to_vec());
            for entry in entries {
                let name = entry.file_name();
                if name.to_bytes() == b"." || name.to_bytes() == b".." {
                    continue;
                }
                if values.len() >= FILES_LIMIT {
                    return Err(failure(
                        ErrorKind::InvalidInput,
                        "inventory file count exceeds budget",
                    ));
                }
                let relative = prefix.join(std::ffi::OsString::from_vec(name.to_bytes().to_vec()));
                if entry.file_type() == rustix::fs::FileType::Symlink {
                    let target =
                        rustix::fs::readlinkat(directory, name, Vec::new()).map_err(io_error)?;
                    values.insert(
                        relative,
                        format!("symlink:{}:{}", entry.ino(), digest(target.as_bytes())),
                    );
                    continue;
                }
                let mut file = File::from(
                    openat(
                        directory,
                        name,
                        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                        Mode::empty(),
                    )
                    .map_err(io_error)?,
                );
                let metadata = file.metadata().map_err(io_error)?;
                if metadata.ino() != entry.ino() {
                    return Err(failure(
                        ErrorKind::OwnershipMismatch,
                        "inventory entry changed",
                    ));
                }
                if metadata.is_dir() {
                    values.insert(
                        relative.clone(),
                        format!("directory:{}:{}", metadata.dev(), metadata.ino()),
                    );
                    walk(&file, &relative, values, total, depth + 1)?;
                } else if metadata.is_file() {
                    if metadata.len() > FILE_LIMIT as u64 {
                        return Err(failure(
                            ErrorKind::InvalidInput,
                            "inventory file exceeds budget",
                        ));
                    }
                    let mut bytes = vec![];
                    std::io::Read::by_ref(&mut file)
                        .take((FILE_LIMIT + 1) as u64)
                        .read_to_end(&mut bytes)
                        .map_err(io_error)?;
                    *total += bytes.len();
                    if bytes.len() > FILE_LIMIT || *total > INVENTORY_LIMIT {
                        return Err(failure(
                            ErrorKind::InvalidInput,
                            "inventory bytes exceed budget",
                        ));
                    }
                    values.insert(relative, fingerprint(&file, &bytes)?);
                } else {
                    return Err(failure(
                        ErrorKind::InvalidInput,
                        "inventory special files unsupported",
                    ));
                }
            }
            Ok(())
        }
        let directory = open_dir(root)?;
        let mut values = BTreeMap::new();
        walk(&directory, Path::new(""), &mut values, &mut 0, 0)?;
        Ok(Self(values))
    }
    pub fn reconcile(&self, after: &Self, writes: &BTreeMap<PathBuf, String>) -> AdapterResult<()> {
        for path in self.0.keys().chain(after.0.keys()) {
            if self.0.get(path) == after.0.get(path) {
                continue;
            }
            let matches = writes
                .get(path)
                .is_some_and(|expected| after.0.get(path) == Some(expected));
            if !matches {
                return Err(failure(
                    ErrorKind::StateConflict,
                    format!(
                        "unexplained native/concurrent worktree effect: {}",
                        path.display()
                    ),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    fn fixture() -> (tempfile::TempDir, ScopedFiles) {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let project = Project::new(
            "fixture".into(),
            root.clone(),
            "fixture".into(),
            "main".into(),
        );
        let files = ScopedFiles::new(root, &project).unwrap();
        (directory, files)
    }
    #[test]
    fn scoped_text_writes_preserve_mode_and_fence_changed_reads() {
        let (directory, mut files) = fixture();
        let path = directory.path().join("own.txt");
        std::fs::write(&path, "initial").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(
            files
                .write(&json!({"path":"own.txt","content":"no prior read"}))
                .is_err()
        );
        files.read(&json!({"path":"own.txt"})).unwrap();
        std::fs::write(&path, "concurrent").unwrap();
        assert!(
            files
                .write(&json!({"path":"own.txt","content":"lost update"}))
                .is_err()
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "concurrent");
        files.read(&json!({"path":"own.txt"})).unwrap();
        files
            .write(&json!({"path":"own.txt","content":"updated"}))
            .unwrap();
        assert_eq!(path.metadata().unwrap().permissions().mode() & 0o777, 0o755);
        files
            .write(&json!({"path":"new.txt","content":"new"}))
            .unwrap();
        assert_eq!(
            directory
                .path()
                .join("new.txt")
                .metadata()
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert!(
            files
                .write(&json!({"path":"absent/child.txt","content":"no parent"}))
                .is_err()
        );
    }
    #[test]
    fn scoped_paths_aliases_special_files_and_binary_are_denied() {
        let (directory, mut files) = fixture();
        let outside = tempfile::tempdir().unwrap();
        let target = outside.path().join("foreign");
        std::fs::write(&target, "foreign").unwrap();
        std::os::unix::fs::symlink(&target, directory.path().join("link")).unwrap();
        std::fs::hard_link(&target, directory.path().join("hardlink")).unwrap();
        for path in [
            "../foreign",
            ".GIT",
            ".Git/config",
            "agents.md",
            "CLAUDE.md",
            "GROK.md",
            "e\u{301}.txt",
            "link",
            "hardlink",
        ] {
            assert!(
                files
                    .write(&json!({"path":path,"content":"forbidden"}))
                    .is_err(),
                "{path}"
            );
            assert!(files.read(&json!({"path":path})).is_err(), "{path}");
        }
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "foreign");
        std::fs::write(directory.path().join("binary"), [0, 1, 255]).unwrap();
        assert!(files.read(&json!({"path":"binary"})).is_err());
        assert!(
            std::process::Command::new("mkfifo")
                .arg(directory.path().join("fifo"))
                .status()
                .unwrap()
                .success()
        );
        assert!(files.read(&json!({"path":"fifo"})).is_err());
        assert!(
            files
                .write(&json!({"path":"fifo","content":"forbidden"}))
                .is_err()
        );
        assert!(files.read(&json!({"path":"binary","line":1})).is_err());
    }
    #[test]
    fn inventory_includes_ignored_effects_and_rejects_unjournaled_metadata_changes() {
        let (directory, mut files) = fixture();
        std::fs::write(directory.path().join("own.txt"), "baseline").unwrap();
        std::fs::create_dir(directory.path().join(".ignored")).unwrap();
        let before = Inventory::capture(directory.path()).unwrap();
        files.read(&json!({"path":"own.txt"})).unwrap();
        files
            .write(&json!({"path":"own.txt","content":"changed"}))
            .unwrap();
        let after = Inventory::capture(directory.path()).unwrap();
        before.reconcile(&after, &files.writes).unwrap();
        std::fs::write(directory.path().join(".ignored/hook"), "unexplained").unwrap();
        assert!(
            before
                .reconcile(
                    &Inventory::capture(directory.path()).unwrap(),
                    &files.writes
                )
                .is_err()
        );
        std::fs::remove_file(directory.path().join(".ignored/hook")).unwrap();
        std::fs::set_permissions(
            directory.path().join("own.txt"),
            std::fs::Permissions::from_mode(0o777),
        )
        .unwrap();
        assert!(
            before
                .reconcile(
                    &Inventory::capture(directory.path()).unwrap(),
                    &files.writes
                )
                .is_err()
        );
    }
}

#[cfg(test)]
mod configured_rule_tests {
    use super::*;
    #[test]
    fn source_checkout_authority_refs_also_protect_task_copies() {
        let source = tempfile::tempdir().unwrap();
        let task = tempfile::tempdir().unwrap();
        for root in [source.path(), task.path()] {
            std::fs::create_dir(root.join("docs")).unwrap();
            std::fs::write(root.join("docs/rules.txt"), "authoritative rule").unwrap();
        }
        let source = source.path().canonicalize().unwrap();
        let task_root = task.path().canonicalize().unwrap();
        let mut project = Project::new(
            "fixture".into(),
            source.clone(),
            "fixture".into(),
            "main".into(),
        );
        project.rule_refs.push(source.join("docs/rules.txt"));
        let mut files = ScopedFiles::new(task_root.clone(), &project).unwrap();
        assert!(files.read(&json!({"path":"docs/rules.txt"})).is_err());
        assert!(
            files
                .write(&json!({"path":"docs/rules.txt","content":"replacement"}))
                .is_err()
        );
        assert_eq!(
            std::fs::read_to_string(task_root.join("docs/rules.txt")).unwrap(),
            "authoritative rule"
        );
        std::fs::remove_file(task_root.join("docs/rules.txt")).unwrap();
        if !task_root.join("Docs").exists() {
            std::fs::create_dir(task_root.join("Docs")).unwrap();
        }
        let mut absent = ScopedFiles::new(task_root.clone(), &project).unwrap();
        assert!(
            absent
                .write(&json!({"path":"Docs/RULES.txt","content":"case alias"}))
                .is_err()
        );
        assert!(!task_root.join("Docs/RULES.txt").exists());
        std::fs::remove_file(source.join("docs/rules.txt")).unwrap();
        std::fs::write(source.join("docs/RULES.txt"), "uppercase authority ref").unwrap();
        project.rule_refs = vec![source.join("docs/RULES.txt")];
        let mut uppercase = ScopedFiles::new(task_root.clone(), &project).unwrap();
        assert!(
            uppercase
                .write(&json!({"path":"docs/rules.txt","content":"lowercase alias"}))
                .is_err()
        );
        assert!(!task_root.join("docs/rules.txt").exists());
        let alias = source.join("docs/alias.txt");
        std::os::unix::fs::symlink(source.join("docs/RULES.txt"), &alias).unwrap();
        project.rule_refs = vec![alias];
        // An absent Task copy of a configured source symlink must never become
        // an ordinary writable file at the authority name.
        let error = ScopedFiles::new(task_root.clone(), &project).err().unwrap();
        assert_eq!(error.kind, ErrorKind::InvalidConfiguration);
        assert!(!task_root.join("docs/alias.txt").exists());
        project.rule_refs = vec![source.join("docs")];
        assert!(ScopedFiles::new(task_root.clone(), &project).is_err());
        project.rule_refs = vec![PathBuf::from("docs/rules.txt")];
        assert!(ScopedFiles::new(task_root, &project).is_err());
    }
}
