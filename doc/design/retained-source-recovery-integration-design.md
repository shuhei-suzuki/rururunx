# Published committed-source recovery integration

Status: proposed STRICT implementation supplement; no source or recovery port is
implemented by this document. Source baseline
`a79a3051d41bc122e1e7e88cb7399c3cdfb83787`. This realizes
[Runtime design §5.1](runtime-scheduler-integration-design.md#51-existing-workflow-retained-frame-recovery)
and its approved requirements; §5.2 fresh bootstrap recovery stays explicitly
unsupported in this delivery. No old Unit, Session, native handle, completion or
finalization permission is reconstructed.

## 1. Exact first delivery and private source claim

`ManagedWorkflowSources::new` has an empty process-local map. Ordinary `prepare`
refuses an existing Workflow. `frame` needs a map entry before its retained branch.
The new private `recover_retained(task)` is the real reconstruction consumer. It
selects only the exact Published artifact identified by the sole current Workflow
source and immutable Context; a Ready artifact, latest artifact or terminal label
cannot select this route. Its slot contains `prepared: None`; first adoption stays
absent and existing fresh phase admission remains necessary.

The complete Runtime task-driver claim is not implemented. This delivery's private
`SourceRecovery` producer therefore claims only current-owner scoped source
reconstruction, in `Store::begin_retained_source_recovery`. It grants registered
retained reads and installation, never Task-driver or native dispatch permission.
The eventual Runtime driver must bind its genuine claim/version to this source
claim under one transaction before scheduling; approving this delivery does not
make Runtime restart scheduling available. No public JSON/caller capability or
persisted row can reconstruct the private claim.

The producer receives the genuine `RuntimeOwner` epoch and Task identity through
ManagedWorkflowSources. In one Immediate transaction it checks exact Scope,
Registered Project, Running Goal, nonterminal Task, sole Workflow, generation,
current consecutive Context pointer/body, full source hashes and the exact
Published artifact snapshot/dependencies. It validates the Context against the
Workflow with the existing validator and checks artifact scope/revision and
producing Executor identity. It does not clear an active/unknown Workflow phase,
classify an old effect as unconsumed, or publish accepted evidence. The later
Runtime driver must first use genuine retry/reconciliation where necessary.

## 2. Schema7 and durable pins

Native receipts own schema6; this supplement owns schema7, and ReviewRound uses8.
Implementation must first compose the actual schema6 source, then install ordered
6→7 and fresh7 in the same delivery. All mutable tables, including the new table,
receive connection-version7 guards; an already-open6 writer must refuse writes,
and new6 opens refuse7. Migration failure preserves prior bytes/version. No
optional table lookup or schema5 fallback permits a new recovery grant.

`source_recoveries` has one current row per Task, indexed exact Project/Goal/Task,
recovery UUID, epoch, checked version and state Preparing/Installed/Invalid. Its
bounded body binds canonical complete Project/Goal/Task/Workflow/Context hashes,
all actual versions and Workflow generation, exact artifact DTO/dependencies,
governing digest, frame digest and source map. Identity/body/index equality is
mandatory. New reconstruction replaces only an Installed/Invalid source cache
claim; a same-epoch Preparing claim is not stolen on elapsed time. Old identities
and transitions remain in bounded append-only audit. Epoch change invalidates
previous cache claims before admitting new ones, without changing retained work.

Complete body reads have explicit independent bounds: Project/Goal/Task 1MiB each,
Workflow and Context 8MiB each, artifact128KiB, recovery metadata128KiB. Check SQL
encoded length before decoding; source maps retain128 bounded entries. These are
separate body-processing costs, not a claim that a transaction processes at most
128KiB total. Hashes cover whole canonical bodies, not selected fields that could
hide an authority change. Full artifact read includes indexed dependency equality.

A private non-Clone claim and armed abandonment guard remain live through all
reads. Future Drop marks only its exact current Preparing claim Invalid and leaves
any retained helper's actual Unknown/cleanup bookkeeping intact. Failed/CAS-stale
reconstruction never installs a cache. No abandoned-row lookup creates a new live
capability. Restart can create a genuinely new source-only claim after epoch
fencing; it cannot resume old native input or recover lost answer content.

## 3. Reconstruction and actual helper fencing

Serialize the Task source slot throughout reconstruction, acceptance and cache
assignment. Verify the exact retained manifest and complete base/commit graphs via
actual `ResultStore::verify`, then read the exact tree/blob corpus through finite
registered `RetainedGit` actions. Build the existing Frame with committed rule and
configuration bytes; compare its entire dependency map/governing digest to the
claimed artifact and current approved instructions. Verify the retained graph
again after corpus reads. No live executor file or live rules fallback is used.

The current RetainedGit checks only epoch/artifact. Add a private optional source
read binding for this consumer, leaving ordinary historical inspection unchanged.
Before each source-bound helper spawn, its actual reserve-retained-inspection
Immediate transaction also checks current recovery identity/state and complete
Task/Workflow/Context/P/G/artifact pins. The periodic wait fence and the actual
finish-retained-inspection receipt transaction make the same check. A source
change during an await stops reconstruction; the owned child/HelperGuard preserves
existing best-effort cleanup and Unknown receipt behavior. This neither expands
ps observation nor turns reader custody into a native execution capability.

Only Frame's real complete corpus/config/rule producer constructs the private
reconstructed-frame proof. `accept_retained_source_recovery` rechecks the full
original snapshot, current epoch, exact recovery id/version/state and artifact in
one Immediate, then records Installed pins/frame digest and audit. No SharedStore
lock spans Git awaits. Keep the source slot locked through commit and assignment;
crash after commit but before assignment leaves an empty map and requires new
reconstruction, never a persisted-row-as-frame grant.

## 4. Current frame consumption and legitimate pin advancement

Every recovered `capture`, `committed_input` and actual phase/unit grant validates
the installed recovery id, current epoch, full current snapshot and exact retained
artifact/frame/source pins. Caller Project/Task DTOs must equal the current bodies.
The installed cache is not independent authority. `take_initial_executor` returns
None for the recovered route; it cannot fabricate PreparedExecutor provenance.

Existing legitimate transitions do change Task/Workflow/Context bookkeeping.
Advance pins only inside the original typed producer's transaction, using sealed
pre-write recovery validation and exact checked resulting bodies. Needed hooks:
`put_workflow_transition_inner` after Task+Workflow writes, `observe_workflow_gate`
after its exact observation write, and `reserve_execution_inner` after its scoped
Task projection plus Workflow Unit binding. Generic `put_task`/`put_record`, raw
SQL or a later fresh-row match never advance pins. Existing Workflow validators,
CAS, prerequisite and known-gate/Unit checks remain unchanged. Epoch and full
Project/Goal bodies must match; Task instruction digest, Workflow generation and
physical revision/artifact/source map must remain the installed ones.

A changed source/generation or terminal lifecycle invalidates the cache in that
same transaction instead of ratifying its old frame. Cancel/terminal bookkeeping
must still fence old permissions even when a cache is stale; that branch can only
invalidate, never advance/regrant. New artifact publication needs a new genuine
Published-frame reconstruction before another source-consuming phase. Native and
finalization admission (`checked_workflow_binding` / `validate_authority`) checks
current installed pins where this Task owns a recovery claim. These checks confer
no permission independently of existing admission and native input checks.

## 5. Actual controls and acceptance limits

Use isolated account-free Git fixtures and the real Published-artifact producer.
Reopen Runtime (new epoch), create a new Sources instance, reconstruct the exact
artifact after deleting/changing the old executor input, then use actual Sources
capture/committed_input. Assert committed rules/config/content, complete graph and
dependencies, Installed audit/row, and no PreparedExecutor/native/Session grant.
Exercise an actual subsequent typed Workflow transition and fresh phase reservation
so legitimate resulting pins advance while the original frame remains unchanged.

Negative controls cover Ready-only and foreign artifacts; missing graph/blob or
manifest; Task/WF/Context/P/G/source/generation/epoch drift before acceptance and
while a helper waits; competing claims; dropped Future; stale supplied DTO; generic
metadata drift followed by typed transition; cancellation and normal source change.
Assert no cache installation/effect/grant beyond the admitted bounded read, unchanged
retained work/artifacts, and no old capability. Refuse §5.2 pre-artifact recovery
before effects. Test actual migration6→7, pre-open6 writer and reopen refusal,
transaction rollback and current7 valid producers. Compile causal omissions of
selection, full pins, helper fence, final acceptance and generic-drift refusal;
setup errors do not count as assertion kills. Independent immutable source review
and appropriate current-head checks are required before qualifying this component.

Runtime driver/Goal scheduling, pre-artifact recovery, native receipt/schema6
qualification, full ReviewEngine, real accounts/hooks and final two-OS four-Task
acceptance remain separate. No completed preparation or source read means Agent
work Success or a Workflow gate Passed.
