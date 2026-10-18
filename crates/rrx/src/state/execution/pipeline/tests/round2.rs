use super::*;

pub(super) fn rejected(c: &Connection, sql: &str, reason: &str) {
    c.execute_batch("SAVEPOINT b2_case").unwrap();
    let error = c.execute(sql, []).expect_err(sql);
    c.execute_batch("ROLLBACK TO b2_case; RELEASE b2_case")
        .unwrap();
    assert!(error.to_string().contains(reason), "{sql}: {error}");
}

pub(super) fn permitted(c: &Connection, sql: &str) {
    c.execute_batch("SAVEPOINT b2_case").unwrap();
    assert_eq!(
        c.execute(sql, []).unwrap_or_else(|e| panic!("{sql}: {e}")),
        1
    );
    c.execute_batch("ROLLBACK TO b2_case; RELEASE b2_case")
        .unwrap();
}

pub(super) fn update(table: &str, changes: &str) -> String {
    let stamp = if table == "pipeline_ref_journal" {
        "body"
    } else {
        "json_set(body,'$.version',version+1)"
    };
    format!(
        "UPDATE {table} SET version=version+1,body={stamp}{changes} WHERE rowid=(SELECT max(rowid) FROM {table})"
    )
}

pub(super) fn copy(c: &Connection, table: &str, changes: &[(&str, &str)]) -> String {
    let mut q = c.prepare(&format!("PRAGMA table_info({table})")).unwrap();
    let columns = q
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    let values = columns
        .iter()
        .map(|column| {
            changes
                .iter()
                .find(|(name, _)| *name == column)
                .map(|(_, value)| value.to_string())
                .unwrap_or_else(|| column.clone())
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "INSERT OR REPLACE INTO {table}({}) SELECT {values} FROM {table} ORDER BY rowid DESC LIMIT 1",
        columns.join(",")
    )
}

pub(super) async fn populated(table: &str) -> Fixture {
    let f = fixture().await;
    if table == "pipeline_goal_completions" {
        complete_goal(&f);
    } else {
        let id = f.claim();
        f.unknown(&id);
        let request = recovery(&f, &id, "retry");
        f.store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
            .unwrap();
    }
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    write_wait(&tx, &f.owner, &initial_wait(&f.owner)).unwrap();
    tx.commit().unwrap();
    drop(store);
    f
}

/// Every pipeline table retains its history against direct deletion.
#[tokio::test]
async fn b2_all_tables_no_delete() {
    for table in PIPELINE_TABLES {
        let f = populated(table).await;
        let store = f.store();
        let c = &store.connection;
        let count: u64 = c
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert!(count > 0, "{table}");
        rejected(
            c,
            &format!("DELETE FROM {table}"),
            "pipeline history retained",
        );
        assert_eq!(
            c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                .get::<_, u64>(0))
                .unwrap(),
            count,
            "{table}"
        );
    }
}

/// Immutable pipeline records reject even identity-preserving updates.
#[tokio::test]
async fn b2_all_records_no_update() {
    for table in [
        "goal_execution_profiles",
        "pipeline_recoveries",
        "pipeline_qualifications",
        "pipeline_goal_completions",
    ] {
        let f = populated(table).await;
        rejected(
            &f.store().connection,
            &format!("UPDATE {table} SET body=body"),
            "pipeline record immutable",
        );
    }
}

/// Every pipeline insert rejects a retired record or row epoch at the SQL boundary.
#[tokio::test]
async fn b2_all_insert_epochs() {
    for table in PIPELINE_TABLES {
        let f = populated(table).await;
        let store = f.store();
        let c = &store.connection;
        let epoch = match *table {
            "goal_execution_profiles" | "pipeline_recoveries" => "accepted_epoch",
            "pipeline_goal_completions" => "completed_epoch",
            _ => "owner_epoch",
        };
        // The same-key positive control gets past epoch fencing to no_replace.
        rejected(c, &copy(c, table, &[]), "pipeline row cannot be replaced");
        let body = format!("json_set(body,'$.epoch',{})", f.owner.epoch + 1);
        let value = (f.owner.epoch + 1).to_string();
        let sql = copy(c, table, &[(epoch, &value), ("body", &body)]);
        let reason = if matches!(
            *table,
            "pipeline_runs" | "pipeline_steps" | "pipeline_ref_journal" | "pipeline_limit_wait"
        ) {
            "pipeline owner retired"
        } else {
            "pipeline record owner retired"
        };
        rejected(c, &sql, reason);
    }
}

