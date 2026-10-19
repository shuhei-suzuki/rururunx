use super::fix1::{cancel, latest};
use super::round2::{rejected, update};
use super::*;

/// Cancelling an interrupted prepare retains ref observation through operator recovery.
#[tokio::test]
async fn b2_fix3_cancel_interrupted_prepare() {
    for applied in [false, true] {
        let f = fixture().await;
        let id = f.claim();
        let c = f.cursor(&id);
        f.store()
            .pipeline_begin_launch(&f.owner, &c, None, None, &json!({}))
            .unwrap();
        let c = f.cursor(&id);
        f.store()
            .pipeline_record_interrupted(&f.owner, &c, &death())
            .unwrap();
        let interrupted = latest(&f, &id);
        assert_eq!(interrupted.state, "interrupted");
        let journal = f.store().pipeline_goal_view(f.goal).unwrap().journals[0].clone();
        assert_eq!(journal.state, "pending");

        cancel(&f);
        let c = f.cursor(&id);
        f.store().pipeline_cancel_run(&f.owner, &c, None).unwrap();
        let held_step = latest(&f, &id);
        assert_eq!(held_step.kind, "prepare");
        assert_eq!(held_step.seq, interrupted.seq);
        assert_eq!(held_step.attempt, interrupted.attempt + 1);
        assert_eq!(held_step.version, 2);
        assert_eq!(held_step.state, "held");
        assert_eq!(held_step.body["reason"], "cancel_pending");
        assert!(held_step.body.get("intent").is_none());
        assert_eq!(
            read_step(
                &f.store().connection,
                &id,
                interrupted.seq,
                interrupted.attempt
            )
            .unwrap(),
            interrupted
        );
        assert_eq!(read_run(&f.store().connection, &id).unwrap().state, "held");
        assert_eq!(
            f.store().pipeline_goal_view(f.goal).unwrap().journals[0],
            journal
        );

        let request = recovery(&f, &id, "abandon_run");
        let mut evidence = recovery_evidence();
        evidence.journal = Some(PipelineRecoveryJournal {
            id: journal.id,
            version: journal.version,
            observed_commit: if applied {
                journal.new_commit
            } else {
                journal.old_commit
            },
        });
        f.store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &evidence)
            .unwrap();
        assert_eq!(
            f.store().pipeline_goal_view(f.goal).unwrap().journals[0].state,
            if applied { "done" } else { "aborted" }
        );
        assert_eq!(
            read_run(&f.store().connection, &id).unwrap().state,
            "abandoned"
        );
        assert_eq!(latest(&f, &id).state, "abandoned");
    }
}

/// Raw SQL cannot free a running process gate by interrupting without confirmation.
#[tokio::test]
async fn b2_fix3_sql_interrupt_requires_death() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    let c = f.cursor(&id);
    f.store()
        .pipeline_record_running(&f.owner, &c, 123, "birth")
        .unwrap();
    for body in [
        "json_set(body,'$.version',version+1,'$.state','interrupted')",
        "json_set(body,'$.version',version+1,'$.state','interrupted','$.death_confirmed',json('false'))",
    ] {
        rejected(
            &f.store().connection,
            &update(
                "pipeline_steps",
                &format!(",state='interrupted',body={body}"),
            ),
            "invalid pipeline step transition",
        );
    }
    assert_eq!(latest(&f, &id).state, "running");
}

/// Every evidence field is required in the write that interrupts a running process.
#[tokio::test]
async fn b2_fix3_sql_interrupt_requires_evidence() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    let c = f.cursor(&id);
    f.store()
        .pipeline_record_running(&f.owner, &c, 123, "birth")
        .unwrap();
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
                    ",state='interrupted',body=json_set(body,'$.version',version+1,'$.state','interrupted','$.death_confirmed',json('true'),'$.death_evidence',json('{evidence}'))"
                ),
            ),
            "invalid pipeline step transition",
        );
    }
    assert_eq!(latest(&f, &id).state, "running");
}
