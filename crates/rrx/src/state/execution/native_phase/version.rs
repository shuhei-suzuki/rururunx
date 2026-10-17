//! Private original-actor version intent and owned-observation inventory CAS.
//! Compatible effect writers remain journal writers, never observation issuers.
use super::*;
use crate::execution::native::readonly::{
    ConversionIdentity, GIT_ACTIONS, NativeGitSourceSeal, NativePhaseHelperAction, PhaseGitAction,
    require_conversion,
};
use crate::execution::native::version::NativeVersionObservation;
use crate::runtime::phase_effect_admission::PhaseEffectAdmissionGuard;
use rusqlite::{Connection, types::ValueRef};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

mod closure;
pub(crate) use closure::NativeVersionClosurePlan;
pub(super) use closure::{LatestUnitImage, RetiredUnitImage};

const ROWS: usize = 256;
const BODY_BYTES: usize = 8192;
const INVENTORY_BYTES: usize = 2 * 1024 * 1024;
const SELECT: &str = "SELECT id,unit_id,project_id,goal_id,task_id,idempotency_key,state,body,version FROM managed_effects WHERE unit_id=?1 ORDER BY id LIMIT 257";
const SHAPES: &str = "SELECT typeof(id),length(CAST(id AS BLOB)),typeof(unit_id),length(CAST(unit_id AS BLOB)),typeof(project_id),length(CAST(project_id AS BLOB)),typeof(goal_id),length(CAST(goal_id AS BLOB)),typeof(task_id),length(CAST(task_id AS BLOB)),typeof(idempotency_key),length(CAST(idempotency_key AS BLOB)),typeof(state),length(CAST(state AS BLOB)),typeof(body),length(CAST(body AS BLOB)),typeof(version),version,state IN ('pending','confirmed','resolved','unknown') FROM managed_effects WHERE unit_id=?1 ORDER BY id LIMIT 257";

#[derive(Debug)]
struct InventoryWorkLimit;
impl std::fmt::Display for InventoryWorkLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Native effect InventoryWorkLimit; original operation held")
    }
}
impl std::error::Error for InventoryWorkLimit {}

thread_local! { static ACTIVE_BUDGET: Cell<bool> = const { Cell::new(false) }; }
/// Declaration precedes statement creation, so their borrows unwind first.
/// This scope is the sole progress-hook installer on these connections.
pub(super) struct InventoryBudget<'a> {
    connection: &'a Connection,
    interrupted: Arc<AtomicBool>,
}
impl<'a> InventoryBudget<'a> {
    pub(super) fn new(connection: &'a Connection) -> Result<Self> {
        ensure!(
            !ACTIVE_BUDGET.with(|active| active.replace(true)),
            "nested Native inventory budget"
        );
        let interrupted = Arc::new(AtomicBool::new(false));
        let flag = interrupted.clone();
        let mut callbacks = 0u32;
        connection.progress_handler(
            1000,
            Some(move || {
                callbacks = callbacks.saturating_add(1);
                let exhausted = callbacks >= 1000;
                if exhausted {
                    flag.store(true, Ordering::Relaxed);
                }
                exhausted
            }),
        );
        Ok(Self {
            connection,
            interrupted,
        })
    }
    pub(super) fn finish<T>(&self, result: Result<T>) -> Result<T> {
        if self.interrupted.load(Ordering::Relaxed) {
            Err(InventoryWorkLimit.into())
        } else {
            result
        }
    }
}
impl Drop for InventoryBudget<'_> {
    fn drop(&mut self) {
        self.connection.progress_handler(0, None::<fn() -> bool>);
        ACTIVE_BUDGET.with(|active| active.set(false));
    }
}

