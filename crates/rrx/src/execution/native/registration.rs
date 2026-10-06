//! Registration consumes the original Prepared object; it cannot refresh a
//! readiness or helper lineage from persisted facts.
use super::*;

impl PreparedNativePhase {
    pub(crate) fn launch(&self) -> &Arc<crate::state::managed_binding::PhaseLaunchParts> {
        self.actor.launch()
    }
    pub(crate) fn lineage(&self) -> &crate::state::NativeReadyLineage {
        self.quota.lineage()
    }
    pub(crate) fn history(&self) -> &crate::state::NativeHelperHistoryCommit {
        &self.completion.commit
    }
    pub(crate) fn quota(&self) -> &crate::state::NativeQuotaAdmitted { &self.quota }
    pub(crate) fn validate_original(&self) -> Result<()> {
        self.actor.validate_original()?;
        let closed = self.version.closed()?;
        ensure!(self.completion.matches_actor(&self.actor)
            && self.completion.commit.matches_prefix(&self.known, &closed)
            && self.compat.matches(&self.actor, &self.version),
            "Prepared registration original closed prefix differs");
        self.compat.check_command(&self.command)?;
        self.lineage().validate_prepared_shape()
    }
    pub(crate) fn validate_open(&self) -> Result<()> {
        self.validate_original()?;
        self.actor.validate_open()
    }
    pub(crate) fn governing_digest(&self) -> Result<String> {
        let original = self.launch().marker().original_plan();
        crate::state::execution_governing_digest(original.project().0, original.goal().0)
    }
    pub(crate) fn registration_material(&self) -> Result<(Session, NativeSeed, String)> {
        self.validate_open()?;
        let f = self.launch().allocation().facts();
        ensure!(self.command.program.is_absolute() && self.command.cwd.is_absolute(),
            "Native command program/cwd is not absolute");
        // Only environment names are digest material. Cookie, socket and inherited
        // environment values never enter a journal, log or digest.
        let command_digest = native_result::digest(&serde_json::to_vec(&json!({
            "program":self.command.program,"cwd":self.command.cwd,"argv":self.command.argv,
            "environment_keys":self.command.environment.keys().collect::<Vec<_>>(),
            "profile_digest":f.profile_digest,"compatibility":self.compat.declaration_digest()
        }))?);
        let session = Session { id:f.session_id, scope:f.scope.clone(), agent:f.alias.into(),
            provider:f.provider.into(), role:f.role, native_ref:None, pid:None,
            worktree:f.path.into(), state:SessionState::Starting, model:f.model.map(str::to_owned),
            effort:f.effort.map(str::to_owned), recovery:json!({"unit":f.unit_id,
                "generation":f.generation,"epoch":f.epoch}), started_at:now_ms() };
        let seed = NativeSeed { input:f.input.clone(), id:f.invocation_id,
            profile:format!("text_v1/{}",f.profile_digest),
            native_version:self.version.observation()?.version_text()?.into() };
        Ok((session, seed, command_digest))
    }
}
