use super::fix1::{cancel, latest};
use super::round2::{permitted, rejected, update};
use super::*;

fn begin_prepare(f: &Fixture, id: &str) {
    let c = f.cursor(id);
    f.store()
        .pipeline_begin_launch(&f.owner, &c, None, &json!({}))
        .unwrap();
}
fn create(f: &Fixture) -> PipelineJournal {
    f.store().pipeline_goal_view(f.goal).unwrap().journals[0].clone()
}
fn settle(f: &Fixture, id: &str, observed: &str) {
    let j = create(f);
    let c = f.cursor(id);
    f.store()
        .pipeline_release_settle(&f.owner, &c, &j.id, j.version, observed)
        .unwrap();
}

/// Confirmed death remains usable by exit, stop, commit and cancellation APIs.
#[tokio::test]
async fn b2_fix2_death_api_paths() {
    for running in [false, true] {
        for end in ["exit", "hold", "interrupt", "cancel"] {
            let f = fixture().await;
            let id = f.claim();
            f.launching(&id, "implement");
            if running || end == "cancel" {
                let c = f.cursor(&id);
                f.store()
                    .pipeline_record_running(&f.owner, &c, 123, "birth")
                    .unwrap();
            }
            let c = f.cursor(&id);
            match end {
                "exit" => {
                    f.store()
                        .pipeline_record_exit(&f.owner, &c, &death(), Some("success"), &json!({}))
                        .unwrap();
                    let c = f.cursor(&id);
                    f.store()
                        .pipeline_record_commit(
                            &f.owner,
                            &c,
                            &"b".repeat(40),
                            &"c".repeat(40),
                            &json!({}),
                        )
                        .unwrap();
                    let c = f.cursor(&id);
                    f.store()
                        .pipeline_record_interrupted(&f.owner, &c, &death())
                        .unwrap();
                }
                "hold" => f
                    .store()
                    .pipeline_hold(&f.owner, &c, "stop", Some(&death()))
                    .unwrap(),
                "interrupt" => f
                    .store()
                    .pipeline_record_interrupted(&f.owner, &c, &death())
                    .unwrap(),
                "cancel" => {
                    cancel(&f);
                    f.store()
                        .pipeline_cancel_run(&f.owner, &c, Some(&death()))
                        .unwrap();
                }
                _ => unreachable!(),
            }
            assert_eq!(latest(&f, &id).body["death_confirmed"], true);
        }
    }
}

/// Raw SQL cannot free the process gate by exiting without complete death evidence.
#[tokio::test]
async fn b2_fix2_sql_exit_requires_death() {
    for running in [false, true] {
        let f = fixture().await;
        let id = f.claim();
        f.launching(&id, "implement");
        if running {
            let c = f.cursor(&id);
            f.store()
                .pipeline_record_running(&f.owner, &c, 123, "birth")
                .unwrap();
        }
        for body in [
            "json_set(body,'$.version',version+1,'$.state','exited')",
            "json_set(body,'$.version',version+1,'$.state','exited','$.death_confirmed',json('false'))",
        ] {
            rejected(
                &f.store().connection,
                &update("pipeline_steps", &format!(",state='exited',body={body}")),
                "invalid pipeline step transition",
            );
        }
    }
}

/// Every death-evidence field is required in the SQL write that exits a process.
#[tokio::test]
async fn b2_fix2_sql_exit_requires_evidence() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    for field in [
        "leader_exit_observed",
        "group_kill_delivered",
        "leader_reaped",
        "group_empty",
        "discovery_complete",
        "matching_processes",
    ] {
        let mut evidence = serde_json::to_value(death()).unwrap();
        evidence.as_object_mut().unwrap().remove(field);
        rejected(
            &f.store().connection,
            &update(
                "pipeline_steps",
                &format!(
                    ",state='exited',body=json_set(body,'$.version',version+1,'$.state','exited','$.death_confirmed',json('true'),'$.death_evidence',json('{evidence}'))"
                ),
            ),
            "invalid pipeline step transition",
        );
    }
}

