use super::*;
fn step() -> PipelineStep {
    serde_json::from_value(json!({"run_id":"fixture","seq":2,"attempt":1,"kind":"implement","provider":"codex","owner_epoch":1,"round":0,"state":"exited","version":1,"cookie":null,"pid":null,"birth":null,"step_base":"a".repeat(40),"input_sha256":"b".repeat(64),"result_commit":null,"result_tree":null,"body":{"death_confirmed":true,"exit":{"issue":null}}})).unwrap()
}
/// Every adoption requires an explicit observation of certified death.
#[test]
fn b5_store_death_guard() {
    let mut step = step();
    certifying_exit(&step).unwrap();
    for flag in [Value::Null, json!(false)] {
        step.body["death_confirmed"] = flag;
        assert!(certifying_exit(&step).is_err());
    }
}
/// Any recorded exit issue prevents commit and adoption certification.
#[test]
fn b5_store_issue_guard() {
    let mut step = step();
    for issue in [json!("output_io"), json!(false), json!("")] {
        step.body["exit"]["issue"] = issue;
        assert!(certifying_exit(&step).is_err());
    }
}

/// Scope resolution never guesses a binding between multiple plan keys and Task identities.
#[test]
fn b5_scope_binding() {
    let task = TaskId::new();
    let other = TaskId::new();
    let policy = json!({"task_scopes":{"operator-key":["src/**"]}});
    assert_eq!(
        executor_scopes(&policy, &[task], task).unwrap(),
        vec!["src/**"]
    );
    assert!(executor_scopes(&policy, &[task, other], task).is_err());
    assert!(
        executor_scopes(
            &json!({"task_scopes":{"one":["src/**"],"two":["tests/**"]}}),
            &[task],
            task
        )
        .is_err()
    );
}

#[cfg(target_os = "macos")]
async fn exited_with_issue(kind: &str) -> (Fixture, String) {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, kind);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_running(&f.owner, &cursor, 4242, "fixture-birth")
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(
            &f.owner,
            &cursor,
            &death(),
            Some("success"),
            &json!({"issue":"output_io","code":0}),
        )
        .unwrap();
    (f, id)
}
/// The commit Store API enforces issue rejection even when its caller forgets.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_commit_api_issue() {
    let (f, id) = exited_with_issue("implement").await;
    let cursor = f.cursor(&id);
    assert!(
        f.store()
            .pipeline_record_commit(
                &f.owner,
                &cursor,
                &"b".repeat(40),
                &"c".repeat(40),
                &json!({})
            )
            .is_err()
    );
    assert_eq!(f.cursor(&id).step_version, cursor.step_version);
}
/// The adoption Store API enforces issue rejection for decoded Agent output.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_adopt_api_issue() {
    let (f, id) = exited_with_issue("review").await;
    let cursor = f.cursor(&id);
    assert!(
        f.store()
            .pipeline_adopt(&f.owner, &cursor, &json!({}))
            .is_err()
    );
    assert_eq!(f.cursor(&id).step_version, cursor.step_version);
}

#[cfg(target_os = "macos")]
async fn prepare_fixture() -> (Fixture, std::sync::Arc<Runtime>, String) {
    prepare_fixture_with_file(false).await
}
#[cfg(target_os = "macos")]
async fn prepare_fixture_with_file(tracked: bool) -> (Fixture, std::sync::Arc<Runtime>, String) {
    let f = fixture().await;
    let runtime = std::sync::Arc::new(Runtime::new(f.owner.clone(), f.config.clone()).unwrap());
    runtime.start_qualified_service().await.unwrap();
    let project = f.store().project(f.project).unwrap().unwrap();
    let git = crate::execution::pipeline::PipelineGit::new(&project.root).unwrap();
    git.run(&["init", "-b", "main"]).await.unwrap();
    let tree = if tracked {
        let oid = String::from_utf8(
            git.invoke(
                &["hash-object", "-w", "--no-filters", "--stdin"],
                b"safe\n",
                None,
                false,
                4096,
            )
            .await
            .unwrap(),
        )
        .unwrap();
        String::from_utf8(
            git.invoke(
                &["mktree"],
                format!("100644 blob {}\ttracked\n", oid.trim()).as_bytes(),
                None,
                false,
                4096,
            )
            .await
            .unwrap(),
        )
        .unwrap()
        .trim()
        .into()
    } else {
        git.text(&["mktree"]).await.unwrap()
    };
    let base = String::from_utf8(
        git.invoke(&["commit-tree", &tree], b"base\n", None, true, 4096)
            .await
            .unwrap(),
    )
    .unwrap()
    .trim()
    .to_owned();
    git.run(&["update-ref", "refs/heads/main", &base])
        .await
        .unwrap();
    let id = Uuid::new_v4().to_string();
    let work = f
        .config
        .pipeline
        .as_ref()
        .unwrap()
        .work_root
        .join(format!("task_{id}"));
    let run = PipelineRun {
        id: id.clone(),
        kind: "task".into(),
        task_id: Some(f.task),
        goal_id: f.goal,
        project_id: f.project,
        owner_epoch: f.owner.epoch,
        version: 0,
        state: "running".into(),
        round: 0,
        base_commit: base.clone(),
        head_commit: base.clone(),
        worktree: Some(work.to_str().unwrap().into()),
        branch: Some(format!("rrx/task/{id}")),
        body: json!({}),
    };
    let mut first = planned(&id, 1, 1, "prepare");
    first.step_base = base.clone();
    let mut journal = create_journal(&f);
    journal.old_commit = "0".repeat(base.len());
    journal.new_commit = base;
    f.store()
        .pipeline_claim_run(&f.owner, &run, 1, &first, Some(&journal))
        .unwrap();
    (f, runtime, id)
}