/// Mutable pipeline updates require the current owner epoch independently of the API.
#[tokio::test]
async fn b2_all_update_epochs() {
    for table in [
        "pipeline_runs",
        "pipeline_steps",
        "pipeline_ref_journal",
        "pipeline_limit_wait",
    ] {
        let f = populated(table).await;
        let store = f.store();
        let c = &store.connection;
        // Waiting needs a doubled backoff for a same-epoch state-preserving update.
        let changes = if table == "pipeline_limit_wait" {
            ",backoff_ms=backoff_ms*2"
        } else {
            ""
        };
        permitted(c, &update(table, changes));
        let sql = update(
            table,
            &format!(
                "{changes},owner_epoch=owner_epoch+1,body=json_set(body,'$.version',version+1,'$.epoch',owner_epoch+1)"
            ),
        );
        rejected(c, &sql, "pipeline owner retired");
    }
}

/// Child inserts and updates require a parent run in the current epoch.
#[tokio::test]
async fn b2_child_run_epochs() {
    for table in ["pipeline_steps", "pipeline_recoveries"] {
        let f = populated(table).await;
        let mut store = f.store();
        let c = &store.connection;
        rejected(c, &copy(c, table, &[]), "pipeline row cannot be replaced");
        if table == "pipeline_steps" {
            permitted(c, &update(table, ""));
        } else {
            rejected(
                c,
                &format!("UPDATE {table} SET body=body"),
                "pipeline record immutable",
            );
        }
        let (_, epoch) = store.begin_execution_epoch().unwrap();
        let c = &store.connection;
        let column = if table == "pipeline_steps" {
            "owner_epoch"
        } else {
            "accepted_epoch"
        };
        let value = epoch.to_string();
        let body = format!("json_set(body,'$.epoch',{epoch})");
        rejected(
            c,
            &copy(c, table, &[(column, &value), ("body", &body)]),
            "pipeline run owner retired",
        );
        let sql = if table == "pipeline_steps" {
            update(
                table,
                &format!(
                    ",owner_epoch={epoch},body=json_set(body,'$.version',version+1,'$.epoch',{epoch})"
                ),
            )
        } else {
            format!("UPDATE {table} SET accepted_epoch={epoch}")
        };
        rejected(c, &sql, "pipeline run owner retired");
        // Move only the parent epoch, then the same child operations pass the parent guard.
        c.execute(&format!("UPDATE pipeline_runs SET owner_epoch={epoch},version=version+1,body=json_set(body,'$.epoch',{epoch},'$.version',version+1)"), []).unwrap();
        rejected(
            c,
            &copy(c, table, &[(column, &value), ("body", &body)]),
            "pipeline row cannot be replaced",
        );
        if table == "pipeline_steps" {
            permitted(c, &sql);
        } else {
            rejected(c, &sql, "pipeline record immutable");
        }
    }
}

/// Run identity fields remain immutable on otherwise valid versioned writes.
#[tokio::test]
async fn b2_run_identity_fields() {
    let f = fixture().await;
    f.claim();
    let store = f.store();
    let c = &store.connection;
    permitted(c, &update("pipeline_runs", ""));
    for change in [
        ",id='different',body=json_set(body,'$.id','different','$.version',version+1)",
        ",kind='goal'",
        ",task_id=NULL",
        ",goal_id='different'",
        ",project_id='different'",
        ",base_commit=printf('%040d',1)",
        ",worktree='/different'",
        ",branch='different'",
    ] {
        rejected(
            c,
            &update("pipeline_runs", change),
            "invalid pipeline run transition",
        );
    }
}

/// Step identity fields remain immutable on otherwise valid versioned writes.
#[tokio::test]
async fn b2_step_identity_fields() {
    let f = fixture().await;
    let id = f.claim();
    let other = fixture_with_owner(true, Some(f.owner.clone())).await;
    let other_id = other.claim();
    f.process_step(&id, "implement");
    let store = f.store();
    let c = &store.connection;
    permitted(c, &update("pipeline_steps", ""));
    for change in [
        ",seq=3",
        ",attempt=2",
        ",kind='fix'",
        ",provider=NULL",
        ",round=1",
        ",step_base=printf('%040d',1)",
        ",input_sha256=printf('%064d',1)",
    ] {
        rejected(
            c,
            &update("pipeline_steps", change),
            "invalid pipeline step transition",
        );
    }
    assert_ne!(id, other_id);
    rejected(
        c,
        &update("pipeline_steps", &format!(",run_id='{other_id}'")),
        "invalid pipeline step transition",
    );
}

