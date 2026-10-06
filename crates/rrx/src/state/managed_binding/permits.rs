//! Exact connection-local SQL mutation permissions. These are NOT native proofs.
use anyhow::{Result, ensure};
use rusqlite::{
    Connection,
    functions::FunctionFlags,
    types::{Value, ValueRef},
};
use std::sync::{Arc, Mutex};

/// Fixed complete columns, including encoded bodies; callers cannot omit metadata.
pub(super) fn columns(table: &str) -> Option<&'static [&'static str]> {
    Some(match table {
        "workflow_native_contracts" => &[
            "workflow_id",
            "project_id",
            "goal_id",
            "task_id",
            "owner_epoch",
            "origin",
            "profile_digest",
            "contract_state",
            "version",
            "body",
        ],
        "managed_phase_operations" => &[
            "operation_id",
            "workflow_id",
            "project_id",
            "goal_id",
            "task_id",
            "workflow_generation",
            "attempt_index",
            "owner_epoch",
            "unit_id",
            "execution_generation",
            "allocated_session_id",
            "owner_id",
            "pair_id",
            "marker_project_version",
            "marker_goal_version",
            "marker_task_version",
            "marker_workflow_version",
            "marker_project_sha256",
            "marker_goal_sha256",
            "marker_task_sha256",
            "marker_workflow_sha256",
            "marker_digest",
            "origin",
            "phase",
            "actor",
            "provider",
            "alias",
            "role",
            "context_version",
            "phase_open",
            "version",
            "body",
        ],
        "managed_marker_bodies" => &[
            "operation_id",
            "workflow_id",
            "workflow_version",
            "workflow_sha256",
            "body",
        ],
        "managed_phase_owners" => &[
            "owner_id",
            "operation_id",
            "project_id",
            "goal_id",
            "task_id",
            "unit_id",
            "owner_epoch",
            "execution_generation",
            "allocated_session_id",
            "provider",
            "alias",
            "role",
            "worktree",
            "origin",
            "native_invocation_id",
            "validated",
            "version",
            "body",
        ],
        "managed_phase_inputs" => &[
            "pair_id",
            "owner_id",
            "operation_id",
            "project_id",
            "goal_id",
            "task_id",
            "context_version",
            "context_digest",
            "revision",
            "payload_sha256",
            "payload_bytes",
            "profile_digest",
            "version",
            "body",
        ],
        "managed_phase_admissions" => &[
            "pair_id",
            "operation_id",
            "owner_id",
            "native_invocation_id",
            "input_effect_id",
            "frame_sha256",
            "native_thread",
            "native_turn",
            "confirmed",
            "uncertain",
            "settled",
            "version",
            "body",
        ],
        "managed_phase_readiness" => &[
            "operation_id",
            "origin",
            "owner_epoch",
            "state",
            "start_ended",
            "known_terminal",
            "parking_version",
            "version",
            "body",
        ],
        "records" => &[
            "id",
            "kind",
            "project_id",
            "goal_id",
            "task_id",
            "version",
            "body",
        ],
        "audit" => &[
            "sequence",
            "project_id",
            "goal_id",
            "task_id",
            "kind",
            "at",
            "data",
        ],
        _ => return None,
    })
}

