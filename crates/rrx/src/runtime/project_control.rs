//! S3 (R3.4): Project operations through the control API (HOW §3.5).
//!
//! Every operation is snapshot → bounded preflight → commit (D4). The
//! preflight holds neither the Store lock nor `control_admission`; its Git
//! children are owned process groups (D5). The commit re-checks currency in
//! the same Immediate transaction as the write (D6).
use super::{
    Runtime,
    control::{
        ControlAction, ControlRequest, ControlResponse, ProjectLookupFound, ProjectRow,
        ReconcileMark, UnavailableReason,
    },
};
use crate::{
    domain::{Project, ProjectId, ProjectState, RecordKind, Scope},
    git::PreflightGroups,
    project::{ProjectStatus, ProjectStatusView, plan_add, plan_block},
};
use anyhow::{Result, ensure};
use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

/// D5: the deadline of one Project preflight.
pub(super) const PREFLIGHT_DEADLINE: Duration = Duration::from_secs(10);
/// D5: how long a cut-off preflight waits for its killed children's reap.
const REAP_BOUND: Duration = Duration::from_secs(2);
/// Bounds a `ProjectRefused` message.
const REFUSAL_BYTES: usize = 2048;

/// The outcome of one bounded preflight.
pub(super) enum Preflight<T> {
    Done(T),
    /// Another request's preflight for this Project has not joined.
    InFlight,
    /// The deadline passed or the Runtime started stopping.
    Unavailable,
}

/// D6: a typed currency refusal raised inside the commit transaction.
#[derive(Debug)]
struct CurrencyChanged;
impl std::fmt::Display for CurrencyChanged {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Project control currency changed")
    }
}
impl std::error::Error for CurrencyChanged {}
#[derive(Debug)]
struct InFlight;
impl std::fmt::Display for InFlight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Project preflight in flight")
    }
}
impl std::error::Error for InFlight {}

fn unavailable(request: &ControlRequest, reason: UnavailableReason) -> ControlResponse {
    ControlResponse::Unavailable {
        request_id: request.request_id,
        reason,
    }
}
fn refused(error: &anyhow::Error) -> ControlResponse {
    let mut reason = format!("{error:#}");
    if reason.len() > REFUSAL_BYTES {
        let mut end = REFUSAL_BYTES;
        while !reason.is_char_boundary(end) {
            end -= 1;
        }
        reason.truncate(end);
    }
    ControlResponse::ProjectRefused { reason }
}
/// Maps a commit error to its typed answer.
fn commit_refusal(request: &ControlRequest, error: &anyhow::Error) -> ControlResponse {
    if error.chain().any(|e| e.is::<CurrencyChanged>()) || crate::state::snapshot_changed(error) {
        unavailable(request, UnavailableReason::ProjectCurrencyChanged)
    } else if error.chain().any(|e| e.is::<InFlight>()) {
        unavailable(request, UnavailableReason::ProjectPreflightInFlight)
    } else {
        refused(error)
    }
}

