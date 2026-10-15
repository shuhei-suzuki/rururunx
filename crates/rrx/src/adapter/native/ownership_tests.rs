//! The real Registry factory installs the selected vtable. These controls do
//! not allocate a phase, create a Session, or invoke an official CLI.
use super::*;
use crate::config::{AgentConfig, Config};
use std::os::unix::fs::PermissionsExt;

fn installed(
    provider: &str,
) -> (
    tempfile::TempDir,
    AgentRegistry,
    Arc<NativePhasePort>,
    PathBuf,
) {
    let dir = tempfile::tempdir().unwrap();
    let owner = execution::RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
    let counter = dir.path().join("must-not-run");
    let program = dir.path().join("selected-cli");
    std::fs::write(
        &program,
        format!(
            "#!/bin/sh\nprintf invoked > '{}'\nexit 7\n",
            counter.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut config = Config::default();
    config.agents.insert(
        "selected".into(),
        AgentConfig {
            provider: Some(provider.into()),
            command: vec![program.to_string_lossy().into()],
            ..Default::default()
        },
    );
    let registry = AgentRegistry::from_managed_config(&config, owner).unwrap();
    let port = registry.native_phase_port("selected").unwrap();
    (dir, registry, port, counter)
}

#[tokio::test]
async fn selected_port_does_not_retain_adapter_or_session_registry() {
    for provider in ["claude", "codex"] {
        let (_dir, registry, port, counter) = installed(provider);
        let adapter = port.selected_adapter().unwrap();
        let adapter_identity = Arc::downgrade(&adapter);
        let session_identity = Arc::downgrade(&adapter.sessions);
        assert_eq!(port.alias(), "selected");
        assert_eq!(port.provider(), provider);
        assert!(Arc::ptr_eq(&adapter, &port.selected_adapter().unwrap()));
        drop(adapter);
        drop(registry);
        assert!(
            adapter_identity.upgrade().is_none(),
            "port retained adapter"
        );
        assert!(
            session_identity.upgrade().is_none(),
            "port retained sessions"
        );
        assert!(
            port.selected_adapter().is_err(),
            "ended vtable was restored"
        );
        assert!(
            !counter.exists(),
            "ownership observation invoked Native CLI"
        );
    }
}

#[tokio::test]
async fn selected_port_observes_only_original_live_vtable_and_origin() {
    let (_dir, registry, port, counter) = installed("claude");
    let adapter = port.selected_adapter().unwrap();
    let original_origin = port.origin_id();
    drop(registry);
    // A genuine in-flight caller can retain the original vtable; weak routing
    // does not revoke this factual object solely because the Registry dropped.
    assert!(Arc::ptr_eq(&adapter, &port.selected_adapter().unwrap()));
    let (_replacement_dir, replacement, replacement_port, replacement_counter) =
        installed("claude");
    assert_eq!(replacement_port.alias(), port.alias());
    assert_eq!(replacement_port.provider(), port.provider());
    assert_ne!(replacement_port.origin_id(), original_origin);
    assert!(!Arc::ptr_eq(
        &adapter,
        &replacement_port.selected_adapter().unwrap()
    ));
    drop(adapter);
    assert!(
        port.selected_adapter().is_err(),
        "alias replacement restored original origin"
    );
    assert!(replacement_port.selected_adapter().is_ok());
    assert_eq!(port.origin_id(), original_origin);
    assert!(!counter.exists());
    assert!(!replacement_counter.exists());
    drop(replacement);
}

/// EX-1: allocation refuses a Claude Executor before creating provisional assets.
#[tokio::test]
async fn claude_executor_allocation_refuses_before_provisional_assets() {
    let (_dir, owner, task) = execution::results::tests::fixture().await;
    let mut config = Config::default();
    config.agents.insert(
        "codex".into(),
        AgentConfig {
            provider: Some("claude".into()),
            command: vec![
                execution::native::tests::program(_dir.path(), "claude")
                    .to_string_lossy()
                    .into(),
            ],
            ..Default::default()
        },
    );
    let registry = AgentRegistry::from_managed_config(&config, owner.clone()).unwrap();
    let port = registry.native_phase_port("codex").unwrap();
    let (unit, _) = execution::attempts::AttemptManager::new(owner.clone())
        .prepare(task.id, "claude", "implement", None)
        .await
        .unwrap();
    let mut input = execution::native::tests::input(&unit, "complete");
    input.agent = "codex".into();
    let before = owner
        .store
        .lock()
        .unwrap()
        .managed_effects(unit.id)
        .unwrap()
        .len();
    let error = port.allocate(input, None, None).err().unwrap();
    assert_eq!(
        error.downcast_ref::<execution::capability::CapabilityRefusal>(),
        Some(&execution::capability::CapabilityRefusal::ClaudeExecutorUnavailable)
    );
    assert_eq!(
        execution::native::classify_failure(&error),
        execution::NativeFailure::UnsupportedCapability
    );
    assert_eq!(
        owner
            .store
            .lock()
            .unwrap()
            .managed_effects(unit.id)
            .unwrap()
            .len(),
        before
    );
}
