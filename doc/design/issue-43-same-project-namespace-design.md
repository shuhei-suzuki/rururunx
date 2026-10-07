# Issue 43: SAME-Project retained namespace liveness design

Status: proposed HOW only. Author: claude-opus-5-5/high. Not reviewed, not
implemented, not tested, not Native/MVP qualified. Source implementation is NOT
authorized by this file. Root admission and one nonauthor Sol 6.1 high design
review come before any source change. There is no requirements document: the
existing WHAT is sufficient (§1).

## 0. Pins and evidence boundary

| Pin | Use |
| --- | --- |
| Root `646c344a75502e617350766af2f34b7105cccda4` (Root646) | Source baseline; namespace, ticket and writer line references |
| CA `ab0fcc4580896569917f89a7b2857426322293a7` (CAab0) | Actual retained activation consumer, Store activation branch, Driver custody and service |
| CA `40c9bc9f586b49ae5ddb25683b7d8110ac48d281` (CA40c9) | Observed failed genuine SAME-Project control |

`namespace.rs` and `ticket.rs` are byte-identical at all three pins. Root646 does
not contain the CA activation additions. Implementation must be built on a
reviewed composition that contains CAab0, or on a later reviewed CA head
re-inventoried against this design, before any activation positive is claimed.
CA has since advanced to `2c24f3f6e66f45692e416c77dc3e8d64fcbc9276`. These
evidence pins are deliberately not moved.

What was observed. The log has SHA256
`840f8d61bb6d57c7f257a882ab89b3165babaa26087d88b53772c5b925d4d109`, exit 101,
8 passed and 1 failed. In `ca3b_original_composition_cross_pairs_refuse_before_store`,
a retained Driver ended with `Driver namespace original row changed`. The later
valid-pair wait then expired at `activation.rs:201`. Only one place produces that
message: the exact foreign-row comparison in `NamespaceSnapshot::validate_current`
(namespace.rs:115-122). It runs after the count check passed and before
`check_collision`. The result is a setup/continuation failure. It is neither a
mutant kill nor a Native positive.

What was not observed: the Task UUID, the captured or current row contents, SQL
before/after images, capture or commit timestamps, and which Task committed
first. Two sequences are supported by the source but remain conditional and
unverified:

- S2: a sibling's first preparation commits after the other Task's initial
  capture. It moves the Task from version 1 to 2 and binds worktree/branch
  (Root preparation.rs:155-208,554-596).
- S3: both namespaces are current at S1, and one activation commits first. It
  moves the Task from version 2 to 3, context_version from 0 to 1, sets the
  revision and may raise the workflow (CA preparation.rs:80-110, CA
  state/mod.rs:1085-1116). The other retained plan then refuses.

This design repairs both sequences without choosing between them. It invents no
row and no timing.

## 1. WHAT status: existing WHAT sufficient

The behavior is already required; only the HOW is missing.

- Product Requirements:
  - §3 principle 4 (line 30): independent Tasks run concurrently in separate Git
    worktrees.
  - §6.3 (line 254): the scheduler may run independent ready Tasks in parallel.
  - §31 (line 768) and acceptance 1 and 38 (lines 1191, 1228): at least 4
    simultaneous Tasks, and ready independent Goal Tasks run in parallel.
  - Nothing limits this to distinct Projects. §32.2 isolates per-Project
    worktree paths, which is exactly the collision property kept here.
- Runtime/Scheduler requirements:
  - RS-R2 fairness: a held Task or large Project cannot consume every selection
    attempt.
  - RS-R3: lost, duplicate or coalesced events and a dropped driver future cannot
    strand durable ready work.
  - RS-R4: progress and factual observations must not rewrite sibling
    Task/native inputs.
  - RS-AC6: siblings progress.
- Managed binding MB2/MB6 and scheduler design §2/§5/§7 keep own-frame currency
  and the unchanged safety controls. CAS losers re-read or park, and backoff is
  visible and bounded.

The Project collision corpus is HOW-level nongrant safety evidence (namespace.rs:1-2).
A sibling's legitimate write to its own Task is neither authority for nor against
another Task, so refusing on it is a defect against the clauses above. No WHAT
clarification is needed.

## 2. Root cause

Each retained plan carries ONE immutable, complete corpus, captured before real
awaits:

- initial activation: CA workflow.rs:1012-1021, before Source, Context and S1;
- gates and first Executor: driven_initial.rs:99-100, 210-211, 249-250, 309-311;
- first preparation: ticket.rs:63-67.

