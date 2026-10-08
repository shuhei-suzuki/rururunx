//! Bounded durable waiting attention; no readiness credential or Driver claim.
use super::super::*;

pub(super) fn native_binding_hold() -> &'static str {
    "{\"dispatch_available\":false,\"kind\":\"native_binding_unavailable\"}"
}

impl Store {
    pub(crate) fn reconcile_runtime_attention(
        &mut self,
        instance: &str,
        epoch: u64,
        after: u64,
    ) -> Result<(u64, bool)> {
        let tx = self.connection.unchecked_transaction()?;
        let current: (String, u64) = tx.query_row(
            "SELECT instance_id,epoch FROM runtime_epoch WHERE singleton=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        ensure!(
            current == (instance.to_owned(), epoch) && epoch > 0,
            "Runtime control owner retired"
        );
        let mut query=tx.prepare("SELECT s.task_id,s.goal_id,s.project_id,s.queue_sequence,s.attention,EXISTS(SELECT 1 FROM task_drivers d WHERE d.task_id=s.task_id AND d.goal_id=s.goal_id AND d.project_id=s.project_id AND d.owner_epoch=?2 AND d.state='driving') FROM scheduler_tasks s WHERE s.queue_sequence>?1 ORDER BY s.queue_sequence LIMIT 257")?;
        let mut rows = query
            .query_map(params![after, epoch], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, u64>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, bool>(5)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(query);
        let more = rows.len() > 256;
        rows.truncate(256);
        let mut next = after;
        let held = native_binding_hold();
        let mut changed = Vec::new();
        for (task, goal, project, sequence, attention, driving) in rows {
            let count:u64=tx.query_row("SELECT count(*) FROM tasks t JOIN goals g ON g.id=t.goal_id AND g.project_id=t.project_id JOIN goal_authority a ON a.goal_id=g.id AND a.project_id=g.project_id WHERE t.id=?1 AND t.goal_id=?2 AND t.project_id=?3",params![task,goal,project],|r|r.get(0))?;
            ensure!(count == 1, "scheduler accepted Task scope differs");
            if !driving && attention.as_deref() != Some(held) {
                changed.push((task, goal, project, sequence, attention));
            }
            next = sequence;
        }
        tx.commit()?;
        // Unchanged holds never acquire an Immediate writer transaction.
        if !changed.is_empty() {
            let tx = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)?;
            let current: (String, u64) = tx.query_row(
                "SELECT instance_id,epoch FROM runtime_epoch WHERE singleton=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            ensure!(
                current == (instance.to_owned(), epoch) && epoch > 0,
                "Runtime attention owner retired"
            );
            for (task, goal, project, sequence, old) in changed {
                // Attention is factual only. A concurrent change loses this CAS,
                // leaving its actual durable value for the next read-only sweep.
                tx.execute("UPDATE scheduler_tasks SET attention=?1 WHERE task_id=?2 AND goal_id=?3 AND project_id=?4 AND queue_sequence=?5 AND attention IS ?6 AND NOT EXISTS(SELECT 1 FROM task_drivers d WHERE d.task_id=scheduler_tasks.task_id AND d.goal_id=scheduler_tasks.goal_id AND d.project_id=scheduler_tasks.project_id AND d.owner_epoch=?7 AND d.state='driving')",params![held,task,goal,project,sequence,old,epoch])?;
            }
            tx.commit()?;
        }
        Ok((next, more))
    }
}
