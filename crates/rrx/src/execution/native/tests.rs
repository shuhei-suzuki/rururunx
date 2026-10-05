use super::*;
use crate::adapter::InputKind;
use std::{os::unix::fs::PermissionsExt, path::Path};

fn program(root: &Path, provider: &str) -> std::path::PathBuf {
    let path = root.join(format!("fixture-{provider}"));
    let script = format!("#!/usr/bin/python3\nPROVIDER = {provider:?}\n");
    std::fs::write(
        &path,
        format!("{script}{}", include_str!("native_fixture.py")),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}
fn input(unit: &ExecutionUnit, payload: &str) -> ManagedInput {
    ManagedInput {
        authority: unit.authority(),
        artifact: unit.artifact_id,
        input: PreparedInput {
            scope: unit.scope.clone(),
            kind: InputKind::ContextPack,
            revision: unit.base_sha.clone(),
            version: 1,
            source_versions: BTreeMap::new(),
            payload: payload.into(),
        },
    }
}
async fn terminal(sessions: &NativeSessions, handle: &ManagedSessionRef) -> NativeStatus {
    let mut updates = sessions.subscribe(handle).unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let status = updates.borrow().clone();
            if status.work.is_some() {
                break status;
            }
            updates.changed().await.unwrap();
        }
    })
    .await
    .unwrap()
}
#[tokio::test]
async fn four_protocol_fixture_sessions_keep_sibling_work_when_one_is_cancelled() {
    let (dir, owner, task) = results::tests::fixture().await;
    let sessions = NativeSessions::new(owner.clone()).unwrap();
    let attempts = attempts::AttemptManager::new(owner.clone());
    let mut handles = Vec::new();
    let mut units = Vec::new();
    for (index, provider) in ["claude", "codex", "claude", "codex"]
        .into_iter()
        .enumerate()
    {
        let task = if index == 0 {
            task.clone()
        } else {
            let mut next = crate::domain::Task::new(
                task.project_id,
                task.goal_id,
                format!("fixture-{index}"),
                provider.into(),
            );
            owner.store.lock().unwrap().put_task(&mut next).unwrap();
            next
        };
        let (unit, _) = attempts
            .prepare(task.id, provider, "Implement", None)
            .await
            .unwrap();
        let payload = if index == 0 { "cancel-me" } else { "complete" };
        let NativeStart::Launched(handle) = sessions
            .start_inner(
                input(&unit, payload),
                None,
                None,
                Some(program(dir.path(), provider)),
            )
            .await
            .unwrap()
        else {
            panic!("fixture unexpectedly queued")
        };
        handles.push(handle);
        units.push(unit);
    }
    tokio::time::timeout(Duration::from_secs(10), async {
        while units.iter().any(|u| {
            !owner
                .root
                .join("units")
                .join(u.id.to_string())
                .join("output/fixture-ready")
                .exists()
        }) {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    for handle in &handles {
        assert_eq!(
            sessions.status(handle).unwrap().session.state,
            SessionState::Running
        );
        assert_eq!(sessions.status(handle).unwrap().work, None);
    }
    let foreign = ManagedSessionRef {
        unit: handles[1].unit,
        ..handles[0].clone()
    };
    assert!(sessions.status(&foreign).is_err());
    sessions.cancel(&handles[0]).await.unwrap();
    for unit in units.iter().skip(1) {
        std::fs::write(
            owner
                .root
                .join("units")
                .join(unit.id.to_string())
                .join("output/fixture-release"),
            "release\n",
        )
        .unwrap();
    }
    let stopped = terminal(&sessions, &handles[0]).await;
    assert_eq!(stopped.disposition, Disposition::Cancelled);
    assert_eq!(stopped.work, Some(WorkOutcome::Unknown));
    let results = results::ResultStore::new(owner.clone());
    for (unit, handle) in units.iter().zip(&handles).skip(1) {
        let status = terminal(&sessions, handle).await;
        assert_eq!(status.work, Some(WorkOutcome::Success));
        assert_eq!(status.cleanup, CleanupOutcome::Unknown); // Fixture completion is not descendant-death evidence.
        let current = owner.store.lock().unwrap().execution_unit(unit.id).unwrap();
        let sha = results::text(
            &results::git(&unit.worktree, ["rev-parse", "HEAD"])
                .await
                .unwrap(),
        )
        .unwrap();
        let artifact = results
            .capture(&current.authority(), &sha, BTreeMap::new())
            .await
            .unwrap();
        let version = owner
            .store
            .lock()
            .unwrap()
            .task(unit.scope.task_id.unwrap())
            .unwrap()
            .unwrap()
            .version;
        let current = owner.store.lock().unwrap().execution_unit(unit.id).unwrap();
        let published = results
            .publish(&current.authority(), &artifact, version)
            .await
            .unwrap();
        results.verify(&published).await.unwrap();
    }
    assert!(!units[0].worktree.join("fixture-result.txt").exists());
}