impl Runtime {
    /// D5: runs `work` in `spawn_blocking` with its Git children owned by a
    /// fresh `PreflightGroups`, bounded by the deadline and by `stopping`.
    /// `project` (when known) is held in the in-flight set until the task
    /// has joined, which is after every Git child it started was reaped.
    pub(super) async fn project_preflight<T: Send + 'static>(
        &self,
        project: Option<ProjectId>,
        work: impl FnOnce() -> Result<T> + Send + 'static,
    ) -> Preflight<Result<T>> {
        if let Some(id) = project {
            let mut flights = self
                .project_preflights
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if !flights.insert(id) {
                return Preflight::InFlight;
            }
        }
        let groups = Arc::new(PreflightGroups::default());
        let task_groups = groups.clone();
        let mut handle = tokio::task::spawn_blocking(move || {
            crate::git::with_preflight_groups(task_groups, work)
        });
        let deadline = tokio::time::Instant::now() + PREFLIGHT_DEADLINE;
        let finished = loop {
            tokio::select! {
                joined = &mut handle => break Some(joined),
                () = tokio::time::sleep_until(deadline) => break None,
                () = tokio::time::sleep(Duration::from_millis(50)) => {
                    if self.stopping.load(Ordering::SeqCst) {
                        break None;
                    }
                }
            }
        };
        let release = move |flights: &std::sync::Mutex<std::collections::BTreeSet<ProjectId>>| {
            if let Some(id) = project {
                flights
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .remove(&id);
            }
        };
        match finished {
            Some(joined) => {
                release(&self.project_preflights);
                Preflight::Done(joined.unwrap_or_else(|e| Err(anyhow::anyhow!(e))))
            }
            None => {
                // Kill every owned group; the blocking task reaps its own
                // child and only then joins, so the in-flight entry clears
                // after the reap and after the child's pipes closed.
                groups.cancel();
                // C-S3d4: every killed child is reaped before the response;
                // a child that does not die within the bound still keeps
                // the entry held until the join.
                let reap_deadline = tokio::time::Instant::now() + REAP_BOUND;
                while !groups.all_reaped() && tokio::time::Instant::now() < reap_deadline {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                let flights = self.project_preflights.clone();
                tokio::spawn(async move {
                    let _ = handle.await;
                    release(&flights);
                });
                Preflight::Unavailable
            }
        }
    }

    fn project_snapshot(&self) -> Result<Vec<Project>> {
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .projects()
    }
    fn project_in_flight(&self, id: ProjectId) -> bool {
        self.project_preflights
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains(&id)
    }

    /// D6: writes `project` after checking, in the same transaction, the
    /// owner epoch, `!stopping`, the in-flight set and `expected` (the
    /// version the current row must still have, or `None` for a new row).
    async fn commit_project(&self, project: &mut Project, expected: Option<u64>) -> Result<()> {
        let _admission = self.control_admission.lock().await;
        let (instance, epoch) = (self.owner.instance_id().to_owned(), self.owner.epoch());
        let stopping = self.stopping.clone();
        let in_flight = self.project_in_flight(project.id);
        let planned = project.clone();
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .put_project_checked(project, |tx, current| {
                let owner: (String, u64) = tx.query_row(
                    "SELECT instance_id,epoch FROM runtime_epoch WHERE singleton=1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?;
                if owner != (instance, epoch) || stopping.load(Ordering::SeqCst) {
                    return Err(CurrencyChanged.into());
                }
                if in_flight {
                    return Err(InFlight.into());
                }
                match (current, expected) {
                    (None, None) => Ok(()),
                    (Some(current), Some(expected))
                        if current.version == expected
                            && current.version == planned.version
                            && current.root == planned.root
                            && current.repository_identity == planned.repository_identity
                            && current.base_branch == planned.base_branch =>
                    {
                        Ok(())
                    }
                    _ => Err(CurrencyChanged.into()),
                }
            })
    }

    /// The S3 Project actions; `None` for every other action.
    pub(super) async fn handle_project(
        &self,
        request: &ControlRequest,
    ) -> Result<Option<ControlResponse>> {
        let response = match &request.action {
            ControlAction::ProjectLookupRoot { root } => {
                if !root.is_absolute() {
                    return Ok(Some(unavailable(
                        request,
                        UnavailableReason::ProjectPathNotAbsolute,
                    )));
                }
                let found = self
                    .project_snapshot()?
                    .into_iter()
                    .find(|p| p.root == *root)
                    .map(|p| ProjectLookupFound {
                        id: p.id,
                        version: p.version,
                        state: p.state,
                    });
                ControlResponse::ProjectLookup { found }
            }
            ControlAction::ProjectRegister { path, options } => {
                if !path.is_absolute() {
                    return Ok(Some(unavailable(
                        request,
                        UnavailableReason::ProjectPathNotAbsolute,
                    )));
                }
                let snapshot = self.project_snapshot()?;
                let (path, options, config) = (path.clone(), options.clone(), self.config.clone());
                let plan = match self
                    .project_preflight(None, move || {
                        plan_add(&snapshot, &path, options.into(), &config)
                    })
                    .await
                {
                    Preflight::Done(Ok(plan)) => plan,
                    Preflight::Done(Err(error)) => return Ok(Some(refused(&error))),
                    Preflight::InFlight => unreachable!("no Project key"),
                    Preflight::Unavailable => {
                        return Ok(Some(unavailable(
                            request,
                            UnavailableReason::ProjectPreflightUnavailable,
                        )));
                    }
                };
                let (mut project, _) = plan;
                // A registered root is an update (D10), never a second row.
                if project.version != 0 {
                    return Ok(Some(unavailable(
                        request,
                        UnavailableReason::ProjectCurrencyChanged,
                    )));
                }
                match self.commit_project(&mut project, None).await {
                    Ok(()) => ControlResponse::ProjectSaved {
                        project,
                        changed: true,
                    },
                    Err(error) => commit_refusal(request, &error),
                }
            }
            ControlAction::ProjectUpdate {
                project: id,
                expected_project,
                options,
            } => {
                let snapshot = self.project_snapshot()?;
                let Some(current) = snapshot.iter().find(|p| p.id == *id).cloned() else {
                    return Ok(Some(refused(&anyhow::anyhow!("unknown project ID"))));
                };
                if current.version != *expected_project {
                    return Ok(Some(unavailable(
                        request,
                        UnavailableReason::ProjectCurrencyChanged,
                    )));
                }
                let (root, options, config) =
                    (current.root.clone(), options.clone(), self.config.clone());
                let (mut project, changed) = match self
                    .project_preflight(Some(*id), move || {
                        plan_add(&snapshot, &root, options.into(), &config)
                    })
                    .await
                {
                    Preflight::Done(Ok(plan)) => plan,
                    Preflight::Done(Err(error)) => return Ok(Some(refused(&error))),
                    Preflight::InFlight => {
                        return Ok(Some(unavailable(
                            request,
                            UnavailableReason::ProjectPreflightInFlight,
                        )));
                    }
                    Preflight::Unavailable => {
                        return Ok(Some(unavailable(
                            request,
                            UnavailableReason::ProjectPreflightUnavailable,
                        )));
                    }
                };
                ensure!(project.id == *id, "Project update planned another row");
                if !changed {
                    return Ok(Some(ControlResponse::ProjectSaved {
                        project,
                        changed: false,
                    }));
                }
                match self
                    .commit_project(&mut project, Some(*expected_project))
                    .await
                {
                    Ok(()) => ControlResponse::ProjectSaved {
                        project,
                        changed: true,
                    },
                    Err(error) => commit_refusal(request, &error),
                }
            }
            ControlAction::ProjectRemove {
                project: id,
                expected_project,
            } => {
                // D6: the expected version is checked before anything else,
                // so a stale remove does no Git work.
                let Some(mut project) = self.project_snapshot()?.into_iter().find(|p| p.id == *id)
                else {
                    return Ok(Some(refused(&anyhow::anyhow!("unknown project ID"))));
                };
                if project.version != *expected_project {
                    return Ok(Some(unavailable(
                        request,
                        UnavailableReason::ProjectCurrencyChanged,
                    )));
                }
                if project.state == ProjectState::Removed {
                    return Ok(Some(ControlResponse::ProjectSaved {
                        project,
                        changed: false,
                    }));
                }
                // §3.1: the reconcile runs after the version check. A
                // Blocked finding does not prevent the removal; only the
                // bound and the in-flight set are enforced here.
                let checked = project.clone();
                match self
                    .project_preflight(Some(*id), move || Ok(plan_block(&checked)))
                    .await
                {
                    Preflight::Done(_) => {}
                    Preflight::InFlight => {
                        return Ok(Some(unavailable(
                            request,
                            UnavailableReason::ProjectPreflightInFlight,
                        )));
                    }
                    Preflight::Unavailable => {
                        return Ok(Some(unavailable(
                            request,
                            UnavailableReason::ProjectPreflightUnavailable,
                        )));
                    }
                }
                project.state = ProjectState::Removed;
                match self
                    .commit_project(&mut project, Some(*expected_project))
                    .await
                {
                    Ok(()) => ControlResponse::ProjectSaved {
                        project,
                        changed: true,
                    },
                    Err(error) => commit_refusal(request, &error),
                }
            }
            ControlAction::ProjectList { all } => {
                // A missing executable is infrastructure failure, not a
                // Project fault: refuse the whole list, write nothing.
                let cwd = std::env::current_dir()?;
                match self
                    .project_preflight(None, move || {
                        crate::git::git_text(&cwd, &["--version"]).map(drop)
                    })
                    .await
                {
                    Preflight::Done(Ok(())) => {}
                    Preflight::Done(Err(error)) => return Ok(Some(refused(&error))),
                    Preflight::InFlight | Preflight::Unavailable => {
                        return Ok(Some(unavailable(
                            request,
                            UnavailableReason::ProjectPreflightUnavailable,
                        )));
                    }
                }
                let snapshot = self.project_snapshot()?;
                let marks = futures_util::future::join_all(
                    snapshot.iter().map(|project| self.reconcile_one(project)),
                )
                .await;
                let current = self.project_snapshot()?;
                let rows = current
                    .into_iter()
                    .filter(|p| *all || p.state != ProjectState::Removed)
                    .map(|project| {
                        let reconcile = snapshot
                            .iter()
                            .zip(&marks)
                            .find(|(p, _)| p.id == project.id)
                            .map(|(_, mark)| *mark)
                            .unwrap_or(ReconcileMark::NotRegistered);
                        ProjectRow { project, reconcile }
                    })
                    .collect();
                ControlResponse::ProjectRows { rows }
            }
            ControlAction::ProjectStatus { selector, cwd } => {
                if !cwd.is_absolute() {
                    return Ok(Some(unavailable(
                        request,
                        UnavailableReason::ProjectPathNotAbsolute,
                    )));
                }
                let project = match self.resolve_project_route(selector.as_deref(), cwd).await {
                    Ok(project) => project,
                    Err(error) => return Ok(Some(refused(&error))),
                };
                let reconcile = self.reconcile_one(&project).await;
                // C-S3g(iii): a status whose own check did not finish is a
                // typed refusal, never facts presented as current.
                if reconcile == ReconcileMark::Unavailable {
                    return Ok(Some(unavailable(
                        request,
                        UnavailableReason::ProjectPreflightUnavailable,
                    )));
                }
                ControlResponse::ProjectStatusFacts {
                    status: self.project_status_view(project.id)?,
                    reconcile,
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(response))
    }

    /// D8: reconciles one row. An in-flight Project is not waited on; a
    /// `Blocked` write that loses its snapshot CAS is skipped (Q2).
    async fn reconcile_one(&self, project: &Project) -> ReconcileMark {
        if project.state != ProjectState::Registered {
            return ReconcileMark::NotRegistered;
        }
        let checked = project.clone();
        match self
            .project_preflight(Some(project.id), move || Ok(plan_block(&checked)))
            .await
        {
            Preflight::InFlight => ReconcileMark::SkippedInFlight,
            Preflight::Unavailable => ReconcileMark::Unavailable,
            Preflight::Done(Err(_)) => ReconcileMark::Unavailable,
            Preflight::Done(Ok(None)) => ReconcileMark::Checked,
            Preflight::Done(Ok(Some(reason))) => {
                let mut blocked = project.clone();
                blocked.state = ProjectState::Blocked;
                blocked.blocked_reason = Some(reason);
                let expected = project.version;
                // Any refusal leaves the row as it now reads; the response
                // re-reads it and never presents the stale check as current.
                let _ = self.commit_project(&mut blocked, Some(expected)).await;
                ReconcileMark::Checked
            }
        }
    }

    /// D1 (M1): the non-secret status projection, read under one short lock.
    fn project_status_view(&self, id: ProjectId) -> Result<ProjectStatusView> {
        let store = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        let project = store
            .project(id)?
            .ok_or_else(|| anyhow::anyhow!("unknown project"))?;
        let status = ProjectStatus {
            goals: store.goals(id)?,
            tasks: store.tasks(id, None)?,
            sessions: store
                .records(&Scope::project(id), RecordKind::Session)?
                .into_iter()
                .map(|r| serde_json::from_value(r.data).map_err(Into::into))
                .collect::<Result<_>>()?,
            project,
        };
        Ok(ProjectStatusView::from(&status))
    }
}
