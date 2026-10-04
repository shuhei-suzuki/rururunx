# Issue 23: Goal authority, DAG and completion

Status: proposed Design2. Requirements3 `1fc82b1` independently approved; design
review, source integration and acceptance remain pending. No implementation fact
is claimed. Current main4851fcd has schema3 and generic Goal snapshots; the native
branches and proposed #19 schema6/private input projection must be composed before
source acceptance. Allocate the final migration number in that combined revision.

## Authority and trust boundary

Goal definition lives in the existing Goal snapshot: objective/title, criterion
IDs/descriptions/evaluators, constraints, non-goals and fixed source references.
Do not create a second authoritative definition or DAG in another table. Extend
CompletionCriterion with a typed evaluator (RequiredTasksVerified or
Human { goal_pack_input: bool }),
legacy default Unverified, while its public satisfied/evidence fields remain
non-authoritative legacy claims. Accepted definition fields never change.

A private goal_authority row records exact Project/Goal, canonical definition
SHA256, creation origin and approved policy reference, with a scoped foreign key.
It attests acceptance, not alternate content. A mismatch fails managed operations.
Canonical encoding uses fixed field order, UTF8 byte-length framing, ordered
criterion/reference lists and explicit variant tags; no delimiter ambiguity or
unordered JSON map iteration. The creation port computes the digest itself.

The executable's Human ingress is a direct local rrx command handler, taking its
principal from the operating-system caller UID plus the invocation's own origin,
not a principal supplied in model JSON. The runtime controller is constructed by
local runtime composition under a durable, Human-approved policy identity/version.
The Human ingress approves the exact policy content digest; every content change
requires fresh approval. Repository/config/agent JSON cannot activate or replace
that policy, even if it repeats an approved name/version.
Non-serializable authority objects have private constructors in that ingress/
controller module. Native adapters, Workflow, proposal JSON, agent output and
ordinary Store APIs cannot obtain these objects or call an ingress via runtime
IPC. No agent tool endpoint implements Goal authority commands. CLI implementation
is #24; #23 provides and tests the boundary before that handler is exposed.

This is an application trust boundary: a native process with the same user's
machine privileges can invoke a local binary or edit its DB outside these APIs.
No OS sandbox or proof of biological Human identity is claimed. Direct Rust
library composition is trusted code; typed objects do not secure malicious code
linked into the runtime. Audit origin describes the actual ingress used.

Store generic put_goal rejects inserts and EVERY change to the Goal snapshot,
including DAG nodes/edges, followups, context_version, blockers, timestamps, state
and definition fields. Only byte-identical replay is a no-op, with no version or
audit increment. Every actual Goal-row mutation uses its typed authority port.
Legacy followups and blockers remain frozen unverified legacy data, ignored by
managed evaluation; new proposals/reasons never write those fields. Observation
writers use separate ports below. Managed creation
requires registered Project, expected Project version, immutable scoped definition
and a supported evaluator set; transactionally insert Goal Created, authority row
and goal.created audit. Created/Analyzing cannot dispatch. Unaccepted planning
output is an inert definition proposal, never an accepted Goal snapshot.

Legacy Goals lacking authority are definition-unverified, never managed ready or
complete. MVP does not ratify them or infer evaluator types from satisfied/evidence.
Authorized Cancel/Fail remains available for recovery; known terminal rows remain
terminal/unverified. Tests use the real trusted test ingress and explicit Running
transition, replacing old generic setup and mutation helpers. No production
cfg-test capability leaks into normal builds.

## Durable observations and currency

Add typed private tables for proposals and evidence, with independent checked
versions, exact scoped foreign keys and writer fencing. They contain no alternate
DAG, accepted objective or native permission. Goal.blockers is display-only legacy
text; new status reasons are derived from current rows. Generic RecordKind/audit
writes cannot populate these tables or reserved Goal authority events.

Evidence records identify criterion ID, exact target fingerprint, producer/origin,
result and bounded references; append-only after acceptance. RequiredTasksVerified
is evaluated from current Workflow facts rather than caller-provided evidence.
Human attestation uses only the trusted Human ingress; decision identity and
criterion ID are checked against the accepted definition. Attestation and evaluation
never update raw Goal/Task/Session/lock versions.

