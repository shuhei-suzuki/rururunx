//! Complete, finite Project namespace inventory captured outside SharedStore.
//! Historical Task content is collision evidence, never a Driver credential.
use super::*;

const ROWS: usize = 4096;
const BYTES: usize = 16 * 1024 * 1024;
const BODY: usize = 1024 * 1024;
struct Entry {
    id: String,
    project: String,
    goal: String,
    version: u64,
    issue: Option<u64>,
    body: String,
    task: Task,
}
pub(in crate::state) struct NamespaceSnapshot {
    project: ProjectId,
    rows: Vec<Entry>,
}
impl NamespaceSnapshot {
    pub(super) fn read(c: &Connection, project: ProjectId) -> Result<Self> {
        Self::read_bounded(c, project, ROWS, BYTES)
    }
    fn read_bounded(
        c: &Connection,
        project: ProjectId,
        max_rows: usize,
        max_bytes: usize,
    ) -> Result<Self> {
        let mut statement = c.prepare("SELECT CASE WHEN typeof(id)='text' AND length(CAST(id AS BLOB))=36 THEN id END,CASE WHEN typeof(project_id)='text' AND length(CAST(project_id AS BLOB))=36 THEN project_id END,CASE WHEN typeof(goal_id)='text' AND length(CAST(goal_id AS BLOB))=36 THEN goal_id END,version,issue,length(CAST(body AS BLOB)) FROM tasks WHERE project_id=?1 ORDER BY id")?;
        let mut cursor = statement.query([project.to_string()])?;
        let mut headers = Vec::new();
        let mut bytes = 0usize;
        while let Some(row) = cursor.next()? {
            ensure!(
                headers.len() < max_rows,
                "Driver namespace row budget exceeded"
            );
            let length: usize = row.get(5)?;
            bytes = bytes
                .checked_add(length)
                .context("Driver namespace byte overflow")?;
            ensure!(
                length <= BODY && bytes <= max_bytes,
                "Driver namespace byte budget exceeded"
            );
            let id = row
                .get::<_, Option<String>>(0)?
                .context("Driver namespace ID header invalid")?;
            let indexed_project = row
                .get::<_, Option<String>>(1)?
                .context("Driver namespace Project header invalid")?;
            let goal = row
                .get::<_, Option<String>>(2)?
                .context("Driver namespace Goal header invalid")?;
            ensure!(
                id.parse::<TaskId>().is_ok()
                    && indexed_project.parse::<ProjectId>().is_ok()
                    && goal.parse::<GoalId>().is_ok(),
                "Driver namespace UUID header invalid"
            );
            headers.push((
                id,
                indexed_project,
                goal,
                row.get::<_, u64>(3)?,
                row.get::<_, Option<u64>>(4)?,
                length,
            ));
        }
        drop(cursor);
        let mut rows = Vec::with_capacity(headers.len());
        for (id, indexed_project, goal, version, issue, length) in headers {
            let body: String =
                c.query_row("SELECT body FROM tasks WHERE id=?1", [&id], |r| r.get(0))?;
            ensure!(body.len() == length, "Driver namespace snapshot changed");
            let task: Task = decode(body.clone())?;
            ensure!(
                task.id.to_string() == id
                    && task.project_id == project
                    && indexed_project == project.to_string()
                    && task.goal_id.to_string() == goal
                    && task.version == version
                    && task.issue == issue,
                "Driver namespace body/index identity differs"
            );
            ensure!(
                task.worktree.is_some() == task.branch.is_some(),
                "Driver namespace path/branch differs"
            );
            rows.push(Entry {
                id,
                project: indexed_project,
                goal,
                version,
                issue,
                body,
                task,
            });
        }
        Ok(Self { project, rows })
    }
    fn current_count(&self, c: &Connection) -> Result<(usize, i32)> {
        // Compiled sentinel, not a caller budget or truncated admitted inventory.
        // Production schema supplies the Project-leading covering index.
        let mut statement=c.prepare("SELECT count(*) FROM (SELECT 1 FROM tasks INDEXED BY tasks_by_project_issue WHERE project_id=?1 LIMIT 4097)")?;
        let count = statement.query_row([self.project.to_string()], |r| r.get(0))?;
        Ok((
            count,
            statement.get_status(rusqlite::StatementStatus::VmStep),
        ))
    }
    pub(in crate::state) fn validate_current(&self, c: &Connection) -> Result<()> {
        let (count, _) = self.current_count(c)?;
        ensure!(
            count == self.rows.len(),
            "Driver namespace inventory changed"
        );
        for row in &self.rows {
            let exact: bool = c.query_row("SELECT EXISTS(SELECT 1 FROM tasks WHERE id=?1 AND project_id=?2 AND goal_id=?3 AND version=?4 AND issue IS ?5 AND body=?6)",params![row.id,row.project,row.goal,row.version,row.issue,row.body], |r|r.get(0))?;
            ensure!(exact, "Driver namespace original row changed");
        }
        Ok(())
    }
    pub(in crate::state) fn check_collision(&self, task: &Task) -> Result<()> {
        ensure!(
            task.project_id == self.project,
            "Driver namespace Project differs"
        );
        if let (Some(path), Some(branch)) = (&task.worktree, &task.branch) {
            for row in &self.rows {
                if row.task.id != task.id {
                    ensure!(
                        row.task.worktree.as_ref() != Some(path)
                            && row.task.branch.as_ref() != Some(branch),
                        "task worktree/branch already owned"
                    );
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Historical collision data only. No accepted Goal/Driver/Unit/owner proof.
    fn history(count: usize, size: Option<usize>) -> (Connection, Task, usize) {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE TABLE tasks(id TEXT PRIMARY KEY,project_id TEXT,goal_id TEXT,version INTEGER,issue INTEGER,body TEXT); CREATE INDEX tasks_by_project_issue ON tasks(project_id,issue)").unwrap();
        let project = ProjectId::new();
        let goal = GoalId::new();
        let mut first = None;
        let mut bytes = 0;
        for i in 0..count {
            let mut task = Task::new(project, goal, format!("historical-{i}"), "codex".into());
            if let Some(size) = size {
                task.title.clear();
                let prefix = serde_json::to_string(&task).unwrap().len();
                task.title = "x".repeat(size - prefix);
            }
            let body = serde_json::to_string(&task).unwrap();
            bytes += body.len();
            c.execute(
                "INSERT INTO tasks VALUES(?1,?2,?3,?4,?5,?6)",
                params![
                    task.id.to_string(),
                    project.to_string(),
                    goal.to_string(),
                    task.version,
                    task.issue,
                    body
                ],
            )
            .unwrap();
            first.get_or_insert(task);
        }
        (c, first.unwrap(), bytes)
    }
    #[test]
    fn production_namespace_complete_row_boundary_and_surplus() {
        let (c, task, _) = history(ROWS, None);
        let snapshot = NamespaceSnapshot::read(&c, task.project_id).unwrap();
        assert_eq!(snapshot.rows.len(), ROWS);
        snapshot.validate_current(&c).unwrap();
        let extra = Task::new(
            task.project_id,
            task.goal_id,
            "surplus".into(),
            "codex".into(),
        );
        c.execute(
            "INSERT INTO tasks VALUES(?1,?2,?3,0,NULL,?4)",
            params![
                extra.id.to_string(),
                extra.project_id.to_string(),
                extra.goal_id.to_string(),
                serde_json::to_string(&extra).unwrap()
            ],
        )
        .unwrap();
        assert!(
            NamespaceSnapshot::read(&c, task.project_id)
                .err()
                .unwrap()
                .to_string()
                .contains("row budget")
        );
        assert!(snapshot.validate_current(&c).is_err());
        assert_eq!(
            NamespaceSnapshot::read_bounded(&c, task.project_id, ROWS + 1, BYTES)
                .unwrap()
                .rows
                .len(),
            ROWS + 1
        );
    }
    #[test]
    fn production_namespace_encoded_byte_boundary_and_surplus_before_decode() {
        let (c, task, bytes) = history(BYTES / BODY, Some(BODY));
        assert_eq!(bytes, BYTES);
        NamespaceSnapshot::read(&c, task.project_id)
            .unwrap()
            .validate_current(&c)
            .unwrap();
        let surplus = Task::new(
            task.project_id,
            task.goal_id,
            "valid surplus".into(),
            "codex".into(),
        );
        let extra = surplus.id;
        c.execute(
            "INSERT INTO tasks VALUES(?1,?2,?3,0,NULL,?4)",
            params![
                extra.to_string(),
                task.project_id.to_string(),
                task.goal_id.to_string(),
                serde_json::to_string(&surplus).unwrap()
            ],
        )
        .unwrap();
        assert!(
            NamespaceSnapshot::read(&c, task.project_id)
                .err()
                .unwrap()
                .to_string()
                .contains("byte budget")
        );
        c.execute("DELETE FROM tasks WHERE id=?1", [extra.to_string()])
            .unwrap();
        // Deliberately invalid content: complete length overflow must win before
        // copying/decoding any body, including this final surplus row.
        c.execute(
            "INSERT INTO tasks VALUES(?1,?2,?3,0,NULL,'!')",
            params![
                extra.to_string(),
                task.project_id.to_string(),
                task.goal_id.to_string()
            ],
        )
        .unwrap();
        assert!(
            NamespaceSnapshot::read(&c, task.project_id)
                .err()
                .unwrap()
                .to_string()
                .contains("byte budget")
        );
        c.execute("DELETE FROM tasks WHERE id=?1", [extra.to_string()])
            .unwrap();
        c.execute(
            "UPDATE tasks SET body=?1 WHERE id=?2",
            params!["!".repeat(BODY + 1), task.id.to_string()],
        )
        .unwrap();
        assert!(
            NamespaceSnapshot::read(&c, task.project_id)
                .err()
                .unwrap()
                .to_string()
                .contains("byte budget")
        );
    }
    #[test]
    fn namespace_identity_and_complete_current_cas_preserve_collision_guards() {
        let (mut c, task, _) = history(2, None);
        let snapshot = NamespaceSnapshot::read(&c, task.project_id).unwrap();
        let mut assigned = task.clone();
        assigned.worktree = Some("/private/tmp/historical-namespace".into());
        assigned.branch = Some("rrx/historical".into());
        let other = snapshot.rows.iter().find(|r| r.task.id != task.id).unwrap();
        let mut foreign = other.task.clone();
        foreign.worktree = assigned.worktree.clone();
        foreign.branch = Some("distinct".into());
        c.execute(
            "UPDATE tasks SET body=?1 WHERE id=?2",
            params![serde_json::to_string(&foreign).unwrap(), other.id],
        )
        .unwrap();
        assert!(snapshot.validate_current(&c).is_err());
        let current = NamespaceSnapshot::read(&c, task.project_id).unwrap();
        assert!(current.check_collision(&assigned).is_err());
        assigned.worktree = Some("/private/tmp/disjoint-namespace".into());
        current.check_collision(&assigned).unwrap();
        let tx = c.transaction().unwrap();
        current.validate_current(&tx).unwrap();
        tx.execute(
            "UPDATE tasks SET version=1 WHERE id=?1",
            [task.id.to_string()],
        )
        .unwrap();
        assert!(current.validate_current(&tx).is_err());
        assert!(
            NamespaceSnapshot::read(&tx, task.project_id)
                .err()
                .unwrap()
                .to_string()
                .contains("body/index")
        );
        tx.rollback().unwrap();
    }
    #[test]
    fn namespace_rejects_each_index_identity_and_malformed_bound_content() {
        for column in ["id", "project_id", "goal_id", "version", "issue"] {
            let (c, task, _) = history(1, None);
            let original = NamespaceSnapshot::read(&c, task.project_id).unwrap();
            let value = if column == "version" || column == "issue" {
                "1".to_owned()
            } else {
                format!("'{}'", uuid::Uuid::new_v4())
            };
            c.execute(&format!("UPDATE tasks SET {column}={value}"), [])
                .unwrap();
            assert!(original.validate_current(&c).is_err());
            // Changing indexed Project removes the row from the selected scope;
            // the complete snapshot CAS still rejects this disappearance.
            if column == "project_id" {
                assert_eq!(
                    NamespaceSnapshot::read(&c, task.project_id)
                        .unwrap()
                        .rows
                        .len(),
                    0
                );
            } else {
                assert!(NamespaceSnapshot::read(&c, task.project_id).is_err());
            }
        }
        let (c, task, _) = history(1, None);
        c.execute("UPDATE tasks SET body='!'", []).unwrap();
        assert!(NamespaceSnapshot::read(&c, task.project_id).is_err());
    }
    #[test]
    fn namespace_header_projection_refuses_before_body_materialization() {
        for column in ["id", "goal_id"] {
            let (c, task, _) = history(1, None);
            c.execute(
                &format!("UPDATE tasks SET {column}=?1,body='!'"),
                ["x".repeat(BODY + 1)],
            )
            .unwrap();
            let error = NamespaceSnapshot::read(&c, task.project_id).err().unwrap();
            assert!(
                error.to_string().contains("header invalid"),
                "{column}: {error}"
            );
        }
    }
    #[test]
    fn namespace_post_capture_surplus_refuses_with_bounded_actual_sql_steps() {
        let (mut c, task, _) = history(2, None);
        let captured = NamespaceSnapshot::read(&c, task.project_id).unwrap();
        let (count, steps) = captured.current_count(&c).unwrap();
        assert_eq!(count, 2);
        assert!(steps < 100);
        captured.validate_current(&c).unwrap();
        // Historical content only, no accepted Goal/Driver or proof. The current
        // Project grows after a legitimate snapshot; the actual production index
        // and count reader must stop at the sentinel before copying/decode/CAS.
        c.execute("WITH RECURSIVE n(i) AS (VALUES(1) UNION ALL SELECT i+1 FROM n WHERE i<65536) INSERT INTO tasks(id,project_id,goal_id,version,issue,body) SELECT printf('historical-extra-%d',i),?1,?2,0,NULL,'!' FROM n",params![task.project_id.to_string(),task.goal_id.to_string()]).unwrap();
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        let error = captured.validate_current(&tx).unwrap_err();
        assert!(error.to_string().contains("inventory changed"));
        let (count, steps) = captured.current_count(&tx).unwrap();
        assert!(
            steps < 65536,
            "actual current-count VM traversal exceeded finite profile: {steps}"
        );
        assert_eq!(
            count,
            ROWS + 1,
            "current reader did not stop at compiled sentinel"
        );
        tx.rollback().unwrap();
        assert_eq!(
            c.query_row("SELECT count(*) FROM tasks", [], |r| r.get::<_, usize>(0))
                .unwrap(),
            65538
        );
        eprintln!(
            "namespace actual SQLite VM steps: unchanged <100; grown sentinel={count}, steps={steps}"
        );
    }
}
