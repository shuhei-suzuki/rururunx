//! The sole retained nongrant command, then SAME-custody prepared conjunction.
use super::*;

pub(crate) struct PreparedPhaseNoCurrentDispatch {
    custody: Weak<NativePreparationCustody>,
    pub(crate) completion: Arc<version::NativeReadonlyHelperCompletion>,
    pub(crate) issued: Arc<crate::state::NativeReadyLineage>,
}
impl PreparedPhaseNoCurrentDispatch {
    pub(crate) fn validate_original(&self, actor: &Arc<NativePreparationActor>) -> Result<()> {
        actor.validate_original()?;
        let custody = self
            .custody
            .upgrade()
            .context("no-dispatch custody ended")?;
        ensure!(
            self.completion.matches_actor(actor) && custody.no_dispatch_matches(self),
            "no-dispatch value differs from SAME custody"
        );
        Ok(())
    }
}
impl NativeSessions {
    pub(super) async fn close_prepared_on_revocation(
        &self,
        custody: &Arc<NativePreparationCustody>,
    ) -> Result<()> {
        let mut replans = 0u8;
        let mut backoff = 100u64;
        loop {
            let (actor, no_dispatch, lineage, saved) = custody.closure_original()?;
            // A retained uncertain plan is confirmed before any fresh factual
            // snapshot. No later row can refresh a grant lineage.
            if let Some(saved) = saved {
                let confirmed = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .confirm_phase_quota_closure(saved)?;
                if let crate::state::NativeQuotaClosureConfirmation::Known(known) = confirmed {
                    custody.retain_closed(known)?;
                    actor.release_gate();
                    return Ok(());
                }
            }
            let plan = crate::state::Store::plan_phase_quota_closure(
                &self.owner,
                actor.clone(),
                no_dispatch,
                lineage,
                now_ms(),
            )?;
            custody.retain_closure_plan(plan.clone())?;
            let result = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .close_phase_quota(plan.clone());
            match result {
                Ok(Some(known)) => {
                    custody.retain_closed(known)?;
                    actor.release_gate();
                    return Ok(());
                }
                Ok(None) => {
                    custody.clear_definitive_closure_conflict(&plan)?;
                }
                Err(error) => {
                    match self
                        .owner
                        .store
                        .lock()
                        .map_err(|_| anyhow::anyhow!("state poisoned"))?
                        .confirm_phase_quota_closure(plan)
                    {
                        Ok(crate::state::NativeQuotaClosureConfirmation::Known(known)) => {
                            custody.retain_closed(known)?;
                            actor.release_gate();
                            return Ok(());
                        }
                        Ok(crate::state::NativeQuotaClosureConfirmation::RolledBack) => {
                            return Err(error);
                        }
                        Err(error) => {
                            return Err(
                                error.context("SAME nongrant closure commit uncertain; Held")
                            );
                        }
                    }
                }
            }
            replans = replans.saturating_add(1);
            if replans >= 8 {
                tokio::time::sleep(Duration::from_millis(backoff)).await;
                replans = 0;
                backoff = backoff.saturating_mul(2).min(5000);
            }
        }
    }
    pub(super) async fn prepare_phase_quota(
        &self,
        custody: &Arc<NativePreparationCustody>,
    ) -> Result<Arc<crate::state::NativeQuotaAdmitted>> {
        let mut conflicts = 0u8;
        let mut backoff = 100u64;
        loop {
            let (actor, no_dispatch, lineage) = custody.quota_original()?;
            actor.validate_open()?;
            let launch = actor.launch().clone();
            let scheduler = self
                .limits
                .scheduler(self.owner.clone(), launch.allocation().facts().provider);
            let caps = crate::state::NativeQuotaCaps {
                global: scheduler.global_total,
                executor: scheduler.provider_executor,
                provider: scheduler.provider_total,
                project: scheduler.project_tasks,
            };
            let plan = crate::state::Store::plan_phase_quota(
                &self.owner,
                actor.clone(),
                no_dispatch,
                lineage,
                caps,
                now_ms(),
            )?;
            custody.retain_quota_plan(plan.clone())?;
            let admission = launch.admission().enter(launch.clone()).await?;
            let outcome = {
                let mut store = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?;
                match store.commit_phase_quota(plan.clone(), &admission) {
                    Ok(crate::state::NativeQuotaWrite::Known(outcome)) => Some(outcome),
                    Ok(crate::state::NativeQuotaWrite::Conflict) => None,
                    Ok(crate::state::NativeQuotaWrite::Unresolved(cause)) => {
                        custody.report_unresolved_head(&cause);
                        None
                    }
                    Err(error) => match store.confirm_phase_quota(plan, &admission) {
                        Ok(crate::state::NativeQuotaConfirmation::Known(outcome)) => Some(outcome),
                        Ok(crate::state::NativeQuotaConfirmation::RolledBack) => return Err(error),
                        Err(confirm) => {
                            return Err(confirm.context("same private quota commit remains Held"));
                        }
                    },
                }
            };
            drop(admission);
            if let Some(outcome) = outcome {
                custody.retain_quota_outcome(&outcome)?;
                conflicts = 0;
                backoff = 100;
                match outcome {
                    crate::state::NativeQuotaOutcome::Admitted(value) => return Ok(value),
                    crate::state::NativeQuotaOutcome::Parked(value) => {
                        let now = now_ms();
                        let wake = if value.reason() == WaitReason::Capacity {
                            value.due().min(now.saturating_add(1000))
                        } else {
                            value.due()
                        };
                        let delay =
                            Duration::from_millis(u64::try_from(wake.saturating_sub(now).max(1))?);
                        tokio::select! { _=tokio::time::sleep(delay)=>{}, _=custody.revocation()=>{ self.close_prepared_on_revocation(custody).await?;anyhow::bail!("same parked Native operation closed; Root typed non-success closure unavailable") } }
                    }
                }
            } else {
                conflicts = conflicts.saturating_add(1);
                if conflicts >= 8 {
                    tokio::select! { _=tokio::time::sleep(Duration::from_millis(backoff))=>{}, _=custody.revocation()=>{self.close_prepared_on_revocation(custody).await?;anyhow::bail!("same quota operation closed; Root typed non-success closure unavailable") } }
                    conflicts = 0;
                    backoff = backoff.saturating_mul(2).min(5000);
                }
            }
        }
    }
    pub(super) async fn issue_no_current_dispatch(
        &self,
        custody: &Arc<NativePreparationCustody>,
    ) -> Result<Arc<PreparedPhaseNoCurrentDispatch>> {
        let (actor, known, completion) = custody.no_dispatch_original()?;
        actor.validate_open()?;
        ensure!(
            completion.matches_actor(&actor),
            "no-dispatch completion original differs"
        );
        let lineage = known.initial_lineage();
        let launch = actor.launch().clone();
        let admission = launch.admission().enter(launch.clone()).await?;
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .check_phase_no_current_dispatch(&actor, &lineage, &completion, &admission)?;
        drop(admission);
        let value = Arc::new(PreparedPhaseNoCurrentDispatch {
            custody: Arc::downgrade(custody),
            completion,
            issued: lineage,
        });
        custody.retain_no_dispatch(value.clone())?;
        Ok(value)
    }
}

