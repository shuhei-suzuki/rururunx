//! Private facts issued by the actual managed Native registration/input/terminal
//! path. Persisted DTOs cannot recreate these handles. This module does not make
//! composition available; the Root marker/pair transaction remains required.
use super::*;
use crate::{
    execution::phase::NativeAllocation,
    state::managed_binding::{OriginalMarker, PhaseLaunchParts},
};
use std::sync::atomic::{AtomicBool, Ordering};

/// Genuine registered owner. Only the actual Native module can issue it, after
/// its managed registration transaction succeeds with the original launch.
pub(crate) struct NativePhaseSession {
    launch: Arc<PhaseLaunchParts>,
    projection: Mutex<Projection>,
    live: AtomicBool,
}
struct Projection {
    session: Session,
    record_version: u64,
    consumed: Option<Weak<ConsumedPhaseInput>>,
    settlement: Option<Weak<OwnedPhaseSettlement>>,
}
/// A copied binding observation remains tied to its actual private issuer. It is
/// neither a public Session DTO nor authority constructed from a stored row.
pub(crate) struct NativePhaseBinding {
    owner: Arc<NativePhaseSession>,
    session: Session,
    record_version: u64,
    consumed: Option<Arc<ConsumedPhaseInput>>,
    settlement: Option<Arc<OwnedPhaseSettlement>>,
    live: bool,
}
/// Issued only after the actual before-wire transaction consumes this owner's
/// original input pair and records the same Native6 input intent.
pub(crate) struct ConsumedPhaseInput {
    owner: Arc<NativePhaseSession>,
    effect: OperationId,
    frame_sha256: String,
    expected_thread: Option<String>,
    acknowledgement: Mutex<Option<Acknowledgement>>,
}
struct Acknowledgement {
    thread: String,
    turn: Option<String>,
}
/// Shared by the actual Core and its retained Native registry entry. Proofs are
/// held strongly here, while their owner uses Weak backlinks to avoid a cycle.
/// Engine future destruction therefore cannot erase an observed input/settlement.
pub(super) struct PhaseActor {
    pub(super) owner: Arc<NativePhaseSession>,
    retained: Mutex<RetainedProofs>,
}
struct RetainedProofs {
    consumed: Option<Arc<ConsumedPhaseInput>>,
    settlement: Option<Arc<OwnedPhaseSettlement>>,
}
impl PhaseActor {
    /// Called by the real Native launch only after its genuine registration
    /// transaction commits. There is no constructor from a Session row/DTO.
    pub(super) fn registered(
        launch: Arc<PhaseLaunchParts>,
        session: Session,
        record_version: u64,
    ) -> Result<Arc<Self>> {
        let owner = NativePhaseSession::registered(launch, session, record_version)?;
        Ok(Arc::new(Self {
            owner,
            retained: Mutex::new(RetainedProofs {
                consumed: None,
                settlement: None,
            }),
        }))
    }
    /// The caller supplies the exact successful before-wire journal, never a
    /// historical effect lookup. The same private input remains retained once.
    pub(super) fn admitted(
        &self,
        effect: OperationId,
        frame_sha256: String,
        expected_thread: Option<String>,
    ) -> Result<Arc<ConsumedPhaseInput>> {
        let consumed = ConsumedPhaseInput::admitted(
            self.owner.clone(),
            effect,
            frame_sha256,
            expected_thread,
        )?;
        let mut retained = self
            .retained
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase retention unavailable"))?;
        ensure!(
            retained.consumed.is_none(),
            "native phase input already retained"
        );
        retained.consumed = Some(consumed.clone());
        Ok(consumed)
    }
    pub(super) fn consumed(&self) -> Result<Arc<ConsumedPhaseInput>> {
        self.retained
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase retention unavailable"))?
            .consumed
            .clone()
            .context("actual phase input not consumed")
    }
    /// Called only after the same actual terminal commits own logical closure.
    /// The saved Arc is retained unchanged, including when binder delivery fails.
    pub(super) fn settled(
        &self,
        terminal: Arc<NativeTerminal>,
        unit: ExecutionUnit,
        receipt: native_result::NativeResultReceipt,
        session: Session,
        record_version: u64,
    ) -> Result<Arc<OwnedPhaseSettlement>> {
        let consumed = self.consumed()?;
        let settlement = OwnedPhaseSettlement::completed(
            consumed,
            terminal,
            unit,
            receipt,
            session,
            record_version,
        )?;
        let mut retained = self
            .retained
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase retention unavailable"))?;
        ensure!(
            retained.settlement.is_none(),
            "native phase settlement already retained"
        );
        retained.settlement = Some(settlement.clone());
        Ok(settlement)
    }
}
/// Only an actual saved NativeTerminal followed by successful own logical
/// closure can issue this. It is not a review opinion or a cleanup guarantee.
pub(crate) struct OwnedPhaseSettlement {
    owner: Arc<NativePhaseSession>,
    consumed: Arc<ConsumedPhaseInput>,
    terminal: Arc<NativeTerminal>,
    unit: ExecutionUnit,
    receipt: native_result::NativeResultReceipt,
    session: Session,
    record_version: u64,
    thread: String,
    turn: Option<String>,
}
impl NativePhaseSession {
    pub(crate) fn launch_parts(&self) -> &PhaseLaunchParts {
        &self.launch
    }
    pub(crate) fn marker(&self) -> &OriginalMarker {
        self.launch.marker()
    }
    pub(crate) fn allocation(&self) -> &NativeAllocation {
        self.launch.allocation()
    }
    pub(super) fn registered(
        launch: Arc<PhaseLaunchParts>,
        session: Session,
        record_version: u64,
    ) -> Result<Arc<Self>> {
        ensure!(
            launch.is_retained() && Arc::ptr_eq(launch.marker().allocation(), launch.allocation()),
            "actual phase registration lost original retained launch"
        );
        let facts = launch.allocation().facts();
        ensure!(
            record_version > 0
                && session.id == facts.session_id
                && session.scope == *facts.scope
                && session.agent == facts.alias
                && session.provider == facts.provider
                && session.role == facts.role
                && session.worktree == facts.path
                && session.model.as_deref() == facts.model
                && session.effort.as_deref() == facts.effort
                && session.state == SessionState::Starting
                && session.native_ref.is_none(),
            "actual phase registration differs from allocated identity"
        );
        Ok(Arc::new(Self {
            launch,
            projection: Mutex::new(Projection {
                session,
                record_version,
                consumed: None,
                settlement: None,
            }),
            live: AtomicBool::new(true),
        }))
    }
    pub(super) fn project(&self, session: &Session, record_version: u64) -> Result<()> {
        let mut projection = self
            .projection
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase projection unavailable"))?;
        let old = &projection.session;
        ensure!(
            record_version >= projection.record_version
                && old.id == session.id
                && old.scope == session.scope
                && old.agent == session.agent
                && old.provider == session.provider
                && old.role == session.role
                && old.worktree == session.worktree
                && old.model == session.model
                && old.effort == session.effort
                && old.started_at == session.started_at
                && old
                    .native_ref
                    .as_ref()
                    .is_none_or(|id| session.native_ref.as_ref() == Some(id)),
            "native phase projection identity changed"
        );
        if record_version == projection.record_version {
            ensure!(
                serde_json::to_value(old)? == serde_json::to_value(session)?,
                "native phase same-version projection changed"
            );
        }
        projection.session = session.clone();
        projection.record_version = record_version;
        Ok(())
    }
    pub(super) fn revoke(&self) {
        self.live.store(false, Ordering::SeqCst);
    }
    pub(crate) fn binding_snapshot(self: &Arc<Self>) -> Result<NativePhaseBinding> {
        let projection = self
            .projection
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase projection unavailable"))?;
        Ok(NativePhaseBinding {
            owner: self.clone(),
            session: projection.session.clone(),
            record_version: projection.record_version,
            consumed: projection.consumed.as_ref().and_then(Weak::upgrade),
            settlement: projection.settlement.as_ref().and_then(Weak::upgrade),
            live: self.live.load(Ordering::SeqCst),
        })
    }
}
impl NativePhaseBinding {
    pub(crate) fn owner(&self) -> &NativePhaseSession {
        &self.owner
    }
    pub(crate) fn marker(&self) -> &OriginalMarker {
        self.owner.marker()
    }
    pub(crate) fn allocation(&self) -> &NativeAllocation {
        self.owner.allocation()
    }
    pub(crate) fn session(&self) -> &Session {
        &self.session
    }
    pub(crate) fn record_version(&self) -> u64 {
        self.record_version
    }
    pub(crate) fn consumed(&self) -> Option<&ConsumedPhaseInput> {
        self.consumed.as_deref()
    }
    pub(crate) fn settlement(&self) -> Option<&OwnedPhaseSettlement> {
        self.settlement.as_deref()
    }
    pub(crate) fn is_live(&self) -> bool {
        self.live && self.owner.live.load(Ordering::SeqCst)
    }
}
impl ConsumedPhaseInput {
    pub(crate) fn belongs_to(&self, owner: &NativePhaseSession) -> bool {
        std::ptr::eq(self.owner.as_ref(), owner)
    }
    pub(crate) fn acknowledgement(&self) -> Result<Option<(String, Option<String>)>> {
        let acknowledgement = self
            .acknowledgement
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase acknowledgement unavailable"))?;
        Ok(acknowledgement
            .as_ref()
            .map(|ack| (ack.thread.clone(), ack.turn.clone())))
    }
    pub(super) fn admitted(
        owner: Arc<NativePhaseSession>,
        effect: OperationId,
        frame_sha256: String,
        expected_thread: Option<String>,
    ) -> Result<Arc<Self>> {
        ensure!(
            owner.live.load(Ordering::SeqCst)
                && frame_sha256.len() == 64
                && frame_sha256.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "native phase input admission unavailable"
        );
        let input = Arc::new(Self {
            owner: owner.clone(),
            effect,
            frame_sha256,
            expected_thread,
            acknowledgement: Mutex::new(None),
        });
        let mut projection = owner
            .projection
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase projection unavailable"))?;
        // A lost strong reference cannot allow a second consumption: the private
        // one-shot marker remains occupied even when its Weak no longer upgrades.
        ensure!(
            projection.consumed.is_none(),
            "native phase input already consumed"
        );
        projection.consumed = Some(Arc::downgrade(&input));
        Ok(input)
    }
    pub(super) fn acknowledge(&self, thread: &str, turn: Option<&str>) -> Result<()> {
        ensure!(
            !thread.is_empty()
                && thread.len() <= 512
                && turn.is_none_or(|v| !v.is_empty() && v.len() <= 512)
                && self
                    .expected_thread
                    .as_deref()
                    .is_none_or(|expected| expected == thread)
                && (self.owner.allocation().facts().provider != "codex" || turn.is_some()),
            "native phase acknowledgement differs from owned input"
        );
        let mut acknowledgement = self
            .acknowledgement
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase acknowledgement unavailable"))?;
        if let Some(old) = acknowledgement.as_ref() {
            ensure!(
                old.thread == thread && old.turn.as_deref() == turn,
                "native phase acknowledgement changed"
            );
        } else {
            *acknowledgement = Some(Acknowledgement {
                thread: thread.into(),
                turn: turn.map(str::to_owned),
            });
        }
        Ok(())
    }
    pub(crate) fn marker(&self) -> &OriginalMarker {
        self.owner.marker()
    }
    pub(crate) fn allocation(&self) -> &NativeAllocation {
        self.owner.allocation()
    }
    pub(crate) fn effect(&self) -> OperationId {
        self.effect
    }
    pub(crate) fn frame_sha256(&self) -> &str {
        &self.frame_sha256
    }
}
impl OwnedPhaseSettlement {
    pub(crate) fn belongs_to(&self, owner: &NativePhaseSession) -> bool {
        std::ptr::eq(self.owner.as_ref(), owner)
    }
    pub(super) fn completed(
        consumed: Arc<ConsumedPhaseInput>,
        terminal: Arc<NativeTerminal>,
        unit: ExecutionUnit,
        receipt: native_result::NativeResultReceipt,
        session: Session,
        record_version: u64,
    ) -> Result<Arc<Self>> {
        let owner = consumed.owner.clone();
        let facts = owner.allocation().facts();
        // The actual terminal transaction changes only receipt authority. Keep
        // every bounded answer/protocol/diagnostic byte tied to that SAME saved
        // terminal instead of accepting matching selected hashes or public IDs.
        let mut expected_receipt = terminal.receipt().clone();
        expected_receipt.authority = native_result::ReceiptAuthority::OwnedTerminal;
        ensure!(
            terminal.receipt().observed_work == WorkOutcome::Success
                && receipt == expected_receipt
                && receipt.observed_work == WorkOutcome::Success
                && receipt.authority == native_result::ReceiptAuthority::OwnedTerminal
                && receipt.invocation_id == facts.invocation_id
                && receipt.unit_id == facts.unit_id
                && receipt.session_id == facts.session_id
                && receipt.scope == *facts.scope
                && receipt.generation == facts.generation
                && receipt.owner_epoch == facts.epoch
                && receipt.provider == facts.provider
                && unit.id == facts.unit_id
                && unit.scope == *facts.scope
                && unit.generation == facts.generation
                && unit.owner_epoch == facts.epoch
                && unit.provider == facts.provider
                && unit.worktree == facts.path
                && unit.profile_digest == facts.profile_digest
                && unit.work == Some(WorkOutcome::Success)
                && !unit.native_effects_open
                && unit.state == UnitState::WorkKnown
                && unit.session_id == Some(facts.session_id)
                && session.id == facts.session_id
                && session.scope == *facts.scope
                && session.agent == facts.alias
                && session.provider == facts.provider
                && session.role == facts.role
                && session.worktree == facts.path
                && session.model.as_deref() == facts.model
                && session.effort.as_deref() == facts.effort
                && session.state == SessionState::Exited,
            "native phase lacks actual owned logical success"
        );
        let (thread, turn) = {
            let acknowledgement = consumed
                .acknowledgement
                .lock()
                .map_err(|_| anyhow::anyhow!("native phase acknowledgement unavailable"))?;
            let ack = acknowledgement
                .as_ref()
                .context("native phase input lacks actual acknowledgement")?;
            ensure!(
                receipt.native_thread.as_deref() == Some(ack.thread.as_str())
                    && receipt.native_turn == ack.turn
                    && session.native_ref.as_deref() == Some(ack.thread.as_str()),
                "native phase terminal acknowledgement changed"
            );
            (ack.thread.clone(), ack.turn.clone())
        };
        owner.project(&session, record_version)?;
        owner.revoke();
        let settlement = Arc::new(Self {
            owner: owner.clone(),
            consumed,
            terminal,
            unit,
            receipt,
            session,
            record_version,
            thread,
            turn,
        });
        let mut projection = owner
            .projection
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase projection unavailable"))?;
        ensure!(
            projection.settlement.is_none(),
            "native phase terminal already projected"
        );
        projection.settlement = Some(Arc::downgrade(&settlement));
        Ok(settlement)
    }
    pub(crate) fn marker(&self) -> &OriginalMarker {
        self.owner.marker()
    }
    pub(crate) fn allocation(&self) -> &NativeAllocation {
        self.owner.allocation()
    }
    pub(crate) fn consumed(&self) -> &ConsumedPhaseInput {
        &self.consumed
    }
    pub(crate) fn terminal(&self) -> &NativeTerminal {
        &self.terminal
    }
    pub(crate) fn unit(&self) -> &ExecutionUnit {
        &self.unit
    }
    pub(crate) fn receipt(&self) -> &native_result::NativeResultReceipt {
        &self.receipt
    }
    pub(crate) fn session(&self) -> &Session {
        &self.session
    }
    pub(crate) fn record_version(&self) -> u64 {
        self.record_version
    }
    pub(crate) fn thread(&self) -> &str {
        &self.thread
    }
    pub(crate) fn turn(&self) -> Option<&str> {
        self.turn.as_deref()
    }
}