Proposals have two explicit kinds. Followup records purpose, scope, acceptance,
risk, dependencies and material flag, with Proposed/RequiringHuman/Rejected/
Accepted(TaskID). DefinitionChange records bounded requested changes but supports
only RequiringHuman or Rejected: it can never become Accepted or mutate definition.
Unresolved DefinitionChange blocks completion, as does any unresolved required
followup/material decision. Rejection requires the configured controller policy or
Human ingress; agents cannot dismiss their own required work as irrelevant.
Separate bounds prevent definition proposals consuming followup capacity.

Accept Followup atomically creates the same-scope Task and appends its DAG node/
validated dependencies, then records Accepted and one audit. Idempotent replay with
identical decision returns its original Task; conflicting repeated decisions reject.
Untrusted observations cannot smuggle permission into disposition. Automatic
acceptance requires controller policy proving within fixed constraints/non-goals;
material or unverifiable expansion requires Human ingress. #24 owns policy and
external Issue creation; #23 never creates a GitHub Issue as a persistence side effect.

Actual graph additions, lifecycle transitions and Goal pack-reference changes are
explicit authority writes and increment raw Goal.version under exact CAS. They
invalidate captured Goal input currency, including unrelated running siblings;
never refresh those sessions into new authority. #14/#24 must quiesce/reconcile or
schedule these updates before sibling admission, and surface resulting conflicts.
Observations, criteria results, status, Task completion and metrics do not bump
Goal.version. This preserves existing raw native version fences rather than
inventing a per-node exception to them.

A Goal Context Pack reference is a typed scoped artifact/version, independent of
Task contexts. Advancing it is controller authority and monotonic; unresolved
references grant no pack-use authority. #19/#24 resolve the actual producer and
source pins. Fixed Goal source references remain distinct from live Project rules;
source contents or Project rule/config changes invalidate evidence, not definition.

## Transactions, bounds and graph changes

Every managed mutation uses IMMEDIATE, verifies schema/writer epoch, exact Project/
Goal and expected authority/proposal/evidence versions, validates all new data,
checks finite limits, writes and appends scoped audit in one transaction. No external
Git/filesystem/native calls occur while holding SQLite or SharedStore locks. A
conflict/error rolls back all rows/audit/versions and never automatically refreshes
caller authority. Versions are checked positive integers within SQLite i64 range.

Initial caps: accepted definition encoding1MiB and whole Goal JSON4MiB;
4096 total scoped Tasks including unlisted;
16384 edges; 128 criteria; 128 fixed source references; 256 followups and 64
DefinitionChange proposals; 256 evidence records per criterion (32768 per Goal);
16KiB per individual text; 64KiB per proposal/evidence; decoded nesting depth32.
A coherent report scans at most4096 Workflow records,8192 Session records and8192
lock/recovery records, with32MiB aggregate decoded input per Goal. Evidence ledger
aggregate bytes are capped32MiB independently of record-count limits. Count and
SQL length projections enforce both before decoding; no truncated scan is success.
Caps are independent upper bounds, not a promise that all maxima fit simultaneously.
Routine reevaluation never appends evidence; identical attestation replay returns
its existing identity. Historical-record capacity is finite and explicit: session/
lock admission checks remaining Goal capacity before new reservations, and exhaustion
returns a bounded capacity error requiring controller/Human action, never silently
drops history, clears owned work or hot-retries. #14/#24 surface this condition.
Reject before constructing unbounded vectors. SQL count/limited projection reads
precede bounded body decoding; stored legacy oversize rows produce bounded error,
not a truncated ready/success result. Evaluation is iterative O(V+E) plus bounded
Workflow/Session/lock/evidence scans, never recursion or model calls.

Graph nodes are actual same-scope Tasks. Node creation and Task insertion share
one transaction; existing unlisted same-scope legacy Tasks are explicitly reported
and may be reconciled by controller adding that exact identity, after checks.
Reject missing/foreign/duplicate nodes, self edges and duplicate ordered
(prerequisite,dependent) pairs, regardless of hard/soft flag. No node removal,
edge removal or hard-to-soft weakening. Edge additions or soft-to-hard changes
require Created dependent with no Workflow history, Session launch or active
recovery/dispatch claim. This is checked from actual rows, not phase text.

Iterative Kahn ordering considers hard edges only, rejects cycles including old
paths, and breaks ties by Task UUID bytes. Soft edges are advisory; use a bounded
priority count and UUID tie, never require a soft prerequisite or drop a required
Task. Opposite hard/soft directions are permissible when the hard graph is acyclic.
All actual same-scope Tasks are required even if legacy DAG membership is missing.
A failed/cancelled Task stays required; only explicit owning-port recovery of that
same identity and fresh verified success can satisfy it. Replacement is additive,
never supersession. Otherwise an authorized Goal Fail/Cancel is required.