pub(crate) struct PreparedNativePhase {
    pub(super) actor: Arc<NativePreparationActor>,
    pub(super) known: Arc<crate::state::NativePreparationCommit>,
    pub(super) version: Arc<version::NativeVersionHelperCustody>,
    pub(super) completion: Arc<version::NativeReadonlyHelperCompletion>,
    pub(super) compat: Arc<compat::NativeCompatQualification>,
    pub(super) command: Arc<NativeTransportCommand>,
    pub(super) quota: Arc<crate::state::NativeQuotaAdmitted>,
}
impl NativeSessions {
    pub(super) fn issue_prepared(
        &self,
        custody: &Arc<NativePreparationCustody>,
    ) -> Result<Arc<PreparedNativePhase>> {
        let value = custody.prepared_original()?;
        value.actor.validate_open()?;
        let version = value.version.closed()?;
        ensure!(
            value.completion.matches_actor(&value.actor)
                && value
                    .completion
                    .commit
                    .matches_prefix(&value.known, &version)
                && value.compat.matches(&value.actor, &value.version),
            "prepared SAME closed helper prefix/compatibility differs"
        );
        value.compat.check_command(&value.command)?;
        value.quota.lineage().validate_prepared_shape()?;
        let facts = value.actor.launch().allocation().facts();
        let budget = super::native_effect_budget(facts.provider, facts.role)?;
        budget.check_prepared(value.completion.commit.len())?;
        custody.retain_prepared(Arc::new(value))
    }
}

