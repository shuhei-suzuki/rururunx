//! Original preparation custody, not Native permission or result evidence.
//! Source slots retain no strong Driver/Runtime/marker/capacity backlink.
use super::*;
use crate::{
    adapter::{InputKind, native::NativePhasePort},
    execution::{
        attempts::PreparedExecutorRemainder, native::ManagedInput, owner::PreparationGuard,
        phase::NativeAllocation,
    },
    runtime::phase_supervisor::PendingReservationError,
    state::DriverReadTicket,
    workflow::AttemptState,
};
use std::sync::{MutexGuard, Weak};

type SourceSlot = tokio::sync::Mutex<Option<TaskSources>>;
type SourceMap = BTreeMap<TaskId, Arc<SourceSlot>>;

pub(crate) struct SourceNativeHandoff {
    custody: Arc<SourceNativeCustody>,
    ticket: Arc<DriverReadTicket>,
    workflow: RecordId,
}
impl SourceNativeHandoff {
    pub(crate) fn custody(&self) -> &Arc<SourceNativeCustody> {
        &self.custody
    }
    pub(crate) fn ticket(&self) -> &Arc<DriverReadTicket> {
        &self.ticket
    }
    pub(crate) fn workflow(&self) -> RecordId {
        self.workflow
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CustodyState {
    Ready,
    InFlight,
    Accepted,
    Held,
}
struct Assets {
    state: CustodyState,
    prepared: Option<PreparedExecutor>,
    allocation: Option<NativeAllocation>,
    remainder: Option<PreparedExecutorRemainder>,
    rejected_guard: Option<PreparationGuard>,
}
pub(crate) struct SourceNativeCustody {
    producer: Weak<ManagedWorkflowSources>,
    slot: Weak<SourceSlot>,
    ticket: Weak<DriverReadTicket>,
    frame: Arc<Frame>,
    unit: ExecutionUnit,
    selected: Arc<NativePhasePort>,
    operation: OperationId,
    pair: uuid::Uuid,
    session: SessionId,
    invocation: NativeInvocationId,
    seal: Arc<SourceNativePreparationSeal>,
    assets: Mutex<Assets>,
}
/// Only the actual validated first Executor offer creates this immutable seal.
/// No Deserialize/DTO constructor, live permission or strong producer backlink.
pub(crate) struct SourceNativePreparationSeal {
    frame: Arc<Frame>,
    operation: OperationId,
    pair: uuid::Uuid,
    input_bytes: Vec<u8>,
}
fn validate_seal_identity(
    actual: (OperationId, uuid::Uuid, &[u8]),
    original: (OperationId, uuid::Uuid, &[u8]),
) -> Result<()> {
    ensure!(
        actual == original,
        "original committed Source preparation seal differs"
    );
    Ok(())
}
impl SourceNativePreparationSeal {
    pub(crate) fn config(&self) -> &crate::config::Config {
        &self.frame.config
    }
    pub(crate) fn revision(&self) -> &str {
        &self.frame.revision
    }
    pub(crate) fn inventory_digest(&self) -> &str {
        &self.frame.versions["context:committed_inventory"]
    }
    pub(crate) fn inventory(&self) -> &BTreeMap<String, crate::context::committed::InventoryEntry> {
        self.frame.index.inventory()
    }
    pub(crate) fn tree(&self) -> &BTreeMap<String, CommittedTreeEntry> {
        &self.frame.native_tree
    }
    pub(crate) fn qualify_git(
        &self,
        allocation: &NativeAllocation,
        project: &Project,
    ) -> Result<()> {
        let facts = allocation.facts();
        validate_seal_identity(
            (facts.operation_id, facts.pair_id, facts.input_bytes),
            (self.operation, self.pair, &self.input_bytes),
        )?;
        ensure!(
            self.frame.artifact.is_none()
                && facts.input.revision == self.frame.revision
                && self.frame.revision == allocation.unit_snapshot().base_sha,
            "actual readonly artifact lease unavailable"
        );
        let inventory = self.frame.index.inventory();
        ensure!(
            valid_oid(&self.frame.revision)
                && inventory.len() <= 4096
                && self.frame.native_tree.len() == inventory.len()
                && self.frame.versions.get("code") == Some(&self.frame.revision)
                && self.frame.versions.get("context:committed_inventory")
                    == self
                        .frame
                        .index
                        .source_versions()
                        .get("context:committed_inventory"),
            "original committed Frame inventory differs"
        );
        for (key, value) in &self.frame.versions {
            if key == "code" || key == "context:committed_inventory" || key.starts_with("rules:") {
                ensure!(
                    facts.input.source_versions.get(key) == Some(value),
                    "original encoded input dependency differs"
                );
            }
            if let Some(path) = key.strip_prefix("rules:")
                && path != "config"
            {
                ensure!(
                    inventory
                        .get(path)
                        .is_some_and(|e| e.skipped.is_none() && e.sha256.as_ref() == Some(value)),
                    "original rule digest differs"
                );
            }
        }
        if let Some(reference) = &project.config_ref {
            let path = reference
                .strip_prefix(&project.root)?
                .to_str()
                .context("original config path not UTF8")?;
            ensure!(
                inventory.get(path).is_some_and(|e| e.skipped.is_none()
                    && e.sha256.as_ref() == self.frame.versions.get("rules:config")),
                "original config digest differs"
            );
        } else {
            ensure!(
                !self.frame.versions.contains_key("rules:config"),
                "original config unexpectedly present"
            );
        }
        qualify_git_corpus(
            &self.frame.revision,
            inventory,
            &self.frame.native_tree,
            &self.frame.native_bytes,
        )?;
        for path in &self.frame.native_mandatory {
            let entry = inventory
                .get(path)
                .context("original mandatory inventory entry absent")?;
            let original_bytes = self
                .frame
                .native_bytes
                .get(path)
                .context("original mandatory actual bytes absent")?;
            ensure!(
                entry.sha256.as_deref() == Some(digest(original_bytes).as_str())
                    && self
                        .frame
                        .native_tree
                        .get(path)
                        .is_some_and(|p| p.oid == entry.oid),
                "original mandatory bytes/OID correspondence differs"
            );
        }
        Ok(())
    }
}
// Nongrant corpus validation. Only the enclosing SAME Source seal checks
// original operation/pair/input/Frame identity; equal maps cannot create it.
fn qualify_git_corpus(
    revision: &str,
    inventory: &BTreeMap<String, crate::context::committed::InventoryEntry>,
    tree: &BTreeMap<String, CommittedTreeEntry>,
    bytes_by_path: &BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    ensure!(
        valid_oid(revision) && inventory.len() <= 4096 && tree.len() == inventory.len(),
        "original corpus shape differs"
    );
    for (path, entry) in inventory {
        let tree = tree.get(path).context("original tree entry absent")?;
        ensure!(
            entry.skipped.as_deref() != Some(UNSUPPORTED_ENTRY)
                && matches!(tree.mode.as_str(), "100644" | "100755")
                && tree.kind == "blob"
                && tree.oid == entry.oid
                && entry.oid.len() == revision.len()
                && valid_oid(&entry.oid)
                && (entry.bytes.is_none() || entry.bytes == tree.size),
            "original physical tree/inventory differs"
        );
        if let Some(size) = entry.bytes {
            let bytes = bytes_by_path
                .get(path)
                .context("original Source bytes absent")?;
            ensure!(
                bytes.len() == size && entry.sha256.as_deref() == Some(digest(bytes).as_str()),
                "original Source bytes/hash/size differs"
            );
        }
    }
    Ok(())
}

/// A transient borrow origin, never embedded in a Source slot/custody/plan.
pub(crate) struct SourceNativeOrigin {
    producer: Arc<ManagedWorkflowSources>,
    slot: Arc<SourceSlot>,
}
/// Map -> tried slot -> assets; held only during Root's synchronous transfer.
pub(crate) struct SourceNativeTransfer<'a> {
    custody: &'a SourceNativeCustody,
    _map: MutexGuard<'a, SourceMap>,
    _slot: tokio::sync::MutexGuard<'a, Option<TaskSources>>,
    assets: MutexGuard<'a, Assets>,
}
/// Factual restoration state only. Held does not permit another transfer.
pub(crate) enum SourceRefusalRestoration {
    Restored,
    Held,
}

impl SourceNativeCustody {
    /// Borrow only the immutable original offer seal. Root first validates its
    /// SAME Source/ticket/allocation marker; no slot/asset lock or encoding here.
    pub(crate) fn preparation_seal(
        &self,
        allocation: &NativeAllocation,
    ) -> Result<Arc<SourceNativePreparationSeal>> {
        let facts = allocation.facts();
        ensure!(
            facts.operation_id == self.operation && facts.pair_id == self.pair,
            "original accepted Source allocation differs"
        );
        validate_seal_identity(
            (facts.operation_id, facts.pair_id, facts.input_bytes),
            (self.seal.operation, self.seal.pair, &self.seal.input_bytes),
        )?;
        ensure!(
            Arc::ptr_eq(&self.frame, &self.seal.frame),
            "original Source frame replaced"
        );
        Ok(self.seal.clone())
    }
    /// Original selected object only; no allocation or dispatch permission.
    pub(crate) fn selected_port(&self) -> &Arc<NativePhasePort> {
        &self.selected
    }
    pub(crate) fn original_origin(&self) -> Result<SourceNativeOrigin> {
        Ok(SourceNativeOrigin {
            producer: self.producer.upgrade().context("original Sources ended")?,
            slot: self.slot.upgrade().context("original source slot ended")?,
        })
    }
    pub(crate) fn operation(&self) -> OperationId {
        self.operation
    }
    pub(super) fn same_allocation(&self, allocation: &NativeAllocation) -> Result<bool> {
        let f = allocation.facts();
        Ok(
            std::ptr::eq(allocation.selected_port(), self.selected.as_ref())
                && f.operation_id == self.operation
                && f.pair_id == self.pair
                && f.session_id == self.session
                && f.invocation_id == self.invocation
                && serde_json::to_value(allocation.unit_snapshot())?
                    == serde_json::to_value(&self.unit)?,
        )
    }
}

impl SourceNativeOrigin {
    pub(crate) fn lock_transfer<'a>(
        &'a self,
        custody: &'a SourceNativeCustody,
        owner: &Arc<RuntimeOwner>,
        ticket: &DriverReadTicket,
    ) -> Result<SourceNativeTransfer<'a>> {
        ensure!(
            Arc::ptr_eq(owner, &self.producer.owner)
                && custody.producer.ptr_eq(&Arc::downgrade(&self.producer))
                && custody.slot.ptr_eq(&Arc::downgrade(&self.slot))
                && custody
                    .ticket
                    .upgrade()
                    .is_some_and(|original| std::ptr::eq(original.as_ref(), ticket)),
            "foreign original source handoff"
        );
        ticket.preparation_matches(&custody.unit)?;
        let map = self
            .producer
            .tasks
            .lock()
            .map_err(|_| anyhow::anyhow!("Sources poisoned"))?;
        ensure!(
            map.get(&custody.unit.scope.task_id.context("Task missing")?)
                .is_some_and(|s| Arc::ptr_eq(s, &self.slot)),
            "original source membership changed"
        );
        let slot = self
            .slot
            .try_lock()
            .map_err(|_| anyhow::anyhow!("source handoff slot busy"))?;
        let state = slot.as_ref().context("source handoff removed")?;
        ensure!(
            state.recovery.is_none()
                && state.prepared.is_none()
                && Arc::ptr_eq(&state.frame, &custody.frame)
                && state
                    .handoff
                    .as_ref()
                    .is_some_and(|original| std::ptr::eq(original.as_ref(), custody)),
            "source handoff origin replaced"
        );
        let assets = custody
            .assets
            .lock()
            .map_err(|_| anyhow::anyhow!("source custody poisoned"))?;
        ensure!(
            assets.state == CustodyState::Ready
                && assets
                    .prepared
                    .as_ref()
                    .is_some_and(|p| p.retains(owner, &custody.unit).unwrap_or(false))
                && assets
                    .allocation
                    .as_ref()
                    .is_some_and(|a| custody.same_allocation(a).unwrap_or(false)),
            "original handoff assets unavailable"
        );
        Ok(SourceNativeTransfer {
            custody,
            _map: map,
            _slot: slot,
            assets,
        })
    }
}
impl SourceNativeTransfer<'_> {
    /// Root holds actual control admission and independently retained envelope.
    /// No await occurs between this move and actual reserve/restoration.
    pub(crate) fn take_original(&mut self) -> Result<(NativeAllocation, PreparationGuard)> {
        ensure!(
            self.assets.state == CustodyState::Ready
                && self.assets.prepared.is_some()
                && self.assets.allocation.is_some(),
            "original handoff already transferred"
        );
        let prepared = self
            .assets
            .prepared
            .take()
            .expect("checked actual preparation");
        let allocation = self.assets.allocation.take().expect("checked allocation");
        let (remainder, guard) = prepared.into_original_parts();
        self.assets.remainder = Some(remainder);
        self.assets.state = CustodyState::InFlight;
        Ok((allocation, guard))
    }
    /// Only the concrete queue's ownership-returning refusal rejoins the guard.
    /// Malformed returns remain owned/Held; no reconstructed preparation.
    pub(crate) fn restore_refused(
        &mut self,
        error: PendingReservationError,
    ) -> std::result::Result<SourceRefusalRestoration, Box<PendingReservationError>> {
        // A protocol error returns ownership intact to Root. It must retain the
        // returned objects outside these locks, not drop an armed guard here.
        if self.assets.state != CustodyState::InFlight || self.assets.remainder.is_none() {
            self.assets.state = CustodyState::Held;
            return Err(Box::new(error));
        }
        let PendingReservationError {
            allocation,
            preparation,
            ..
        } = error;
        let same = self.custody.same_allocation(&allocation).unwrap_or(false);
        self.assets.allocation = Some(allocation);
        let remainder = self
            .assets
            .remainder
            .take()
            .expect("checked actual remainder");
        match remainder.rejoin(preparation) {
            Ok(prepared) => {
                self.assets.prepared = Some(prepared);
                self.assets.state = if same {
                    CustodyState::Ready
                } else {
                    CustodyState::Held
                };
            }
            Err(parts) => {
                let (remainder, guard) = *parts;
                self.assets.remainder = Some(remainder);
                self.assets.rejected_guard = Some(guard);
                self.assets.state = CustodyState::Held;
            }
        }
        Ok(if self.assets.state == CustodyState::Ready {
            SourceRefusalRestoration::Restored
        } else {
            SourceRefusalRestoration::Held
        })
    }
    /// Root has ALREADY stored the actual capacity in its retained envelope.
    /// A Source slot keeps no strong capacity/marker/Driver backlink.
    pub(crate) fn finish_accepted(mut self) {
        if self.assets.state == CustodyState::InFlight {
            self.assets.remainder = None;
            self.assets.state = CustodyState::Accepted;
        } else {
            self.assets.state = CustodyState::Held;
        }
    }
}
impl Drop for SourceNativeTransfer<'_> {
    fn drop(&mut self) {
        if self.assets.state == CustodyState::InFlight {
            self.assets.state = CustodyState::Held;
        }
    }
}