## Coherent readiness and verified success

A deferred read transaction, beginning before the first authority read, produces
one durable snapshot: Project/Goal/definition proof, all scoped Tasks, Workflow
records, typed Sessions and complete locks/recovery claims plus proposal/evidence
versions. Validate row/body scope agreement and bounds before returning results.
Return a report with its version frame and per-node reasons, never a dispatch
permit. Scheduler/Workflow admission revalidates current scope and existing native
input/lock authority at its own actual consumer.

One private Store-derived managed-Goal admission predicate requires Registered
Project, exact present authority row with matching accepted definition digest,
Running Goal and no whole-Goal hold. No caller boolean/token replaces it. Invoke
it before protected Task preparation/reservation/Start in Generic lifecycle
validation (adapter.rs1102), Claude ownership(f9b671f168), Codex ownership(a7a219b182),
Grok ownership169, Workflow step/retry active2424 and put_workflow_transition407,
and again in dispatch/ALLOW put_session_if_current733 under the transaction.
Workflow progression uses the predicate; conservative TerminalDecision/
TerminalRecovery and factual historical observation retain their separate closure
guards under inactive owners, never grant dispatch or fabricate process death.
Current code admits Created/Analyzing in Workflow and excludes only four Goal
states in native owners; all those consumers must migrate. A later hold increments
Goal.version and conflicts with captured scope. Preserve independent own-Session
DENY/terminal observations: the new predicate grants no reason to block a denial
with a parent fence. Goal/project-scoped Consultant planning may run outside
protected Task dispatch, with separately reviewed scope rules; it cannot mutate
accepted definition, mint a Task permit or bypass native owner checks.

Logical ready requires Registered Project, verified Running Goal, valid listed
Created/admissible Task, hard prerequisites with verified success, and no native/
Workflow/recovery ownership for that Task. Running/preparing classify running or
blocked with ownership reason; missing public ID/PID never proves idle. Lost,
unknown, active reservations and live typed Sessions remain owned. Review/Broker
pending facts classify waiting-human or blocked with their real cause. Failed/
cancelled prerequisites block descendants. Whole-Goal Paused/Blocked/WaitingHuman/
terminal states prevent readiness; computed node reasons do not persist those states.
Independent Tasks can remain ready while another is waiting or unresolved.

Define a private WorkflowSuccessProof extractor shared with Workflow validation,
using the actual durable Workflow record and Task. Require Task Completed,
finished=true, no active attempt/held/terminal-failure decision, current generation,
every configured mandatory gate Succeeded with its owned Passed observation and
exact scoped evidence, explicit MergeGate and Cleanup, and no unfinished effects.
Do not accept Task Merged/Completed alone, free strings or generic Record JSON.
Store put_record cannot create/replace authoritative Workflow; private transition
history/claim validation remains mandatory. Reject malformed/missing/conflicting
records and absent producer support instead of fabricating success.

Proof also checks complete Session/lock/recovery ownership from the coherent
snapshot: no Starting/live/Lost/native uncertainty or unreconciled active lock,
including Session-less dispatch reservations. #19 consumed phase owners and #9/#13
real review/merge/cleanup producer certificates must resolve when configured.
Actual #8 controlled Workflow completion is the initial non-Human evaluator
consumer; synthetic provider fixtures establish model wiring only. Final native/
review/Git acceptance requires the downstream real producer integrations, and
unsupported certificates keep completion unsatisfied. #41 terminal guards and
#43 private binding remain separate mandatory Workflow checks.

RequiredTasksVerified requires at least one required Task, complete DAG membership
and verified success for every actual scoped Task. Human criteria require current
attestation. All criteria results carry actionable unsatisfied/stale/unsupported
reasons. No count, public satisfied bit or LLM assertion is sufficient.

## Completion, lifecycle and recovery

