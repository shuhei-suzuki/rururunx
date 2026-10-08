use super::*;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn retained_inspection_ambient_common_directory_cannot_substitute_complete_backup() {
    let (dir, owner, task) = fixture().await;
    let (_unit, artifact) = captured(&owner, &task).await;
    let backup = dir.path().join("complete-backup.git");
    git(
        dir.path(),
        [
            "clone",
            "--mirror",
            "--no-local",
            "--no-hardlinks",
            "--",
            artifact.repository.to_str().unwrap(),
            backup.to_str().unwrap(),
        ],
    )
    .await
    .unwrap();
    // Original refs/manifest remain correct, but all original graph objects are missing.
    let objects = artifact.repository.join("objects");
    std::fs::rename(&objects, dir.path().join("missing-objects")).unwrap();
    std::fs::create_dir(&objects).unwrap();
    assert!(
        ResultStore::new(owner.clone())
            .verify(&artifact)
            .await
            .is_err()
    );
    let metadata = dir.path().join("artifact.json");
    std::fs::write(&metadata, serde_json::to_vec(&artifact).unwrap()).unwrap();
    drop(owner);
    let mut command = tokio::process::Command::new(std::env::current_exe().unwrap());
    command.args(["--exact",
        "execution::results::tests::retained_tests::routing_tests::retained_inspection_routing_worker",
        "--ignored", "--nocapture"])
        .env("RRX_RETAINED_TEST_STATE", dir.path().join("state.db"))
        .env("RRX_RETAINED_TEST_ARTIFACT", &metadata)
        .env("GIT_COMMON_DIR", &backup);
    let observed = process::capture_observed(&mut command).await.unwrap();
    assert!(
        observed.receipt.status.success(),
        "actual public verification accepted incomplete original storage through ambient common directory"
    );
}

#[tokio::test]
#[ignore = "owned routing parent passes fixture paths and environment"]
async fn retained_inspection_routing_worker() {
    let state = std::env::var_os("RRX_RETAINED_TEST_STATE").expect("owned fixture state required");
    let metadata =
        std::env::var_os("RRX_RETAINED_TEST_ARTIFACT").expect("owned fixture artifact required");
    let artifact: ResultArtifact =
        serde_json::from_slice(&std::fs::read(metadata).unwrap()).unwrap();
    let owner = RuntimeOwner::open(Path::new(&state)).unwrap();
    assert!(
        ResultStore::new(owner).verify(&artifact).await.is_err(),
        "public retained verification accepted missing original graph"
    );
}

async fn ancestor_fixture() -> (
    crate::runtime::LegacyFixture,
    Arc<RuntimeOwner>,
    Task,
    ResultArtifact,
    Vec<String>,
) {
    let (dir, owner, task) = fixture().await;
    let root = dir.path().join("repo");
    let first = text(&git(&root, ["rev-parse", "HEAD"]).await.unwrap()).unwrap();
    std::fs::write(root.join("answer.txt"), "middle\n").unwrap();
    let middle = commit(&root).await;
    std::fs::write(root.join("answer.txt"), "tip\n").unwrap();
    commit(&root).await;
    let (_unit, artifact) = captured(&owner, &task).await;
    (dir, owner, task, artifact, vec![first, middle])
}

async fn remove_ancestors(repository: &Path, backup: &Path, ancestors: &[String]) {
    // Unpack the actual fixture's fetched packs into a fresh owned object store;
    // this makes selective ancestor deletion independent of fetch's storage choice.
    let objects = repository.join("objects");
    let packs = std::fs::read_dir(objects.join("pack"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "pack"))
        .map(|p| std::fs::read(p).unwrap())
        .collect::<Vec<_>>();
    assert!(
        !packs.is_empty(),
        "owned fixture fetch should have produced a pack"
    );
    std::fs::rename(&objects, backup).unwrap();
    std::fs::create_dir(&objects).unwrap();
    for bytes in packs {
        let mut command = git_command(repository).unwrap();
        command
            .args(["unpack-objects", "-r"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let mut child = process::OwnedProcess::spawn(&mut command).unwrap();
        let mut stdin = child.child.stdin.take().unwrap();
        let write = async {
            stdin.write_all(&bytes).await?;
            stdin.shutdown().await
        };
        let (_, observed) = tokio::try_join!(
            async { Ok::<_, anyhow::Error>(write.await?) },
            process::capture_child(child)
        )
        .unwrap();
        assert!(observed.receipt.status.success());
    }
    for sha in ancestors {
        std::fs::remove_file(objects.join(&sha[..2]).join(&sha[2..])).unwrap();
    }
}

#[tokio::test]
async fn retained_inspection_shallow_boundary_cannot_hide_missing_ancestors() {
    let (dir, owner, _task, artifact, ancestors) = ancestor_fixture().await;
    let results = ResultStore::new(owner.clone());
    results.verify(&artifact).await.unwrap();
    remove_ancestors(
        &artifact.repository,
        &dir.path().join("complete-objects"),
        &ancestors,
    )
    .await;
    assert!(results.verify(&artifact).await.is_err());
    std::fs::write(
        artifact.repository.join("shallow"),
        format!("{}\n", artifact.revision),
    )
    .unwrap();
    assert!(
        results.verify(&artifact).await.is_err(),
        "shallow boundary hid missing required ancestors from actual public verify"
    );
    std::fs::remove_file(artifact.repository.join("shallow")).unwrap();
    std::fs::remove_dir_all(artifact.repository.join("objects")).unwrap();
    std::fs::rename(
        dir.path().join("complete-objects"),
        artifact.repository.join("objects"),
    )
    .unwrap();
    results.verify(&artifact).await.unwrap();
}

#[tokio::test]
async fn retained_inspection_graft_cannot_hide_missing_ancestors() {
    let (dir, owner, _task, artifact, ancestors) = ancestor_fixture().await;
    let results = ResultStore::new(owner.clone());
    remove_ancestors(
        &artifact.repository,
        &dir.path().join("complete-objects"),
        &ancestors,
    )
    .await;
    assert!(results.verify(&artifact).await.is_err());
    std::fs::write(
        artifact.repository.join("info/grafts"),
        format!("{}\n", artifact.revision),
    )
    .unwrap();
    assert!(
        results.verify(&artifact).await.is_err(),
        "graft metadata must not truncate retained ancestry"
    );
}
