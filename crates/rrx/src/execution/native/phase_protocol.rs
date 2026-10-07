//! Private facts issued by the actual managed Native registration/input/terminal
//! path. Persisted DTOs cannot recreate these handles. This module does not make
//! composition available; the Root marker/pair transaction remains required.
use super::*;
use crate::state::{KnownTransportRegistration, NativeTransportStartPlan, RegistrationAck};
use crate::{
    execution::phase::NativeAllocation,
    state::managed_binding::{OriginalMarker, PhaseLaunchParts},
};
use std::sync::{
    OnceLock,
    atomic::{AtomicU8, Ordering},
};
const CANDIDATE: u8 = 0;
const LIVE: u8 = 1;
const REVOKED: u8 = 2;
pub(crate) enum Activation {
    Live,
    RevokedKnown,
    Mismatch,
}
fn activate_ack(
    ack_cell: &OnceLock<RegistrationAck>,
    state: &AtomicU8,
    ack: RegistrationAck,
) -> Activation {
    if ack_cell.set(ack).is_err() {
        return Activation::Mismatch;
    }
    match state.compare_exchange(CANDIDATE, LIVE, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(_) => Activation::Live,
        Err(REVOKED) => Activation::RevokedKnown,
        _ => Activation::Mismatch,
    }
}
fn known_ack(state: &AtomicU8, ack: &OnceLock<RegistrationAck>) -> Result<RegistrationAck> {
    ensure!(
        matches!(state.load(Ordering::SeqCst), LIVE | REVOKED),
        "unregistered Candidate cannot close"
    );
    ack.get()
        .copied()
        .context("SAME activation acknowledgement absent")
}