impl ManagedWorkflowSources {
    pub(crate) async fn offer_first_executor(
        self: &Arc<Self>,
        task: &Task,
        record: &Record,
        context: &ContextVersion,
        ticket: DriverReadTicket,
        selected: Arc<NativePhasePort>,
    ) -> Result<SourceNativeHandoff> {
        ticket.matches_input_view(task, record, context)?;
        ticket.validate_current_read()?;
        let workflow: WorkflowSnapshot = serde_json::from_value(record.data.clone())?;
        let index = workflow
            .active
            .context("first Executor reservation missing")?;
        let attempt = workflow
            .history
            .get(index)
            .context("first Executor index invalid")?;
        ensure!(
            workflow.generation == 1
                && index + 1 == workflow.history.len()
                && attempt.phase.actor() == Actor::Executor
                && attempt.state == AttemptState::Running
                && attempt.generation == workflow.generation
                && attempt.context_version == context.version
                && !attempt.dispatch_started
                && attempt.session_id.is_none()
                && attempt.execution.is_none()
                && attempt.native_wait.is_none()
                && attempt.next_due.is_none()
                && attempt.observations.is_empty()
                && attempt.completed_at.is_none()
                && workflow.held_reason.is_none()
                && attempt.agent.as_deref() == Some(task.executor.as_str())
                && workflow.history[..index].iter().all(|a| matches!(
                    a.phase,
                    Phase::Issue | Phase::Worktree
                ) && a.state
                    == AttemptState::Succeeded
                    && a.session_id.is_none()
                    && a.execution.is_none())
                && workflow
                    .configured_phases
                    .iter()
                    .find(|p| p.actor() == Actor::Executor)
                    == Some(&attempt.phase)
                && std::ptr::eq(selected.owner(), self.owner.as_ref())
                && selected.alias() == task.executor,
            "first Executor handoff selection/history differs"
        );
        let slot = self
            .tasks
            .lock()
            .map_err(|_| anyhow::anyhow!("Sources poisoned"))?
            .get(&task.id)
            .cloned()
            .context("original source slot missing")?;
        let mut state = slot.lock().await;
        let state = state.as_mut().context("source preparation missing")?;
        let prepared = state
            .prepared
            .as_ref()
            .context("source preparation already transferred")?;
        let unit = prepared.unit().clone();
        ticket.preparation_matches(&unit)?;
        ensure!(
            state.handoff.is_none()
                && state.recovery.is_none()
                && state.frame.artifact.is_none()
                && unit.scope == task.scope()
                && unit.phase == attempt.phase.key()
                && unit.provider == selected.provider()
                && unit.kind == UnitKind::Executor
                && unit.state == UnitState::Preparing
                && unit.disposition == Disposition::Active
                && unit.generation == 1
                && unit.owner_epoch == self.owner.epoch()
                && unit.native_effects_open
                && unit.result_finalization_open
                && unit.work.is_none()
                && unit.session_id.is_none()
                && unit.artifact_id.is_none()
                && state.frame.revision == unit.base_sha
                && attempt.unit.as_ref().is_some_and(|u| u.scope == unit.scope
                    && u.unit == unit.id
                    && u.generation == unit.generation
                    && u.epoch == unit.owner_epoch)
                && prepared.retains(&self.owner, &unit)?,
            "first Executor lacks same adopted preparation"
        );
        // This temporary frame checker is not stored in the Source custody.
        let proof = InitialInputFrame {
            producer: self.clone(),
            slot: slot.clone(),
            frame: state.frame.clone(),
            unit: unit.clone(),
        };
        proof.validate_context(task, record, context)?;
        let config = state.frame.config.agents.get(&task.executor);
        let allocation = selected.allocate(
            ManagedInput {
                agent: task.executor.clone(),
                authority: unit.authority(),
                artifact: None,
                input: PreparedInput {
                    scope: context.scope.clone(),
                    kind: InputKind::ContextPack,
                    revision: context.revision.clone(),
                    version: context.version,
                    source_versions: context.source_hashes.clone(),
                    payload: serde_json::to_string(&context.data)?,
                },
            },
            config.and_then(|c| c.model.clone()),
            config.and_then(|c| c.effort.clone()),
        )?;
        ensure!(
            serde_json::to_value(allocation.unit_snapshot())? == serde_json::to_value(&unit)?,
            "actual allocation preparation changed"
        );
        ticket.validate_current_read()?;
        let ticket = Arc::new(ticket);
        let facts = allocation.facts();
        // proof.validate_context above established exact committed template,
        // payload and all Context pins before any original assets transfer.
        let seal = Arc::new(SourceNativePreparationSeal {
            frame: state.frame.clone(),
            operation: facts.operation_id,
            pair: facts.pair_id,
            input_bytes: facts.input_bytes.to_vec(),
        });
        let map = self
            .tasks
            .lock()
            .map_err(|_| anyhow::anyhow!("Sources poisoned"))?;
        ensure!(
            map.get(&task.id).is_some_and(|s| Arc::ptr_eq(s, &slot)),
            "original Sources removed during allocation"
        );
        let custody = Arc::new(SourceNativeCustody {
            producer: Arc::downgrade(self),
            slot: Arc::downgrade(&slot),
            ticket: Arc::downgrade(&ticket),
            frame: state.frame.clone(),
            unit,
            selected,
            operation: facts.operation_id,
            pair: facts.pair_id,
            session: facts.session_id,
            invocation: facts.invocation_id,
            seal,
            assets: Mutex::new(Assets {
                state: CustodyState::Ready,
                prepared: state.prepared.take(),
                allocation: Some(allocation),
                remainder: None,
                rejected_guard: None,
            }),
        });
        state.handoff = Some(custody.clone());
        // No await follows installation: caller synchronously gives this SAME
        // envelope to Root retention before constructing the admission future.
        Ok(SourceNativeHandoff {
            custody,
            ticket,
            workflow: record.id,
        })
    }
}

