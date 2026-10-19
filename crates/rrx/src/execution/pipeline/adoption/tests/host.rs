use super::*;
use crate::execution::pipeline::PipelineGit;

struct Fixture {
    _dir: tempfile::TempDir,
    work: TaskWorkspace,
    step: PipelineStep,
    attempt: std::path::PathBuf,
}
impl Fixture {
    async fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let project = root.join("project");
        let private = root.join("private");
        std::fs::create_dir(&project).unwrap();
        std::fs::create_dir(&private).unwrap();
        let git = PipelineGit::new(&project).unwrap();
        git.run(&["init", "-b", "main"]).await.unwrap();
        let tree = git.text(&["mktree"]).await.unwrap();
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
        let task = private.join("task");
        git.run(&[
            "worktree",
            "add",
            "-b",
            "rrx/task/fixture",
            task.to_str().unwrap(),
            &base,
        ])
        .await
        .unwrap();
        let work =
            TaskWorkspace::bound(&project, &private, &task, "rrx/task/fixture", &base).unwrap();
        let controls = work.controls().await.unwrap();
        let attempt = private.join("attempt");
        std::fs::create_dir(&attempt).unwrap();
        let mut exit = json!({"code":0,"issue":null});
        for (key, name, bytes) in [
            ("stdout", "stdout.jsonl", b"{}\n".as_slice()),
            ("stderr", "stderr.log", b"".as_slice()),
            ("last", "last.md", b"done\n".as_slice()),
        ] {
            std::fs::write(attempt.join(name), bytes).unwrap();
            exit[key] = json!({"bytes":bytes.len(),"sha256":hash(bytes),"complete":true});
        }
        let step = PipelineStep {
            run_id: "fixture".into(),
            seq: 2,
            attempt: 1,
            kind: "implement".into(),
            provider: Some("codex".into()),
            owner_epoch: 1,
            round: 0,
            state: "exited".into(),
            version: 1,
            cookie: None,
            pid: None,
            birth: None,
            step_base: base,
            input_sha256: hash(b"prompt"),
            result_commit: None,
            result_tree: None,
            body: json!({"death_confirmed":true,"provider_outcome":"success","exit":exit,"executor_controls":controls}),
        };
        Self {
            _dir: dir,
            work,
            step,
            attempt,
        }
    }
    fn change(&self) {
        file(self.work.git.root(), "src/file.rs", b"safe code\n");
    }
    async fn build(&self) -> Result<Candidate> {
        AdoptionChecker::build(&self.work, &self.step, &scope(), &self.attempt).await
    }
    async fn set_base(&mut self, entries: &[u8]) {
        let tree = String::from_utf8(
            self.work
                .git
                .invoke(&["mktree"], entries, None, false, 4096)
                .await
                .unwrap(),
        )
        .unwrap();
        let commit = String::from_utf8(
            self.work
                .git
                .invoke(&["commit-tree", tree.trim()], b"base\n", None, true, 4096)
                .await
                .unwrap(),
        )
        .unwrap();
        let commit = commit.trim();
        self.work
            .git
            .run(&["update-ref", "--no-deref", &self.work.branch, commit])
            .await
            .unwrap();
        self.work.git.run(&["read-tree", commit]).await.unwrap();
        self.step.step_base = commit.into();
        self.work.base = commit.into();
        self.save_controls().await;
    }
    async fn blob(&self, bytes: &[u8]) -> String {
        String::from_utf8(
            self.work
                .git
                .invoke(
                    super::super::super::git::BLOB_ARGS,
                    bytes,
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
    }
    async fn save_controls(&mut self) {
        self.step.body["executor_controls"] =
            serde_json::to_value(self.work.controls().await.unwrap()).unwrap();
    }
}

async fn hostile_subrepo(f: &Fixture) -> std::path::PathBuf {
    let sub = f.work.git.root().join("sub");
    std::fs::create_dir(&sub).unwrap();
    let git = PipelineGit::new(&sub).unwrap();
    git.run(&["init", "-b", "main"]).await.unwrap();
    file(&sub, "file", b"initial\n");
    git.run(&["add", "file"]).await.unwrap();
    file(&sub, "file", b"changed\n");
    let marker = f.attempt.join("filter-marker");
    git.run(&[
        "config",
        "--local",
        "filter.fixture.clean",
        &format!("touch '{}'; cat", marker.display()),
    ])
    .await
    .unwrap();
    file(&sub, ".git/info/attributes", b"* filter=fixture\n");
    git.run(&["status", "--porcelain=v1", "--ignore-submodules=all"])
        .await
        .unwrap();
    assert!(
        marker.exists(),
        "the hostile filter fixture must be executable"
    );
    std::fs::remove_file(&marker).unwrap();
    marker
}

/// A gitlink base cannot enter an Agent-written subrepository or execute its filter.
#[tokio::test]
async fn b5_f1_gitlink_filter() {
    let mut f = Fixture::new().await;
    f.set_base(format!("160000 commit {}\tsub\n", f.step.step_base).as_bytes())
        .await;
    let marker = hostile_subrepo(&f).await;
    f.change();
    assert_eq!(
        f.build().await.err().unwrap().to_string(),
        "git_driver_declared"
    );
    assert!(!marker.exists());
    assert_eq!(f.work.ref_value().await.unwrap(), f.step.step_base);
    assert!(f.work.clean_at(&f.step.step_base).await.is_err());
    assert!(!marker.exists());
}

/// Status never enters a subrepository introduced only through the Agent's index.
#[tokio::test]
async fn b5_f1_index_gitlink_filter() {
    let f = Fixture::new().await;
    let marker = hostile_subrepo(&f).await;
    f.work
        .git
        .run(&[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            &f.step.step_base,
            "sub",
        ])
        .await
        .unwrap();
    f.change();
    assert!(f.build().await.is_err());
    assert!(!marker.exists());
    assert!(f.work.clean_at(&f.step.step_base).await.is_err());
    assert!(!marker.exists());
}

/// Clean verification refuses an index-only gitlink before status can hide its directory.
#[tokio::test]
async fn b5_f1h2_index_gitlink_clean() {
    let f = Fixture::new().await;
    let marker = hostile_subrepo(&f).await;
    f.work
        .git
        .run(&[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            &f.step.step_base,
            "sub",
        ])
        .await
        .unwrap();
    assert_eq!(
        f.work
            .clean_at(&f.step.step_base)
            .await
            .unwrap_err()
            .to_string(),
        "git_driver_declared"
    );
    assert!(!marker.exists());
}

/// Hidden tracked instructions refuse adoption and clean verification without moving the Task ref.
#[tokio::test]
async fn b5_f3_hidden_index_instructions() {
    for flag in ["--assume-unchanged", "--skip-worktree"] {
        let mut f = Fixture::new().await;
        let oid = f.blob(b"base instructions\n").await;
        f.set_base(format!("100644 blob {oid}\tAGENTS.md\n").as_bytes())
            .await;
        file(f.work.git.root(), "AGENTS.md", b"base instructions\n");
        f.work.clean_at(&f.step.step_base).await.unwrap();
        f.work
            .git
            .run(&["update-index", flag, "AGENTS.md"])
            .await
            .unwrap();
        file(f.work.git.root(), "AGENTS.md", b"hostile instructions\n");
        assert!(
            f.work
                .git
                .run(&["status", "--porcelain=v1", "-z"])
                .await
                .unwrap()
                .is_empty(),
            "the index flag must hide the rewritten tracked file"
        );
        assert_eq!(
            f.build().await.err().unwrap().to_string(),
            "git_driver_declared",
            "{flag} build"
        );
        assert_eq!(f.work.ref_value().await.unwrap(), f.step.step_base);
        assert_eq!(
            f.work
                .clean_at(&f.step.step_base)
                .await
                .unwrap_err()
                .to_string(),
            "git_driver_declared",
            "{flag} clean"
        );
        assert_eq!(f.work.ref_value().await.unwrap(), f.step.step_base);
    }
}

/// The binary macro in any tree attribute casing refuses secret-bearing changes.
#[tokio::test]
async fn b5_f1_binary_tree() {
    for name in [".gitattributes", ".GitAttributes"] {
        let mut f = Fixture::new().await;
        let oid = f.blob(b"* binary\n").await;
        f.set_base(format!("100644 blob {oid}\t{name}\n").as_bytes())
            .await;
        file(
            f.work.git.root(),
            "src/file.rs",
            b"EXAMPLE_SECRET=fixture\n",
        );
        assert_eq!(
            f.build().await.err().unwrap().to_string(),
            "git_driver_declared"
        );
        assert_eq!(f.work.ref_value().await.unwrap(), f.step.step_base);
    }
}

/// Agent-created self-ignore rules cannot hide protected paths from adoption or move the ref.
#[tokio::test]
async fn b5_f2_ignored_build() {
    let f = Fixture::new().await;
    f.change();
    file(
        f.work.git.root(),
        ".gitignore",
        b"/.gitignore\n/AGENTS.override.md\n",
    );
    file(f.work.git.root(), "AGENTS.override.md", b"instruction\n");
    assert_eq!(f.build().await.err().unwrap().to_string(), "protected_path");
    assert_eq!(f.work.ref_value().await.unwrap(), f.step.step_base);
    assert!(!f.attempt.join("diff.patch").exists());
}

/// Base-tree ignores cannot hide a local instruction file from the clean check.
#[tokio::test]
async fn b5_f2_ignored_clean() {
    let mut f = Fixture::new().await;
    let rules = b"/CLAUDE.local.md\n";
    let oid = f.blob(rules).await;
    f.set_base(format!("100644 blob {oid}\t.gitignore\n").as_bytes())
        .await;
    file(f.work.git.root(), ".gitignore", rules);
    f.work.clean_at(&f.step.step_base).await.unwrap();
    file(f.work.git.root(), "CLAUDE.local.md", b"instruction\n");
    assert_eq!(
        f.work
            .clean_at(&f.step.step_base)
            .await
            .unwrap_err()
            .to_string(),
        "protected_path"
    );
    assert_eq!(f.work.ref_value().await.unwrap(), f.step.step_base);
}

/// Ordinary ignored build output is neither a captured change nor worktree drift.
#[tokio::test]
async fn b5_f2_ignored_build_output() {
    let mut f = Fixture::new().await;
    let rules = b"/target/\n";
    let oid = f.blob(rules).await;
    f.set_base(format!("100644 blob {oid}\t.gitignore\n").as_bytes())
        .await;
    file(f.work.git.root(), ".gitignore", rules);
    file(
        f.work.git.root(),
        "target/debug/output",
        b"ordinary output\n",
    );
    f.work.clean_at(&f.step.step_base).await.unwrap();
    f.change();
    let candidate = f.build().await.unwrap();
    assert!(
        f.work
            .git
            .run(&["ls-tree", "-r", &candidate.tree, "--", "target"])
            .await
            .unwrap()
            .is_empty()
    );
    f.step.state = "committed".into();
    f.step.result_commit = Some(candidate.commit);
    f.step.result_tree = Some(candidate.tree);
    AdoptionChecker::finish(&f.work, &f.step).await.unwrap();
    assert_eq!(
        std::fs::read(f.work.git.root().join("target/debug/output")).unwrap(),
        b"ordinary output\n"
    );
}

/// Agent-written ignore rules cannot hide protected paths even with `.gitignore` in scope.
#[tokio::test]
async fn b5_f3_gitignore_scope_build() {
    let scope = [".gitignore".to_owned(), "src/**".to_owned()];
    let f = Fixture::new().await;
    f.change();
    file(f.work.git.root(), ".gitignore", b"/scratch/\n");
    file(f.work.git.root(), "scratch/CLAUDE.md", b"instruction\n");
    assert_eq!(
        AdoptionChecker::build(&f.work, &f.step, &scope, &f.attempt)
            .await
            .err()
            .unwrap()
            .to_string(),
        "protected_path"
    );
    assert_eq!(f.work.ref_value().await.unwrap(), f.step.step_base);
    assert!(!f.attempt.join("diff.patch").exists());
    // Base ignore rules still admit ordinary ignored output under the same scope.
    let mut f = Fixture::new().await;
    let rules = b"/target/\n";
    let oid = f.blob(rules).await;
    f.set_base(format!("100644 blob {oid}\t.gitignore\n").as_bytes())
        .await;
    file(f.work.git.root(), ".gitignore", rules);
    file(f.work.git.root(), "target/output", b"ordinary output\n");
    f.change();
    AdoptionChecker::build(&f.work, &f.step, &scope, &f.attempt)
        .await
        .unwrap();
}

/// A staged rename source is a path record, never an ignored entry.
#[tokio::test]
async fn b5_f3_rename_source_not_ignored() {
    let mut f = Fixture::new().await;
    let oid = f.blob(b"notes\n").await;
    f.set_base(format!("100644 blob {oid}\t!! CLAUDE.md\n").as_bytes())
        .await;
    file(f.work.git.root(), "!! CLAUDE.md", b"notes\n");
    std::fs::create_dir(f.work.git.root().join("src")).unwrap();
    f.work
        .git
        .run(&["mv", "!! CLAUDE.md", "src/moved.md"])
        .await
        .unwrap();
    assert_eq!(
        f.work
            .git
            .run(&["status", "--porcelain=v1", "-z"])
            .await
            .unwrap(),
        b"R  src/moved.md\0!! CLAUDE.md\0",
        "the fixture must produce a rename record"
    );
    AdoptionChecker::build(&f.work, &f.step, &["**".into()], &f.attempt)
        .await
        .unwrap();
}

/// Unicode attribute aliases in the selected tree are inspected even before materialization.
#[tokio::test]
async fn b5_f2_tree_attribute_unicode() {
    let mut f = Fixture::new().await;
    let oid = f.blob(b"* ident\n").await;
    f.set_base(format!("100644 blob {oid}\t.gitattributeſ\n").as_bytes())
        .await;
    assert_eq!(
        f.work
            .git
            .reject_tree_drivers(&f.step.step_base)
            .await
            .unwrap_err()
            .to_string(),
        "git_driver_declared"
    );
    assert_eq!(f.work.ref_value().await.unwrap(), f.step.step_base);
}

/// Tree driver inspection rejects gitlinks and accepts ordinary blobs.
#[tokio::test]
async fn b5_f3_tree_gitlink_direct() {
    let mut f = Fixture::new().await;
    f.set_base(format!("160000 commit {}\tsub\n", f.step.step_base).as_bytes())
        .await;
    assert_eq!(
        f.work
            .git
            .reject_tree_drivers(&f.step.step_base)
            .await
            .unwrap_err()
            .to_string(),
        "git_driver_declared"
    );

    let oid = f.blob(b"ordinary content\n").await;
    f.set_base(format!("100644 blob {oid}\tfile\n").as_bytes())
        .await;
    f.work
        .git
        .reject_tree_drivers(&f.step.step_base)
        .await
        .unwrap();
}

/// A tiny binary threshold cannot omit secret-bearing text from the scan diff.
#[tokio::test]
async fn b5_f1_bigfile_secret() {
    let mut f = Fixture::new().await;
    f.work
        .git
        .run(&["config", "--local", "core.bigFileThreshold", "1"])
        .await
        .unwrap();
    f.save_controls().await;
    file(
        f.work.git.root(),
        "src/file.rs",
        b"EXAMPLE_SECRET=fixture\n",
    );
    assert!(
        f.build()
            .await
            .err()
            .unwrap()
            .chain()
            .any(|e| e.to_string() == "secret_detected")
    );
    assert_eq!(f.work.ref_value().await.unwrap(), f.step.step_base);
}

/// Ident normalization cannot hide a scope-external rewrite beside an allowed change.
#[tokio::test]
async fn b5_f1_ident_scope() {
    let mut f = Fixture::new().await;
    let attr = f.blob(b"outside ident\n").await;
    let original = f.blob(b"$Id$\n").await;
    f.set_base(
        format!("100644 blob {attr}\t.gitattributes\n100644 blob {original}\toutside\n").as_bytes(),
    )
    .await;
    file(f.work.git.root(), ".gitattributes", b"outside ident\n");
    file(
        f.work.git.root(),
        "outside",
        b"$Id: EXAMPLE_SECRET=fixture $\n",
    );
    f.change();
    assert_eq!(
        f.build().await.err().unwrap().to_string(),
        "git_driver_declared"
    );
    assert_eq!(f.work.ref_value().await.unwrap(), f.step.step_base);
}

/// Symbolic Task refs cannot redirect adoption to the Goal branch.
#[tokio::test]
async fn b5_f1_symbolic_task_ref() {
    let f = Fixture::new().await;
    f.change();
    f.work
        .git
        .run(&[
            "update-ref",
            "refs/heads/rrx/goal/fixture",
            &f.step.step_base,
        ])
        .await
        .unwrap();
    f.work
        .git
        .run(&[
            "symbolic-ref",
            &f.work.branch,
            "refs/heads/rrx/goal/fixture",
        ])
        .await
        .unwrap();
    assert!(f.work.ref_value().await.is_err());
    assert!(f.build().await.is_err());
    assert_eq!(
        f.work
            .git
            .text(&["rev-parse", "refs/heads/rrx/goal/fixture"])
            .await
            .unwrap(),
        f.step.step_base
    );
}

/// Exited adoption rebuilds interrupted diff publication and finishes with sealed evidence.
#[tokio::test]
async fn b5_f1_artifact_resume_adoption() {
    for crash_point in 0..3 {
        let mut f = Fixture::new().await;
        f.change();
        f.build().await.unwrap();
        let path = f.attempt.join("diff.patch");
        let diff = std::fs::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        let mut artifact = PendingArtifact::new(&path).unwrap();
        let count = [0, diff.len() / 2, diff.len()][crash_point];
        artifact
            .pending
            .as_file_mut()
            .write_all(&diff[..count])
            .unwrap();
        artifact.pending.keep().unwrap();
        assert!(!path.exists());
        let candidate = f.build().await.unwrap();
        f.step.state = "committed".into();
        f.step.result_commit = Some(candidate.commit.clone());
        f.step.result_tree = Some(candidate.tree);
        f.step.body["commit_evidence"] = candidate.evidence;
        diff_evidence(&f.step, &f.attempt).unwrap();
        AdoptionChecker::finish(&f.work, &f.step).await.unwrap();
        assert_eq!(f.work.ref_value().await.unwrap(), candidate.commit);
        assert_eq!(std::fs::read(&path).unwrap(), diff);
        assert_eq!(
            std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o400
        );
    }
}
/// Captured bytes become the recorded commit and are adopted with a clean matching checkout.
#[tokio::test]
async fn b5_plumbing_adoption() {
    let mut f = Fixture::new().await;
    f.change();
    let candidate = f.build().await.unwrap();
    assert_eq!(f.work.ref_value().await.unwrap(), f.step.step_base);
    f.step.state = "committed".into();
    f.step.result_commit = Some(candidate.commit.clone());
    f.step.result_tree = Some(candidate.tree);
    AdoptionChecker::finish(&f.work, &f.step).await.unwrap();
    assert_eq!(
        f.work
            .git
            .text(&["show", &format!("{}:src/file.rs", candidate.commit)])
            .await
            .unwrap(),
        "safe code"
    );
    f.work.clean_at(&candidate.commit).await.unwrap();
}
/// An Agent-created commit is refused even with a plausible parent and rrx trailers.
#[tokio::test]
async fn b5_agent_commit() {
    let f = Fixture::new().await;
    f.change();
    let tree = f
        .work
        .git
        .text(&["rev-parse", &format!("{}^{{tree}}", f.step.step_base)])
        .await
        .unwrap();
    let spoof = String::from_utf8(
        f.work
            .git
            .invoke(
                &["commit-tree", &tree, "-p", &f.step.step_base],
                b"Rrx-Run: fixture\nRrx-Step: 2-1\n",
                None,
                true,
                4096,
            )
            .await
            .unwrap(),
    )
    .unwrap()
    .trim()
    .to_owned();
    f.work
        .git
        .run(&["update-ref", &f.work.branch, &spoof, &f.step.step_base])
        .await
        .unwrap();
    assert!(f.build().await.is_err());
    assert!(!f.attempt.join("diff.patch").exists());
}
/// Missing changes cannot create an adopted empty Executor result.
#[tokio::test]
async fn b5_no_change() {
    for index_only in [false, true] {
        let f = Fixture::new().await;
        if index_only {
            let blob = String::from_utf8(
                f.work
                    .git
                    .invoke(
                        crate::execution::pipeline::git::BLOB_ARGS,
                        b"index only\n",
                        None,
                        false,
                        4096,
                    )
                    .await
                    .unwrap(),
            )
            .unwrap()
            .trim()
            .to_owned();
            f.work
                .git
                .run(&[
                    "update-index",
                    "--add",
                    "--cacheinfo",
                    "100644",
                    &blob,
                    "src/ghost.rs",
                ])
                .await
                .unwrap();
        }
        assert_eq!(f.build().await.err().unwrap().to_string(), "no_change");
    }
}
/// Git control changes after launch prevent blob construction.
#[tokio::test]
async fn b5_control_changed() {
    let f = Fixture::new().await;
    f.change();
    f.work
        .git
        .run(&["config", "--local", "fixture.changed", "yes"])
        .await
        .unwrap();
    assert!(f.build().await.is_err());
    assert!(!f.attempt.join("diff.patch").exists());
}
/// Declared attribute drivers cannot execute during adoption.
#[tokio::test]
async fn b5_driver_precedes_status() {
    let f = Fixture::new().await;
    f.change();
    file(
        f.work.git.root(),
        "src/deep/.gitattributes",
        b"* filter=malicious\n",
    );
    assert_eq!(
        f.build().await.err().unwrap().to_string(),
        "git_driver_declared"
    );
}
/// Evidence changed after exit cannot be trusted by the adoption caller.
#[tokio::test]
async fn b5_evidence_caller() {
    let f = Fixture::new().await;
    f.change();
    std::fs::write(f.attempt.join("stdout.jsonl"), b"xx\n").unwrap();
    assert!(f.build().await.is_err());
}
/// A failed exit cannot produce a commit even when the workspace has valid changes.
#[tokio::test]
async fn b5_failed_exit() {
    let mut f = Fixture::new().await;
    f.change();
    f.step.body["exit"]["code"] = json!(1);
    assert!(f.build().await.is_err());
}
/// Commit construction requires certified death and an issue-free exit.
#[tokio::test]
async fn b5_uncertified_exit() {
    let mut f = Fixture::new().await;
    f.change();
    f.step.body["death_confirmed"] = json!(false);
    assert!(f.build().await.is_err());
    f.step.body["death_confirmed"] = json!(true);
    f.step.body["exit"]["issue"] = json!("failed");
    assert!(f.build().await.is_err());
}
/// A resumed commit accepts only the recorded SHA with its recorded tree.
#[tokio::test]
async fn b5_recorded_identity() {
    let mut f = Fixture::new().await;
    f.change();
    let candidate = f.build().await.unwrap();
    f.step.state = "committed".into();
    f.step.result_commit = Some(candidate.commit);
    f.step.result_tree = Some("f".repeat(40));
    assert!(AdoptionChecker::finish(&f.work, &f.step).await.is_err());
}
/// A recorded commit with the correct tree but a different parent cannot move the Task ref.
#[tokio::test]
async fn b5_f3_recorded_parent() {
    let mut f = Fixture::new().await;
    f.change();
    let candidate = f.build().await.unwrap();
    let commit = String::from_utf8(
        f.work
            .git
            .invoke(
                &["commit-tree", &candidate.tree],
                b"other\n",
                None,
                true,
                4096,
            )
            .await
            .unwrap(),
    )
    .unwrap()
    .trim()
    .to_owned();
    f.step.state = "committed".into();
    f.step.result_commit = Some(commit);
    f.step.result_tree = Some(candidate.tree);
    assert_eq!(
        AdoptionChecker::finish(&f.work, &f.step)
            .await
            .unwrap_err()
            .to_string(),
        "recorded_commit_changed"
    );
    assert_eq!(f.work.ref_value().await.unwrap(), f.step.step_base);
}
/// An unrelated ref move is preserved and held during committed recovery.
#[tokio::test]
async fn b5_unrelated_ref() {
    let mut f = Fixture::new().await;
    f.change();
    let candidate = f.build().await.unwrap();
    f.step.state = "committed".into();
    f.step.result_commit = Some(candidate.commit);
    f.step.result_tree = Some(candidate.tree.clone());
    let unrelated = String::from_utf8(
        f.work
            .git
            .invoke(
                &["commit-tree", &candidate.tree, "-p", &f.step.step_base],
                b"other\n",
                None,
                true,
                4096,
            )
            .await
            .unwrap(),
    )
    .unwrap()
    .trim()
    .to_owned();
    f.work
        .git
        .run(&["update-ref", &f.work.branch, &unrelated, &f.step.step_base])
        .await
        .unwrap();
    assert_eq!(
        AdoptionChecker::finish(&f.work, &f.step)
            .await
            .unwrap_err()
            .to_string(),
        "ref_moved"
    );
    assert_eq!(f.work.ref_value().await.unwrap(), unrelated);
}
/// Initialization discards only the bound Task worktree's unadopted changes.
#[tokio::test]
async fn b5_initialize() {
    let f = Fixture::new().await;
    f.change();
    let tree = f
        .work
        .git
        .text(&["rev-parse", &format!("{}^{{tree}}", f.step.step_base)])
        .await
        .unwrap();
    let agent_commit = String::from_utf8(
        f.work
            .git
            .invoke(
                &["commit-tree", &tree, "-p", &f.step.step_base],
                b"unadopted Agent commit\n",
                None,
                true,
                4096,
            )
            .await
            .unwrap(),
    )
    .unwrap()
    .trim()
    .to_owned();
    f.work
        .git
        .run(&[
            "update-ref",
            &f.work.branch,
            &agent_commit,
            &f.step.step_base,
        ])
        .await
        .unwrap();
    let controls = serde_json::from_value(f.step.body["executor_controls"].clone()).unwrap();
    f.work.initialize(&controls).await.unwrap();
    f.work.clean_at(&f.step.step_base).await.unwrap();
    assert!(!f.work.git.root().join("src/file.rs").exists());
}

/// A detached HEAD at the expected base still cannot identify the Task branch.
#[tokio::test]
async fn b5_symbolic_head() {
    let f = Fixture::new().await;
    f.change();
    f.work
        .git
        .run(&["update-ref", "--no-deref", "HEAD", &f.step.step_base])
        .await
        .unwrap();
    assert!(f.build().await.is_err());
    assert!(!f.attempt.join("diff.patch").exists());
}

/// A secret added to source never reaches published diff evidence or a ref.
#[tokio::test]
async fn b5_candidate_secret() {
    let f = Fixture::new().await;
    file(
        f.work.git.root(),
        "src/file.rs",
        b"EXAMPLE_SECRET=fixture\n",
    );
    assert!(
        f.build()
            .await
            .err()
            .unwrap()
            .chain()
            .any(|error| error.to_string() == "secret_detected")
    );
    assert!(!f.attempt.join("diff.patch").exists());
    assert_eq!(f.work.ref_value().await.unwrap(), f.step.step_base);
}

/// A committed step resumes after ref-CAS using the recorded SHA and tree.
#[tokio::test]
async fn b5_recorded_ref_resume() {
    let mut f = Fixture::new().await;
    f.change();
    let candidate = f.build().await.unwrap();
    f.step.state = "committed".into();
    f.step.result_commit = Some(candidate.commit.clone());
    f.step.result_tree = Some(candidate.tree);
    f.work
        .git
        .run(&[
            "update-ref",
            &f.work.branch,
            &candidate.commit,
            &f.step.step_base,
        ])
        .await
        .unwrap();
    AdoptionChecker::finish(&f.work, &f.step).await.unwrap();
    f.work.clean_at(&candidate.commit).await.unwrap();
}
