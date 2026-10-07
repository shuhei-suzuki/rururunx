//! Captured cooperative compatibility, never a permission or sandbox proof.
use super::*;
use crate::config::{NativeCompatConfig, NativeHookWrites, NativeRequiredHook};

pub(crate) struct NativeCompatDeclaration {
    config: NativeCompatConfig,
    digest: String,
}
impl NativeCompatDeclaration {
    pub(crate) fn digest(&self) -> &str {
        &self.digest
    }
    // Only registry installation creates the immutable declaration Arc.
    pub(crate) fn installed(config: &NativeCompatConfig) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            config: config.clone(),
            digest: native_result::digest(&config.canonical()?),
        }))
    }
}
pub(super) struct NativeCompatStatic {
    declaration: Arc<NativeCompatDeclaration>,
    source: Arc<crate::execution::workflow_source::SourceNativePreparationSeal>,
    actor: Arc<NativePreparationActor>,
    role: SessionRole,
}
pub(super) struct NativeCompatQualification {
    captured: NativeCompatStatic,
    version: Arc<version::NativeVersionHelperCustody>,
}

fn unsupported(message: &'static str) -> anyhow::Error {
    crate::adapter::AdapterError {
        kind: ErrorKind::UnsupportedCapability,
        message: message.into(),
    }
    .into()
}
fn qualify_role_hooks(
    role: SessionRole,
    config: &NativeCompatConfig,
    required: &[NativeRequiredHook],
) -> Result<()> {
    ensure!(
        matches!(role, SessionRole::Executor | SessionRole::Reviewer),
        "unsupported native role"
    );
    if role == SessionRole::Reviewer
        && config
            .user_hooks
            .iter()
            .map(|h| h.writes)
            .chain(required.iter().map(|h| h.writes))
            .any(|w| w != NativeHookWrites::None)
    {
        return Err(unsupported(
            "reviewer source is readonly; source-writing hook declared",
        ));
    }
    Ok(())
}
pub(super) fn qualify_compat_static(
    actor: &Arc<NativePreparationActor>,
) -> Result<NativeCompatStatic> {
    actor.validate_open()?;
    let f = actor.launch().allocation().facts();
    let adapter = actor
        .launch()
        .allocation()
        .selected_port()
        .selected_adapter()?;
    let declaration = adapter
        .compatibility
        .clone()
        .ok_or_else(|| unsupported("native compatibility undeclared"))?;
    let source = actor.launch().preparation_seal()?;
    let config = &declaration.config;
    ensure!(
        config.profile == "rrx-native-inherited-v1" && config.settings == "inherited",
        unsupported("native compatibility profile unsupported")
    );
    let pin = match f.provider {
        "claude" => "2.1.283",
        "codex" => "codex-cli 0.160.0",
        _ => return Err(unsupported("native compatibility provider unsupported")),
    };
    ensure!(
        config.cli_version == pin,
        unsupported("native compatibility version unsupported")
    );
    let frame_config = source
        .config()
        .agents
        .get(f.alias)
        .and_then(|a| a.compatibility.as_ref())
        .ok_or_else(|| unsupported("original Frame compatibility absent"))?;
    ensure!(
        native_result::digest(&frame_config.canonical()?) == declaration.digest,
        "installed compatibility differs from SAME original Frame"
    );
    ensure!(
        matches!(
            (f.role, actor.launch().allocation().unit_snapshot().kind),
            (SessionRole::Executor, UnitKind::Executor)
                | (SessionRole::Reviewer, UnitKind::Reviewer)
        ),
        "native compatibility role identity differs"
    );
    let required = &source.config().native.required_hooks;
    qualify_role_hooks(f.role, config, required)?;
    for hook in required {
        ensure!(
            source
                .inventory()
                .get(&hook.path)
                .is_some_and(|e| e.skipped.is_none() && e.bytes.is_some() && e.sha256.is_some()),
            unsupported("required native hook is not an ordinary committed blob")
        );
    }
    Ok(NativeCompatStatic {
        declaration,
        source,
        actor: actor.clone(),
        role: f.role,
    })
}
impl NativeCompatStatic {
    pub(super) fn observe(
        self,
        version: Arc<version::NativeVersionHelperCustody>,
    ) -> Result<Arc<NativeCompatQualification>> {
        self.actor.validate_open()?;
        ensure!(
            version.matches_actor(&self.actor),
            "compatibility observed another original actor"
        );
        version.closed()?;
        let profile = match self.declaration.config.cli_version.as_str() {
            "2.1.283" => "claude-2.1.283",
            "codex-cli 0.160.0" => "codex-cli-0.160.0",
            _ => return Err(unsupported("compatibility version unsupported")),
        };
        ensure!(
            version.observation()?.qualified_profile() == Some(profile),
            unsupported("native compatibility observed version differs")
        );
        Ok(Arc::new(NativeCompatQualification {
            captured: self,
            version,
        }))
    }
}
impl NativeCompatQualification {
    pub(super) fn declaration_digest(&self) -> &str {
        &self.captured.declaration.digest
    }
    pub(super) fn matches(
        &self,
        actor: &Arc<NativePreparationActor>,
        version: &Arc<version::NativeVersionHelperCustody>,
    ) -> bool {
        Arc::ptr_eq(&self.captured.actor, actor)
            && Arc::ptr_eq(&self.version, version)
            && self.captured.role == actor.launch().allocation().facts().role
    }
    pub(super) fn check_command(&self, command: &prepared::NativeTransportCommand) -> Result<()> {
        let f = self.captured.actor.launch().allocation().facts();
        ensure!(
            command.program == f.program
                && command.cwd == f.path
                && command.argv
                    == prepared::command_argv(
                        f.provider,
                        self.captured.role,
                        f.model,
                        f.effort,
                        command.native_session
                    )?,
            "native command differs from captured compatibility"
        );
        check_overlay(&command.environment)?;
        // SAME source object remains retained; no settings file or hook ref read.
        ensure!(
            self.captured.source.revision() == f.input.revision,
            "compatibility original Frame changed"
        );
        Ok(())
    }
}