/// Journal identity fields remain immutable on otherwise valid versioned writes.
#[tokio::test]
async fn b2_journal_identity_fields() {
    let f = fixture().await;
    f.claim();
    let store = f.store();
    let c = &store.connection;
    permitted(c, &update("pipeline_ref_journal", ""));
    for change in [
        ",id='different'",
        ",goal_id='different'",
        ",project_id='different'",
        ",run_id='different'",
        ",kind='task_release'",
        ",ref_name='different'",
        ",old_commit='different'",
        ",new_commit='different'",
    ] {
        rejected(
            c,
            &update("pipeline_ref_journal", change),
            "invalid pipeline journal transition",
        );
    }
}

/// Cookies can first be assigned only by launch intent and cannot then change.
#[tokio::test]
async fn b2_cookie_assignment_and_retention() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    {
        let store = f.store();
        let c = &store.connection;
        rejected(
            c,
            &update("pipeline_steps", ",cookie='0123456789abcdef'"),
            "invalid pipeline step transition",
        );
        permitted(
            c,
            &update(
                "pipeline_steps",
                ",state='launching',cookie='0123456789abcdef',body=json_set(body,'$.version',version+1,'$.state','launching')",
            ),
        );
    }
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_begin_launch(&f.owner, &cursor, Some("0123456789abcdef"), &json!({}))
        .unwrap();
    let store = f.store();
    for change in [",cookie=NULL", ",cookie='fedcba9876543210'"] {
        rejected(
            &store.connection,
            &update("pipeline_steps", change),
            "invalid pipeline step transition",
        );
    }
    permitted(&store.connection, &update("pipeline_steps", ""));
}

/// PID and birth identity can be assigned on running and remain fixed afterwards.
#[tokio::test]
async fn b2_pid_birth_assignment_and_retention() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    {
        let store = f.store();
        rejected(
            &store.connection,
            &update("pipeline_steps", ",pid=123,birth='birth'"),
            "invalid pipeline step transition",
        );
        permitted(
            &store.connection,
            &update(
                "pipeline_steps",
                ",state='running',pid=123,birth='birth',body=json_set(body,'$.version',version+1,'$.state','running')",
            ),
        );
    }
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_running(&f.owner, &cursor, 123, "birth")
        .unwrap();
    let store = f.store();
    for change in [",pid=124", ",pid=NULL", ",birth='other'", ",birth=NULL"] {
        rejected(
            &store.connection,
            &update("pipeline_steps", change),
            "invalid pipeline step transition",
        );
    }
    permitted(&store.connection, &update("pipeline_steps", ""));
}

/// Result commits require exited-to-committed evidence and cannot change afterwards.
#[tokio::test]
async fn b2_result_assignment_and_retention() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    {
        let store = f.store();
        rejected(
            &store.connection,
            &update(
                "pipeline_steps",
                ",result_commit=printf('%040d',1),result_tree=printf('%040d',2)",
            ),
            "invalid pipeline step transition",
        );
    }
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(&f.owner, &cursor, &death(), Some("success"), &json!({}))
        .unwrap();
    {
        let store = f.store();
        rejected(
            &store.connection,
            &update(
                "pipeline_steps",
                ",state='committed',result_commit=printf('%040d',1),body=json_set(body,'$.version',version+1,'$.state','committed')",
            ),
            "invalid pipeline step transition",
        );
        permitted(
            &store.connection,
            &update(
                "pipeline_steps",
                ",state='committed',result_commit=printf('%040d',1),result_tree=printf('%040d',2),body=json_set(body,'$.version',version+1,'$.state','committed')",
            ),
        );
    }
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_commit(
            &f.owner,
            &cursor,
            &"b".repeat(40),
            &"c".repeat(40),
            &json!({}),
        )
        .unwrap();
    let store = f.store();
    for change in [
        ",result_commit=NULL",
        ",result_commit=printf('%040d',1)",
        ",result_tree=NULL",
        ",result_tree=printf('%040d',2)",
    ] {
        rejected(
            &store.connection,
            &update("pipeline_steps", change),
            "invalid pipeline step transition",
        );
    }
    permitted(&store.connection, &update("pipeline_steps", ""));
}

