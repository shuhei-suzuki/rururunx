//! Actual standalone Native6 producers over migrated pre-acceptance history.
//! No fixture seeds a Unit, Session, invocation, phase actor, Driver or grant.
use super::*;
use crate::{
    domain::{Goal, Project, Task},
    state::{APPLICATION_ID, Store},
};
use rusqlite::{Connection, functions::FunctionFlags, params};
use std::sync::mpsc as std_mpsc;

async fn legacy_fixture() -> (tempfile::TempDir, Arc<RuntimeOwner>, Task) {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("repo");
    std::fs::create_dir(&source).unwrap();
    results::git(&source, ["init", "-b", "main"]).await.unwrap();
    std::fs::write(source.join("answer.txt"), "base\n").unwrap();
    results::git(&source, ["add", "answer.txt"]).await.unwrap();
    results::git(
        &source,
        [
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            "fixture",
        ],
    )
    .await
    .unwrap();
    let source = source.canonicalize().unwrap();
    let mut project = Project::new(
        "legacy".into(),
        source.clone(),
        crate::git::repository_identity(&source, "main").unwrap(),
        "main".into(),
    );
    project.version = 1;
    let mut goal = Goal::new(project.id, "pre-acceptance historical Goal".into(), vec![]);
    goal.version = 1;
    let mut task = Task::new(project.id, goal.id, "legacy task".into(), "codex".into());
    task.version = 1;
    // This is the same old9 factual layout used by the Binding migration corpus.
    // No trigger/function can supply current private authority.
    let db = dir.path().join("state.db");
    let c = Connection::open(&db).unwrap();
    for sql in [
        include_str!("../../../state/schema.sql"),
        include_str!("../../../state/execution.sql"),
        include_str!("../../../state/execution/native_results.sql"),
        include_str!("../../../state/execution/source_recovery.sql"),
        include_str!("../../../state/execution/verification.sql"),
        include_str!("../../../state/runtime/schema.sql"),
    ] {
        c.execute_batch(sql).unwrap();
    }
    c.create_scalar_function(
        "rrx_writer_contract_version",
        0,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_INNOCUOUS,
        |_| Ok(9_i64),
    )
    .unwrap();
    let tables = {
        let mut q=c.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT GLOB 'sqlite_*' ORDER BY name").unwrap();
        q.query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    };
    for table in tables {
        for action in ["INSERT", "UPDATE", "DELETE"] {
            c.execute_batch(&format!("CREATE TRIGGER writer_{table}_{action} BEFORE {action} ON {table} WHEN rrx_writer_contract_version()<>9 BEGIN SELECT RAISE(ABORT,'incompatible rrx writer contract'); END;")).unwrap();
        }
    }
    c.execute(
        "INSERT INTO runtime_epoch(singleton,instance_id,epoch) VALUES(1,?1,0)",
        [uuid::Uuid::new_v4().to_string()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO projects(id,root,version,body) VALUES(?1,?2,1,?3)",
        params![
            project.id.to_string(),
            project.root.to_str(),
            serde_json::to_string(&project).unwrap()
        ],
    )
    .unwrap();
    c.execute(
        "INSERT INTO goals(id,project_id,version,body) VALUES(?1,?2,1,?3)",
        params![
            goal.id.to_string(),
            project.id.to_string(),
            serde_json::to_string(&goal).unwrap()
        ],
    )
    .unwrap();
    c.execute(
        "INSERT INTO tasks(id,project_id,goal_id,version,body) VALUES(?1,?2,?3,1,?4)",
        params![
            task.id.to_string(),
            project.id.to_string(),
            goal.id.to_string(),
            serde_json::to_string(&task).unwrap()
        ],
    )
    .unwrap();
    c.pragma_update(None, "application_id", APPLICATION_ID)
        .unwrap();
    c.pragma_update(None, "user_version", 9).unwrap();
    drop(c);
    assert_eq!(Store::open(&db).unwrap().schema_version().unwrap(), 10);
    let owner = RuntimeOwner::open(&db).unwrap();
    {
        let store = owner.store.lock().unwrap();
        assert!(!store.managed_phase_required(&task.scope()).unwrap());
        assert!(
            store
                .execution_units(Some(&task.scope()))
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .records(&task.scope(), crate::domain::RecordKind::Session)
                .unwrap()
                .is_empty()
        );
    }
    (dir, owner, task)
}

async fn held_success() -> (
    tempfile::TempDir,
    Arc<RuntimeOwner>,
    Arc<NativeSessions>,
    ExecutionUnit,
    ManagedSessionRef,
    Connection,
    std::path::PathBuf,
) {
    let (dir, owner, task) = legacy_fixture().await;
    let sessions = Arc::new(NativeSessions::new(owner.clone()).unwrap());
    let (unit, _) = attempts::AttemptManager::new(owner.clone())
        .prepare(task.id, "codex", "Implement", None)
        .await
        .unwrap();
    let NativeStart::Launched(handle) = sessions
        .start_inner(
            input(&unit, "answer-hold-after-final"),
            None,
            None,
            Some(program(dir.path(), "codex")),
        )
        .await
        .unwrap()
    else {
        panic!("actual legacy peer did not launch")
    };
    tokio::time::timeout(Duration::from_secs(5), async {
        while !sessions
            .status(&handle)
            .unwrap()
            .pending
            .iter()
            .any(|p| p["id"] == "answer-barrier")
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let connection = Connection::open(dir.path().join("state.db")).unwrap();
    connection.execute_batch(&format!("CREATE TRIGGER native_receipt_fixture_fault BEFORE INSERT ON native_results WHEN NEW.unit_id='{}' BEGIN SELECT RAISE(ABORT,'synthetic bounded receipt failure'); END;",unit.id)).unwrap();
    let output = owner
        .root
        .join("units")
        .join(unit.id.to_string())
        .join("output");
    std::fs::write(output.join("fixture-delay-publication"), "delay").unwrap();
    std::fs::write(output.join("fixture-release"), "release").unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while !output.join("fixture-publication-ready").exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    (dir, owner, sessions, unit, handle, connection, output)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn legacy_native_status_releases_saved_terminal_mutex_before_store_and_core_drop() {
    let (_dir, owner, sessions, unit, handle, connection, output) = held_success().await;
    let pending = sessions
        .entries
        .lock()
        .unwrap()
        .get(&handle.session)
        .unwrap()
        .terminal
        .clone();
    let actual = pending.lock().unwrap().as_ref().unwrap().clone();
    assert_eq!(actual.receipt.observed_work, WorkOutcome::Success);
    let (store_ready_tx, store_ready_rx) = std_mpsc::channel();
    let (store_release_tx, store_release_rx) = std_mpsc::channel();
    let store_owner = owner.clone();
    let holder = std::thread::spawn(move || {
        let _store = store_owner.store.lock().unwrap();
        store_ready_tx.send(()).unwrap();
        store_release_rx
            .recv_timeout(Duration::from_secs(5))
            .unwrap();
    });
    store_ready_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let (status_enter_tx, status_enter_rx) = std_mpsc::channel();
    let (status_done_tx, status_done_rx) = std_mpsc::channel();
    let reader_sessions = sessions.clone();
    let reader_handle = handle.clone();
    let reader = {
        let _frozen = pending.lock().unwrap();
        let reader = std::thread::spawn(move || {
            status_enter_tx.send(()).unwrap();
            let status = reader_sessions.status(&reader_handle);
            status_done_tx.send(status).unwrap();
        });
        status_enter_rx
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        std::thread::sleep(Duration::from_millis(20));
        reader
    };
    // The actual status call is blocked by the real Store mutex. Its original
    // actor-issued Arc must stay available to actual Core completion/Drop.
    std::thread::sleep(Duration::from_millis(50));
    assert!(matches!(
        status_done_rx.try_recv(),
        Err(std_mpsc::TryRecvError::Empty)
    ));
    let mut unlocked_samples = 0;
    for _ in 0..20 {
        if let Ok(frozen) = pending.try_lock() {
            assert!(Arc::ptr_eq(frozen.as_ref().unwrap(), &actual));
            unlocked_samples += 1;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    // Release real locks before asserting the causal control, including mutants.
    store_release_tx.send(()).unwrap();
    holder.join().unwrap();
    let held = status_done_rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
        .unwrap();
    reader.join().unwrap();
    assert_eq!(
        unlocked_samples, 20,
        "actual status retained frozen mutex across Store wait"
    );
    assert_eq!(held.observed_work, Some(WorkOutcome::Success));
    assert_eq!(held.work, None);
    assert!(held.receipt.is_none());
    connection
        .execute_batch("DROP TRIGGER native_receipt_fixture_fault")
        .unwrap();
    let restored = sessions.status(&handle).unwrap();
    assert_eq!(restored.work, Some(WorkOutcome::Success));
    assert_eq!(restored.session.state, SessionState::Exited);
    let receipt = owner
        .store
        .lock()
        .unwrap()
        .native_result(restored.receipt.unwrap())
        .unwrap();
    assert_eq!(receipt.text.as_deref(), Some("APPROVE actual answer"));
    assert!(pending.lock().unwrap().is_none());
    let mut updates = sessions.subscribe(&handle).unwrap();
    updates.borrow_and_update();
    std::fs::write(output.join("fixture-publication-release"), "release").unwrap();
    // Actual Core return and legacy Drop preserve the same committed proof after
    // status compare-cleared it; no fresh Unknown is fabricated from missing rows.
    tokio::time::timeout(Duration::from_secs(5), updates.changed())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updates.borrow().work, Some(WorkOutcome::Success));
    assert_eq!(updates.borrow().receipt, restored.receipt);
    assert_eq!(updates.borrow().session.state, SessionState::Exited);
    assert_eq!(
        owner
            .store
            .lock()
            .unwrap()
            .execution_unit(unit.id)
            .unwrap()
            .work,
        Some(WorkOutcome::Success)
    );
    sessions.release(&handle).unwrap();
}