The writer (Root state/mod.rs:1753-1755, CA1796-1798) requires every captured
row to stay byte-identical until the protected commit. That includes the complete
bodies of other Tasks and other Goals. No producer can refresh only the corpus,
and the plan cannot be rebuilt without replaying awaited Source or gate work. As
a result, any committed write to another Task row in the Project between capture
and apply permanently refuses the plan. This includes metadata writes that do
not collide at all.

Collision safety needs two things: a complete corpus that is current AT the
protected commit, and a collision check against it. It does not need the corpus
to be unchanged since capture.

## 3. Decision

Keep the existing writer consumer, full-corpus CAS and collision check exactly as
they are. Add three narrow private pieces:

1. Typed pre-write freshness classification inside the SAME Immediate
   transaction. If the corpus is stale:
   - roll back explicitly and confirm the connection settled (autocommit);
   - return the known-uncommitted outcome `NamespaceStale`, which does not arm
     reconciliation of the SAME plan.
2. Namespace-only recapture into the SAME retained plan:
   - runs outside the Store and custody locks;
   - reads inside a query-only coherent snapshot that first revalidates the SAME
     original ticket;
   - changes nothing else in the plan or ticket.
3. A bounded, cancellable, per-turn retry with backoff in the existing callers.

Rejected alternatives:

- extending the deadline;
- a distinct-Project or serialized control;
- removing or weakening freshness;
- comparing only collision projections. This needs a body decode or `json_extract`
  of up to 16 MiB under Store;
- re-reading the ticket or rebuilding the plan;
- new indexed worktree/branch columns. That is a schema change and is not
  authorized.

## 4. Private types, state and custody

All new items are crate-private, non-Clone and non-Deserialize, and have no
public constructor. None of them is a grant.

```rust
// state/runtime/driver/namespace.rs
pub(in crate::state) enum StaleKind { Inventory, Row }
pub(in crate::state) enum NamespaceFreshness { Current, Stale(StaleKind) }
impl NamespaceSnapshot {
    /// Same SQL as today's validate_current; stops at the first stale row.
    pub(in crate::state) fn freshness(&self, c: &Connection) -> Result<NamespaceFreshness>;
    /// Same meaning and messages as today, now implemented over freshness().
    pub(in crate::state) fn validate_current(&self, c: &Connection) -> Result<()>;
}
/// Typed marker. Only the Store stale branch produces it, after a settled rollback.
#[derive(Debug)] pub(crate) struct NamespaceStale { kind: StaleKind }
/// Typed marker. The per-turn bound is exhausted and the plan was abandoned.
#[derive(Debug)] pub(crate) struct NamespaceContended { attempts: u32 }

/// Presence is fixed when the original ticket is enriched. Only the SAME retained
/// plan replaces the contents. The mutex guards a pointer clone/swap only.
pub(in crate::state) struct NamespaceSlot {
    current: std::sync::Mutex<Option<Arc<NamespaceSnapshot>>>,
    generation: std::sync::atomic::AtomicU64,
}
impl NamespaceSlot {
    pub(in crate::state) fn empty() -> Self;
    pub(in crate::state) fn filled(s: NamespaceSnapshot) -> Self;
    pub(in crate::state) fn is_present(&self) -> bool;
    pub(in crate::state) fn current(&self) -> Result<Arc<NamespaceSnapshot>>;
    fn replace(&self, fresh: NamespaceSnapshot) -> Result<()>; // requires present
    #[cfg(test)] pub(in crate::state) fn generation(&self) -> u64;
}

pub(crate) const NAMESPACE_ATTEMPTS: u32 = 6;
const NAMESPACE_BACKOFF_MS: [u64; 5] = [0, 10, 20, 40, 80];
pub(crate) enum NamespaceStep<T> { Done(T), Retry }
/// One per Store-entry call site per turn; counts applies of ONE plan.
pub(crate) struct NamespaceRetry { applied: u32 }
impl NamespaceRetry {
    pub(crate) fn new() -> Self;
    pub(crate) async fn settle<T>(&mut self, plan: &Arc<DriverPreparationAdvance>,
        lifetime: &WorkerLifetime, outcome: Result<T>) -> Result<NamespaceStep<T>>;
    #[cfg(test)] pub(crate) fn applied(&self) -> u32;
}

// state/runtime/driver/ticket.rs
pub(super) namespace: NamespaceSlot, // was Option<NamespaceSnapshot>

// state/runtime/driver/preparation.rs
impl Applying<'_> {
    /// Stale branch only. The guard holds only a reference, so forget is sound.
    pub(in crate::state) fn known_uncommitted(self);
}
impl DriverPreparationAdvance {
    pub(in crate::state) fn input_namespace(&self) -> Result<Arc<NamespaceSnapshot>>;
    pub(in crate::state) fn namespace_precheck_tx(&self, ns: &NamespaceSnapshot,
        tx: &Transaction<'_>) -> Result<NamespaceFreshness>;
    pub(crate) fn recapture_namespace(self: &Arc<Self>) -> Result<()>;
}
```