/// Process identity and result columns enforce their persisted value bounds.
#[tokio::test]
async fn b2_process_column_bounds() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    let store = f.store();
    let c = &store.connection;
    for cookie in [
        "''".to_string(),
        "'a'".to_string(),
        "printf('%0129d',0)".to_string(),
    ] {
        rejected(
            c,
            &update(
                "pipeline_steps",
                &format!(
                    ",state='launching',cookie={cookie},body=json_set(body,'$.version',version+1,'$.state','launching')"
                ),
            ),
            "CHECK constraint failed",
        );
    }
    drop(store);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_begin_launch(&f.owner, &cursor, Some("0123456789abcdef"), &json!({}))
        .unwrap();
    {
        let store = f.store();
        for pid in [0, 1, -1] {
            rejected(
                &store.connection,
                &update(
                    "pipeline_steps",
                    &format!(
                        ",state='running',pid={pid},body=json_set(body,'$.version',version+1,'$.state','running')"
                    ),
                ),
                "CHECK constraint failed",
            );
        }
    }
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(&f.owner, &cursor, &death(), Some("success"), &json!({}))
        .unwrap();
    let store = f.store();
    for change in [
        ",result_commit='short',result_tree=printf('%040d',2)",
        ",result_commit=printf('%040d',1),result_tree='short'",
    ] {
        rejected(
            &store.connection,
            &update(
                "pipeline_steps",
                &format!(
                    "{change},state='committed',body=json_set(body,'$.version',version+1,'$.state','committed')"
                ),
            ),
            "CHECK constraint failed",
        );
    }
}

/// Daemon process exclusion blocks other Projects until the incumbent is proven dead.
#[tokio::test]
async fn b2_cross_project_process_exclusion() {
    let a = fixture().await;
    let b = fixture_with_owner(true, Some(a.owner.clone())).await;
    assert_ne!(a.project, b.project);
    let ar = a.claim();
    let br = b.claim();
    a.launching(&ar, "implement");
    b.process_step(&br, "test");
    for state in ["launching", "running", "unknown", "held"] {
        let store = a.store();
        let c = &store.connection;
        c.execute_batch("SAVEPOINT b2_incumbent").unwrap();
        if state != "launching" {
            if state == "held" {
                c.execute(&format!("UPDATE pipeline_steps SET state='running',version=version+1,body=json_set(body,'$.state','running','$.version',version+1) WHERE run_id='{ar}' AND seq=2"), []).unwrap();
            }
            c.execute(&format!("UPDATE pipeline_steps SET state='{state}',version=version+1,body=json_set(body,'$.state','{state}','$.version',version+1) WHERE run_id='{ar}' AND seq=2"), []).unwrap();
        }
        let sql = format!(
            "UPDATE pipeline_steps SET state='launching',cookie='fedcba9876543210',version=version+1,body=json_set(body,'$.version',version+1,'$.state','launching') WHERE run_id='{br}' AND seq=2"
        );
        rejected(c, &sql, "invalid pipeline step transition");
        c.execute_batch("ROLLBACK TO b2_incumbent; RELEASE b2_incumbent")
            .unwrap();
    }
}

/// Secondary unique keys cannot replace retained runs without a primary-key collision.
#[tokio::test]
async fn b2_run_secondary_no_replace() {
    let f = fixture().await;
    f.claim();
    {
        let store = f.store();
        let c = &store.connection;
        c.pragma_update(None, "recursive_triggers", false).unwrap();
        for collision in ["task_id", "worktree", "branch"] {
            let mut changes = vec![
                ("id", "'new-id'"),
                ("task_id", "'new-task'"),
                ("worktree", "'/new-worktree'"),
                ("branch", "'new-branch'"),
                ("body", "json_set(body,'$.id','new-id')"),
            ];
            changes.retain(|(name, _)| *name != collision);
            rejected(
                c,
                &copy(c, "pipeline_runs", &changes),
                "pipeline row cannot be replaced",
            );
        }
    }
    drop(f);
    let f = fixture().await;
    complete_goal(&f);
    let store = f.store();
    rejected(
        &store.connection,
        &copy(
            &store.connection,
            "pipeline_runs",
            &[
                ("id", "'new-goal-run'"),
                ("state", "'running'"),
                ("version", "1"),
                (
                    "body",
                    "json_set(body,'$.id','new-goal-run','$.state','running','$.version',1)",
                ),
            ],
        ),
        "pipeline row cannot be replaced",
    );
}

/// The open-step partial key rejects replacement with a distinct primary key.
#[tokio::test]
async fn b2_step_secondary_no_replace() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    let store = f.store();
    let c = &store.connection;
    c.pragma_update(None, "recursive_triggers", false).unwrap();
    rejected(
        c,
        &copy(c, "pipeline_steps", &[("seq", "3")]),
        "pipeline row cannot be replaced",
    );
}

