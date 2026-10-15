//! Provider-independent version guards; AC-01 replacement remains U7 work.
use super::*;

/// A real peer's unqualified version is refused before invocation or Session binding.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn pin_c3_old_observed_cli_version_is_refused_before_native() {
    let mut f = fixture_with("codex", true, |config| {
        let script = std::path::PathBuf::from(&config.agents["worker"].command[0]);
        let source = std::fs::read_to_string(&script).unwrap();
        assert!(source.contains("\"codex-cli 0.160.0\""), "SETUP");
        std::fs::write(
            &script,
            source.replacen("\"codex-cli 0.160.0\"", "\"codex-cli 0.159.0\"", 1),
        )
        .unwrap();
    });
    f.register_real_git_project();
    assert!(f.runtime.installed.is_ok(), "SETUP: graph unavailable");
    let (_, tasks) = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    success::wait_for(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.refusal.is_some())
                || count(&f, "native_invocations") > 0
        },
        "C3: preparation did not reach a decision",
        60,
    )
    .await;
    assert!(
        f.runtime.phase_jobs.observed_jobs().iter().any(|j| {
            j.refusal.as_deref().is_some_and(|r| {
                r.contains("original helper qualification failed or remains unknown")
            })
        }),
        "C3: observed version was not refused before NativeInput"
    );
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