Human criterion target fingerprint includes exact accepted definition digest,
Project repository/rule/config/source identity pins, canonical DAG digest, all
required Task identities and Task/Workflow versions, relevant Session/lock/recovery
versions. Exclude raw Goal.version and lifecycle-only hold/resume changes; they
still participate in dispatch and final publication CAS, not criterion semantics.
Include Goal Context Pack identity/version only when the accepted Human evaluator
declares goal_pack_input=true; unrelated pack refresh does not stale attestation.
Actual DAG/followup additions, source/config/rule or required-Task evidence changes
stale it. Attestation itself is excluded from that fingerprint, so attesting one
criterion does not stale another. Evidence/proposal ledger revisions are included
in the final commit compare frame. Source revalidation runs off-lock, then all DB
versions and exact observed source digest compare under IMMEDIATE; stale source
or second-writer changes produce conflict without a refreshed completion retry.
Physical source observations are not an atomic filesystem snapshot: pin immutable
Git/artifact identities and source digests, revalidate immediately before the DB
compare, and preserve runtime-owned review/source locks through publication. Same-
user external writes beyond runtime ownership remain an explicit trust limit;
#16 tests drift detection and never treats raw DB CAS as filesystem proof.

Only the private evaluator publishes Completed, from a verified nonterminal Running
Goal with all accepted criteria satisfied, all required Tasks settled successfully,
no unresolved proposals/decisions and no native/Workflow/review/approval/lock/
recovery ownership. Persist a bounded completion certificate containing prior
frame and results, then update Goal state/version and audit atomically. The new
terminal version does not retrospectively stale its own prior completion proof.
Human/controller cannot bypass this port by requesting Completed.

Controller transitions: Created→Analyzing/Running, Analyzing→Running, Running→
Paused/Blocked/WaitingHuman, explicit hold resolution→Running. Nonterminal states
may explicitly Fail/Cancel through controller policy or Human ingress with reason;
terminal states never reopen. Registered Project is required for progression;
conservative Fail/Cancel may reconcile Blocked Project without new dispatch.
Pause/cancel/failure do not release native resources or assert death. #14 owns
actual process/recovery reconciliation before claims are released.

Restart reloads proof/definition and recomputes every result. Cached reports are
not authority. Legacy terminal Goals retain unverified terminal classification;
unverified nonterminal Goals never become ready. Persisted whole-Goal holds need
their recorded authorized resolution; computed Lost/blocked reasons may disappear
when actual prerequisite recovery resolves, leaving the same Running Goal.

## Integration and verification

Coordinate migration with #19: writer-epoch fencing must cover every generic,
private Goal, proposal/evidence, Workflow, Session, lock, audit and context write,
including already-open old connections. Drain all old runtimes before upgrade;
refuse live/uncertain old ownership under #19 rules. Preserve old Goal bytes and
classify unverified rather than relabel origin/evaluator. New binaries reject
unsupported future formats; migration failure leaves original DB intact.

Inventory current main put_goal callers (45 including tests): native currency
controls edit objective/constraints today; replace those with authorized lifecycle
or additive graph writes so their actual stale-version consumer remains tested.
Workflow fixtures set Paused/Running/Cancelled through generic put_goal today;
migrate to real controller ports. Preserve overlapping two-Project scope tests and
all Claude/Codex/Grok raw P/G/T+full lock checks; no native currency exceptions.

Controls and compiled causal mutants cover hard cycles and soft cycles, ordered
edge duplication/scope, unlisted Task scans, atomic acceptance/fault rollback/
second-connection conflicts, definition edits/criterion replacement leading to
Task-only completion, forged Human/controller origin, unsupported/stale evidence,
actual Workflow success versus Merged/raw Completed, Lost/session-less claims,
explicit hold resolution versus display blockers, idempotent material followups,
generic Goal DAG removal/weakening/retroactive edge, context rewind/forgery and
followup/blocker edits (no version/audit changes), Created/Analyzing/Blocked/
WaitingHuman/legacy actual native launch refusal with Running positive controls,
Human attest-under-WaitingHuman then resume+complete, pack refresh with and without
declared pack input, and followup graph drift staling attestation,
terminal reopening and observation writes invalidating sibling native currency.
Include a generic DAG-write mutant making a formerly blocked dependent ready,
an omitted managed-Goal hold/proof predicate at actual native wire, and ignored
attestation DAG-drift mutant. Mutants must reach the intended consumer, compile
and fail an assertion; setup
refusal, timeout or unrelated safety guards provide no causal credit. Restore
source and passing controls after each isolated mutation.

Requirements passed independently; Design2 now needs delta review before source.
Source updates Goal master design/README to actual behavior, runs full appropriate
shared-state/native/Workflow regressions plus fmt/clippy/build and exact Linux/
macOS CI, independently reviews the immutable source/security scope, and composes
compatible #19/#43 migration and producers. No CLI/event loop #24, scheduler #14/
#27, native Goal #25 or final16 dogfood acceptance is implied by this model.