/// Task-only conditional config is rejected before any file is checked out.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_f1_prepare_conditional_driver() {
    use crate::execution::pipeline::PipelineGit;
    for condition in ["onbranch", "gitdir"] {
        let (f, runtime, id) = prepare_fixture_with_file(true).await;
        let project = f.store().project(f.project).unwrap().unwrap();
        let git = PipelineGit::new(&project.root).unwrap();
        let view = f.store().pipeline_goal_view(f.goal).unwrap();
        let branch = view.runs[0].branch.as_ref().unwrap();
        let work = Path::new(view.runs[0].worktree.as_ref().unwrap());
        let root = &f.config.pipeline.as_ref().unwrap().work_root;
        let marker = root.join("smudge-marker");
        let include = root.join("task.inc");
        let attributes = root.join("filter-attributes");
        std::fs::write(&attributes, b"* filter=fixture\n").unwrap();
        std::fs::write(
            &include,
            format!(
                "[core]\nattributesFile = {}\n[filter \"fixture\"]\nsmudge = touch '{}'; cat\n",
                attributes.display(),
                marker.display()
            ),
        )
        .unwrap();
        let key = if condition == "onbranch" {
            format!("includeIf.onbranch:{branch}.path")
        } else {
            let common = git
                .text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
                .await
                .unwrap();
            format!("includeIf.gitdir:{common}/worktrees/**.path")
        };
        git.run(&["config", "--local", &key, include.to_str().unwrap()])
            .await
            .unwrap();
        assert!(
            git.reject_tree_drivers(&view.runs[0].base_commit)
                .await
                .is_ok()
        );
        assert!(!runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
        let view = f.store().pipeline_goal_view(f.goal).unwrap();
        assert_eq!(view.steps[0].state, "held");
        assert_eq!(view.steps[0].body["reason"], "git_driver_declared");
        assert!(!marker.exists());
        assert!(!work.join("tracked").exists());
        runtime.shutdown().await.unwrap();
    }
}

/// Project working attributes are irrelevant to materializing a clean Task tree.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_f1_prepare_ignores_project_files() {
    let (f, runtime, id) = prepare_fixture().await;
    let project = f.store().project(f.project).unwrap().unwrap();
    std::fs::create_dir(project.root.join("node_modules")).unwrap();
    std::fs::write(
        project.root.join("node_modules/.gitattributes"),
        b"* -diff\n",
    )
    .unwrap();
    assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    runtime.shutdown().await.unwrap();
}

/// Prepare refuses symbolic Goal refs without touching their targets.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_f1_prepare_symbolic_goal() {
    use crate::execution::pipeline::PipelineGit;
    let (f, runtime, id) = prepare_fixture().await;
    let project = f.store().project(f.project).unwrap().unwrap();
    let git = PipelineGit::new(&project.root).unwrap();
    let goal = format!("refs/heads/rrx/goal/{}", f.goal);
    git.run(&["symbolic-ref", &goal, "refs/heads/main"])
        .await
        .unwrap();
    let base = git.text(&["rev-parse", "refs/heads/main"]).await.unwrap();
    assert!(!runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    assert_eq!(
        git.text(&["rev-parse", "refs/heads/main"]).await.unwrap(),
        base
    );
    assert_eq!(
        f.store().pipeline_goal_view(f.goal).unwrap().steps[0].state,
        "held"
    );
    runtime.shutdown().await.unwrap();
}