/// Exited and committed rows cannot acquire a new death fact in a later SQL write.
#[tokio::test]
async fn b2_fix2_sql_late_death_refused() {
    for committed in [false, true] {
        let f = fixture().await;
        let id = f.claim();
        f.launching(&id, "implement");
        let c = f.cursor(&id);
        f.store()
            .pipeline_record_exit(&f.owner, &c, &death(), Some("success"), &json!({}))
            .unwrap();
        if committed {
            let c = f.cursor(&id);
            f.store()
                .pipeline_record_commit(&f.owner, &c, &"b".repeat(40), &"c".repeat(40), &json!({}))
                .unwrap();
        }
        for absent in [
            "json_remove(body,'$.death_confirmed')",
            "json_set(body,'$.death_confirmed',json('false'))",
        ] {
            let store = f.store();
            let c = &store.connection;
            c.execute_batch("SAVEPOINT missing_fact").unwrap();
            // Remove evidence rather than manufacture a successful death observation.
            c.execute(
                &update(
                    "pipeline_steps",
                    &format!(",body=json_set({absent},'$.version',version+1)"),
                ),
                [],
            )
            .unwrap();
            for target in [
                if committed { "committed" } else { "exited" },
                "interrupted",
            ] {
                rejected(
                    c,
                    &update(
                        "pipeline_steps",
                        &format!(
                            ",state='{target}',body=json_set(body,'$.version',version+1,'$.state','{target}','$.death_confirmed',json('true'))"
                        ),
                    ),
                    "invalid pipeline step transition",
                );
            }
            c.execute_batch("ROLLBACK TO missing_fact; RELEASE missing_fact")
                .unwrap();
        }
    }
}

/// Cancellation abandons held runs whose last step cannot own a live process.
#[tokio::test]
async fn b2_fix2_cancel_safe_held() {
    for kind in [
        "never_launched",
        "dead",
        "rejected",
        "internal",
        "internal_unstarted",
    ] {
        let f = fixture().await;
        let id = f.claim();
        match kind {
            "never_launched" => {
                f.process_step(&id, "implement");
                let c = f.cursor(&id);
                f.store()
                    .pipeline_hold(&f.owner, &c, "native_quota_exhausted", None)
                    .unwrap();
            }
            "dead" => {
                f.launching(&id, "implement");
                let c = f.cursor(&id);
                f.store()
                    .pipeline_hold(&f.owner, &c, "stop", Some(&death()))
                    .unwrap();
            }
            "rejected" => {
                f.launching(&id, "implement");
                let c = f.cursor(&id);
                f.store().pipeline_reject(&f.owner, &c, "rejected").unwrap();
            }
            "internal" => {
                begin_prepare(&f, &id);
                settle(&f, &id, &create(&f).new_commit);
                let c = f.cursor(&id);
                f.store()
                    .pipeline_hold(&f.owner, &c, "prepare_failed", None)
                    .unwrap();
            }
            "internal_unstarted" => {
                let c = f.cursor(&id);
                f.store()
                    .pipeline_hold(&f.owner, &c, "prepare_failed", None)
                    .unwrap();
            }
            _ => unreachable!(),
        }
        cancel(&f);
        let c = f.cursor(&id);
        f.store().pipeline_cancel_run(&f.owner, &c, None).unwrap();
        assert_eq!(
            read_run(&f.store().connection, &id).unwrap().state,
            "abandoned"
        );
    }
}

/// A Running Goal cannot abandon a held run without an abandon recovery record.
#[tokio::test]
async fn b2_fix2_running_abandon_requires_recovery() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    let c = f.cursor(&id);
    f.store()
        .pipeline_hold(&f.owner, &c, "quota", None)
        .unwrap();
    rejected(
        &f.store().connection,
        &update(
            "pipeline_runs",
            ",state='abandoned',body=json_set(body,'$.version',version+1,'$.state','abandoned')",
        ),
        "invalid pipeline run transition",
    );
}

/// Retrying its own held Goal creation lets the new prepare observe old or new again.
#[tokio::test]
async fn b2_fix2_retry_own_create() {
    for applied in [false, true] {
        let f = fixture().await;
        let id = f.claim();
        begin_prepare(&f, &id);
        settle(&f, &id, &"b".repeat(40));
        assert_eq!(create(&f).state, "held");
        let request = recovery(&f, &id, "retry");
        f.store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
            .unwrap();
        begin_prepare(&f, &id);
        let j = create(&f);
        settle(
            &f,
            &id,
            if applied {
                &j.new_commit
            } else {
                &j.old_commit
            },
        );
        assert_eq!(create(&f).state, if applied { "done" } else { "pending" });
    }
}

