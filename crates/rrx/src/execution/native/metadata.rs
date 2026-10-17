//! Fresh P metadata custody. The directory and accepted additions never carry authority.
use super::*;
use rustix::fs::{Mode, OFlags, open, openat};
use std::{fs::File, os::unix::fs::MetadataExt, path::PathBuf};

const DIRECTORY: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);

pub(crate) struct SchemaDirectory {
    actor: Arc<NativePreparationActor>,
    parents: UnitTemp,
    directory: File,
    name: String,
    pub(super) path: PathBuf,
}
fn identity(file: &File) -> Result<(u64, u64)> {
    let m = file.metadata()?;
    ensure!(
        m.is_dir() && m.uid() == rustix::process::getuid().as_raw() && m.mode() & 0o022 == 0,
        "P directory ownership or mode refused"
    );
    Ok((m.dev(), m.ino()))
}
// Retain every component that could rename the CLI's output name. Only the
// owner root is opened by path; all descendants are opened relative to it.
struct UnitTemp {
    owner: Arc<RuntimeOwner>,
    root: File,
    units: File,
    unit: File,
    temp: File,
    unit_name: String,
}
impl UnitTemp {
    fn new(owner: &Arc<RuntimeOwner>, unit: UnitId) -> Result<Self> {
        ensure!(
            rustix::process::getuid() == rustix::process::geteuid(),
            "P uid/euid differs"
        );
        let root = File::from(open(&owner.root, DIRECTORY, Mode::empty())?);
        identity(&root)?;
        let units = File::from(openat(&root, "units", DIRECTORY, Mode::empty())?);
        identity(&units)?;
        let unit_name = unit.to_string();
        let unit = File::from(openat(
            &units,
            unit_name.as_str(),
            DIRECTORY,
            Mode::empty(),
        )?);
        identity(&unit)?;
        let temp = File::from(openat(&unit, "tmp", DIRECTORY, Mode::empty())?);
        identity(&temp)?;
        let value = Self {
            owner: owner.clone(),
            root,
            units,
            unit,
            temp,
            unit_name,
        };
        value.verify()?;
        Ok(value)
    }
    fn verify(&self) -> Result<()> {
        let root = File::from(open(&self.owner.root, DIRECTORY, Mode::empty())?);
        ensure!(identity(&root)? == identity(&self.root)?, "P root replaced");
        let lock = File::from(openat(
            &self.root,
            "owner.lock",
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )?);
        let m = lock.metadata()?;
        ensure!(
            m.is_file() && (m.dev(), m.ino()) == self.owner.lock_identity()?,
            "P owner lock replaced"
        );
        for (parent, name, retained) in [
            (&self.root, "units", &self.units),
            (&self.units, self.unit_name.as_str(), &self.unit),
            (&self.unit, "tmp", &self.temp),
        ] {
            let current = File::from(openat(parent, name, DIRECTORY, Mode::empty())?);
            ensure!(
                identity(&current)? == identity(retained)?,
                "P ancestor replaced"
            );
        }
        Ok(())
    }
}
pub(super) fn validate_unit_temp(owner: &Arc<RuntimeOwner>, unit: UnitId) -> Result<()> {
    UnitTemp::new(owner, unit).map(|_| ())
}
impl SchemaDirectory {
    pub(super) fn new(
        owner: &Arc<RuntimeOwner>,
        actor: &Arc<NativePreparationActor>,
    ) -> Result<Arc<Self>> {
        let profile = version::qualified_physical_profile(owner, actor)?;
        let parents = UnitTemp::new(owner, profile.unit)?;
        let parent = &parents.temp;
        let name = format!("p-schema-{}", uuid::Uuid::new_v4());
        rustix::fs::mkdirat(parent, name.as_str(), Mode::from_raw_mode(0o700))?;
        // Record the leftover before any later open, planning or helper failure.
        // This finite factual audit grants no cleanup or preparation authority.
        owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .audit(
                actor.launch().allocation().facts().scope,
                "native.phase.schema_directory",
                serde_json::json!({"cleanup":"not_cleaned"}),
            )?;
        let directory = File::from(openat(parent, name.as_str(), DIRECTORY, Mode::empty())?);
        let value = Arc::new(Self {
            actor: actor.clone(),
            path: profile.temp.join(&name),
            parents,
            directory,
            name,
        });
        value.verify(actor)?;
        Ok(value)
    }
    pub(crate) fn verify(&self, actor: &Arc<NativePreparationActor>) -> Result<()> {
        actor.validate_original()?;
        ensure!(Arc::ptr_eq(&self.actor, actor), "P Schema foreign actor");
        self.parents.verify()?;
        let parent = &self.parents.temp;
        let current = File::from(openat(
            parent,
            self.name.as_str(),
            DIRECTORY,
            Mode::empty(),
        )?);
        ensure!(
            identity(&current)? == identity(&self.directory)?
                && current.metadata()?.mode() & 0o777 == 0o700,
            "P Schema directory identity changed"
        );
        Ok(())
    }
    pub(super) fn gate(&self) -> Result<schema::Gate> {
        self.verify(&self.actor)?;
        let files = schema::read_directory(&self.directory)?;
        self.verify(&self.actor)?;
        Ok(schema::compare(&files))
    }
    pub(super) fn cleanup(&self) -> Result<()> {
        self.verify(&self.actor)?;
        crate::runtime::startup::path::clean(&self.directory, 0, &mut 0)?;
        self.verify(&self.actor)?;
        rustix::fs::unlinkat(
            &self.parents.temp,
            self.name.as_str(),
            rustix::fs::AtFlags::REMOVEDIR,
        )?;
        Ok(())
    }
}