fn check_overlay(env: &BTreeMap<String, String>) -> Result<()> {
    const KEYS: &[&str] = &[
        "RRX_UNIT_ID",
        "RRX_PROCESS_COOKIE",
        "RRX_PROFILE",
        "RRX_RUNTIME_SOCKET",
        "RRX_TMPDIR",
        "TMPDIR",
        "RRX_OUTPUT_DIR",
        "RRX_PORT_START",
        "RRX_PORT_END",
        "RRX_DOCKER_PROJECT",
        "COMPOSE_PROJECT_NAME",
        "CARGO_TARGET_DIR",
        "GRADLE_USER_HOME",
        "SCCACHE_DIR",
        "PATH",
        "GIT_CONFIG_COUNT",
    ];
    ensure!(env.len() <= 64, "native environment overlay exceeds bound");
    let count: usize = env
        .get("GIT_CONFIG_COUNT")
        .context("native Git overlay count absent")?
        .parse()?;
    ensure!(
        (3..=131).contains(&count),
        "native Git overlay count differs"
    );
    let start = count - 3;
    let mut expected: std::collections::BTreeSet<String> =
        KEYS.iter().map(|s| (*s).into()).collect();
    for (i, (key, value)) in [
        ("gc.auto", "0"),
        ("maintenance.auto", "false"),
        ("core.fsmonitor", "false"),
    ]
    .into_iter()
    .enumerate()
    {
        let k = format!("GIT_CONFIG_KEY_{}", start + i);
        let v = format!("GIT_CONFIG_VALUE_{}", start + i);
        ensure!(
            env.get(&k).map(String::as_str) == Some(key)
                && env.get(&v).map(String::as_str) == Some(value),
            "native Git overlay alters required configuration"
        );
        expected.insert(k);
        expected.insert(v);
    }
    ensure!(
        env.keys().all(|k| expected.contains(k)),
        "native overlay changes inherited settings or hooks"
    );
    Ok(())
}

#[cfg(test)]
mod primitive_tests {
    use super::*;
    fn declaration(writes: NativeHookWrites) -> NativeCompatConfig {
        NativeCompatConfig {
            profile: "rrx-native-inherited-v1".into(),
            cli_version: "2.1.283".into(),
            settings: "inherited".into(),
            user_hooks: vec![crate::config::NativeUserHook {
                label: "hook".into(),
                reference: "opaque/settings#hook".into(),
                writes,
            }],
        }
    }
    #[test]
    fn nongrant_native_role_hooks_include_user_and_committed_project() {
        for writes in [NativeHookWrites::None, NativeHookWrites::Worktree] {
            let config = declaration(writes);
            assert!(qualify_role_hooks(SessionRole::Executor, &config, &[]).is_ok());
            assert_eq!(
                qualify_role_hooks(SessionRole::Reviewer, &config, &[]).is_ok(),
                writes == NativeHookWrites::None
            );
            let config = declaration(NativeHookWrites::None);
            let required = [NativeRequiredHook {
                path: "hook.sh".into(),
                writes,
            }];
            assert!(qualify_role_hooks(SessionRole::Executor, &config, &required).is_ok());
            assert_eq!(
                qualify_role_hooks(SessionRole::Reviewer, &config, &required).is_ok(),
                writes == NativeHookWrites::None
            );
        }
    }
    #[test]
    fn nongrant_native_overlay_preserves_native_roots_and_required_hooks() {
        let mut env = BTreeMap::from([("GIT_CONFIG_COUNT".into(), "3".into())]);
        for (n, (key, value)) in [
            ("gc.auto", "0"),
            ("maintenance.auto", "false"),
            ("core.fsmonitor", "false"),
        ]
        .into_iter()
        .enumerate()
        {
            env.insert(format!("GIT_CONFIG_KEY_{n}"), key.into());
            env.insert(format!("GIT_CONFIG_VALUE_{n}"), value.into());
        }
        check_overlay(&env).unwrap();
        for key in [
            "HOME",
            "CODEX_HOME",
            "CLAUDE_CONFIG_DIR",
            "XDG_CONFIG_HOME",
            "GIT_CONFIG_GLOBAL",
            "GIT_CONFIG_SYSTEM",
            "GIT_CONFIG_NOSYSTEM",
            "UNKNOWN",
        ] {
            let mut mutated = env.clone();
            mutated.insert(key.into(), "opaque".into());
            assert!(check_overlay(&mutated).is_err(), "{key}");
        }
        env.insert("GIT_CONFIG_KEY_0".into(), "core.hooksPath".into());
        assert!(check_overlay(&env).is_err());
    }
}