/// Genuine registered owner. Only the actual Native module can issue it, after
/// its managed registration transaction succeeds with the original launch.
pub(crate) struct NativePhaseSession {
    launch: Arc<PhaseLaunchParts>,
    projection: Mutex<Projection>,
    origin: Arc<NativeTransportStartPlan>,
    ack: OnceLock<RegistrationAck>,
    state: AtomicU8,
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
pub(crate) struct PhaseActor {
    pub(super) owner: Arc<NativePhaseSession>,
    retained: Mutex<RetainedProofs>,
}
struct RetainedProofs {
    dispatches: BTreeMap<OperationId, Arc<crate::state::NativeDispatchCommit>>,
    consumed: Option<Arc<ConsumedPhaseInput>>,
    settlement: Option<Arc<OwnedPhaseSettlement>>,
    terminal_plan: Option<Arc<crate::state::NativeTerminalPlan>>,
}
impl PhaseActor {
    pub(crate) fn is_candidate(&self) -> bool {
        self.owner.state.load(Ordering::SeqCst) == CANDIDATE && self.owner.ack.get().is_none()
    }
    /// Preallocated outside admission, with no registration or effect authority.
    pub(crate) fn prepared_candidate(plan: &Arc<NativeTransportStartPlan>) -> Result<Arc<Self>> {
        let owner = NativePhaseSession::candidate(plan.clone())?;
        Ok(Arc::new(Self {
            owner,
            retained: Mutex::new(RetainedProofs {
                dispatches: BTreeMap::new(),
                consumed: None,
                settlement: None,
                terminal_plan: None,
            }),
        }))
    }
    /// Admission remains held, SharedStore is released. No allocation or SQL.
    pub(crate) fn activate(&self, known: KnownTransportRegistration) -> Activation {
        let Some(ack) = known.activation(&self.owner.origin) else {
            return Activation::Mismatch;
        };
        activate_ack(&self.owner.ack, &self.owner.state, ack)
    }
    pub(super) fn retain_dispatch(
        &self,
        commit: crate::state::NativeDispatchCommit,
    ) -> Result<(OperationId, String, Option<String>, bool)> {
        ensure!(
            commit.belongs_to(&self.owner),
            "dispatch belongs to another actor"
        );
        let facts = commit.facts();
        let mut retained = self
            .retained
            .lock()
            .map_err(|_| anyhow::anyhow!("Native retention unavailable"))?;
        ensure!(
            retained.dispatches.len() < 256 && !retained.dispatches.contains_key(&facts.0),
            "Native retained dispatch profile exhausted"
        );
        retained.dispatches.insert(facts.0, Arc::new(commit));
        Ok(facts)
    }
    pub(super) fn dispatch(
        &self,
        id: OperationId,
    ) -> Result<Arc<crate::state::NativeDispatchCommit>> {
        self.retained
            .lock()
            .map_err(|_| anyhow::anyhow!("Native retention unavailable"))?
            .dispatches
            .get(&id)
            .cloned()
            .context("SAME actual dispatch commit absent")
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
    pub(super) fn terminal_plan(
        &self,
        runtime: &RuntimeOwner,
        terminal: &Arc<NativeTerminal>,
    ) -> Result<Arc<crate::state::NativeTerminalPlan>> {
        if let Some(plan) = self
            .retained
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase retention unavailable"))?
            .terminal_plan
            .clone()
        {
            ensure!(
                plan.belongs_to(&self.owner, terminal),
                "native phase terminal plan changed"
            );
            return Ok(plan);
        }
        // Full snapshots and encoding do not run under the actor retention or
        // SharedStore mutex. Install once only after that finite work finishes.
        let plan = crate::state::Store::plan_native_phase_terminal(
            runtime,
            &self.owner,
            terminal.clone(),
        )?;
        let mut retained = self
            .retained
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase retention unavailable"))?;
        if let Some(original) = &retained.terminal_plan {
            ensure!(
                original.belongs_to(&self.owner, terminal),
                "native phase terminal plan changed"
            );
            return Ok(original.clone());
        }
        retained.terminal_plan = Some(plan.clone());
        Ok(plan)
    }
    pub(super) fn replan_terminal_after_absence(
        &self,
        runtime: &RuntimeOwner,
        original: &Arc<crate::state::NativeTerminalPlan>,
        terminal: &Arc<NativeTerminal>,
    ) -> Result<Arc<crate::state::NativeTerminalPlan>> {
        ensure!(
            original.belongs_to(&self.owner, terminal) && original.confirmed_absent(runtime)?,
            "native terminal commit remains uncertain"
        );
        let next = crate::state::Store::plan_native_phase_terminal(
            runtime,
            &self.owner,
            terminal.clone(),
        )?;
        let mut retained = self
            .retained
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase retention unavailable"))?;
        let saved = retained
            .terminal_plan
            .as_ref()
            .context("native terminal plan lost")?;
        ensure!(
            Arc::ptr_eq(saved, original),
            "native terminal replan raced with another original plan"
        );
        retained.terminal_plan = Some(next.clone());
        Ok(next)
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
        images: crate::state::SettledTerminalImages,
    ) -> Result<Arc<OwnedPhaseSettlement>> {
        if let Some(previous) = self
            .retained
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase retention unavailable"))?
            .settlement
            .clone()
        {
            ensure!(
                Arc::ptr_eq(&previous.terminal, &terminal)
                    && previous.receipt == receipt
                    && previous.record_version == record_version
                    && serde_json::to_value(&previous.unit)? == serde_json::to_value(&unit)?
                    && serde_json::to_value(&previous.session)? == serde_json::to_value(&session)?,
                "native phase changed settlement replay"
            );
            // A replay keeps the first commit's sealed images.
            drop(images);
            return Ok(previous);
        }
        let consumed = self.consumed()?;
        let settlement = OwnedPhaseSettlement::completed(
            consumed,
            terminal,
            unit,
            receipt,
            session,
            record_version,
            images,
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
    /// Sealed rows of the SAME terminal commit; compared, never returned.
    images: crate::state::SettledTerminalImages,
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
    fn candidate(origin: Arc<NativeTransportStartPlan>) -> Result<Arc<Self>> {
        let launch = origin.launch().clone();
        let session = origin.session().clone();
        let record_version = 1;
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
            origin,
            ack: OnceLock::new(),
            state: AtomicU8::new(CANDIDATE),
        }))
    }
    pub(super) fn project(&self, session: &Session, record_version: u64) -> Result<()> {
        self.validate_known_registration()?;
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
                && old.recovery == session.recovery
                && old.pid.is_none_or(|pid| session.pid == Some(pid))
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
        self.state.store(REVOKED, Ordering::SeqCst);
    }
    pub(crate) fn is_live(&self) -> bool {
        self.state.load(Ordering::SeqCst) == LIVE && self.ack.get().is_some()
    }
    pub(crate) fn origin(&self) -> &Arc<NativeTransportStartPlan> {
        &self.origin
    }
    pub(crate) fn validate_known_registration(&self) -> Result<RegistrationAck> {
        known_ack(&self.state, &self.ack)
    }
    pub(crate) fn registered_readiness(&self) -> Result<u64> {
        self.origin.registered_readiness()
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
            live: self.is_live(),
        })
    }
}
impl NativePhaseBinding {
    pub(crate) fn owner_arc(&self) -> &Arc<NativePhaseSession> {
        &self.owner
    }
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
        self.live && self.owner.is_live()
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
            owner.is_live()
                && frame_sha256.len() == 64
                && frame_sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
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
        images: crate::state::SettledTerminalImages,
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
            images,
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
    /// Exact invocation, receipt, Session, owner, readiness and admission rows
    /// of the SAME terminal commit.
    pub(crate) fn validate_terminal_images_tx(&self, tx: &rusqlite::Transaction<'_>) -> Result<()> {
        self.images.validate_tx(tx)
    }
    /// The stored Unit row equals the terminal's sealed postimage exactly.
    pub(crate) fn validate_terminal_unit_tx(&self, tx: &rusqlite::Transaction<'_>) -> Result<()> {
        self.images.validate_unit_tx(tx)
    }
}