#[derive(Clone, PartialEq, Eq)]
struct EffectImage {
    text: [String; 8],
    version: u64,
}
fn shape(lengths: &[usize; 8], version: u64) -> Result<usize> {
    ensure!(
        lengths[..5].iter().all(|n| *n == 36),
        "Native effect indexed identity shape differs"
    );
    ensure!(
        lengths[5] > 0
            && lengths[5] <= 256
            && lengths[6] > 0
            && lengths[6] <= 9
            && lengths[7] <= BODY_BYTES
            && version > 0
            && version <= i64::MAX as u64,
        "Native effect indexed/body bound exceeded"
    );
    lengths.iter().try_fold(8usize * 8 + 8 + 8, |sum, n| {
        sum.checked_add(*n)
            .context("Native inventory size overflow")
    })
}
impl EffectImage {
    fn bytes(&self) -> Result<usize> {
        shape(&std::array::from_fn(|i| self.text[i].len()), self.version)
    }
    fn decode(&self, unit: UnitId) -> Result<ManagedEffect> {
        self.bytes()?;
        for raw in &self.text[..5] {
            ensure!(
                uuid::Uuid::parse_str(raw)?.to_string() == *raw,
                "Native effect noncanonical identity"
            );
        }
        let value = crate::execution::strict_json::decode(
            self.text[7].as_bytes(),
            crate::execution::strict_json::Limits {
                frame_bytes: BODY_BYTES,
                depth: 12,
                nodes: 256,
                string_bytes: 4096,
                total_string_bytes: BODY_BYTES,
                object_entries: 64,
                array_entries: 16,
            },
        )?;
        let effect: ManagedEffect = serde_json::from_value(value.clone())?;
        ensure!(
            serde_json::to_value(&effect.receipt)? == value["receipt"],
            "Native receipt classification lost during deserialize"
        );
        // This inventory includes every kind, including older dispatch and
        // Docker rows. Only P helpers produce the finite 16/128 vocabulary.
        ensure!(
            !effect.kind.starts_with("native_phase_")
                || (effect.receipt.len() <= 16
                    && effect
                        .receipt
                        .iter()
                        .all(|(k, v)| k.len() <= 128 && v.len() <= 128)),
            "Native finite receipt bound exceeded"
        );
        let (p, g, t) = scope_keys(&effect.scope)?;
        ensure!(
            effect.id.to_string() == self.text[0]
                && effect.unit_id == unit
                && effect.unit_id.to_string() == self.text[1]
                && p == self.text[2]
                && g == self.text[3]
                && t == self.text[4]
                && effect.idempotency_key == self.text[5]
                && key(effect.state) == self.text[6]
                && effect.version == self.version,
            "Native effect complete indexed/body image differs"
        );
        Ok(effect)
    }
    fn generated(effect: &ManagedEffect) -> Result<Self> {
        let (p, g, t) = scope_keys(&effect.scope)?;
        let image = Self {
            text: [
                effect.id.to_string(),
                effect.unit_id.to_string(),
                p,
                g,
                t,
                effect.idempotency_key.clone(),
                key(effect.state),
                serde_json::to_string(effect)?,
            ],
            version: effect.version,
        };
        image.decode(effect.unit_id)?;
        Ok(image)
    }
    fn values(&self) -> Vec<SqlValue> {
        self.text
            .iter()
            .map(|v| SqlValue::Text(v.clone()))
            .chain(std::iter::once(SqlValue::Integer(self.version as i64)))
            .collect()
    }
}
#[derive(Clone, PartialEq, Eq)]
struct Inventory {
    rows: Vec<EffectImage>,
}
impl Inventory {
    fn validate_bound(&self) -> Result<()> {
        ensure!(
            self.rows.len() <= ROWS,
            "Native complete effect row bound exceeded"
        );
        let bytes = self.rows.iter().try_fold(16usize, |sum, row| {
            sum.checked_add(row.bytes()?)
                .context("Native inventory overflow")
        })?;
        ensure!(
            bytes <= INVENTORY_BYTES,
            "Native complete effect inventory exceeds inclusive 2-MiB bound"
        );
        Ok(())
    }
    /// No TEXT values are copied until the complete first pass qualifies.
    /// Both passes belong to the caller's SAME transaction and VM budget.
    fn read(tx: &Transaction<'_>, unit: UnitId) -> Result<Self> {
        let id = unit.to_string();
        let mut shapes = Vec::new();
        let mut bytes = 16usize;
        {
            let mut statement = tx.prepare(SHAPES)?;
            let mut rows = statement.query([&id])?;
            while let Some(row) = rows.next()? {
                ensure!(
                    shapes.len() < ROWS,
                    "Native complete effect inventory 257th row"
                );
                let mut lengths = [0usize; 8];
                for (i, len) in lengths.iter_mut().enumerate() {
                    ensure!(
                        row.get_ref(i * 2)?.as_str()? == "text",
                        "Native effect indexed value is not TEXT"
                    );
                    *len = usize::try_from(row.get::<_, i64>(i * 2 + 1)?)?;
                }
                ensure!(
                    row.get_ref(16)?.as_str()? == "integer",
                    "Native effect version is not INTEGER"
                );
                let version = u64::try_from(row.get::<_, i64>(17)?)?;
                ensure!(
                    row.get::<_, bool>(18)?,
                    "Native effect state is not an allowed token"
                );
                bytes = bytes
                    .checked_add(shape(&lengths, version)?)
                    .context("Native inventory overflow")?;
                ensure!(
                    bytes <= INVENTORY_BYTES,
                    "Native inventory first pass exceeds inclusive 2-MiB bound"
                );
                shapes.push((lengths, version));
            }
        }
        let mut extracted = Vec::with_capacity(shapes.len());
        {
            let mut statement = tx.prepare(SELECT)?;
            let mut rows = statement.query([&id])?;
            while let Some(row) = rows.next()? {
                let expected = shapes
                    .get(extracted.len())
                    .context("Native inventory pass count changed")?;
                let mut lengths = [0usize; 8];
                for (i, len) in lengths.iter_mut().enumerate() {
                    let ValueRef::Text(raw) = row.get_ref(i)? else {
                        anyhow::bail!("Native indexed TEXT changed")
                    };
                    *len = raw.len();
                }
                let version = u64::try_from(row.get::<_, i64>(8)?)?;
                shape(&lengths, version)?;
                ensure!(
                    (lengths, version) == *expected,
                    "Native inventory passes differ"
                );
                let text: Vec<String> = (0..8)
                    .map(|i| row.get(i))
                    .collect::<rusqlite::Result<_>>()?;
                let image = EffectImage {
                    text: text
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("Native effect column count"))?,
                    version,
                };
                image.decode(unit)?;
                extracted.push(image);
            }
        }
        ensure!(
            extracted.len() == shapes.len(),
            "Native inventory extraction incomplete"
        );
        let inventory = Self { rows: extracted };
        inventory.validate_bound()?;
        Ok(inventory)
    }
    fn with(&self, image: EffectImage) -> Result<Self> {
        let mut next = self.clone();
        next.rows.push(image);
        next.rows.sort_by(|a, b| a.text[0].cmp(&b.text[0]));
        next.validate_bound()?;
        Ok(next)
    }
    fn reserve_git_batch(
        &self,
        budget: crate::execution::native::NativeEffectBudget,
    ) -> Result<()> {
        ensure!(
            self.rows.len() <= budget.git,
            "Git batch reserves 13 rows and future native input"
        );
        let maximum = shape(&[36, 36, 36, 36, 36, 256, 9, BODY_BYTES], i64::MAX as u64)?;
        let bytes = self.rows.iter().try_fold(16usize, |sum, row| {
            sum.checked_add(row.bytes()?)
                .context("Git inventory overflow")
        })?;
        ensure!(
            bytes
                .checked_add(GIT_ACTIONS * maximum)
                .is_some_and(|n| n <= INVENTORY_BYTES),
            "Git batch maximal row byte headroom absent"
        );
        Ok(())
    }
    fn with_git_intent(&self, image: EffectImage) -> Result<Self> {
        ensure!(
            self.rows.len() < 255,
            "Git helper reserves future native input"
        );
        self.with(image)
    }
    fn with_version_intent(
        &self,
        image: EffectImage,
        budget: crate::execution::native::NativeEffectBudget,
    ) -> Result<Self> {
        ensure!(
            self.rows.len() <= budget.version,
            "version helper reserves one future input slot"
        );
        self.with(image)
    }
}