/// Executor preflight persists a declared-driver reason instead of generic worktree drift.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_f1_launch_driver_reason() {
    use sha2::{Digest, Sha256};
    let (f, runtime, id) = prepare_fixture().await;
    assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let work = Path::new(view.runs[0].worktree.as_ref().unwrap());
    std::fs::write(work.join(".gitattributes"), b"* filter=fixture\n").unwrap();
    let mut next = planned(&id, 2, 1, "implement");
    next.step_base = view.runs[0].base_commit.clone();
    next.input_sha256 = format!("{:x}", Sha256::digest(b"implement\n"));
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap();
    let result = runtime
        .launch_pipeline_executor(f.cursor(&id), b"implement\n".to_vec())
        .await
        .unwrap();
    if let crate::execution::pipeline::LaunchResult::Started(custody) = result {
        custody.request_stop();
        custody.wait().await.unwrap();
        panic!("declared driver reached launch");
    }
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(view.steps.last().unwrap().state, "held");
    assert_eq!(
        view.steps.last().unwrap().body["reason"],
        "git_driver_declared"
    );
    runtime.shutdown().await.unwrap();
}

/// The limited Store API refuses an exited Agent result carrying an exit issue.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_f1_limited_issue() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(
            &f.owner,
            &cursor,
            &death(),
            Some("limit"),
            &json!({"issue":"output_io"}),
        )
        .unwrap();
    let cursor = f.cursor(&id);
    assert!(
        f.store()
            .pipeline_record_limited(&f.owner, &cursor)
            .is_err()
    );
    assert_eq!(f.cursor(&id).step_version, cursor.step_version);
}

/// Secret additions beginning with pluses hold the Runtime step without moving its ref.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_f1_plus_secret_held() {
    use crate::execution::pipeline::PipelineGit;
    let (f, runtime, id) = executor_exited_fixture().await;
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let work = Path::new(view.runs[0].worktree.as_ref().unwrap());
    std::fs::write(work.join("src/output.rs"), b"++ EXAMPLE_SECRET=fixture\n").unwrap();
    assert!(
        !runtime
            .adopt_pipeline_executor(f.cursor(&id))
            .await
            .unwrap()
    );
    let after = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(after.steps.last().unwrap().state, "held");
    assert_eq!(
        after.steps.last().unwrap().body["reason"],
        "secret_detected"
    );
    assert_eq!(
        PipelineGit::new(work)
            .unwrap()
            .text(&["rev-parse", "HEAD"])
            .await
            .unwrap(),
        view.runs[0].head_commit
    );
    runtime.shutdown().await.unwrap();
}

/// Committed recovery holds missing diff evidence before applying the recorded ref update.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_f1_missing_committed_diff() {
    use crate::execution::pipeline::{AdoptionChecker, PipelineGit, TaskWorkspace};
    let (f, runtime, id) = executor_exited_fixture().await;
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let run = &view.runs[0];
    let step = view.steps.last().unwrap();
    let project = f.store().project(f.project).unwrap().unwrap();
    let config = f.config.pipeline.as_ref().unwrap();
    let path = Path::new(run.worktree.as_ref().unwrap());
    let work = TaskWorkspace::bound(
        &project.root,
        &config.work_root,
        path,
        run.branch.as_ref().unwrap(),
        &step.step_base,
    )
    .unwrap();
    let attempt = config.work_root.join("runs").join(&id).join("2-1");
    let candidate = AdoptionChecker::build(&work, step, &["src/**".into()], &attempt)
        .await
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_commit(
            &f.owner,
            &cursor,
            &candidate.commit,
            &candidate.tree,
            &candidate.evidence,
        )
        .unwrap();
    std::fs::remove_file(attempt.join("diff.patch")).unwrap();
    assert!(
        !runtime
            .adopt_pipeline_executor(f.cursor(&id))
            .await
            .unwrap()
    );
    let after = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(after.steps.last().unwrap().state, "held");
    assert_eq!(
        after.steps.last().unwrap().body["reason"],
        "output_evidence_changed"
    );
    assert_eq!(
        PipelineGit::new(path)
            .unwrap()
            .text(&["rev-parse", "HEAD"])
            .await
            .unwrap(),
        run.head_commit
    );
    runtime.shutdown().await.unwrap();
}

