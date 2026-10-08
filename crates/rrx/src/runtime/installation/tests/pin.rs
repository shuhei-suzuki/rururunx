//! Claude CLI pin 2.1.294 (design `issue-43-claude-cli-pin-2-1-294.md` §4).
//! Accepted ingress, the installed issuer and the genuine service loop; the
//! only variation is the declared or the peer-reported CLI version.
use super::*;

async fn refused(f: &ControlFixture, needle: &str, label: &str) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        let jobs = f.runtime.phase_jobs.observed_jobs();
        if jobs
            .iter()
            .any(|j| j.refusal.as_deref().is_some_and(|r| r.contains(needle)))
        {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "{label}: refusal {needle:?} not observed; jobs {jobs:?}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

/// C2: a 2.1.283 declaration after the bump is refused, typed, before any
/// NativeInput, Native invocation or Session.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn pin_c2_old_declared_cli_version_is_refused_before_native() {
    let mut f = fixture_with("claude", true, |config| {
        let compat = config.agents.get_mut("worker").unwrap();
        compat.compatibility.as_mut().unwrap().cli_version = "2.1.283".into();
    });
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    refused(&f, "native compatibility version unsupported", "C2").await;
    assert_eq!(
        count(&f, "native_invocations"),
        0,
        "C2: no Native invocation"
    );
    assert!(
        success::links(&f, &tasks[0]).is_empty(),
        "C2: no Session binding"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

/// C3: declared 2.1.294 but the real peer reports 2.1.283. The SAME version
/// observation does not qualify, so the helper qualification refuses before
/// any NativeInput (the contract per Sol 6058424878 M01).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn pin_c3_old_observed_cli_version_is_refused_before_native() {
    let mut f = fixture_with("claude", true, |config| {
        let script = std::path::PathBuf::from(&config.agents["worker"].command[0]);
        let source = std::fs::read_to_string(&script).unwrap();
        assert!(source.contains("\"2.1.294 (Claude Code)\""), "SETUP");
        std::fs::write(
            &script,
            source.replacen("\"2.1.294 (Claude Code)\"", "\"2.1.283 (Claude Code)\"", 1),
        )
        .unwrap();
    });
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    refused(
        &f,
        "original helper qualification failed or remains unknown",
        "C3",
    )
    .await;
    assert_eq!(
        count(&f, "native_invocations"),
        0,
        "C3: no Native invocation"
    );
    assert!(
        success::links(&f, &tasks[0]).is_empty(),
        "C3: no Session binding"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
