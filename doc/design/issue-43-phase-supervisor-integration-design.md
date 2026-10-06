# Retained pending phase supervisor integration

This component implements the real pre-marker capacity and ownership retention
part of [Binding10](issue-43-managed-binding-design.md), §§2 and 5. Its fixed
source base is `c5c967bfd72a1caf19e36823d1c467df0edb7960` (actual Runtime9 plus
the effect-free selected Native allocation and refusal-only legacy consumers).
It creates no Native readiness, Driver claim, OriginalMarker or binding proof.
Binding10 installation and all genuine producer/consumer composition remain
separate gates. Runtime9 schema/producer review failures remain retained.

## 1. Actual ownership and private API

The same `Runtime` retains an `Arc<PhaseSupervisor>` beside its actual owner and
DriverRegistry. It does not open another RuntimeOwner or advance an epoch.
Runtime's actual control admission orders start, pending reservations and stop.
Only a started, non-stopping Runtime accepts a reservation. No public control
request, Session DTO, SQL row or configured provider name constructs a handle.

`reserve_pending(allocation: NativeAllocation, preparation: PreparationGuard)`
consumes the real selected-port allocation and armed preparation guard. It checks
the exact owner/state-root/instance/epoch, Unit/scope/generation and preparation
identity before queue insertion. Guard inspection is read-only and cannot disarm
it. A refusal returns both original objects for the caller's existing abandonment
path. It performs no Native helper, registration, input or public adapter start.

The Runtime-owned slot retains an immutable `Arc<NativeAllocation>` and the armed
guard, exact operation ID and a bounded current observation. The private
`PendingPhaseCapacity` is non-Clone/non-Deserialize and holds the exact slot and
supervisor identity. Borrowing its immutable allocation holds no queue mutex
across canonical snapshots, hashes, SQL, child wait or await. The Arc shares
read access to a genuine allocation; it does not manufacture a second allocation.
The Engine may drop its caller handle without dropping the Runtime-owned slot or
guard. Capacity remains charged until exact unmarked abandonment, genuine closed
operation handoff, or supervisor shutdown; dropping a handle is not closure.

The handle proves only the presence of that real reserved slot. It is initially
nongrant: the actual Driver pin/claim and Marker transaction must separately check
and consume it. Copying observation fields cannot authorize this consumption.
Root's marker planner borrows the original allocation. The eventual `PhaseLaunch`
must carry this same retained-capacity handle, rather than extract the allocation
and leave the Runtime without its slot. Marker commit/handoff failure must leave
the original slot retained and distinguish rollback from uncertain commit.

The final actor handoff must consume the same operation's original allocation
exactly once and retain its preparation guard until the actual Native start has
accepted ownership. It requires the genuine committed OriginalMarker, original
Driver claim, prepared pair and capacity handoff; there is no callback that
temporarily always accepts these missing proofs. This first component exposes no
marked/start/settled transition until those private types and consumers compose.

## 2. Capacity, fairness and level observations

Queue admission atomically checks configured global sessions and per-Project
tasks, additionally capped by this finite profile at 128 retained slots globally.
Each Project is independently charged; one full Project does not consume another
Project's available allowance. Existing actual Project/native/quota admission
checks remain separate and can impose stricter caps. Queue capacity does not
assert quota availability or permit a session to bypass those checks.

The retained current queue is authoritative for pending ownership; wakeups are
hints. A stable Project-rotating cursor emits at most64 distinct operations per
sweep, rotating within each Project as well. Removal/insertion cannot cause one
large Project to permanently hide another. At most128 slots and each allocation's
existing complete input bounds make storage finite; this is not a byte-budget
certificate for the future Workflow/ledger writer. No unbounded notification
history is retained.

The real Runtime service actor reconciles pending observations outside the queue
mutex, using the original allocation's readonly current-Unit checker. Applying a
result rechecks the exact retained slot identity. It records only PendingMarker,
HeldOwnerChanged or Stopping observations, never an owned Native success or a
NoCurrentDispatch proof. Unchanged Held observations cause no Store writes.
Finite fallback wake intervals are100ms to5s; no passive status call performs
bind, claim, start or reconciliation that mints authority. Later genuine start,
ACK, terminal and deferred-binding facts must come from their actual producers.