Issuer and custody rules:

- The sole corpus issuer stays `NamespaceSnapshot::read`, with driver-module
  visibility. Four callers reach it: `read_driver_ticket`, `with_initial_namespace`,
  `with_gate_namespace` and `recapture_namespace`. Each first runs
  `validate_current_tx` of the SAME original ticket, in the same query-only
  coherent read (`managed_binding::snapshot`).
- Before reading, `recapture_namespace` requires three things: the plan is
  retained by its actual association slot (`is_retained`), `reconciliation_ready()`
  is false, and the slot is present. It checks retention again after the read,
  then swaps the pointer. During the read and decode it holds no Store, custody,
  registry, admission or slot lock.
- Slot presence never changes after ticket enrichment. Recapture cannot create a
  corpus for a plan that never had one, so later Preparing/base plans stay without
  one.
- `Applying` keeps today's rule: any Err or unwind of a protected apply arms
  reconciliation. There is one exception. Only the stale branch consumes the guard
  with `known_uncommitted`, and only after `tx.rollback()` succeeded and
  `connection.is_autocommit()` is true. The live caller keeps ownership of the
  plan.
- Abandonment: on exhaustion, cancellation or recapture failure, `settle` calls
  `plan.allow_reconciliation()`. The existing service pass (service.rs:20-50) then
  proves rollback through `validate_rollback` and retires custody. No plan is
  reminted.

## 5. Protected write algorithm (inside Store)

Freshness is classified after every existing own-authority and lifecycle check
and before the first protected write. Each attempt clones `input_namespace()`
once. The precheck and the writer use that same Arc.

W1 `Store::put_workflow_transition_inner` (CA672; Root665). Used by
`activate_driven_workflow`, `apply_driven_initial_gate` and
`reserve_driven_first_executor`:

- `let mut applying = driver_input.map(begin_input).transpose()?;` (CA692).
- Unchanged: validate_input_before_tx, Driver validate, Source before_write,
  Project/Goal versions, lifecycle, Session and lock guards, validate_transition
  and the verification rules.
- New, immediately before `plan.write_input_unit_tx` (CA898-902; Root891-895):
  1. Run the precheck.
  2. If it returns `Stale(kind)`: run `tx.rollback()?`, then
     `ensure!(self.connection.is_autocommit())`, then call `known_uncommitted` on
     the guard taken from `applying`, and return `NamespaceStale { kind }`.
  3. If rollback or the autocommit check fails, return an ordinary error and leave
     `applying` armed.
- Unchanged afterwards: Context, Unit, the Task writer (now given the same
  per-attempt Arc; CA1086-1091; Root1079-1084), the Record writer, the
  contract/activation writes, Source after_write, finish_input_tx, commit, and
  publication or Deferred handling.

W2 `Store::apply_driver_preparation` (CA622; Root529), first preparation only
(`plan.initial`):

- `let applying = Applying(plan);` (CA634; Root541).
- Unchanged: ticket validate_current_tx, outstanding effects, Driver validate,
  history, and the Project namespace parent/base-branch checks.
- New, immediately before `insert_unit` (CA657; Root564): the same stale branch.
- The writer call uses the same per-attempt Arc (CA661-668; Root569-575) and is
  otherwise unchanged.
- Preparing/base edges (`initial == false`) carry no corpus and never reach this
  branch.

W3 `put_task_tx_at` None route (Root1683,1757-1768; CA1726,1800-1811) is
unchanged. It reads and decodes the current Project corpus under its own
transaction, so it has no retained corpus that can go stale. It does not replace
W1 or W2, and it gains no new caller.

The writer `put_task_tx_at_with_namespace` (Root1685; CA1728) keeps its
signature. It still calls `validate_current` and then `check_collision`. Inside
the same Immediate transaction, its repeated freshness check cannot see a
different result from the precheck. It remains the actual collision consumer.

## 6. Retry algorithm (outside Store)

`settle(plan, lifetime, outcome)` runs these steps:

1. `applied += 1`. `Ok(v)` returns `Done(v)`.
2. An Err with no `NamespaceStale` in its chain is returned unchanged. It is
   already armed.
3. If the outcome is stale and `applied >= NAMESPACE_ATTEMPTS`: call
   `allow_reconciliation()` and return `NamespaceContended { attempts: applied }`.
4. Backoff:
   - base = `NAMESPACE_BACKOFF_MS[applied-1]`;
   - jitter = (low 64 bits of the selected Task UUID XOR `applied`) modulo
     `base+1`, which is 0 when base is 0;
   - `select! biased` on `lifetime.cancelled()`: if cancelled, abandon and return
     Err; otherwise sleep base+jitter.
   - No lock is held during backoff.
5. Call the `#[cfg(test)]` retry seam, which callers that have test hooks install
   through `NamespaceRetry`.
6. Call `plan.recapture_namespace()`. On Err, abandon and return it.
7. Return `Retry`.

Per-turn limits: at most 6 Immediate attempts, at most 5 recaptures, and at most
300 ms of total sleep (the bases sum to 150 ms and jitter never exceeds its base).

Every attempt reuses the same `Arc` plan together with its ticket, Source frame,
Unit, gate completion, activation roster and prescribed images. No awaited
Source, gate, helper, Git or native operation is repeated. On Err the caller's
`&mut Task` and `&mut Record` are untouched, because they are assigned only after
commit (CA1150-1151). The retry therefore passes identical inputs.

Caller wiring (no new Store parameters):

| Route | Caller (CAab0) | Wiring |
| --- | --- | --- |
| Initial activation | workflow.rs:1145-1164 | Loop per attempt: `admit_activation(&plan, lifetime)`, lock Store, call `activate_driven_workflow`, drop Store, drop the admission, then `settle`. The global FIFO `control_admission` and the Store are never held during backoff or recapture. Each attempt re-runs the existing admission checks (service running, composition, worker linkage, retention). A Deferred outcome goes to the unchanged recovery loop (1165-1198). |
| Gate Claim / Reserve / Complete | driven_initial.rs:110-112, 161-163, 380-382 | New private `WorkflowEngine::apply_driven_plan(&self, plan, lifetime, apply: fn(&mut Store, &Arc<DriverPreparationAdvance>) -> Result<()>)`. It loops with `settle` and locks Store only for each attempt. |
| First Executor reservation | driven_initial.rs:281-284 | Same helper with `Store::reserve_driven_first_executor`. |
| First preparation | attempts.rs:436-450 (same lines at Root646) | The Driver branch loops with `settle`, using the `driver` lifetime. The existing resource `admission` stays held across the bounded retry (at most 300 ms; it is not a Store, Root or custody lock). The non-driver branches are unchanged. |

`offer_driven_first_executor` (driven_initial.rs:170-235) carries a
gate-enriched ticket into a Source handoff but never calls a Task writer. It is
unchanged.

## 7. Consumer coverage

| Consumer | Effect |
| --- | --- |
| read_driver_ticket first-bootstrap capture (ticket.rs:63-67) | Wraps the same read in `NamespaceSlot::filled`; otherwise uses `empty`. |
| with_initial_namespace / with_gate_namespace (ticket.rs:98-123) | Assign `filled`. Same validation and read. |
| Presence checks in plan_initial_input, plan_initial_gate, plan_first_executor (CA preparation.rs:57, gates.rs:73, executor.rs:54) | `is_some()` becomes `is_present()`. Same refusal. |
| InitialGate Reserve/Claim/Complete, first Executor reservation, initial activation | W1 precheck plus bounded retry. Complete keeps its retained completion and never re-invokes the gate. |
| First preparation | W2 precheck plus bounded retry. |
| Preparing/base edges and helper ticket consumers (attempts.rs:99-159, git_io.rs:35-60, workflow_source.rs:552-581, state/execution/effects.rs:59,148, driver/marker.rs:168-174) | Consult no corpus; unchanged. The separate physical worktree/HEAD/cleanliness checks are unchanged. |
| validate_current_tx, validate_ancillary_tx, validate_selection_tx, validate_source_tx | Unchanged. They never consult the slot. |
| validate_rollback / validate_result / publish / reconcile (CA preparation.rs:549-620,694-760) | Unchanged. They never refresh or consult the slot. A stale attempt never reaches them. An abandoned plan reaches them through the existing service pass. |
| Service reconcile (service.rs:20-50) and WorkerLifetime Drop (driver.rs:87-104) | Code unchanged. The stale outcome is disarmed while the Store mutex is still held, so the service can never see a ready flag for it. Dropping the lifetime still arms reconciliation. |
| Generic None route callers (Root state/mod.rs:496,1089; runtime/goals.rs:178; execution.rs:898,1066; execution/artifacts.rs:323) | Unchanged; listed for inventory only. SourceStop, Cancel, G3 and artifact paths gain nothing. |