/// Secondary unique recovery and completion references cannot replace another identity.
#[tokio::test]
async fn b2_record_secondary_no_replace() {
    for (table, changes) in [
        ("pipeline_recoveries", vec![("id", "'new-recovery'")]),
        (
            "pipeline_goal_completions",
            vec![("goal_id", "'different-goal'")],
        ),
    ] {
        let f = populated(table).await;
        let store = f.store();
        let c = &store.connection;
        c.pragma_update(None, "recursive_triggers", false).unwrap();
        rejected(
            c,
            &copy(c, table, &changes),
            "pipeline row cannot be replaced",
        );
    }
}

/// Each journal partial unique key protects history independently of journal id.
#[tokio::test]
async fn b2_journal_secondary_no_replace() {
    let f = fixture().await;
    f.claim();
    {
        let store = f.store();
        let c = &store.connection;
        c.pragma_update(None, "recursive_triggers", false).unwrap();
        rejected(
            c,
            &copy(
                c,
                "pipeline_ref_journal",
                &[("id", "'new-create'"), ("ref_name", "'new-ref'")],
            ),
            "pipeline row cannot be replaced",
        );
    }
    drop(f);
    let a = fixture().await;
    let b = fixture_with_owner(true, Some(a.owner.clone())).await;
    let (ar, j) = held_release(&a);
    let br = b.claim();
    b.advance_prepare(&br);
    let store = a.store();
    let c = &store.connection;
    c.pragma_update(None, "recursive_triggers", false).unwrap();
    let source = format!(" WHERE id='{}'", j.id);
    let release = copy(
        c,
        "pipeline_ref_journal",
        &[
            ("id", "'new-release'"),
            ("ref_name", "'new-ref'"),
            ("state", "'pending'"),
            ("version", "1"),
        ],
    );
    // Restrict the source explicitly; both Projects have a release journal.
    let release = release.replace(
        " ORDER BY rowid DESC",
        &format!("{source} ORDER BY rowid DESC"),
    );
    rejected(c, &release, "pipeline row cannot be replaced");
    // The other run has no release, so only the open-ref key can collide.
    drop(store);
    let store = a.store();
    let c = &store.connection;
    let run = format!("'{br}'");
    let goal = format!("'{}'", b.goal);
    let project = format!("'{}'", b.project);
    let open = copy(
        c,
        "pipeline_ref_journal",
        &[
            ("id", "'new-open'"),
            ("run_id", &run),
            ("goal_id", &goal),
            ("project_id", &project),
            ("state", "'pending'"),
            ("version", "1"),
        ],
    )
    .replace(
        " ORDER BY rowid DESC",
        &format!("{source} ORDER BY rowid DESC"),
    );
    assert_ne!(ar, br);
    rejected(c, &open, "pipeline row cannot be replaced");
}

/// Mutable insert triggers require initial versions, states, and empty launch evidence.
#[tokio::test]
async fn b2_initial_insert_rules() {
    let a = fixture().await;
    a.claim();
    let b = fixture_with_owner(true, Some(a.owner.clone())).await;
    let br = b.claim();
    b.advance_prepare(&br);
    let d = fixture_with_owner(true, Some(a.owner.clone())).await;
    let run = format!("'{br}'");
    let goal = format!("'{}'", d.goal);
    let project = format!("'{}'", d.project);
    let ref_name = format!("'refs/heads/rrx/goal/{}'", d.goal);
    let epoch = a.owner.epoch.to_string();
    let wait_body = format!("'{{\"version\":1,\"state\":\"waiting\",\"epoch\":{epoch}}}'");
    let store = a.store();
    let c = &store.connection;
    for (table, base, cases, reason) in [
        (
            "pipeline_runs",
            vec![
                ("id", "'new-run'"),
                ("task_id", "'new-task'"),
                ("worktree", "'/new-worktree'"),
                ("branch", "'new-branch'"),
                ("body", "json_set(body,'$.id','new-run')"),
            ],
            vec![
                ("version", "2"),
                ("state", "'held'"),
                ("round", "1"),
                ("head_commit", "printf('%040d',1)"),
            ],
            "invalid pipeline run",
        ),
        (
            "pipeline_steps",
            vec![
                ("run_id", run.as_str()),
                ("seq", "2"),
                ("kind", "'test'"),
                ("version", "1"),
                ("state", "'planned'"),
                (
                    "body",
                    "json_set(body,'$.version',1,'$.state','planned','$.command_id','test')",
                ),
            ],
            vec![
                ("state", "'launching'"),
                ("version", "2"),
                ("cookie", "'0123456789abcdef'"),
                ("pid", "123"),
                ("result_commit", "printf('%040d',1)"),
                ("result_tree", "printf('%040d',2)"),
            ],
            "invalid pipeline step insert",
        ),
        (
            "pipeline_ref_journal",
            vec![
                ("id", "'new-journal'"),
                ("goal_id", goal.as_str()),
                ("project_id", project.as_str()),
                ("ref_name", ref_name.as_str()),
                ("state", "'pending'"),
                ("version", "1"),
            ],
            vec![("version", "2"), ("state", "'done'")],
            "invalid pipeline journal",
        ),
        (
            "pipeline_limit_wait",
            vec![
                ("singleton", "1"),
                ("owner_epoch", epoch.as_str()),
                ("version", "1"),
                ("state", "'waiting'"),
                ("not_before", "1"),
                ("backoff_ms", "120000"),
                ("probe_run", "NULL"),
                ("probe_seq", "NULL"),
                ("probe_attempt", "NULL"),
                ("body", wait_body.as_str()),
            ],
            vec![
                ("version", "2"),
                ("state", "'clear'"),
                ("backoff_ms", "60000"),
            ],
            "invalid pipeline limit wait",
        ),
    ] {
        let build = |changes: &[(&str, &str)]| {
            let mut merged = base.clone();
            for change in changes {
                merged.retain(|(name, _)| name != &change.0);
                merged.push(*change);
            }
            let sql = copy(c, table, &merged);
            if table == "pipeline_limit_wait" {
                sql.replace("FROM pipeline_limit_wait ORDER", "FROM pipeline_runs ORDER")
            } else {
                sql
            }
        };
        permitted(c, &build(&[]));
        for (column, value) in cases {
            let body = match column {
                "version" | "state" => Some(format!(
                    "json_set({},'$.{column}',{value})",
                    base.iter()
                        .find(|(n, _)| *n == "body")
                        .map(|(_, v)| *v)
                        .unwrap_or("body")
                )),
                _ => None,
            };
            let mut changes = vec![(column, value)];
            if let Some(body) = &body {
                changes.push(("body", body.as_str()));
            }
            rejected(c, &build(&changes), reason);
        }
    }
}