## 3. Abandonment and lifecycle

Explicit unmarked abandonment consumes the exact private caller handle and removes
only its own slot. The guard and allocation are dropped after releasing the queue
mutex, so existing nongrant preparation retirement cannot deadlock on SharedStore.
A bare operation ID cannot abandon a slot. No general ID-based removal API is
provided for future marked operations. The later marked lifecycle requires its
genuine typed closure; a missing terminal observation keeps ownership charged.

Runtime stop excludes further reservations at the same control admission boundary.
The service shutdown drains only this component's provably never-handed-off,
unmarked slots, outside the queue mutex. There are no Native children owned by
these slots in this stage. Runtime final Drop also revokes admission and releases
unmarked guards outside that mutex. Dropping an Engine future while the Runtime
remains alive has neither effect. Subsequent Native integration must replace this
unmarked-only shutdown with retained real actor cancellation/reconciliation and
must not apply it to marked or launched operations.

## 4. Qualification and missing genuine consumers

Controls must obtain allocations from actual installed Claude/Codex private ports,
prepare real Units/namespaces and create Goals through actual trusted Runtime9
ingress. No SQL-seeded owner, fabricated private constructor, public ID token or
Fake adapter stands in for a producer. Test caller Drop, global/per-Project
capacity refusal with returned objects, independent Project fairness, exact
abandonment, stale owner/preparation identity, shutdown races and guard release
outside queue locks. Fixed-clean source precedes formal tests and independent
review. Compiled mutations must reach actual assertions, not setup/compile failure.

Current `DriverRegistry` has only a readable `is_current` check and no actual
native registration/claim producer. Current Root planning/classification scaffold
does not mint OriginalMarker or PhaseLaunch. SourceRecovery7 original-pin
successor recognition, genuine native private start/binding ports and all protected
writers are not yet composed. These are explicit readiness gates; this real
retained queue alone does not make an Agent runnable or complete Phase2.


## 5. Concrete Runtime-owned Native invocation custody

The next private increment implements Binding10 §5.1 start-future ownership. A
separate `PhaseJobs` registry belongs to the SAME Runtime beside PhaseSupervisor;
it is not stored inside a pending slot or the supervisor. The actual Native
binding/launch parts retain that supervisor, so storing them in its own slot would
create a permanent strong-reference cycle. The job registry contains at most128
entries and reserves the SAME actual allocation before the marker transaction.
The existing actual pending slot remains the global/per-Project capacity owner.
No job reservation mints an OriginalMarker, Native owner or input admission.

Known marker commit/confirmation hands off once directly into a Runtime-owned
Tokio task. It borrows `selected_port()` from the SAME original allocation and
calls its concrete private `start_phase(launch)`; no alias lookup, copied port,
arbitrary callback or fabricated proof participates. Before spawning, the job
retains the SAME launch parts. The registry owns the actual JoinHandle and the
job retains the actual typed returned Native result or error. Engine callers get
only a nongrant level observation; dropping or cancelling that waiter neither
aborts the start future nor consumes its binding proof. Waiting/error/panic are
held observations, never NoCurrentDispatch, Task failure, retry or cleanup proof.

Commit uncertainty retains the original pre-transaction plan and reserved job.
Only confirmation of that SAME actual plan can perform the first handoff. A known
unpublished rollback may remove only its matching unstarted job reservation;
no launched or returned job has a generic ID-based removal path. Shutdown refuses
to report completion while jobs remain held. This increment does not provide
actual Native cancellation/terminal/binder handoff, job retirement, restart
reconstruction or process-reclamation guarantees. Those remain composition gates,
and the installation issuer continues to refuse Agent execution.

Qualification must distinguish compilation and nongrant regression controls from
actual producer-backed future-cancellation, error/wait/proof retention and panic
controls. Genuine positive qualification remains unavailable while the composed
Driver/Native producer is incomplete; no Fake or SQL-seeded authority substitutes
for it. This design is an implementation step of the existing approved custody
contract, not a relaxation of preflight or a declaration of Phase2 completion.
