use rrx::{
    adapter::{InputKind, SharedStore},
    context::*,
    domain::*,
    git::WorktreeManager,
    state::Store,
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};
use tempfile::TempDir;

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
struct Fixture {
    _temp: TempDir,
    root: PathBuf,
    worktree: PathBuf,
    store: SharedStore,
    project: Project,
    task: Task,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap().join("repo");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "-b", "main"]);
        git(&root, &["config", "user.name", "Fixture"]);
        git(&root, &["config", "user.email", "fixture@example.invalid"]);
        git(&root, &["config", "commit.gpgsign", "false"]);
        git(&root, &["config", "core.hooksPath", ".git/hooks"]);
        std::fs::write(root.join(".gitignore"), "worktree/\nignored/\n").unwrap();
        std::fs::write(
            root.join("RULES.md"),
            "MANDATORY: preserve Project boundaries.\n",
        )
        .unwrap();
        std::fs::create_dir(root.join("src")).unwrap();
        std::fs::write(
            root.join("src/lib.rs"),
            "mod codec;\npub fn run() { codec::encode(); }\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/codec.rs"),
            "pub fn encode() -> String { String::new() }\n",
        )
        .unwrap();
        std::fs::write(root.join("unrelated.txt"), "weather sunshine\n").unwrap();
        git(&root, &["add", "."]);
        git(&root, &["commit", "-m", "fixture"]);
        let mut store = Store::open(&temp.path().join("state.db")).unwrap();
        let mut project = Project::new(
            "fixture".into(),
            root.clone(),
            rrx::git::repository_identity(&root, "main").unwrap(),
            "main".into(),
        );
        project.rule_refs = vec![root.join("RULES.md")];
        store.put_project(&mut project).unwrap();
        let mut goal = Goal::new(
            project.id,
            "Retain exact scope".into(),
            vec![CompletionCriterion {
                id: "done".into(),
                description: "verified independently".into(),
                evidence: None,
                satisfied: false,
            }],
        );
        store.put_goal(&mut goal).unwrap();
        let mut task = Task::new(project.id, goal.id, "Implement codec".into(), "fake".into());
        task.issue = Some(42);
        task.acceptance_criteria = vec!["never omit required evidence".into()];
        store.put_task(&mut task).unwrap();
        let worktree = WorktreeManager::create(&mut store, task.id)
            .unwrap()
            .worktree;
        task = store.task(task.id).unwrap().unwrap();
        Self {
            _temp: temp,
            root,
            worktree,
            store: Arc::new(Mutex::new(store)),
            project,
            task,
        }
    }
    fn engine(&self) -> RepositoryContext {
        RepositoryContext::new(self.store.clone())
    }
    async fn map(&self) -> RepositoryMap {
        self.engine()
            .index(&self.task.scope(), vec![])
            .await
            .unwrap()
    }
}
fn budget() -> Budget {
    Budget {
        estimated_tokens: 100_000,
        bytes: 100_000,
    }
}
fn ready(outcome: SelectionOutcome) -> ContextSlice {
    match outcome {
        SelectionOutcome::Ready { slice } => *slice,
        other => panic!("expected ready: {other:?}"),
    }
}