/// Journal and wait versions advance by exactly one per update.
#[tokio::test]
async fn b2_journal_wait_versions() {
    for table in ["pipeline_ref_journal", "pipeline_limit_wait"] {
        let f = populated(table).await;
        let store = f.store();
        let c = &store.connection;
        let extra = if table == "pipeline_limit_wait" {
            ",backoff_ms=backoff_ms*2"
        } else {
            ""
        };
        permitted(c, &update(table, extra));
        for delta in [0, 2] {
            let sql = format!(
                "UPDATE {table} SET version=version+{delta},body=json_set(body,'$.version',version+{delta}){extra}"
            );
            rejected(
                c,
                &sql,
                if table == "pipeline_limit_wait" {
                    "invalid pipeline limit wait transition"
                } else {
                    "invalid pipeline journal transition"
                },
            );
        }
    }
}

/// Takeover updates preserve the mutable row's state and execution facts.
#[tokio::test]
async fn b2_takeover_changes_only_owner() {
    for (table, changes, reason) in [
        (
            "pipeline_runs",
            vec![
                ",state='held',body=json_set(body,'$.state','held')",
                ",round=1",
                ",head_commit=printf('%040d',1)",
            ],
            "invalid pipeline run transition",
        ),
        (
            "pipeline_steps",
            vec![
                ",state='held',body=json_set(body,'$.state','held')",
                ",cookie='0123456789abcdef'",
                ",pid=123",
                ",result_commit=printf('%040d',1)",
            ],
            "invalid pipeline step transition",
        ),
        (
            "pipeline_ref_journal",
            vec![",state='done'"],
            "invalid pipeline journal transition",
        ),
        (
            "pipeline_limit_wait",
            vec![
                ",state='clear',body=json_set(body,'$.state','clear')",
                ",not_before=2",
                ",backoff_ms=240000",
                ",probe_run='different'",
                ",probe_seq=1",
                ",probe_attempt=1",
            ],
            "invalid pipeline limit wait transition",
        ),
    ] {
        let f = fixture().await;
        f.claim();
        let mut store = f.store();
        let tx = store.connection.transaction().unwrap();
        write_wait(&tx, &f.owner, &initial_wait(&f.owner)).unwrap();
        tx.commit().unwrap();
        let (_, epoch) = store.begin_execution_epoch().unwrap();
        let c = &store.connection;
        if table == "pipeline_steps" {
            c.execute(&format!("UPDATE pipeline_runs SET owner_epoch={epoch},version=version+1,body=json_set(body,'$.epoch',{epoch},'$.version',version+1)"), []).unwrap();
        }
        let stamp = if table == "pipeline_ref_journal" {
            "body".to_owned()
        } else {
            format!("json_set(body,'$.epoch',{epoch},'$.version',version+1)")
        };
        let base = format!("UPDATE {table} SET owner_epoch={epoch},version=version+1,body={stamp}");
        permitted(c, &base);
        for change in changes {
            // Merge state changes into the stamped body rather than overriding its epoch/version.
            let change = change.replace(
                "body=json_set(body,'$.state',",
                &format!("body=json_set(body,'$.epoch',{epoch},'$.version',version+1,'$.state',"),
            );
            rejected(c, &format!("{base}{change}"), reason);
        }
    }
}

