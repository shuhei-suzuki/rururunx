use super::*;

struct Fixture {
    _dir: tempfile::TempDir,
    git: PipelineGit,
    base: String,
    config: PathBuf,
}

impl Fixture {
    async fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let project = root.join("project");
        std::fs::create_dir(&project).unwrap();
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
        let task = root.join("task");
        git.run(&[
            "worktree",
            "add",
            "-b",
            "task",
            task.to_str().unwrap(),
            &base,
        ])
        .await
        .unwrap();
        let git = PipelineGit::new(&task).unwrap();
        let git_dir = git
            .text(&["rev-parse", "--absolute-git-dir"])
            .await
            .unwrap();
        Self {
            _dir: dir,
            git,
            base,
            config: PathBuf::from(git_dir).join("config.worktree"),
        }
    }
}

/// A linked worktree without the extension ignores worktree config but still rejects local drivers.
#[tokio::test]
async fn b5_config_linked_without_extension() {
    let f = Fixture::new().await;
    // Git does not read this file without the extension, even if it is malformed.
    std::fs::write(&f.config, b"[invalid\n").unwrap();
    f.git.reject_drivers(&f.base).await.unwrap();
    f.git
        .run(&["config", "--local", "filter.fixture.clean", "false"])
        .await
        .unwrap();
    assert_eq!(
        f.git.reject_drivers(&f.base).await.unwrap_err().to_string(),
        "git_driver_declared"
    );
}

/// An enabled worktree scope rejects drivers declared only through its included config.
#[tokio::test]
async fn b5_config_linked_with_extension() {
    let f = Fixture::new().await;
    // Exercise Git's boolean normalization, not a literal string comparison of config text.
    f.git
        .run(&["config", "--local", "extensions.worktreeConfig", "on"])
        .await
        .unwrap();
    f.git.reject_drivers(&f.base).await.unwrap();
    let included = f.config.with_file_name("driver.inc");
    std::fs::write(&included, b"[filter \"fixture\"]\nclean = false\n").unwrap();
    std::fs::write(&f.config, b"[include]\npath = driver.inc\n").unwrap();
    let local = f
        .git
        .run(&["config", "--includes", "--list", "-z", "--local"])
        .await
        .unwrap();
    assert!(
        !local
            .windows(b"filter.fixture".len())
            .any(|s| s == b"filter.fixture")
    );
    assert_eq!(
        f.git.reject_drivers(&f.base).await.unwrap_err().to_string(),
        "git_driver_declared"
    );
}