pub(crate) struct NativeVersionHelperPlan {
    ready: Arc<NativePreparationCommit>,
    before: Arc<Inventory>,
    pending: Arc<Inventory>,
    effect: EffectImage,
    action: NativePhaseHelperAction,
    seal: Option<Arc<NativeGitSourceSeal>>,
    conversion: Option<Arc<ConversionIdentity>>,
    directory: Option<Arc<crate::execution::native::metadata::SchemaDirectory>>,
    contract: Option<crate::execution::codex_wire::CodexWireContract>,
}
/// Known factual settlement of SAME owned observation. Issued only after the
/// exact closure transaction commits; subsequent helpers still need authority.
pub(crate) struct NativeHelperSettlementCommit {
    original: Arc<NativeHelperSettlementPlan>,
}
/// Current acknowledgement of the same actual readonly helper history only.
/// Nongrant for full preparation, registration, static admission and input;
/// no SQL lookup can construct its private original.
pub(crate) struct NativeHelperHistoryCommit {
    original: Arc<NativePreparationCommit>,
    history: Vec<Arc<NativeHelperSettlementCommit>>,
}
pub(super) struct NativeRegistrationInventory {
    unit: UnitId,
    expected: Inventory,
}
impl NativeRegistrationInventory {
    pub(super) fn validate_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        ensure!(
            Inventory::read(tx, self.unit)? == self.expected,
            "SAME registration helper-plus-intent inventory differs"
        );
        Ok(())
    }
}
impl NativeHelperHistoryCommit {
    pub(super) fn registration_inventory(
        &self,
        effect: &ManagedEffect,
    ) -> Result<NativeRegistrationInventory> {
        let tail = self
            .history
            .last()
            .context("closed helper history absent")?;
        let mut expected = (*tail.original.after).clone();
        expected.rows.push(EffectImage::generated(effect)?);
        expected.rows.sort_by(|a, b| a.text[0].cmp(&b.text[0]));
        expected.validate_bound()?;
        Ok(NativeRegistrationInventory {
            unit: effect.unit_id,
            expected,
        })
    }
    pub(crate) fn matches_prefix(
        &self,
        known: &Arc<NativePreparationCommit>,
        version: &Arc<NativeHelperSettlementCommit>,
    ) -> bool {
        Arc::ptr_eq(&self.original, known)
            && self
                .history
                .first()
                .is_some_and(|k| Arc::ptr_eq(k, version))
    }
    pub(crate) fn len(&self) -> usize {
        self.history
            .last()
            .map_or(0, |k| k.original.after.rows.len())
    }
    pub(super) fn validate_inventory(&self, tx: &Transaction<'_>) -> Result<()> {
        let tail = self
            .history
            .last()
            .context("closed helper history absent")?;
        ensure!(
            Inventory::read(
                tx,
                self.original.actor().launch().allocation().facts().unit_id
            )? == *tail.original.after,
            "SAME helper completion inventory changed"
        );
        Ok(())
    }
    pub(crate) fn matches_actor(
        &self,
        actor: &Arc<crate::execution::native::NativePreparationActor>,
    ) -> bool {
        !self.history.is_empty() && Arc::ptr_eq(self.original.actor(), actor)
    }
}
impl NativeHelperSettlementCommit {
    #[cfg(test)]
    pub(crate) fn observed_plan(&self) -> &Arc<NativeVersionHelperPlan> {
        &self.original.original
    }