#[cfg(test)]
mod seal_identity_tests {
    use super::*;
    #[test]
    fn nongrant_original_corpus_checks_bytes_hash_tree_type_oid_path_and_size() {
        use crate::context::committed::InventoryEntry;
        let r = "a".repeat(40);
        let tree = parse_committed_tree(
            format!("100644 blob {} 3\tfile.txt\0", "b".repeat(40)).as_bytes(),
        )
        .unwrap();
        let index = BTreeMap::from([(
            "file.txt".into(),
            InventoryEntry {
                oid: "b".repeat(40),
                sha256: Some(digest(b"abc")),
                bytes: Some(3),
                skipped: None,
            },
        )]);
        let bytes = BTreeMap::from([("file.txt".into(), b"abc".to_vec())]);
        qualify_git_corpus(&r, &index, &tree, &bytes).unwrap();
        for mutate in 0..7 {
            let mut changed = tree.clone();
            let entry = changed.get_mut("file.txt").unwrap();
            match mutate {
                0 => entry.oid = "c".repeat(40),
                1 => entry.mode = "120000".into(),
                2 => entry.mode = "160000".into(),
                3 => entry.kind = "commit".into(),
                4 => entry.size = Some(4),
                5 => {
                    let entry = changed.remove("file.txt").unwrap();
                    changed.insert("other.txt".into(), entry);
                }
                _ => changed.clear(),
            }
            assert!(qualify_git_corpus(&r, &index, &changed, &bytes).is_err());
        }
        let mut changed_bytes = bytes.clone();
        changed_bytes.insert("file.txt".into(), b"xyz".to_vec());
        assert!(qualify_git_corpus(&r, &index, &tree, &changed_bytes).is_err());
        changed_bytes.insert("file.txt".into(), b"abcd".to_vec());
        assert!(qualify_git_corpus(&r, &index, &tree, &changed_bytes).is_err());
        let mut changed = index.clone();
        changed.get_mut("file.txt").unwrap().sha256 = Some("c".repeat(64));
        assert!(qualify_git_corpus(&r, &changed, &tree, &bytes).is_err());
        let mut large = index.clone();
        let entry = large.get_mut("file.txt").unwrap();
        entry.bytes = None;
        entry.sha256 = None;
        entry.skipped = Some("file exceeds 256 KiB".into());
        qualify_git_corpus(&r, &large, &tree, &BTreeMap::new()).unwrap();
        large.get_mut("file.txt").unwrap().skipped = Some(UNSUPPORTED_ENTRY.into());
        assert!(qualify_git_corpus(&r, &large, &tree, &BTreeMap::new()).is_err());
        assert!(qualify_git_corpus(&"a".repeat(64), &index, &tree, &bytes).is_err());
    }
    #[test]
    fn nongrant_seal_scalar_original_operation_pair_and_full_encoded_bytes() {
        let operation = OperationId::new();
        let pair = uuid::Uuid::new_v4();
        let original = (
            operation,
            pair,
            b"original exact encoded Context/template/pins".as_slice(),
        );
        validate_seal_identity(original, original).unwrap();
        for changed in [
            (OperationId::new(), pair, original.2),
            (operation, uuid::Uuid::new_v4(), original.2),
            (operation, pair, b"row-equal replacement payload".as_slice()),
            (operation, pair, &original.2[..original.2.len() - 1]),
        ] {
            assert!(
                validate_seal_identity(changed, original).is_err(),
                "original seal facts must refuse substitution"
            );
        }
    }
}