impl NativeSessions {
    pub(super) async fn prepare_phase_metadata(
        &self,
        original: Arc<NativePreparationCustody>,
        version: Arc<version::NativeVersionHelperCustody>,
    ) -> Result<Arc<version::NativeVersionHelperCustody>> {
        let actor = original.state_actor()?;
        version.closed()?;
        ensure!(
            version.matches_actor(&actor),
            "P metadata foreign version actor"
        );
        let directory = SchemaDirectory::new(&self.owner, &actor)?;
        let plan = crate::state::Store::plan_phase_metadata_intent(
            &self.owner,
            version.closed()?,
            readonly::NativePhaseHelperAction::Schema,
            Some(directory.clone()),
        )?;
        let mut command =
            version::physical_command(&self.owner, &actor, plan.action(), None, None)?;
        command.arg("--out").arg(&directory.path);
        let schema = self
            .run_phase_helper(original.clone(), actor.clone(), plan, command, None)
            .await?;
        schema.closed()?;
        directory.cleanup()?;
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .audit(
                actor.launch().allocation().facts().scope,
                "native.phase.schema_directory",
                serde_json::json!({"cleanup":"cleaned"}),
            )?;
        let plan = crate::state::Store::plan_phase_metadata_intent(
            &self.owner,
            schema.closed()?,
            readonly::NativePhaseHelperAction::Handshake,
            None,
        )?;
        let command = version::physical_command(&self.owner, &actor, plan.action(), None, None)?;
        self.run_phase_helper(original, actor, plan, command, None)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{GoalId, ProjectId, Scope, TaskId};
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    /// P retains only owned, non-writable ancestors and refuses replacement without touching outside data.
    #[test]
    fn u5r1b_p_ancestors_bind_private_materialization_and_refuse_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        std::fs::set_permissions(&owner.root, std::fs::Permissions::from_mode(0o755)).unwrap();
        let manager = resources::ResourceManager::new(owner.clone());
        let profile = manager
            .draft(
                UnitId::new(),
                &Scope::task(ProjectId::new(), GoalId::new(), TaskId::new()),
                dir.path(),
            )
            .unwrap();
        manager.materialize(&profile).unwrap();
        for path in [
            profile.root.parent().unwrap(),
            &profile.root,
            &profile.temp,
            &profile.output,
            &profile.cache,
            &profile.tool_bin,
        ] {
            assert_eq!(path.metadata().unwrap().mode() & 0o777, 0o755);
        }
        let retained = UnitTemp::new(&owner, profile.unit).unwrap();
        for path in [profile.root.parent().unwrap(), &profile.root, &profile.temp] {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        retained.verify().unwrap();
        let outside = dir.path().join("outside");
        std::fs::create_dir(&outside).unwrap();
        std::fs::write(outside.join("sentinel"), "unchanged").unwrap();
        for path in [
            &owner.root,
            profile.root.parent().unwrap(),
            &profile.root,
            &profile.temp,
        ] {
            for mode in [0o770, 0o707, 0o777] {
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
                assert!(UnitTemp::new(&owner, profile.unit).is_err());
                assert!(retained.verify().is_err());
            }
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
            retained.verify().unwrap();
            let backup = path.with_extension("retained");
            std::fs::rename(path, &backup).unwrap();
            std::os::unix::fs::symlink(&outside, path).unwrap();
            assert!(retained.verify().is_err());
            std::fs::remove_file(path).unwrap();
            std::fs::DirBuilder::new().mode(0o700).create(path).unwrap();
            assert!(retained.verify().is_err());
            std::fs::remove_dir(path).unwrap();
            std::fs::rename(&backup, path).unwrap();
            retained.verify().unwrap();
        }
        // The predicate consumes a real foreign-owned descriptor, without chown or authority seeds.
        let foreign = File::from(open("/", DIRECTORY, Mode::empty()).unwrap());
        assert_ne!(
            foreign.metadata().unwrap().uid(),
            rustix::process::getuid().as_raw()
        );
        assert!(identity(&foreign).is_err());
        assert_eq!(
            std::fs::read_to_string(outside.join("sentinel")).unwrap(),
            "unchanged"
        );
    }
}
