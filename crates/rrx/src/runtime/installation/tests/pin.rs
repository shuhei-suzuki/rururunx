//! Claude CLI pin 2.1.294 (design `issue-43-claude-cli-pin-2-1-294.md` §4).
//! B1 precedes version probing; these Claude Executor pin cases are retired.
//! AC-01 removes number pins; current production pins are unchanged by U2.
use super::*;

/// A stale Claude declaration cannot precede or bypass the B1 role refusal.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn pin_c2_old_declared_cli_version_is_refused_before_native() {
    let f = fixture_with("claude", true, |config| {
        let compat = config.agents.get_mut("worker").unwrap();
        compat.compatibility.as_mut().unwrap().cli_version = "2.1.283".into();
    });
    assert_claude_executor_refused(f).await;
}

/// An obsolete peer version is never probed when B1 refuses its Executor role.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn pin_c3_old_observed_cli_version_is_refused_before_native() {
    let f = fixture_with("claude", true, |config| {
        let script = std::path::PathBuf::from(&config.agents["worker"].command[0]);
        let source = std::fs::read_to_string(&script).unwrap();
        assert!(source.contains("\"2.1.294 (Claude Code)\""), "SETUP");
        std::fs::write(
            &script,
            source.replacen("\"2.1.294 (Claude Code)\"", "\"2.1.283 (Claude Code)\"", 1),
        )
        .unwrap();
    });
    assert_claude_executor_refused(f).await;
}
