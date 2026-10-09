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
    project::{
        ProjectStatus, ProjectStatusView, plan_add, plan_block, public_project, public_reason,
    },
};
use anyhow::{Result, ensure};
use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

/// D5: the deadline of one Project preflight.
pub(super) const PREFLIGHT_DEADLINE: Duration = Duration::from_secs(10);

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
/// M4: a refusal carries only the non-secret, bounded reason.
fn refused(error: &anyhow::Error) -> ControlResponse {
    ControlResponse::ProjectRefused {
        reason: public_reason(&format!("{error:#}")),
    }
}
fn saved(project: Project, changed: bool) -> ControlResponse {
    ControlResponse::ProjectSaved {
        project: public_project(project),
        changed,
    }
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

/// D8: one row's mark and the version that mark is about.
#[derive(Clone, Copy)]
struct Reconciled {
    mark: ReconcileMark,
    version: u64,
}
impl Reconciled {
    /// M2: the mark for the row as it is reported, at `current`.
    fn for_version(self, current: u64) -> ReconcileMark {
        match self.mark {
            ReconcileMark::Checked | ReconcileMark::NotRegistered if current != self.version => {
                ReconcileMark::Changed
            }
            mark => mark,
        }
    }
}

// M1 causal control: pauses one Project's commit inside its transaction
// check, twice on one barrier (reached, then resume). It grants nothing and
// changes no outcome.
#[cfg(test)]
static COMMIT_PAUSE: std::sync::Mutex<Option<(ProjectId, Arc<std::sync::Barrier>)>> =
    std::sync::Mutex::new(None);
#[cfg(test)]
fn commit_pause(id: ProjectId) {
    let pause = {
        let mut slot = COMMIT_PAUSE.lock().unwrap_or_else(|e| e.into_inner());
        match slot.as_ref() {
            Some((target, _)) if *target == id => slot.take().map(|(_, barrier)| barrier),
            _ => None,
        }
    };
    if let Some(barrier) = pause {
        barrier.wait();
        barrier.wait();
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
        // M3 (Sol 6080740320): the entry is released only once every group
        // this preflight led is confirmed gone; a lingering group keeps the
        // Project in flight until then, however long that takes.
        let release =
            move |flights: Arc<std::sync::Mutex<std::collections::BTreeSet<ProjectId>>>,
                  groups: Arc<PreflightGroups>| async move {
                while !groups.all_reaped() {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                if let Some(id) = project {
                    flights
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .remove(&id);
                }
            };
        let flights = self.project_preflights.clone();
        match finished {
            Some(joined) => {
                if groups.all_reaped() {
                    release(flights, groups).await;
                } else {
                    tokio::spawn(release(flights, groups));
                }
                Preflight::Done(joined.unwrap_or_else(|e| Err(anyhow::anyhow!(e))))
            }
            None => {
                // Kill every owned group; the blocking task reaps its own
                // child and only then joins, so the in-flight entry clears
                // after the reap, after the child's pipes closed and after
                // its group is gone.
                groups.cancel_and_await_reap().await;
                tokio::spawn(async move {
                    let _ = handle.await;
                    release(flights, groups).await;
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

    /// D6: writes `project` after checking, in the same transaction, the
    /// owner epoch, `!stopping`, the in-flight set and `expected` (the
    /// version the current row must still have, or `None` for a new row).
    async fn commit_project(&self, project: &mut Project, expected: Option<u64>) -> Result<()> {
        let _admission = self.control_admission.lock().await;
        let (instance, epoch) = (self.owner.instance_id().to_owned(), self.owner.epoch());
        let stopping = self.stopping.clone();
        // M1: the in-flight set stays locked until the write has committed,
        // so no preflight for this Project can register between the check
        // and the write. Lock order: in-flight set, then Store.
        let flights = self
            .project_preflights
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let in_flight = flights.contains(&project.id);
        let planned = project.clone();
        let written = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .put_project_checked(project, |tx, current| {
                #[cfg(test)]
                commit_pause(planned.id);
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
            });
        drop(flights);
        written
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
                    Ok(()) => saved(project, true),
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
                    return Ok(Some(saved(project, false)));
                }
                match self
                    .commit_project(&mut project, Some(*expected_project))
                    .await
                {
                    Ok(()) => saved(project, true),
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
                    return Ok(Some(saved(project, false)));
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
                    Ok(()) => saved(project, true),
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
                        // M2: a row that appeared or changed after its check
                        // is reported without claiming that check.
                        let reconcile = snapshot
                            .iter()
                            .zip(&marks)
                            .find(|(p, _)| p.id == project.id)
                            .map_or(ReconcileMark::Changed, |(_, mark)| {
                                mark.for_version(project.version)
                            });
                        ProjectRow {
                            project: public_project(project),
                            reconcile,
                        }
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
                let checked = self.reconcile_one(&project).await;
                // C-S3g(iii): a status whose own check did not finish is a
                // typed refusal, never facts presented as current.
                if checked.mark == ReconcileMark::Unavailable {
                    return Ok(Some(unavailable(
                        request,
                        UnavailableReason::ProjectPreflightUnavailable,
                    )));
                }
                let status = self.project_status_view(project.id)?;
                ControlResponse::ProjectStatusFacts {
                    reconcile: checked.for_version(status.project.version),
                    status,
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(response))
    }

    /// D8: reconciles one row. An in-flight Project is not waited on; a
    /// `Blocked` write that loses its snapshot CAS is skipped (Q2).
    async fn reconcile_one(&self, project: &Project) -> Reconciled {
        let at = |mark, version| Reconciled { mark, version };
        if project.state != ProjectState::Registered {
            return at(ReconcileMark::NotRegistered, project.version);
        }
        let checked = project.clone();
        match self
            .project_preflight(Some(project.id), move || Ok(plan_block(&checked)))
            .await
        {
            Preflight::InFlight => at(ReconcileMark::SkippedInFlight, project.version),
            Preflight::Unavailable | Preflight::Done(Err(_)) => {
                at(ReconcileMark::Unavailable, project.version)
            }
            Preflight::Done(Ok(None)) => at(ReconcileMark::Checked, project.version),
            Preflight::Done(Ok(Some(reason))) => {
                let mut blocked = project.clone();
                blocked.state = ProjectState::Blocked;
                blocked.blocked_reason = Some(reason);
                let expected = project.version;
                // Any refusal leaves the row as it now reads; the response
                // re-reads it, and M2 reports a changed row as `changed`.
                match self.commit_project(&mut blocked, Some(expected)).await {
                    Ok(()) => at(ReconcileMark::Checked, blocked.version),
                    Err(_) => at(ReconcileMark::Checked, expected),
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, execution::RuntimeOwner};
    use std::time::Instant;

    fn runtime(dir: &std::path::Path) -> Runtime {
        let owner = RuntimeOwner::open(&dir.join("state.db")).unwrap();
        Runtime::new(owner, Config::default()).unwrap()
    }

    /// C-S3d4 (Sol 6080740320 L1): the Runtime's own cut-off answers only
    /// after the killed child is reaped. The reap is delayed by 500 ms and
    /// the cut-off comes from `stopping`. Mutant: the cut-off only cancels
    /// (no reap wait) → the child is still a zombie at the answer → FAIL.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn runtime_cut_off_answers_only_after_the_reap() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = runtime(dir.path());
        let pidfile = dir.path().join("pid");
        let script = format!("echo $$ > '{}'; sleep 30", pidfile.display());
        let work = move || {
            crate::git::set_reap_delay(Duration::from_millis(500));
            let mut child = std::process::Command::new("sh");
            child.args(["-c", &script]);
            crate::git::run_owned(child).map(drop)
        };
        let (stopping, watched) = (runtime.stopping.clone(), pidfile.clone());
        tokio::spawn(async move {
            while !watched.exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            stopping.store(true, Ordering::SeqCst);
        });
        let answer = runtime.project_preflight(None, work).await;
        assert!(matches!(answer, Preflight::Unavailable));
        let pid: i32 = std::fs::read_to_string(&pidfile)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert_eq!(
            rustix::process::test_kill_process(rustix::process::Pid::from_raw(pid).unwrap()),
            Err(rustix::io::Errno::SRCH),
            "answered before the reap"
        );
    }

    /// M3 (Sol 6080740320): a preflight whose Git group was not seen empty
    /// finishes, but its Project stays in flight until the group is gone.
    /// The lingering member is this test's own child joined to the group and
    /// left an unreaped zombie. Mutant: release without the group check →
    /// FAIL.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn lingering_group_keeps_the_project_in_flight() {
        use std::os::unix::process::CommandExt;
        let dir = tempfile::tempdir().unwrap();
        let runtime = runtime(dir.path());
        let (pidfile, go) = (dir.path().join("pid"), dir.path().join("go"));
        let script = format!(
            "echo $$ > '{}'; while [ ! -e '{}' ]; do sleep 0.01; done",
            pidfile.display(),
            go.display()
        );
        let work = move || {
            let mut child = std::process::Command::new("sh");
            child.args(["-c", &script]);
            crate::git::run_owned(child).map(drop)
        };
        let id = ProjectId::new();
        let joiner = {
            let (pidfile, go) = (pidfile.clone(), go.clone());
            tokio::task::spawn_blocking(move || {
                while !pidfile.exists() {
                    std::thread::sleep(Duration::from_millis(10));
                }
                let leader: i32 = std::fs::read_to_string(&pidfile)
                    .unwrap()
                    .trim()
                    .parse()
                    .unwrap();
                let member = std::process::Command::new("sleep")
                    .arg("30")
                    .process_group(leader)
                    .spawn()
                    .unwrap();
                std::fs::write(&go, "").unwrap();
                member
            })
        };
        let answer = runtime.project_preflight(Some(id), work).await;
        assert!(matches!(answer, Preflight::Done(Err(_))));
        let held = runtime.project_preflights.lock().unwrap().contains(&id);
        assert!(held, "a lingering group released its Project");
        let mut member = joiner.await.unwrap();
        tokio::task::spawn_blocking(move || member.wait().unwrap())
            .await
            .unwrap();
        let started = Instant::now();
        while runtime.project_preflights.lock().unwrap().contains(&id) {
            assert!(started.elapsed() < Duration::from_secs(5), "never released");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    /// M1 (Sol 6079932858, 6080740320 causal control): while a commit is
    /// between its in-flight check and its write, no preflight can register
    /// for that Project. The commit is paused inside its transaction check.
    /// Mutant: read the in-flight set and release it before the transaction
    /// → the registration succeeds during the pause → FAIL.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn m1_no_preflight_registers_between_check_and_commit() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        std::fs::create_dir(&repo).unwrap();
        for args in [
            &["init", "-b", "main"][..],
            &[
                "-c",
                "user.name=F",
                "-c",
                "user.email=f@example.invalid",
                "commit",
                "--allow-empty",
                "-m",
                "f",
            ][..],
        ] {
            assert!(
                std::process::Command::new("git")
                    .current_dir(&repo)
                    .args(args)
                    .status()
                    .unwrap()
                    .success()
            );
        }
        let runtime = Arc::new(runtime(dir.path()));
        let (planned, _) = plan_add(
            &[],
            &repo,
            crate::project::AddProject::default(),
            &Config::default(),
        )
        .unwrap();
        let mut project = planned;
        runtime.commit_project(&mut project, None).await.unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        *COMMIT_PAUSE.lock().unwrap() = Some((project.id, barrier.clone()));
        let committing = {
            let (runtime, mut renamed) = (runtime.clone(), project.clone());
            renamed.name = "renamed".into();
            let expected = project.version;
            tokio::spawn(async move { runtime.commit_project(&mut renamed, Some(expected)).await })
        };
        let wait = barrier.clone();
        tokio::task::spawn_blocking(move || wait.wait())
            .await
            .unwrap();
        let started = Instant::now();
        let registered = runtime.project_preflights.try_lock().is_ok();
        // Resume first, so a failing assertion never leaves the commit paused.
        let resume = barrier.clone();
        tokio::task::spawn_blocking(move || resume.wait())
            .await
            .unwrap();
        committing.await.unwrap().unwrap();
        assert!(
            !registered,
            "a preflight could register inside the commit window"
        );
        assert!(started.elapsed() < Duration::from_secs(5));
        assert!(runtime.project_preflights.try_lock().is_ok());
    }
}
