use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca5d_integrity_decision_uses_genuine_allocation_and_negative_raw_members() {
    let mut f = fixture("claude", true);
    f.register_real_git_project();
    let (_, tasks) = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    let (record, workflow) = wait_bound(&f, &tasks[0]).await;
    let reference = workflow
        .history
        .iter()
        .find_map(|a| a.execution.as_ref())
        .unwrap();
    let allocation = f
        .runtime
        .phase_jobs
        .original_allocation(reference.unit)
        .unwrap();
    assert_eq!(allocation.facts().scope, &tasks[0].scope());
    assert_eq!(allocation.facts().unit_id, reference.unit);
    let (body, profile): (String, String) = raw(&f)
        .query_row(
            "SELECT body,profile_digest FROM workflow_native_contracts WHERE workflow_id=?1",
            [record.id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    crate::state::managed_binding::check_native_contract_integrity(
        &allocation,
        record.id,
        body.clone(),
        &profile,
    )
    .unwrap();
    let mut negative: serde_json::Value = serde_json::from_str(&body).unwrap();
    let members = negative["members"].as_array_mut().unwrap();
    members.push(serde_json::json!("0".repeat(64)));
    members.sort_by(|a, b| a.as_str().unwrap().cmp(b.as_str().unwrap()));
    // Preserve genuine membership, so digest removal alone accepts this forged
    // negative row. An earlier membership refusal cannot earn integrity credit.
    let error = crate::state::managed_binding::check_native_contract_integrity(
        &allocation,
        record.id,
        String::from_utf8(canonical(&negative)).unwrap(),
        &profile,
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "actual Workflow roster digest differs",
        "CA5d unit integrity refusal"
    );
    let unchanged: String = raw(&f)
        .query_row(
            "SELECT body FROM workflow_native_contracts WHERE workflow_id=?1",
            [record.id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(unchanged, body, "CA5d negative stimulus wrote a contract");
    // Unit integrity only: this is not a non-member marker or genuine-stage kill.
    release_peer(&f, &workflow);
    finish(f).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca6_genuine_contract_refuses_generic_workflow_and_contract_writers() {
    let mut f = fixture("claude", true);
    f.register_real_git_project();
    let (_, tasks) = accept(&f, 1).await;
    let (park, _open) = service_park(&f);
    let deferred = Arc::new(AtomicBool::new(false));
    let signal = deferred.clone();
    let resume = Arc::new(tokio::sync::Semaphore::new(0));
    let permits = resume.clone();
    let seams = Arc::new(ActivationSeams {
        postcommit: Some(Arc::new(|_| {
            anyhow::bail!("CA6 genuine Deferred before cache publication")
        })),
        deferred: Some(Arc::new(move |_| {
            signal.store(true, Ordering::SeqCst);
            let permits = permits.clone();
            Box::pin(async move {
                permits.acquire().await.unwrap().forget();
                Ok(())
            })
        })),
        ..Default::default()
    });
    let (observations, release) = install_pause(&f, Some(seams));
    f.runtime.start().await.unwrap();
    wait_for(
        || observations.lock().unwrap().len() == 1,
        "SETUP: S1 absent",
    )
    .await;
    let original = observations.lock().unwrap()[0].clone();
    park.close();
    f.runtime.wake.notify_one();
    wait_for(|| park.parked(), "SETUP: S5 absent").await;
    release.add_permits(1);
    wait_for(
        || deferred.load(Ordering::SeqCst),
        "SETUP: genuine Deferred absent",
    )
    .await;
    // The actual existing Runtime sweep publishes this SAME live retained plan.
    // The worker remains at S4, so no gate or marker can precede this refusal.
    assert_eq!(f.runtime.observe_task_drivers().unwrap(), 1);
    assert!(!original.plan.is_retained().unwrap());
    let expected = original.plan.planned_binding();
    assert_eq!(
        original.association.binding().unwrap(),
        (expected.0, expected.1, expected.2, expected.3.into())
    );
    let (mut task, mut record, project, goal) = {
        let store = f.owner.store.lock().unwrap();
        (
            store.task(tasks[0].id).unwrap().unwrap(),
            store
                .records(&tasks[0].scope(), RecordKind::Workflow)
                .unwrap()
                .remove(0),
            store.project(tasks[0].project_id).unwrap().unwrap(),
            store.goal(tasks[0].goal_id).unwrap().unwrap(),
        )
    };
    let before_record: String = raw(&f)
        .query_row(
            "SELECT body FROM records WHERE id=?1",
            [record.id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    let before_task: String = raw(&f)
        .query_row(
            "SELECT body FROM tasks WHERE id=?1",
            [task.id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    let error = f
        .owner
        .store
        .lock()
        .unwrap()
        .put_workflow_transition(
            &mut task,
            &mut record,
            None,
            project.version,
            goal.version,
            crate::state::WorkflowAccess::StateOnly,
        )
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("private managed Record writer required"),
        "CA6 generic writer refused before named protected Record stage: {error:#}"
    );
    assert_eq!(
        raw(&f)
            .query_row(
                "SELECT body FROM records WHERE id=?1",
                [record.id.to_string()],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        before_record
    );
    assert_eq!(
        raw(&f)
            .query_row(
                "SELECT body FROM tasks WHERE id=?1",
                [task.id.to_string()],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        before_task
    );
    let c = raw(&f);
    for sql in [
        "UPDATE workflow_native_contracts SET body=body WHERE workflow_id=?1",
        "DELETE FROM workflow_native_contracts WHERE workflow_id=?1",
        "INSERT OR REPLACE INTO workflow_native_contracts SELECT * FROM workflow_native_contracts WHERE workflow_id=?1",
    ] {
        assert!(
            c.execute(sql, [record.id.to_string()]).is_err(),
            "CA6 immutable contract accepted {sql}"
        );
        assert_contract(&f, &original, "CA6 immutable produced contract");
    }
    resume.add_permits(1);
    let (_, workflow) = wait_bound(&f, &tasks[0]).await;
    park.open();
    release_peer(&f, &workflow);
    finish(f).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn nongrant_ca4e_task_controls_cannot_supply_drift_producer() {
    let mut f = fixture("claude", true);
    f.register_real_git_project();
    let (_, tasks) = accept(&f, 1).await;
    let (observations, _release) = install_pause(&f, None);
    f.runtime.start().await.unwrap();
    wait_for(
        || observations.lock().unwrap().len() == 1,
        "SETUP: S1 absent",
    )
    .await;
    let before = f
        .owner
        .store
        .lock()
        .unwrap()
        .task(tasks[0].id)
        .unwrap()
        .unwrap();
    for retry in [true, false] {
        let action = if retry {
            ControlAction::TaskRetry {
                scope: before.scope(),
                expected_task: before.version,
                reason: "CA4e route probe".into(),
            }
        } else {
            ControlAction::TaskCancel {
                scope: before.scope(),
                expected_task: before.version,
                reason: "CA4e route probe".into(),
            }
        };
        assert!(matches!(
            f.runtime
                .handle_control(&f.socket, f.request(action))
                .await
                .unwrap(),
            ControlResponse::Unavailable {
                reason: UnavailableReason::TaskDriverUnavailable,
                ..
            }
        ));
    }
    assert_eq!(
        serde_json::to_value(
            f.owner
                .store
                .lock()
                .unwrap()
                .task(before.id)
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(before).unwrap()
    );
    eprintln!(
        "SETUP_CA4e: genuine Task drift producer unavailable; recovery-stage control earns no credit"
    );
    eprintln!(
        "SETUP_CA4f: no Context/frame control producer established; drift-stage control earns no credit"
    );
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}
