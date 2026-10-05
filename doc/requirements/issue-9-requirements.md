# Issue 9 requirements: configurable independent review

Status: proposed requirements. Merged dependencies #2/#4/#8 supply state, adapters and
Workflow. Core production work has not started. Prepared-input integration depends
on reviewed #19, provider typed-input contracts and the record-only binding port #43.
Review Bundle/delta construction #20 depends on this engine. Issue #9 must prove
the typed consumer with actual Git-backed producer fixtures; the full #20 producer and native #16 integration remain explicit MVP gates, avoiding a dependency
cycle or an invented producer-readiness claim. Issue 9 owns and freezes the typed
consumer contract for bundle/core/specialization, revision and identity deltas,
expansion/coverage manifests and provenance/currency rejection. Its Git-backed
fixtures conform to that contract; #20 must produce that same contract, rather
than a differently shaped production artifact that fixture acceptance never tested.

## Purpose

Replace a hard-coded Triple Review with durable configurable independent Review
Sets. One, two and three-or-more reviewers must share equivalent factual evidence,
preserve individual findings and make acceptance depend on explicit policy plus
verified safety, instead of merely counting native transport completions.

## Scope

A Review Set contains immutable policy and reviewer roster, one active round and
append-only round history. Each round binds exact Project/Goal/Task, actual bound
worktree identity, committed target, captured sources, Task context/checkpoint
provenance and factual Review Bundle. Two reviewers are first-class. The
Claude/Codex/Grok triple is a preset; counts are never hard-coded to three.

The roster freezes each unique slot key, registered agent and requested model/effort.
Every new (round, slot) invocation allocates its own fresh independent native Session;
the native Session identity is never frozen across rounds. All slots consume the identical frozen Task context and locked revision
without incrementing Task.context_version or owner currency per reviewer. Temporal
concurrency is best effort within permits; queued/serialized members retain the
same input exclusions and residual native filesystem visibility caveat. A reviewed explicit ReviewSet delegation owns the entire roster;
existing single-actor context ownership cannot be bypassed by role or JSON claims.
Executor, Consultant and ApprovalReviewer cannot impersonate a formal slot.

Formal reviewers need contract-declared read-only operation with actual runtime
permission construction/denial verified; actual supported native profile conformance is required under I9-AC-21.f; final representative
Issue #16 enforcement/availability dogfood remains a separate mandatory MVP gate. Unsupported capabilities fail before model input.
Existing native permissions/auth/hooks/rules
remain authoritative. Reviewer or external mutation invalidates publication;
attribute it to a slot only when actual evidence establishes that attribution.
Formal gate review requires a clean committed target and an executor write lock
for the immutable target;
optional unlocked/advisory review is non-gating. Safe WHOLE-round closure (every
admitted member fully settled or safely cancelled and delegated phase closure
recorded) releases the round lock; uncertainty retains it. Source integrity
checks supplement that lock throughout. Dirty/advisory review cannot issue a gate certificate.

The common factual core has one exact digest across slots. Declared specialization
artifacts have separate immutable per-slot hashes and are structurally additive:
the byte-identical core remains mandatory and cannot be replaced or omitted.
Specialization identity/source is visible and attributable. Activation is focus-only:
explicit exclusion of mandatory rubric areas/rules or blocking severities refuses.
This is an activation-policy check, not a semantic non-contradiction theorem for
arbitrary natural-language text. rrx excludes raw executor conversation from ALL initial/member expansion inputs,
rules/source slices and manifests in EVERY round unconditionally. Same-round peer
findings/current-round output remain excluded until every roster slot settles. Runtime DB/sidecars, peer
outputs and raw executor transcripts are ineligible for RepositoryMap selection
and expansion, including explicit/symlink references. Mandatory references that
conflict with this exclusion hold before input rather than silently dropping rules. Peer
outputs/runtime state stay outside the reviewed source and bundle paths. Native
read-only permissions are retained: the runtime does not claim OS isolation from
same-user-readable runtime files. Certificates state this runtime-input independence
scope and residual filesystem visibility; native #16 acceptance checks the actual
input/storage paths and observed independence. After prior independent rounds finish, later rounds may include verified
defects and actual commit/delta/check artifacts as facts, plus attributed fix-
resolution claims and structured prior findings as attributed unverified
claims with original author slot, severity and exact text hash. Prior history uses
the bounded mandatory manifest and required unresolved-text delivery of I9-AC-14.e;
resolved/dismissed full texts may remain exact-referenced expandable history. Whole raw peer
outputs/transcripts are never injected. Dismissal adjudication history retains
actor identity. Dismissal claims/rationales remain labelled claims,
not verified facts; re-raising a dismissed finding remains possible.
Eligible opinions after this declared prior-round exposure may satisfy the
independent floor only with I9-AC-8.c actual profile eligibility: independence means current-round runtime-input independence,
not lifetime blindness to prior findings. This also applies to a slot’s fresh native
Session in a new full-roster retry round when its prior invocation had no structured
result; no within-round retry is permitted. Result/certificate
distinguish no prior-claim exposure from exposed re-review and never label an
exposed opinion blind. Before any later-round input, every earlier round's native
ownership must be safely settled; current-round peer findings remain excluded
until every roster slot settles.

## Verdicts, verification and completion

Slots return APPROVE, REQUEST_CHANGES or ESCALATE, plus typed findings and explicit
per-prior-finding dispositions assigned under I9-AC-9.h. Not every slot owes every prior
finding a disposition. Each disposition identifies immutable original finding
hash/round/slot, exact inspected current target locations and evidence references,
and one of confirmed-fixed, still-present, false-positive, severity-downgrade,
not-applicable, duplicate or superseded-by-target-change (I9-AC-9.a). It retains the
original severity/text/opinion. Unknown, missing or malformed required dispositions
hold; a general APPROVE is never an implicit confirmation. Only the eligible finder/
independent confirmer or trusted Human resolution rules below can make that evidence
an effective clearance. Target change or duplicated wording alone cannot clear it. Native
failure, timeout, malformed output, cancellation and Lost are separate outcomes.
Only completed, well-formed, exact-target APPROVE counts. Required review
instructions/skills must resolve and be deliverable to every designated provider
before any member input; missing, foreign, unresolvable or undeliverable mandatory
content rejects with Human attention. Record delivery in the bundle separately
from native-side activation observed or unverifiable; no invented activation.
The governing review rubric, mandatory rules and required skills resolve from the
activated Runtime/registered Project snapshot and its exact revision/digests,
outside reviewed Task-author authority. Task-delta edits to these resources are
reviewed source/proposals, never governing instructions for their author's review.
Native auto-discovery must preserve this declared governing authority without
bypassing user rules/settings; unsupported or unverifiable conflicting instruction
activation rejects the formal profile before member input. This does not claim
that today's Engine already reads rules from Task worktrees: its registered-primary
Project resolution is separate from this future ReviewSet authority contract.
REQUEST_CHANGES never
becomes APPROVE merely because its findings were dismissed. ESCALATE requires a
visible human-attention hold. An approval containing a potential blocking finding
cannot satisfy the round until that finding is resolved independently. Trusted Human
false-positive/downgrade/non-applicable/duplicate adjudication of an unchanged-target
finding may let THAT SAME safely settled round certify only when there is no ESCALATE
or disputed verification, all source/policy/coverage pins remain current and mode+
independent floor are already met. It changes no original verdict or author eligibility.
REQUEST_CHANGES never becomes APPROVE. ESCALATE/dispute requires a new full-roster
round after trusted resolution; a changed-target fix always requires a new round.
The stable I9-AC-2.a/I9-AC-9.c controls distinguish unchanged-target adjudication from a new round.

Severity is Critical/High/Medium/Low. The immutable policy's blocking set always
includes Critical/High/Medium; configuration may add Low but cannot exempt that
Core floor. Findings are proposals until
verification supplies actor identity (agent/Session or Human), exact target,
inspected/changed locations and actual evidence. No supervisor LLM invents facts.
An executor-only dismissal cannot erase a formal blocker: false-positive dismissal
requires explicit Human judgment OR later-round confirmation by the original
finder occupying the same frozen roster slot identity (registered agent ID plus
slot key) in a later round with a fresh Session, whose identity AND family are
neither recorded delta authors, or by at least
two policy-designated slots each excluded from all recorded authors.
Each confirmer must differ from every recorded delta author in registered agent
identity and native provider family, and a two-confirmer pair must also differ from
each other in both registered agent identity and native provider family; alias/allow-duplicate-agent permission cannot create dismissal independence.
Confirmer designation for inherited findings is frozen at Set/obligation-lineage
activation BEFORE ITS FIRST admitted round. Before a later round is not sufficient.
Any later policy/roster designation requires actual trusted Human-ingress
supersession and is post-observation; a newly designated agent/slot that already
expressed an opinion on that finding before designation cannot serve as its confirmer
pair. Inherited finding designation/history cannot reset through successor Set/alias.
No post-opinion Workflow selection can pick favourable confirmers. Any eligible
non-author still-present or equivalent non-clearing disposition contradicting a
proposed clearance for that same finding/current target is disputed verification:
it holds for trusted Human, including an original finder dissent against two diverse
confirmed-fixed opinions. Earlier unresolved contradictory evidence remains retained
and requires explicit adjudication, not erasure by majority. Trusted Human resolution
can adjudicate the conflict but never replace the independent certificate floor.
Inputs separate original finding and actual repository evidence from the labelled
executor dismissal claim, never presenting that claim as fact. Disputed evidence holds for Human.
A fix candidate records actual commit/delta/check artifacts and the author's
attributed resolution claim; it requires a new round. Clearing a potential or
verified blocker as fixed uses the SAME independent eligibility as dismissal or
downward out-of-blocking-set adjudication: explicit Human, original eligible
non-author finder in a later round, or two eligible pairwise agent/family-distinct
confirmers. General APPROVE without that explicit finding-resolution evidence is
insufficient. Finder failure/timeout, quorum/any tolerance or supersession cannot
erase the blocker or turn an executor fix claim into a fact.
Original outputs, verdicts and verification history are never rewritten.

Completion modes count well-formed exact-round APPROVE opinions: all (M), quorum
(N, 1 <= N <= M), and any (one). A separately mandatory independent-approval floor
also applies: at least one eligible non-author Session for QUICK/STANDARD and two
for STRICT. Explicit allow-self opinions may count toward all/quorum/any but never
toward that independent floor. A zero-eligible roster, or one unable to meet its
floor, rejects before input. STRICT author plus one eligible approval cannot pass.
Ordinary eligible independent approvers are distinct Sessions; mutual agent/provider
family diversity is required only by the frozen Project/Workflow policy, not
implied by the Core floor. Default no-duplicate-agent policy still applies unless
explicitly changed. Dismissal confirmer-pair diversity remains mandatory. Only quorum accepts a quorum parameter. Verified blockers veto
every mode, and unresolved potential blockers prevent a certificate. A
certificate is the immutable Review-Set-satisfied result binding exact Scope,
Workflow phase/generation/active claim and required review-instruction/skill
digest, round/target,
core/specialization hashes, policy/roster, per-member prior-claim exposure
(author slot/text hash), context provenance, individual settled
outcomes and verification/evidence references. The byte-identical core carries the
bounded prior-claim manifest/hashes/actors and mandatory status metadata. Required
unresolved claim texts are identical obligations for every slot, delivered inline
or as required expansion. Optional resolved/dismissed history delivery may differ;
per-member exposure records the actual text set delivered, never conceals exposure. Specialization is separately
declared/hashed and cannot remove this core. It is evidence, not merge authority.
Every mode-counted opinion obeys the same current-round exclusion, including an
explicit allow-self author, duplicate-agent and queued/serialized slot. The no-show
window lasts until every admitted roster slot has settled, not only floor-eligible
members. No current-round exposed opinion can count toward mode or independent floor.

Task Context Pack/checkpoint and condenser-authored narratives are attributed claims,
with source actor/content digest, unless a specific typed runtime-observed artifact
verifies the particular fact. Their immutable content/hash is a fact about the
captured bytes, not truth of statements such as tests passed or a blocker was false.
They cannot replace accepted constraints or independently clear a blocker.
Formal specialization acting as instructions uses the same activated governing
rubric provenance; Task-authored specialization is reviewed content/proposal only.

| Slot result | Counting and round consequence |
| --- | --- |
| APPROVE, no unresolved or verified blocker (including unverified Low only when outside the effective blocking set) | Counts once after safe settlement; Low remains recorded |
| APPROVE with potential blocker | Holds until independent resolution; verified blocker vetoes |
| REQUEST_CHANGES, all findings independently dismissed | Does not count; another round is needed for this slot to approve |
| ESCALATE or disputed verification | Visible Human hold; no certificate |
| Native failure with no unexamined structured result | Never counts; explicit quorum/any may tolerate it |
| Timeout or safely Cancelled with diagnostic-only progress | Never counts; explicit quorum/any may tolerate it, preserving diagnostics |
| Timeout or Cancelled with partial verdict/findings | Holds for independent inspection; safe cancellation alone cannot erase it |
| Malformed/overflowed/partial output | Holds for evidence inspection; never plain tolerated failure |
| Lost or uncertain native cleanup | Holds ownership and Human attention; no roster reduction |

Provider transport diagnostics are a distinct verified channel from structured
verdict/findings. Ambiguous assistant output is potential result content, not
classified as harmless progress. Required opinion approvals are M for all, N for quorum and 1 for any. In
addition, F eligible non-author Session approvals are required (F=1 for
QUICK/STANDARD, F=2 for STRICT), with no unresolved potential/verified blocker.
Thus any=quorum(1) and all=quorum(M); all tolerates no failed/non-approving slot,
and STRICT any still requires two eligible approvals.
Default completion waits for every slot to settle. An explicit early-stop policy
may cancel remaining work only after settled exact-round approvals meet BOTH the
mode count and independent floor, with no unresolved potential/verified blocker.
An author-only opinion can never trigger it. It retains all partial output and
applies the same blocker verification and safe cleanup conditions. Cancelled
members with partial/ambiguous structured content still hold until independently
resolved; safe process cleanup alone does not permit a certificate.
STRICT or a frozen security_review=true policy disables early stop. The security flag is
explicit for any class and is mandatory where Project/Workflow policy requires security review.
Other explicitly enabled early stop
never permits discarding emitted findings. Certificate/status records each
cancelled-before-output-by-early-stop slot explicitly: opt-in early stop trades
unemitted review coverage for latency, without claiming that slot inspected or
found no blocker. Within-round retry is disabled in Core: a retry needs a new immutable
round, includes prior failure/attempt history and consumes the round limit.
Each new round reruns the full frozen roster; approvals never carry across rounds
or changed targets. Same-target retry/dismissal rounds consume a typed identity
delta (baseline tree == new tree, with both exact HEADs retained) and prior-round
provenance; changed-target rounds consume
an actual revision delta. Rerun authorization compares repository tree content, not
HEAD alone: tree-identical commits are same-target for retry authority even while
their exact new HEAD is retained in provenance. Autonomous changed-target rerun
requires prior attributed verification under I9-AC-18.j and the bounded concern-linked
committed delta in I9-AC-18.a. The executor's
fix rationale is a labelled unverified claim, never proof of clearance. Empty,
unrelated or merely cosmetic deltas cannot authorize blind resampling. A new
full frozen roster independently assesses every retained claim under the unchanged
lineage limit. Deterministic verification may support it but is not required for
every semantic correction; no supervisor model attests relevance or resolution.
Certificates contain
outcomes from that exact round.

Lost/uncertain rounds surface a durable Human attention item with exact ownership
and blocker evidence. Trusted native recovery #14 must produce authoritative
terminal/cleanup evidence before safe release or superseding a round; Human
opinion, a dead PID or an edited terminal label cannot manufacture that evidence.
Until that port is integrated the round remains explicitly held. Automatic Lost
recovery and lossless hot upgrade are not claimed by this Core.

## Constraints and non-scope

Policy bounds: 1–32 slots, 1–32 configured local parallel launches, effective=min(configured,roster size,actual
Runtime/Project/provider permits); record requested/effective values rather than
rejecting policy parallelism above roster size. Reject 0/33; 64 cumulative rounds per immutable obligation-budget lineage,256
findings per result,8192 UTF-8 bytes per COMPLETE encoded finding entry (identity,
severity, original claim, locations, evidence references and framing INCLUDED),
not 8192 text bytes plus metadata; and 1 MiB per COMPLETE prepared member
frame (core+specialization+inline required claims+rules/wrapper) AND per result envelope.
The core alone is not the combined-frame ceiling; no additional specialization bypass. Limits are simultaneous ceilings: maximum count does not promise every
maximum-size text fits the envelope. No semantic truncation occurs. A 128 MiB
cumulative obligation-budget-lineage artifact quota bounds ACTUAL RETAINED rrx evidence,
not repeated delivery bytes. Charge owned copied blobs once by exact content/owner identity plus
EVERY distinct persisted frame/manifest/reference/row; a shared identical blob may
deduplicate, but two actual copies cannot be hidden by equal hashes. Git-object-backed
external immutable source references charge their retained manifests and any actual
rrx copy, with explicit external owner/availability/bounds. References never make
retained storage free or permit unbounded source delivery. Missing/unavailable exact
source evidence holds rather than reconstructing invented bytes.
Before any round input, reserve sufficient worst-case retained charge for every
roster slot: actual frozen core/frame bytes plus each bounded result, diagnostic,
the worst-case RETAINED expansion representation (copied bytes or exact immutable
reference/manifest with known owner bounds), retained native Session/private owner/pair/operation/receipt/
usage/audit row overhead from the actual reviewed producer contract, and shared
verification/journal allowances. Missing/noncomputable profile bounds refuse before
input. Stable source metadata can be charged by its known bytes; dynamic fields use
their frozen upper bound. Numeric bounds are
1 MiB result/acquired model-content prefix per slot, 64 KiB transport diagnostics
per slot, a FIXED full 8-MiB verification/check artifact allowance and FIXED full 4-MiB
control/adjudication/attention/manifest allowance reserved per round (not configurable
ceiling knobs), plus core and member-frame bytes and the existing
expansion ceilings. These are simultaneous bounds, not a promise every maximum
configuration fits 128 MiB. If remaining quota cannot cover the complete reservation,
reject before input. In-flight admitted results/prefixes/diagnostics always fit the
reservation; overflow records its bounded evidence and never counts as approval.
Actual retained bytes consume quota; unused reservation can be released only after
safe round closure, never by uncertain/Lost ownership. At a verification/journal
allowance ceiling, reject new additions before action, retain mandatory bounded
status/overflow provenance within a reserved 64 KiB control margin (part of that
4 MiB allowance), and hold rather than
lose existing evidence. External evidence stays exact-referenced under separately
declared owner bounds. Exhaustion retains byte/hash provenance and Human attention.
Transport output is
bounded at acquisition, retaining the observed bounded prefix, exact byte count/
retained-prefix digest and overflow reason; unseen suffix content is never treated as safe.
Malformed/partial output needs independent inspection or a fresh successful round
with explicit resolution; an unknown suffix cannot be dismissed as failure-only.
Native-admission startup timeout is 1–3600 seconds, default 60, beginning at
permit acquisition before spawn/handshake/auth/capability checks. The separate
review timeout is 1–3600 seconds after native admission, default 600. A separate
settlement deadline of 1–3600 seconds (default 30) starts at any cancellation,
startup/review expiry or terminal cleanup request. For normal completion it starts
at the earliest owned supervisor acceptance of a COMPLETE terminal review result,
owned native terminal event or explicit cleanup request; natural exit does not wait
for a close request. Streaming/partial opinion tokens are not a complete terminal
result. Store the monotonic start/cause once and never restart/extend it on subsequent
cleanup stages. All three deadlines are frozen in policy. A profile
requires reviewed normal post-result AND cancellation/startup-expiry/review-expiry/
early-stop forced-cleanup shapes and declared envelopes from its actual production caller contract: a reviewed upper bound of the
supported NORMAL AND cancellation/forced-termination cleanup paths through actual owned settlement, including required
native/tool/server shutdown stages and bounded dispatch/scheduling overhead. A p95,
typical latency or single observation is not that contract bound. The declaration
must be backed by actual producer code/profile and bounded shutdown/termination/
reap/owned-task settlement enforcement, with exact residual-uncertainty conditions;
an arbitrary timeout constant does not establish supported normal cleanup. Unknown
or escaped cohort ownership cannot be certified by selected-group death. Core
I9-AC-4/I9-AC-16.b require these ACTUAL supported #5/#6/#7 declarations before closure;
I9-AC-21.f public tracking alone cannot satisfy them. Until they exist the affected
preset/criterion stays open and refuses before effects, including Triple. Known unbounded user hooks, tool/MCP shutdown or uncontained cohorts make the profile
Unsupported BEFORE input. Their actual declared/detected configuration enters the
profile contract; auth/hooks/rules/defaults are preserved, not silently removed.
Unexpected late hooks, OS delay or newly escaped/uncertain ownership after dispatch
remains tracked Unknown/Lost for #14, never selected-group death or cleanup fiction.
Both envelopes are conditional on stated supported producer stages, not a
universal real-time OS bound. Profile freezes an explicit safety margin in integer
milliseconds: minimum/default 100ms, stronger finite margin required if its reviewed
producer contract says so. Envelope uses integer milliseconds CEILED from finer
units (never rounded down); normal is END-TO-END including its own escalation to
forced cleanup and cancellation during already-started normal cleanup. The single
settlement timer never restarts. A producer instead declaring a normal prefix must
check prefix+remaining forced envelope for every supported escalation/cancel point,
not just the maximum of two standalone durations. Check BOTH complete normal and
forced envelopes. Checked
max(normal_ms,forced_ms)+required_margin_ms<=deadline_ms accepts; insufficient/
unknown margin refuses;
100ms is a proposed configured margin, never physical jitter/death proof. Actual Issue 16
measures hook/tool-stage refusal, jitter-at-margin and residual normal/escaped holds;
no percentile or arbitrary timeout substitutes for actual native producer conformance.
The configured settlement deadline must cover that envelope plus required margin; if unavailable or too short, the preset refuses
before member input. With default 30s/100ms margin, known 29.9 s accepts;
29.901/29.999/30s or unknown refuses. Default 30 is provisional, not
an asserted native guarantee.
A supported explicit non-default deadline remains within 1–3600. OS/uncertain cases
still yield the declared held outcome; a contract bound is not a universal deadline
guarantee or real-model timing proof.
Terminal cleanup includes normal completion after a valid APPROVE: slow cleanup
can still cause an absorbing Lost hold and prevent that otherwise successful round.
Missing authoritative terminal/cleanup evidence at settlement expiry becomes a
durable uncertain/Lost Human hold, retaining permits/locks/owned supervisor
bookkeeping. The owned operation remains supervised; expiry never drops its
native-operation ownership or invents death. Once persisted as uncertain/Lost, this universal absorbing
hold does not release permits/locks on a late observation even from the same original
supervisor. Late terminal/cleanup observations are retained as attributed evidence only. Trusted
#14 recovery must explicitly consume that evidence before ownership release; this is a
deliberate conservative availability limitation until that port is integrated.
Broken-supervisor/restart evidence likewise cannot release the hold. Timely safe settlement
remains a distinct
failure/cancellation outcome; startup expiry is a distinct failure only after authoritative
safe cleanup, otherwise an uncertain/Lost hold. It never counts as approval. Resource-queue wait is a separate visible state with 1–3600-second
attention threshold, default 600; crossing it emits durable attention but the slot
STAYS queued under ordinary non-Lost-blocked contention and automatically admits
when actual permits return, after fresh
source/claim/round/lock/currency checks. It neither requires Human rescheduling nor
pretends a model timed out. An explicit policy/trusted Human termination still applies;I9-AC-14.h separately defines
frozen optional queue expiry and exact round-lock timing.
Existing Runtime,
Project and agent resource limits can lower local parallelism, never be bypassed.

