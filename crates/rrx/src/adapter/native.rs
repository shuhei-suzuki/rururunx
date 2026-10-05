//! Native adapter routing requires explicit execution identity and retained inputs.
//! No generic launch or caller-supplied recovery/PID can substitute for admission.
use super::*;
use crate::execution::{
    self, WorkOutcome,
    native::{ManagedInput, ManagedSessionRef, NativeSessions, NativeStart, NativeStatus},
};

pub(crate) struct NativeAdapter {
    pub(crate) owner: Arc<execution::RuntimeOwner>,
    pub(crate) name: String,
    pub(crate) provider: String,
    pub(crate) program: PathBuf,
    pub(crate) sessions: Arc<NativeSessions>,
}
fn mapped(error: anyhow::Error) -> AdapterError {
    // Never turn a provider payload into a durable diagnostic.
    let kind = if let Some(e) = error.downcast_ref::<AdapterError>() {
        e.kind
    } else if let Some(e) = error.downcast_ref::<execution::NativeFailure>() {
        match e {
            execution::NativeFailure::AuthenticationUnavailable => {
                ErrorKind::AuthenticationUnavailable
            }
            execution::NativeFailure::UnsupportedCapability => ErrorKind::UnsupportedCapability,
            execution::NativeFailure::AuthorityUnavailable => ErrorKind::StateConflict,
            execution::NativeFailure::TransportLost => ErrorKind::SessionLost,
            _ => ErrorKind::ParseFailure,
        }
    } else {
        ErrorKind::StateConflict
    };
    super::error(
        kind,
        "managed native operation unavailable; inspect scoped status",
    )
}
fn converted(status: NativeStatus) -> SessionStatus {
    SessionStatus {
        session: status.session.clone(),
        exit_code: None,
        stdout: vec![],
        stderr: vec![],
        stdout_truncated: false,
        stderr_truncated: false,
        failure: status.failure.map(|f| f.diagnostic().to_owned()),
        execution: Some(status),
    }
}
impl NativeAdapter {
    fn handle(&self, reference: &SessionRef) -> AdapterResult<ManagedSessionRef> {
        let handle = reference.execution.as_ref().ok_or_else(|| {
            error(
                ErrorKind::OwnershipMismatch,
                "native status requires exact execution identity",
            )
        })?;
        if handle.session != reference.id || handle.scope != reference.scope {
            return Err(error(
                ErrorKind::OwnershipMismatch,
                "foreign native Session reference",
            ));
        }
        let status = self.sessions.status(handle).map_err(mapped)?;
        if status.session.agent != self.name || status.session.provider != self.provider {
            return Err(error(
                ErrorKind::OwnershipMismatch,
                "foreign native adapter identity",
            ));
        }
        Ok(handle.clone())
    }
}
impl AgentAdapter for NativeAdapter {
    fn capabilities(&self) -> BTreeSet<Capability> {
        BTreeSet::from([
            Capability::Execute,
            Capability::Review,
            Capability::NonInteractive,
            Capability::PermissionInterception,
            Capability::UsageTelemetry,
        ])
    }
    fn probe(&self) -> AdapterResult<AgentInfo> {
        Ok(AgentInfo {
            agent: self.name.clone(),
            provider: self.provider.clone(),
            adapter_version: "result-profile-candidate-1".into(),
            executable: self.program.clone(),
            authenticated: None,
            model_configuration: true,
            effort_configuration: true,
            capabilities: self.capabilities(),
        })
    }
    fn managed_provider(&self) -> Option<&str> {
        Some(&self.provider)
    }
    fn start(&self, _request: LaunchRequest) -> AdapterFuture<'_, Session> {
        Box::pin(async {
            Err(error(
                ErrorKind::OwnershipMismatch,
                "native launch requires an admitted execution unit",
            ))
        })
    }
    fn start_managed(
        &self,
        request: LaunchRequest,
        input: ManagedInput,
    ) -> AdapterFuture<'_, NativeStart> {
        Box::pin(async move {
            if request.mode != LaunchMode::NonInteractive
                || !request.environment.is_empty()
                || input.agent != self.name
                || request.scope != input.authority.scope
                || request.input.scope != input.input.scope
                || request.input.kind != input.input.kind
                || request.input.revision != input.input.revision
                || request.input.version != input.input.version
                || request.input.source_versions != input.input.source_versions
                || request.input.payload != input.input.payload
            {
                return Err(error(
                    ErrorKind::InvalidInput,
                    "managed launch identity/environment mismatch",
                ));
            }
            {
                let store = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| error(ErrorKind::StateFailure, "state poisoned"))?;
                let unit = store
                    .validate_execution(&input.authority, true, false)
                    .map_err(mapped)?;
                let project = store
                    .project(unit.scope.project_id)
                    .map_err(mapped)?
                    .ok_or_else(|| error(ErrorKind::StateFailure, "Project missing"))?;
                let role = match unit.kind {
                    execution::UnitKind::Executor => SessionRole::Executor,
                    execution::UnitKind::Reviewer => SessionRole::Reviewer,
                    execution::UnitKind::Verifier => SessionRole::Consultant,
                    _ => {
                        return Err(error(
                            ErrorKind::InvalidInput,
                            "legacy native unit unavailable",
                        ));
                    }
                };
                if unit.provider != self.provider
                    || unit.worktree != request.worktree
                    || request.role != role
                    || input.input.kind
                        != if unit.kind == execution::UnitKind::Executor {
                            InputKind::ContextPack
                        } else {
                            InputKind::ReviewBundle
                        }
                    || serde_json::to_value(&project).ok()
                        != serde_json::to_value(&request.project).ok()
                {
                    return Err(error(
                        ErrorKind::OwnershipMismatch,
                        "managed launch Project/path/role mismatch",
                    ));
                }
                let workflows = store
                    .records(&unit.scope, RecordKind::Workflow)
                    .map_err(mapped)?;
                if !workflows.is_empty() {
                    if workflows.len() != 1 {
                        return Err(error(
                            ErrorKind::StateConflict,
                            "managed Workflow identity unavailable",
                        ));
                    }
                    let workflow: crate::workflow::WorkflowSnapshot =
                        serde_json::from_value(workflows[0].data.clone()).map_err(|_| {
                            error(ErrorKind::StateFailure, "managed Workflow invalid")
                        })?;
                    let attempt = workflow
                        .active
                        .and_then(|i| workflow.history.get(i))
                        .ok_or_else(|| {
                            error(
                                ErrorKind::StateConflict,
                                "managed Workflow reservation missing",
                            )
                        })?;
                    let context = store
                        .context(&unit.scope, Some(input.input.version))
                        .map_err(mapped)?
                        .ok_or_else(|| {
                            error(
                                ErrorKind::StateConflict,
                                "managed immutable context missing",
                            )
                        })?;
                    if attempt.unit.as_ref() != Some(&execution::ManagedUnitRef::from(&unit))
                        || attempt.agent.as_deref() != Some(&self.name)
                        || attempt.phase.key() != unit.phase
                        || !attempt.dispatch_started
                        || attempt.state != crate::workflow::AttemptState::Running
                        || attempt.session_id.is_some()
                        || attempt.context_version != context.version
                        || context.revision != input.input.revision
                        || context.source_hashes != input.input.source_versions
                        || serde_json::to_string(&context.data).ok().as_ref()
                            != Some(&input.input.payload)
                    {
                        return Err(error(
                            ErrorKind::OwnershipMismatch,
                            "managed input differs from reserved immutable ContextVersion",
                        ));
                    }
                }
            }
            self.sessions
                .start_inner(
                    input,
                    request.model,
                    request.effort,
                    Some(self.program.clone()),
                )
                .await
                .map_err(mapped)
        })
    }
    fn status(&self, reference: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        Box::pin(async move {
            let handle = self.handle(&reference)?;
            Ok(converted(self.sessions.status(&handle).map_err(mapped)?))
        })
    }
    fn transport_succeeded(&self, status: &SessionStatus) -> bool {
        let Some(execution) = &status.execution else {
            return false;
        };
        if execution.session.agent != self.name
            || execution.session.provider != self.provider
            || status.session.state != SessionState::Exited
            || execution.work != Some(WorkOutcome::Success)
            || status.failure.is_some()
        {
            return false;
        }
        self.sessions.status(&execution.handle).is_ok_and(|saved| {
            saved.work == Some(WorkOutcome::Success)
                && serde_json::to_value(&saved.session).ok()
                    == serde_json::to_value(&status.session).ok()
        })
    }
    fn stop(&self, reference: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        Box::pin(async move {
            let handle = self.handle(&reference)?;
            let mut updates = self.sessions.subscribe(&handle).map_err(mapped)?;
            self.sessions.cancel(&handle).await.map_err(mapped)?;
            tokio::time::timeout(Duration::from_secs(15), async {
                loop {
                    let status = converted(self.sessions.status(&handle).map_err(mapped)?);
                    if status.terminal() {
                        return Ok(status);
                    }
                    updates.changed().await.map_err(|_| {
                        error(ErrorKind::SessionLost, "native supervisor unavailable")
                    })?;
                }
            })
            .await
            .map_err(|_| error(ErrorKind::Timeout, "native stop observation incomplete"))?
        })
    }
    fn attach(&self, _reference: SessionRef) -> AdapterFuture<'_, ()> {
        Box::pin(async { Err(unsupported(Capability::Attach)) })
    }
    fn resume(&self, _reference: SessionRef) -> AdapterFuture<'_, Session> {
        Box::pin(async { Err(unsupported(Capability::Resume)) })
    }
    fn release(&self, reference: SessionRef) -> AdapterResult<()> {
        let handle = self.handle(&reference)?;
        self.sessions.release(&handle).map_err(mapped)
    }
    fn subscribe(&self, reference: SessionRef) -> AdapterResult<watch::Receiver<SessionStatus>> {
        let handle = self.handle(&reference)?;
        let mut source = self.sessions.subscribe(&handle).map_err(mapped)?;
        let (send, receive) = watch::channel(converted(source.borrow().clone()));
        tokio::spawn(async move {
            while source.changed().await.is_ok() {
                if send.send(converted(source.borrow().clone())).is_err() {
                    break;
                }
            }
        });
        Ok(receive)
    }
    fn usage(
        &self,
        reference: SessionRef,
        phase: String,
        review_round: Option<u32>,
    ) -> AdapterFuture<'_, Usage> {
        Box::pin(async move {
            let handle = self.handle(&reference)?;
            let status = self.sessions.status(&handle).map_err(mapped)?;
            let metrics = status.metrics.as_ref();
            let counter = |key: &str| metrics.and_then(|m| m[key].as_u64());
            Ok(Usage {
                scope: status.session.scope.clone(),
                session_id: status.session.id,
                agent: self.name.clone(),
                phase,
                review_round,
                input_tokens: counter("input"),
                cached_input_tokens: counter("cache_read"),
                output_tokens: counter("output"),
                estimated_cost: metrics.and_then(|m| m["cost"].as_f64()),
                context_pack_version: None,
                context_pack_size: None,
                repo_map_size: None,
                cache_metadata: Value::Null,
                missing_reason: metrics.is_none().then(|| {
                    "native counters not observed; subscription capacity is separate".into()
                }),
            })
        })
    }
    fn pending_approvals(&self, reference: SessionRef) -> AdapterFuture<'_, Value> {
        Box::pin(async move {
            let handle = self.handle(&reference)?;
            Ok(json!(
                self.sessions.status(&handle).map_err(mapped)?.pending
            ))
        })
    }
    fn submit_approval(&self, reference: SessionRef, decision: Value) -> AdapterFuture<'_, ()> {
        Box::pin(async move {
            let handle = self.handle(&reference)?;
            let authority = serde_json::from_value(decision["authority"].clone())
                .map_err(|_| error(ErrorKind::InvalidInput, "approval authority missing"))?;
            let id = decision["id"].clone();
            let hash = decision["operation_hash"]
                .as_str()
                .ok_or_else(|| error(ErrorKind::InvalidInput, "approval digest missing"))?
                .to_owned();
            let allow = decision["allow"]
                .as_bool()
                .ok_or_else(|| error(ErrorKind::InvalidInput, "approval decision missing"))?;
            self.sessions
                .approve(&handle, authority, id, hash, allow)
                .await
                .map_err(mapped)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::{attempts::AttemptManager, native::tests as fixture, results};
    #[tokio::test]
    async fn managed_adapter_routes_aliases_and_observes_work_independently_of_cleanup() {
        for (provider, success) in [
            ("claude", false),
            ("codex", false),
            ("claude", true),
            ("codex", true),
        ] {
            let (dir, owner, task) = results::tests::fixture().await;
            let sessions = Arc::new(NativeSessions::new(owner.clone()).unwrap());
            let adapter = NativeAdapter {
                owner: owner.clone(),
                name: "configured-executor".into(),
                provider: provider.into(),
                program: fixture::program(dir.path(), provider),
                sessions,
            };
            let (unit, _) = AttemptManager::new(owner.clone())
                .prepare(task.id, provider, "implement", None)
                .await
                .unwrap();
            let mut input = fixture::input(&unit, "ordinary-failure");
            input.agent = adapter.name.clone();
            // Both providers can finish this account-free owned error terminal.
            if success {
                input.input.payload = "capacity-retry-success".into();
            } else if provider == "claude" {
                input.input.payload = "quota-foreign".into();
            }
            let request = LaunchRequest {
                project: owner
                    .store
                    .lock()
                    .unwrap()
                    .project(task.project_id)
                    .unwrap()
                    .unwrap(),
                scope: unit.scope.clone(),
                worktree: unit.worktree.clone(),
                role: SessionRole::Executor,
                mode: LaunchMode::NonInteractive,
                input: input.input.clone(),
                environment: BTreeMap::new(),
                model: None,
                effort: None,
            };
            assert_eq!(
                adapter.start(request.clone()).await.unwrap_err().kind,
                ErrorKind::OwnershipMismatch
            );
            let mut foreign = request.clone();
            foreign.role = SessionRole::Reviewer;
            assert!(adapter.start_managed(foreign, input.clone()).await.is_err());
            assert!(
                owner
                    .store
                    .lock()
                    .unwrap()
                    .managed_effects(unit.id)
                    .unwrap()
                    .iter()
                    .all(|e| e.kind != "native_version")
            );
            let NativeStart::Launched(handle) =
                adapter.start_managed(request, input).await.unwrap()
            else {
                panic!("fixture queued")
            };
            let reference = SessionRef {
                id: handle.session,
                scope: handle.scope.clone(),
                execution: Some(handle.clone()),
            };
            assert!(
                adapter
                    .status(SessionRef {
                        execution: None,
                        ..reference.clone()
                    })
                    .await
                    .is_err()
            );
            let mut updates = adapter.subscribe(reference.clone()).unwrap();
            let status = tokio::time::timeout(Duration::from_secs(15), async {
                loop {
                    let status = adapter.status(reference.clone()).await.unwrap();
                    if status.terminal() {
                        break status;
                    }
                    updates.changed().await.unwrap();
                }
            })
            .await
            .unwrap();
            assert_eq!(status.session.agent, "configured-executor");
            assert_eq!(status.session.provider, provider);
            let native = status.execution.as_ref().unwrap();
            assert_eq!(
                native.work,
                Some(if success {
                    WorkOutcome::Success
                } else {
                    WorkOutcome::Failure
                })
            );
            assert_eq!(native.cleanup, execution::CleanupOutcome::Unknown);
            assert_eq!(status.exit_code, None);
            assert_eq!(adapter.transport_succeeded(&status), success);
            if !success {
                let mut forged = status.clone();
                forged.execution.as_mut().unwrap().work = Some(WorkOutcome::Success);
                assert!(!adapter.transport_succeeded(&forged));
            }
            adapter.release(reference).unwrap();
        }
    }
    #[tokio::test]
    async fn managed_adapter_cancel_closes_only_its_exact_session() {
        let (dir, owner, task) = results::tests::fixture().await;
        let sessions = Arc::new(NativeSessions::new(owner.clone()).unwrap());
        let adapter = NativeAdapter {
            owner: owner.clone(),
            name: "claude".into(),
            provider: "claude".into(),
            program: fixture::program(dir.path(), "claude"),
            sessions,
        };
        let (unit, _) = AttemptManager::new(owner.clone())
            .prepare(task.id, "claude", "implement", None)
            .await
            .unwrap();
        let input = fixture::input(&unit, "hold");
        let request = LaunchRequest {
            project: owner
                .store
                .lock()
                .unwrap()
                .project(task.project_id)
                .unwrap()
                .unwrap(),
            scope: unit.scope.clone(),
            worktree: unit.worktree.clone(),
            role: SessionRole::Executor,
            mode: LaunchMode::NonInteractive,
            input: input.input.clone(),
            environment: BTreeMap::new(),
            model: None,
            effort: None,
        };
        let NativeStart::Launched(handle) = adapter.start_managed(request, input).await.unwrap()
        else {
            panic!("fixture queued")
        };
        let reference = SessionRef {
            id: handle.session,
            scope: handle.scope.clone(),
            execution: Some(handle),
        };
        let mut foreign = reference.clone();
        foreign.execution.as_mut().unwrap().unit = execution::UnitId::new();
        assert!(adapter.stop(foreign).await.is_err());
        let status = adapter.stop(reference.clone()).await.unwrap();
        assert_eq!(status.session.state, SessionState::Stopped);
        let native = status.execution.unwrap();
        assert_eq!(native.work, Some(WorkOutcome::Unknown));
        assert_eq!(native.disposition, execution::Disposition::Cancelled);
        assert_eq!(native.cleanup, execution::CleanupOutcome::Unknown);
        adapter.release(reference).unwrap();
    }
}