## 8. Failure classification and uncertain commit

| Point | Class | Durable state | Custody | Next |
| --- | --- | --- | --- | --- |
| Own authority, lifecycle, prerequisite, Source or Unit drift inside Immediate | Ordinary refusal | Rolled back | Armed; the service proves rollback and retires | Existing Driver outcome or hold |
| Precheck Stale with rollback settled | `NamespaceStale` (retryable) | Known uncommitted: no protected write ran, explicit rollback, autocommit confirmed | Retained, not armed | `settle` |
| Rollback error, or connection not autocommit | Ordinary error | Existing uncertainty | Armed | Existing reconcile |
| Collision at the writer against a current corpus | Refusal | Rolled back | Armed | Err; never retried |
| Recapture fails: own drift; malformed header, body or index; row or byte budget; owner retired | Refusal | None | Abandoned | Err |
| Retry bound exhausted | `NamespaceContended` | None | Abandoned | Err with the existing hold. Gate Complete stays Evaluating with its existing hold reason; the gate is not replayed |
| Cancellation during backoff | Refusal | None | Abandoned | Err |
| Commit error, or publication error outside activation | Existing | Existing | Armed | Existing publish/rollback reconcile |
| Activation post-commit publication | Existing `Deferred` | Committed | Retained | Unchanged CA recovery (workflow.rs:1165-1198) |
| Same plan delivered again after success | Refusal at `begin_input` / `is_retained` | No second write, audit or contract | Retired | Err |

`NamespaceStale` can occur only before any protected write in its attempt, so it
can never be confused with an uncertain commit. A retry never follows a commit
attempt.

## 9. Exact authority that remains unchanged

The following checks and inputs stay exactly as they are:

- the SAME Runtime instance and epoch;
- the owner pointer and the live association;
- complete Project, Goal and Task bodies, versions, issue index and Project root;
- the current definition and Goal lifecycle;
- Driver row id, epoch, version, state and body, and the association cache;
- the claim and the ScopePlan's full pins;
- the Workflow, the latest Context and the complete WorktreeLock set;
- the selected generation, the active Unit and the full Unit body, path and branch;
- the selected native port and provider;
- the exact retained Source7 row and its absence inventory;
- the exact Task and Workflow rows of every hard prerequisite (claim.rs:206-229).
  A hard predecessor that also appears in the collision corpus does NOT make its
  prerequisite check expendable;
- the prescribed Task, Workflow, Context, Unit and Driver post-images;
- the activation roster and contract, and the gate receipt;
- the outstanding-effect refusal and the Project parent/base-branch checks;
- the generic writer's activity, immutable-binding, path/branch pairing and
  normal-direct-child checks.

Authentication, settings, hooks, permissions, protected base, and the
preflight/helper/source checks are inherited unchanged. No schema, guard layout,
public protocol, Agent capability or permission changes.

## 10. Freshness safety replacement/recapture boundary

Replaced: only the assumption that a corpus captured before awaits must still be
current at commit. Not replaced: the in-transaction requirement that the corpus
used for the collision check is complete and exactly current at commit.

Recapture may replace exactly one value, the plan's `NamespaceSnapshot`, and only
under all of these conditions:

- the new value is a complete, bounded corpus produced by the sole issuer;
- it is read in a coherent query-only snapshot of the owner's own state database;
- the SAME original ticket revalidated as current in that same snapshot;
- the plan is retained and has not been abandoned.

Recapture cannot change any ticket field, plan field, prescribed image, pin,
Source frame, Unit, admission or Driver binding. No authority validator consults
the corpus.

At commit the writer still rejects:

- any inserted, deleted, changed, malformed or surplus row (count sentinel 4097
  plus exact rows);
- any reuse of the path OR the branch by another Task.

There is no stale allowance, partial corpus, row-derived grant, or fallback to
the generic route.

## 11. Costs, locks and lifetimes

- Inside Immediate, per attempt: the precheck runs one count (at most 4097 index
  steps) and at most 4096 exact-row EXISTS queries. The unchanged writer repeats
  them once, so the cost is at most 2x. Exact-row equality compares encoded bytes.
  No decode, hash, encoding, filesystem, spawn or await is added under Store.
