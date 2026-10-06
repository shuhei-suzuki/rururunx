//! The sole retained nongrant command, then SAME-custody prepared conjunction.
use super::*;

pub(super) struct NativeTransportCommand {
    pub(super) program: std::path::PathBuf,
    pub(super) cwd: std::path::PathBuf,
    pub(super) argv: Vec<String>,
    pub(super) environment: BTreeMap<String, String>,
    pub(super) native_session: Option<uuid::Uuid>,
}
pub(super) fn command_argv(provider: &str, role: SessionRole, model: Option<&str>, effort: Option<&str>, session: Option<uuid::Uuid>) -> Result<Vec<String>> {
    ensure!(matches!(role, SessionRole::Executor | SessionRole::Reviewer), "native command role unsupported");
    for value in [model, effort].into_iter().flatten() {
        ensure!(!value.is_empty() && value.len() <= 128 && !value.starts_with('-') && !value.chars().any(char::is_control), "native option value unsupported");
    }
    let argv = match provider {
        "codex" => { ensure!(session.is_none(), "unexpected Codex native UUID"); vec!["app-server".into(), "--listen".into(), "stdio://".into()] },
        "claude" => {
            let mut argv: Vec<String> = ["-p", "--input-format", "stream-json", "--output-format", "stream-json", "--verbose", "--session-id"].map(String::from).into();
            argv.push(session.context("Claude native UUID absent")?.to_string());
            argv.extend(["--permission-prompt-tool", "stdio", "--settings", "{\"forceLoginMethod\":\"claudeai\"}"].map(String::from));
            for (flag, value) in [("--model", model), ("--effort", effort)] { if let Some(value) = value { argv.extend([flag.into(), value.into()]); } }
            if role != SessionRole::Executor { argv.extend(["--permission-mode".into(), "plan".into()]); }
            argv
        },
        _ => anyhow::bail!("native command provider unsupported"),
    };
    ensure!(argv.len() <= 18 && argv.iter().all(|a| a.len() <= 128), "native command argv exceeds bounded profile");
    Ok(argv)
}
pub(super) fn plan_native_command(owner: &Arc<RuntimeOwner>, actor: &Arc<NativePreparationActor>, completion: &Arc<version::NativeReadonlyHelperCompletion>, compat: &Arc<compat::NativeCompatQualification>) -> Result<Arc<NativeTransportCommand>> {
    actor.validate_open()?;
    ensure!(completion.matches_actor(actor), "command requires SAME closed helper completion");
    let profile = version::qualified_physical_profile(owner, actor)?;
    let f = actor.launch().allocation().facts();
    let native_session = (f.provider == "claude").then(uuid::Uuid::new_v4);
    let command = Arc::new(NativeTransportCommand {
        program: f.program.into(), cwd: f.path.into(),
        argv: command_argv(f.provider, f.role, f.model, f.effort, native_session)?,
        environment: profile.environment(&actor.launch().allocation().unit_snapshot().cookie, &owner.socket)?, native_session,
    });
    ensure!(serde_json::to_vec(&json!({"program":command.program,"cwd":command.cwd,"argv":command.argv,"environment":command.environment})).map(|b|b.len())? <= 64*1024, "encoded native command exceeds bound");
    compat.check_command(&command)?;
    Ok(command)
}
