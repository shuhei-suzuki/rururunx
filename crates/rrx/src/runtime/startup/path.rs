//! MP-2: retained descriptor traversal; raw aliases never become path components.
use crate::execution::RuntimeOwner;
use anyhow::{Context, Result, ensure};
use rustix::fs::{Mode, OFlags, mkdirat, open, openat};
use sha2::{Digest, Sha256};
use std::{fs::File, os::unix::fs::MetadataExt, path::PathBuf, sync::Arc};
const FLAGS: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);
fn identity(f: &File) -> Result<(u64, u64)> {
    let m = f.metadata()?;
    Ok((m.dev(), m.ino()))
}
fn directory(f: &File, private: bool) -> Result<()> {
    let m = f.metadata()?;
    let uid = if private {
        rustix::process::geteuid()
    } else {
        rustix::process::getuid()
    };
    ensure!(
        m.is_dir()
            && m.uid() == uid.as_raw()
            && if private {
                m.mode() & 0o777 == 0o700
            } else {
                m.mode() & 0o022 == 0
            },
        "startup directory ownership or mode refused"
    );
    Ok(())
}
pub(super) fn equal_uids(real: u32, effective: u32) -> bool {
    real == effective
}
fn privilege() -> Result<()> {
    ensure!(
        equal_uids(
            rustix::process::getuid().as_raw(),
            rustix::process::geteuid().as_raw()
        ),
        "startup uid/euid mismatch"
    );
    Ok(())
}
fn child(parent: &File, name: &str, shared: bool) -> Result<File> {
    match mkdirat(parent, name, Mode::from_raw_mode(0o700)) {
        Ok(()) => {}
        Err(rustix::io::Errno::EXIST) if shared => {}
        Err(e) => return Err(e.into()),
    }
    let file = File::from(openat(parent, name, FLAGS, Mode::empty())?);
    directory(&file, true)?;
    Ok(file)
}
fn matches(parent: &File, name: &str, retained: &File) -> Result<()> {
    let current = File::from(openat(parent, name, FLAGS, Mode::empty())?);
    directory(&current, true)?;
    ensure!(
        identity(&current)? == identity(retained)?,
        "startup directory identity changed"
    );
    Ok(())
}
fn digest(domain: &[u8], parts: &[&[u8]]) -> String {
    let mut h = Sha256::new();
    h.update(domain);
    for p in parts {
        h.update((p.len() as u64).to_be_bytes());
        h.update(p);
    }
    format!("{:x}", h.finalize())
}
pub(super) struct Paths {
    owner: Arc<RuntimeOwner>,
    root: File,
    startup: File,
    epoch: File,
    epoch_name: String,
}
pub(super) struct AliasPath {
    paths: Arc<Paths>,
    directory: File,
    name: String,
    pub(super) path: PathBuf,
}
impl Paths {
    pub(super) fn cleanup(&self) -> Result<()> {
        privilege()?;
        matches(&self.root, "startup", &self.startup)?;
        matches(&self.startup, &self.epoch_name, &self.epoch)?;
        for e in rustix::fs::Dir::read_from(&self.epoch)? {
            ensure!(
                matches!(e?.file_name().to_bytes(), b"." | b".."),
                "startup epoch not empty"
            );
        }
        rustix::fs::unlinkat(
            &self.startup,
            &self.epoch_name,
            rustix::fs::AtFlags::REMOVEDIR,
        )?;
        Ok(())
    }
    pub(super) fn new(owner: Arc<RuntimeOwner>) -> Result<Arc<Self>> {
        privilege()?;
        let root = File::from(open(&owner.root, FLAGS, Mode::empty())?);
        directory(&root, false)?;
        let lock = File::from(openat(
            &root,
            "owner.lock",
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )?);
        ensure!(
            lock.metadata()?.is_file() && identity(&lock)? == owner.lock_identity()?,
            "startup owner.lock identity differs"
        );
        let startup = child(&root, "startup", true)?;
        let epoch_name = format!(
            "e-{}",
            digest(
                b"rrx/startup/epoch/v1\0",
                &[owner.instance_id().as_bytes(), &owner.epoch().to_be_bytes()]
            )
        );
        let epoch = child(&startup, &epoch_name, false)?;
        Ok(Arc::new(Self {
            owner,
            root,
            startup,
            epoch,
            epoch_name,
        }))
    }
    pub(super) fn alias(
        self: &Arc<Self>,
        installation: uuid::Uuid,
        alias: &str,
    ) -> Result<AliasPath> {
        let name = digest(
            b"rrx/startup/alias/v1\0",
            &[installation.as_bytes(), alias.as_bytes()],
        );
        let directory = child(&self.epoch, &name, false)?;
        let path = self
            .owner
            .root
            .join("startup")
            .join(&self.epoch_name)
            .join(&name);
        Ok(AliasPath {
            paths: self.clone(),
            directory,
            name,
            path,
        })
    }
}
impl AliasPath {
    pub(super) fn verify(&self) -> Result<()> {
        privilege()?;
        let p = &self.paths;
        directory(&p.root, false)?;
        let current = File::from(open(&p.owner.root, FLAGS, Mode::empty())?);
        ensure!(
            identity(&current)? == identity(&p.root)?,
            "startup root identity changed"
        );
        let lock = File::from(openat(
            &p.root,
            "owner.lock",
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )?);
        ensure!(
            lock.metadata()?.is_file() && identity(&lock)? == p.owner.lock_identity()?,
            "startup lock replaced"
        );
        matches(&p.root, "startup", &p.startup)?;
        matches(&p.startup, &p.epoch_name, &p.epoch)?;
        matches(&p.epoch, &self.name, &self.directory)?;
        Ok(())
    }
    pub(super) fn verify_empty(&self) -> Result<()> {
        self.verify()?;
        for entry in rustix::fs::Dir::read_from(&self.directory)? {
            let entry = entry.context("startup directory listing failed")?;
            ensure!(
                matches!(entry.file_name().to_bytes(), b"." | b".."),
                "startup cwd is not empty"
            );
        }
        Ok(())
    }
    pub(super) fn schema(&self) -> Result<File> {
        self.verify_empty()?;
        child(&self.directory, "schema-out", false)
    }
    pub(super) fn verify_schema(&self, retained: &File) -> Result<()> {
        self.verify()?;
        matches(&self.directory, "schema-out", retained)?;
        for e in rustix::fs::Dir::read_from(&self.directory)? {
            ensure!(
                matches!(e?.file_name().to_bytes(), b"." | b".." | b"schema-out"),
                "startup cwd changed"
            );
        }
        Ok(())
    }
    pub(super) fn clean_schema(&self, retained: &File) -> Result<()> {
        self.verify_schema(retained)?;
        clean(retained, 0, &mut 0)?;
        self.verify_schema(retained)?;
        rustix::fs::unlinkat(
            &self.directory,
            "schema-out",
            rustix::fs::AtFlags::REMOVEDIR,
        )?;
        self.verify_empty()
    }
    pub(super) fn cleanup(&self) -> Result<()> {
        self.verify()?;
        clean(&self.directory, 0, &mut 0)?;
        self.verify()?;
        rustix::fs::unlinkat(
            &self.paths.epoch,
            &self.name,
            rustix::fs::AtFlags::REMOVEDIR,
        )?;
        Ok(())
    }
}

