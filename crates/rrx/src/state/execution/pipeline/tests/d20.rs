use super::round2::{permitted, rejected, update};
use super::*;

/// Process intent requires a launch mark and persists its wall and boot values.
#[tokio::test]
async fn launch_required() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    let cursor = f.cursor(&id);
    assert!(
        f.store()
            .pipeline_begin_launch(
                &f.owner,
                &cursor,
                Some("d20-cookie-1234567"),
                None,
                &json!({})
            )
            .is_err()
    );
    let launch = rrx_process_tracker::Launch {
        wall: (1000, 1),
        boot: Some("A".into()),
        leader_birth: None,
    };
    f.store()
        .pipeline_begin_launch(
            &f.owner,
            &cursor,
            Some("d20-cookie-1234567"),
            Some(&launch),
            &json!({}),
        )
        .unwrap();
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(
        view.steps.last().unwrap().body["launch"],
        json!({"wall":"1000:1","boot":"A"})
    );
}

/// SQL requires textual launch walls and preserves the entire recorded mark.
#[tokio::test]
async fn launch_sql() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    for launch in ["null", "{}", "{\"wall\":4}"] {
        let sql = update(
            "pipeline_steps",
            &format!(
                ",state='launching',cookie='d20-cookie-1234567',body=json_set(body,'$.version',version+1,'$.state','launching','$.launch',json('{launch}'))"
            ),
        );
        rejected(
            &f.store().connection,
            &sql,
            "invalid pipeline step transition",
        );
    }
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_begin_launch(
            &f.owner,
            &cursor,
            Some("d20-cookie-1234567"),
            Some(&rrx_process_tracker::launch_mark().unwrap()),
            &json!({}),
        )
        .unwrap();
    for change in [
        "json_remove(body,'$.launch')",
        "json_set(body,'$.launch.wall','other')",
        "json_set(body,'$.launch.boot','other')",
    ] {
        rejected(
            &f.store().connection,
            &update(
                "pipeline_steps",
                &format!(",body=json_set({change},'$.version',version+1)"),
            ),
            "invalid pipeline step transition",
        );
    }
    permitted(&f.store().connection, &update("pipeline_steps", ""));
}

/// Death APIs require launch evidence and a matched birth whenever one was recorded.
#[tokio::test]
async fn death_api() {
    for running in [false, true] {
        let f = fixture().await;
        let id = f.claim();
        f.launching(&id, "implement");
        if running {
            let c = f.cursor(&id);
            f.store()
                .pipeline_record_running(&f.owner, &c, 123, "1000:1")
                .unwrap();
        }
        let c = f.cursor(&id);
        let mut proof = death();
        proof.launch_checked = false;
        assert!(
            f.store()
                .pipeline_record_exit(&f.owner, &c, &proof, Some("success"), &json!({}))
                .is_err()
        );
        proof.launch_checked = true;
        proof.leader_birth_matched = false;
        let result =
            f.store()
                .pipeline_record_exit(&f.owner, &c, &proof, Some("success"), &json!({}));
        assert_eq!(result.is_err(), running);
        if running {
            f.store()
                .pipeline_record_exit(&f.owner, &c, &death(), Some("success"), &json!({}))
                .unwrap();
        }
    }
}

/// SQL death transitions enforce launch and conditional birth evidence independently of APIs.
#[tokio::test]
async fn death_sql() {
    for running in [false, true] {
        let f = fixture().await;
        let id = f.claim();
        f.launching(&id, "implement");
        if running {
            let c = f.cursor(&id);
            f.store()
                .pipeline_record_running(&f.owner, &c, 123, "1000:1")
                .unwrap();
        }
        for (launch, birth) in [(false, true), (true, false), (true, true)] {
            let mut proof = death();
            proof.launch_checked = launch;
            proof.leader_birth_matched = birth;
            let proof = serde_json::to_string(&proof).unwrap();
            let sql = update(
                "pipeline_steps",
                &format!(
                    ",state='exited',body=json_set(body,'$.version',version+1,'$.state','exited','$.death_confirmed',json('true'),'$.death_evidence',json('{proof}'))"
                ),
            );
            if !launch || running && !birth {
                rejected(
                    &f.store().connection,
                    &sql,
                    "invalid pipeline step transition",
                );
            } else {
                permitted(&f.store().connection, &sql);
            }
        }
    }
}
