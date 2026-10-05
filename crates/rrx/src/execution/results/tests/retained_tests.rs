use super::*;
use crate::execution::retained_io::RetainedGit;

async fn captured(owner: &Arc<RuntimeOwner>, task: &Task) -> (ExecutionUnit, ResultArtifact) {
    let attempts = crate::execution::attempts::AttemptManager::new(owner.clone());
    let (unit, _) = attempts
        .prepare(task.id, "codex", "Implement", None)
        .await
        .unwrap();
    let unit = owner
        .store
        .lock()
        .unwrap()
        .finish_execution(
            &unit.authority(),
            WorkOutcome::Success,
            Disposition::Completed,
        )
        .unwrap();
    let artifact = ResultStore::new(owner.clone())
        .capture(&unit.authority(), &unit.base_sha, BTreeMap::new())
        .await
        .unwrap();
    (unit, artifact)
}
fn inspections(owner: &RuntimeOwner, unit: UnitId) -> Vec<ManagedEffect> {
    owner
        .store
        .lock()
        .unwrap()
        .managed_effects(unit)
        .unwrap()
        .into_iter()
        .filter(|e| e.kind == "retained_git")
        .collect()
}
fn preserved(owner: &RuntimeOwner, task: &Task) -> serde_json::Value {
    let store = owner.store.lock().unwrap();
    json!({"task":store.task(task.id).unwrap(),
        "units":store.execution_units(Some(&task.scope())).unwrap(),
        "artifacts":store.result_artifacts(&task.scope()).unwrap()})
}

#[tokio::test]
async fn retained_inspection_survives_retirement_disposal_and_runtime_restart() {
    let (dir, owner, task) = fixture().await;
    let (unit, artifact) = captured(&owner, &task).await;
    let results = ResultStore::new(owner.clone());
    let task_version = owner
        .store
        .lock()
        .unwrap()
        .task(task.id)
        .unwrap()
        .unwrap()
        .version;
    let artifact = results
        .publish(&unit.authority(), &artifact, task_version)
        .await
        .unwrap();
    let count = inspections(&owner, unit.id).len();
    assert_eq!(count, 5); // Actual public publish -> verify registers all five commands.
    let attempts = crate::execution::attempts::AttemptManager::new(owner.clone());
    let old = owner.store.lock().unwrap().execution_unit(unit.id).unwrap();
    attempts.retire(&old.authority(), false).unwrap();
    let (next, _) = attempts
        .prepare(task.id, "codex", "Implement", None)
        .await
        .unwrap();
    assert_ne!(next.id, unit.id);
    // Retained verification must not require any historical profile/temp/output paths.
    std::fs::remove_dir_all(owner.root.join("units").join(unit.id.to_string())).unwrap();
    let before = preserved(&owner, &task);
    results.verify(&artifact).await.unwrap();
    assert_eq!(before, preserved(&owner, &task));
    let effects = inspections(&owner, unit.id);
    assert_eq!(effects.len(), count + 5);
    for effect in &effects[count..] {
        assert_eq!(effect.state, EffectState::Confirmed);
        assert!(effect.expected_target.starts_with(&format!(
            "artifact:{}:{}:epoch:{}:{}:",
            artifact.id, artifact.version, owner.epoch, artifact.manifest_sha256
        )));
    }
    let old_epoch = owner.epoch;
    drop(results);
    drop(attempts);
    drop(owner);
    let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
    assert!(owner.epoch > old_epoch);
    let before = preserved(&owner, &task);
    ResultStore::new(owner.clone())
        .verify(&artifact)
        .await
        .unwrap();
    assert_eq!(before, preserved(&owner, &task));
    let effects = inspections(&owner, unit.id);
    assert_eq!(effects.len(), count + 10);
    assert!(
        effects
            .last()
            .unwrap()
            .expected_target
            .contains(&format!(":epoch:{}:", owner.epoch))
    );
}