- Outside locks, per recapture: one coherent query-only read of the SAME ticket
  scope (existing bounds), plus at most 4096 rows, 16 MiB encoded in total and
  1 MiB per body. Headers are checked before any body is copied. At most 5
  recaptures per turn.
- Slot mutex: Arc clone/swap only. Custody mutex: `is_retained` reads only.
- Lifetimes: a superseded corpus is dropped when the writer's Arc clone ends.
  Transient peak: two corpora per retained plan. The 4096-slot registry still
  bounds the number of retained plans. Encoded budgets are not a numeric heap
  ceiling, and heap use per decoded corpus is unmeasured.
- Generation counter: u64 with a checked increment; overflow refuses.
- Fairness:
  - A stale attempt means at least one other committed Task-row write, or a
    membership change, happened in the same Project during that attempt's window.
    Every retry failure is therefore paired with another Task's progress. This is
    lock-free progress, not wait-free.
  - The Store and the admission are released during backoff and recapture.
  - Activation admission is a FIFO tokio mutex; the std Store mutex is not FIFO.
  - This is an opportunity bound, not a latency guarantee.

## 12. Security: attack surface, authority matrix, source-to-sink

Source-to-sink path. The corpus flows:

1. from `tasks` rows in the owner's selected state database;
2. into `managed_binding::snapshot` (read-only, query_only, with
   application/schema/instance/epoch checks);
3. through `validate_current_tx` of the SAME ticket;
4. into `NamespaceSnapshot::read`;
5. into `NamespaceSlot::replace`;
6. out through `input_namespace`;
7. into the W1/W2 precheck (Immediate);
8. into the writer's `validate_current` and `check_collision`;
9. and finally `write_snapshot`.

No other sink reads the slot.

| Principal / input | Can trigger recapture | Can supply the corpus | Can make a protected write succeed |
| --- | --- | --- | --- |
| Private owner: a live Driver worker with its own retained plan | Only for its own retained, unabandoned plan | No; the sole issuer reads the owner's database | Only with exactly current own authority and a fresh, complete, non-colliding corpus |
| Another Task (its own Driver or another legitimate writer) | Indirectly, by committing its own rows | No | No; its row joins our fresh corpus, and its path/branch refuses ours |
| Public Store API / generic None route | Indirectly | No | No new path; the existing route is unchanged |
| Public/native DTOs, Agent JSON, Unit/Session/Workflow DTOs, scalar IDs, discovery data | No | No | No; none of the types can be built from them |
| Nominal administrator/Human labels, CLI fields, environment, `--state` | No | No | No; labels are content and mint nothing |
| Same-UID raw SQLite writer (outside the application boundary) | Can make the corpus stale, flap it, or inject malformed or colliding rows | No | No colliding commit. It can force a typed refusal or hold (bounded DoS), as today |

Threats addressed:

- TOCTOU between capture and commit: closed by the in-transaction CAS.
- Unbounded retry or resource exhaustion: fixed attempts, sleeps and budgets.
- Confused deputy through a caller-supplied corpus: no public constructor.
- A reconciliation race retiring a live plan: the guard is disarmed only after a
  settled rollback, while the Store mutex is held.

Security references are supplementary and do not limit this analysis.

## 13. Impact analysis