/// A Cancelled Goal records an observed applied creation as done.
#[tokio::test]
async fn b2_fix2_cancelled_settle_new() {
    let f = fixture().await;
    let id = f.claim();
    begin_prepare(&f, &id);
    cancel(&f);
    settle(&f, &id, &create(&f).new_commit);
    assert_eq!(create(&f).state, "done");
}

/// A Cancelled Goal aborts an observed unapplied creation and interrupts prepare.
#[tokio::test]
async fn b2_fix2_cancelled_settle_old() {
    let f = fixture().await;
    let id = f.claim();
    begin_prepare(&f, &id);
    cancel(&f);
    settle(&f, &id, &create(&f).old_commit);
    assert_eq!(create(&f).state, "aborted");
    assert_eq!(latest(&f, &id).state, "interrupted");
}

/// Cancel preserves an executed prepare journal until its ref has been observed.
#[tokio::test]
async fn b2_fix2_cancel_preserves_executed_create() {
    let f = fixture().await;
    let id = f.claim();
    begin_prepare(&f, &id);
    cancel(&f);
    let c = f.cursor(&id);
    f.store().pipeline_cancel_run(&f.owner, &c, None).unwrap();
    assert_eq!(create(&f).state, "pending");
    settle(&f, &id, &create(&f).new_commit);
    let c = f.cursor(&id);
    f.store().pipeline_cancel_run(&f.owner, &c, None).unwrap();
    assert_eq!(
        read_run(&f.store().connection, &id).unwrap().state,
        "abandoned"
    );
}

/// Cancel retains an executed creation across held and retried prepare attempts.
#[tokio::test]
async fn b2_fix2_cancel_preserves_create_history() {
    for retried in [false, true] {
        let f = fixture().await;
        let id = f.claim();
        begin_prepare(&f, &id);
        let c = f.cursor(&id);
        f.store()
            .pipeline_hold(&f.owner, &c, "prepare_failed", None)
            .unwrap();
        if retried {
            let request = recovery(&f, &id, "retry");
            f.store()
                .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
                .unwrap();
        }
        cancel(&f);
        let c = f.cursor(&id);
        f.store().pipeline_cancel_run(&f.owner, &c, None).unwrap();
        assert_eq!(create(&f).state, "pending");
        let j = create(&f);
        let mut evidence = recovery_evidence();
        evidence.journal = Some(PipelineRecoveryJournal {
            id: j.id,
            version: j.version,
            observed_commit: j.new_commit,
        });
        let request = recovery(&f, &id, "abandon_run");
        f.store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &evidence)
            .unwrap();
        assert_eq!(create(&f).state, "done");
    }
}

/// Recovery records observed Goal creation independently of cancellation.
#[tokio::test]
async fn b2_fix2_cancelled_recover_create() {
    for observation in ["new", "old", "unexpected"] {
        let f = fixture().await;
        let id = f.claim();
        begin_prepare(&f, &id);
        settle(&f, &id, &"b".repeat(40));
        cancel(&f);
        let j = create(&f);
        let mut evidence = recovery_evidence();
        evidence.journal = Some(PipelineRecoveryJournal {
            id: j.id,
            version: j.version,
            observed_commit: match observation {
                "new" => j.new_commit,
                "old" => j.old_commit,
                _ => "c".repeat(40),
            },
        });
        let request = recovery(&f, &id, "abandon_run");
        let result = f
            .store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &evidence);
        if observation == "unexpected" {
            assert!(result.is_err());
            assert_eq!(create(&f).state, "held");
        } else {
            result.unwrap();
            assert_eq!(
                create(&f).state,
                if observation == "new" {
                    "done"
                } else {
                    "aborted"
                }
            );
        }
    }
}