Default formal policy forbids self-review slots and duplicate registered agent IDs.
Duplicate identity means the same activated registered agent ID, not merely the
same native provider family. Distinct production aliases of one family are separate
IDs only when registered by activated Runtime/Project policy and explicitly named
in the frozen formal roster. They need no duplicate-ID flag; their activated alias
permission and shared family remain visible in the result/certificate. Repeating
one ID requires explicit allow-duplicate-agent. Author-family exclusion and optional
provider-diversity policy apply independently to both cases.
A self-review slot is the current Task executor agent OR any registered identity
or native family in the cumulative delta-author set (including document drafting
and former executors). It requires explicit allow-self before input, including a
different alias of an author's family; with that permission its opinion never
satisfies the independent floor or any blocker-resolution confirmation.
Explicit policy may allow duplicate-agent fresh independent Sessions;
that permission and provider/model diversity are visible in the result. A single
Session cannot occupy two slots. The shipped `triple-adversarial` preset has the Claude/Codex/Grok roster and all-of-three policy,
without an implicit self-review exception. With an MVP executor in that roster,
default selection rejects before input. The separately shipped `triple-adversarial-author-visible` variant explicitly activates
allow-self independent Sessions before any opinion; actual I9-AC-8.c/I9-AC-18.g conformance makes
that variant usable, not the default adversarial variant; the author slot is never eligible
as a blocker-resolution confirmer. Acceptance exercises both default rejection
and actual registered production-adapter explicit-policy consumer execution with a declared synthetic native transport peer, rather than calling the unresolved
default a working configuration. Formal policies and the triple preset default require explicit
resolved model/
effort configuration before input; unresolved None rejects. An explicit advisory
policy can inherit native defaults but cannot silently replace formal settings.
Record requested, native-reported and unavailable values separately; optional
require-verified-effective policy rejects unavailable native measurements.
Report actual provider model/effort when exposed, otherwise an explicit reason. A known mismatch to requested Some fails the slot;
Advisory requested None may bind the native default without an invented
measurement; formal policy does not silently inherit it.

Non-scope: Review Bundle/delta construction #20, Approval Broker #10, general trusted
recovery #14, global scheduler/recovery #14/#27, complete CLI/TUI #15 and actual native
dogfood #16. This engine integrates their typed evidence/ownership contracts rather
than fabricating them. Reviewer scheduling obeys available resource authority.
Native two/triple dogfood is mandatory for the MVP Goal in #16; Issue #9 core can prove
orchestration without claiming that actual native runtime acceptance is complete.

## Persistence and acceptance evidence

Reopening preserves immutable roster/policy/target, independent outcomes/findings,
verification history, exact input digests, nullable measurements and uncertain
ownership. Competing coordinators cannot duplicate a slot or publish stale rounds.
State/slot/audit updates are atomically visible. Format upgrade rejects incompatible
fresh or already-open old writers without changing evidence; generic history or
caller JSON cannot create a slot, certificate or native admission.
Issue 9 owns ReviewSet-specific private authority constructors and the trusted
library ingress/controller composition, tested at that boundary without a CLI.
It reuses the approved Issue 23 application trust rule, not an absent implementation
or a dependency on merging Issue 23. A future CLI/controller caller must compose
this actual port; native/Workflow/Broker runtime channels cannot expose it.
Only actual trusted Human ingress may mint Human-action authority; automated controller
authority is separate and limited by I9-AC-9.i. This application composition records authority,
with actual principal/origin and exact policy/target/evidence identity. Native,
Workflow, agent output/proposal and Approval Broker runtime JSON/API/IPC channels
cannot label themselves Human or obtain that authority. Broker decisions never
substitute for Human adjudication/activation. This covers blocker dispositions,
post-opinion relaxation, supersession/decomposition, authorship/unknown-delta
disposition and approval of NEW governing-policy/rubric content digests.
Instantiating an already exactly Human-approved digest is separately frozen-policy
controller work under I9-AC-9.i, not authority to approve new content. Direct same-UID machine/CLI/DB action
outside these runtime APIs is an explicit trust limit, consistent with Issue 23.
This includes a native shell directly invoking the same-UID local rrx binary: CLI
UID/origin records application provenance, not evidence that a biological Human
initiated it. No ancestry/environment authentication or OS containment is claimed.
Such direct machine action is outside the runtime API guarantee; supervised runtime
JSON/API/IPC calls still cannot acquire or relabel Human authority. Certificates and
status disclose this limit and record the actual accepted UID/ingress origin;
this is not biological identity proof or an OS sandbox. Trusted Rust composition
is not authenticated against malicious linked code. No extra authentication
infrastructure is implied or claimed here. The principal/origin follows the SAME
approved23 derivation contract; composing code may not supply an arbitrary principal
or relabel library-composition as CLI/UID ingress. Preserve actual invocation origin
and test that native/API/IPC and library-origin relabeling cannot mint CLI authority.
This reuses the contract/type semantics, not a mandatory23 implementation merge.

Budget lineage is a stable immutable (Project, original Task-origin, review phase)
identity, shared across that obligation's successors/decomposed descendants. Distinct
Requirements/Design/Implementation/Security phases have distinct origin-phase budgets;
new current Task/Set IDs never reset an inherited root. Multiple applicable inherited
roots are jointly charged/checked; dropping one cannot restore quota/author eligibility.
At most one nonterminal Review Set exists per CURRENT Task/Workflow review phase,
not one per shared budget root. Concurrent descendant Sets are allowed when physically
safe and actual permits permit, with ATOMIC shared 64-round/128-MiB retained reservation/
4-GiB delivery counters. Concurrent admissions exceeding remaining shared capacity
admit at most one; rollback changes no root and every held reservation stays charged.
The stable I9-AC-14.g controls cover shared-root descendant admission and atomic overflow.

At most one nonterminal Review Set owns a current Task/Workflow review phase. A new
Set cannot approval-shop by resetting blockers, holds, history, round count or
artifact quota. Explicit trusted Human-ingress supersession records authority/
reason and
inherits unresolved potential/verified blockers, adjudication and cumulative
limits across roster, policy and generation changes. Target change never erases
unresolved findings; verification must explicitly resolve applicability or fixes.
Lineage exhaustion can terminate without certificate or hold for explicit Task
terminal scope reorganization; decomposition cannot resume an exhausted obligation lineage or restore certificate capacity. It cannot reset into a new passing Set. Decomposition requires an explicit
Human actor/reason, never autonomous Goal follow-up creation. Child Tasks inherit unresolved
potential/verified blocker vetoes and original evidence, cumulative author attribution from the
parent frozen base, all remaining budgets and the same parent obligation-lineage identity.
Counters/quota are shared across those descendants, not restarted per child Task. The inherited
findings require the same independent resolution before any child certificate. Decomposition never
certifies the old phase; Goal history alone is insufficient enforcement.
Unreleased Lost/uncertain native members in any Set of the obligation lineage fence
all new Sets, rounds and descendant formal review admission, even after authorized
Set termination or Human decomposition. Descendants inherit the hold and uncertain
target-mutation taint as well as blocker/authorship/budget obligations. Held shares
and worktree locks continue to count; an independently allocated worktree does not
erase the lineage hold. Trusted Issue 14 recovery is the only releasable exit.
Obligation lineage is the runtime-owned stable identity allocated for the initial
Project/Task/Workflow-phase obligation, retained by successor Sets, generations and
actual decomposition descendants. A new Task cannot choose a base behind which
applicable held/exhausted obligations disappear. Formal admission retains exact
registered-base ancestry or the recorded legitimate base-sync operation. Ancestry
identifies retained contributions; it does not alone prove every later obligation
is affected by every reachable uncertified contribution.

Until actual #12 applicability is available, unknown out-of-band recorded contributions
must conservatively accumulate applicable/unknown author exclusions/budgets/vetoes even for an apparently
unrelated new obligation. This can make STRICT or ALL classes zero-eligible after
several native families contributed; it is an explicit PRE12 AVAILABILITY LIMIT,
not proof that upstream bytes were authored by the new Task. I9-AC-10.a still records true
byte/upstream attribution separately. No bytes-only intersection or trusted opinion
may waive semantic applicability.21.h/public Issue #9 and #16 track author-set growth from
out-of-band merges, inherited-budget refusals and resulting zero-eligible frequency/
Task-time. Proven actual 12 disjoint evidence is the scoped future exit; until then
refuse/inherit, never silently claim availability or wash the old uncertified history.

Issue 9 owns the typed applicability consumer/checker and trusted verification boundary.
Issue #12 owns the reviewed impact/applicability producer using #18/#20 exact source/
artifact inputs. This ownership and conformance obligation must be publicly tracked
under I9-AC-21.h before #9 closes, without a 9→12 merge cycle. Until the actual reviewed producer/
conformance exists, production applicability is UNKNOWN and inherits or refuses;
9 Git-backed controlled consumer fixtures do not certify fresh-budget/disjoint
production admission or close its MVP availability gate. A lexical RepoMap, filename
intersection, model/Human claim or fixture boolean is not semantic applicability proof.

Applicability is explicitly evidence-bound. A fresh unrelated obligation requires
trusted recorded impact/applicability evidence covering its changed AND relied-on
source/dependencies/context and the retained contribution/finding scopes. Filename
or target-diff intersection alone, relabeling/rebasing, or an executor/Human JSON
assertion cannot establish disjointness. The evidence identifies exact snapshots,
inspected relationships/locations, retained finding identities, observed checks and
scope conclusion through the trusted verification contract; unknown or incomplete
applicability conservatively inherits the affected obligations or refuses admission.
Reworking, superseding, decomposing or covering an affected prior obligation inherits
its exact unresolved vetoes, original evidence, cumulative authors and shared exhausted/
remaining budget. A proven disjoint new obligation may allocate a fresh lineage,
budget and authorship set; the old uncertified history stays explicitly unaccepted
and cannot acquire a certificate or disappear. No certificate-floor override exists.

Actual Lost/uncertain native/resource holds retain their full reviewed effect scope
until trusted 14 recovery. Root/common-Git overlap can still fence the whole Project
or physically overlapping Projects regardless of disjoint review obligations;
new labels, merge, Human opinion or separate worktrees never release those holds.
Lineage states are uncertified-open, certified-pending-merge, certified-and-merged,
held, exhausted and terminated-without-certificate. A certified target handed to
the registered base by an actual scoped merge artifact proving that same reviewed/
accepted target contribution is certified-and-merged and becomes legitimate upstream
attribution for a new unrelated Task under I9-AC-10.a without inheriting its review budget/
author exclusions. Its original locks/operations still retain their actual scope.
The accepted certificate and actual merge artifact bind exact reviewed target/
contribution; generic Merged/Completed labels, names, claimed merge or Git reachability
cannot create that handoff or wash held/exhausted history. Out-of-band merges retain
explicit uncertified provenance and applicable obligations, assessed under the same
evidence rule. Termination alone never certifies prior contribution. This scoped
policy scopes exhaustion to actual affected obligations: exhaustion is
not a permanent veto on proven unrelated obligations. #16 measures applicability
holds, disjoint progress and actual resource-scope blocking separately (I9-AC-11.d).
Untracked semantic copying/re-authoring from main is outside this ancestry/reuse
identity guarantee; ordinary provenance/unknown-authorship rules still apply and
no exhaustive semantic copy detection is claimed.

Hold exits are explicit: persistent nonblocking REQUEST_CHANGES or too few approvals
requires an authorized Human/Workflow choice of a new full-roster round under its existing retry authority, explicit
Human-only post-observation policy relaxation above the frozen mandatory floor, or termination without a
certificate. Automated Workflow may not relax a frozen policy in response to opinions. A
deterministic conditional policy branch frozen before first admission is not post-observation
relaxation; its condition/authority/outcome are retained and it cannot weaken mandatory floors
or clear blockers. Same-target Workflow reruns after a nonapproval require a recorded new repository/verification fact; without it, only explicit authorized Human adjudication can
request another round. Autonomous same-target causes are explicit: (a) one
confirmation round per (Task/phase obligation lineage, target tree) after nonapproval or a
potential-blocker hold (including APPROVE with a potential blocker), for a newly recorded
verification inspection
with exact locations, target and check/observed-source digest not used for that
finding in any prior round. The inspection must address a retained finding or explicit concern
of a nonapproving or potential-blocker-bearing slot; an unrelated inspection cannot authorize resampling that slot. Even
inspections of different findings cannot authorize a second autonomous confirmation round for
the same lineage/tree. Further same-tree confirmation needs explicit Human adjudication; an
executor rationale alone is insufficient and remains
a labelled claim; (b) a safely settled native failure/timeout/cancellation with only
verified diagnostic progress, only when every other settled slot APPROVEs and no
slot has a potential/verified blocker or nonapproval. Mixed transient failure and
REQUEST_CHANGES/ESCALATE cannot resample that dissent: cause(a) concern-linked
inspection or explicit Human adjudication is required. At most two total cause(b) OR cause(c)
full-roster retry rounds per lineage
and at most one per (lineage, exact target tree, cause identity). Cause(b) identity
is provider-family/error-kind; cause(c) identity is exact frozen qualified schedule,
actual resource scope and schedule/queue/capacity failure kind.
The same cause on a changed target can consume the remaining lineage-wide allowance;
changing tree never resets the separate two-retry ceiling or 64-round quota. They consume the same 64-round
and artifact limits. A repeated proof/cause, exhausted retry allowance, unresolved
partial result or Lost requires Human/recovery rather than another automatic sample.
Cause(c) is an explicitly frozen finite controller-contention retry: a qualified
schedule/queue expiry or lost-free capacity change safely closes a non-certifying
round, all actually started owners are genuinely fully settled, and every received
opinion APPROVEs with no potential/verified blocker, nonapproval, ambiguous suffix
or unexamined result. A never-started cancelled slot is not a native failure or
APPROVE. A genuine recorded changed scheduling/capacity fact not previously used
for the same cause is required before the full-roster retry. It SHARES the two-
retry ceiling with(b), charges the same64-round/artifact budgets, and cannot reset
on a changed tree. Retained Lost/unknown blocking-capacity cause never qualifies(c);
unchanged blocked scheduling parks. Dissent or partial/unknown outputs require the
existing inspection/Human path, not this scheduling exemption. Test healthy opinion-
free schedule closure→genuine fair-capacity change→bounded full roster, third retry
refusal and foreignLost no retry/budget burn. All previous exposures/findings remain
retained and the new roster still needs genuine current native channel qualification.
Repeated blind same-target sampling is not authorized by the lineage ceiling.
A relaxed policy starts a new round or explicitly superseding Set,
retaining prior opinions, obligations and lineage limits; it never manufactures an
APPROVE. ESCALATE/dispute requires authorized Human adjudication,
then a new full-roster round or termination without certificate; malformed/partial
output needs independently recorded inspection/resolution and new round; unknown
suffix requires new successful review rather than pretending it was examined;
quota/round exhaustion requires termination without certificate or explicit terminal scope reorganization with inherited obligation lineage; Lost remains held until trusted native #14
cleanup. Generic late terminal/cleanup claims cannot substitute for trusted native recovery
identity and exact persisted Lost owner/attempt authority. That would launder uncertain ownership into capacity release or fresh dispatch.
No such reviewed atomic recovery handoff exists here, so factual late evidence is retained but
cannot authorize release. None of these
exits counts the prior failed/non-approving slot as approval.

Rule, policy, context/Workflow generation or required instruction change during a
round invalidates certificate publication. Findings may carry a typed upward risk
signal to Workflow; no automatic downgrade. An upward signal immediately holds certification;
already admitted peers settle normally as retained evidence, with no new input under obsolete
currency. After settlement the signal stays a PROPOSAL until exact non-author verification,
trusted Human acceptance or deterministic Workflow policy evidence verifies its cause.
Only then applies monotonic escalation and the next full-roster round meets that floor.
Trusted Human may reject an unverified signal with reason/evidence; this rejects a
proposal, never downgrades an applied class or disables a governing minimum. An
allow-self author's signal cannot certify itself. The stable I9-AC-12.i controls cover immediate hold, verification and accepted class changes. If the frozen roster cannot meet it, hold for actual
trusted Human roster selection or terminate without certificate. Post-opinion
roster change/supersession requires that authority; automated Workflow cannot swap
out a dissenting slot. A frozen pre-admission conditional branch may strengthen
policy without changing the roster, retaining exact trigger/activation provenance.
Risk-escalation hold exits only through that settled escalation/new
round or termination without certificate, preserving native holds.

Reviewer-assigned severity is immutable in the original finding. Any adjudication
that lowers a finding out of the effective blocking set uses the same independent
confirmation/Human rule as false-positive dismissal; executor-only downgrade
cannot remove the veto. Upward risk/severity can conservatively hold immediately.
Certificates preserve original and adjudicated severity plus actor/evidence.

Every policy meets the effective Project/Workflow/class minimum, including required
approval count, roster size/provider diversity, blocking severities, read-only/
clean/lock requirements, early-stop prohibition and verified-configuration policy.
Supersession cannot fall below that minimum or reset the lineage's required floor.
Human relaxation above the floor is explicit with authority/reason in the result;
mandatory rules are never disabled. Self-review and dismissal exclusion includes
all recorded agents/native provider families that modified the cumulative Task
delta since the frozen Task base revision, including fixes by former executors,
not only the current Task.executor. Authorship is established by runtime-owned Session/dispatch
evidence binding the registered native identity/family to observed before/after tree deltas, or
explicit attributable Human records. Git author/committer strings, trailers and executor claims
are claims, never attribution authority. Delta content not covered by that evidence is unknown;
overlapping uncertain ownership cannot silently be assigned to a convenient identity. Unknown
authorship is surfaced and requires
explicit disposition. Policy may ONLY refuse or conservatively exclude every
candidate native family; it cannot relabel unknown as Human/unattributed to restore
eligibility. Specific actor/Human attribution requires trusted ingress with evidence. Actual typed
deterministic runtime-effect attribution is separately covered by I9-AC-12.k; a generic hook
name or successful command cannot silently make changes non-native-authored.
The stable I9-AC-12.d controls reject nonconservative unknown-delta dispositions and native self-confirmation.