#[tokio::test]
async fn local_map_selection_graph_and_observable_estimates() {
    let f = Fixture::new();
    let engine = f.engine();
    let map = f.map().await;
    assert_eq!(map.freshness().scope, f.task.scope());
    assert_eq!(map.freshness().worktree, f.worktree);
    assert!(
        map.files()["src/codec.rs"]
            .symbols
            .iter()
            .any(|s| s.name == "encode")
    );
    assert!(
        map.files()["src/lib.rs"]
            .dependencies
            .contains("src/codec.rs")
    );
    assert!(
        map.files()["src/lib.rs"]
            .imports
            .iter()
            .any(|s| s.contains("mod codec"))
    );
    let request = SelectionRequest {
        task_text: "encode".into(),
        ..Default::default()
    };
    let slice = ready(engine.select(&map, &request, budget()).await.unwrap());
    assert!(
        slice
            .payload()
            .contains("MANDATORY: preserve Project boundaries.")
    );
    assert!(slice.payload().contains("Retain exact scope"));
    assert!(slice.payload().contains("never omit required evidence"));
    assert!(slice.evidence().selected.contains_key("src/codec.rs"));
    assert!(!slice.evidence().selected.contains_key("unrelated.txt"));
    assert_eq!(slice.evidence().estimated_tokens, slice.payload().len());
    assert_eq!(slice.evidence().measured_tokens, None);
    let second = ready(engine.select(&map, &request, budget()).await.unwrap());
    assert_eq!(slice.payload(), second.payload());
    let input = slice
        .prepared_input(&engine, InputKind::ContextPack, 1)
        .await
        .unwrap();
    assert_eq!(input.scope, f.task.scope());
    assert_eq!(input.revision, map.freshness().revision);
    assert!(
        slice
            .prepared_input(&engine, InputKind::ContextPack, 0)
            .await
            .is_err()
    );
    let events = f
        .store
        .lock()
        .unwrap()
        .events(&f.task.scope(), 0, 100)
        .unwrap();
    assert!(events.iter().any(|e| e.kind == "context.index.generated"));
    let event = events
        .iter()
        .find(|e| e.kind == "context.selection")
        .unwrap();
    assert!(event.data["evidence"]["measured_tokens"].is_null());
    assert!(
        !event
            .data
            .to_string()
            .contains("preserve Project boundaries")
    );
}
#[tokio::test]
async fn mandatory_rules_evidence_and_expansion_never_silently_drop() {
    let f = Fixture::new();
    std::fs::create_dir(f.worktree.join("ignored")).unwrap();
    std::fs::write(
        f.worktree.join("ignored/proof.txt"),
        "TEST PROOF: all required checks passed\n",
    )
    .unwrap();
    let engine = f.engine();
    let map = engine
        .index(&f.task.scope(), vec!["ignored/proof.txt".into()])
        .await
        .unwrap();
    let request = SelectionRequest {
        mandatory_evidence: vec!["ignored/proof.txt".into()],
        ..Default::default()
    };
    let slice = ready(engine.select(&map, &request, budget()).await.unwrap());
    assert!(slice.payload().contains("TEST PROOF"));
    let required = slice.evidence().required_bytes;
    let tiny = Budget {
        estimated_tokens: required - 1,
        bytes: required - 1,
    };
    assert!(matches!(
        engine.select(&map, &request, tiny).await.unwrap(),
        SelectionOutcome::NeedsBudget { .. }
    ));
    let expanded = ready(
        engine
            .expand(
                &map,
                &request,
                &Expansion::File {
                    path: "src/codec.rs".into(),
                },
                budget(),
            )
            .await
            .unwrap(),
    );
    assert!(expanded.payload().contains("pub fn encode()"));
    assert!(expanded.payload().contains("TEST PROOF"));
    assert!(matches!(
        engine
            .expand(
                &map,
                &request,
                &Expansion::File {
                    path: "src/codec.rs".into()
                },
                Budget {
                    estimated_tokens: required,
                    bytes: required
                }
            )
            .await
            .unwrap(),
        SelectionOutcome::NeedsBudget { .. }
    ));
    assert!(
        engine
            .select(
                &map,
                &SelectionRequest {
                    mandatory_evidence: vec!["missing.txt".into()],
                    ..Default::default()
                },
                budget()
            )
            .await
            .is_err()
    );
}
#[tokio::test]
async fn budget_counts_actual_rendered_wrapper_and_optional_omissions() {
    let f = Fixture::new();
    let engine = f.engine();
    let map = f.map().await;
    let full = ready(
        engine
            .select(
                &map,
                &SelectionRequest {
                    task_text: "codec encode".into(),
                    ..Default::default()
                },
                budget(),
            )
            .await
            .unwrap(),
    );
    let exactly = Budget {
        estimated_tokens: full.evidence().required_bytes,
        bytes: full.evidence().required_bytes,
    };
    let slice = ready(
        engine
            .select(
                &map,
                &SelectionRequest {
                    task_text: "codec encode".into(),
                    ..Default::default()
                },
                exactly,
            )
            .await
            .unwrap(),
    );
    assert_eq!(slice.payload().len(), exactly.bytes);
    assert!(slice.evidence().omitted.contains(&"src/codec.rs".into()));
    assert!(
        engine
            .select(
                &map,
                &SelectionRequest::default(),
                Budget {
                    estimated_tokens: 0,
                    bytes: 10
                }
            )
            .await
            .is_err()
    );
}
#[tokio::test]
async fn explicit_file_symbol_callers_and_callees_are_scoped() {
    let f = Fixture::new();
    let engine = f.engine();
    let map = f.map().await;
    for expansion in [
        Expansion::File {
            path: "src/codec.rs".into(),
        },
        Expansion::Symbol {
            name: "encode".into(),
        },
        Expansion::Callers {
            name: "encode".into(),
        },
        Expansion::Callees { name: "run".into() },
    ] {
        let slice = ready(
            engine
                .expand(&map, &SelectionRequest::default(), &expansion, budget())
                .await
                .unwrap(),
        );
        assert!(slice.payload().contains("pub fn encode()"));
    }
    assert!(
        engine
            .expand(
                &map,
                &SelectionRequest::default(),
                &Expansion::Symbol {
                    name: "missing".into()
                },
                budget()
            )
            .await
            .is_err()
    );
    assert!(
        engine
            .expand(
                &map,
                &SelectionRequest::default(),
                &Expansion::File {
                    path: "../RULES.md".into()
                },
                budget()
            )
            .await
            .is_err()
    );
    assert!(
        f.store
            .lock()
            .unwrap()
            .events(&f.task.scope(), 0, 100)
            .unwrap()
            .iter()
            .any(|e| e.kind == "context.expansion")
    );
}
#[tokio::test]
async fn dirty_sources_added_deleted_and_head_changes_invalidate() {
    let f = Fixture::new();
    let engine = f.engine();
    let map = f.map().await;
    std::fs::write(
        f.worktree.join("src/codec.rs"),
        "pub fn encode() { panic!(\"dirty\") }\n",
    )
    .unwrap();
    assert!(engine.validate(&map).await.is_err());
    let dirty = f.map().await;
    assert!(dirty.files()["src/codec.rs"].changed);
    let selected = ready(
        engine
            .select(&dirty, &SelectionRequest::default(), budget())
            .await
            .unwrap(),
    );
    assert!(selected.evidence().selected.contains_key("src/codec.rs"));
    std::fs::write(f.worktree.join("new.rs"), "fn new_symbol() {}\n").unwrap();
    assert!(engine.validate(&dirty).await.is_err());
    let added = f.map().await;
    assert!(added.files()["new.rs"].changed);
    std::fs::remove_file(f.worktree.join("src/codec.rs")).unwrap();
    assert!(engine.validate(&added).await.is_err());
    let deleted = f.map().await;
    assert_eq!(
        deleted.freshness().source_hashes["worktree:src/codec.rs"],
        "missing"
    );
    git(&f.worktree, &["add", "."]);
    git(&f.worktree, &["commit", "-m", "changed"]);
    assert!(engine.validate(&deleted).await.is_err());
}
#[tokio::test]
async fn ignored_and_primary_rules_configs_have_independent_hash_freshness() {
    let f = Fixture::new();
    std::fs::create_dir(f.worktree.join("ignored")).unwrap();
    std::fs::write(f.worktree.join("ignored/proof.txt"), "first").unwrap();
    let engine = f.engine();
    let map = engine
        .index(&f.task.scope(), vec!["ignored/proof.txt".into()])
        .await
        .unwrap();
    std::fs::write(f.worktree.join("ignored/proof.txt"), "second").unwrap();
    assert!(engine.validate(&map).await.is_err());
    let map = engine
        .index(&f.task.scope(), vec!["ignored/proof.txt".into()])
        .await
        .unwrap();
    std::fs::write(
        f.root.join("RULES.md"),
        "Changed primary rule, unchanged Task HEAD\n",
    )
    .unwrap();
    assert!(engine.validate(&map).await.is_err());
    let mut project = f
        .store
        .lock()
        .unwrap()
        .project(f.project.id)
        .unwrap()
        .unwrap();
    std::fs::write(
        f.root.join("project.toml"),
        "minimum_workflow = 'standard'\n",
    )
    .unwrap();
    project.config_ref = Some(f.root.join("project.toml"));
    f.store.lock().unwrap().put_project(&mut project).unwrap();
    let map = f.map().await;
    assert!(
        map.freshness()
            .source_hashes
            .contains_key("config:project.toml")
    );
    std::fs::write(f.root.join("project.toml"), "minimum_workflow = 'triple'\n").unwrap();
    assert!(engine.validate(&map).await.is_err());
}
#[tokio::test]
async fn same_issue_number_foreign_scope_and_project_state_cannot_reuse_context() {
    let f = Fixture::new();
    let other = Fixture::new();
    assert_eq!(f.task.issue, other.task.issue);
    let map = f.map().await;
    assert!(other.engine().validate(&map).await.is_err());
    let foreign = Scope::task(f.project.id, f.task.goal_id, other.task.id);
    assert!(f.engine().index(&foreign, vec![]).await.is_err());
    let mut project = f
        .store
        .lock()
        .unwrap()
        .project(f.project.id)
        .unwrap()
        .unwrap();
    project.state = ProjectState::Blocked;
    project.blocked_reason = Some("missing repository".into());
    f.store.lock().unwrap().put_project(&mut project).unwrap();
    assert!(f.engine().validate(&map).await.is_err());
    assert!(f.engine().index(&f.task.scope(), vec![]).await.is_err());
}
#[tokio::test]
async fn state_version_recheck_rejects_changed_task_goal_and_worktree_branch() {
    let f = Fixture::new();
    let map = f.map().await;
    let mut task = f.task.clone();
    task.title = "new task purpose".into();
    f.store.lock().unwrap().put_task(&mut task).unwrap();
    assert!(f.engine().validate(&map).await.is_err());
    let map = f.map().await;
    let mut goal = f.store.lock().unwrap().goal(task.goal_id).unwrap().unwrap();
    goal.objective = "new Goal".into();
    f.store.lock().unwrap().put_goal(&mut goal).unwrap();
    assert!(f.engine().validate(&map).await.is_err());
    let map = f.map().await;
    git(&f.worktree, &["checkout", "-b", "foreign-branch"]);
    assert!(f.engine().validate(&map).await.is_err());
}

