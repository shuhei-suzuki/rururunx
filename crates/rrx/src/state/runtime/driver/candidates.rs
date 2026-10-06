//! Coherent bounded nongrant scheduling hints. No row is execution authority.
use super::*;
#[derive(Clone, Debug)]
pub(crate) struct CandidateKey {
    pub(crate) project_rotation: u64,
    pub(crate) project_id: String,
    pub(crate) goal_rotation: u64,
    pub(crate) goal_id: String,
    pub(crate) queue_sequence: u64,
    pub(crate) task_id: String,
}
pub(crate) enum CandidatePage {
    GlobalFull,
    Rows { keys: Vec<CandidateKey>, more: bool },
}
impl Store {
    pub(crate) fn current_task_bounded(&self, key: &CandidateKey) -> Result<Task> {
        let (id,version,project,goal,body):(String,u64,String,String,Option<String>)=self.connection.query_row(
            "SELECT t.id,t.version,t.project_id,t.goal_id,CASE WHEN typeof(t.body)='text' AND length(CAST(t.body AS BLOB))<=1048576 THEN t.body END FROM tasks t JOIN scheduler_tasks s ON s.task_id=t.id AND s.goal_id=t.goal_id AND s.project_id=t.project_id WHERE t.id=?1 AND s.goal_id=?2 AND s.project_id=?3 AND s.queue_sequence=?4",
            params![key.task_id,key.goal_id,key.project_id,key.queue_sequence],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))?;
        let task: Task = decode(body.context("current_task_bounded: body bound or type refused")?)?;
        ensure!(
            task.id.to_string() == id
                && task.version == version
                && task.project_id.to_string() == project
                && task.goal_id.to_string() == goal
                && id == key.task_id
                && project == key.project_id
                && goal == key.goal_id,
            "current_task_bounded: body/index identity differs"
        );
        Ok(task)
    }
    pub(crate) fn ready_driver_candidates(
        &self,
        instance: &str,
        epoch: u64,
        after: Option<&CandidateKey>,
        global_limit: usize,
        project_limit: usize,
    ) -> Result<CandidatePage> {
        ensure!(
            epoch > 0 && global_limit > 0 && project_limit > 0,
            "candidate reader limits invalid"
        );
        let tx = self.connection.unchecked_transaction()?;
        let mut query=tx.prepare("WITH occupied AS (SELECT task_id,project_id FROM task_drivers WHERE state='driving' UNION SELECT task_id,project_id FROM execution_units WHERE native_effects_open=1 OR result_finalization_open=1 UNION SELECT task_id,project_id FROM managed_phase_operations WHERE phase_open=1), total AS (SELECT count(*) AS n FROM occupied), eligible AS (SELECT p.rotation AS pr,s.project_id,g.rotation AS gr,s.goal_id,s.queue_sequence,s.task_id FROM scheduler_tasks s JOIN scheduler_projects p ON p.project_id=s.project_id JOIN scheduler_goals g ON g.goal_id=s.goal_id AND g.project_id=s.project_id WHERE EXISTS(SELECT 1 FROM runtime_epoch WHERE singleton=1 AND instance_id=?1 AND epoch=?2) AND (SELECT n FROM total)<?3 AND NOT EXISTS(SELECT 1 FROM task_drivers d WHERE d.task_id=s.task_id) AND NOT EXISTS(SELECT 1 FROM execution_units u WHERE u.task_id=s.task_id) AND (SELECT count(*) FROM occupied o WHERE o.project_id=s.project_id)<?4 AND (?5 IS NULL OR (p.rotation,s.project_id,g.rotation,s.goal_id,s.queue_sequence,s.task_id)>(?5,?6,?7,?8,?9,?10)) ORDER BY p.rotation,s.project_id,g.rotation,s.goal_id,s.queue_sequence,s.task_id LIMIT 65) SELECT e.instance_id,e.epoch,total.n,c.pr,c.project_id,c.gr,c.goal_id,c.queue_sequence,c.task_id FROM runtime_epoch e CROSS JOIN total LEFT JOIN eligible c ON 1 WHERE e.singleton=1 ORDER BY c.pr,c.project_id,c.gr,c.goal_id,c.queue_sequence,c.task_id")?;
        let mut rows = query.query(params![
            instance,
            epoch,
            global_limit,
            project_limit,
            after.map(|k| k.project_rotation),
            after.map(|k| &k.project_id),
            after.map(|k| k.goal_rotation),
            after.map(|k| &k.goal_id),
            after.map(|k| k.queue_sequence),
            after.map(|k| &k.task_id)
        ])?;
        let mut keys = Vec::new();
        let mut global_full = false;
        let mut seen = false;
        while let Some(row) = rows.next()? {
            seen = true;
            ensure!(
                row.get::<_, String>(0)? == instance && row.get::<_, u64>(1)? == epoch,
                "candidate Runtime owner retired"
            );
            global_full = row.get::<_, usize>(2)? >= global_limit;
            if let Some(project_rotation) = row.get::<_, Option<u64>>(3)? {
                keys.push(CandidateKey {
                    project_rotation,
                    project_id: row.get(4)?,
                    goal_rotation: row.get(5)?,
                    goal_id: row.get(6)?,
                    queue_sequence: row.get(7)?,
                    task_id: row.get(8)?,
                });
            }
        }
        ensure!(seen, "candidate Runtime epoch missing");
        drop(rows);
        drop(query);
        tx.commit()?;
        if global_full {
            return Ok(CandidatePage::GlobalFull);
        }
        let more = keys.len() > 64;
        keys.truncate(64);
        Ok(CandidatePage::Rows { keys, more })
    }
}
