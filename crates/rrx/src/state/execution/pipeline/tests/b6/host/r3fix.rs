use super::*;
use std::os::unix::fs::MetadataExt;

async fn prepare_again(
    f: &Fixture,
    id: &str,
) -> crate::execution::pipeline::tests_runner::PreparedTest {
    let cursor = f.cursor(id);
    let (run, step) = f
        .store()
        .pipeline_launch_snapshot(&f.owner, &cursor)
        .unwrap();
    let profile = f
        .store()
        .pipeline_test_context(&f.owner, &cursor)
        .unwrap()
        .unwrap();
    PipelineTestRunner::prepare(
        &f.owner,
        f.config.pipeline.as_ref().unwrap(),
        &run,
        &step,
        profile,
    )
    .await
    .unwrap()
}

async fn hardlinked_leftover(ignored: bool) {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    let first = prepare_again(&f, &id).await;
    let git = PipelineGit::new(&first.checkout).unwrap();
    let relative = if ignored {
        let common = git
            .text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
            .await
            .unwrap();
        std::fs::write(
            std::path::Path::new(&common).join("info/exclude"),
            b"ignored-build/\n",
        )
        .unwrap();
        std::fs::create_dir(first.checkout.join("ignored-build")).unwrap();
        std::fs::write(first.checkout.join("ignored-build/output"), b"build-output").unwrap();
        "ignored-build/output".to_owned()
    } else {
        git.text(&["ls-files"])
            .await
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .to_owned()
    };
    let file = first.checkout.join(&relative);
    let outside = f._dir.path().join("outside-link-target");
    let contents = std::fs::read(&file).unwrap();
    let mode = std::fs::metadata(&file).unwrap().mode();
    std::fs::copy(&file, &outside).unwrap();
    std::fs::remove_file(&file).unwrap();
    std::fs::hard_link(&outside, &file).unwrap();
    assert_eq!(std::fs::metadata(&file).unwrap().mode(), mode);
    assert!(
        git.run(&["status", "--porcelain=v1", "--untracked-files=all"])
            .await
            .unwrap()
            .is_empty()
    );
    let rebuilt = prepare_again(&f, &id).await;
    let file = rebuilt.checkout.join(relative);
    if ignored {
        assert!(!file.exists());
        std::fs::create_dir(file.parent().unwrap()).unwrap();
    } else {
        assert_eq!(std::fs::metadata(&file).unwrap().nlink(), 1);
        assert_eq!(std::fs::read(&file).unwrap(), contents);
    }
    std::fs::write(file, b"new checkout output").unwrap();
    assert_eq!(std::fs::read(outside).unwrap(), contents);
    assert_eq!(
        std::fs::metadata(rebuilt.checkout.join(".git"))
            .unwrap()
            .nlink(),
        1
    );
    runtime.shutdown().await.unwrap();
}

/// A Git-clean same-content same-mode tracked hardlink is rebuilt without changing its target.
#[tokio::test]
async fn b6_f3_checkout_tracked_hardlink() {
    hardlinked_leftover(false).await;
}

/// An ignored leftover hardlink is rebuilt without changing its outside target.
#[tokio::test]
async fn b6_f3_checkout_ignored_hardlink() {
    hardlinked_leftover(true).await;
}

/// An unreadable ignored directory cannot survive as a successfully reused checkout.
#[tokio::test]
async fn b6_f3_checkout_unreadable_reuse() {
    use std::os::unix::fs::PermissionsExt;
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    let first = prepare_again(&f, &id).await;
    let git = PipelineGit::new(&first.checkout).unwrap();
    let common = git
        .text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .await
        .unwrap();
    std::fs::write(
        std::path::Path::new(&common).join("info/exclude"),
        b"unreadable/\n",
    )
    .unwrap();
    let nested = first.checkout.join("unreadable");
    std::fs::create_dir(&nested).unwrap();
    std::fs::write(nested.join("output"), b"ignored output").unwrap();
    let cursor = f.cursor(&id);
    let (run, step) = f
        .store()
        .pipeline_launch_snapshot(&f.owner, &cursor)
        .unwrap();
    let profile = f
        .store()
        .pipeline_test_context(&f.owner, &cursor)
        .unwrap()
        .unwrap();
    std::fs::set_permissions(&nested, std::fs::Permissions::from_mode(0o000)).unwrap();
    let result = PipelineTestRunner::prepare(
        &f.owner,
        f.config.pipeline.as_ref().unwrap(),
        &run,
        &step,
        profile,
    )
    .await;
    let remains = nested.exists();
    if remains {
        std::fs::set_permissions(&nested, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    // Removal may itself refuse the unreadable directory; either way no successful reuse.
    assert!(result.is_err() || !remains);
    runtime.shutdown().await.unwrap();
}

/// Retained secrets hold adoption with only fixed pattern, filename and offset-digest metadata.
#[tokio::test]
async fn b6_f3_test_secret_detected() {
    for (code, file) in [
        ("print 'ghp_fixture-marker';", "stdout.log"),
        ("print STDERR 'ghp_fixture-marker';", "stderr.log"),
    ] {
        let (f, runtime, id) = planned_test(code, 10, 1024, 1024).await;
        finish(&f, &runtime, &id, true).await;
        assert!(!runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
        let view = f.store().pipeline_goal_view(f.goal).unwrap();
        let step = view.steps.last().unwrap();
        assert_eq!(step.state, "held");
        assert_eq!(step.body["reason"], "secret_detected");
        assert_eq!(
            step.body["secret"],
            json!({"pattern":"fixed_secret", "file":file,
            "offset_sha256": hash(&0usize.to_le_bytes())})
        );
        assert!(
            !serde_json::to_string(&view)
                .unwrap()
                .contains("ghp_fixture-marker")
        );
        runtime.shutdown().await.unwrap();
    }
}

/// Missing streams and digest mismatches retain the evidence-incomplete classification.
#[tokio::test]
async fn b6_f3_test_incomplete_evidence() {
    use std::os::unix::fs::PermissionsExt;
    for missing in [true, false] {
        let (f, runtime, id) = planned_test("print 'ok';", 10, 1024, 1024).await;
        finish(&f, &runtime, &id, true).await;
        let file = f
            .config
            .pipeline
            .as_ref()
            .unwrap()
            .work_root
            .join("runs")
            .join(&id)
            .join("2-1/stdout.log");
        if missing {
            std::fs::remove_file(file).unwrap();
        } else {
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
            std::fs::write(file, b"no").unwrap();
        }
        assert!(!runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
        let view = f.store().pipeline_goal_view(f.goal).unwrap();
        let step = view.steps.last().unwrap();
        assert_eq!(step.state, "held");
        assert_eq!(step.body["reason"], "test_evidence_incomplete");
        assert!(step.body["secret"].is_null());
        runtime.shutdown().await.unwrap();
    }
}