#[tokio::test]
async fn retained_inspection_rejects_forged_metadata_and_stale_epoch_before_spawn() {
    let (_dir, owner, task) = fixture().await;
    let (unit, artifact) = captured(&owner, &task).await;
    let results = ResultStore::new(owner.clone());
    let mut forged = Vec::new();
    let mut a = artifact.clone();
    a.revision = "f".repeat(40);
    forged.push(a);
    let mut a = artifact.clone();
    a.base_sha = "e".repeat(40);
    forged.push(a);
    let mut a = artifact.clone();
    a.version += 1;
    forged.push(a);
    let mut a = artifact.clone();
    a.state = ArtifactState::Published;
    forged.push(a);
    let mut a = artifact.clone();
    a.unit_id = UnitId::new();
    forged.push(a);
    let mut a = artifact.clone();
    a.manifest_sha256 = "a".repeat(64);
    forged.push(a);
    let mut a = artifact.clone();
    a.dependencies.insert("rules".into(), "changed".into());
    forged.push(a);
    let mut a = artifact.clone();
    a.repository = owner.root.join("forged.git");
    forged.push(a);
    for artifact in forged {
        assert!(results.verify(&artifact).await.is_err());
    }
    assert!(inspections(&owner, unit.id).is_empty());
    assert!(
        owner
            .store
            .lock()
            .unwrap()
            .reserve_retained_inspection(owner.epoch + 1, &artifact, OperationId::new(), "fsck")
            .is_err()
    );
    assert!(inspections(&owner, unit.id).is_empty());
    results.verify(&artifact).await.unwrap();
    assert_eq!(inspections(&owner, unit.id).len(), 5);
}