    pub(crate) fn matches_plan(&self, plan: &Arc<NativeVersionHelperPlan>) -> bool {
        Arc::ptr_eq(&self.original.original, plan)
    }
}
pub(crate) struct NativeHelperIntentCommit {
    original: Arc<NativeVersionHelperPlan>,
}
impl NativeHelperIntentCommit {
    pub(crate) fn matches_plan(&self, plan: &Arc<NativeVersionHelperPlan>) -> bool {
        Arc::ptr_eq(&self.original, plan)
    }
}
impl NativeVersionHelperPlan {
    pub(crate) fn directory(
        &self,
    ) -> Option<&Arc<crate::execution::native::metadata::SchemaDirectory>> {
        self.directory.as_ref()
    }
    pub(crate) fn contract(&self) -> Option<&crate::execution::codex_wire::CodexWireContract> {
        self.contract.as_ref()
    }
    pub(crate) fn action(&self) -> &NativePhaseHelperAction {
        &self.action
    }
    pub(crate) fn actor(&self) -> &Arc<crate::execution::native::NativePreparationActor> {
        self.ready.actor()
    }
    pub(crate) fn git_seal(&self) -> Option<&Arc<NativeGitSourceSeal>> {
        self.seal.as_ref()
    }
    pub(crate) fn conversion(&self) -> Option<&Arc<ConversionIdentity>> {
        self.conversion.as_ref()
    }
    fn validate_ready(&self, tx: &Transaction<'_>) -> Result<()> {
        self.actor().validate_original()?;
        self.ready.validate_version_ready(tx)?;
        if let Some(seal) = &self.seal {
            seal.validate_deadline()?;
        }
        ensure!(
            tx.query_row(
                "SELECT epoch FROM runtime_epoch WHERE singleton=1",
                [],
                |r| r.get::<_, u64>(0)
            )? == self.actor().launch().allocation().facts().epoch,
            "Native runtime epoch differs"
        );
        Ok(())
    }
}
pub(crate) struct NativeHelperSettlementPlan {
    original: Arc<NativeVersionHelperPlan>,
    observation: Arc<NativeVersionObservation>,
    after: Arc<Inventory>,
    effect: EffectImage,
}
impl Store {
    pub(crate) fn confirm_phase_helper_history(
        &mut self,
        history: Vec<Arc<NativeHelperSettlementCommit>>,
        admission: &PhaseEffectAdmissionGuard,
    ) -> Result<NativeHelperHistoryCommit> {
        ensure!(
            !history.is_empty()
                && history.len() <= crate::execution::native::readonly::HELPER_LIMIT,
            "complete finite actual helper history absent"
        );
        let last = history.last().expect("checked history");
        let ready = last.original.original.ready.clone();
        // Qualifiers/hash/fs checks run before this method outside SharedStore.
        // Here only SAME original objects and already-owned physical images.
        let codex = ready.actor().launch().allocation().facts().provider == "codex";
        let prefix = if codex { 3 } else { 1 };
        ensure!(
            history.len() == GIT_ACTIONS + prefix,
            "complete fresh metadata plus Git history absent"
        );
        for (index, known) in history.iter().enumerate() {
            let original = &known.original.original;
            ensure!(
                Arc::ptr_eq(&original.ready, &ready) && known.original.observation.qualified(),
                "helper history replaced original readiness actor or observation"
            );
            let ordered = match (&original.action, index) {
                (NativePhaseHelperAction::Version, 0) => true,
                (NativePhaseHelperAction::Schema, 1) if codex => true,
                (NativePhaseHelperAction::Handshake, 2) if codex => true,
                (NativePhaseHelperAction::Git(a), i) if i >= prefix => {
                    *a == PhaseGitAction::ORDERED[i - prefix]
                }
                _ => false,
            };
            ensure!(
                ordered
                    && (index == 0
                        || Arc::ptr_eq(&original.before, &history[index - 1].original.after)),
                "helper history SAME ordered inventory link differs"
            );
        }
        selected_database(&self.connection, ready.actor().launch())?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        {
            let budget = InventoryBudget::new(&tx)?;
            budget.finish((|| {
                admission.validate_for(ready.actor().launch())?;
                ready.actor().validate_open()?;
                // Reuse the actual current tail's SAME readiness/epoch/seal
                // deadline under this Immediate after admission waiting.
                last.original.original.validate_ready(&tx)?;
                let current =
                    Inventory::read(&tx, ready.actor().launch().allocation().facts().unit_id)?;
                ensure!(
                    current == *last.original.after,
                    "helper history latest exact inventory differs"
                );
                for known in &history {
                    ensure!(
                        known.original.observation.complete()
                            && known.original.effect.text[6] == "confirmed"
                            && current.rows.contains(&known.original.effect)
                            && known
                                .original
                                .effect
                                .decode(ready.actor().launch().allocation().facts().unit_id)?
                                .receipt
                                == known.original.observation.safe_receipt(),
                        "helper history original observed/settled image differs"
                    );
                }
                Ok(())
            })())?;
        }
        tx.commit()?;
        Ok(NativeHelperHistoryCommit {
            original: ready,
            history,
        })
    }
    pub(crate) fn plan_phase_version_intent(
        runtime: &crate::execution::RuntimeOwner,
        ready: Arc<NativePreparationCommit>,
    ) -> Result<Arc<NativeVersionHelperPlan>> {
        ready.actor().validate_open()?;
        let launch = ready.actor().launch();
        let unit = launch.allocation().unit_snapshot();
        ensure!(
            unit.phase != WORKFLOW_SOURCE_BOOTSTRAP,
            "version helper excludes source bootstrap"
        );
        let before = snapshot(runtime, |tx| {
            let budget = InventoryBudget::new(tx)?;
            budget.finish((|| {
                ready.validate_version_ready(tx)?;
                ensure!(
                    !verification::is_command_unit(tx, unit.id)?,
                    "version helper excludes command-only verifier"
                );
                Inventory::read(tx, unit.id)
            })())
        })?;
        for row in &before.rows {
            let effect = row.decode(unit.id)?;
            ensure!(
                effect.scope == unit.scope
                    && effect.kind == "git_helper"
                    && matches!(effect.state, EffectState::Confirmed | EffectState::Resolved),
                "unexpected or unresolved original effect baseline"
            );
        }
        let id = OperationId::new();
        let facts = launch.allocation().facts();
        let identity = crate::execution::native_result::digest(
            format!(
                "{}:{}:{}:{}:version",
                facts.operation_id, facts.pair_id, facts.epoch, facts.profile_digest
            )
            .as_bytes(),
        );
        let effect = EffectImage::generated(&ManagedEffect {
            id,
            unit_id: unit.id,
            scope: unit.scope.clone(),
            kind: NativePhaseHelperAction::Version.kind().into(),
            idempotency_key: format!("native-version-{id}"),
            expected_target: format!("version:{identity}"),
            state: EffectState::Pending,
            receipt: BTreeMap::new(),
            version: 1,
        })?;
        let budget = crate::execution::native::native_effect_budget(facts.provider, facts.role)?;
        let prefix_rows = if facts.provider == "codex" {
            GIT_ACTIONS + 3
        } else {
            GIT_ACTIONS + 1
        };
        let bytes = before
            .rows
            .iter()
            .try_fold(16usize, |n, r| Ok::<_, anyhow::Error>(n + r.bytes()?))?;
        ensure!(
            bytes
                + prefix_rows * shape(&[36, 36, 36, 36, 36, 256, 9, BODY_BYTES], i64::MAX as u64)?
                <= INVENTORY_BYTES,
            "P prefix maximal image headroom absent"
        );
        let pending = Arc::new(before.with_version_intent(effect.clone(), budget)?);
        Ok(Arc::new(NativeVersionHelperPlan {
            ready,
            before: Arc::new(before),
            pending,
            effect,
            action: NativePhaseHelperAction::Version,
            seal: None,
            conversion: None,
            directory: None,
            contract: None,
        }))
    }
    pub(crate) fn plan_phase_git_intent(
        runtime: &crate::execution::RuntimeOwner,
        previous: Arc<NativeHelperSettlementCommit>,
        action: PhaseGitAction,
        seal: Arc<NativeGitSourceSeal>,
        conversion: Option<Arc<ConversionIdentity>>,
    ) -> Result<Arc<NativeVersionHelperPlan>> {
        let original = &previous.original.original;
        original.actor().validate_open()?;
        ensure!(
            seal.matches_actor(original.actor()),
            "Git seal original actor differs"
        );
        require_conversion(action, &seal, conversion.as_ref())?;
        seal.validate_deadline()?;
        ensure!(
            previous.original.observation.qualified()
                && previous.original.effect.text[6] == "confirmed",
            "previous original helper was not qualified/confirmed"
        );
        match original.action {
            NativePhaseHelperAction::Version | NativePhaseHelperAction::Handshake => {
                let codex = original.actor().launch().allocation().facts().provider == "codex";
                ensure!(
                    matches!(original.action, NativePhaseHelperAction::Handshake) == codex,
                    "Git requires fresh P metadata predecessor"
                );
                ensure!(
                    action == PhaseGitAction::SourceTop
                        && previous.original.observation.qualified(),
                    "Git requires SAME actual qualified version predecessor"
                );
                let facts = original.actor().launch().allocation().facts();
                previous.original.after.reserve_git_batch(
                    crate::execution::native::native_effect_budget(facts.provider, facts.role)?,
                )?;
            }
            NativePhaseHelperAction::Schema => anyhow::bail!("Git cannot skip fresh P Handshake"),
            NativePhaseHelperAction::Git(last) => ensure!(
                action.ordinal() == last.ordinal() + 1
                    && original
                        .seal
                        .as_ref()
                        .is_some_and(|s| Arc::ptr_eq(s, &seal)),
                "Git history order/seal differs"
            ),
        }
        // Snapshot is only a drift check. The authoritative predecessor object
        // remains the SAME already-acknowledged after, never reconstructed rows.
        snapshot(runtime, |tx| {
            let budget = InventoryBudget::new(tx)?;
            budget.finish((|| {
                original.validate_ready(tx)?;
                ensure!(
                    Inventory::read(tx, original.actor().launch().allocation().facts().unit_id)?
                        == *previous.original.after,
                    "Git predecessor exact inventory changed"
                );
                Ok(())
            })())
        })?;
        let before = previous.original.after.clone();
        let id = OperationId::new();
        let allocation = original.actor().launch().allocation();
        let effect = EffectImage::generated(&ManagedEffect {
            id,
            unit_id: allocation.facts().unit_id,
            scope: allocation.unit_snapshot().scope.clone(),
            kind: NativePhaseHelperAction::Git(action).kind().into(),
            idempotency_key: format!("native-git-{id}"),
            expected_target: seal.target(action),
            state: EffectState::Pending,
            receipt: BTreeMap::new(),
            version: 1,
        })?;
        let pending = Arc::new(before.with_git_intent(effect.clone())?);
        Ok(Arc::new(NativeVersionHelperPlan {
            ready: original.ready.clone(),
            before,
            pending,
            effect,
            action: NativePhaseHelperAction::Git(action),
            seal: Some(seal),
            conversion,
            directory: None,
            contract: None,
        }))
    }
    pub(crate) fn plan_phase_metadata_intent(
        runtime: &crate::execution::RuntimeOwner,
        previous: Arc<NativeHelperSettlementCommit>,
        action: NativePhaseHelperAction,
        directory: Option<Arc<crate::execution::native::metadata::SchemaDirectory>>,
    ) -> Result<Arc<NativeVersionHelperPlan>> {
        let original = &previous.original.original;
        original.actor().validate_open()?;
        ensure!(
            original.actor().launch().allocation().facts().provider == "codex"
                && previous.original.observation.qualified()
                && previous.original.effect.text[6] == "confirmed",
            "P metadata predecessor unqualified"
        );
        ensure!(
            match (&original.action, &action) {
                (NativePhaseHelperAction::Version, NativePhaseHelperAction::Schema) =>
                    directory.is_some(),
                (NativePhaseHelperAction::Schema, NativePhaseHelperAction::Handshake) =>
                    directory.is_none(),
                _ => false,
            },
            "P metadata order differs"
        );
        let contract = if matches!(action, NativePhaseHelperAction::Handshake) {
            Some(previous.original.observation.schema_contract()?)
        } else {
            None
        };
        if let Some(dir) = &directory {
            dir.verify(original.actor())?;
        }
        snapshot(runtime, |tx| {
            let budget = InventoryBudget::new(tx)?;
            budget.finish((|| {
                original.validate_ready(tx)?;
                ensure!(
                    Inventory::read(tx, original.actor().launch().allocation().facts().unit_id)?
                        == *previous.original.after,
                    "P metadata predecessor complete inventory changed"
                );
                Ok(())
            })())
        })?;
        let before = previous.original.after.clone();
        let facts = original.actor().launch().allocation().facts();
        let maximum =
            crate::execution::native::native_effect_budget(facts.provider, facts.role)?.git;
        ensure!(
            before.rows.len() < maximum,
            "P metadata row reserve exhausted"
        );
        // Reserve maximal private images for both metadata helpers and Git13.
        let remaining = if matches!(action, NativePhaseHelperAction::Schema) {
            GIT_ACTIONS + 2
        } else {
            GIT_ACTIONS + 1
        };
        let bytes = before
            .rows
            .iter()
            .try_fold(16usize, |n, r| Ok::<_, anyhow::Error>(n + r.bytes()?))?;
        ensure!(
            bytes + remaining * shape(&[36, 36, 36, 36, 36, 256, 9, BODY_BYTES], i64::MAX as u64)?
                <= INVENTORY_BYTES,
            "P metadata maximal byte reserve absent"
        );
        let id = OperationId::new();
        let effect = EffectImage::generated(&ManagedEffect {
            id,
            unit_id: facts.unit_id,
            scope: facts.scope.clone(),
            kind: action.kind().into(),
            idempotency_key: format!("native-{}-{id}", action.label()),
            expected_target: format!(
                "{}:{}",
                action.label(),
                crate::execution::native_result::digest(
                    format!(
                        "{}:{}:{}:{}",
                        facts.operation_id, facts.pair_id, facts.epoch, facts.profile_digest
                    )
                    .as_bytes()
                )
            ),
            state: EffectState::Pending,
            receipt: BTreeMap::new(),
            version: 1,
        })?;
        let pending = Arc::new(before.with(effect.clone())?);
        Ok(Arc::new(NativeVersionHelperPlan {
            ready: original.ready.clone(),
            before,
            pending,
            effect,
            action,
            seal: None,
            conversion: None,
            directory,
            contract,
        }))
    }
    pub(crate) fn reserve_phase_metadata_intent(
        &mut self,
        plan: Arc<NativeVersionHelperPlan>,
        admission: &PhaseEffectAdmissionGuard,
    ) -> Result<NativeHelperIntentCommit> {
        ensure!(
            matches!(
                plan.action,
                NativePhaseHelperAction::Schema | NativePhaseHelperAction::Handshake
            ) && plan.seal.is_none(),
            "metadata cannot consume a Git seal"
        );
        self.reserve_phase_helper_intent(plan, admission)
    }
    pub(crate) fn reserve_phase_version_intent(
        &mut self,
        plan: Arc<NativeVersionHelperPlan>,
        admission: &PhaseEffectAdmissionGuard,
    ) -> Result<NativeHelperIntentCommit> {
        ensure!(
            matches!(plan.action, NativePhaseHelperAction::Version) && plan.seal.is_none(),
            "version intent cannot admit Git"
        );
        self.reserve_phase_helper_intent(plan, admission)
    }
    pub(crate) fn reserve_phase_git_intent(
        &mut self,
        plan: Arc<NativeVersionHelperPlan>,
        admission: &PhaseEffectAdmissionGuard,
    ) -> Result<NativeHelperIntentCommit> {
        let NativePhaseHelperAction::Git(action) = plan.action else {
            anyhow::bail!("Git intent cannot admit version")
        };
        let seal = plan
            .seal
            .as_ref()
            .context("Git intent lacks original Source seal")?;
        ensure!(
            seal.matches_actor(plan.actor()),
            "Git intent actor/seal differs"
        );
        require_conversion(action, seal, plan.conversion.as_ref())?;
        self.reserve_phase_helper_intent(plan, admission)
    }
    fn reserve_phase_helper_intent(
        &mut self,
        plan: Arc<NativeVersionHelperPlan>,
        admission: &PhaseEffectAdmissionGuard,
    ) -> Result<NativeHelperIntentCommit> {
        selected_database(&self.connection, plan.actor().launch())?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        {
            let budget = InventoryBudget::new(&tx)?;
            budget.finish((|| {
                admission.validate_for(plan.actor().launch())?;
                plan.actor().validate_open()?;
                plan.validate_ready(&tx)?;
                plan.pending.validate_bound()?;
                ensure!(Inventory::read(&tx,plan.actor().launch().allocation().facts().unit_id)? == *plan.before,
                    "Native original intent complete baseline changed");
                ensure!(!tx.query_row("SELECT EXISTS(SELECT 1 FROM managed_effects WHERE id=?1 OR (unit_id=?2 AND idempotency_key=?3))",
                    params![plan.effect.text[0],plan.effect.text[1],plan.effect.text[5]],|r|r.get::<_,bool>(0))?, "Native original helper identity already exists");
                ensure!(tx.execute("INSERT INTO managed_effects(id,unit_id,project_id,goal_id,task_id,idempotency_key,state,body,version) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                    params_from_iter(plan.effect.values()))? == 1, "Native intent insertion missing");
                Ok(())
            })())?;
        }
        tx.commit()?;
        Ok(NativeHelperIntentCommit { original: plan })
    }
    pub(crate) fn validate_phase_version_fence(
        &mut self,
        plan: &Arc<NativeVersionHelperPlan>,
        intent: &NativeHelperIntentCommit,
    ) -> Result<()> {
        ensure!(
            intent.matches_plan(plan),
            "Native fence intent original differs"
        );
        selected_database(&self.connection, plan.actor().launch())?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        {
            let budget = InventoryBudget::new(&tx)?;
            budget.finish((|| {
                plan.actor().validate_open()?;
                plan.validate_ready(&tx)?;
                ensure!(
                    Inventory::read(&tx, plan.actor().launch().allocation().facts().unit_id)?
                        == *plan.pending,
                    "Native helper pending expected inventory changed"
                );
                Ok(())
            })())?;
        }
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn plan_phase_version_settlement(
        plan: Arc<NativeVersionHelperPlan>,
        observation: Arc<NativeVersionObservation>,
    ) -> Result<Arc<NativeHelperSettlementPlan>> {
        ensure!(
            observation.matches_plan(&plan),
            "Native settlement lacks same owned observation"
        );
        let mut effect = plan
            .effect
            .decode(plan.actor().launch().allocation().facts().unit_id)?;
        effect.version = effect
            .version
            .checked_add(1)
            .context("Native effect version overflow")?;
        effect.state = if observation.settle_confirmed() {
            EffectState::Confirmed
        } else {
            EffectState::Unknown
        };
        effect.receipt = observation.safe_receipt();
        let effect = EffectImage::generated(&effect)?;
        let after = Arc::new(plan.before.with(effect.clone())?);
        Ok(Arc::new(NativeHelperSettlementPlan {
            original: plan,
            observation,
            after,
            effect,
        }))
    }
}

#[cfg(test)]
mod inventory_tests {
    use super::*;
    fn tested_budget() -> crate::execution::native::NativeEffectBudget {
        crate::execution::native::native_effect_budget(
            "claude",
            crate::domain::SessionRole::Reviewer,
        )
        .unwrap()
    }

    #[test]
    fn nongrant_provider_role_budgets_gate_each_actual_inventory_consumer() {
        use crate::{
            domain::SessionRole::{ApprovalReviewer, Consultant, Executor, Reviewer},
            execution::native::native_effect_budget,
        };
        for (provider, role, version, git, prepared, setup, helpers) in [
            ("codex", Executor, 230, 233, 246, 8, 13),
            ("codex", Reviewer, 232, 235, 248, 6, 13),
            ("claude", Executor, 237, 238, 252, 1, 13),
            ("claude", Reviewer, 237, 238, 252, 1, 13),
        ] {
            let budget = native_effect_budget(provider, role).unwrap();
            let row = image(UnitId::new());
            let before = Inventory {
                rows: vec![row.clone(); version],
            };
            assert_eq!(
                before
                    .with_version_intent(row.clone(), budget)
                    .unwrap()
                    .rows
                    .len(),
                version + 1
            );
            let too_many = Inventory {
                rows: vec![row.clone(); version + 1],
            };
            assert!(too_many.with_version_intent(row.clone(), budget).is_err());
            assert_eq!(too_many.rows.len(), version + 1);
            let before = Inventory {
                rows: vec![row.clone(); git],
            };
            before.reserve_git_batch(budget).unwrap();
            let too_many = Inventory {
                rows: vec![row.clone(); git + 1],
            };
            assert!(too_many.reserve_git_batch(budget).is_err());
            assert_eq!(git + helpers + usize::from(provider == "claude"), prepared);
            budget.check_prepared(prepared).unwrap();
            assert!(budget.check_prepared(prepared + 1).is_err());
            assert!(prepared + 1 + setup <= 255);
            assert_eq!(
                prepared + 1 + setup + 1,
                if provider == "codex" { 256 } else { 255 }
            );
        }
        for role in [Executor, Reviewer, Consultant, ApprovalReviewer] {
            assert!(native_effect_budget("unknown", role).is_err());
        }
        for provider in ["codex", "claude"] {
            for role in [Consultant, ApprovalReviewer] {
                assert!(native_effect_budget(provider, role).is_err());
            }
        }
    }

    #[test]
    fn nongrant_git_inventory_reserves_13_rows_and_maximal_bytes() {
        let row = image(UnitId::new());
        let mut inventory = Inventory {
            rows: vec![row.clone(); 238],
        };
        inventory.reserve_git_batch(tested_budget()).unwrap();
        inventory.rows.push(row.clone());
        assert!(inventory.reserve_git_batch(tested_budget()).is_err());
        let mut inventory = Inventory {
            rows: vec![row.clone(); 238],
        };
        for row in &mut inventory.rows {
            row.text[7].clear();
        }
        let maximum = shape(&[36, 36, 36, 36, 36, 256, 9, BODY_BYTES], i64::MAX as u64).unwrap();
        let desired = INVENTORY_BYTES - GIT_ACTIONS * maximum;
        let used = 16
            + inventory
                .rows
                .iter()
                .map(|r| r.bytes().unwrap())
                .sum::<usize>();
        let mut remaining = desired - used;
        for row in &mut inventory.rows {
            let extra = remaining.min(BODY_BYTES - row.text[7].len());
            row.text[7].push_str(&"x".repeat(extra));
            remaining -= extra;
        }
        assert_eq!(remaining, 0);
        inventory.reserve_git_batch(tested_budget()).unwrap();
        inventory
            .rows
            .iter_mut()
            .find(|r| r.text[7].len() < BODY_BYTES)
            .unwrap()
            .text[7]
            .push('x');
        assert!(inventory.reserve_git_batch(tested_budget()).is_err());
        let full = Inventory {
            rows: vec![row.clone(); 255],
        };
        assert!(full.with_git_intent(row.clone()).is_err());
        let before = Inventory {
            rows: vec![row.clone(); 254],
        };
        assert_eq!(before.with_git_intent(row).unwrap().rows.len(), 255);
    }

    fn database() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        // Nongrant low-level inventory fixture, never a Native owner/actor.
        c.execute_batch("CREATE TABLE managed_effects(id TEXT,unit_id TEXT,project_id TEXT,goal_id TEXT,task_id TEXT,idempotency_key TEXT,state TEXT,body TEXT,version INTEGER)").unwrap();
        c
    }
    fn image(unit: UnitId) -> EffectImage {
        EffectImage::generated(&ManagedEffect {
            id: OperationId::new(),
            unit_id: unit,
            scope: Scope {
                project_id: ProjectId::new(),
                goal_id: Some(GoalId::new()),
                task_id: Some(TaskId::new()),
            },
            kind: "git_helper".into(),
            idempotency_key: "finite-git-helper".into(),
            expected_target: "/qualified/worktree".into(),
            state: EffectState::Confirmed,
            receipt: BTreeMap::new(),
            version: 1,
        })
        .unwrap()
    }
    fn insert(c: &Connection, row: &EffectImage) {
        c.execute(
            "INSERT INTO managed_effects VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params_from_iter(row.values()),
        )
        .unwrap();
    }
    #[test]
    fn nongrant_inventory_roundtrip_all_columns_and_strict_body() {
        let mut c = database();
        let unit = UnitId::new();
        let original = image(unit);
        insert(&c, &original);
        let tx = c.transaction().unwrap();
        let inventory = Inventory::read(&tx, unit).unwrap();
        assert!(inventory.rows == vec![original.clone()]);
        tx.commit().unwrap();
        c.execute(
            "UPDATE managed_effects SET idempotency_key=?1",
            ["index-drift"],
        )
        .unwrap();
        assert!(Inventory::read(&c.transaction().unwrap(), unit).is_err());
        c.execute(
            "UPDATE managed_effects SET idempotency_key=?1,body=?2",
            params![
                original.text[5],
                original.text[7].replacen('{', "{\"extra\":true,", 1)
            ],
        )
        .unwrap();
        assert!(Inventory::read(&c.transaction().unwrap(), unit).is_err());
    }
    /// I-P receipt values survive the real bounded decoder without granting any preparation authority.
    #[test]
    fn u5_receipt_decoder_preserves_classification_and_existing_bounds() {
        let mut c = database();
        let unit = UnitId::new();
        let mut effect = image(unit).decode(unit).unwrap();
        effect.kind = "native_phase_schema".into();
        effect.receipt = BTreeMap::from([
            ("ipc_marker_refused".into(), "1".into()),
            ("ipc_invalid".into(), "false".into()),
        ]);
        let row = EffectImage::generated(&effect).unwrap();
        insert(&c, &row);
        let tx = c.transaction().unwrap();
        let decoded = Inventory::read(&tx, unit).unwrap().rows[0]
            .decode(unit)
            .unwrap();
        assert_eq!(decoded.receipt, effect.receipt);
        tx.rollback().unwrap();
        for kind in ["native_phase_schema", "native_phase_future"] {
            effect.kind = kind.into();
            effect.receipt = (0..16)
                .map(|i| (format!("k{i}"), "x".repeat(128)))
                .collect();
            EffectImage::generated(&effect).unwrap();
            effect.receipt.insert("extra".into(), "x".into());
            assert!(EffectImage::generated(&effect).is_err());
            effect.receipt = BTreeMap::from([("key".into(), "x".repeat(129))]);
            assert!(EffectImage::generated(&effect).is_err());
        }
    }

    /// Shared inventories preserve existing dispatch and Docker receipt values.
    #[test]
    fn u5r1a_inventory_preserves_nonhelper_receipts() {
        let mut c = database();
        let unit = UnitId::new();
        for (kind, key, value) in [
            ("native_dispatch", "detail", "x".repeat(129)),
            ("native_dispatch", "detail", "x".repeat(256)),
            ("docker_target", "config", "61".repeat(200)),
        ] {
            let mut effect = image(unit).decode(unit).unwrap();
            effect.kind = kind.into();
            effect.state = EffectState::Unknown;
            effect.receipt.insert(key.into(), value);
            insert(&c, &EffectImage::generated(&effect).unwrap());
        }
        let tx = c.transaction().unwrap();
        let inventory = Inventory::read(&tx, unit).unwrap();
        assert_eq!(inventory.rows.len(), 3);
        for row in inventory.rows {
            assert!(
                row.decode(unit)
                    .unwrap()
                    .receipt
                    .values()
                    .next()
                    .unwrap()
                    .len()
                    > 128
            );
        }
    }

    #[test]
    fn nongrant_inventory_oversized_index_refuses_before_body_decode() {
        let mut c = database();
        let unit = UnitId::new();
        let row = image(unit);
        insert(&c, &row);
        c.execute(
            "UPDATE managed_effects SET idempotency_key=?1,body='invalid-json'",
            ["x".repeat(257)],
        )
        .unwrap();
        let error = Inventory::read(&c.transaction().unwrap(), unit)
            .err()
            .unwrap();
        assert!(error.to_string().contains("indexed/body bound"));
        c.execute(
            "UPDATE managed_effects SET idempotency_key='ok',project_id=?1",
            ["x".repeat(37)],
        )
        .unwrap();
        let error = Inventory::read(&c.transaction().unwrap(), unit)
            .err()
            .unwrap();
        assert!(error.to_string().contains("indexed identity shape"));
    }
    #[test]
    fn nongrant_inventory_256_257_and_inclusive_byte_boundaries() {
        let mut c = database();
        let unit = UnitId::new();
        for _ in 0..256 {
            insert(&c, &image(unit));
        }
        assert_eq!(
            Inventory::read(&c.transaction().unwrap(), unit)
                .unwrap()
                .rows
                .len(),
            256
        );
        insert(&c, &image(unit));
        assert!(
            Inventory::read(&c.transaction().unwrap(), unit)
                .err()
                .unwrap()
                .to_string()
                .contains("257th")
        );
        let row = image(unit);
        let mut inventory = Inventory {
            rows: vec![row; 256],
        };
        for row in &mut inventory.rows {
            row.text[7] = "x".repeat(7900);
        }
        let used = 16
            + inventory
                .rows
                .iter()
                .map(|r| r.bytes().unwrap())
                .sum::<usize>();
        assert!(used < INVENTORY_BYTES);
        let mut remaining = INVENTORY_BYTES - used;
        for row in &mut inventory.rows {
            let extra = remaining.min(BODY_BYTES - row.text[7].len());
            row.text[7].push_str(&"x".repeat(extra));
            remaining -= extra;
        }
        assert_eq!(remaining, 0);
        assert!(inventory.validate_bound().is_ok());
        let row = inventory
            .rows
            .iter_mut()
            .find(|row| row.text[7].len() < BODY_BYTES)
            .unwrap();
        row.text[7].push('x');
        assert!(inventory.validate_bound().is_err());
        assert!(shape(&[36, 36, 36, 36, 36, 1, 7, BODY_BYTES], 1).is_ok());
        assert!(shape(&[36, 36, 36, 36, 36, 1, 7, BODY_BYTES + 1], 1).is_err());
    }
    #[test]
    fn nongrant_vm_budget_is_cumulative_and_resets_on_error_and_unwind() {
        let c = database();
        let query = "WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<30000) SELECT sum(x) FROM n";
        {
            let budget = InventoryBudget::new(&c).unwrap();
            assert!(
                budget
                    .finish(
                        c.query_row(query, [], |r| r.get::<_, i64>(0))
                            .map_err(Into::into)
                    )
                    .is_ok()
            );
            let error = budget
                .finish(
                    c.query_row(query, [], |r| r.get::<_, i64>(0))
                        .map_err(Into::into),
                )
                .err()
                .unwrap();
            assert!(error.downcast_ref::<InventoryWorkLimit>().is_some());
        }
        assert!(c.query_row(query, [], |r| r.get::<_, i64>(0)).is_ok());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _budget = InventoryBudget::new(&c).unwrap();
            let mut statement = c.prepare("SELECT 1").unwrap();
            let _rows = statement.query([]).unwrap();
            panic!("actual enclosing Rust unwind");
        }));
        assert!(result.is_err());
        assert!(c.query_row(query, [], |r| r.get::<_, i64>(0)).is_ok());
        assert!(InventoryBudget::new(&c).is_ok());
    }
    #[test]
    fn nongrant_version_postimage_reserves_slot_and_charges_new_body() {
        let unit = UnitId::new();
        let row = image(unit);
        let baseline = Inventory {
            rows: vec![row.clone(); 237],
        };
        assert_eq!(
            baseline
                .with_version_intent(row.clone(), tested_budget())
                .unwrap()
                .rows
                .len(),
            238
        );
        let full = Inventory {
            rows: vec![row.clone(); 255],
        };
        assert!(
            full.with_version_intent(row.clone(), tested_budget())
                .is_err()
        );
        let mut baseline = Inventory {
            rows: vec![row.clone(); 255],
        };
        for row in &mut baseline.rows {
            row.text[5] = "x".repeat(256);
            row.text[7].clear();
        }
        let desired = INVENTORY_BYTES - row.bytes().unwrap() + 1;
        let used = 16
            + baseline
                .rows
                .iter()
                .map(|row| row.bytes().unwrap())
                .sum::<usize>();
        let mut remaining = desired - used;
        for row in &mut baseline.rows {
            let extra = remaining.min(BODY_BYTES - row.text[7].len());
            row.text[7].push_str(&"x".repeat(extra));
            remaining -= extra;
        }
        assert_eq!(remaining, 0);
        assert!(baseline.validate_bound().is_ok());
        assert!(baseline.with(row.clone()).is_err());
        baseline
            .rows
            .iter_mut()
            .find(|r| !r.text[7].is_empty())
            .unwrap()
            .text[7]
            .pop();
        assert!(baseline.with(row.clone()).is_ok());
        let pending = baseline.with(row.clone()).unwrap();
        let mut receipt_postimage = row;
        receipt_postimage.text[7].push('x');
        assert!(baseline.with(receipt_postimage).is_err());
        assert!(pending.validate_bound().is_ok());
    }
    #[test]
    fn nongrant_unrelated_history_is_bounded_by_vm_work_without_partial_inventory() {
        let mut c = database();
        c.execute_batch("WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<300000) INSERT INTO managed_effects SELECT '', 'unrelated', '', '', '', '', '', '', 1 FROM n").unwrap();
        let tx = c.transaction().unwrap();
        {
            let budget = InventoryBudget::new(&tx).unwrap();
            let error = budget
                .finish(Inventory::read(&tx, UnitId::new()))
                .err()
                .unwrap();
            assert!(error.downcast_ref::<InventoryWorkLimit>().is_some());
        }
        tx.rollback().unwrap();
        assert_eq!(
            c.query_row("SELECT 1", [], |r| r.get::<_, i64>(0)).unwrap(),
            1
        );
    }
}