| Changed item (pinned) | Consumers found | Impact type | Treatment |
| --- | --- | --- | --- |
| `NamespaceSnapshot::validate_current` namespace.rs:115-123 | Writer at Root1754 / CA1797; namespace.rs tests that assert `inventory changed` | Functional: none. Compatibility: none | Reimplement over `freshness` with identical messages |
| New `freshness`, `StaleKind`, `NamespaceFreshness` | Precheck only | Functional | New |
| `check_collision` namespace.rs:124-140 | Writer only | None | Unchanged |
| `NamespaceSnapshot::read` and ROWS/BYTES/BODY namespace.rs:5-7,21-23 | ticket.rs:63-67,98-123; recapture; tests | Performance: extra reads only after a stale result | Budgets unchanged |
| `DriverReadTicket.namespace` ticket.rs:18 | ticket.rs:63-67,98-123; CA preparation.rs:57,461-466,661-667; gates.rs:73; executor.rs:54 (Root preparation.rs:374-379,569-575) | Internal type change | The field is `pub(super)`, so only the driver module can read it. Implementation must grep `.namespace` in that module to confirm |
| `input_namespace` return type | CA state/mod.rs:1090; Root1083; W2 | Internal | Arc clone held for the transaction |
| `Applying` Root preparation.rs:312-317 / CA324-329 | apply_driver_preparation, begin_input, state/mod.rs:692 | Custody semantics | Add `known_uncommitted`; Drop semantics unchanged |
| W1 precheck CA898 / Root891 | Activation, gate, executor | Functional fix | Placed before the first write |
| W2 precheck CA657 / Root564 | First preparation | Functional fix | Placed before the first write |
| `put_task_tx_at_with_namespace` Root1685 / CA1728; None route Root1757-1768 / CA1800-1811 | All `put_task_tx` callers | None | Unchanged |
| Callers: workflow.rs:1145-1164; driven_initial.rs:110-112,161-163,281-284,380-382; attempts.rs:436-450 | Driver worker turns | Functional; at most 300 ms added latency | Retry loops |
| Activation admission installation.rs:148-199 | Activation | Admission is now taken per attempt | Existing checks rerun on each attempt |
| Service reconcile service.rs:20-50; WorkerLifetime Drop driver.rs:87-104; pending_preparations | Abandon/reconcile | Custody interaction | Code unchanged; verified by N12 and M3 |
| Constants NAMESPACE_ATTEMPTS / NAMESPACE_BACKOFF_MS | settle, tests | New thresholds | Tests must reference them by name, not as literals |
| Existing fixtures: namespace.rs unit tests; CAab0 activation tests (40 s wait_for); the CA3b distinct-Project variant | Test suites | Messages and timeouts unchanged | Keep them; add the SAME-Project control (§14). The distinct-Project variant earns no credit for this contract |
| clippy `too_many_arguments` (already allowed on activate_driven_workflow and put_workflow_transition_inner) | Lint | None | No new parameter on existing APIs; new functions take at most 4 inputs |
| Schema, guard layout, public protocol, capabilities, permissions, environment variables, paths | None | None | Not changed |

Unaffected, with reasons:

- Marker publication ticket CAS: it does not read the corpus.
- Physical helper namespace verification: it is a different check.
- Session binder and record-only semantics: no Task writer participates.
- Quota and capacity accounting: no counts change.
- Generic None route: it reads current rows under its own transaction.

## 14. Verification contract

Genuine producers are required for every control:

- an actual accepted Goal through accepted same-UID Runtime ingress;
- a real registered Git Project;
- the installed graph, Sources and allocation;
- configured account-free external CLI protocol peers (installation/tests.rs:1-65).
  These peers do not imply official-provider qualification.

Forbidden in every control: synthesized SQL positives, seeded Drivers or tickets,
fabricated proofs, and capability advertisement.

Waiting helpers observe the actual Driver exit and fail fast with its named
error. A timeout, compile failure or setup refusal is a SETUP failure, never a
mutant kill.

Each negative first reaches its genuine prerequisites: both Tasks at the S1 hook
and zero `workflow_native_contracts`. It then asserts the named class, and asserts
that complete images are unchanged for both Tasks across: `tasks`, `records`
(workflow and session), `context_versions`, `execution_units`, `task_execution`,
`task_drivers`, `workflow_native_contracts`, scoped audit/event counts, and quota
rows.

Positive controls:

- P1 `ca3c_same_project_simultaneous_s1_release_binds_both`. Restores the CA40c9
  CA3b shape: one Project, one accepted Goal, at least 2 dependency-free Tasks,
  the same executor and distinct reviewer rosters. Keeps the cross-pair refusals
  CA3bi-iii. Holds both Tasks at the S1 hook, then releases them together. For
  each Task, assert:
  - the SAME plan was published (existing observe seam);
  - a Bound execution reference and the exact contract body;
  - worktree paths and branches that differ between Tasks and are direct
    children of `worktree_root`;
  - distinct Driver, Unit and Session ids;
  - one gate receipt per completed initial phase.
  Also assert that exactly the later committer recorded at least one
  `NamespaceStale`, slot generation of at least 1, and
  `applied <= NAMESPACE_ATTEMPTS`.
- P2 unrelated foreign metadata progress: a third Task in a second accepted Goal
  of the same Project drives its own first preparation while A and B wait at S1.
  A and B still bind.
- P3 bounded contention: release together as many SAME-Project Tasks as the
  current effective Driver/Project capacity admits at once (at least 3, target 4),
  and report the admitted count. Every Task binds without exhaustion, and at least
  one sibling commit is observed during a backoff. This is not the
  four-Task/two-Project qualification.
- P4 lost notification: suppress the first committer's post-commit wake hint
  through an existing seam; the stale plan still converges within its turn. If no
  such seam exists, keep this as an honest SETUP prerequisite.