Individual actual native/resource permits release at each member's own full authoritative
settlement PLUS exact member closure under I9-AC-18.h; a result/terminal event alone cannot
release a retained server or unknown cohort. Round lock/evidence/budget remain independent.
After ALL current-round owners actually settle and the exact delegated phase closure
is recorded, release that round's lock/member delegation/resources. A nonterminal Set
then retains only immutable findings/obligations/history/shared budgets; it has no
worktree-write or Task-context ownership during executor remediation. An unsettled/Lost
round never takes this path. The real Workflow remediation consumer, not generic Set
JSON, returns control to the scoped Executor for fix+commit+milestone context publication.
Next full-round admission acquires ONE roster-wide delegation bound to the then-current
Workflow-published ContextVersion/target/source, with no Set/member-originated version
bump. Stale/advanced context refuses. Acceptance I9-AC-20.d: test two rounds across actual fix/commit showing
only Workflow milestone context versions, no settled-Set executor blockage and no
permember/current-round head rewrite. The compiled #19/#43/#9 integration must implement
this handoff; a proposed delegation cannot count as existing ownership release.

A Reviewer may request bounded broader context without mutating Task.context_version
or the shared factual core. Record exact requested/returned source hashes, slot
attribution, budget outcome and additive artifact provenance. Peers remain unchanged;
material needed by all slots is offered in a new full-roster round. Expansion can
never silently inject peer output, alter mandatory facts or weaken live source checks.

