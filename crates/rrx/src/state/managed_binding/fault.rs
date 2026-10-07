//! Fault-only test seam for the SC writers: fails exactly one named write of
//! the named Task, either before its transaction or after a genuine commit.
//! It never commits, plans, confirms or grants anything itself.
use crate::domain::TaskId;
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CommitFault {
    BeforeCommit,
    AfterCommit,
    /// A typed pre-write Conflict instead of the write (observed only).
    Conflict,
}
type Armed = ((TaskId, &'static str), CommitFault);
static FAULTS: Mutex<Vec<Armed>> = Mutex::new(Vec::new());

pub(crate) const BIND: &str = "bind";
pub(crate) const CLAIM: &str = "claim";
pub(crate) const OBSERVED: &str = "observed";
pub(crate) const CLOSURE: &str = "closure";
/// The success Driver publication (a refused cache publication).
pub(crate) const PUBLISH: &str = "publish";
/// The observed planner (a refused plan, not a write).
pub(crate) const OBSERVED_PLAN: &str = "observed plan";

pub(crate) fn arm_commit_fault(task: TaskId, site: &'static str, fault: CommitFault) {
    FAULTS.lock().unwrap().push(((task, site), fault));
}
/// Whether an armed fault is still unconsumed (its write never ran).
pub(crate) fn armed(task: TaskId, site: &'static str) -> bool {
    FAULTS
        .lock()
        .unwrap()
        .iter()
        .any(|(armed, _)| *armed == (task, site))
}
pub(super) fn take(task: Option<TaskId>, site: &'static str) -> Option<CommitFault> {
    let task = task?;
    let mut faults = FAULTS.lock().unwrap();
    let index = faults
        .iter()
        .position(|(armed, _)| *armed == (task, site))?;
    Some(faults.swap_remove(index).1)
}
/// The precommit fault: an `Err` before any transaction (nothing written).
pub(super) fn before(fault: Option<CommitFault>) -> anyhow::Result<()> {
    anyhow::ensure!(
        fault != Some(CommitFault::BeforeCommit),
        "injected precommit SC write fault"
    );
    Ok(())
}
/// The postcommit fault: an `Err` replacing a genuinely committed result.
pub(super) fn after<T>(
    fault: Option<CommitFault>,
    committed: bool,
    result: T,
) -> anyhow::Result<T> {
    anyhow::ensure!(
        !(committed && fault == Some(CommitFault::AfterCommit)),
        "injected postcommit SC write fault"
    );
    Ok(result)
}