pub(super) struct NativeTransportCommand {
    pub(super) program: std::path::PathBuf,
    pub(super) cwd: std::path::PathBuf,
    pub(super) argv: Vec<String>,
    pub(super) environment: BTreeMap<String, String>,
    pub(super) native_session: Option<uuid::Uuid>,
}
pub(super) fn command_argv(
    provider: &str,
    role: SessionRole,
    model: Option<&str>,
    effort: Option<&str>,
    session: Option<uuid::Uuid>,
) -> Result<Vec<String>> {
    ensure!(
        matches!(role, SessionRole::Executor | SessionRole::Reviewer),
        "native command role unsupported"
    );
    for value in [model, effort].into_iter().flatten() {
        ensure!(
            !value.is_empty()
                && value.len() <= 128
                && !value.starts_with('-')
                && !value.chars().any(char::is_control),
            "native option value unsupported"
        );
    }
    let argv = match provider {
        "codex" => {
            ensure!(session.is_none(), "unexpected Codex native UUID");
            vec!["app-server".into(), "--listen".into(), "stdio://".into()]
        }
        "claude" => {
            let mut argv: Vec<String> = [
                "-p",
                "--input-format",
                "stream-json",
                "--output-format",
                "stream-json",
                "--verbose",
                "--session-id",
            ]
            .map(String::from)
            .into();
            argv.push(session.context("Claude native UUID absent")?.to_string());
            argv.extend(
                [
                    "--permission-prompt-tool",
                    "stdio",
                    "--settings",
                    "{\"forceLoginMethod\":\"claudeai\"}",
                ]
                .map(String::from),
            );
            for (flag, value) in [("--model", model), ("--effort", effort)] {
                if let Some(value) = value {
                    argv.extend([flag.into(), value.into()]);
                }
            }
            if role != SessionRole::Executor {
                argv.extend(["--permission-mode".into(), "plan".into()]);
            }
            argv
        }
        _ => anyhow::bail!("native command provider unsupported"),
    };
    ensure!(
        argv.len() <= 18 && argv.iter().all(|a| a.len() <= 128),
        "native command argv exceeds bounded profile"
    );
    Ok(argv)
}
pub(super) fn plan_native_command(
    owner: &Arc<RuntimeOwner>,
    actor: &Arc<NativePreparationActor>,
    completion: &Arc<version::NativeReadonlyHelperCompletion>,
    compat: &Arc<compat::NativeCompatQualification>,
) -> Result<Arc<NativeTransportCommand>> {
    actor.validate_open()?;
    ensure!(
        completion.matches_actor(actor),
        "command requires SAME closed helper completion"
    );
    let profile = version::qualified_physical_profile(owner, actor)?;
    let f = actor.launch().allocation().facts();
    let native_session = (f.provider == "claude").then(uuid::Uuid::new_v4);
    let command = Arc::new(NativeTransportCommand {
        program: f.program.into(),
        cwd: f.path.into(),
        argv: command_argv(f.provider, f.role, f.model, f.effort, native_session)?,
        environment: profile.environment(
            &actor.launch().allocation().unit_snapshot().cookie,
            &owner.socket,
        )?,
        native_session,
    });
    ensure!(serde_json::to_vec(&json!({"program":command.program,"cwd":command.cwd,"argv":command.argv,"environment":command.environment})).map(|b|b.len())? <= 64*1024, "encoded native command exceeds bound");
    compat.check_command(&command)?;
    Ok(command)
}

#[cfg(test)]
mod primitive_tests {
    use super::*;
    #[test]
    fn nongrant_native_command_keeps_exact_permission_vector_and_bounded_values() {
        let session = Some(uuid::Uuid::new_v4());
        let executor = command_argv(
            "claude",
            SessionRole::Executor,
            Some("model"),
            Some("high"),
            session,
        )
        .unwrap();
        let reviewer = command_argv(
            "claude",
            SessionRole::Reviewer,
            Some("model"),
            Some("high"),
            session,
        )
        .unwrap();
        assert_eq!(executor.len(), 16);
        assert_eq!(reviewer.len(), 18);
        assert_eq!(&reviewer[..16], &executor);
        assert_eq!(&reviewer[16..], ["--permission-mode", "plan"]);
        assert_eq!(
            &executor[8..12],
            [
                "--permission-prompt-tool",
                "stdio",
                "--settings",
                "{\"forceLoginMethod\":\"claudeai\"}"
            ]
        );
        assert_eq!(
            command_argv("codex", SessionRole::Reviewer, None, None, None).unwrap(),
            ["app-server", "--listen", "stdio://"]
        );
        for value in ["", "--bare", "line\nbreak"] {
            assert!(
                command_argv("claude", SessionRole::Executor, Some(value), None, session).is_err()
            );
        }
        assert!(
            command_argv(
                "claude",
                SessionRole::Executor,
                Some(&"x".repeat(128)),
                None,
                session
            )
            .is_ok()
        );
        assert!(
            command_argv(
                "claude",
                SessionRole::Executor,
                Some(&"x".repeat(129)),
                None,
                session
            )
            .is_err()
        );
    }
    #[test]
    fn nongrant_native_codex_pin_allows_only_single_line_ending() {
        for text in [
            "codex-cli 0.160.0",
            "codex-cli 0.160.0\n",
            "codex-cli 0.160.0\r\n",
        ] {
            assert!(version::qualified_codex_phase_version(text));
        }
        for text in [
            "codex-cli 0.160.1",
            "codex-cli 0.160.0 \n",
            " codex-cli 0.160.0",
            "codex-cli 0.160.0\n\n",
            "codex-cli 0.160.0\r",
            "codex-cli 0.160.0\ntrailer",
        ] {
            assert!(!version::qualified_codex_phase_version(text), "{text:?}");
        }
    }
}