/// Ordinary retries remain available twice after a recovery creates a new attempt.
#[tokio::test]
async fn b2_fix2_recovery_then_retries() {
    for kind in ["implement", "test"] {
        let f = fixture().await;
        let id = f.claim();
        f.launching(&id, kind);
        let c = f.cursor(&id);
        f.store()
            .pipeline_record_launch_unknown(&f.owner, &c, "lost")
            .unwrap();
        let request = recovery(&f, &id, "retry");
        f.store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
            .unwrap();
        for attempt in [3, 4] {
            let c = f.cursor(&id);
            f.store()
                .pipeline_begin_launch(
                    &f.owner,
                    &c,
                    Some(&format!("retry-cookie-{attempt:016}")),
                    &json!({}),
                )
                .unwrap();
            let c = f.cursor(&id);
            f.store()
                .pipeline_record_interrupted(&f.owner, &c, &death())
                .unwrap();
            let c = f.cursor(&id);
            f.store()
                .pipeline_plan_next(&f.owner, &c, &planned(&id, 2, attempt, kind), true)
                .unwrap();
            assert_eq!(latest(&f, &id).attempt, attempt);
        }
    }
}

/// Explicit nonpositive rowids cannot be persisted in pipeline history.
#[tokio::test]
async fn b2_fix2_nonpositive_rowid() {
    for rowid in [0, -1, -2] {
        let f = fixture_mode(false).await;
        let store = f.store();
        // Qualification has no other producer preconditions; this isolates the rowid guard.
        rejected(
            &store.connection,
            &format!(
                "INSERT INTO pipeline_qualifications(rowid,owner_epoch,subject,state,body) VALUES({rowid},{},'platform','refused','{{}}')",
                f.owner.epoch
            ),
            "pipeline rowid must be positive",
        );
    }
}

/// Implicit inserts use the negative BEFORE sentinel and persist positive rowids.
#[tokio::test]
async fn b2_fix2_implicit_rowid() {
    let f = fixture_mode(false).await;
    let store = f.store();
    let c = &store.connection;
    permitted(
        c,
        &format!(
            "INSERT INTO pipeline_qualifications(rowid,owner_epoch,subject,state,body) VALUES(100,{},'platform','refused','{{}}')",
            f.owner.epoch
        ),
    );
    c.execute_batch(
        "CREATE TEMP TABLE rowid_observation(value INTEGER);
        CREATE TEMP TRIGGER observe_rowid BEFORE INSERT ON main.pipeline_qualifications
        BEGIN INSERT INTO rowid_observation VALUES(NEW.rowid); END;",
    )
    .unwrap();
    for subject in ["platform", "mode_quiescence"] {
        store.connection.execute("INSERT INTO pipeline_qualifications(owner_epoch,subject,state,body) VALUES(?1,?2,'refused','{}')", params![f.owner.epoch, subject]).unwrap();
    }
    let c = &store.connection;
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM rowid_observation WHERE value=-1",
            [],
            |r| r.get::<_, u64>(0)
        )
        .unwrap(),
        2
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM pipeline_qualifications WHERE rowid>0",
            [],
            |r| r.get::<_, u64>(0)
        )
        .unwrap(),
        2
    );
}

/// A rejected cookie requires discovery even after the cancelled run is abandoned.
#[tokio::test]
async fn b2_fix2_pipeline_cookie_discovery() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    let c = f.cursor(&id);
    f.store().pipeline_reject(&f.owner, &c, "rejected").unwrap();
    cancel(&f);
    let c = f.cursor(&id);
    f.store().pipeline_cancel_run(&f.owner, &c, None).unwrap();
    let v = f.store().pipeline_mode_inventory().unwrap();
    assert_eq!(
        v.pipeline_runs + v.pipeline_journals + v.pipeline_processes,
        0
    );
    assert_eq!(
        v.to_managed_native,
        PipelineQuiescenceVerdict::CookieDiscoveryRequired
    );
}

/// A closed native Unit's retained cleanup cookie requires discovery for pipeline mode.
#[test]
fn b2_fix2_native_cookie_discovery() {
    let (_legacy, store, _) = super::round2::native_inventory_fixture();
    let v = store.pipeline_mode_inventory().unwrap();
    assert_eq!(
        v.to_cli_pipeline,
        PipelineQuiescenceVerdict::CookieDiscoveryRequired
    );
}