/// Descriptor-relative, no-follow traversal of this run's retained directory.
/// Unknown or unsafe entries leave the directory available for investigation.
pub(crate) fn clean(dir: &File, depth: usize, count: &mut usize) -> Result<()> {
    ensure!(depth <= 4, "startup cleanup depth bound");
    for entry in rustix::fs::Dir::read_from(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        if matches!(name.to_bytes(), b"." | b"..") {
            continue;
        }
        *count += 1;
        ensure!(*count <= 1024, "startup cleanup entry bound");
        let stat = rustix::fs::statat(dir, name, rustix::fs::AtFlags::SYMLINK_NOFOLLOW)?;
        if rustix::fs::FileType::from_raw_mode(stat.st_mode) == rustix::fs::FileType::Symlink {
            // unlinkat removes this link itself, never its target.
            let current = rustix::fs::statat(dir, name, rustix::fs::AtFlags::SYMLINK_NOFOLLOW)?;
            ensure!(
                (current.st_dev, current.st_ino) == (stat.st_dev, stat.st_ino),
                "startup cleanup replaced symlink"
            );
            rustix::fs::unlinkat(dir, name, rustix::fs::AtFlags::empty())?;
            continue;
        }
        let file = File::from(openat(
            dir,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )?);
        let m = file.metadata()?;
        ensure!(
            m.uid() == rustix::process::getuid().as_raw()
                && m.dev() == dir.metadata()?.dev()
                && (m.is_file() && m.nlink() == 1 || m.is_dir()),
            "startup cleanup unsafe entry"
        );
        if m.is_dir() {
            clean(&file, depth + 1, count)?;
        }
        let current = File::from(openat(
            dir,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )?);
        ensure!(
            identity(&file)? == identity(&current)?,
            "startup cleanup replaced entry"
        );
        rustix::fs::unlinkat(
            dir,
            name,
            if m.is_dir() {
                rustix::fs::AtFlags::REMOVEDIR
            } else {
                rustix::fs::AtFlags::empty()
            },
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    fn owner(dir: &tempfile::TempDir) -> Arc<RuntimeOwner> {
        RuntimeOwner::open(&dir.path().join("state.db")).unwrap()
    }
    /// MP-2 successful cleanup is bounded and cannot follow a replaced alias outside its retained epoch.
    #[test]
    fn u4_success_cleanup_is_private_and_bounded() {
        for scenario in ["success", "replaced", "symlink", "depth", "entries"] {
            let temp = tempfile::tempdir().unwrap();
            let owner = owner(&temp);
            let paths = Paths::new(owner).unwrap();
            let alias = paths.alias(uuid::Uuid::new_v4(), "a").unwrap();
            let path = alias.path.clone();
            let outside = temp.path().join("outside");
            std::fs::create_dir(&outside).unwrap();
            std::fs::write(outside.join("keep"), "preserve").unwrap();
            match scenario {
                "success" => {
                    let dir = alias.schema().unwrap();
                    std::fs::create_dir(path.join("schema-out/sub")).unwrap();
                    std::fs::write(path.join("schema-out/sub/data"), "bounded").unwrap();
                    alias.clean_schema(&dir).unwrap();
                    alias.verify_empty().unwrap();
                }
                "replaced" => {
                    std::fs::rename(&path, path.with_extension("old")).unwrap();
                    std::fs::create_dir(&path).unwrap();
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
                        .unwrap();
                }
                "symlink" => std::os::unix::fs::symlink(&outside, path.join("escape")).unwrap(),
                "depth" => {
                    let mut p = path.clone();
                    for _ in 0..5 {
                        p = p.join("d");
                        std::fs::create_dir(&p).unwrap();
                    }
                }
                _ => {
                    for i in 0..1025 {
                        std::fs::write(path.join(i.to_string()), b"x").unwrap();
                    }
                }
            }
            assert_eq!(
                alias.cleanup().is_ok(),
                matches!(scenario, "success" | "symlink"),
                "{scenario}"
            );
            assert_eq!(
                std::fs::read_to_string(outside.join("keep")).unwrap(),
                "preserve"
            );
            if matches!(scenario, "success" | "symlink") {
                paths.cleanup().unwrap();
                assert!(!path.exists());
                assert!(!path.parent().unwrap().exists());
            } else {
                assert!(path.exists());
            }
        }
    }
    /// MP-2 accepts safe roots, hashes raw alias bytes and refuses reuse or inode replacement.
    #[test]
    fn private_descriptors_bind_root_lock_epoch_and_hashed_alias() {
        for mode in [0o755, 0o700] {
            let temp = tempfile::tempdir().unwrap();
            let owner = owner(&temp);
            std::fs::set_permissions(&owner.root, std::fs::Permissions::from_mode(mode)).unwrap();
            let paths = Paths::new(owner.clone()).unwrap();
            let installation = uuid::Uuid::new_v4();
            for alias in ["normal", "../outside", "/absolute", "a/b"] {
                let alias = paths.alias(installation, alias).unwrap();
                alias.verify_empty().unwrap();
                assert!(alias.path.starts_with(owner.root.join("startup")));
                assert_eq!(alias.name.len(), 64);
                assert_eq!(alias.directory.metadata().unwrap().mode() & 0o777, 0o700);
            }
            assert_eq!(owner.root.metadata().unwrap().mode() & 0o777, mode);
            assert!(Paths::new(owner.clone()).is_err(), "MP-2: epoch reused");
            assert!(
                paths.alias(installation, "normal").is_err(),
                "MP-2: alias reused"
            );
            let alias = paths.alias(installation, "replace").unwrap();
            std::fs::rename(&alias.path, alias.path.with_extension("old")).unwrap();
            std::fs::create_dir(&alias.path).unwrap();
            std::fs::set_permissions(&alias.path, std::fs::Permissions::from_mode(0o700)).unwrap();
            assert!(
                alias.verify_empty().is_err(),
                "MP-2: replacement directory accepted"
            );
            std::fs::rename(owner.root.join("owner.lock"), owner.root.join("old.lock")).unwrap();
            std::fs::write(owner.root.join("owner.lock"), "unowned").unwrap();
            let alias = paths.alias(installation, "lock-replace").unwrap();
            assert!(
                alias.verify_empty().is_err(),
                "MP-2: replacement lock accepted"
            );
        }
        assert!(equal_uids(501, 501));
        assert!(!equal_uids(501, 502), "MP-2: uid/euid mismatch accepted");
    }
    /// MP-2 unsafe roots and startup aliases remain unchanged and never escape the boundary.
    #[test]
    fn unsafe_root_and_startup_components_are_refused_unchanged() {
        for mode in [0o775, 0o757, 0o777] {
            let temp = tempfile::tempdir().unwrap();
            let owner = owner(&temp);
            std::fs::set_permissions(&owner.root, std::fs::Permissions::from_mode(mode)).unwrap();
            assert!(Paths::new(owner.clone()).is_err());
            assert!(!owner.root.join("startup").exists());
            assert_eq!(owner.root.metadata().unwrap().mode() & 0o777, mode);
        }
        for kind in ["file", "mode", "outside", "inside"] {
            let temp = tempfile::tempdir().unwrap();
            let owner = owner(&temp);
            let startup = owner.root.join("startup");
            let target = temp.path().join("target");
            std::fs::create_dir(&target).unwrap();
            std::fs::write(target.join("keep"), "preserve").unwrap();
            match kind {
                "file" => std::fs::write(&startup, "keep").unwrap(),
                "mode" => {
                    std::fs::create_dir(&startup).unwrap();
                    std::fs::set_permissions(&startup, std::fs::Permissions::from_mode(0o755))
                        .unwrap();
                }
                "outside" => std::os::unix::fs::symlink(&target, &startup).unwrap(),
                _ => {
                    let inside = owner.root.join("units");
                    std::fs::create_dir(&inside).unwrap();
                    std::os::unix::fs::symlink(inside, &startup).unwrap();
                }
            }
            assert!(Paths::new(owner).is_err(), "MP-2: {kind} accepted");
            assert_eq!(
                std::fs::read_to_string(target.join("keep")).unwrap(),
                "preserve"
            );
        }
        // Nonprivileged descriptor predicate coverage, separate from producer proof.
        let root = File::open("/").unwrap();
        let m = root.metadata().unwrap();
        assert_eq!(
            directory(&root, false).is_ok(),
            m.uid() == rustix::process::getuid().as_raw() && m.mode() & 0o022 == 0
        );
    }
    /// MP-2 shares only the private startup parent across epochs and retains replaced/foreign entries.
    #[test]
    fn fresh_epochs_share_the_parent_without_adopting_or_deleting_old_entries() {
        let temp = tempfile::tempdir().unwrap();
        let owner = owner(&temp);
        let state = owner.state_path().to_path_buf();
        let root = owner.root.clone();
        let installation = uuid::Uuid::new_v4();
        let paths = Paths::new(owner.clone()).unwrap();
        let alias = paths.alias(installation, "../outside").unwrap();
        let original = alias.path.clone();
        let old = original.with_extension("old");
        std::fs::rename(&original, &old).unwrap();
        let outside = temp.path().join("outside");
        std::fs::create_dir(&outside).unwrap();
        std::fs::write(outside.join("keep"), "keep").unwrap();
        std::os::unix::fs::symlink(&outside, &original).unwrap();
        assert!(
            alias.verify_empty().is_err(),
            "MP-2: replaced symlink adopted"
        );
        assert!(
            paths.alias(installation, "../outside").is_err(),
            "MP-2: existing alias symlink reused"
        );
        assert!(
            Paths::new(owner.clone()).is_err(),
            "MP-2: existing epoch reused"
        );
        drop(alias);
        drop(paths);
        drop(owner);
        let owner = RuntimeOwner::open(&state).unwrap();
        let paths = Paths::new(owner).unwrap();
        let alias = paths.alias(installation, "../outside").unwrap();
        alias.verify_empty().unwrap();
        assert_ne!(alias.path, original, "MP-2: old epoch adopted");
        assert!(original.symlink_metadata().unwrap().is_symlink());
        assert!(old.is_dir());
        assert_eq!(
            std::fs::read_to_string(outside.join("keep")).unwrap(),
            "keep"
        );
        assert_eq!(
            root.join("startup").metadata().unwrap().mode() & 0o777,
            0o700
        );
    }
}