/// Prepare records its journal before creating a clean registered Task worktree.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_prepare_materializes() {
    let (f, runtime, id) = prepare_fixture().await;
    assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let run = &view.runs[0];
    let step = &view.steps[0];
    assert_eq!(step.state, "adopted");
    assert!(step.body["adoption"]["controls"].is_object());
    assert!(view.journals.iter().all(|j| j.state == "done"));
    let git =
        crate::execution::pipeline::PipelineGit::new(Path::new(run.worktree.as_ref().unwrap()))
            .unwrap();
    assert_eq!(
        git.text(&["rev-parse", "HEAD"]).await.unwrap(),
        run.base_commit
    );
    assert!(
        git.run(&["status", "--porcelain=v1", "-z"])
            .await
            .unwrap()
            .is_empty()
    );
    runtime.shutdown().await.unwrap();
}

/// A partial Task materialization is retained for operator recovery.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_prepare_partial() {
    let (f, runtime, id) = prepare_fixture().await;
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let work = Path::new(view.runs[0].worktree.as_ref().unwrap());
    std::fs::create_dir(work).unwrap();
    std::fs::write(work.join("keep"), b"retained").unwrap();
    assert!(!runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    assert_eq!(std::fs::read(work.join("keep")).unwrap(), b"retained");
    assert_eq!(
        f.store().pipeline_goal_view(f.goal).unwrap().steps[0].state,
        "held"
    );
    runtime.shutdown().await.unwrap();
}

/// A recovered prepare with no Task artifacts is interrupted without inventing death evidence.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_prepare_absence() {
    let (f, runtime, id) = prepare_fixture().await;
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_begin_launch(&f.owner, &cursor, None, &json!({}))
        .unwrap();
    assert!(!runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(view.steps[0].state, "interrupted");
    assert!(view.steps[0].body["death_evidence"].is_null());
    runtime.shutdown().await.unwrap();
}

/// A dirty planned checkout cannot launch the Executor through its Runtime entry point.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_dirty_launch_caller() {
    use sha2::{Digest, Sha256};
    let (f, runtime, id) = prepare_fixture().await;
    runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap();
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    std::fs::write(
        Path::new(view.runs[0].worktree.as_ref().unwrap()).join("unexpected"),
        b"drift",
    )
    .unwrap();
    let mut next = planned(&id, 2, 1, "implement");
    next.input_sha256 = format!("{:x}", Sha256::digest(b"implement\n"));
    next.step_base = view.runs[0].base_commit.clone();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap();
    let result = runtime
        .launch_pipeline_executor(f.cursor(&id), b"implement\n".to_vec())
        .await
        .unwrap();
    let held = matches!(result, crate::execution::pipeline::LaunchResult::Held);
    if let crate::execution::pipeline::LaunchResult::Started(custody) = result {
        custody.request_stop();
        custody.wait().await.unwrap();
    }
    assert!(held);
    assert_eq!(
        f.store()
            .pipeline_goal_view(f.goal)
            .unwrap()
            .steps
            .last()
            .unwrap()
            .state,
        "held"
    );
    runtime.shutdown().await.unwrap();
}

#[cfg(target_os = "macos")]
async fn executor_exited_fixture() -> (Fixture, std::sync::Arc<Runtime>, String) {
    executor_exited_with_output("{\"type\":\"turn.completed\"}\n", "0\n").await
}
#[cfg(target_os = "macos")]
async fn executor_exited_with_output(
    output: &str,
    status: &str,
) -> (Fixture, std::sync::Arc<Runtime>, String) {
    use crate::execution::pipeline::{
        LaunchResult, PipelineRole, step_arguments, test_support::FakeCli,
    };
    use sha2::{Digest, Sha256};
    let (f, runtime, id) = prepare_fixture().await;
    assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let work = Path::new(view.runs[0].worktree.as_ref().unwrap());
    let prompt = b"Implement the accepted Task.\n";
    let mut next = planned(&id, 2, 1, "implement");
    next.step_base = view.runs[0].base_commit.clone();
    next.input_sha256 = format!("{:x}", Sha256::digest(prompt));
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap();
    let config = f.config.pipeline.as_ref().unwrap();
    let fake = FakeCli {
        directory: config.clis.codex.executable.parent().unwrap().into(),
        cli: config.clis.codex.clone(),
    };
    let attempt = config.work_root.join("runs").join(&id).join("2-1");
    let args = step_arguments(PipelineRole::Implement, &fake.cli, work, &attempt, "").unwrap();
    fake.expect_arguments(&args.iter().map(String::as_str).collect::<Vec<_>>());
    fake.control("exec-source", "pub fn result() {}\n");
    fake.control("exec-last", "Implementation complete.\n");
    fake.control("exec-output", output);
    fake.control("exec-status", status);
    let LaunchResult::Started(custody) = runtime
        .launch_pipeline_executor(f.cursor(&id), prompt.to_vec())
        .await
        .unwrap()
    else {
        panic!("Executor did not start")
    };
    custody.wait().await.unwrap();
    (f, runtime, id)
}