/// Every Agent kind requires its designated provider while internal kinds require NULL.
#[tokio::test]
async fn b2_provider_kind_matrix() {
    let f = fixture().await;
    let id = f.claim();
    f.advance_prepare(&id);
    let store = f.store();
    let c = &store.connection;
    for (kind, provider) in [
        ("implement", "'codex'"),
        ("fix", "'codex'"),
        ("confirm", "'codex'"),
        ("review", "'claude'"),
        ("prepare", "NULL"),
        ("test", "NULL"),
        ("release", "NULL"),
    ] {
        let kind_sql = format!("'{kind}'");
        let base = [
            ("seq", "2"),
            ("kind", kind_sql.as_str()),
            ("provider", provider),
            ("version", "1"),
            ("state", "'planned'"),
            (
                "body",
                "json_set(body,'$.version',1,'$.state','planned','$.command_id','test')",
            ),
        ];
        permitted(c, &copy(c, "pipeline_steps", &base));
        for bad in ["NULL", "'codex'", "'claude'"] {
            if bad == provider {
                continue;
            }
            let mut changes = base.to_vec();
            changes[2] = ("provider", bad);
            rejected(
                c,
                &copy(c, "pipeline_steps", &changes),
                "CHECK constraint failed",
            );
        }
    }
}

pub(super) fn native_inventory_fixture() -> (crate::runtime::LegacyFixture, Store, ExecutionUnit) {
    let (legacy, mut store, unit) = native_open_inventory_fixture();
    store.begin_execution_epoch().unwrap();
    (legacy, store, unit)
}
pub(super) fn native_open_inventory_fixture()
-> (crate::runtime::LegacyFixture, Store, ExecutionUnit) {
    let (legacy, mut store) = crate::runtime::legacy_store(vec![crate::runtime::LegacyTask {
        key: "task",
        executor: "codex",
        workflow: WorkflowClass::Quick,
        risk: crate::domain::RiskClass::R0,
        reviewers: &[],
    }]);
    let task = legacy.task();
    let (_, epoch) = store.begin_execution_epoch().unwrap();
    let at = now_ms();
    let id = UnitId::new();
    let project = store.project(task.project_id).unwrap().unwrap();
    let spec = ExecutionUnit {
        id,
        scope: task.scope(),
        kind: UnitKind::Executor,
        generation: 0,
        owner_epoch: epoch,
        version: 0,
        phase: "Implement".into(),
        provider: "codex".into(),
        state: UnitState::Reserved,
        native_effects_open: true,
        result_finalization_open: true,
        work: None,
        cleanup: CleanupOutcome::Unknown,
        disposition: Disposition::Active,
        worktree: project.worktree_root.join(id.to_string()),
        branch: Some(format!("rrx/{id}")),
        base_sha: "a".repeat(40),
        profile_digest: "b".repeat(64),
        cookie: Uuid::new_v4().to_string(),
        session_id: None,
        artifact_id: None,
        wait_reason: None,
        capacity_retry_at: None,
        created_at: at,
        updated_at: at,
    };
    let unit = store.reserve_execution(spec, task.version).unwrap();
    (legacy, store, unit)
}

/// Native inventory returns the real persisted cleanup Unit cookie without writing.
#[test]
fn b2_inventory_native_body_cookie() {
    let (_legacy, store, unit) = native_inventory_fixture();
    let before = store.connection.total_changes();
    let inventory = store.pipeline_mode_inventory().unwrap();
    assert_eq!(inventory.native_cookies, vec![unit.cookie]);
    assert_eq!(store.connection.total_changes(), before);
}

/// Missing or unusable native process identity cannot establish mode quiescence.
#[test]
fn b2_inventory_missing_cookie_refuses() {
    let (_legacy, store, unit) = native_inventory_fixture();
    assert_eq!(
        store.pipeline_mode_inventory().unwrap().native_cookies,
        vec![unit.cookie]
    );
    for cookie in ["NULL", "''", "'malformed'"] {
        store
            .connection
            .execute(
                &format!(
                    "UPDATE execution_units SET body=json_set(body,'$.cookie',{cookie}) WHERE id=?1"
                ),
                [unit.id.to_string()],
            )
            .unwrap();
        let error = store.pipeline_mode_inventory().unwrap_err();
        assert!(
            error
                .to_string()
                .contains("native cleanup cookie unavailable")
                || format!("{error:#}").contains("invalid type: null"),
            "{cookie}: {error:#}"
        );
    }
}