#[tokio::test]
async fn in_flight_index_releases_store_and_rejects_concurrent_state_mutation() {
    use std::{future::Future, task::Poll};
    let f = Fixture::new();
    let engine = f.engine();
    let scope = f.task.scope();
    let mut operation = Box::pin(engine.index(&scope, vec![]));
    // Poll actual native indexing through its first asynchronous Git wait. The
    // current-thread runtime has not run its piped-output reader tasks yet.
    std::future::poll_fn(|cx| {
        assert!(matches!(operation.as_mut().poll(cx), Poll::Pending));
        Poll::Ready(())
    })
    .await;
    let mut store = f
        .store
        .try_lock()
        .expect("Git/source work must release SharedStore");
    let mut task = store.task(f.task.id).unwrap().unwrap();
    task.title = "concurrent authorized update".into();
    store.put_task(&mut task).unwrap();
    drop(store);
    assert!(
        operation.await.is_err(),
        "obsolete snapshot must not publish index evidence"
    );
    let events = f.store.lock().unwrap().events(&scope, 0, 100).unwrap();
    assert!(!events.iter().any(|e| e.kind == "context.index.generated"));
}
#[tokio::test]
async fn missing_moved_or_replaced_source_root_cannot_rebind() {
    let f = Fixture::new();
    let other = Fixture::new();
    let map = f.map().await;
    std::fs::rename(&f.root, f.root.with_file_name("moved")).unwrap();
    assert!(f.engine().validate(&map).await.is_err());
    std::fs::rename(&other.root, &f.root).unwrap();
    assert!(f.engine().index(&f.task.scope(), vec![]).await.is_err());
}
#[tokio::test]
async fn symlink_nested_repo_metadata_namespace_and_special_files_fail_closed() {
    let f = Fixture::new();
    let other = Fixture::new();
    let engine = f.engine();
    for path in [
        "../other",
        ".git/config",
        f.root.join("RULES.md").to_str().unwrap(),
    ] {
        assert!(
            engine
                .index(&f.task.scope(), vec![path.into()])
                .await
                .is_err()
        );
    }
    std::os::unix::fs::symlink(other.root.join("RULES.md"), f.worktree.join("link")).unwrap();
    assert!(
        engine
            .index(&f.task.scope(), vec!["link".into()])
            .await
            .is_err()
    );
    std::fs::remove_file(f.worktree.join("link")).unwrap();
    std::os::unix::fs::symlink(&other.root, f.worktree.join("foreign")).unwrap();
    assert!(
        engine
            .index(&f.task.scope(), vec!["foreign/RULES.md".into()])
            .await
            .is_err()
    );
    std::fs::remove_file(f.worktree.join("foreign")).unwrap();
    std::fs::create_dir(f.worktree.join("nested")).unwrap();
    git(&f.worktree.join("nested"), &["init", "-b", "main"]);
    std::fs::write(f.worktree.join("nested/data"), "FOREIGN").unwrap();
    assert!(
        engine
            .index(&f.task.scope(), vec!["nested/data".into()])
            .await
            .is_err()
    );
    std::fs::remove_dir_all(f.worktree.join("nested")).unwrap();
    let mut project = f
        .store
        .lock()
        .unwrap()
        .project(f.project.id)
        .unwrap()
        .unwrap();
    project.rule_refs = vec![f.worktree.join("src/codec.rs")];
    f.store.lock().unwrap().put_project(&mut project).unwrap();
    assert!(
        engine.index(&f.task.scope(), vec![]).await.is_err(),
        "primary references must not enter Task namespace"
    );
    project.rule_refs = vec![f.root.join("RULES.md")];
    f.store.lock().unwrap().put_project(&mut project).unwrap();
    std::fs::remove_file(f.root.join("RULES.md")).unwrap();
    std::os::unix::fs::symlink(other.root.join("RULES.md"), f.root.join("RULES.md")).unwrap();
    assert!(
        engine.index(&f.task.scope(), vec![]).await.is_err(),
        "authoritative rule symlink must not cross Projects"
    );
    std::fs::remove_file(f.root.join("RULES.md")).unwrap();
    std::fs::write(f.root.join("RULES.md"), "restored mandatory rule").unwrap();
    assert!(
        Command::new("mkfifo")
            .arg(f.worktree.join("fifo"))
            .status()
            .unwrap()
            .success()
    );
    assert!(
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            engine.index(&f.task.scope(), vec!["fifo".into()])
        )
        .await
        .unwrap()
        .is_err()
    );
}
#[tokio::test]
async fn binary_bounded_inputs_and_ignored_path_admission_are_explicit() {
    let f = Fixture::new();
    std::fs::write(f.worktree.join("binary"), [0, 255, 1]).unwrap();
    std::fs::create_dir(f.worktree.join("ignored")).unwrap();
    std::fs::write(f.worktree.join("ignored/proof"), "proof").unwrap();
    let map = f.map().await;
    assert!(map.skipped().contains_key("binary"));
    assert!(
        map.freshness()
            .source_hashes
            .contains_key("worktree:binary")
    );
    assert!(!map.files().contains_key("ignored/proof"));
    assert!(
        f.engine()
            .select(
                &map,
                &SelectionRequest {
                    changed_files: vec!["ignored/proof".into()],
                    ..Default::default()
                },
                budget()
            )
            .await
            .is_err()
    );
    assert!(
        f.engine()
            .select(
                &map,
                &SelectionRequest {
                    task_text: "x".repeat(8193),
                    ..Default::default()
                },
                budget()
            )
            .await
            .is_err()
    );
    std::fs::write(f.worktree.join("large"), vec![b'a'; 256 * 1024 + 1]).unwrap();
    assert!(f.engine().index(&f.task.scope(), vec![]).await.is_err());
}