/// Not serializable; only managed-binding Store code can build a full row plan.
/// Value::Real is rejected, rather than conflating SQLite numeric encodings.
#[allow(dead_code)] // Actual marker/binder producer is composed separately.
pub(super) struct ExactRowMutation {
    table: &'static str,
    action: &'static str,
    old: Vec<Value>,
    new: Vec<Value>,
}
#[allow(dead_code)]
impl ExactRowMutation {
    pub(super) fn new(
        table: &'static str,
        action: &'static str,
        old: Option<Vec<Value>>,
        new: Option<Vec<Value>>,
    ) -> Result<Self> {
        let n = columns(table)
            .ok_or_else(|| anyhow::anyhow!("unknown private permit table"))?
            .len();
        ensure!(
            matches!(
                (action, old.is_some(), new.is_some()),
                ("INSERT", false, true) | ("UPDATE", true, true) | ("DELETE", true, false)
            ),
            "invalid exact mutation action"
        );
        for image in [&old, &new].into_iter().flatten() {
            ensure!(
                image.len() == n && image.iter().all(|v| !matches!(v, Value::Real(_))),
                "incomplete exact mutation image"
            );
        }
        Ok(Self {
            table,
            action,
            old: old.unwrap_or_else(|| vec![Value::Null; n]),
            new: new.unwrap_or_else(|| vec![Value::Null; n]),
        })
    }
}
struct Active {
    serial: u64,
    rows: Vec<Option<ExactRowMutation>>,
}
#[derive(Default)]
struct State {
    next: u64,
    active: Option<Active>,
}
#[derive(Default)]
pub(in crate::state) struct PrivatePermitManager {
    state: Mutex<State>,
}
struct Revoke<'a> {
    manager: &'a PrivatePermitManager,
    serial: u64,
}
impl Drop for Revoke<'_> {
    fn drop(&mut self) {
        // Recover poisoned state only to revoke; never to grant a permission.
        let mut state = self.manager.state.lock().unwrap_or_else(|e| e.into_inner());
        if state
            .active
            .as_ref()
            .is_some_and(|a| a.serial == self.serial)
        {
            state.active = None;
        }
    }
}
#[allow(dead_code)]
impl PrivatePermitManager {
    /// No async boundary. The closure must commit or roll back its own transaction;
    /// no manager lock is held while SQLite invokes the permission callback.
    pub(super) fn with_exact_permit<T>(
        &self,
        rows: Vec<ExactRowMutation>,
        f: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        ensure!(
            !rows.is_empty() && rows.len() <= 128,
            "private mutation plan bound"
        );
        let serial = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("private permit poisoned"))?;
            ensure!(state.active.is_none(), "nested private permit refused");
            state.next = state
                .next
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("private permit serial overflow"))?;
            let serial = state.next;
            state.active = Some(Active {
                serial,
                rows: rows.into_iter().map(Some).collect(),
            });
            serial
        };
        let _revoke = Revoke {
            manager: self,
            serial,
        };
        f()
    }
    /// Call before transaction commit when the port requires every planned write.
    pub(super) fn ensure_consumed(&self) -> Result<()> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("private permit poisoned"))?;
        ensure!(
            state
                .active
                .as_ref()
                .is_some_and(|a| a.rows.iter().all(Option::is_none)),
            "exact mutation plan not consumed"
        );
        Ok(())
    }
    fn matches(&self, ctx: &rusqlite::functions::Context<'_>) -> bool {
        let Some(table) = ctx.get_raw(0).as_str().ok() else {
            return false;
        };
        let Some(action) = ctx.get_raw(1).as_str().ok() else {
            return false;
        };
        let Some(cols) = columns(table) else {
            return false;
        };
        if ctx.len() != 2 + 2 * cols.len() {
            return false;
        }
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        let Some(active) = state.active.as_mut() else {
            return false;
        };
        for slot in &mut active.rows {
            let Some(row) = slot else { continue };
            if row.table == table
                && row.action == action
                && row
                    .old
                    .iter()
                    .chain(&row.new)
                    .enumerate()
                    .all(|(i, v)| equal(v, ctx.get_raw(i + 2)))
            {
                *slot = None;
                return true;
            }
        }
        false
    }
}
fn equal(expected: &Value, actual: ValueRef<'_>) -> bool {
    match (expected, actual) {
        (Value::Null, ValueRef::Null) => true,
        (Value::Integer(a), ValueRef::Integer(b)) => *a == b,
        (Value::Text(a), ValueRef::Text(b)) => a.as_bytes() == b,
        (Value::Blob(a), ValueRef::Blob(b)) => a.as_slice() == b,
        _ => false,
    }
}
pub(in crate::state) fn register_permit_function(
    c: &Connection,
    manager: Arc<PrivatePermitManager>,
) -> Result<()> {
    c.create_scalar_function(
        "rrx_binding_permit",
        -1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_INNOCUOUS,
        move |ctx| Ok(ctx.len() >= 2 && manager.matches(ctx)),
    )?;
    super::schema::register_projection(c)?;
    Ok(())
}