/// Indexed run, step, and wait identity stamps must agree with the JSON snapshot.
#[tokio::test]
async fn b2_body_identity_stamps() {
    for table in ["pipeline_runs", "pipeline_steps", "pipeline_limit_wait"] {
        let f = populated(table).await;
        let store = f.store();
        let c = &store.connection;
        let extra = if table == "pipeline_limit_wait" {
            ",backoff_ms=backoff_ms*2"
        } else {
            ""
        };
        permitted(c, &update(table, extra));
        let mut fields = vec![
            ("version", "999"),
            ("state", "'different'"),
            ("epoch", "999"),
        ];
        if table == "pipeline_runs" {
            fields.push(("id", "'different'"));
        }
        for (field, value) in fields {
            let sql = update(
                table,
                &format!("{extra},body=json_set(body,'$.version',version+1,'$.{field}',{value})"),
            );
            rejected(c, &sql, "CHECK constraint failed");
        }
    }
}

/// Waiting backoff doubles to its cap and never uses an arbitrary extension width.
#[tokio::test]
async fn b2_wait_backoff_rules() {
    let f = fixture().await;
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    write_wait(&tx, &f.owner, &initial_wait(&f.owner)).unwrap();
    tx.commit().unwrap();
    for _ in 0..6 {
        let c = &store.connection;
        rejected(
            c,
            &update("pipeline_limit_wait", ",backoff_ms=60000"),
            "invalid pipeline limit wait transition",
        );
        permitted(
            c,
            &update(
                "pipeline_limit_wait",
                ",backoff_ms=MIN(backoff_ms*2,1800000)",
            ),
        );
        c.execute(
            &update(
                "pipeline_limit_wait",
                ",backoff_ms=MIN(backoff_ms*2,1800000)",
            ),
            [],
        )
        .unwrap();
    }
    assert_eq!(
        read_wait(&store.connection).unwrap().unwrap().backoff_ms,
        1800000
    );
}

/// A daemon wait gates Agents across Projects and admits only its designated probe.
#[tokio::test]
async fn b2_wait_gates_other_project_agents() {
    let a = fixture().await;
    let b = fixture_with_owner(true, Some(a.owner.clone())).await;
    let ar = a.claim();
    let br = b.claim();
    a.advance_prepare(&ar);
    b.process_step(&br, "implement");
    let store = a.store();
    let c = &store.connection;
    let launch = format!(
        "UPDATE pipeline_steps SET version=version+1,state='launching',cookie='0123456789abcdef',body=json_set(body,'$.version',version+1,'$.state','launching') WHERE run_id='{br}' AND seq=2"
    );
    permitted(c, &launch);
    drop(store);
    {
        let mut store = a.store();
        let tx = store.connection.transaction().unwrap();
        write_wait(&tx, &a.owner, &initial_wait(&a.owner)).unwrap();
        tx.commit().unwrap();
    }
    let store = a.store();
    let c = &store.connection;
    rejected(c, &launch, "invalid pipeline step transition");
    // A probe for another Project cannot admit this Agent.
    c.execute(&format!("UPDATE pipeline_limit_wait SET version=version+1,state='probing',probe_run='{ar}',probe_seq=1,probe_attempt=1,body=json_set(body,'$.version',version+1,'$.state','probing')"), []).unwrap();
    rejected(c, &launch, "invalid pipeline step transition");
    drop(store);
    // Reset to waiting via the supported extension, then designate this Agent.
    let mut store = a.store();
    let tx = store.connection.transaction().unwrap();
    let mut wait = read_wait(&tx).unwrap().unwrap();
    wait.state = "waiting".into();
    wait.backoff_ms *= 2;
    wait.probe_run = None;
    wait.probe_seq = None;
    wait.probe_attempt = None;
    write_wait(&tx, &a.owner, &wait).unwrap();
    wait = read_wait(&tx).unwrap().unwrap();
    wait.state = "probing".into();
    wait.probe_run = Some(br);
    wait.probe_seq = Some(2);
    wait.probe_attempt = Some(1);
    write_wait(&tx, &a.owner, &wait).unwrap();
    tx.commit().unwrap();
    permitted(&store.connection, &launch);
}