Human resolution of ESCALATE is typed adjudication history with actor/reason/evidence
included in a later full-roster round; it can resolve the escalated decision without
being counted as a model APPROVE. Repeated ESCALATE remains visible and may terminate
without certificate. Authorized Set termination on Lost produces no certificate and
releases no native ownership/lock; the Task remains WAITING_HUMAN until trusted
restart/native recovery #14 proves cleanup. Its attention reason is recovery-pending/
nonactionable, distinct from a Human decision with an available trusted ingress.
The status does not imply an existing UI action can release it. GitHub irreversible external outcome
reconciliation (#13) is distinct. These future ports are not fabricated by this engine.

APPROVE with only nonblocking Low findings may count even before verification;
those findings remain recorded and can be raised to a blocker on actual evidence.
REQUEST_CHANGES with
empty or nonblocking findings never counts. This preserves the slot's opinion
while quorum/any tolerate explicit nonapproval; mode never removes blocker checks.
Independent partial/malformed inspection uses the same non-author confirmer or
Human eligibility as dismissal. Mandatory core/specialization overflow rejects
before model input with actual size/hash/NeedsContext evidence and Human attention;
no semantic truncation or success through a byte cap is permitted.

Policy minimums are explicit resolved inputs, frozen before round admission. Core
baseline requires one eligible non-author independent Session approval for
QUICK/STANDARD and two for STRICT, Critical/High/
Medium blocking, clean/read-only/lock safeguards and explicit reviewer config.
Runtime/Project configuration may strengthen this floor; implicit missing policy
or unsupported configuration rejects. The preset can raise count/diversity but
cannot weaken class minimums or reset lineage requirements.

All Lost/uncertain member reservations continue to consume their actual provider,
Project and runtime resource shares and remain visible in status. Unrelated Tasks
can use remaining permits; exhaustion is reported as held capacity, never silently
ignored or bypassed. No operator release exists before trusted native #14 recovery.
This engine cannot promise unaffected throughput after every global permit is held.

Source/dirty/lock/rule/context/generation drift invalidates certification and keeps
any unsafe native ownership held. After safe settlement, authorized Human or
Workflow records inspected locations/cause and a verified committed target plus
source/lock/policy evidence before a new full-roster round; no automatic reset or
discard of reviewer edits is authorized. Otherwise terminate without certificate.
Resource-queue attention continues automatically on actual available permits with
fresh target/claim/round currency, or explicit policy/trusted termination; it does not reuse expired authority
or count the waiting slot. NeedsContext can exit by an explicit bounded expansion/
new factual bundle, or termination. Every exit retains the lineage's obligations,
round/byte counters and original outcomes.
Mandatory core contains exact scope/target/base/delta identities and source hashes,
accepted constraints, governing rubric, checkpoint/context provenance, labelled
prior obligations/claims and an exact coverage manifest. Complete large diff/source
bytes may be mandatory expandable references rather than duplicated in that core.
Required references bind exact content/ranges and remain obligations, not optional
omissions. Each slot must receive its required review coverage through the reviewed
bounded provider expansion port before its opinion can certify; delivery provenance
is not proof of semantic reading or correctness. Missing/unsupported delivery holds.
Under default 1 MiB per-slot expansion limits, required expanded target/claim bytes
are certifiable only near that limit; larger coverage needs an explicitly activated
non-default budget (up to the fixed maxima), and measurement records that budget.
Each reference/artifact obeys 1 MiB and cumulative frozen budgets; exact ranges can
progressively disclose a larger diff without semantic truncation. A >1 MiB target
diff is certificate-capable when mandatory core and complete required delivery fit
all frozen ceilings. Core itself above 1 MiB or required coverage beyond budgets is
an explicit availability limit: NeedsContext/new permitted policy round or termination,
never a certificate with omitted material. #16 measures this frequency. No generic
protected live free-text input is an expansion port; actual slot/provider authority
and typed source-delivery integration remain prerequisites.

## Consolidated acceptance

Numbered I9-AC-1–21, including their mapped case lists and all normative acceptance
subcriteria with explicit stable IDs below, are the complete closing set EXCEPT
explicitly POST-closure MVP criterion21.n. Each pre-closure ID requires linked actual evidence; no unnumbered acceptance paragraph adds a hidden
closing obligation. Definitions above specify the tested behavior. Items 1–8 map to the public
Issue #9 checkboxes; items 9–21 add integrity/availability and their explicit cases. Every criterion
needs actual consumer evidence; proposed tests and independent development reviews
are not runtime proof.

1. First-class two-reviewer rosters run through real Store/Workflow
   delegation, using one fixed Task context, exact locked revision and
   independent member ownership. Reject 0/33 slots, invalid parallelism and duplicate
   Session ownership. M=2/parallelism8 accepts with effective at most2; lower actual
   resource authority queues remaining slots without policy rewrite.
   The separate M=1 Scope closing coverage is I9-AC-1.c; one eligible member within M=2
   does not prove the one-reviewer roster positive.
2. Two-of-three quorum counts exact-round APPROVE only; reject N=0/N>M and quorum
   parameters on all/any. All, quorumM and any/quorum1 have equivalent boundary
   outcomes and retain every nonapproval/blocker.
3. All-of-two requires both approvals. Exercise empty/nonblocking REQUEST_CHANGES,
   persistent nonapproval and audited Human relaxation/new-round or termination
   without fabricated approval.
4. Shipped all-of-three `triple-adversarial-author-visible` resolves actual registered production Claude (#5),
   Codex (#6) and Grok (#7) read-only declarations and supported explicit model/effort configurations.
   With each MVP executor-in-roster, prove default self-review rejection and explicit
   independent-Session permission success. An author opinion never satisfies the
   independent floor; reject a pure-author formal roster and STRICT with only one
   eligible independent approval. The two non-author Triple Sessions can meet the
   STRICT floor only with actual I9-AC-8.c current-roster profile conformance while all three opinions must APPROVE under all mode. Unsupported/unregistered configurations
   reject before any member input. Fixture-only registrations do not close this.
5. Failure/timeout/safe cancellation never counts. Separate diagnostics from partial/
   malformed/overflow verdict/findings; ambiguous/unknown suffix holds for independent
   inspection/new successful round. Lost never implies terminal cleanup or release.
6. Reopen and competing coordinators preserve every member opinion/finding, original
   severity, actual input/configuration measurements, target and verifier history.
   They cannot duplicate admission, replace a member or reset evidence.
7. Formal clean/locked/read-only checks exclude Executor dispatch and detect Reviewer
   or external dirty/source/lock drift. Restoration/new-round evidence and held
   ownership survive invalidation; advisory review cannot issue a gate certificate.
8. Prove identical mandatory core hashes, additive specialization/expansion, no same-round peer
   output/raw executor transcript in initial/expanded runtime inputs, rules/source
   slices or manifests. Runtime DB/sidecars and output artifacts remain ineligible,
   including explicit/symlink references. Prior-round structured claims are labelled
   separately from verified facts; whole raw peer outputs stay excluded.
   Changed and same-target rounds consume actual Git-backed typed revision/identity
   deltas with prior unresolved facts and fix/check provenance; stale/foreign/relabelled
   bundles reject. Production #20 producer and representative native #16 independence/
   result/efficiency dogfood remain separate mandatory MVP gates.
9. APPROVE with a potential blocker holds; verified blockers veto every completion
   mode. Preserve unverified nonblocking Low and immutable original severity.
   Executor-only dismissal/downgrade and two duplicate-agent/provider-family confirmers
   reject. Independent eligible confirmation or explicit Human judgment records
   evidence; fix candidates/commit/checks require a new full-roster round and explicit
   eligible resolution before a blocker is cleared.
10. Same-target retry and changed-target remediation rerun the full frozen roster,
    retain prior outcomes and never carry approvals. Within-round retry is absent.
    Verify/fix/commit/test/re-review and repeated Human-resolved ESCALATE preserve
    adjudication as labelled history, not model APPROVE.
11. Supersession retains unresolved obligations and lineage floors across executor,
    roster, policy and generation changes. Cumulative 64-round/128-MiB limits never
    reset: every actually admitted round consumes one count at the first member's actual
    admission, including later held/superseded rounds. A fully queued frozen proposal
    has not admitted a round or spent autonomous retry/confirmation allowances;I9-AC-14.h
    defines stale/expired proposal history and retained byte charges.
    After 63 admitted rounds, admission 64 is accepted within all other limits;
    admission 65 rejects. Scope decomposition
    preserves the same obligation-lineage vetoes, author attribution and budgets in descendants; Human authorization is required and no child certifies the prior phase.
12. Class/Project policy floors reject missing/weaker policies (including blocking
    {Critical,High} without Medium), early-stop under STRICT/security and absent
    required configuration. Pairwise independence/delta-author exclusion and unknown
    authorship decisions are visible; allow-duplicate-agent permission does not authorize
    dismissal by an author.
13. Certificate binds exact phase/generation/claim, target/source/core/member hashes,
    mandatory instruction/skill/context/checkpoint provenance and settled evidence.
    Source/rules/policy/risk/context drift and cross-phase replay cannot pass.
    Certificate binds native_execution and read-only/source/method enforcement basis.
    The actual review-gate consumer rejects synthetic or otherwise unready basis as
    production gating evidence; fixture certificates remain explicitly nongating.
    Test refusal through the actual downstream certificate consumer, with independently
reachable basis and readiness predicates as I9-AC-21.i specifies. Certificate is
    not merge permission;43 binding remains record-only, never a production-gate override.
    The actual downstream consumer independently reports typed scope/phase/generation/
    source-binding assessment EVEN when final gate permission refuses nongating basis.
    Fitting nongating artifact reports binding-valid+permission-refused; drift/replay
    reports its EXACT binding refusal+permission-refused. Mutation evidence must change
    that actual binding assessment, never merely assert the always-refused permission.
    This assessment cannot mint a certificate or bypass21.i/21.j readiness.
14. Slots and local parallelism each reject 0/33 and accept 1/32 within policy;
    exercise 256 findings,8192-byte text,1-MiB envelopes, mandatory overflow,
    quota/round exhaustion, startup/review timeouts 1–3600 and queue-attention exits retain evidence
    and fail visibly without semantic truncation or hidden resource bypass.
15. Lost retains actual resource shares and native locks even after Set termination
    without certificate. Remaining permits still serve unrelated Projects/Tasks;
    exhausted capacity is explicitly reported. Reviewer-mutation, drift, NeedsContext
    and queue holds require their recorded authorized exits. No fictitious cleanup.
16. Actual reviewed #19/#43 shared typed-input/member delegation and native #5/#6/#7 caller
    contracts compose with the real Workflow consumer. Incompatible old writers,
    generic history/JSON authority fabrication and stale concurrent updates reject
    without evidence loss. Meaningful boundary mutants, independent requirements/
    design/source gates and exact-head Linux/macOS CI complete core verification.
    Real two/triple model results are separately required by #16; actual declarations/
    supported preset consumer proof are required here.

**12.a — Family authority.** A native provider family is the trusted registered adapter's declared native
implementation family (for example claude, codex or grok), not an agent alias or
caller-supplied output. Undeclared/unknown family cannot establish independence.
Eligibility excludes every recorded native delta-author identity AND family;
unknown authorship needs explicit recorded disposition before formal input.
Human-authored changes are separately attributed, not invented native families.

**16.a — Scoped measurements.** Each member's queue/startup/review/settlement timings and usage carry exact
Project/Goal/Task, phase, Set/round/slot, Session and agent attribution. Provider
token/cache/cost measurements remain nullable with reasons, separate from runtime
byte/token estimates; source bytes and reviewer amplification are attributable.
Acceptance I9-AC-16 includes this fixture-level attribution, without inventing native
measurements or replacing the #16 efficiency comparison.

**21.a — Public inherited obligations.** Inherited MVP obligations are linked explicitly: [#20 acceptance](https://github.com/shuhei-suzuki/rururunx/issues/20)
requires production deterministic bundle/delta/expansion integration and the same
mechanism for two/Triple; [#16 acceptance](https://github.com/shuhei-suzuki/rururunx/issues/16)
requires actual native two/Triple results, isolation and efficiency comparison.
[#14 recovery](https://github.com/shuhei-suzuki/rururunx/issues/14) must consume
ReviewSet Lost/uncertain member holds and reconcile exact native ownership, locks
and resource permits with trusted evidence. Issue #9 never releases them by opinion
or reset. Core closure reports these still-open MVP obligations explicitly.


**12.b — Cumulative author eligibility.** Authorship is cumulative from the lineage's frozen Task base revision to the
reviewed target. Later identity/short revision deltas never erase earlier native
contributors or former executor families. Independent approval counts eligible
Sessions, not distinct families: two separately owned Sessions from the remaining
non-author family can meet STRICT only with actual I9-AC-8.c registered-profile conformance for that exact
roster/schedule/source scope AND the frozen roster/duplicate-agent/diversity policy. A fixed Triple roster with Claude and Codex as recorded
authors has only one eligible slot and rejects before input, naming excluded
families and the unmet floor; neither supersession nor relaxation can lower it.
An explicitly allocated eligible custom roster can satisfy it, without treating
an author alias as independent. Test both outcomes.

**16.b — Production adapter contract consumer.** Items 1–4 require actual runtime consumer execution through registered production
Claude/Codex/Grok capability resolution and provider adapter contracts. Deterministic
stand-ins may replace only the native process/transport peer. Conformance evidence must identify
the exact reviewed #5/#6/#7 adapter contract fixture source commit/path/digest and the replayed
or derived frame/error/exit shapes: startup and capability negotiation, requested/effective
configuration, input, permission denial, auth/stderr-only failures, partial/overflow output,
terminal response, normal post-result exit/cleanup ordering and its reviewed
observed/declared latency envelope, and stalled cancellation. Conformance records
configured settlement deadline and refuses unsupported/too-short configuration. Unsupported shapes reject rather than idealizing
successful startup/read-only/settlement; the production registry, scoped Store,
private input ports and Workflow consumer remain real. Every such evidence record
states native_execution=synthetic, the stand-in identity and contract-fixture source digest.
Native read-only enforcement and real transport/model acceptance remain unverified by these
peers and are mandatory #16 evidence; production runtime permission construction/denial can be
tested here. Declaration-only or
fixture-only adapter success cannot close these items. This proves deterministic
runtime plumbing, not real native-model acceptance reserved for #16.

**14.a — Expansion bounds.** Expansion has frozen per-(round, slot) request limit 1–64 (default 8) and cumulative byte
budget 1–8 MiB (default 1 MiB), with at most 1 MiB per returned artifact and 32 MiB
aggregate expansion bytes per round. Every actual retained expansion blob/manifest/reference counts
against the 128-MiB RETENTION quota by I9-AC-14.c. Every delivery, including repeated identical
content, separately consumes frozen per-slot/round actual-byte budgets and telemetry. Exhaustion holds NeedsContext with recorded
count/bytes; no semantic truncation or context-pointer update hides it. Independent
expansion input retains the same guarded core/target/provenance exclusions.

**8.a — Native visibility.** Where a provider exposes read/tool traces, observed access to current-round peer
output or runtime-private DB/artifact paths is an independence violation holding
certification; record exact scoped trace evidence. Missing native read visibility
is explicitly unverifiable, never a proof that no read occurred. Real isolation
confirmation remains #16. Parallel remediation in another worktree is not run by
this engine: external fixing follows product-requirements §24's separate-worktree rule and
requires safe round settlement plus a verified committed target before new review.

17. Additional mandatory cases mapped to I9-AC-1–4 and 8–16:

- **17.a** Two-reviewer matrix: QUICK/STANDARD non-author pair, explicit allow-self author
  plus one non-author, and STRICT non-author pair succeed when their mode settles;
  STRICT author plus one eligible slot rejects before input. Ordinary same-family
  eligible Sessions remain allowed under the frozen policy. Multi-family authorship
  rejects an insufficient Triple but permits an eligible custom STRICT roster.
- **17.b** Early-stop cannot fire on author-only approval, below-floor counts or potential
  blockers. It preserves diagnostics/partial output and holds on partial findings
  until independent resolution. STRICT/security rejects enabling it.
- **17.c** New concern-linked inspected verification authorizes only one autonomous confirmation round per lineage/tree. Sequential fresh inspections of different Low findings after nonblocking dissent reject a second automatic same-tree round; unrelated inspections and repeated rationale without new evidence reject. Two safely settled transient retry rounds consume
  lineage budgets; repeated cause on the same tree/exhaustion/partial/Lost do not
  automatically retry. Test the same cause on a changed tree accepts only with the
  remaining lineage-wide allowance, and a third transient retry requires Human.
  I9-AC-21/#16 reports both repeated-same-tree and lineage-wide-ceiling interruptions.
  Mixed diagnostic-only transient failure plus nonblocking REQUEST_CHANGES rejects
  cause(b) autonomous retry; all-other-APPROVE diagnostic-only failure may permit it.
- **17.d** Missing/unresolvable/undeliverable required review/security instructions or skills
  reject before any member input. Delivery and native activation measurements
  remain distinct. A policy adding Low treats an unverified Low as a potential
  blocker rather than using the table's nonblocking-Low exception.
- **17.e** Every per-member entry within the certificate lists prior structured claims seen, author/text hashes,
  trace-observed independence violations or native visibility unavailable. Test
  expansion request/byte/aggregate and lineage-quota boundaries with unchanged
  core/ContextVersion and retained evidence. The process stand-in flag accompanies
  actual registered-adapter two/quorum/Triple consumer fixtures.


**12.c — Draft author attribution.** Cumulative authorship conservatively includes Task document-drafting families as
well as implementation/fix families, including paths outside a phase's narrowly
reviewed slice. It does not silently narrow author exclusions by artifact. A
Claude document drafter plus Codex implementation author therefore leaves only
one eligible fixed Triple slot for STRICT; reject with named exclusions or use an
explicit eligible custom roster. Human authorship remains separately attributed.

18. Additional mandatory cases mapped to I9-AC-1, 4, 10, 14–16:

- **18.a** Tree-identical commits remain same-target. Autonomous changed-target
  admission needs an actual nonempty committed source/artifact delta linked to an
  immutable retained concern-root from an actual prior nonapproval finding/slot OR
  an unresolved potential/verified finding in the effective blocking set, including
  a blocker carried by APPROVE,
  preceded by I9-AC-18.j exact-target recorded verification, with exact before/after trees, affected hunk hashes/locations and a truthful,
  explicitly unverified executor fix claim explaining that relationship. Pure
  relocation, rename-only, unrelated edits or cosmetic bytes without a specific
  formatting concern refuse. The typed checker checks this factual linkage and
  retained provenance; it does not certify a semantic fix. Actual applicable
  dependency/impact claims still need genuine #12 evidence; unknown impact cannot
  certify a reduced delta slice and therefore uses full required coverage.
  The full frozen roster reruns with all unresolved original claims, and ONLY its
  current independent dispositions/approvals can clear them or meet the floor.
  One changed candidate cannot borrow approval from prior rounds. Every candidate
  charges the shared64-round ceiling and remaining retry/artifact allowances;
  repeated identical trees follow I9-AC-17.c, and supersession/decomposition/new
  concern IDs never reset ancestry or exhausted budgets. There is no additional
  one-candidate-per-concern Human gate: bounded verify→fix→commit→re-review remains
  autonomous, without self-clearance, shared-budget reset or borrowing prior approval.
  This is not a categorical no-resampling guarantee: the bounded nonblocking-dissent
  rerun residual in18.i remains explicitly accepted. Tests include
  a genuine concern-linked multi-step fix with two changed commits/new full rosters,
  APPROVE+Medium→verify→fix→commit→full-roster rerun,
  labelled-claim-not-clearance, retained veto, cosmetic/unrelated/empty refusal,
  same-tree rerun refusal and ancestor-budget reset refusal. This positive closes
  through real fixed nongating conformance delegation as I9-AC-21.j specifies;
  it does not require pretending an absent #12/#60 semantic proof producer exists.
- **18.b** Freeze and test startup/review/settlement deadline minima/maxima. After a stalled
  cancellation expires, status exposes exact uncertain member ownership and
  durable Human attention while the owned operation is still supervised and all
  permits/locks remain held. Later original-supervisor cleanup is retained as factual evidence but neither grants an approval nor releases the absorbing hold before trusted #14 recovery. Repeat with restart/broken supervisor continuity; ownership remains held in both cases.
- **18.c** Custom two-reviewer and four-plus-reviewer rosters deliver distinct explicit
  model/effort per slot. Known effective mismatch fails only that slot without
  counting approval; require-verified-effective rejects unavailable measurements
  before inference (safe startup may occur to resolve them). Formal None rejects;
  advisory None records native defaults/unavailable facts separately. Result and
  certificate retain requested, reported and unavailable fields distinctly.
- **18.d** Parallelism below roster size queues slots while preserving identical core
  hashes, Task context and runtime-input exclusions. Document-only plus code author
  families yield the stated named STRICT Triple rejection or custom-roster result.


19. Blocker fix-resolution remains independent: in 2-of-3, an executor fix claim
    plus check artifacts, finder diagnostic-only timeout and two general APPROVEs
    holds without certificate. Explicit eligible independent fix confirmation plus
    approvals can pass. Preserve original finding, fix claim and confirmation
    separately in the input/certificate; author or same-family alias cannot clear it.
    Under any/all/supersession no unresolved prior blocker disappears.
20. Default QUICK Triple with a document-drafting family and a different code-author
    family rejects the roster before input, naming excluded self slots; explicit allow-self can collect their
    opinions, while only eligible non-author approvals meet the floor. A distinct
    agent alias of an author family follows the same rule. STRICT with one eligible
    slot rejects; a permitted custom non-author roster remains usable. After every
    1/2/4+ member round, including queued slots, Task.context_version and native owner
    currency have not changed per member; all share one context/core, and an attempted
    per-member context clone/bump rejects.
21. Before closing this Issue, closure evidence must quote/link the public acceptance
    entries (Issue bodies or immutable commit-pinned requirements permalinks) that own the specific inherited
    obligations: #14 exact ReviewSet Lost/uncertain member ownership/locks/permits;
    #20 production deterministic two/Triple bundle/delta/expansion; #16 real native
    two/Triple isolation/results/efficiency plus timeout/retry/partial-output and settlement-expiry Lost hold
    frequencies plus restart/broken-supervisor, native rubric conflicts and queue attention,
    retained capacity/Task-time and Human-interruption impact. I9-AC-21 is the canonical handoff: EVERY subcriterion attributing a measurement to
    #16 must be quoted by stable ID from public #16 acceptance before closure,
    This includes deferred evidence as well as measurements for ALL IDs referring
    to named inherited owners: I9-AC-16.b/I9-AC-16.c native read-only enforcement on real transports,
    I9-AC-8.a trace-observed isolation and I9-AC-16.a efficiency attribution. #14 must also
    explicitly consume I9-AC-11.b lineage/descendant holds and target-mutation taint,
    not just individual member locks. #20 must quote I9-AC-8.g consumer-contract
    conformance, including stale/foreign/relabelled refusal.
    Including I9-AC-8.c (native auto-injection versus passive visibility), I9-AC-8.d (blind
    versus prior-claim-exposed rounds), I9-AC-14.d (large required coverage/over-cap core,
    including accumulated claims), I9-AC-20.a/I9-AC-20.b (two-author STRICT and zero-eligible
    roster availability), I9-AC-21.b (partial-output early-stop/Human attention), and
    I9-AC-21.c/I9-AC-21.d (normal/cancel/timeout settlement holds, capacity and Task-time).
    It additionally requires I9-AC-8.h claim exposure/verdict-input exclusion, I9-AC-11.e narrow/
    widened native effect scope, I9-AC-14.c retention versus finite actual-delivery/provider
    metrics (actual #21 telemetry), and I9-AC-9.c Human self-adjudication impact,I9-AC-9.f Human-only blocker clearance impact,
    I9-AC-18.g normal cleanup bound/profile readiness, I9-AC-11.d scoped applicability/resource impact and
    round/artifact-quota exhaustion frequency, retained
    Task-time and decomposition/termination impact (I9-AC-14.e below). Branch blob URLs
    are not stable obligation owners; a permalink retains the gated SHA after
    branch deletion. If a public owner does not carry
    its obligation, this Issue cannot close until that tracking is concrete. This
    requires traceability, not completion of #20 or a cyclic merge dependency.
    Status/Human attention must state that no in-runtime Human action can release
    a Lost hold before trusted #14 recovery. This canonical handoff additionally
    covers I9-AC-21.e trusted 15/24 Human ingress,I9-AC-21.f actual 5/6/7 producer contract ownership,
    and I9-AC-21.g actual #14/#27 fairness/resource recovery and #12 verification evidence.
    Public tracking, rather than downstream12/15/24/27 implementations, is required
    before #9 closure; actual core-native declarations/support under 4/I9-AC-16.b remain real
    pre-closure inputs, not tracking exemptions. No cyclic merge dependency is introduced.
    New I9-AC-21.h tracks12 applicability producer/conformance and 9 consumer ownership.
    Public #12 also quotes I9-AC-10.d complete delta impact-closure producer obligations, while
    16 records actual full-fallback frequency and delta-efficiency separately. Public #13
    quotes I9-AC-13.a exact post-certificate irreversible reconciliation, preserving original
    certificate/outcomes; tracked obligations do not assert those ports implemented.

**21.b — Default measurement handoff.** Safe retry/timeout defaults are retained until real #16 measurements justify a
separately reviewed policy change. Native dogfood reports how often partial-output
cancellation, retry exhaustion and long reviews require Human attention; early
stop is expected to hold when cancelled members emitted unexamined content.
This does not invent an automatic weak-policy fallback to reduce interruptions.

**2.a — Potential Medium veto.** I9-AC-2 also rejects outvoting a potential Medium finding: two approvals plus
that dissent stay held, while safely settled Low-only dissent outside the blocking
set may be tolerated. Independent resolution is recorded before any certificate. Test
unchanged-target safely settled quorum after nondisputed trusted Human dismissal versus
a new full-roster round after dispute, ESCALATE or actual fix; no approval carry or floor substitution.

**8.b — Copy provenance.** I9-AC-8 excludes raw-output/transcript copies by recorded provenance as well as storage
location: known exact output-hash copies and paths declared by the scoped artifact policy are
excluded with a visible note, or hold before input if mandatory. A committed copy under
doc/verification does not become eligible merely because Git tracks it. Unknown content is not
claimed to be exhaustively recognized; native/source visibility limits are reported. Test
committed copies, explicit requests and symlink aliases. If an excluded copy is
mandatory Task-base→target coverage, hold before input with its exact named path.
The supported exit is an authorized new target commit removing or summarizing that
copy while preserving every mandatory unresolved original claim in typed claim
coverage. No automatic transcript rewrite or raw executor-chat eligibility is
introduced. #16 reports this hold/exit frequency and Task-time.

**7.a — In-round mutation authorship.** Any reviewer/external mutation invalidates
that entire round and retains observed tree/source/trace evidence, attribution or
unknown-origin taint and actual ownership holds. Mere temporal overlap does not
identify a native author. If a genuine owned restoration producer proves exact
locked HEAD/tree AND all relevant dirty/ignored/governing-source bytes restored, and
mutated bytes never became accepted Task artifacts/contributions, those discarded
transient bytes do not add a Task delta-author family. They still remain invalid
review history; proven native read-only violation revokes that actual profile
qualification until genuine corrected conformance, and unknown native/resource
cleanup stays held regardless restoration. Restoration is no cleanup certificate.
If mutated bytes are retained in the next accepted target, actual proven native
origin joins cumulative authors; retained unattributable bytes follow I9-AC-12.d refusal/
conservative exclusion. Accepted historical contributions remain cumulative even
if later reverted, so reversion cannot launder an already accepted author. Unknown
restoration/applicability cannot claim the discarded-transient exemption. Tests cover
exact verified restoration, retained native bytes, retained unattributable overlapping
mutation, earlier accepted-then-reverted contribution and uncertain cleanup; criterion
I9-AC-20.b/21/#16 separately measures resulting zero-eligible frequency/Task impact.
#9 owns the typed restoration-evidence checker, not an automatic reset/discard
mechanism. Production restoration is an explicitly authorized Human/Workflow action
with actual #60 owned source/effect evidence (and #14 recovery where native ownership
is uncertain); absent that reviewed producer, no production exemption is available.
The checker positive uses exact Git/dirty/ignored/governing-byte restoration in an
owned controlled fixture through21.j, restoration_basis=controlled-fixture; this is
noncertifying consumer evidence, not a production restoration/cleanup receipt.
Missing/partial/unknown evidence keeps authorship taint and all native holds.

**12.d — Forged attribution.** I9-AC-12, I9-AC-20 tests forged Git author/trailers: runtime-owned Codex dispatch/delta attribution
keeps Codex excluded despite a commit string naming Human/Grok. Uncovered delta bytes require
explicit recorded disposition before formal input. Test policy attempting a nonconservative
unknown-delta disposition rejects; a changed executor cannot silently self-confirm its
own earlier changes.

**9.b — Class-wide verification.** I9-AC-9, I9-AC-10 class-wide verification
records the checked locations and N inspected / M changed, preserving scope and check artifact
provenance; synthetic facts do not claim exhaustive native inspection.

**4.a — Default Triple positive.** A Human-authored Task with no native-family delta
author admits the default no-self Triple preset with three actual registered I9-AC-8.c-conformant
profiles under QUICK/STANDARD and STRICT. All three opinions and exact settlement remain
required by all mode, and the appropriate independent floor applies. Controlled transport
may test mechanics only; the qualifying real-native profile basis cannot be mocked.
In the isolated component positive, the test's explicit authored delta is recorded
through the trusted library-composition ingress with origin=library-composition and
nongating provenance. This is declared fixture authorship at creation, not a later
unknown→Human relabel, a native-author override or biological Human proof. It is
usable only by fixed21.j conformance, never formal production attribution. The
same harness-written delta WITHOUT that explicit record must refuse with zero
eligible native reviewers. Production Human attribution still requires actual
trusted ingress and full original byte/contribution provenance.

**20.a — Strict roster availability.** STRICT fixed Triple is unavailable when document drafting and implementation have contributed
two native families. Supported alternatives are an eligible custom roster with actual I9-AC-8.c conformance meeting the same
floor, or explicit Human adjudication selecting an eligible roster, termination without a certificate; allow-self never supplies the missing
independent approval. #16 must exercise this availability limit alongside supported Triple
configurations.

**9.a — Blocker dispositions.** Every disposition removing a potential/verified blocker from the effective veto
uses the same explicit Human/original eligible finder/two diverse eligible confirmer
rule. Fix, false-positive dismissal, severity downgrade, not-applicable, duplicate
and superseded-by-target-change are typed dispositions; unknown dispositions reject.
Actual source removal/checks are facts, but executor-only applicability conclusions
never clear the blocker. I9-AC-9, I9-AC-19 test target-change applicability and duplicate
claims under all/quorum/any/supersession: unresolved independent confirmation holds.

**11.a — Exhausted decomposition.** I9-AC-11 tests Human-authorized decomposition after lineage exhaustion with an
open High: descendants retain its veto, original base authors and shared exhausted
round/quota counters. A child certificate remains held; relabeling the frozen child
base or creating an autonomous Goal follow-up cannot restore eligibility/budget.

**1.a — Observed parallel admission.** I9-AC-1, I9-AC-14, I9-AC-18 prove actual parallel admission with production adapters and reviewed
synthetic contract peers: with M available resource shares and local parallelism M,
every member is observed admitted before any member settles, with overlapping
owned operation lifetimes. This observed admission and overlap is retained
as evidence for two and Triple rosters. Parallelism 1 reports serialized execution,
never parallel success; unavailable shares queue visibly without bypass.

**3.a — Policy relaxation.** I9-AC-3 rejects automatic post-opinion all→quorum relaxation; explicit Human
relaxation above floors starts a new retained round. A frozen pre-admission conditional
policy is separately attested, not fabricated after dissent.

**18.e — Concern-linked confirmation.** I9-AC-18 allows one
concern-linked inspection confirmation for APPROVE with a potential Medium blocker;
it still requires independent blocker resolution and rejects a second same-tree
autonomous confirmation.

**21.c — Settlement availability metrics.** I9-AC-21's #16 handoff must include settlement-expiry
Lost frequency and retained capacity/Task-time, not only inference timeout counts.
Report successful-result normal-completion cleanup expiry separately from
cancellation/timeout cleanup expiry, user-hook/tool-stage or jitter/escape conditions,
and restart/broken-supervisor Lost causes (I9-AC-18.b). Record frequency, held capacity and
Task-time separately; any may retain an absorbing hold until #14. Actual 14 owner recovery
is the product restart-recovery requirement, not a retrospective APPROVE.

**12.e — Human floor preservation.** Human adjudication never substitutes for the mandatory independent approval floor
and never mints a formal Review-Set certificate below F. It may select an eligible
roster or terminate without a certificate. Obligation-preserving decomposition is structural history, not an eligibility/budget recovery exit. No Human-only Workflow gate override is implemented or promised by this Issue; any future override requires its own explicit requirement/ownership gate. I9-AC-12, I9-AC-20 test
two-author-family STRICT Triple plus Human adjudication: still no certificate.

**20.b — Zero eligible provider set.** When all three MVP native families are cumulative authors, every class has zero
eligible reviewers within that provider set. QUICK and STRICT reject before input
with named exclusions; allow-self, decomposition and Human opinion never restore
eligibility. Supported exits are an eligible registered non-author provider that
meets the same capabilities/floors, or termination without a certificate. Such a
fourth provider is not promised by this MVP. I9-AC-17, I9-AC-20 cover this unready state;
I9-AC-21 requires #16 to report its frequency and availability/Human impact.

**8.c — Fresh native identity.** Every member in every round requires a freshly created native Session, never an
executor Session, a resumed native history or a prior-round native Session. A new
runtime UUID alone is insufficient: the actual adapter startup contract must
guarantee fresh native creation and reject conflicting resume/continue configuration
before input. I9-AC-8 rejects owned executor/history reuse and unsupported
fresh-session capability. Native auto-loaded global/user instruction files, memory and external history/context
channels require concrete registered-profile evidence before they can support an
independent-gate readiness claim. Preserve native auth/hooks/rules/default behavior.
Classify each channel RELATIVE to the frozen roster, admission schedule and actual
source scope/keys, across ANY provider families. Current-peer capability means a real
current-round peer has a reachable write/load path before another member consumes it;
shared family alone is not such proof, and different family alone is not isolation.
A single-slot-per-family Triple still needs actual channel inventories/conformance: it
admits only when they demonstrate no current-peer injection path for that roster/schedule,
otherwise refuses before input. Classify actual observed current-peer injection, documented
current-peer-capable channel, reachable channel with unknown scope/timing, or an actual
supported profile whose reviewed source/control/conformance demonstrates its bounded
current-round peer-free starting/input behavior. Family equality alone is no class.
Observed injection invalidates the round. A documented capable or unknown reachable
current-peer channel rejects the WHOLE formal round before ANY member input. Uncertainty
disclosure, fresh SID, read-only label or an abstract 'inert' declaration is insufficient.
Its actual registered producer must
provide repository/session memory key/scope, read/load and write timing, method/source
version and concrete supported-profile controls; no docs-only or synthetic label positive.
A load-at-start/concurrent admission argument requires demonstrated peer-free initial
snapshot AND absence of current-round peer writes before ALL relevant loads/admissions;
overlap alone supplies neither. Serialized same-family members with an unproven capable
channel cannot supply STRICT's F=2. Actual proven custom same-family profiles remain
possible; there is no universal family-diversity floor or deletion/disable/default override.
Unknown or unavailable required profile evidence for ANY frozen roster member/channel
rejects the WHOLE formal round before any member input, irrespective of all/quorum/any,
allow-self or an otherwise sufficient floor. Do not skip an unproved slot or reduce M.
Test a four-slot roster with three proved profiles and one unproved reachable channel:
zero input/claim even when quorum and F could be met by the other three. Separately
labelled nongating diagnostics/opinions cannot become certificates.

Certificates cover runtime-controlled input/provenance checks plus this declared actual
profile basis; they do not prove OS secrecy, passive filesystem unreadability or removal
of all prior native history. Known unrelated/prior exposure is still recorded. Governing
rubric currency remains exact activated normative authority; native memory is not a
substitute for it. Actual installed-profile conformance/#16 reports eligibility/refusal
and availability for serialized/concurrent profiles, observing auto-injection separately
from passive tool/filesystem visibility. Tests reject declared capable/unknown reachable
channels without actual conformance and observed current-peer injection; actual supported
peer-free profile controls may admit, preserving defaults. No claim that slot A actually
stored or injected a peer finding follows merely from channel documentation.

Primary documentation, not inspected private user memory, establishes the channel:
[Claude memory](https://code.claude.com/docs/en/memory) describes repository-shared
worktree auto-memory loaded into new local conversations; separate linked cwd is
not isolation. [OpenAI config reference](https://developers.openai.com/ja-JP/docs/config-file/config-reference)
documents existing-memory injection enabled by default and an idle eligibility
window for generation. These sources do not prove this project's actual peer finding
was stored/injected or installed effective policy. Actual installed-profile evidence
remains required; no memory/history deletion or native default override is implied.


**14.b — Expansion renewal accounting.** Expansion request/slot byte allowances reset only for a newly admitted (round,slot);
per-round aggregate and inherited lineage quota never reset by slot renewal.
I9-AC-17 tests that accounting across two rounds.

**9.c — Human self-adjudication attribution.** Human blocker adjudication is
an intentional application-trust exemption from native author exclusions, not an
independent opinion. Frozen policy may require non-self trusted Human adjudication
for STRICT/security; test refusal when required. The Core default permits actual
trusted Human self-adjudication in every class, including security, with explicit
certificate attribution: the operator is the application policy principal, not an
independent reviewer. This never supplies F or clears Lost/ownership. Stricter activated
policy may require non-self Human provenance. Test default flagged self-adjudication
and non-self-required rejection for Human-authored STRICT/security blockers.
The WHY is explicit escalation to the trusted application policy principal under
product §29 (and §26 for its PR/merge subset), whose adjudication is recorded as Human authority rather than a model
confirmation. Even this decision cannot count as an independent reviewer, lower F,
resolve conflicting trusted decisions, release Lost, or erase original findings.
Frozen activated policy can require independent Human review instead. This deliberate
operator policy is measured under I9-AC-9.f/#16; it is not an implicit review-floor waiver.
I9-AC-9/I9-AC-19 preserve actor/reason/evidence/flag and the explicit policy, never present
it as independent model confirmation/approval or lower F.9.f/16 reports self-adjudication
frequency and Task/Human impact, separately from model author exclusions. Exercise the
I9-AC-2.a same-round/new-round branches through actual scoped Human ingress and preserve
all original opinions.

**12.f — Activated governing authority.** Policy authority (I9-AC-3, I9-AC-12, I9-AC-15) resolves from an activated Runtime/registered
Project policy snapshot outside the reviewed Task delta and executor-writable
worktree, with source revision/digest and explicit controller/Human activation
provenance. Task-authored config edits are proposals only, never activated authority
for that Task's own review. A conditional branch/allow-self/allow-duplicate-agent/
early-stop/diversity permission cannot be smuggled into admission by changing a
worktree config before freeze. Independent policy activation retains the prior
mandatory floors and cannot clear the reviewed Task's blocker obligations. Test
author-written conditional relaxation is rejected while the prior activated policy
remains effective. Native filesystem protections remain separately declared.
Governing review instructions, required skills and mandatory rules use the same
activated revision/provenance; Task-delta edits are reviewed proposals, not rubric.

**10.a — Upstream integration attribution.** Base-sync attribution (I9-AC-10, I9-AC-12, I9-AC-20) records actual registered-base ancestry
and the exact integration operation. Legitimate upstream content reachable from
the registered base branch is attributed upstream, never invented as this Task's
native author or unknown contribution. Retain all previously recorded Task authors
across merge/rebase and retain the immutable original obligation base in history.
A base advance is allowed only by the recorded registered-base operation; it cannot
move this lineage's authored changes behind a new base to erase exclusion. Test
upstream merge adds no unknown authors and keeps prior exclusions, while a proposed
base advance to a Task-authored commit/foreign branch rejects. Previously Task-authored
content remains attributed even if later reachable upstream.

**20.c — Certificate-capable exits.** Supported two-author-family STRICT exit explicitly permits an eligible custom
roster of two fresh Sessions from the remaining non-author family when activated
policy allows duplicate-agent identities or distinct registered aliases AND actual I9-AC-8.c
profile conformance for that exact roster/schedule/source scope is present; mandatory
family diversity, if imposed by that policy, still applies. Otherwise use an actually
supported non-author provider or terminate. Decomposition never recovers eligibility
or exhausted quota. Tests distinguish certificate-capable roster selection from
terminal-only exits. Actual #5/#6/#7 or a publicly linked reviewed producer follow-up
must own at least one conformant repeated-family profile for this two-author STRICT case.
Issue 9 MUST stay open until an actual qualifying repeated-family profile demonstrates
the required 4+/two-author STRICT positive cases. While absent, this pattern is explicitly
Core-unready and refuses before input; the refusal alone does NOT close those positive
obligations or limit the promised scope to single-author Tasks. No fourth-provider promise, floor waiver or unsupported profile
can stand in for the required positive.

**21.d — Accepted pre-recovery risk.** The conservative pre-#14 settlement-expiry Lost policy can
permanently hold its Task lineage/worktree and owned shares even after late cleanup.
For a supported profile whose actual effect/containment producer independently proves
a bounded worktree-only effect scope, expiry retains that proven scope; permission
read-only labels alone cannot narrow it. If actual evidence leaves potential root/
common-Git effects or escaped-work scope uncertain, hold conservatively widens to
that full physical scope, potentially fencing the Project and physically overlapping
Projects until actual Issue 14 recovery. A timer expiration neither proves a larger effect
occurred nor erases an already proven bound. Public Issue and I9-AC-21.c/#16 report the
true scope, including Project/overlapping-Project fences separately from Task holds. The absorbing choice also applies when the original supervisor
remains continuous and observes complete settlement after the declared deadline: it
missed the qualified profile envelope, so that late observation is factual evidence,
not timely completion/release authority. It retains ownership until authentic14 recovery.
This is an intentional availability limit, not an assertion that every late process
is physically alive. #16 separately reports deadline-expired continuous-owner late
settlement versus broken supervision/restart, escapes and other uncertain causes.

**14.c — Retained artifact accounting.** The 128-MiB quota bounds actual retained
rrx evidence, separate from cumulative actual delivery. Shared core stored once per round is
charged once to lineage artifact quota; distinct member frames, results, expansions
and manifests are charged by actual retained bytes, including duplicated core bytes
when a member frame stores them. Actual native input bytes remain measured.
Identical source blobs can be deduplicated by actual ownership/content identity;
separate copies/rows/manifests all charge their actual retained bytes. Immutable
external Git source references charge their persisted metadata and any rrx copies,
with declared external owner/availability/bounds. Repeated delivery of that source
still counts every actual UTF-8 byte against the frozen expansion ceiling. Across
shared 64 rounds,32 slots and 1-MiB complete prepared frames plus 32-MiB aggregate
expansion per round, rrx-controlled prepared-frame+expansion delivery is finitely
bounded by 4 GiB per obligation lineage (including descendants/successor Sets),
not an unlimited reference exemption. Native added prefixes/wire envelopes have
separate actual producer bounds and nullable native telemetry;4 GiB never claims
native auto-loaded context or token/cost measurement. Test deduplicated/reference-
backed repeated large coverage with every delivery charged and distinct metadata
retained, plus copied-content reservation/quota exhaustion before round 64. #16
reports retained versus delivered bytes, amplification and quota-before-round-limit
frequency separately under I9-AC-21; actual #21 provider token/cost counters remain nullable.
All retained ReviewSet evidence consumes the same 128 MiB lineage quota, including
transport diagnostics, verification/check logs, adjudication, attention records,
manifests, result/expanded/member frames and claim/exposure records. No uncharged
evidence kind creates unbounded output; references identify externally retained
artifacts and their declared separately owned bounds, never hidden copies here.

**16.c — Read-only proof basis.** Certificate basis is reviewed adapter contract
plus runtime permission construction until #16 real native enforcement evidence;
synthetic peers do not prove native enforcement. Result/certificate and public
checkbox annotation records `read_only_basis=contract+construction`, explicitly
native-enforcement-pending16. Actual native memory/channel qualification remains
mandatory here and is separate from this read-only enforcement basis.

**8.d — Prior-round exposure.** A transient same-target retry with one prior
non-result slot and completed peer findings delivers only permitted labelled prior
claims after safe settlement. Its eligible fresh opinions may count F only with I9-AC-8.c
actual registered-profile conformance for current-round independence; certificate records exposure and never claims
blind/lifetime independence. A current-round peer finding delivered before every
roster slot settles rejects certification. #16 reports blind and exposed
round results separately.

**12.g — Governing rubric provenance.** A Task-authored weakening/removal of review
instructions, skills or mandatory rules is delivered as reviewed diff/content while
the activated version remains the governing rubric and certificate digest. A conflicting
native auto-discovery profile that cannot preserve the declared rubric rejects before
member input without bypassing native defaults. Current registered-primary resolution
is not mislabelled as Task-root authority. Policy weakening is likewise only a proposal.

**11.b — Unreleased lineage ownership.** Terminate-then-new-Set and terminate-then-
decompose cannot admit any descendant formal review while a Lost/uncertain member
remains in the obligation lineage. Hold/taint/locks/actual resource shares survive
new worktree/Task identities; unrelated lineages may use remaining resources.
Trusted #14 recovery is the only releasable exit, never a generic late terminal fact.

**9.d — Human ingress provenance.** Native/Workflow/Broker model JSON, proposal,
runtime API/IPC and ordinary persistence cannot mint Human dismissal, downgrade,
authorship disposition, relaxation or policy activation; actor=Human is rejected
with retained evidence. Actual trusted Human ingress succeeds with exact
principal/origin/digest/evidence. Direct same-UID local binary/DB/machine action
outside these APIs is explicitly outside the application guarantee, as in Issue 23.
The positive authority is the actual Issue9-owned provider-neutral library Human-ingress port,
not a mock actor field or an assumed future Issue 23 implementation. Its evidence
records origin=library-composition and proves that application port, not biological
Human action; no CLI ingress is implemented or implied by this library fixture. Production
certificates consuming a Human event bind its actual principal, invocation origin, scoped
event/reason/evidence and immutable digest, and require that origin in the activated
accepted-ingress set. Bare library/test-fixture origins are nongating; handler existence
alone does not authorize an event. Test unaccepted library-origin activation/adjudication/
supersession rejection. Issue #9 closing Human-boundary positives for I9-AC-2.a/3/11/I9-AC-9.c, I9-AC-9.d, I9-AC-9.i
use its ACTUAL library-composition port with explicitly nongating origin and negative
relabel tests; they prove the component, never product certificate authority. Actual
product-ingress positive belongs to public #15/#24 under I9-AC-21.e, I9-AC-21.i and remains a mandatory
MVP enablement gate, not a conditional Issue #9 library-test waiver;
never accept native/controller/API origin relabeling. On actual23 composition, this interim
derivation is REPLACED by importing the one canonical #23 type/derivation, not wrapped
with a second independently trusted constructor. Convergence tests show identical
principal/origin for identical accepted invocation and reject the removed interim
constructor; trusted library origin remains nongating. No divergent derivation may
remain callable alongside the production23 authority.

**8.e — All-slot no-show window.** With an allow-self author slot or a queued/
duplicate slot still active after eligible peers finish, current-round findings
remain excluded from every runtime input/expansion. A contaminated opinion cannot
count toward mode or floor even if the independent floor could otherwise pass.

**8.f — Narrative provenance.** A checkpoint or condenser narrative asserting a
dismissal/tests pass without matching actual runtime evidence remains a labelled
actor/hash claim in core; it cannot clear a blocker or certify checks. Genuine scoped
check artifacts retain distinct runtime provenance, and content hashes attest bytes.

**12.h — Specialization authority.** Task-authored focus/ignore instructions are
reviewed source/proposals, never activated formal specialization. Activated scoped
specialization with controller/Human provenance is focus-only and cannot drop core.
Explicit ignore-security/ignore-mandatory-rule or suppress-blocking-severity intent
refuses activation, including a trusted-origin request; byte-identical core delivery
alone cannot qualify that slot. Test genuine focus activation and this refusal.
Task-authored hints remain unactivated proposals, never permission to omit rules.

**11.c — New Task lineage base.** Registered-base sync retains recorded uncertified
contributions even after upstream reachability. This includes contributions recorded
by earlier Workflow epochs whose legacy Passed/Completed labels provide no current
ReviewSet certificate. They retain evidence-backed authors under12.d; legacy content
lacking that binding remains unknown authorship (refuse/exclude every candidate family).
All uncertified obligations remain; unknown applicability inherits/refuses rather than applying an epoch cutoff or converting an
old native result into certified upstream proof. A new obligation may be fresh only
with trusted exact applicability evidence proving disjoint changed AND relied-on
source/dependency/context and retained finding scopes. Affected or unknown obligations
inherit exact vetoes/authors/shared remaining or exhausted budget, or refuse. No
filename-only, label, rebase or Human/executor JSON laundering is accepted. Untracked
semantic copying/re-authoring is not claimed exhaustively detected.

**14.d — Large target coverage.** A >1 MiB target diff with bounded core and exact
required expansion references can pass only after each slot's complete required
coverage is delivered within frozen limits. Missing coverage or unsupported typed
delivery holds. An over-cap mandatory core or beyond-budget required coverage has
the declared NeedsContext/termination availability outcome, measured by #16.

**18.f — Four-plus policy.** Four-plus production-adapter consumer evidence uses
activated explicit allow-duplicate-agent or production-registered aliases of the
MVP adapters, with that permission visible. Fixture-only registrations cannot pass.

**10.d — Delta coverage scope.** Round 1 requires complete Task-base→target
review coverage delivered to each slot. A later certificate-capable delta round
may use an exact settled full-coverage baseline with the same frozen roster,
policy, governing instructions and obligation scope. Current relevant source/rule
pins match that baseline except the exact declared cumulative target delta;
unrecorded source/rule drift invalidates baseline eligibility. Every baseline slot must have
complete delivered coverage and a structured complete opinion, with actual native
settlement and no Lost/partial/failure. No prior approval is carried into counts. Certificate records every baseline slot's
unchanged-region basis as delivered+prior-approving OR delivered+prior-nonapproving,
without injecting prior verdict/tally into the new member input. A prior RequestChanges
baseline may qualify only as complete delivered/assessed coverage with exact unresolved
claims included, never an assertion unchanged regions were approved. Test that provenance
on a later fresh approving delta round, and policy may require approving baseline if
activated before input. The new required coverage is cumulative baseline→current target delta plus all
unresolved claim locations/evidence and current mandatory core, PLUS all affected
consumers of changed symbols/APIs/constants/thresholds/environment names/input sets/
paths/dependencies. Exact current-target impact evidence under the typed consumer
contract identifies those required unchanged callers/consumers and inspected scope,
with exact method/source/evidence basis and known-incompleteness classes. Certificate
records that basis and never claims an exhaustive semantics theorem. Method capability
must cover each changed kind: lexical-only map evidence cannot claim closure for
unknown dynamic dispatch/config/threshold/environment consumers; unsupported kinds
are UNKNOWN, even if a producer labels an incomplete list complete. A false completeness
label without method-supported evidence rejects. Tests exercise lexical-only changed
threshold/environment consumers falling back to full coverage;
missing/stale/unknown impact closure invalidates delta eligibility and forces full
Task-base→target coverage. A full fallback still preserves explicit impact-analysis
verification obligations; broad diff delivery alone is not semantic impact proof.
There may be no
unresolved finding outside that required scope. The certificate explicitly binds
baseline round/tree/coverage and new delta coverage, rather than claiming each fresh
Session reread unchanged files. Intermediate delta rounds never become a new full
baseline just by producing APPROVE. Missing/nonqualifying baseline, changed roster/
policy/instructions/scope or unresolved outside required coverage forces full
Task-base→target delivery before certification. Test qualifying delta continuation,
rejected delta-only without baseline, unchanged-file coverage provenance, cumulative
multi-fix delta, a changed contract with unchanged callers requiring their delivered
coverage, stale/missing impact evidence and full fallback. Actual delivery counts
against frozen slot/round byte ceilings even when referenced source is deduplicated;
retention follows I9-AC-14.c, not a second charge per delivery. The qualifying-delta
consumer positive closes using exact Git-backed controlled impact evidence through
fixed noncertifying21.j ingress, impact_basis=controlled-fixture, conforming to #9's
typed contract. It does not certify semantic applicability or an absent #12 producer.
Production reduced coverage stays unavailable (full Task-base→target fallback) until
actual #12 completeness proof composes, publicly tracked under21.h/21 without a
#9→#12→#20→#9 merge cycle. Public checkbox8 requires typed delta consumption plus
this safe full fallback, not unavailable production reduced-coverage certification.

**8.g — Consumer contract ownership.** Actual Git-backed bundle, revision/identity
delta and expansion fixtures conform to the typed consumer contract owned by #9.
Foreign/stale/relabelled artifacts refuse through the actual consumer. Public #20
acceptance must quote this conformance obligation before #9 closes; production
#20 remains separately unimplemented, with no cyclic merge gate.

**11.d — Certified upstream and scoped applicability.** A certified-and-actually-merged
target is legitimate accepted upstream for a new unrelated Task, with no inherited
review authors/budget. An out-of-band merge retains explicitly unaccepted provenance
and applicable obligations; it never becomes a certificate. Test actual disjoint
new progress after unrelated exhaustion, dependency/context overlap inheritance,
relabel/rebase laundering rejection, unknown applicability refusal/inheritance and
root/common-Git scoped Lost blocking even a review-disjoint Task. #16 measures
applicability/unknown holds and their Task-time separately from disjoint progress
and actual native/resource-scope blocking. Human opinion cannot release Lost or
waive the certificate floor. Current applicability follows this evidence-bound scope rule.

**14.f — Queue attention bounds.** Queue threshold 0/3601 rejects;1/3600 is accepted
within the remaining policy, default 600 is recorded. Under ordinary non-Lost-blocked
contention attention crossing stays queued; permits returning after the threshold
automatically admit with current currency. Lost-blocked wholly queued proposals
park under18.h; genuine recovery permits full revalidation without a new retry cause.
Test that no Human decision is required and stale target still refuses. Queue attention
frequency/Task-time goes to 16; it never invents native timeout or releases ownership.

**9.e — Original finder identity.** A non-finder duplicate slot sharing an agent
ID cannot act as the original finder. Only the same frozen slot key+agent in a later
round's fresh Session can use that disposition, if neither its identity nor family
is an author; changed rosters use Human or the independent confirmer pair.

**14.e — Claim history and lineage exhaustion.** The mandatory core includes a
bounded exact manifest of prior round/slot/finding artifact hashes and current
resolution-status provenance. Full resolved/dismissed/superseded texts remain
immutable exact-referenced history; unresolved potential/verified blocking claims
must be delivered to every member as inline text or required hash-bound expansion
before certification. All delivery charges the frozen slot/round/lineage budgets;
no hidden unbounded history or silent omission is allowed. Record which claim
texts each slot received and label its exposure accordingly. Test accumulated
claims near the 1 MiB core cap: a bounded manifest/required delivery can continue,
while mandatory metadata or required texts beyond capacity holds NeedsContext or
terminates without certificate. #16 measures this availability outcome plus round/
artifact-quota exhaustion frequency and its Task/decomposition/termination impact.

**12.i — Supersession/risk roster authority.** Automated Workflow supersession to
remove a nonblocking dissent rejects. Risk escalation from STANDARD to STRICT with
one eligible slot holds for trusted Human roster selection or termination; it
cannot silently relax the floor or create a new roster after opinions. All
obligations/budgets/holds remain. A frozen stronger no-roster-change branch records
its pre-admission activation and exact trigger. Test immediate risk-signal hold,
independent/Human verification rejecting escalation and verified risk raising the class;
rejected signals cannot silently change roster/class and accepted signals cannot waive F
or downgrade an already applied class.

**14.g — Admission reservation and retention.** Reject a round before input when
remaining lineage quota cannot cover its worst-case frozen reservation; an exact-
reservation boundary admits and retains every bounded in-flight result/diagnostic/
expansion. Test numeric diagnostic/model prefix and shared verification/journal
ceilings, unknown suffix and reserved control margin. Held members retain unused
reserved capacity; safe closure releases only unspent reserve, never actual history. Test
two children sharing one origin-phase root versus distinct phase roots, concurrent combined
overflow, inherited author/veto union and fresh-ID reset refusal. Atomic shared-root overflow
admits at most one winner without losing any retained charge.

**13.a — Post-certificate veto.** Later actual evidence raising a certificate-bound
Low into the effective blocking set makes that original certificate stale/non-
consumable while preserving it. A downstream actual review-gate consumer rejects
it; already irreversible merge/PR outcomes stay factual for 13 reconciliation, never
fabricated rollback. A new certificate requires applicable current review authority.

**9.f — Clearance availability.** If neither an eligible original finder nor a
pair of diverse non-author confirmers exists, blocker clearance is trusted Human-
only and holds until that adjudication or termination; quorum/allow-self cannot
clear it. #16 measures this Human-interruption frequency/Task impact, distinct
from independent approval-floor availability, under I9-AC-21's canonical handoff.

**18.g — Normal and forced cleanup bounds.** Actual #5/#6/#7 caller conformance declares supported
normal AND cancel/startup-timeout/review-timeout/early-stop forced hook/tool/native/
server shutdown, term/kill/reap and owned-settlement stages, with each enforceable
conditional envelope and required margin. Both must fit, unknown either refuses. At default 30s with 100ms margin, 29.9s accepts;
29.901/29.999/30s or unknown in EITHER path refuses. Test normal 2s+forced 35s at 30s
refuses BEFORE input, fitting both COMPLETE envelopes accepts, and forced-unknown
refuses. Normal 20 s prefix then forced 15s at 30s refuses even though both standalone
durations fit. Cancel during normal cleanup uses the SAME earliest timer and the
remaining composed bound; never restart/extend to hide35s of actual ownership. Ceil
29.9004s to 29901ms then refuse at 30s/100ms; exact29900ms accepts.
A profile-required stronger margin is enforced;
configured deadline1–3600 must cover exact envelope+margin using checked arithmetic.
Test preserved user hook/MCP configuration with known bound versus unbounded preinput
Unsupported; after dispatch unexpected delayed hook/jitter-at-margin/escaped cohort
retains actual supervision/Unknown ownership for #14. No disabling userdefaults and no
universal real-time claim. One normal-result timer starts at complete owned result/
native terminal/cleanup request, never partial tokens or repeated terminal extension.
Percentile/typical/arbitrary-timeout-only declarations refuse. Actual Issue 16 owns observed
profile/default suitability; library timestamps alone do not validate native support.


**9.g — Actual disposition callback.** A later complete opinion must explicitly deliver
the required assigned prior-finding dispositions under I9-AC-9.h through the typed
production consumer contract. Prove an
eligible finder’s confirmed-fixed with exact inspected locations/check evidence clears
its prior High after a new full round, while general APPROVE, unknown hash, stale target,
wrong slot/family, malformed disposition or executor-only claim keeps the veto.
Finder still-present versus two diverse confirmed-fixed holds for trusted Human;
post-opinion confirmer redesignation rejects, including an activated designation
between rounds after that agent/slot already expressed its opinion.
Retain every original finding/opinion and disposition independently.

**21.e — Trusted product Human ingress.** Issue 9 owns the library authority/ingress
contract and negative native/Workflow/Broker JSON/API/IPC tests. Actual #15 TUI and #24
Goal/controller handlers must publicly own their composition for every Human-only
adjudication, policy/scope/supersession and termination exit, including exact scoped
reason/evidence and principal provenance. No such handler is claimed implemented.
Until available, status exposes integration-pending/nonactionable attention rather
than a fictitious button/CLI escape. Public acceptance quotes this stable ID before 9
closes; product handler completion remains a separate MVP gate, not a9→15 merge cycle.

**21.f — Native contract producer ownership.** Issue 9 owns consumer conformance and
runtime declaration checks; actual 5/6/7 adapters or explicitly linked reviewed follow-up
Issues own producer obligations for I9-AC-8.c fresh no-resume native identity,I9-AC-16.b/I9-AC-18.g normal
cleanup shape/bounds,I9-AC-14.g bounded retained Session/private pair/operation/receipt/usage/
audit overhead needed for complete pre-input reservation,I9-AC-18.c requested/effective configuration,I9-AC-16.b/I9-AC-16.c runtime read-only
permission construction,I9-AC-16.d actual read-only permission-request DENY/zero-ALLOW
and visible typed reason/hold through supported profiles, and I9-AC-8.a available access traces. Public owner entries identify
exact supported profiles and refusal limits before #9 closes. Existing declarations or
synthetic fixtures do not assert these producers or real native acceptance complete.
Actual I9-AC-8.c registered-profile conformance is a pre-closure positive prerequisite for
EVERY floor-support case in 1/4/I9-AC-12.b/I9-AC-17.a/I9-AC-18.f/I9-AC-20.a/I9-AC-20.c, not a tracked-follow-up waiver.
Unproven repeated-family profiles are not advertised as a supported STRICT exit; use
an actually eligible provider or terminate. Test real declared documented-capable
duplicate roster refusal and single-slot-per-family Triple actual proven admission or
explicit refusal. Source-control mechanics may be synthetic only where clearly labelled;
no abstract inert profile substitutes for actual declared production conformance.
Minimum qualifying evidence is a reviewed actual producer source/configuration/control
inventory for every reachable repository/session memory channel (scope/key, load/write
timing and source method), actual runtime permission construction, AND owned isolated
real-native conformance observations for the exact roster/schedule/source scope under
preserved defaults. Source declarations and synthetic transport tests alone cannot qualify
I9-AC-8.c. These producer-specific native checks may run independently before Issue 9 closure;
they do not require completion of Issue 16 representative ReviewSet dogfood, do not complete
it, and do not claim OS secrecy. Unknown reachable channels remain unready. Public
producer acceptance records whether the preserved-profile normal/forced cleanup fits the
default 30-second deadline; otherwise only an explicitly preactivated finite 1–3600-second
policy may enable a proved profile. No automatic extension or hook/default removal.
Issue 16 separately measures default and configured-profile availability.
Unknown actual overhead bounds refuse admission; tests use the real bounded producer
DTO/output path and actual row-byte accounting, not library constants in place of a
native declaration.16.b/I9-AC-14.g carry exact admitted/refused boundary evidence.

**21.g — Fairness with held shares.** ReviewSet retains exact global/Project/provider
shares under Lost; its local parallelism/per-Project cap never releases them or invents
extra capacity. Prove remaining actual capacity progresses a disjoint Project and
report zero-eligible-capacity as held-resource starvation, not runnable scheduling
success. Actual #27 fair admission and #14 trusted recovery must publicly own cross-Project
progress with real held shares, using #12 verification evidence and #16 scoped metrics.
If a Project can consume all global/provider capacity in retained uncertainty, that
cross-Project availability gap is explicit until the reviewed scheduler/recovery
composition;9 library tests/measurements alone cannot satisfy MVP fairness. Tests keep
root-scoped holds while allowing truly disjoint progress when limits permit. Status
and 16 distinguish actionable Human attention from recovery-pending/nonactionable holds.

**8.i — Observed executor-conversation injection.** Actual observed native injection
of raw executor/author conversational claims, verdict/tally or unapproved instructions
OUTSIDE the authorized frozen factual bundle invalidates/holds the whole round; it
cannot earn a certificate by declaring residual exposure. Extend I9-AC-8.a actual trace
violations to these proven channels and retain exact source/provenance separately
from claims of mere capability. Authored code/requirements or actual governing rules
intentionally included as authorized factual review material are not this prohibited
conversation channel. Unknown/unobserved prior memory exposure remains classified
under I9-AC-8.h, never fabricated as observed injection or OS secrecy. After actual full
settlement, any new full-roster round needs actual qualified channel/profile remedy
and current source/policy, not Human permission to count contaminated approval.
Test actual trace-observed executor-chat injection→hold versus authorized authored
source delivery and disclosed unobserved prior history; #16 measures frequency/impact.

**8.h — Prior verdict input exclusion.** Runtime member input/expansion excludes
separate prior-round verdict/approval-tally/non-finding peer-opinion fields, including
a same-tree transient retry after other prior slots APPROVEd. Mandatory finding
resolution-status metadata is about finding applicability/adjudication, not vote
counts. Preserve exact original finding claim text/hash and recorded prior delivery:
that prose can incidentally state an opinion and remains labelled attributed claim,
not a promise of semantic blindness. Private retained outcomes/certificates keep
prior verdicts for provenance, never feed them back as vote hints. This exclusion
applies to runtime-controlled delivery, not a claim about unobserved native memory.
Each member additionally records prior-round/executor-derived native-memory exposure
class (observed, documented-capable, unknown or actual supported-controlled basis),
separate from exact runtime-delivered text hashes. It cannot claim that delivery
manifest inventories every native auto-injected byte. Actual Issue 16 reports
claim exposure/residual native memory separately; no history/state deletion bypass.

**11.e — Native hold effect scope.** A supported formal read-only member’s declared
normal hold covers its exact Task worktree/round lock and actual global/Project/
provider resource shares. Widen physical scope to the actual root/common-Git overlap
when the reviewed producer declares those effects or actual evidence leaves them
unbounded/uncertain; never narrow by Reviewer label alone. An unsupported unbounded
profile refuses before admission; unexpected already-dispatched escaped/unknown
effects retain the full conservative affected physical scope until 14 recovery.
Test narrow supported hold permitting a disjoint Task when permits allow, and
widened root/common-Git uncertainty blocking physically affected Tasks/Projects.
This profile/evidence basis is not an OS sandbox or generic terminal death proof.

**12.j — Conflicting native rubric availability.** Task-edited OR legitimate base-sync
CLAUDE.md/AGENTS/rules that native discovery would activate differently from the
accepted governing snapshot refuse before input, with exact conflicting paths and
revision/digests. Supported exit is ONLY an actually reviewed native profile/view
that delivers reviewed source with activated rubric while preserving required native
user defaults, or termination without certificate. Such a view is not implemented; its pending actual producer must not be inferred
from this requirement; native cwd/worktree ownership requires actual 19/profile
composition. Legitimate base-sync may explicitly activate a new trusted governing
snapshot ONLY after trusted Human approval of its EXACT NEW content digest,
retaining floors/obligations, never automatic Task-edit self-activation. Controller
base-sync with an unapproved changed rubric digest MUST refuse.
Required reviewed-source/activated-rubric view producers belong actual #5/#6/#7
profiles with #19 workspace/input composition; the #9 consumer is currently BLOCKED
for this conflicting-rule Task class until that proof exists. Public I9-AC-8.l/I9-AC-21.l
feasibility tracking must name its exact missing view/owner/proof before design,
including an actual supported rule-editing positive or explicit unready result.
This is no formal-gating scope exemption or whole-MVP waiver.
Test both Task edit and base-sync drift, refusal reason and available verified exit
versus pending-profile/noexit.16 records frequency/Task availability/Human impact.

**3.b — Per-phase policy and configuration.**9 owns the typed activated Runtime/Project
per-review-phase roster/completion/policy consumer (existing ProjectOverlay cannot
add providers). Public handlers/configuration belong15/24. Test STANDARD Requirements,
Design and Implementation review plus STRICT SecurityReview with distinct activated
policies/rosters/phase lineages, security flag/floor and exact phase certificates;
wrongphase/policy replay rejects. Missing activation surface is explicit, not a
worktree-authored config fallback. The Issue 9 typed library policy consumer closes
I9-AC-3.b only with actual source/API evidence. Persisted Runtime/Project configuration schema
and actual TUI/controller activation surfaces remain separately owned product gates (I9-AC-3.c).

**3.c — Persisted configuration producer.** Actual #15/#24 public acceptance owns
the persisted Runtime/Project per-phase policy schema and activation handlers, including
requested/effective roster/model/effort/completion bounds and activated authority provenance.
Issue 9 library DTO tests do not implement ProjectOverlay provider additions or product UI.
Quote I9-AC-3.b, I9-AC-3.c in the public producer handoff; missing configuration stays visible.

**21.i — Production readiness and principal convergence.** Actual production review-gate
consumer requires real supported native producer/profile conformance and exact current
certificate/coverage/independence basis, actual Issue 14 recovery, actual trusted policy
activation and required #15/#24 Human adjudication ingress; multi-Project additionally
requires #27 fair admission. Until these compose, single AND multi-Project product gating
remain disabled.9 trusted controller/library ingress may instantiate ONLY an
already exactly Human-approved initial policy digest and
prove component mechanics, but does not claim completed product handlers or production
ready certificates. Component native/controlled tests may retain nongating evidence;
accepted pre-recovery risk I9-AC-21.d remains observable there, not an enablement exemption.
Applicability-dependent fresh budget additionally needs I9-AC-21.h actual #12 producer; unknown
inherit/refuse does not require pretending it exists. Readiness and eligibility
are scoped to the actual Task/lineage/roster, not a blanket Project promise. Recorded
uncertified ancestry with unknown applicability MUST retain inherited exclusions;
missing the required eligible floor stays BLOCKED before input even if all generic
native/product handlers are present. Actual #12 proof is required if this lineage
asserts disjoint fresh obligations/budget; no Human 'accept upstream' wash is added.
Test interim→product-readiness on retained uncertified history: no producer still
inherits/refuses, actual applicability proof may qualify disjoint obligations, and
true Lost/root/common-Git scope remains held. Clean/unaffected scopes need no fake
#12 merge prerequisite. Readiness comes from actual
implementation composition, never public JSON capability. Test actual downstream
consumer returns distinct typed basis refusal and readiness-unavailable reasons.
Both certificate provenance/basis and actual production readiness are required
before claim/reservation/dispatch; bad-basis refusal must remain observable while
readiness is absent. This
makes nongating/synthetic/foreign-basis refusal reachable even when readiness is
unavailable; global-unready cannot mask the basis check. For #9 closure, tests through this same real consumer prove bad basis with absent
readiness reports basis refusal, and a nongating conformance basis cannot certify;
typed basis refusal and readiness-unavailable remain distinct. The genuinely valid
formal-basis/readiness matrix is the separately numbered post-closure MVP obligation
I9-AC-21.n. Its pending positive cannot be counted as completed #9 evidence or become
an unnamed #9 closure dependency. Shared checker
mechanics and interim-refusal evidence cannot claim production acceptance. When #9/#23 compose, #23 owns the ONE canonical trusted-ingress derivation/type
implementation; #9 imports it and owns actual convergence caller tests. #15/#24
construct genuine production capabilities through actual scoped CLI/TUI ingress.
Native/API/JSON/library-fixture origins cannot mint Human authority; #9 owns convergence tests through #15/#24 actual handlers
reject identical native/API/IPC/library→CLI relabel attempts. Actual product readiness
requires each consumed Human event to pass I9-AC-9.d accepted-origin/digest checks, not merely
an available handler or a trusted test library principal. No #23 merge prerequisite
is imposed merely to review #9's provider-neutral contract.

**21.h — Applicability producer and production enablement.** Public #12 acceptance
must own the reviewed trusted impact/applicability producer using #18/#20 scoped exact
inputs and #9 typed consumer/checker conformance. Method and known-incompleteness basis
must be explicit; unknown/incomplete or missing producer inherits/refuses, never
fresh-budget production certification. Pre12 conservative author-exclusion accumulation
and zero-eligible Task impact from recorded out-of-band merges are explicit measurements. Actual 9 fixtures prove the consumer/refusal
boundary only. This is a separately tracked MVP availability/impact gate without
an implementation dependency cycle. Multi-Project production ReviewSet integration
also remains DISABLED until I9-AC-21.i actual readiness, including14 retained-share recovery
and #27 fair admission, composes; a synthetic library success or per-Project cap cannot erase held shares.
Public Issue 9 states that gate;I9-AC-21.g/#16 owns its actual cross-Project evidence.

The pinned requirements file is the sole normative Issue 9 requirements/acceptance
source. Public Issue Goal/scope/user checks remain, and former refinement entries are
archived as historical provenance, never competing current instructions. Numeric
boundary examples specify observable accept/refuse outcomes, not implemented algorithms.
The canonical public handoff table below summarizes I9-AC-21 ownership; individual
stable subcriteria still retain their exact stated acceptance. Quoting a gate never
claims it complete or waives actual 9 core support prerequisites. Closure checks
completeness directly from EVERY stable ID naming an external owner, including
pre-closure integration inputs and post-closure MVP handoffs. The table is an informative
index, not a second normative owner list; a missing row cannot excuse a stable ID's
public quote or proof, and closure must report/reconcile index omissions; missing public
owner quotes refuse closure, including the repeated-family positive obligations. For I9-AC-20.c/I9-AC-18.f, absence of a named
public #5/#6/#7 or reviewed follow-up producer commitment is an explicit closure-blocking
status with owner/required proof, not an implicit wait or fallback waiver. No commitment
is claimed merely because this table names an Issue; it must actually be quoted publicly.

| Stable acceptance keys | Public owner / actual responsibility |
| --- | --- |
| I9-AC-11.b, I9-AC-11.e, I9-AC-18.b, I9-AC-21.a, I9-AC-21.c, I9-AC-21.d, I9-AC-21.g | #14 exact held native/lineage/resource recovery; #27 fair admission; #16 scoped hold/availability measurements |
| I9-AC-8.g, I9-AC-10.d, I9-AC-18.a | #20 production typed bundle/delta/coverage; #9 consumer contract; #12 actual complete delta impact-closure producer; #18 captured inputs; #16 actual full-fallback/delta-efficiency |
| I9-AC-11.c, I9-AC-11.d, I9-AC-21.h | #12 trusted applicability producer, #18/#20 inputs, #9 consumer; #16 actual disjoint/unknown/resource impact and pre12 author-growth/zero-eligible availability |
| 1, 4/I9-AC-4.a, I9-AC-12.b, I9-AC-17.a, I9-AC-18.f, I9-AC-20.a, I9-AC-20.c, I9-AC-16.b, I9-AC-16.c, I9-AC-16.d, I9-AC-18.g, I9-AC-14.g, I9-AC-14.i, I9-AC-8.a, I9-AC-8.c, I9-AC-21.f | #5/#6/#7 actual supported native/config/permission/cleanup and actual bounded retained-overhead declarations or tracked reviewed follow-up; #16 real enforcement/trace/default evidence |
| I9-AC-2.b, I9-AC-7.a, I9-AC-8.b, I9-AC-8.l, I9-AC-21.m, I9-AC-18.h, I9-AC-8.d, I9-AC-8.h, I9-AC-8.i, I9-AC-12.j, I9-AC-14.f, I9-AC-16.a, I9-AC-14.c, I9-AC-14.d, I9-AC-14.e, I9-AC-17.c, I9-AC-20.a, I9-AC-20.b, I9-AC-21.b, I9-AC-9.c, I9-AC-9.f, I9-AC-14.h | #16 actual exposure, efficiency, finite delivered versus retained bytes, quotas/retries/eligibility/Human impact; #21 nullable native telemetry |
| I9-AC-3.b, I9-AC-3.c, I9-AC-9.d, I9-AC-9.i, I9-AC-21.e, I9-AC-21.i, I9-AC-21.j | #9 typed phase-policy/ingress/readiness consumer; #23 shared principal derivation; #15/#24 actual scoped product policy/Human handlers; #14/#27 actual recovery/admission |
| I9-AC-12.k, I9-AC-18.a | #60 actual owned deterministic-effect attribution producer; #12 inspected effect/source evidence; #9 checker; LLM/unknown origin refuses/conservative exclusion |
| I9-AC-13.a | #13 actual post-certificate irreversible reconciliation; no fabricated rollback or later certificate washing |
| I9-AC-1, I9-AC-1.a, I9-AC-16, I9-AC-18.a, I9-AC-18.h, I9-AC-20.d, I9-AC-12.j, I9-AC-21.j | PRE-CLOSURE: #19/#43 actual member/phase input, original-frame binding and full owned closure; #5/#6/#7 actual supported native ports; #9 fixed conformance ingress + refusing formal consumer. POST-CLOSURE: #16 representative dogfood; #14 restart/uncertain recovery; #27 fair admission |
| I9-AC-12.j | #5/#6/#7 actual reviewed-source/activated-rubric native view producer; #19 actual owned workspace/input; #9 refusing consumer; #16 rule-edit availability; I9-AC-8.l/21.l public feasibility tracking |
| I9-AC-7.a | #9 typed restoration-evidence checker; #60 actual authorized restoration/source-effect producer; #14 actual uncertain-owner recovery; #16 restoration/eligibility availability |
| I9-AC-21.g, I9-AC-18.h | #14 actual retained-share recovery, #27 admission, #12 relevant verification evidence, #16 scoped recovery/fairness metrics |
| I9-AC-21.n | POST-CLOSURE MVP: #9 real formal consumer; #14/#15/#24/#27 actual readiness composers; #16 representative positive/removal matrix |
| I9-AC-1.b, I9-AC-1.c, I9-AC-8.k, I9-AC-18.i | #9 frozen roster/schedule/rerun consumer; #5/#6/#7 actual qualified native profiles; #27 permits; #16 observed qualification and rerun metrics |
| I9-AC-21.k | #8/#19/#43 real legacy Workflow upgrade/ownership consumer; #9 unavailable formal-gate regression; #12 actual impact evidence; #16 withdrawal/Task availability measurements |
| I9-AC-18.c, I9-AC-14.j, I9-AC-14.h, I9-AC-21.l, I9-AC-13 | #5/#6/#7 actual requested/effective configuration; #60 actual between-round/source-capture effect ownership; #14/#27 retained-share recovery/admission; #15/#24 actual attention/termination ingress; #43 record-only boundary; #16 profile-availability and quota impact |
| I9-AC-9.j | #10 recognized Human effects are not Broker/native authority; #15/#24 genuine origin negative controls; #23 canonical ingress |
| I9-AC-18.j | #9 prior-target verification consumer; #12 searched evidence; #19/#60 actual attributed owned observations; claims cannot clear blockers |
| I9-AC-8.m | #5/#6/#7 real native property qualification; #9 exact current scope/key/config instantiation; #16 representative user-scope proof |
| I9-AC-21.o | #9 exact checkbox/mode proof; #5/#6/#7 actual channel profile; #16 native enforcement and representative dogfood |



**12.k — Actual deterministic runtime authorship.** A scoped trusted runtime effect
may record a non-native RuntimeAuthor only from an actual #60 owned producer binding exact
operation/config/executable effect inventory, before/after trees and current source,
with a reviewed method demonstrating the declared deterministic non-LLM modification.
Preserved hook/formatter/test/codegen execution is not sufficient by name, exit status
or selected-group death. Hooks that can invoke a native/LLM writer retain its recorded
native family; unknown origin remains unknown and refuses/conservatively excludes.
Missing actual producer cannot earn RuntimeAuthor or restore roster eligibility.
Issue 9 closing evidence is the actual typed consumer/checker contract plus missing/
foreign/stale/unknown/LLM-origin refusal or conservative exclusion, not a fabricated
positive RuntimeAuthor. Public #60/#12 MUST own I9-AC-12.k actual deterministic formatter
producer/attribution positive under preserved defaults and actual native-source effect
settlement; that positive remains a separate MVP gate, not an undeclared mandatory #60
merge for this checker. A controlled producer DTO/mechanical fixture does not grant
production RuntimeAuthor. Until genuine #60 proof composes, RuntimeAuthor is unavailable
and cannot restore eligibility. No source-execution or effect-ownership exemption is
introduced; affected production capture still obeys its actual #19/#60 integration gates.

**14.h — Queue lock and bounded exit.** Acquire the real round lock/delegation at the
first member's actual admission after permits and exact fresh source/context checks.
An entirely queued roster owns no round/worktree lock or native context slot; its
prepared frozen bundle may become stale and must be revalidated before admission.
Once any member starts, retain the round lock through all slots actually settled or
queued slots safely cancelled; new source drift holds, never permits peer exposure.
An entirely queued bundle is a frozen proposal, not an admitted round. Only the first
actual member admission atomically consumes the shared 64-round count and applicable
autonomous retry/confirmation allowance, with current quota/source/context checks.
If the proposal goes stale or expires before that, discard it without those count
charges, preserve exact immutable target/policy/reason provenance and charge ALL
actually retained proposal identity/hash/revision/policy metadata bytes to lineage128-MiB
quota AND a separate cumulative4-MiB never-admitted-proposal metadata allowance. Each
proposal record is at most8192 bytes, at most512 such records per lineage. Never
persist prepared member model frames/copies for an unadmitted proposal: reference
existing immutable source/context with exact hashes; any volatile preparation is
bounded and discarded if stale. Materialize/persist complete bounded frames only at
first actual admission after revalidation and full reservation. Proposal limit reaches
NeedsProposalCapacity with provenance/attention, NOT exhausted review-round/128-MiB
lineage status; no counter reset or unsafe ownership release. When quota was otherwise
available,512 stale proposals cannot burn more than4 MiB without admitting a review. No
in-place target rebind or silent history pruning; a new proposal has a new identity.
Pending proposals hold no native/context/round lock or review-round-count reservation.
Actual source-capture effects still need60 ownership; proposal allowance/storage capacity refuses new proposals. Actual new
full-round admission still obeys remaining 128-MiB quota, never ignores retained bytes. A frozen optional queue-expiry policy is Disabled by default or1–3600seconds per slot
measured from its enqueue;0/3601 reject. Expiry safely cancels an unstarted queued slot,
terminates an already admitted round without certificate and retains its consumed count;
if entirely queued it only closes the unadmitted proposal as specified above,
and starts actual owned cancellation for any started members. It releases no Lost/unknown
owner or unused reservation before safe whole-round closure. Attention default 600 is
independent notification, not expiry or mandatory Human rescheduling. Status distinguishes
ordinary queue from queue behind retained uncertain capacity. A fully queued proposal
blocked by that retained uncertain share parks under I9-AC-18.h without admitting a
round; an already partially admitted round takes its automatic safe noncertifying
closure. Both rules apply even with optional
queue expiry Disabled; #14/#27 recovery/admission
and #15/#24 actual trusted handlers remain pending where required. Tests cover entirely
queued/no lock, one started+queued/lock retained, expiry safe closure versus Lost held,
permits returned after attention and target changes while ALL slots queue: no
round/retry allowance spent, actual retained bytes remain charged, new identity and
no silent target rewrite. Test512 target changes: no copied prepared frame history,
actual proposal metadata≤4 MiB and preserved remaining review capacity; next proposal
holds NeedsProposalCapacity without claiming budget-lineage exhaustion. #16 reports
queue staleness/proposal-capacity attention separately from admitted quota exhaustion.

**20.d — Actual remediation handoff.** Two rounds around actual executor fix/commit
must release only a completely settled round's lock/member delegation, return control
through actual Workflow remediation and bind the next roster to its single new published
milestone context. Neither generic Set JSON nor per-member context increments may create
that handoff. Original findings/budgets survive; unsettled/Lost still blocks.

**16.d — Reviewer permission-request behavior.** Formal read-only native permission
requests for prohibited actions receive deterministic DENY with bounded exact scoped
request/diagnostic and visible reason/status.9 never mints a10 Broker ALLOW or free-text
Human command authority; a request requiring unavailable product attention is explicitly
nonactionable, not only a generic timeout. Actual provider owned historical observation/
DENY contract and settlement remain mandatory; no label-based privilege or Lost release.
Test reachable request→zero ALLOW/action wire and visible typed denial/hold through
the actual PRODUCTION adapter with a controlled native peer, including existing
provider-specific lifecycle write failures. This closes construction/path coverage,
not real-native enforcement;16.b/16.c and21.o explicitly retain that mandatory16 gate.

**14.i — Fixed shared reservation arithmetic.** Before each ACTUAL admitted round reserve full 8 MiB
verification plus 4 MiB control allowances, including its 64-KiB attention margin, alongside
actual core/member frames, each maximum 1-MiB result,64-KiB diagnostic, declared retained
expansion and actual reviewed producer row overhead. With Triple, three 1-MiB frames,
three 1-MiB results and default three 1-MiB retained expansions contribute9 MiB plus192 KiB
diagnostics; shared allowances add12 MiB. Any separately retained core/copies/refs plus
actual authority/usage/audit rows add their real charges: the illustrative21 MiB+192 KiB
is NOT a universal complete-reservation total. Representation dedup counts only actual
shared identity once. Simultaneous ceilings do not guarantee64 rounds fit128 MiB.
Unused reservation is released only by safe round closure; retained history persists.
Test this formula with actual producer rows and separate/shared core representations,
exact remaining-quota boundary and refusal without input; external refs never erase bytes.

**9.h — Bounded required disposition assignment.** Before later-round input freeze
an exact per-slot required-disposition manifest for EVERY still-unresolved potential
or verified finding in the effective blocking set. Assign the eligible original
finder OR the already policy-designated independent confirmer pair; identities obey
I9-AC-9.e/I9-AC-9.g and the pre-observation designation rule. If no eligible frozen designation
exists,I9-AC-9.f Human-only resolution/hold applies; never choose an opinion-shopping pair.
A trusted Human-resolved finding has
no required new clearance output. All members still receive/inspect all unresolved
claims under I9-AC-14.e and may report contrary evidence; unassigned opinions do not silently
clear a blocker. No automatic favourable confirmer selection after results.
Each result may carry at most256 disposition entries, each at most8192 complete encoded
UTF-8 bytes (identity, outcome, inspected locations/evidence included), separately from
its256 NEW finding count; the combined entire result remains1 MiB. Result mandatory
metadata has a frozen actual DTO upper bound at most64 KiB. Pre-input check each slot's
required count<=256 AND count*8192+its metadata upper bound+65536<=1 MiB with checked arithmetic;
65536 bytes reserve at least8 NEW full-size8192-byte findings separately from mandatory
dispositions. More may fit only within the unchanged complete1-MiB envelope; result
overflow is explicit bounded nonapproval, never instruction to omit a new finding.
Otherwise NeedsContext/trusted existing resolution or termination without certificate,
never send an impossible output obligation or omit a required finding. Maximum count
never promises maximum bytes fit. Missing assigned dispositions holds; nonassigned
Low/nonblocking disposition is optional within the same cap, not an approval hint.
Test exact byte/count boundary,256 count simultaneous refusal when worst-case bytes
cannot fit; at 64-KiB mandatory metadata112 dispositions+8 new findings fits,113
dispositions refuses before input. A new blocker in the reserved headroom is retained
and vetoes certification; overflow is nonapproval. Test accumulated claims across rounds, frozen eligibility and missing assigned
output→hold. Required output reservation uses these actual producer DTO bounds.

**2.b — Quorum with retained uncertainty.** Quorum2/3 or any with enough settled
APPROVEs plus one Lost/uncertain member has NO certificate and remains recovery-pending,
even with early-stop. Quorum tolerates safely settled failures, never outstanding native
effects/target-mutation taint. #16 publicly owns this availability frequency/Task-time;
only actual Issue 14 recovery can release that owner, never vote count or trusted Human opinion.

**9.j — Recognized Human actions cannot be delegated to Broker.** Runtime-controlled
Human dismissal/downgrade/authorship disposition, policy relaxation/activation,
confirmer/roster redesignation, supersession/decomposition and termination actions
require the genuine canonical #23 Human ingress. Native/Workflow/Broker API or IPC
cannot acquire that provenance through an ALLOW, including a recognized command
exposing one of those enrolled handlers. #10 must classify these recognized effects
as Human-only, never deterministic-auto or cross-agent ALLOW;15/24 handlers refuse
controller/native/Broker-origin authority with retained evidence. Test actual
recognized executor→Broker request→Human-only refusal and attempted handler JSON/API
origin forgery→no blocker clearance/certificate. Public10/15/24 acceptance quotes
this stable criterion under21. This is an application-channel control, not process
ancestry authentication: arbitrary out-of-band same-UID shell/binary/DB access remains
the approved trust limit. An unimplemented CLI/Broker chain is not a reproduced
production exploit. Certificate/status discloses known concurrent supervised work
when Human events occurred; concurrency is neither origin proof nor automatic refusal.

**9.i — Action authority and application origin.** These authorities remain separate:

| Action | Required actual authority |
| --- | --- |
| Human fallback dismissal/downgrade/not-applicable/duplicate adjudication; policy relaxation; post-observation roster/confirmers change or supersession; Human authorship/unknown-origin disposition; decomposition | Actual trusted Human ingress with scoped principal/event/reason/evidence; automated controller cannot mint/borrow it |
| Original eligible finder or frozen independent confirmer verification | Actual fresh owned verifier input/result/settlement provenance under I9-AC-9.a, I9-AC-9.e, I9-AC-9.g, I9-AC-9.h; never a controller-generated finding |
| Initial INSTANCE of an EXACT already Human-approved policy digest; frozen conditional branch, scheduling, verified bounded retry/fix/re-review progression | Separate automated controller under that exact activated policy, within immutable floor/lineage/designation; never post-opinion relaxation, roster/confirmer swap, budget reset or approval carry; bounded18.a/18.i reruns retain their disclosed residual |
| Approve any NEW policy/rubric CONTENT digest, including rules introduced by base-sync | Actual trusted Human ingress approving that exact digest; controller base-sync/source capture cannot mint it |
| Genuine safe cancellation/termination without certificate allowed by frozen policy | Actual owned controller/supervisor cancellation and required settlement/closure; never Human clearance, approval or Lost release |

The existing Human-ingress library composition tests its application boundary, not
biological identity. A controller observing dissent cannot relabel itself Human,
drop the dissenting roster, change all→quorum or redesignate favourable confirmers.
Trusted Human action after dissent remains explicit history; frozen-policy automation
still verifies/fixes/reviews autonomously within bounds. Actual 15/24 public handoffs
quote this split, including controller-origin negative tests through the actual port.
Actual trusted ingress provenance, rather than JSON actor fields, enforces
it when implemented; its concrete types/constructors belong to design. Same-UID arbitrary linked code/OS/DB remains the stated trust limit.

**18.h — Individual settled resource release.** Each member releases ONLY its own
actual provider/Project/global native permit after its genuine owner's COMPLETE
required settlement and exact member closure. Native outcome/result/group label alone
cannot release retained server/tool/unknown ownership. Keep round worktree lock/member
input exclusion, finding/evidence obligations and byte reservation until whole-round
safe closure. Lost/uncertain shares remain held for #14. Test two healthy Sets each
with two slots competing for ONE provider permit: settled slot releases its permit,
queued slot progresses, no healthy hold-and-wait or context bump. Real #9/#19/#43 member
closure producer remains an integration gate. Queue behind unrelated retained Lost capacity has a controller liveness exit. At
every relevant global/Project/provider scope, non-Lost-reachable capacity equals the
configured limit minus genuinely retained Lost/unknown shares at that scope. Healthy
busy ownership does not reduce this reachable capacity: capacity2 with one Lost and
one healthy-busy share leaves one reachable share, so a one-permit slot waits normally.
Only a queued slot (or its qualified simultaneous-start cohort, I9-AC-8.k) needing
more than that reachable capacity is blocked by retained Lost. Before first round
admission, every frozen roster slot and its qualified schedule must be non-Lost-
reachable; otherwise park the wholly queued proposal as nonactionable recovery-pending,
without admitting a round or charging round/retry allowances. Keep ONE proposal
identity per unchanged cause, original frozen target/policy/hash references, charged
bounded metadata and unrelated ownership holds; repeated unchanged wakes cannot
create new proposals or burn the512-record allowance. Genuine #14 capacity restoration
may automatically revalidate all original source/claim/context/lock/policy/quota and
qualified-schedule predicates, then admit that SAME still-current proposal. Parking
adds no retry cause/spend because no round was admitted; an existing retry proposal's
original allowance is still consumed at first admission as14.h requires. A stale
proposal or frozen explicit expiry closes under14.h; new target/policy requires a
new identity and charged record, never in-place rebinding. Test repeated Lost/healthy
capacity cycles with one unchanged parked record, no admission while held, genuine
recovery revalidation/admission and stale-target refusal. This avoids burning a
healthy lineage's budget against a known foreign Lost.
A partially admitted round stopped by an unrelated retained Lost/unknown capacity
cause has no autonomous cause(c) retry. After genuine #14 restores that capacity, its
new full round still requires existing trusted Human/frozen authorized policy action
under9.i; mere capacity return or unchanged wake cannot invent it. Until that product
port exists, status is Human/recovery-pending with original charged history. #16
reports this specific interruption/Task-time separately, never hides it as healthy
contention or releases the foreign owner.

If reachable capacity becomes insufficient AFTER first input, safely cancel never-
started slots and settle/cancel genuine started owners, then close without certificate.
Consumed round/history charges remain; an uncertain own owner keeps its lock for #14.
Controller safe termination needs no new Human decision, and never releases foreign
Lost shares. Repeated unchanged capacity causes park without new admission or round
charge. Capacity restoration alone is not retry authority: a later full-roster round
still requires an existing finite frozen-policy retry authorization or trusted Human
action under9.i/18.a; it cannot bypass dissent, allowances or lineage budgets. Test
mixed occupancy, wholly queued no-spend, partial charged closure, and repeated foreign
Lost with no silent64-round exhaustion. Actual #14/#27 production admission/recovery
and #16 scoped wait/availability measurements remain mandatory.

**21.j — Interim formal refusal and real fixed conformance ingress.** Until actual
I9-AC-21.i readiness composes, production Workflow formal-review consumers return
ReviewGatingUnavailable before claim/reservation/native input, with no legacy weaker
single-Reviewer fallback. Separately, a sealed fixed conformance ingress may exercise
NONCERTIFYING ReviewSet delegation through the SAME actual production Store/Workflow,
native adapter/preparation/ownership and member-closure ports. This ingress ITSELF
requires the actual controller-owned dedicated conformance-state/fixture provenance
of 21.m, before claim/reservation/native setup/input: owned freshly created fixture
repositories and state, genuine creation/root/common-Git identity and nonoverlap
with registered user Projects. A public test-mode flag/path, existing user registry
or native/API claim cannot mint that provenance. Direct calls on real-state/user-
Project targets refuse before effects; tests reach the actual port and prove zero
claim/process/model bytes. Pending/unknown physical scope still refuses, preserving
actual native defaults/ownership. It binds an immutable
nongating basis and cannot mint a formal certificate, satisfy a phase gate or allow
PR/merge/finalization. It is not a public user/native JSON nongating flag or permission
bypass. Actual provider-neutral #5/#6/#7 workload ownership and full owned settlement,
supported profile/default settings, current source/lifecycle/frame/locks and input eligibility remain mandatory BEFORE its effects.
Unproved producers/profiles still refuse; conformance is not ReviewGatingReady.

I9-AC-1/I9-AC-1.a/I9-AC-16/I9-AC-18.a/I9-AC-18.h/I9-AC-20.d positive native orchestration and two-round real Workflow
remediation close through this actual fixed conformance ingress. I9-AC-13/I9-AC-21.i/I9-AC-21.j
close their interim boundary through the real formal consumer's refusing path;
production formal certificate positive and representative native #16 remain mandatory
MVP gates. Nongating outcomes, old single-reviewer passes and JSON basis cannot satisfy
the formal consumer. Tests cover actual positive fixed ingress and unreachable public
relabel, plus RequirementsReview/DesignReview/ImplementationReview/SecurityReview under
QUICK/STANDARD/STRICT initial/retry/resume_gate/request_finalization where applicable:
unavailable formal paths have zero marker/claim/model bytes. New QUICK cannot skip
formal review to PR. Component synthetic tests prove mechanics only; they cannot
replace the actual production-port native conformance positive. README/status and
master explicitly distinguish fixed noncertifying conformance from unavailable formal
gating. This staging resolves the actual-consumer closure contradiction without
claiming the later product handlers/recovery are already present.

**21.k — Legacy single-reviewer gate withdrawal.** The implementing PR must inventory
actual merged #8 single-reviewer writers and downstream consumers under genuine #12
impact evidence. At deliberate upgrade, pre-epoch live/evaluating native reservations
remain held/drained under #19/#14, never retagged as new settled owners. Historical
legacy Passed review records stay factual read-only history, but cannot satisfy new
formal review gates or downstream PR/MergeGate/finalization. No automatic grandfathered
certificate, context clone or native repeat is allowed. Tests cover live Evaluating
upgrade refusal with unchanged original bytes/ownership, legacy Passed→PR/MergeGate
refusal, and already external-effect history retained for actual reconciliation.
No automatic rollback/republication of an already-created PR/merge is inferred.
The master Workflow examples/status must state this proposed withdrawal and its
availability impact honestly; currently merged component behavior remains historical
until the actual co-integrated change is implemented and deployed.

Stable closure quotes use the explicit namespace I9-AC-<key>, for example
I9-AC-16.a (this document's acceptance) versus GitHub Issue #16 (its public owner).
Existing short keys remain permanent aliases to preserve historical evidence; every new
public handoff uses the prefixed form. No renumbering of prior evidence is implied.

Every criterion and bold subcriterion ID above is a stable closure-evidence key.
Absent suffixes are intentional reserved IDs; reordered subcriteria retain their
stable IDs rather than renumber historical evidence.

Fixture mechanism, private port layout, hashes/transaction algorithms and controlled
transport barriers belong in Issue 9 design; linked evidence must demonstrate these
observable acceptance outcomes through actual production consumers.


**14.j — Between-round verification accounting.** Retained remediation/check evidence
produced after a settled round and before the next admission charges the same lineage
128-MiB retained quota directly, with at most8 MiB complete encoded evidence per
between-round remediation step. Reserve its worst-case bytes BEFORE owned verification
effects; no admitted-round allowance or nonexistent reservation is borrowed. The next
round's actual delivery charges its frozen delivery budget again and retained blobs
deduplicate only by proven exact identity. No evidence is silently discarded at quota
exhaustion. Exact8-MiB/128-MiB and +1 refusal with prior history intact are required.
Actual effect ownership/producer availability still obeys #60, not a byte-quota permit.

**21.l — Required profile availability decision.** Required 4+/repeated-family and
multi-author STRICT positives remain actual Issue #9 closure gates, not refusal-only
substitutes. After a recorded bounded supported-profile investigation (at most three
candidate registered profiles per required configuration), emit explicit owner/evidence/
missing-capability attention and a tracked scope/producers proposal. The controller
must publish/link a named public feasibility/scope-decision tracking Issue with an
actual owner, evidence links and due condition after the bounded investigation leaves every
candidate infeasible/unknown. Public Issue9 displays its required4+/repeated-family
item as distinctly BLOCKED, with that link, before source acceptance/closure; generic
progress cannot obscure it. The trusted Human scope decision remains separately
reviewed and explicit; this document does not pre-authorize
one or close the required positive automatically. Preserve all holds/floors/defaults.
Document drafting remains cumulative author contribution because its instructions and
artifacts influence the implementation/security target even outside a narrow diff;
no phase label launders it. Actual proven unrelated contribution uses I9-AC-11.d's
trusted applicability rule, not a blanket drafter exclusion waiver. #16 measures the
stated availability cost; a pending provider proof cannot be called completed native support.


Informative native-channel documentation basis retrieved 2026-10-05 (not an installed
profile certificate): [Claude memory](https://code.claude.com/docs/en/memory) describes
“Per repository, shared across worktrees”; [OpenAI configuration reference](https://developers.openai.com/ja-JP/docs/config-file/config-reference)
describes `memories.use_memories` default true and `min_rollout_idle_hours` default 6.
The [OpenAI feature reference](https://developers.openai.com/zh-Hans/docs/config-file/config-basic)
separately lists the memories feature default false; a field default alone never
proves installed feature activation/injection. Observed CLI versions are Claude 2.1.283
and Codex 0.160.0 from --version only. No private user memory/config/history was read.
Channel classification/readiness relies on exact registered source/config and actual
owned profile conformance, preserving defaults; mutable vendor docs are informative
capability context, never self-updating normative authority or proof of peer injection.


**8.j — Default-preserving profile controls and exposure scope.** For the avoidance
of doubt, invocation-scoped disabling or overriding native memory/history/loading/
generation defaults is FORBIDDEN just as mutating/deleting user memory/config/hooks is.
Recorded model/effort and actual read-only permission construction are the existing
supported policy choices, not permission to disable memory or hide native rules.
Profile controls mean genuinely proved default-preserving source/scope/timing and
owned conformance, never a silent `--bare` or memory-disable workaround. Required
serialized repeated-family positives remain inside acceptance; currently unproved
profiles are unready, not removed from the required roster or declared impossible
from a shared-family label alone. Actual supported profile evidence may qualify;
absence follows I9-AC-21.l attention without automatic closure/scope waiver.

The certificate claims runtime-controlled initial/expanded input independence and
its exact proved CURRENT-round peer-channel basis, not physical erasure of all native
history. rrx NEVER delivers raw executor conversation, prior verdict/tally vote hints,
Broker/consultant chat or current-peer findings outside the authorized frozen factual
bundle. Current-peer capable/unknown channels remain whole-roster unavailable before
input. Known non-current-peer provider-memory exposure (including Broker/consultant
or prior-round-derived history) is classified and disclosed as a residual/confound;
documentation of a channel alone is not observation that a particular claim injected.
This residual can coexist with F ONLY under the stated runtime-controlled-input and
actual current-peer-free profile basis; it supplies no extra independence guarantee.
If unapproved executor/Broker/consultant chat, vote hints or peer output are actually
observed in a slot's native input, I9-AC-8.i invalidates the WHOLE round regardless of
origin/family. A frozen stricter Project policy may reject known prior exposure entirely.

The WHY is the Goal's preserved native defaults and honest runtime-input trust boundary:
we can forbid our own delivery and require actual current-peer profile proof without
pretending fresh Session physically clears native memory. This is no family floor,
OS/same-UID secrecy claim or permission to label unknown current-peer channels safe.
Test Broker/consultant of an eligible slot's family: documented-capable non-peer
exposure is disclosed with that exact limited F basis, actual unapproved-chat injection
invalidates, and unknown/capable CURRENT-peer exposure refuses before any slot input.
Actual provider source/config/channel conformance remains required, not fixture labels.

**21.m — Acceptance isolation and checkbox evidence.** ALL conformance, failure,
mutation and acceptance runs use dedicated runtime state and isolated owned fixture
repositories/Projects that do NOT physically overlap real registered user Projects or
common-Git roots. Actual production code/ports are exercised against those fixtures;
this is not a private DB-seeded authority or substitute native producer. Preserve native
auth/hooks/rules/defaults and scope actual effects/cleanup. Status reports exact held
scope, including session-less/root/common-Git uncertainty. An actual Lost created in a
user's real state remains unreleasable before genuine #14 recovery; acceptance never
uses that state to force a Lost. This follows the Goal's synthetic-fixture rule and
does not discard unknown fixture workload/process bookkeeping at test exit.
Native history/memory can have provider-user-global retention under preserved
defaults; isolated rrx DB/repositories do not prove native-store isolation. Inventory
actual source/config/channel write/generation timing and each fixture's potential
contribution without reading private user stores or assuming a conditional feature
is activated. Real-native conformance uses ONLY benign synthetic PUBLIC material
safe to persist; adversarial executor-chat/forged-origin protocol negatives use
controlled transport peers and are never injected as harmful native instructions.
Do not disable/delete defaults/history or claim observed pollution from capability
documentation. Actual native channel readiness still follows8.c/8.j/8.l, and #16
reports this potential retention/residual independently from observed injection.


**21.o — Public checkbox evidence mapping.** The eight original checkbox mappings
are:1→I9-AC-1+1.a;2→2+2.a+2.b;3→3;4→4+4.a;5→5;6→6;7→7+7.a;8→8+8.g+10.d.
All other stable criteria still independently block WHOLE Issue9 closure. In particular,
M=1/4+ Scope positives1.c/1.b are separate from the two-reviewer checkbox.
Public checkboxes1–8 close individually on those exact actual component
consumer/production-port conformance evidence AND required registered native-profile
proof, with separate annotations: `orchestration_transport=synthetic|real` and
`profile_qualification=real-native(<exact evidence reference>)|pending` and
`read_only_basis=contract+construction; native-enforcement-pending16` until the
actual #16 proof exists. ALL eight checkbox closures require their respective native orchestration/configured
profile basis to have actual qualification; protocol negatives may use controlled
peers but cannot qualify a fabricated profile. Only actual
qualified native-profile evidence permits a check mark. `profile_qualification=pending` is ONLY an
unticked status value and can NEVER accompany a ticked item. This prohibition does
not apply to the separately mandated native-enforcement-pending16 annotation. Synthetic transport mechanics never imply synthetic profile qualification.
No checkbox is ticked solely by a proposed test, synthetic profile declaration or
unready port. The annotation explicitly names representative real-model/formal
acceptance as mandatory #16/MVP PENDING when not yet proved; component check marks do
not claim that completion. Actual native profile/channel qualification required here
is not replaced by synthetic transport. Issue #9 closes only when all its required
I9-AC predicates/positive configurations genuinely pass; #16 evidence remains a separate
whole-MVP gate. The public Issue must retain these exact evidence annotations when
checks are eventually ticked, rather than letting an unqualified 'works' imply native
formal certification. Original scope and checks stay unchanged and currently unchecked.


**1.b — Four-plus independent closing obligation.** First-class four-plus rosters
retain the actual Store/Workflow delegation, slot-count and independence controls of1
and18.f/20.c, including the required real registered repeated-family profile proof.
Public checkbox1 closes on the exact two-reviewer IDs in I9-AC-21.o; Issue9 closure still
requires this separate four-plus positive, with no unsupported-profile waiver.

**8.k — Frozen qualified admission schedule.** The actual registered native channel
qualification binds the frozen roster, source/key scope, load/write timing basis and
a finite permitted admission envelope (including queue delay, start skew and allowed
parallelism/permit schedules). It must cover every reachable schedule under the frozen
queue/expiry policy, or constrain actual admission to its proven envelope. Best-effort
resource concurrency never weakens this independent-input condition. A concurrent-
only profile requires its complete qualified cohort capacity before ANY member model
input; it cannot silently become serialized when permits change.

Every queued member's actual pre-input admission revalidates the elapsed schedule,
current permits and exact original channel qualification. Outside the qualified
envelope, no further member input is delivered and the WHOLE round is non-certifying;
already admitted owners settle normally with retained evidence. A new full-roster
round needs existing genuine retry authority, never a refreshed basis assertion. An
unbounded queue policy cannot qualify a profile with only a finite start window. Test a finite-window SERIALIZED-CAPABLE profile whose first owner starts, then
healthy permit contention delays the queued second member beyond its qualified
window: second input zero, zero certificate, explicit schedule refusal and genuine
first-owner settlement retained. Separately, concurrent-only cohort capacity missing
at initial admission has zero member input. Acquire the COMPLETE qualified cohort's
required global/Project/provider permits all-or-nothing: a waiting cohort retains
ZERO partial member permits. A wholly unadmitted queued Set holds no round worktree
lock; an already admitted round retains its existing lock/evidence obligations.
Test two2-member cohorts competing for capacity3: exactly one admits2, the other
queues with0, then progresses after genuine first-cohort settlement. Verify that incremental partial holding cannot satisfy this acceptance;
the concrete mutation belongs to design/source verification. Actual failure during qualified cohort
startup makes the whole round non-certifying rather than serialized. Cohort capacity
reservation is not a guarantee that every native startup physically succeeds. A mutant omitting this real queued-member check must fail.
Actual default-preserving native producer evidence remains required, not a synthetic
clock or channel label positive.

**21.n — Post-closure production readiness matrix.** Actual #14 recovery, #15/#24
trusted product ingress/policy and #27 fair admission composers, with #9's real formal
consumer, own the valid-formal-basis/readiness-positive matrix. After genuine producers
exist, independently remove each required readiness predicate and prove the real
consumer refuses a genuinely valid formal basis before effects; restored readiness
and current basis permit the actual positive. A previously genuinely minted certificate
may be retained while a readiness capability becomes unavailable, but fixture overrides,
conformance certificates and fabricated ready flags cannot substitute. This matrix is
mandatory MVP/#16 integration and explicitly pending until actual production composition;
it is not an extra #9 merge/closure prerequisite.21.i closes only its stated reachable
interim refusing path, never claims this positive completed.

**18.j — Verification precedes remediation.** Each autonomous concern-linked fix
candidate references a recorded inspection/verification outcome at its exact prior
target BEFORE the fix commit: attributed verified-defect report, false-positive
claim or Human-judgment request, inspected locations and exact evidence references,
with class-wide N inspected/M changed where applicable. These are labelled actor
reports, not deterministic semantic truth or blocker clearance. The existing
nonempty concern-linked committed delta and later independent disposition remain
mandatory. Missing, wrong-target or post-commit verification refuses autonomous
changed-target admission. Test actual verification→fix→commit→re-review and missing/
late verification refusals; no supervisor model or deterministic proof for every
semantic fix is required. #9 owns this consumer/evidence check; public #12 owns
its searched scope/impact evidence and #19/#60 the actual attributed owned execution
observations where native effects are claimed, with concrete handoffs under21.
Those producer requirements do not create a 9→12 implementation merge cycle.

**18.i — Nonblocking rerun limitation.** Concern-linked nonempty committed bytes plus
a truthful unverified fix claim authorize bounded independent reruns under18.a, not
semantic correctness of the fix. Semantically neutral edits can therefore lead to
new opinions on prior nonblocking dissent without a mandatory9.h blocker disposition.
This is a disclosed limitation, never clearance of an unresolved blocker. #16 reports
approvals after k reruns following nonblocking dissent, linked delta/fix claims and
actual charges. A frozen stricter policy may require the original dissenting slot's
disposition; the Core does not impose a new per-fix Human gate or deterministic
semantic proof producer. Existing64-round/shared allowances and anti-cosmetic controls
remain in force.


**1.c — Explicit one-reviewer roster.** M=1 non-author actual registered qualified
Reviewer accepts QUICK/STANDARD through the genuine fixed conformance consumer;
formal eligibility requires mode and independent floorF=1. STRICT M=1 refuses
before input becauseF=2, even with any/quorum1. Use21.m's separate transport/profile
evidence annotations. M=2 with only one eligible member is not this positive.

**8.m — Qualification template and actual-scope instantiation.** Qualification
evidence binds an exact installed provider version, effective supported configuration,
complete reachable channel inventory and concrete key-derivation/load-write predicates,
proved by actual owned native fixture conformance. It is not permanently keyed to
the fixture's literal repository ID, nor a blanket family/default assertion. Each
actual round must instantiate those SAME proved predicates against its exact real
Project/authorized reviewer workspace/source and native key scopes, frozen roster/
schedule, current installed version/configuration and governing source. Unsupported
key derivation, stale/changed configuration or unproved channel preconditions refuse
BEFORE input. Fixture keys do not become user keys; an exact source-pin change never
auto-refreshes qualification. Actual current-peer exclusion must follow the proved
channel properties, not a string substitution or private user-memory inspection.
Test one genuinely qualified property template on a second disjoint owned fixture
scope, then different/unproved key/config/channel/schedule refusal. User Project
instantiation uses those same private predicates; it is not an adversarial conformance
run on a user Project. Missing actual native template/instantiation producer remains
unready; #5/#6/#7 own its actual supported channel evidence and #9 its consumer.

**8.l — Native channel feasibility and exposure reporting.** Before the formal
design gate, record each required candidate family's feasibility under PRESERVED
native defaults: feasible with exact actual source/control/conformance evidence,
infeasible-under8.j with concrete incompatible channel property, or unknown with
named missing proof/owner. Document which actual load/write key/timing properties
would suffice: no reachable current-peer writer in the qualified profile; genuinely
disjoint memory keys under the exact authorized workspace/source authority; or a
proved peer-free initial snapshot with ALL relevant loads completed before ANY
current-round write under the frozen finite schedule. None is assumed from family,
SID/read-only label, docs-only capability or feature field defaults. Unbounded writes,
unknown load timing or an unenforceable schedule do not qualify. Conditional feature
and injection defaults are distinct; neither alone proves installed activation.

Unknown/infeasible status surfaces21.l attention EARLY, before implementation can
claim a profile positive, with bounded candidate investigation and concrete owner/
proof. Design may specify honest refusing consumers while feasibility is unknown;
Issue9 closure and its required repeated-family positives still cannot pass without
the actual qualified producer. This is no scope waiver or endless automatic candidate
loop, and no invocation-scoped memory/default change. Actual registered conformance
uses21.j's genuine isolated ingress and producer ownership; vendor docs alone cannot
make a feasible certificate. Preserve all historical observed review evidence.

Per-member certificate/status exposure metadata separately flags any documented-
capable EXECUTOR/BROKER/CONSULTANT chat channel versus prior-round reviewer history.
Under Core default a disclosed unobserved NON-current-peer channel can coexist with
the limited independence basis of8.j; a frozen stricter policy may refuse it. Actual
observed unapproved chat still invalidates the whole round under8.i, and unknown or
capable CURRENT-peer channels always refuse under8.c/8.k. Test these classes and
exposure-specific policy refusal; lower observability never implies broader secrecy.
#16 records actual observations independently from capability/disclosure flags.


## Requirements review37 dispositions

Informative review provenance only: this section adds, removes or amends no
requirement. The cited18.a/18.i/11.c/18.h criteria govern.

Native review58639061-4cd8-41f2-af8d-e1ef4458cd4e at10e53704 found no Critical/High
and three Medium observations. M1 is accepted as wording precision:18.a no longer
claims to eliminate every form of opinion resampling; existing18.i explicitly retains
bounded concern-linked reruns after nonblocking dissent. No new per-fix Human gate,
semantic fix oracle or retry authority is added. M2 is clarified under11.c: recorded
uncertified contributions from earlier Workflow epochs remain factual negative lineage
evidence; legacy Passed/Completed never confers a current certificate. Applicable or
unknown histories keep existing author/veto/budget inheritance, with12's actual producer
still pending. No scope or epoch label washes that history.

M3 proposes a NEW automatic recovery retry after a partially admitted round is closed
by foreign Lost capacity. It does not establish a contradiction in18.h: the existing
contract deliberately distinguishes an unchanged wholly-queued proposal (no admitted
round) from a partly admitted/charged round. The latter remains Human/frozen-authorized-
policy pending after genuine14 recovery; unchanged wakes/capacity restoration cannot
mint cause(c) retry. Retain that reviewed availability limit and16 measurements rather
than adopting unrequested recovery authority. All actual Lost/resource holds persist.
Low observations do not change native isolation, quota ceilings or registered profile
readiness: runtime-controlled input exclusions, measured residual exposure and finite
actual admitted bytes retain their existing explicit limits. No native acceptance,
ReviewGatingReady producer or source implementation is claimed by this clarification.

## Ordered stable acceptance index

This complete index aids navigation; the named criterion defines its actual closing
proof and public producer handoffs. PRE means its #9 consumer/contract/handoff proof
is required before closure, not that all named later MVP producers are implemented.
Only21.n is explicitly POST-closure MVP. Numeric1–21 remain the primary public
acceptance above; body definitions cannot add an unnamed closing obligation. This navigation table
intentionally has NO separate owner column. Closure compares its stable keys and
stages with EVERY criterion body and reconciles every missing/extra key; producer
owners/quotes come from those bodies and the canonical handoff table.

| Stable key | Stage |
| --- | --- |
| I9-AC-1.a | PRE #9; later MVP only as named in criterion |
| I9-AC-1.b | PRE #9; later MVP only as named in criterion |
| I9-AC-1.c | PRE #9; later MVP only as named in criterion |
| I9-AC-2.a | PRE #9; later MVP only as named in criterion |
| I9-AC-2.b | PRE #9; later MVP only as named in criterion |
| I9-AC-3.a | PRE #9; later MVP only as named in criterion |
| I9-AC-3.b | PRE #9; later MVP only as named in criterion |
| I9-AC-3.c | PRE #9; later MVP only as named in criterion |
| I9-AC-4.a | PRE #9; later MVP only as named in criterion |
| I9-AC-7.a | PRE #9; later MVP only as named in criterion |
| I9-AC-8.a | PRE #9; later MVP only as named in criterion |
| I9-AC-8.b | PRE #9; later MVP only as named in criterion |
| I9-AC-8.c | PRE #9; later MVP only as named in criterion |
| I9-AC-8.d | PRE #9; later MVP only as named in criterion |
| I9-AC-8.e | PRE #9; later MVP only as named in criterion |
| I9-AC-8.f | PRE #9; later MVP only as named in criterion |
| I9-AC-8.g | PRE #9; later MVP only as named in criterion |
| I9-AC-8.h | PRE #9; later MVP only as named in criterion |
| I9-AC-8.i | PRE #9; later MVP only as named in criterion |
| I9-AC-8.j | PRE #9; later MVP only as named in criterion |
| I9-AC-8.k | PRE #9; later MVP only as named in criterion |
| I9-AC-8.l | PRE #9; later MVP only as named in criterion |
| I9-AC-8.m | PRE #9; later MVP only as named in criterion |
| I9-AC-9.a | PRE #9; later MVP only as named in criterion |
| I9-AC-9.b | PRE #9; later MVP only as named in criterion |
| I9-AC-9.c | PRE #9; later MVP only as named in criterion |
| I9-AC-9.d | PRE #9; later MVP only as named in criterion |
| I9-AC-9.e | PRE #9; later MVP only as named in criterion |
| I9-AC-9.f | PRE #9; later MVP only as named in criterion |
| I9-AC-9.g | PRE #9; later MVP only as named in criterion |
| I9-AC-9.h | PRE #9; later MVP only as named in criterion |
| I9-AC-9.i | PRE #9; later MVP only as named in criterion |
| I9-AC-9.j | PRE #9; later MVP only as named in criterion |
| I9-AC-10.a | PRE #9; later MVP only as named in criterion |
| I9-AC-10.d | PRE #9; later MVP only as named in criterion |
| I9-AC-11.a | PRE #9; later MVP only as named in criterion |
| I9-AC-11.b | PRE #9; later MVP only as named in criterion |
| I9-AC-11.c | PRE #9; later MVP only as named in criterion |
| I9-AC-11.d | PRE #9; later MVP only as named in criterion |
| I9-AC-11.e | PRE #9; later MVP only as named in criterion |
| I9-AC-12.a | PRE #9; later MVP only as named in criterion |
| I9-AC-12.b | PRE #9; later MVP only as named in criterion |
| I9-AC-12.c | PRE #9; later MVP only as named in criterion |
| I9-AC-12.d | PRE #9; later MVP only as named in criterion |
| I9-AC-12.e | PRE #9; later MVP only as named in criterion |
| I9-AC-12.f | PRE #9; later MVP only as named in criterion |
| I9-AC-12.g | PRE #9; later MVP only as named in criterion |
| I9-AC-12.h | PRE #9; later MVP only as named in criterion |
| I9-AC-12.i | PRE #9; later MVP only as named in criterion |
| I9-AC-12.j | PRE #9; later MVP only as named in criterion |
| I9-AC-12.k | PRE #9; later MVP only as named in criterion |
| I9-AC-13.a | PRE #9; later MVP only as named in criterion |
| I9-AC-14.a | PRE #9; later MVP only as named in criterion |
| I9-AC-14.b | PRE #9; later MVP only as named in criterion |
| I9-AC-14.c | PRE #9; later MVP only as named in criterion |
| I9-AC-14.d | PRE #9; later MVP only as named in criterion |
| I9-AC-14.e | PRE #9; later MVP only as named in criterion |
| I9-AC-14.f | PRE #9; later MVP only as named in criterion |
| I9-AC-14.g | PRE #9; later MVP only as named in criterion |
| I9-AC-14.h | PRE #9; later MVP only as named in criterion |
| I9-AC-14.i | PRE #9; later MVP only as named in criterion |
| I9-AC-14.j | PRE #9; later MVP only as named in criterion |
| I9-AC-16.a | PRE #9; later MVP only as named in criterion |
| I9-AC-16.b | PRE #9; later MVP only as named in criterion |
| I9-AC-16.c | PRE #9; later MVP only as named in criterion |
| I9-AC-16.d | PRE #9; later MVP only as named in criterion |
| I9-AC-17.a | PRE #9; later MVP only as named in criterion |
| I9-AC-17.b | PRE #9; later MVP only as named in criterion |
| I9-AC-17.c | PRE #9; later MVP only as named in criterion |
| I9-AC-17.d | PRE #9; later MVP only as named in criterion |
| I9-AC-17.e | PRE #9; later MVP only as named in criterion |
| I9-AC-18.a | PRE #9; later MVP only as named in criterion |
| I9-AC-18.b | PRE #9; later MVP only as named in criterion |
| I9-AC-18.c | PRE #9; later MVP only as named in criterion |
| I9-AC-18.d | PRE #9; later MVP only as named in criterion |
| I9-AC-18.e | PRE #9; later MVP only as named in criterion |
| I9-AC-18.f | PRE #9; later MVP only as named in criterion |
| I9-AC-18.g | PRE #9; later MVP only as named in criterion |
| I9-AC-18.h | PRE #9; later MVP only as named in criterion |
| I9-AC-18.i | PRE #9; later MVP only as named in criterion |
| I9-AC-18.j | PRE #9; later MVP only as named in criterion |
| I9-AC-20.a | PRE #9; later MVP only as named in criterion |
| I9-AC-20.b | PRE #9; later MVP only as named in criterion |
| I9-AC-20.c | PRE #9; later MVP only as named in criterion |
| I9-AC-20.d | PRE #9; later MVP only as named in criterion |
| I9-AC-21.a | PRE #9; later MVP only as named in criterion |
| I9-AC-21.b | PRE #9; later MVP only as named in criterion |
| I9-AC-21.c | PRE #9; later MVP only as named in criterion |
| I9-AC-21.d | PRE #9; later MVP only as named in criterion |
| I9-AC-21.e | PRE #9; later MVP only as named in criterion |
| I9-AC-21.f | PRE #9; later MVP only as named in criterion |
| I9-AC-21.g | PRE #9; later MVP only as named in criterion |
| I9-AC-21.h | PRE #9; later MVP only as named in criterion |
| I9-AC-21.i | PRE #9; later MVP only as named in criterion |
| I9-AC-21.j | PRE #9; later MVP only as named in criterion |
| I9-AC-21.k | PRE #9; later MVP only as named in criterion |
| I9-AC-21.l | PRE #9; later MVP only as named in criterion |
| I9-AC-21.m | PRE #9; later MVP only as named in criterion |
| I9-AC-21.n | POST MVP |
| I9-AC-21.o | PRE #9; later MVP only as named in criterion |
