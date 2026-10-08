use super::*;

/// Validate retained authority before introducing any new tables or version.
pub(super) fn validate_legacy(tx: &Transaction<'_>) -> Result<()> {
    let invalid: bool = tx.prepare("PRAGMA foreign_key_check")?.exists([])?;
    ensure!(!invalid, "legacy foreign key violation");
    for table in ["projects", "goals", "tasks", "records"] {
        let mut statement = tx.prepare(&format!("SELECT id,version,body FROM {table}"))?;
        for row in statement.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, u64>(1)?,
                r.get::<_, String>(2)?,
            ))
        })? {
            let (id, version, body) = row?;
            let (actual_id, actual_version) = match table {
                "projects" => {
                    let p: Project = decode(body)?;
                    let root: String =
                        tx.query_row("SELECT root FROM projects WHERE id=?1", [&id], |r| r.get(0))?;
                    ensure!(
                        p.root.to_string_lossy() == root
                            && !p.name.trim().is_empty()
                            && p.root.is_absolute(),
                        "legacy Project mismatch"
                    );
                    (p.id.to_string(), p.version)
                }
                "goals" => {
                    let g: Goal = decode(body)?;
                    indexed_scope(tx, table, &id, &g.scope())?;
                    ensure!(
                        !g.objective.trim().is_empty() && !g.completion_criteria.is_empty(),
                        "invalid legacy Goal"
                    );
                    validate_goal_references(tx, &g)?;
                    (g.id.to_string(), g.version)
                }
                "tasks" => {
                    let t: Task = decode(body)?;
                    indexed_scope(tx, table, &id, &t.scope())?;
                    let issue: Option<u64> =
                        tx.query_row("SELECT issue FROM tasks WHERE id=?1", [&id], |r| r.get(0))?;
                    ensure!(
                        t.issue == issue
                            && t.worktree.is_some() == t.branch.is_some()
                            && !t.title.trim().is_empty()
                            && !t.executor.trim().is_empty(),
                        "invalid legacy Task"
                    );
                    (t.id.to_string(), t.version)
                }
                _ => {
                    let r: Record = decode(body)?;
                    indexed_scope(tx, table, &id, &r.scope)?;
                    let kind: String =
                        tx.query_row("SELECT kind FROM records WHERE id=?1", [&id], |r| r.get(0))?;
                    ensure!(r.kind.key() == kind, "legacy Record kind mismatch");
                    match r.kind {
                        RecordKind::Session => {
                            let s: Session = serde_json::from_value(r.data)?;
                            ensure!(
                                s.scope == r.scope && s.id.0 == r.id.0 && s.worktree.is_absolute(),
                                "legacy Session mismatch"
                            );
                        }
                        RecordKind::Workflow => {
                            let w: crate::workflow::WorkflowSnapshot =
                                serde_json::from_value(r.data)?;
                            ensure!(w.sources.scope == r.scope, "legacy Workflow scope mismatch");
                        }
                        RecordKind::WorktreeLock => {
                            let _: crate::git::WorktreeLock = serde_json::from_value(r.data)?;
                        }
                        _ => {}
                    }
                    (r.id.to_string(), r.version)
                }
            };
            ensure!(
                actual_id == id && actual_version == version,
                "legacy indexed/body identity mismatch"
            );
        }
    }
    let mut s =
        tx.prepare("SELECT project_id,goal_id,task_id,owner,version,body FROM context_versions")?;
    for row in s.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, u64>(4)?,
            r.get::<_, String>(5)?,
        ))
    })? {
        let (p, g, t, owner, version, body) = row?;
        let c: ContextVersion = decode(body)?;
        ensure!(
            c.scope.project_id.to_string() == p
                && c.scope.goal_id.map(|id| id.to_string()) == Some(g)
                && c.scope.task_id.map(|id| id.to_string()) == t
                && context_owner(&c.scope)? == owner
                && c.version == version
                && !c.revision.is_empty(),
            "legacy Context mismatch"
        );
    }
    let mut s = tx.prepare("SELECT project_id,goal_id,task_id,session_id,body FROM usage")?;
    for row in s.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
        ))
    })? {
        let (p, g, t, id, body) = row?;
        let u: Usage = decode(body)?;
        ensure!(
            u.scope.project_id.to_string() == p
                && u.scope.goal_id.map(|id| id.to_string()) == g
                && u.scope.task_id.map(|id| id.to_string()) == t
                && u.session_id.to_string() == id,
            "legacy Usage mismatch"
        );
    }
    Ok(())
}
fn indexed_scope(tx: &Transaction<'_>, table: &str, id: &str, scope: &Scope) -> Result<()> {
    validate_scope(scope)?;
    let project: String = tx.query_row(
        &format!("SELECT project_id FROM {table} WHERE id=?1"),
        [id],
        |r| r.get(0),
    )?;
    ensure!(
        project == scope.project_id.to_string(),
        "legacy Project scope mismatch"
    );
    if table != "goals" {
        let goal: Option<String> = tx.query_row(
            &format!("SELECT goal_id FROM {table} WHERE id=?1"),
            [id],
            |r| r.get(0),
        )?;
        ensure!(
            goal == scope.goal_id.map(|id| id.to_string()),
            "legacy Goal scope mismatch"
        );
    }
    if table == "records" {
        let task: Option<String> =
            tx.query_row("SELECT task_id FROM records WHERE id=?1", [id], |r| {
                r.get(0)
            })?;
        ensure!(
            task == scope.task_id.map(|id| id.to_string()),
            "legacy Task scope mismatch"
        );
    }
    Ok(())
}