#[cfg(test)]
mod registration_primitive_tests {
    use super::*;
    fn ack() -> RegistrationAck {
        RegistrationAck {
            readiness: 3,
            unit_version: 7,
            source: crate::state::RegistrationAckSource::Committed,
        }
    }
    #[test]
    fn candidate_and_unacknowledged_revoked_cannot_close() {
        let cell = OnceLock::new();
        let state = AtomicU8::new(CANDIDATE);
        assert!(known_ack(&state, &cell).is_err());
        state.store(REVOKED, Ordering::SeqCst);
        assert!(known_ack(&state, &cell).is_err());
        let acknowledged = OnceLock::new();
        assert!(acknowledged.set(ack()).is_ok());
        state.store(CANDIDATE, Ordering::SeqCst);
        assert!(known_ack(&state, &acknowledged).is_err());
    }
    #[test]
    fn known_commit_activates_once_and_retains_ack_after_revocation() {
        let cell = OnceLock::new();
        let state = AtomicU8::new(CANDIDATE);
        assert!(matches!(
            activate_ack(&cell, &state, ack()),
            Activation::Live
        ));
        assert_eq!(known_ack(&state, &cell).unwrap().readiness, 3);
        state.store(REVOKED, Ordering::SeqCst);
        assert_eq!(known_ack(&state, &cell).unwrap().unit_version, 7);
        assert!(matches!(
            activate_ack(&cell, &state, ack()),
            Activation::Mismatch
        ));
        assert_eq!(state.load(Ordering::SeqCst), REVOKED);
    }
    #[test]
    fn stop_before_ack_is_retained_without_reopening() {
        let cell = OnceLock::new();
        let state = AtomicU8::new(REVOKED);
        assert!(matches!(
            activate_ack(&cell, &state, ack()),
            Activation::RevokedKnown
        ));
        assert_eq!(state.load(Ordering::SeqCst), REVOKED);
        assert!(known_ack(&state, &cell).is_ok());
    }
}