/// The Executor Runtime path launches through custody and records a checked commit before adoption.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_executor_flow() {
    let (f, runtime, id) = executor_exited_fixture().await;
    assert!(
        runtime
            .adopt_pipeline_executor(f.cursor(&id))
            .await
            .unwrap()
    );
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let work = Path::new(view.runs[0].worktree.as_ref().unwrap());
    let attempt = f
        .config
        .pipeline
        .as_ref()
        .unwrap()
        .work_root
        .join("runs")
        .join(&id)
        .join("2-1");
    let adopted = view.steps.last().unwrap();
    assert_eq!(adopted.state, "adopted");
    assert_eq!(
        adopted.result_commit.as_deref(),
        Some(view.runs[0].head_commit.as_str())
    );
    let git = crate::execution::pipeline::PipelineGit::new(work).unwrap();
    assert_eq!(
        git.text(&["rev-parse", "HEAD"]).await.unwrap(),
        view.runs[0].head_commit
    );
    assert!(
        git.run(&["status", "--porcelain=v1", "-z"])
            .await
            .unwrap()
            .is_empty()
    );
    assert!(attempt.join("diff.patch").is_file());
    runtime.shutdown().await.unwrap();
}

/// Committed recovery verifies retained stdout and last-message bytes before moving the ref.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_f11_committed_evidence() {
    use crate::execution::pipeline::{AdoptionChecker, PipelineGit, TaskWorkspace};
    use std::os::unix::fs::PermissionsExt;
    for name in ["stdout.jsonl", "last.md"] {
        let (f, runtime, id) = executor_exited_fixture().await;
        let view = f.store().pipeline_goal_view(f.goal).unwrap();
        let run = &view.runs[0];
        let step = view.steps.last().unwrap();
        let project = f.store().project(f.project).unwrap().unwrap();
        let config = f.config.pipeline.as_ref().unwrap();
        let work_path = Path::new(run.worktree.as_ref().unwrap());
        let work = TaskWorkspace::bound(
            &project.root,
            &config.work_root,
            work_path,
            run.branch.as_ref().unwrap(),
            &step.step_base,
        )
        .unwrap();
        let attempt = config.work_root.join("runs").join(&id).join("2-1");
        let candidate = AdoptionChecker::build(&work, step, &["src/**".into()], &attempt)
            .await
            .unwrap();
        let cursor = f.cursor(&id);
        f.store()
            .pipeline_record_commit(
                &f.owner,
                &cursor,
                &candidate.commit,
                &candidate.tree,
                &candidate.evidence,
            )
            .unwrap();
        let path = attempt.join(name);
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[0] ^= 1;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::write(&path, bytes).unwrap();
        assert!(
            !runtime
                .adopt_pipeline_executor(f.cursor(&id))
                .await
                .unwrap()
        );
        let view = f.store().pipeline_goal_view(f.goal).unwrap();
        assert_eq!(view.steps.last().unwrap().state, "held");
        assert_eq!(view.runs[0].head_commit, run.base_commit);
        assert_eq!(
            PipelineGit::new(work_path)
                .unwrap()
                .text(&["rev-parse", "HEAD"])
                .await
                .unwrap(),
            run.base_commit
        );
        runtime.shutdown().await.unwrap();
    }
}

/// A known provider limit reaches the existing limited path without committing worktree changes.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_executor_limit_wiring() {
    let (f, runtime, id) = executor_exited_with_output("{\"type\":\"turn.failed\",\"error\":{\"code\":\"rate_limit_exceeded\",\"message\":\"Rate limit reached\"}}\n", "1\n").await;
    assert!(
        !runtime
            .adopt_pipeline_executor(f.cursor(&id))
            .await
            .unwrap()
    );
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(view.steps.last().unwrap().state, "limited");
    assert!(view.steps.last().unwrap().result_commit.is_none());
    assert_eq!(view.runs[0].head_commit, view.runs[0].base_commit);
    runtime.shutdown().await.unwrap();
}