// The owned wrapper checks durable intent from inside the actual child before
// execing real Git. It reads only this fixture's SQLite ledger, never credentials.
fn observer(dir: &Path, database: &Path, marker: &Path, sleep: bool) -> PathBuf {
    let python = crate::execution::resources::resolve_program("python3").unwrap();
    let git = crate::execution::resources::resolve_program("git").unwrap();
    let program = dir.join(format!("git-observer-{}", uuid::Uuid::new_v4()));
    std::fs::write(
        &program,
        format!(
            r#"#!{python}
import json, os, pathlib, sqlite3, sys, time
db = sqlite3.connect(pathlib.Path({database}).as_uri() + '?mode=ro', uri=True)
operation = os.environ['RRX_PROCESS_COOKIE']
body, = db.execute('SELECT body FROM managed_effects WHERE id=?', (operation,)).fetchone()
effect = json.loads(body)
assert effect['kind'] == 'retained_git' and effect['state'] == 'pending' and effect['version'] == 1
assert ':epoch:' in effect['expected_target']
assert 'RRX_UNIT_ID' not in os.environ and 'RRX_RUNTIME_SOCKET' not in os.environ
assert os.environ['GIT_NO_LAZY_FETCH'] == '1' and os.environ['GIT_NO_REPLACE_OBJECTS'] == '1'
unit, = db.execute('SELECT body FROM execution_units WHERE id=?', (effect['unit_id'],)).fetchone()
assert operation != json.loads(unit)['cookie']
pathlib.Path({marker}).write_text(operation)
db.close()
if {sleep}: time.sleep(30)
os.execv({git}, [{git}] + sys.argv[1:])
"#,
            python = python.display(),
            database = serde_json::to_string(database.to_str().unwrap()).unwrap(),
            marker = serde_json::to_string(marker.to_str().unwrap()).unwrap(),
            git = serde_json::to_string(git.to_str().unwrap()).unwrap(),
            sleep = if sleep { "True" } else { "False" }
        ),
    )
    .unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    program
}
async fn marker_ready(marker: &Path) -> OperationId {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            if let Ok(text) = std::fs::read_to_string(marker) {
                break text.parse().unwrap();
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn retained_inspection_has_pending_intent_and_distinct_cookie_before_child_exec() {
    let (dir, owner, task) = fixture().await;
    let (unit, artifact) = captured(&owner, &task).await;
    let marker = dir.path().join("observed");
    let program = observer(dir.path(), &dir.path().join("state.db"), &marker, false);
    let io = RetainedGit::new(owner.clone(), &artifact)
        .unwrap()
        .with_program(program);
    ResultStore::new(owner.clone())
        .verify_inner(&artifact, RetainedReader::Historical(&io))
        .await
        .unwrap();
    let last = marker_ready(&marker).await;
    let effects = inspections(&owner, unit.id);
    assert_eq!(effects.len(), 5);
    assert_eq!(effects.last().unwrap().id, last);
    assert!(effects.iter().all(|e| e.state == EffectState::Confirmed));
    assert!(
        io.run(["update-ref", "refs/rrx/unsafe", &artifact.revision])
            .await
            .is_err()
    );
    assert_eq!(inspections(&owner, unit.id).len(), 5);
}

#[tokio::test]
async fn retained_inspection_inflight_artifact_fence_and_abandonment_remain_unknown() {
    let (dir, owner, task) = fixture().await;
    let (_unit, artifact) = captured(&owner, &task).await;
    let marker = dir.path().join("fenced");
    let program = observer(dir.path(), &dir.path().join("state.db"), &marker, true);
    let io = RetainedGit::new(owner.clone(), &artifact)
        .unwrap()
        .with_program(program);
    let worker = tokio::spawn(async move {
        io.run(["fsck", "--full", "--strict", "--no-dangling"])
            .await
    });
    let operation = marker_ready(&marker).await;
    owner
        .store
        .lock()
        .unwrap()
        .invalidate_execution_artifact(artifact.id, "fixture invalidation")
        .unwrap();
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(2), worker)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    let effect = owner
        .store
        .lock()
        .unwrap()
        .managed_effect(operation)
        .unwrap();
    assert_eq!(effect.state, EffectState::Unknown);
    assert!(effect.expected_target.contains(&artifact.id.to_string()));
    // Use another independent owned fixture to exercise actual async wait drop.
    let (dir, owner, task) = fixture().await;
    let (_unit, artifact) = captured(&owner, &task).await;
    let marker = dir.path().join("abandoned");
    let program = observer(dir.path(), &dir.path().join("state.db"), &marker, true);
    let io = RetainedGit::new(owner.clone(), &artifact)
        .unwrap()
        .with_program(program);
    let worker = tokio::spawn(async move {
        io.run(["fsck", "--full", "--strict", "--no-dangling"])
            .await
    });
    let operation = marker_ready(&marker).await;
    worker.abort();
    assert!(worker.await.unwrap_err().is_cancelled());
    assert_eq!(
        owner
            .store
            .lock()
            .unwrap()
            .managed_effect(operation)
            .unwrap()
            .state,
        EffectState::Unknown
    );
}

#[tokio::test]
async fn retained_inspection_detects_reference_graph_and_manifest_corruption() {
    let (_dir, owner, task) = fixture().await;
    let (_unit, artifact) = captured(&owner, &task).await;
    let results = ResultStore::new(owner.clone());
    results.verify(&artifact).await.unwrap();
    let reference = format!("refs/rrx/{}/commit", artifact.id);
    git(&artifact.repository, ["update-ref", "-d", &reference])
        .await
        .unwrap();
    assert!(results.verify(&artifact).await.is_err());
    git(
        &artifact.repository,
        ["update-ref", &reference, &artifact.revision],
    )
    .await
    .unwrap();
    let blob = text(
        &git(
            &artifact.repository,
            ["rev-parse", &format!("{}:answer.txt", artifact.revision)],
        )
        .await
        .unwrap(),
    )
    .unwrap();
    let object = artifact
        .repository
        .join("objects")
        .join(&blob[..2])
        .join(&blob[2..]);
    let bytes = std::fs::read(&object).unwrap();
    std::fs::remove_file(&object).unwrap();
    assert!(results.verify(&artifact).await.is_err());
    std::fs::write(&object, bytes).unwrap();
    results.verify(&artifact).await.unwrap();
    std::fs::write(&artifact.manifest, vec![b'x'; 128 * 1024 + 1]).unwrap();
    let count = inspections(&owner, artifact.unit_id).len();
    assert!(results.verify(&artifact).await.is_err());
    assert_eq!(inspections(&owner, artifact.unit_id).len(), count);
}
