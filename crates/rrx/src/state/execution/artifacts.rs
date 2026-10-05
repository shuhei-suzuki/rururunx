use super::*;

impl Store {
    pub fn result_artifact(&self, id: ArtifactId) -> Result<ResultArtifact> {
        self_artifact_tx(&self.connection, id)
    }
    pub fn result_artifacts(&self, scope: &Scope) -> Result<Vec<ResultArtifact>> {
        let (p, g, t) = scope_keys(scope)?;
        let mut s=self.connection.prepare("SELECT id FROM result_artifacts WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 ORDER BY rowid")?;
        let ids = s
            .query_map(params![p, g, t], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        ids.into_iter()
            .map(|id| {
                let a = self.result_artifact(id.parse()?)?;
                ensure!(a.scope == *scope, "foreign artifact");
                Ok(a)
            })
            .collect()
    }
    pub(crate) fn stage_result(
        &mut self,
        authority: &ExecutionAuthority,
        artifact: &ResultArtifact,
    ) -> Result<()> {
        ensure!(
            artifact.unit_id == authority.unit_id
                && artifact.scope == authority.scope
                && artifact.state == ArtifactState::Staging
                && artifact.version == 1
                && valid_oid(&artifact.revision)
                && valid_oid(&artifact.base_sha)
                && artifact.repository.is_absolute()
                && artifact.manifest.is_absolute(),
            "invalid staged artifact"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        validate_authority(&tx, authority, false, true)?;
        let (p, g, t) = scope_keys(&artifact.scope)?;
        tx.execute("INSERT INTO result_artifacts(id,unit_id,project_id,goal_id,task_id,state,version,body) VALUES(?1,?2,?3,?4,?5,'staging',1,?6)",
            params![artifact.id.to_string(),artifact.unit_id.to_string(),p,g,t,serde_json::to_string(artifact)?])?;
        append_event(
            &tx,
            &artifact.scope,
            "execution.result_staged",
            json!({"unit":artifact.unit_id,"artifact":artifact.id,"sha":artifact.revision}),
        )?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn ready_result(&mut self, artifact: &ResultArtifact, expected: u64) -> Result<()> {
        ensure!(
            artifact.state == ArtifactState::Ready
                && artifact.manifest_sha256.len() == 64
                && artifact.version == expected + 1,
            "invalid ready artifact"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let old = self_artifact_tx(&tx, artifact.id)?;
        ensure!(
            old.state == ArtifactState::Staging
                && old.version == expected
                && old.scope == artifact.scope
                && old.unit_id == artifact.unit_id
                && old.revision == artifact.revision
                && old.base_sha == artifact.base_sha
                && old.repository == artifact.repository
                && old.manifest == artifact.manifest,
            "artifact identity/CAS changed"
        );
        let changed=tx.execute("UPDATE result_artifacts SET state='ready',version=?1,body=?2 WHERE id=?3 AND version=?4 AND state='staging'",
            params![artifact.version,serde_json::to_string(artifact)?,artifact.id.to_string(),expected])?;
        ensure!(changed == 1, "artifact CAS conflict");
        for (name, digest) in &artifact.dependencies {
            tx.execute(
                "INSERT INTO artifact_dependencies(artifact_id,name,digest) VALUES(?1,?2,?3)",
                params![artifact.id.to_string(), name, digest],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn publish_execution_result(
        &mut self,
        authority: &ExecutionAuthority,
        id: ArtifactId,
        expected_task: u64,
    ) -> Result<ResultArtifact> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = validate_authority(&tx, authority, false, true)?;
        let workflows: u64 = tx.query_row(
            "SELECT COUNT(*) FROM records WHERE task_id=?1 AND kind='workflow'",
            [unit.scope.task_id.unwrap().to_string()],
            |r| r.get(0),
        )?;
        ensure!(
            workflows == 0,
            "Workflow-owned results require coordinated publication"
        );
        ensure!(
            unit.work == Some(WorkOutcome::Success) && unit.kind == UnitKind::Executor,
            "result requires known successful executor work"
        );
        let mut artifact = self_artifact_tx(&tx, id)?;
        ensure!(
            artifact.unit_id == unit.id
                && artifact.scope == unit.scope
                && artifact.state == ArtifactState::Ready,
            "ready artifact binding mismatch"
        );
        let mut task: Task = read_tx(&tx, "tasks", &unit.scope.task_id.unwrap().to_string())?
            .context("unknown Task")?;
        ensure!(
            task.version == expected_task && !task_terminal(task.state),
            "Task result CAS/lifecycle conflict"
        );
        artifact.state = ArtifactState::Published;
        artifact.version += 1;
        tx.execute("UPDATE result_artifacts SET state='published',version=?1,body=?2 WHERE id=?3 AND state='ready'",params![artifact.version,serde_json::to_string(&artifact)?,id.to_string()])?;
        unit.result_finalization_open = false;
        unit.artifact_id = Some(id);
        write_unit(&tx, &mut unit)?;
        task.revision = Some(artifact.revision.clone());
        let next = put_task_tx(&tx, &task)?;
        append_event(
            &tx,
            &unit.scope,
            "execution.result_published",
            json!({"unit":unit.id,"artifact":id,"sha":artifact.revision,"task_version":next.version}),
        )?;
        tx.commit()?;
        Ok(artifact)
    }
    pub(crate) fn close_result_as_draft(
        &mut self,
        authority: &ExecutionAuthority,
        reason: &str,
    ) -> Result<()> {
        ensure!(
            !reason.is_empty() && reason.len() <= 256,
            "invalid draft reason"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = validate_authority(&tx, authority, false, true)?;
        unit.result_finalization_open = false;
        write_unit(&tx, &mut unit)?;
        append_event(
            &tx,
            &unit.scope,
            "execution.result_retained_draft",
            json!({"unit":unit.id,"reason":reason}),
        )?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn invalidate_execution_artifact(
        &mut self,
        id: ArtifactId,
        reason: &str,
    ) -> Result<()> {
        ensure!(
            !reason.is_empty() && reason.len() <= 256,
            "invalid artifact reason"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut artifact = self_artifact_tx(&tx, id)?;
        artifact.state = ArtifactState::Invalid;
        artifact.version += 1;
        tx.execute(
            "UPDATE result_artifacts SET state='invalid',version=?1,body=?2 WHERE id=?3",
            params![
                artifact.version,
                serde_json::to_string(&artifact)?,
                id.to_string()
            ],
        )?;
        append_event(
            &tx,
            &artifact.scope,
            "execution.artifact_invalid",
            json!({"artifact":id,"reason":reason}),
        )?;
        tx.commit()?;
        Ok(())
    }
}

pub(in crate::state) fn publish_workflow_result_tx(
    tx: &Transaction<'_>,
    publication: &WorkflowPublication,
    task: &Task,
    next_record: &Record,
    previous_record: &Record,
    context: &ContextVersion,
) -> Result<()> {
    use crate::workflow::{Actor, AttemptState, WorkflowSnapshot};
    let mut unit = validate_authority(tx, &publication.authority, false, true)?;
    ensure!(
        unit.kind == UnitKind::Executor
            && unit.work == Some(WorkOutcome::Success)
            && !unit.native_effects_open
            && unit.scope == task.scope(),
        "publication requires the owning successful executor terminal"
    );
    let before: WorkflowSnapshot = serde_json::from_value(previous_record.data.clone())?;
    let after: WorkflowSnapshot = serde_json::from_value(next_record.data.clone())?;
    let index = before
        .active
        .context("publication requires active phase reservation")?;
    let old = before
        .history
        .get(index)
        .context("publication reservation index invalid")?;
    let next = after
        .history
        .get(index)
        .context("publication phase history missing")?;
    ensure!(
        old.phase.actor() == Actor::Executor
            && old.phase.key() == unit.phase
            && old.session_id == unit.session_id
            && unit.session_id.is_some()
            && before.generation == after.generation
            && next.state == AttemptState::Succeeded
            && after.active.is_none(),
        "publication Workflow/Session/generation binding mismatch"
    );
    let evidence = after
        .completed
        .get(&old.phase)
        .context("publication lacks passed phase evidence")?;
    let mut artifact = self_artifact_tx(tx, publication.artifact)?;
    ensure!(
        artifact.unit_id == unit.id
            && artifact.scope == unit.scope
            && artifact.state == ArtifactState::Ready
            && task.revision.as_ref() == Some(&artifact.revision)
            && context.revision == artifact.revision
            && after.sources.revision == artifact.revision
            && evidence.revision == artifact.revision
            && evidence.session_id == unit.session_id
            && evidence
                .artifacts
                .contains(&format!("rrx-artifact:{}", artifact.id)),
        "publication exact artifact/evidence binding mismatch"
    );
    ensure!(
        artifact
            .dependencies
            .iter()
            .all(
                |(key, value)| after.sources.source_versions.get(key) == Some(value)
                    && context.source_hashes.get(key) == Some(value)
            ),
        "publication dependency versions changed"
    );
    artifact.state = ArtifactState::Published;
    let expected = artifact.version;
    bump(&mut artifact.version)?;
    ensure!(tx.execute("UPDATE result_artifacts SET state='published',version=?1,body=?2 WHERE id=?3 AND version=?4 AND state='ready'",params![artifact.version,serde_json::to_string(&artifact)?,artifact.id.to_string(),expected])?==1,"artifact publication CAS mismatch");
    unit.result_finalization_open = false;
    unit.artifact_id = Some(artifact.id);
    write_unit(tx, &mut unit)?;
    append_event(
        tx,
        &unit.scope,
        "execution.result_published",
        json!({"unit":unit.id,"artifact":artifact.id,"sha":artifact.revision,"workflow":next_record.id,"context_version":context.version}),
    )?;
    Ok(())
}
