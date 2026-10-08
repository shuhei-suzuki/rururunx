//! EF controls (EF HOW R2 §4): the epoch fence of a genuinely bound managed
//! Native Session. Every state comes from accepted ingress, the installed
//! issuer and the protocol fixture; nothing is SQL-seeded and no authority is
//! constructed. Raw reads only observe.
use super::*;
use crate::domain::{Session, SessionState};
use std::collections::BTreeMap;

const LINKS: &str = "SELECT kind FROM audit WHERE kind LIKE 'rrx.private.workflow.%' AND task_id=?1 ORDER BY sequence";

async fn wait_for(mut condition: impl FnMut() -> bool, label: &str, seconds: u64) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(seconds);
    while !condition() {
        assert!(tokio::time::Instant::now() < deadline, "{label}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
fn links(conn: &rusqlite::Connection, task: &Task) -> Vec<String> {
    let mut statement = conn.prepare(LINKS).unwrap();
    statement
        .query_map([task.id.to_string()], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}
/// Every row of every table, by table, in rowid order.
fn dump(conn: &rusqlite::Connection) -> BTreeMap<String, Vec<String>> {
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    tables
        .into_iter()
        .map(|table| {
            let mut statement = conn.prepare(&format!("SELECT * FROM \"{table}\"")).unwrap();
            let columns = statement.column_count();
            let mut rows: Vec<String> = statement
                .query_map([], |r| {
                    (0..columns)
                        .map(|i| {
                            r.get::<_, rusqlite::types::Value>(i)
                                .map(|v| format!("{v:?}"))
                        })
                        .collect::<rusqlite::Result<Vec<_>>>()
                        .map(|v| v.join("|"))
                })
                .unwrap()
                .map(Result::unwrap)
                .collect();
            rows.sort();
            (table, rows)
        })
        .collect()
}
fn ids(conn: &rusqlite::Connection, sql: &str) -> Vec<String> {
    let mut statement = conn.prepare(sql).unwrap();
    statement
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}
fn rows(conn: &rusqlite::Connection, sql: &str, key: &str) -> Vec<String> {
    let mut statement = conn.prepare(sql).unwrap();
    let columns = statement.column_count();
    statement
        .query_map([key], |r| {
            (0..columns)
                .map(|i| {
                    r.get::<_, rusqlite::types::Value>(i)
                        .map(|v| format!("{v:?}"))
                })
                .collect::<rusqlite::Result<Vec<_>>>()
                .map(|v| v.join("|"))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}
fn session_id(conn: &rusqlite::Connection, task: &Task) -> String {
    conn.query_row(
        "SELECT allocated_session_id FROM managed_phase_owners WHERE task_id=?1",
        [task.id.to_string()],
        |r| r.get(0),
    )
    .unwrap()
}
fn session(conn: &rusqlite::Connection, id: &str) -> (crate::domain::Record, Session) {
    let body: String = conn
        .query_row("SELECT body FROM records WHERE id=?1", [id], |r| r.get(0))
        .unwrap();
    let record: crate::domain::Record = serde_json::from_str(&body).unwrap();
    let session: Session = serde_json::from_value(record.data.clone()).unwrap();
    (record, session)
}
fn unit_id(conn: &rusqlite::Connection, session: &str) -> String {
    conn.query_row(
        "SELECT unit_id FROM session_units WHERE session_id=?1",
        [session],
        |r| r.get(0),
    )
    .unwrap()
}
/// The rows EF must leave byte-identical: operation, marker, Workflow,
/// owner, private links and Task.
fn protected(conn: &rusqlite::Connection, task: &Task) -> Vec<Vec<String>> {
    let t = task.id.to_string();
    vec![
        rows(
            conn,
            "SELECT * FROM managed_phase_operations WHERE task_id=?1",
            &t,
        ),
        rows(
            conn,
            "SELECT m.* FROM managed_marker_bodies m JOIN managed_phase_operations o ON o.operation_id=m.operation_id WHERE o.task_id=?1",
            &t,
        ),
        rows(
            conn,
            "SELECT * FROM records WHERE kind='workflow' AND task_id=?1",
            &t,
        ),
        rows(
            conn,
            "SELECT * FROM managed_phase_owners WHERE task_id=?1",
            &t,
        ),
        rows(
            conn,
            "SELECT * FROM audit WHERE kind LIKE 'rrx.private.workflow.%' AND task_id=?1",
            &t,
        ),
        rows(conn, "SELECT * FROM tasks WHERE id=?1", &t),
    ]
}
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// A genuinely bound managed Session: accepted Goal, installed issuer,
/// normal Bound, then shutdown and release of the old owner custody.
async fn bound_and_stopped(
    provider: &str,
) -> (
    tempfile::TempDir,
    std::path::PathBuf,
    crate::config::Config,
    Task,
) {
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = tasks[0].clone();
    f.runtime.start().await.unwrap();
    {
        let conn = raw(&f);
        wait_for(
            || {
                links(&conn, &task).first().map(String::as_str)
                    == Some("rrx.private.workflow.session_bound")
            },
            "SETUP: normal Bound absent",
            60,
        )
        .await;
    }
    let config = f.runtime.config.clone();
    let state = f.owner.state_path().to_path_buf();
    let _ = f.runtime.shutdown().await;
    let ControlFixture {
        _dir,
        owner,
        runtime,
        ..
    } = f;
    let weak = Arc::downgrade(&runtime);
    drop(runtime);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while weak.upgrade().is_some() || Arc::strong_count(&owner) > 1 {
        assert!(
            tokio::time::Instant::now() < deadline,
            "SETUP: the old owner custody did not drop"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    drop(owner);
    (_dir, state, config, task)
}

/// EF1 + EF2 + EF4 (Lost re-fence): the managed Session is fenced `Lost`
/// with its exact postimage, the same epoch fences the Unit, nothing else
/// changes, and nothing is reconstructed.
async fn ef1(provider: &str) {
    let (_dir, state, config, task) = bound_and_stopped(provider).await;
    let conn = crate::state::current_test_writer(&state).unwrap();
    let sid = session_id(&conn, &task);
    let (before_record, before) = session(&conn, &sid);
    assert!(
        matches!(before.state, SessionState::Starting | SessionState::Running),
        "SETUP: EF1 {provider}: open Session, got {:?}",
        before.state
    );
    let unit = unit_id(&conn, &sid);
    let generation: i64 = conn
        .query_row(
            "SELECT generation FROM task_execution WHERE task_id=?1",
            [task.id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    let protected_before = protected(&conn, &task);
    let identity_before = rows(
        &conn,
        "SELECT session_id,project_id,goal_id,task_id,provider,native_ref,malformed FROM scoped_session_identities WHERE session_id=?1",
        &sid,
    );
    let audit_before: i64 = conn
        .query_row("SELECT coalesce(max(sequence),0) FROM audit", [], |r| {
            r.get(0)
        })
        .unwrap();
    let pending_before = ids(
        &conn,
        "SELECT id FROM managed_effects WHERE state='pending'",
    );
    let leases_before = ids(&conn, "SELECT unit_id FROM quota_leases WHERE active=1");
    drop(conn);

    use crate::state::epoch_fence_fault::{Observation, observations};
    let _ = observations();
    let t0 = now_ms();
    let owner = crate::execution::RuntimeOwner::open(&state).unwrap();
    let t1 = now_ms();
    let seen = observations();
    let conn = crate::state::current_test_writer(&state).unwrap();
    let (after_record, after) = session(&conn, &sid);
    // EF1: the Session postimage.
    assert_eq!(after.state, SessionState::Lost, "EF1 {provider}: Lost");
    assert_eq!(
        after_record.version,
        before_record.version + 1,
        "EF1 {provider}: version exactly +1"
    );
    assert!(
        matches!(seen.as_slice(), [Observation { ok: true, permit_active: false, at }] if (t0..=t1).contains(at)),
        "EF1 {provider}: one managed fence, committed, permit revoked on the same manager: {seen:?}"
    );
    assert_eq!(
        after_record.updated_at, seen[0].at,
        "EF1 {provider}: updated_at is exactly the epoch's single fence instant"
    );
    assert!(
        after.id == before.id
            && after.scope == before.scope
            && after.provider == before.provider
            && after.agent == before.agent
            && after.role == before.role
            && after.worktree == before.worktree
            && after.model == before.model
            && after.effort == before.effort
            && after.native_ref == before.native_ref,
        "EF1 {provider}: immutable identity and native_ref unchanged"
    );
    // EF1: projections.
    let dispatch: String = conn
        .query_row(
            "SELECT dispatch_state FROM session_units WHERE session_id=?1",
            [&sid],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(dispatch, "unknown", "EF1 {provider}: dispatch unknown");
    let identity_version: i64 = conn
        .query_row(
            "SELECT record_version FROM scoped_session_identities WHERE session_id=?1",
            [&sid],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        identity_version as u64, after_record.version,
        "EF1 {provider}: identity projection follows its source"
    );
    assert_eq!(
        rows(
            &conn,
            "SELECT session_id,project_id,goal_id,task_id,provider,native_ref,malformed FROM scoped_session_identities WHERE session_id=?1",
            &sid,
        ),
        identity_before,
        "EF1 {provider}: identity source unchanged"
    );
    // EF1: events.
    let saved: i64 = conn
        .query_row(
            "SELECT count(*) FROM audit WHERE sequence>?1 AND kind='session.saved' AND json_extract(data,'$.id')=?2",
            rusqlite::params![audit_before, sid],
            |r| r.get(0),
        )
        .unwrap();
    let lost: i64 = conn
        .query_row(
            "SELECT count(*) FROM audit WHERE sequence>?1 AND kind='execution.session_epoch_lost' AND json_extract(data,'$.session')=?2",
            rusqlite::params![audit_before, sid],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!((saved, lost), (1, 1), "EF1 {provider}: one saved, one lost");
    // EF2: the same epoch fenced the Unit.
    let fenced = owner
        .store
        .lock()
        .unwrap()
        .execution_unit(unit.parse().unwrap())
        .unwrap();
    assert!(
        !fenced.native_effects_open && !fenced.result_finalization_open,
        "EF2 {provider}: Unit authorities closed"
    );
    assert_eq!(
        fenced.work,
        Some(crate::execution::WorkOutcome::Unknown),
        "EF2 {provider}: WorkUnknown"
    );
    let generation_after: i64 = conn
        .query_row(
            "SELECT generation FROM task_execution WHERE task_id=?1",
            [task.id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        generation_after,
        generation + 1,
        "EF2 {provider}: generation advanced"
    );
    let cleanup: i64 = conn
        .query_row(
            "SELECT count(*) FROM cleanup_jobs WHERE unit_id=?1",
            [&unit],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(cleanup, 1, "EF2 {provider}: cleanup job present");
    let pending: i64 = conn
        .query_row(
            "SELECT count(*) FROM managed_effects WHERE state='pending'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let active: i64 = conn
        .query_row(
            "SELECT count(*) FROM quota_leases WHERE active=1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        (pending, active),
        (0, 0),
        "EF2 {provider}: no pending effect and no active quota lease remain"
    );
    // Per id, for the effects/leases that existed before the epoch (may be
    // none in this fixture; the count is reported, not claimed non-empty).
    for id in &pending_before {
        let state: String = conn
            .query_row("SELECT state FROM managed_effects WHERE id=?1", [id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            state,
            "unknown",
            "EF2 {provider}: pre-epoch pending effect {id} (of {}) is Unknown",
            pending_before.len()
        );
    }
    for unit in &leases_before {
        let active: i64 = conn
            .query_row(
                "SELECT active FROM quota_leases WHERE unit_id=?1",
                [unit],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            active,
            0,
            "EF2 {provider}: pre-epoch active lease {unit} (of {}) released",
            leases_before.len()
        );
    }
    assert_eq!(
        protected(&conn, &task),
        protected_before,
        "EF2 {provider}: operation, marker, Workflow, owner, links and Task byte-identical"
    );
    // EF2: the new Runtime reconstructs nothing.
    let runtime = Arc::new(Runtime::new(owner.clone(), config.clone()).unwrap());
    runtime.start().await.unwrap();
    let started = std::time::Instant::now();
    while started.elapsed() < Duration::from_secs(6) {
        runtime.wake.notify_one();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(
        runtime.phase_jobs.observed_jobs().is_empty(),
        "EF2 {provider}: no reconstructed job"
    );
    assert_eq!(
        protected(&conn, &task),
        protected_before,
        "EF2 {provider}: no link, plan or Task change after the new Runtime ran"
    );
    let _ = runtime.shutdown().await;
    let weak = Arc::downgrade(&runtime);
    drop(runtime);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while weak.upgrade().is_some() || Arc::strong_count(&owner) > 1 {
        assert!(
            tokio::time::Instant::now() < deadline,
            "SETUP: second custody did not drop"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    drop(owner);
    // EF4: Lost is not a skip state; a second restart re-fences Lost -> Lost.
    let _ = observations();
    let owner = crate::execution::RuntimeOwner::open(&state).unwrap();
    let seen = observations();
    let (again_record, again) = session(&conn, &sid);
    assert_eq!(
        again.state,
        SessionState::Lost,
        "EF4 {provider}: still Lost"
    );
    assert_eq!(
        again_record.version,
        after_record.version + 1,
        "EF4 {provider}: Lost re-fenced, version +1 under the permit"
    );
    assert!(
        matches!(seen.as_slice(), [Observation { ok: true, permit_active: false, at }] if *at == again_record.updated_at),
        "EF4 {provider}: one re-fence at its epoch instant, permit revoked: {seen:?}"
    );
    drop(owner);
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ef1_claude_managed_session_fenced_lost_exactly() {
    ef1("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ef1_codex_managed_session_fenced_lost_exactly() {
    ef1("codex").await;
}

/// EF3: with the Unit fence written earlier in the same epoch transaction,
/// a missing permit or a one-byte-wrong old image makes the open fail and
/// leaves every table unchanged; the generic writer stays refused.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ef3_wrong_or_missing_permit_rolls_back_the_whole_epoch() {
    use crate::state::epoch_fence_fault::{Fault, Observation, arm, observations};
    let (_dir, state, _config, task) = bound_and_stopped("claude").await;
    let conn = crate::state::current_test_writer(&state).unwrap();
    let sid = session_id(&conn, &task);
    let before = dump(&conn);
    for fault in [Fault::NoPermit, Fault::WrongOld] {
        let _ = observations();
        arm(fault);
        let error = match crate::execution::RuntimeOwner::open(&state) {
            Ok(_) => panic!("EF3 {fault:?}: open must fail"),
            Err(error) => error,
        };
        arm(Fault::None);
        // The failed writer's OWN manager, observed before its Store dropped.
        let seen = observations();
        assert!(
            matches!(
                seen.as_slice(),
                [Observation {
                    ok: false,
                    permit_active: false,
                    ..
                }]
            ),
            "EF3 {fault:?}: the failed fence left no permit on its manager: {seen:?}"
        );
        assert_eq!(
            dump(&conn),
            before,
            "EF3 {fault:?}: whole state unchanged after the failed epoch ({error:#})"
        );
    }
    // version+1 with identity and body unchanged passes the identity,
    // native_ref and version triggers; only the private writer guard refuses.
    let refused = conn
        .execute("UPDATE records SET version=version+1 WHERE id=?1", [&sid])
        .expect_err("EF3: a generic UPDATE of the protected Session is still refused");
    assert!(
        refused
            .to_string()
            .contains("private managed Record writer required"),
        "EF3: refused by the permit guard, not another trigger: {refused}"
    );
    assert_eq!(dump(&conn), before, "EF3: refusal left no change");
    // The contrast: no fault, the same state opens and fences.
    let owner = crate::execution::RuntimeOwner::open(&state).unwrap();
    assert_eq!(session(&conn, &sid).1.state, SessionState::Lost);
    assert!(
        matches!(
            observations().as_slice(),
            [Observation {
                ok: true,
                permit_active: false,
                ..
            }]
        ),
        "EF3: contrast fence committed and revoked its permit"
    );
    drop(owner);
}

/// EF4: a closed managed Session (terminal by `session_terminal`) is not
/// touched by the epoch fence.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ef4_terminal_managed_session_is_skipped() {
    let mut f = fixture_mode("claude", true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = tasks[0].clone();
    f.runtime.start().await.unwrap();
    let conn = raw(&f);
    wait_for(
        || {
            links(&conn, &task).first().map(String::as_str)
                == Some("rrx.private.workflow.session_bound")
        },
        "SETUP: normal Bound absent",
        60,
    )
    .await;
    let sid = session_id(&conn, &task);
    let unit = f
        .owner
        .store
        .lock()
        .unwrap()
        .execution_unit(unit_id(&conn, &sid).parse().unwrap())
        .unwrap();
    let profile = crate::execution::resources::ResourceManager::new(f.owner.clone())
        .profile(&unit)
        .unwrap();
    std::fs::write(profile.output.join("fixture-bootstrap-release"), "release").unwrap();
    std::fs::write(profile.output.join("fixture-release"), "release").unwrap();
    wait_for(
        || {
            links(&conn, &task).last().map(String::as_str)
                == Some("rrx.private.workflow.phase_closed")
        },
        "SETUP: success phase_closed absent",
        90,
    )
    .await;
    let (closed_record, closed) = session(&conn, &sid);
    assert!(
        matches!(
            closed.state,
            SessionState::Exited | SessionState::Stopped | SessionState::Failed
        ),
        "SETUP: EF4 terminal Session, got {:?}",
        closed.state
    );
    let state = f.owner.state_path().to_path_buf();
    let _ = f.runtime.shutdown().await;
    let ControlFixture {
        _dir,
        owner,
        runtime,
        ..
    } = f;
    let weak = Arc::downgrade(&runtime);
    drop(runtime);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while weak.upgrade().is_some() || Arc::strong_count(&owner) > 1 {
        assert!(
            tokio::time::Instant::now() < deadline,
            "SETUP: custody did not drop"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    drop(owner);
    let owner = crate::execution::RuntimeOwner::open(&state).unwrap();
    let (record, after) = session(&conn, &sid);
    assert_eq!(
        (record.version, after.state),
        (closed_record.version, closed.state),
        "EF4: terminal managed Session untouched"
    );
    drop(owner);
    drop(_dir);
}
