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
    parent: File,
    directory: File,
    name: String,
    pub(super) path: PathBuf,
}
fn identity(file: &File) -> Result<(u64, u64)> {
    let m = file.metadata()?;
    ensure!(
        m.is_dir() && m.uid() == rustix::process::getuid().as_raw(),
        "P directory owner differs"
    );
    Ok((m.dev(), m.ino()))
}
impl SchemaDirectory {
    pub(super) fn new(
        owner: &Arc<RuntimeOwner>,
        actor: &Arc<NativePreparationActor>,
    ) -> Result<Arc<Self>> {
        let profile = version::qualified_physical_profile(owner, actor)?;
        let parent = File::from(open(&profile.temp, DIRECTORY, Mode::empty())?);
        identity(&parent)?;
        let name = format!("p-schema-{}", uuid::Uuid::new_v4());
        rustix::fs::mkdirat(&parent, name.as_str(), Mode::from_raw_mode(0o700))?;
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
        let directory = File::from(openat(&parent, name.as_str(), DIRECTORY, Mode::empty())?);
        let value = Arc::new(Self {
            actor: actor.clone(),
            path: profile.temp.join(&name),
            parent,
            directory,
            name,
        });
        value.verify(actor)?;
        Ok(value)
    }
    pub(crate) fn verify(&self, actor: &Arc<NativePreparationActor>) -> Result<()> {
        actor.validate_original()?;
        ensure!(Arc::ptr_eq(&self.actor, actor), "P Schema foreign actor");
        let parent_path = self.path.parent().context("P Schema parent absent")?;
        ensure!(
            parent_path.canonicalize()? == parent_path,
            "P Schema parent aliases another namespace"
        );
        let parent = File::from(open(parent_path, DIRECTORY, Mode::empty())?);
        let current = File::from(openat(
            &parent,
            self.name.as_str(),
            DIRECTORY,
            Mode::empty(),
        )?);
        ensure!(
            identity(&parent)? == identity(&self.parent)?
                && identity(&current)? == identity(&self.directory)?
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
            &self.parent,
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