- P5 an inserted valid non-colliding row, or a deleted non-predecessor row: the
  plan retries, then binds.

Negative controls:

| ID | Injection (after prerequisites) | Required observation |
| --- | --- | --- |
| N1 | A separate SQLite writer sets a sibling's worktree to B's path, with a distinct branch | One recapture, then `task worktree/branch already owned`; no further retry; images unchanged |
| N2 | Same as N1, but the branch only | Same as N1 |
| N3 | An inserted valid Task row that uses B's branch | Inventory stale, recapture, then collision refusal |
| N4 | An inserted malformed row | Recapture refuses; plan abandoned; images unchanged |
| N5 | The Project grown beyond 4096 rows | Recapture refuses with `row budget`; images unchanged |
| N6 | During backoff (retry seam), change B's own Goal body or Driver row | Recapture refuses on own authority; slot generation unchanged; images unchanged |
| N7 | Drift of a hard predecessor's Task or Workflow | Prerequisite refusal, never retried. Stays a SETUP prerequisite until a genuine completed-predecessor producer exists; no seeded Workflow |
| N8 | The same Arc plan delivered again after success | Refusal; no second contract, audit or Driver advance |
| N9 | Precommit failure (CA4c seam) on the attempt after a recapture | Ordinary refusal; rollback proven; custody retired; no retry |
| N10 | Postcommit fault (CA4d seam) on the attempt after a recapture | Deferred; after the existing recovery, exactly one contract and one Driver advance |
| N11 | The retry seam changes a non-colliding foreign row before every attempt | Exactly NAMESPACE_ATTEMPTS applies, then `NamespaceContended`; the service retires the plan after proving rollback; images unchanged; bounded time |
| N12 | The retry seam runs the actual service reconcile pass after a stale outcome | The plan is still retained, then binds |
| N13 | Stop the Task Driver during backoff | Plan abandoned; rollback proven; no write |

Named compiled mutants. Each is committed and compiles cleanly, and must be
killed at the stated assertion. Restoring the exact source must pass.

| ID | Omission | Kill |
| --- | --- | --- |
| M1 | The precheck always returns Current | P1: the later committer exits with `original row changed` |
| M2 | Recapture does not replace the slot | P1: `NamespaceContended` |
| M3 | The stale branch leaves `Applying` armed | N12 retained assertion |
| M4 | Collision errors are classified as stale | N1 single-recapture and class assertion |
| M5 | Recapture skips the ticket's `validate_current_tx` | N6 generation assertion |
| M6 | The attempt bound is removed | N11 attempt-count assertion |
| M7 | The writer skips `check_collision` | N1 unchanged-image assertion |
| M8 | The branch disjunct of the collision check is dropped | N2 |
| M9 | The count check is removed from freshness/validate_current | N3 |
| M10 | Read skips malformed rows | N4 |
| M11 | Read truncates at 4096 | N5 |
| M12 | Disarm without a settled rollback (cfg(test) seam forces rollback failure) | Assertion that the plan stays armed and is reconciled |
| M13 | Backoff ignores cancellation | N13 |
| M14 | Exhaustion without abandonment | N11 retirement assertion |

The existing own-authority, prerequisite, Driver and activation suites rerun
unchanged; this design does not re-qualify their mutants. Also required: fmt,
Clippy with `-D warnings`, a build, the affected suites in both default parallel
mode and `--test-threads=1`, then the full workspace tests. Linux and macOS CI are
separate gates.

## 15. Open boundaries

This design does not change or decide any of the following: SourceStop/Cancel,
R2, G3, RN1, later/retry/Reviewer phases, artifact and full continuation,
recovery, and genuine initialization or record-only Session binder semantics.

Residual liveness limit: sustained contention beyond the per-turn bound produces
a typed, visible hold. Whether the Driver re-plans later belongs to the existing
Driver policy; this design does not introduce re-planning.

These remain separate mandatory open requirements:

- the four-Task/two-Project run;
- initial Bound and the whole Workflow;
- PR1-4;
- official providers;
- both operating systems;
- the full MVP;
- the three source reviews;
- the merge gates.

The exact observed row chronology remains unknown.

## 16. Master design

This author turn changes no master design, docs status, README, license,
production or test file. The implementation PR updates
`doc/design/master/multi-project-runtime.md` with the current fact: retained
Driver plans recapture only their nongrant Project collision corpus, under a
bounded retry, and the protected writer requires that corpus to be complete and
current at commit.
