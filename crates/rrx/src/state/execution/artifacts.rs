use super::*;

// Historical inspection has current Runtime read authority, not the producing
// Task's native/finalization authority. Retired generations remain inspectable.
fn validate_retained_inspection(
    connection: &Connection,
    epoch: u64,
    artifact: &ResultArtifact,
) -> Result<()> {
    let current: u64 = connection.query_row(
        "SELECT epoch FROM runtime_epoch WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    ensure!(
        current == epoch && epoch > 0,
        "retained reader epoch retired"
    );
    let recorded = self_artifact_tx(connection, artifact.id)?;
    ensure!(
        matches!(
            recorded.state,
            ArtifactState::Ready | ArtifactState::Published
        ) && serde_json::to_value(&recorded)? == serde_json::to_value(artifact)?
            && valid_oid(&recorded.revision)
            && valid_oid(&recorded.base_sha)
            && recorded.manifest_sha256.len() == 64
            && recorded
                .manifest_sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit()),
        "retained reader artifact snapshot changed"
    );
    let unit = unit_tx(connection, recorded.unit_id)?;
    ensure!(
        unit.scope == recorded.scope && unit.kind == UnitKind::Executor,
        "retained reader producing unit mismatch"
    );
    Ok(())
}

impl Store {
    pub(crate) fn validate_retained_inspection(
        &self,
        epoch: u64,
        artifact: &ResultArtifact,
    ) -> Result<()> {
        validate_retained_inspection(&self.connection, epoch, artifact)
    }
    pub(crate) fn reserve_retained_inspection(
        &mut self,
        epoch: u64,
        artifact: &ResultArtifact,
        id: OperationId,
        action: &str,
    ) -> Result<()> {
        self.reserve_retained_inspection_bound(epoch, artifact, id, action, None)
    }
    pub(crate) fn reserve_retained_inspection_bound(
        &mut self,
        epoch: u64,
        artifact: &ResultArtifact,
        id: OperationId,
        action: &str,
        recovery: Option<&source_recovery::SourceReadBinding>,
    ) -> Result<()> {
        ensure!(
            matches!(
                action,
                "commit_ref"
                    | "base_ref"
                    | "commit_graph"
                    | "base_graph"
                    | "fsck"
                    | "source_tree"
                    | "source_blob"
            ),
            "unsupported retained reader action"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        validate_retained_inspection(&tx, epoch, artifact)?;
        if let Some(binding) = recovery {
            ensure!(
                binding.owns_artifact(artifact.id),
                "source-bound reader artifact differs"
            );
            source_recovery::validate_binding(&tx, binding)?;
        }
        let effect = ManagedEffect {
            id,
            unit_id: artifact.unit_id,
            scope: artifact.scope.clone(),
            kind: "retained_git".into(),
            idempotency_key: format!("retained-{id}"),
            expected_target: format!(
                "artifact:{}:{}:epoch:{epoch}:{}:{action}",
                artifact.id, artifact.version, artifact.manifest_sha256
            ),
            state: EffectState::Pending,
            receipt: BTreeMap::new(),
            version: 1,
        };
        let (p, g, t) = scope_keys(&effect.scope)?;
        tx.execute("INSERT INTO managed_effects(id,unit_id,project_id,goal_id,task_id,idempotency_key,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,'pending',1,?7)",
            params![id.to_string(),effect.unit_id.to_string(),p,g,t,effect.idempotency_key,serde_json::to_string(&effect)?])?;
        append_event(
            &tx,
            &artifact.scope,
            "execution.retained_reader_intent",
            json!({"unit":artifact.unit_id,"artifact":artifact.id,"version":artifact.version,"epoch":epoch,"operation":id,"action":action}),
        )?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn finish_retained_inspection(
        &mut self,
        epoch: u64,
        artifact: &ResultArtifact,
        id: OperationId,
        exit: Option<i32>,
        group_error: bool,
        program_digest: &str,
    ) -> Result<()> {
        self.finish_retained_inspection_bound(
            epoch,
            artifact,
            id,
            (exit, group_error, program_digest),
            None,
        )
    }
    pub(crate) fn finish_retained_inspection_bound(
        &mut self,
        epoch: u64,
        artifact: &ResultArtifact,
        id: OperationId,
        receipt: (Option<i32>, bool, &str),
        recovery: Option<&source_recovery::SourceReadBinding>,
    ) -> Result<()> {
        let (exit, group_error, program_digest) = receipt;
        ensure!(
            program_digest.len() == 64 && program_digest.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid retained reader program reference digest"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        validate_retained_inspection(&tx, epoch, artifact)?;
        if let Some(binding) = recovery {
            ensure!(
                binding.owns_artifact(artifact.id),
                "source-bound reader artifact differs"
            );
            source_recovery::validate_binding(&tx, binding)?;
        }
        let mut effect = effect_tx(&tx, id)?;
        ensure!(
            effect.unit_id == artifact.unit_id
                && effect.scope == artifact.scope
                && effect.kind == "retained_git"
                && effect.version == 1
                && effect.state == EffectState::Pending
                && effect.expected_target.starts_with(&format!(
                    "artifact:{}:{}:epoch:{epoch}:{}:",
                    artifact.id, artifact.version, artifact.manifest_sha256
                )),
            "retained helper intent changed"
        );
        effect.state = EffectState::Confirmed;
        effect.version = 2;
        effect.receipt = BTreeMap::from([
            (
                "exit".into(),
                exit.map_or_else(|| "signal".into(), |code| code.to_string()),
            ),
            (
                "group_cleanup".into(),
                if group_error { "unknown" } else { "requested" }.into(),
            ),
            ("program_reference_sha256".into(), program_digest.into()),
        ]);
        ensure!(tx.execute("UPDATE managed_effects SET state='confirmed',version=2,body=?1 WHERE id=?2 AND version=1 AND state='pending'",
            params![serde_json::to_string(&effect)?,id.to_string()])? == 1, "retained helper CAS conflict");
        append_event(
            &tx,
            &artifact.scope,
            "execution.effect_reconciled",
            json!({"unit":artifact.unit_id,"operation":id,"state":EffectState::Confirmed}),
        )?;
        tx.commit()?;
        Ok(())
    }
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
        verified: &ResultArtifact,
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
        let id = verified.id;
        let mut artifact = self_artifact_tx(&tx, id)?;
        ensure!(
            artifact.unit_id == unit.id
                && artifact.scope == unit.scope
                && artifact.state == ArtifactState::Ready
                && serde_json::to_value(&artifact)? == serde_json::to_value(verified)?,
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
    #[cfg(test)]
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
    let mut unit = validate_authority(tx, publication.authority(), false, true)?;
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
    let mut artifact = self_artifact_tx(tx, publication.artifact().id)?;
    ensure!(
        serde_json::to_value(&artifact)? == serde_json::to_value(publication.artifact())?,
        "verified publication artifact snapshot changed"
    );
    ensure!(
        artifact.unit_id == unit.id
            && artifact.scope == unit.scope
            && artifact.state == ArtifactState::Ready
            && task.revision.as_ref() == Some(&artifact.revision)
            && context.revision == artifact.revision
            && after.sources.revision == artifact.revision
            && after.sources.artifact == Some(artifact.id)
            && context.source_hashes.get("workflow:artifact") == Some(&artifact.id.to_string())
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

/// Close readonly finalization only together with acceptance of its exact input.
pub(in crate::state) fn complete_workflow_readonly_tx(
    tx: &Transaction<'_>,
    completion: &ReadonlyCompletion,
    task: &Task,
    next_record: &Record,
    previous_record: &Record,
    context: &ContextVersion,
) -> Result<()> {
    use crate::workflow::{Actor, AttemptState, WorkflowSnapshot};
    let mut unit = validate_authority(tx, completion.authority(), false, true)?;
    ensure!(
        matches!(unit.kind, UnitKind::Reviewer | UnitKind::Verifier)
            && unit.work == Some(WorkOutcome::Success)
            && !unit.native_effects_open
            && unit.scope == task.scope(),
        "readonly acceptance requires successful owning terminal"
    );
    let before: WorkflowSnapshot = serde_json::from_value(previous_record.data.clone())?;
    let after: WorkflowSnapshot = serde_json::from_value(next_record.data.clone())?;
    let index = before
        .active
        .context("readonly acceptance requires active reservation")?;
    let old = before
        .history
        .get(index)
        .context("readonly reservation invalid")?;
    let next = after
        .history
        .get(index)
        .context("readonly phase history missing")?;
    ensure!(
        old.phase.actor() == Actor::Reviewer
            && old.phase.key() == unit.phase
            && old.unit.as_ref() == Some(&ManagedUnitRef::from(&unit))
            && old.session_id == unit.session_id
            && unit.session_id.is_some()
            && before.generation == after.generation
            && next.state == AttemptState::Succeeded
            && after.active.is_none(),
        "readonly Workflow/Session/generation binding mismatch"
    );
    let artifact = self_artifact_tx(tx, completion.artifact().id)?;
    ensure!(
        serde_json::to_value(&artifact)? == serde_json::to_value(completion.artifact())?
            && artifact.state == ArtifactState::Published
            && artifact.scope == unit.scope
            && unit.artifact_id == Some(artifact.id)
            && unit.base_sha == artifact.revision,
        "readonly verified artifact snapshot changed"
    );
    let evidence = after
        .completed
        .get(&old.phase)
        .context("readonly passed evidence missing")?;
    ensure!(
        task.revision.as_ref() == Some(&artifact.revision)
            && context.revision == artifact.revision
            && after.sources.revision == artifact.revision
            && after.sources.artifact == Some(artifact.id)
            && context.source_hashes.get("workflow:artifact") == Some(&artifact.id.to_string())
            && evidence.revision == artifact.revision
            && evidence.session_id == unit.session_id
            && evidence.review_approved == Some(true),
        "readonly exact input/evidence binding mismatch"
    );
    ensure!(
        artifact
            .dependencies
            .iter()
            .all(
                |(key, value)| after.sources.source_versions.get(key) == Some(value)
                    && context.source_hashes.get(key) == Some(value)
            ),
        "readonly dependency versions changed"
    );
    unit.result_finalization_open = false;
    write_unit(tx, &mut unit)?;
    tx.execute("INSERT INTO cleanup_jobs(unit_id,next_due,attempts,version) VALUES(?1,?2,0,1) ON CONFLICT(unit_id) DO NOTHING",
        params![unit.id.to_string(), now_ms()])?;
    append_event(
        tx,
        &unit.scope,
        "execution.readonly_result_accepted",
        json!({"unit":unit.id,"artifact":artifact.id,"sha":artifact.revision,"workflow":next_record.id,"context_version":context.version}),
    )?;
    Ok(())
}
