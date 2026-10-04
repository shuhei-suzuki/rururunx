# Issue 9 requirements: configurable independent review

Status: proposed requirements. Merged dependencies #2/#4/#8 supply state, adapters and
Workflow. Core production work has not started. Prepared-input integration depends
on reviewed #19, provider typed-input contracts and the record-only binding port #43.
Review Bundle/delta construction #20 depends on this engine. Issue #9 must prove
the typed consumer with actual Git-backed producer fixtures; the full #20 producer and native #16 integration remain explicit MVP gates, avoiding a dependency
cycle or an invented producer-readiness claim. Issue9 owns and freezes the typed
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
permission construction/denial verified; native enforcement remains unverified
until real Issue16 evidence. Unsupported capabilities fail before model input.
Existing native permissions/auth/hooks/rules
remain authoritative. Reviewer or external mutation invalidates publication;
attribute it to a slot only when actual evidence establishes that attribution.
Formal gate review requires a clean committed target and an executor write lock
for the immutable target;
optional unlocked/advisory review is non-gating. Safe settlement or verified
cancellation releases the round lock; uncertainty retains it. Source integrity
checks supplement that lock throughout. Dirty/advisory review cannot issue a gate certificate.

The common factual core has one exact digest across slots. Declared specialization
artifacts have separate immutable per-slot hashes and are structurally additive:
the byte-identical core remains mandatory and cannot be replaced or omitted.
Specialization identity/source is visible and attributable; no semantic
non-contradiction guarantee is invented. rrx never injects same-round peer findings/current-round output or raw executor
conversation into any initial/member expansion input, rules/source slices or
manifests before every roster slot settles. Runtime DB/sidecars, peer
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
the bounded mandatory manifest and required unresolved-text delivery of 14.e;
resolved/dismissed full texts may remain exact-referenced expandable history. Whole raw peer
outputs/transcripts are never injected. Dismissal adjudication history retains
actor identity. Dismissal claims/rationales remain labelled claims,
not verified facts; re-raising a dismissed finding remains possible.
Eligible opinions after this declared prior-round exposure may satisfy the
independent floor: independence means current-round runtime-input independence,
not lifetime blindness to prior findings. This also applies to a slot’s fresh native
Session in a new full-roster retry round when its prior invocation had no structured
result; no within-round retry is permitted. Result/certificate
distinguish no prior-claim exposure from exposed re-review and never label an
exposed opinion blind. Before any later-round input, every earlier round's native
ownership must be safely settled; current-round peer findings remain excluded
until every roster slot settles.

## Verdicts, verification and completion

Slots return APPROVE, REQUEST_CHANGES or ESCALATE, plus typed findings and explicit
per-prior-finding dispositions. Each disposition identifies immutable original finding
hash/round/slot, exact inspected current target locations and evidence references,
and one of confirmed-fixed, still-present, false-positive, severity-downgrade,
not-applicable, duplicate or superseded-by-target-change (9.a). It retains the
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
Test same-round quorum after nondisputed Human dismissal versus new-round dispute,
ESCALATE and actual fix; no carried approval or certificate-floor substitution.

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
Any later policy/roster designation requires trusted recorded Human/controller
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
never permits discarding inconvenient findings. Within-round retry is disabled in Core: a retry needs a new immutable
round, includes prior failure/attempt history and consumes the round limit.
Each new round reruns the full frozen roster; approvals never carry across rounds
or changed targets. Same-target retry/dismissal rounds consume a typed identity
delta (baseline tree == new tree, with both exact HEADs retained) and prior-round
provenance; changed-target rounds consume
an actual revision delta. Rerun authorization compares repository tree content, not
HEAD alone: tree-identical commits are same-target for retry authority even while
their exact new HEAD is retained in provenance. Autonomous changed-target rerun
requires a nonempty source/artifact delta linked to a retained finding or explicit
nonapproval concern with actual inspected locations/check evidence. An empty
commit, unrelated change or cosmetic token delta alone is no authorization; a
whitespace fix qualifies only with checked evidence for a specific formatting
finding/concern. No supervisor model invents material relevance. Otherwise Human
adjudication is required under the same lineage limit. Certificates contain
outcomes from that exact round.

Lost/uncertain rounds surface a durable Human attention item with exact ownership
and blocker evidence. Trusted native recovery #14 must produce authoritative
terminal/cleanup evidence before safe release or superseding a round; Human
opinion, a dead PID or an edited terminal label cannot manufacture that evidence.
Until that port is integrated the round remains explicitly held. Automatic Lost
recovery and lossless hot upgrade are not claimed by this Core.

## Constraints and non-scope

Policy bounds: 1–32 slots, 1–32 local parallel launches, 64 cumulative rounds per Task/phase lineage, 256
findings per result, 8192 UTF-8 bytes per finding and 1 MiB per factual bundle/result
envelope. Limits are simultaneous ceilings: maximum count does not promise every
maximum-size text fits the envelope. No semantic truncation occurs. A 128 MiB
cumulative Task/phase-lineage artifact quota bounds ACTUAL RETAINED rrx evidence,
not repeated delivery bytes. This clarifies/revises the earlier ambiguous expansion-
delivery charge: charge owned copied blobs once by exact content/owner identity plus
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
per slot, 8 MiB verification/check artifacts and 4 MiB bounded control/adjudication/
attention/manifests per round, plus core and member-frame bytes and the existing
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
requires reviewed normal post-result cleanup shape and observed/declared latency
envelope from its actual production caller contract: a reviewed upper bound of the
supported NORMAL cleanup path through actual owned settlement, including required
native/tool/server shutdown stages and bounded dispatch/scheduling overhead. A p95,
typical latency or single observation is not that contract bound. The declaration
must be backed by actual producer code/profile and bounded shutdown/termination/
reap/owned-task settlement enforcement, with exact residual-uncertainty conditions;
an arbitrary timeout constant does not establish supported normal cleanup. Unknown
or escaped cohort ownership cannot be certified by selected-group death. Core
criteria4/16.b require these ACTUAL supported5/6/7 declarations before closure;
21.f public tracking alone cannot satisfy them. Until they exist the affected
preset/criterion stays open and refuses before effects, including Triple. Unknown tails or
unbounded required stages make the profile unsupported; exceptional OS/ownership
uncertainty remains the explicit held outcome. Durations use integer milliseconds;
strict deadline>envelope gives at least 1 ms normal-bound margin, without claiming a
universal timing guarantee. This conservative minimum is provisional and actual 16
measures its practical sufficiency; no statistical percentile substitutes for proof.
The configured settlement deadline must exceed that envelope; if unavailable or too short, the preset refuses
before member input. With default 30, a known envelope strictly below 30 seconds is
accepted; an envelope at/above 30 or unknown refuses. Default 30 is provisional, not
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
attention threshold, default 600; exceeding it holds for Human without pretending
a model timed out. Existing Runtime,
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
Session cannot occupy two slots. The Triple preset ships the Claude/Codex/Grok roster and all-of-three policy,
without an implicit self-review exception. With an MVP executor in that roster,
default selection rejects before input. An explicit visible allow-self independent-
Session policy makes that same preset usable; the author slot is never eligible
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
Issue9 owns ReviewSet-specific private authority constructors and the trusted
library ingress/controller composition, tested at that boundary without a CLI.
It reuses the approved Issue23 application trust rule, not an absent implementation
or a dependency on merging Issue23. A future CLI/controller caller must compose
this actual port; native/Workflow/Broker runtime channels cannot expose it.
Only that trusted ingress/controller composition can mint nonserializable authority,
with actual principal/origin and exact policy/target/evidence identity. Native,
Workflow, agent output/proposal and Approval Broker runtime JSON/API/IPC channels
cannot label themselves Human or obtain that authority. Broker decisions never
substitute for Human adjudication/activation. This covers blocker dispositions,
post-opinion relaxation, supersession/decomposition, authorship/unknown-delta
disposition and governing-policy activation. Direct same-UID machine/CLI/DB action
outside these runtime APIs is an explicit trust limit, consistent with Issue23;
this is not biological identity proof or an OS sandbox. Trusted Rust composition
is not authenticated against malicious linked code. No extra authentication
infrastructure is implied or claimed here. The principal/origin follows the SAME
approved23 derivation contract; composing code may not supply an arbitrary principal
or relabel library-composition as CLI/UID ingress. Preserve actual invocation origin
and test that native/API/IPC and library-origin relabeling cannot mint CLI authority.
This reuses the contract/type semantics, not a mandatory23 implementation merge.

At most one nonterminal Review Set owns a Task/Workflow phase lineage. A new
Set cannot approval-shop by resetting blockers, holds, history, round count or
artifact quota. Explicit trusted Human/controller supersession records authority/
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
erase the lineage hold. Trusted Issue14 recovery is the only releasable exit.
Obligation lineage is the runtime-owned stable identity allocated for the initial
Project/Task/Workflow-phase obligation, retained by successor Sets, generations and
actual decomposition descendants. A new Task cannot choose a base behind which
applicable held/exhausted obligations disappear. Formal admission retains exact
registered-base ancestry or the recorded legitimate base-sync operation. Ancestry
identifies retained contributions; it does not alone prove every later obligation
is affected by every reachable uncertified contribution.

Issue9 owns the typed applicability consumer/checker and trusted verification boundary.
Actual12 owns the reviewed impact/applicability producer using18/20 exact source/
artifact inputs. This ownership and conformance obligation must be publicly tracked
under21.h before9 closes, without a9→12 merge cycle. Until the actual reviewed producer/
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
attribution for a new unrelated Task under 10.a without inheriting its review budget/
author exclusions. Its original locks/operations still retain their actual scope.
The accepted certificate and actual merge artifact bind exact reviewed target/
contribution; generic Merged/Completed labels, names, claimed merge or Git reachability
cannot create that handoff or wash held/exhausted history. Out-of-band merges retain
explicit uncertified provenance and applicable obligations, assessed under the same
evidence rule. Termination alone never certifies prior contribution. This scoped
policy corrects the prior Project-wide ancestry availability defect: exhaustion is
not a permanent veto on proven unrelated obligations. #16 measures applicability
holds, disjoint progress and actual resource-scope blocking separately (11.d).
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
inspection or explicit Human adjudication is required. At most two such
full-roster retry rounds per lineage
and at most one per (lineage, exact target tree, provider-family/error-kind) cause.
The same cause on a changed target can consume the remaining lineage-wide allowance;
changing tree never resets the separate two-retry ceiling or 64-round quota. They consume the same 64-round
and artifact limits. A repeated proof/cause, exhausted retry allowance, unresolved
partial result or Lost requires Human/recovery rather than another automatic sample.
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
currency. After settlement, Workflow applies the escalation and a new full-roster round must
meet the escalated floor. If the frozen roster cannot meet it, hold for actual
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
explicit policy/Human disposition; a changed executor cannot silently self-confirm
its own earlier changes.

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
Resource-queue attention exits by authorized rescheduling with available permits
and a fresh target/claim check, or termination; it does not reuse expired authority
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

Numbered criteria 1–21, including their mapped case lists and all normative acceptance
subcriteria with explicit stable IDs below, are the complete closing set. Each ID
requires linked actual evidence; no unnumbered acceptance paragraph adds a hidden
closing obligation. Definitions above specify the tested behavior. Items 1–8 map to the public
Issue #9 checkboxes; items 9–21 add integrity/availability and their explicit cases. Every criterion
needs actual consumer evidence; proposed tests and independent development reviews
are not runtime proof.

1. First-class two-reviewer and four-plus rosters run through real Store/Workflow
   delegation, using one fixed Task context, exact locked revision and
   independent member ownership. Reject 0/33 slots, invalid parallelism and duplicate
   Session ownership.
   One eligible non-author slot can pass QUICK/STANDARD; STRICT with one slot
   rejects before input because F=2, even if its mode would need only one opinion.
2. Two-of-three quorum counts exact-round APPROVE only; reject N=0/N>M and quorum
   parameters on all/any. All, quorumM and any/quorum1 have equivalent boundary
   outcomes and retain every nonapproval/blocker.
3. All-of-two requires both approvals. Exercise empty/nonblocking REQUEST_CHANGES,
   persistent nonapproval and audited Human relaxation/new-round or termination
   without fabricated approval.
4. All-of-three Triple preset resolves actual registered production Claude (#5),
   Codex (#6) and Grok (#7) read-only declarations and supported explicit model/effort configurations.
   With each MVP executor-in-roster, prove default self-review rejection and explicit
   independent-Session permission success. An author opinion never satisfies the
   independent floor; reject a pure-author formal roster and STRICT with only one
   eligible independent approval. The two non-author Triple Sessions can meet the
   STRICT floor while all three opinions must APPROVE under all mode. Unsupported/unregistered configurations
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
    reset: every admitted round consumes one count, including held/superseded rounds.
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
    Test refusal through the actual downstream certificate consumer. Certificate is
    not merge permission;43 binding remains record-only, never a production-gate override.
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
Acceptance criterion 16 includes this fixture-level attribution, without inventing native
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
non-author family can meet STRICT when the frozen roster/duplicate-agent/diversity
policy permits them. A fixed Triple roster with Claude and Codex as recorded
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
against the 128-MiB RETENTION quota by 14.c. Every delivery, including repeated identical
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

17. Additional mandatory cases mapped to criteria 1–4 and 8–16:

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
  Criterion 21/#16 reports both repeated-same-tree and lineage-wide-ceiling interruptions.
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

18. Additional mandatory cases mapped to criteria 1, 4, 10, 14–16:

- **18.a** An empty commit with identical tree stays same-target for retry authority;
  unrelated or unexplained whitespace deltas cannot trigger another autonomous
  sample after dissent. A checked formatting repair linked to the retained
  concern can authorize a changed-target round; prior opinions never carry over.
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
    frequencies, retained capacity/Task-time and Human-interruption impact. Criterion
    21 is the canonical handoff: EVERY subcriterion attributing a measurement to
    #16 must be quoted by stable ID from public #16 acceptance before closure,
    This includes deferred evidence as well as measurements for ALL IDs referring
    to named inherited owners: 16.b/16.c native read-only enforcement on real transports,
    8.a trace-observed isolation and 16.a efficiency attribution. #14 must also
    explicitly consume 11.b lineage/descendant holds and target-mutation taint,
    not just individual member locks. #20 must quote 8.g consumer-contract
    conformance, including stale/foreign/relabelled refusal.
    Including 8.c (native auto-injection versus passive visibility), 8.d (blind
    versus prior-claim-exposed rounds), 14.d (large required coverage/over-cap core,
    including accumulated claims), 20.a/20.b (two-author STRICT and zero-eligible
    roster availability), 21.b (partial-output early-stop/Human attention), and
    21.c/21.d (normal/cancel/timeout settlement holds, capacity and Task-time).
    It additionally requires 8.h claim exposure/verdict-input exclusion, 11.e narrow/
    widened native effect scope, 14.c retention versus finite actual-delivery/provider
    metrics (actual #21 telemetry), and 9.f Human-only blocker clearance impact,
    18.g normal cleanup bound/profile readiness, 11.d scoped applicability/resource impact and
    round/artifact-quota exhaustion frequency, retained
    Task-time and decomposition/termination impact (14.e below). Branch blob URLs
    are not stable obligation owners; a permalink retains the gated SHA after
    branch deletion. If a public owner does not carry
    its obligation, this Issue cannot close until that tracking is concrete. This
    requires traceability, not completion of #20 or a cyclic merge dependency.
    Status/Human attention must state that no in-runtime Human action can release
    a Lost hold before trusted #14 recovery. This canonical handoff additionally
    covers 21.e trusted 15/24 Human ingress,21.f actual 5/6/7 producer contract ownership,
    and 21.g actual 14/27 fairness/resource recovery and 12 verification evidence.
    Public tracking, rather than downstream12/15/24/27 implementations, is required
    before9 closure; actual core-native declarations/support under4/16.b remain real
    pre-closure inputs, not tracking exemptions. No cyclic merge dependency is introduced.
    New21.h tracks12 applicability producer/conformance and9 consumer ownership.

**21.b — Default measurement handoff.** Safe retry/timeout defaults are retained until real #16 measurements justify a
separately reviewed policy change. Native dogfood reports how often partial-output
cancellation, retry exhaustion and long reviews require Human attention; early
stop is expected to hold when cancelled members emitted unexamined content.
This does not invent an automatic weak-policy fallback to reduce interruptions.

**2.a — Potential Medium veto.** Criterion 2 also rejects outvoting a potential Medium finding: two approvals plus
that dissent stay held, while safely settled Low-only dissent outside the blocking
set may be tolerated. Independent resolution is recorded before any certificate.

**8.b — Copy provenance.** Criterion 8 excludes raw-output/transcript copies by recorded provenance as well as storage
location: known exact output-hash copies and paths declared by the scoped artifact policy are
excluded with a visible note, or hold before input if mandatory. A committed copy under
doc/verification does not become eligible merely because Git tracks it. Unknown content is not
claimed to be exhaustively recognized; native/source visibility limits are reported. Test
committed copies, explicit requests and symlink aliases.

**12.d — Forged attribution.** Criterion 12/20 tests forged Git author/trailers: runtime-owned Codex dispatch/delta attribution
keeps Codex excluded despite a commit string naming Human/Grok. Uncovered delta bytes require
explicit recorded disposition before formal input.

**9.b — Class-wide verification.** Criterion 9/10 class-wide verification
records the checked locations and N inspected / M changed, preserving scope and check artifact
provenance; synthetic facts do not claim exhaustive native inspection.

**20.a — Strict roster availability.** STRICT fixed Triple is unavailable when document drafting and implementation have contributed
two native families. Supported alternatives are an eligible custom roster meeting the same
floor, or explicit Human adjudication selecting an eligible roster, termination without a certificate; allow-self never supplies the missing
independent approval. #16 must exercise this availability limit alongside supported Triple
configurations.

**9.a — Blocker dispositions.** Every disposition removing a potential/verified blocker from the effective veto
uses the same explicit Human/original eligible finder/two diverse eligible confirmer
rule. Fix, false-positive dismissal, severity downgrade, not-applicable, duplicate
and superseded-by-target-change are typed dispositions; unknown dispositions reject.
Actual source removal/checks are facts, but executor-only applicability conclusions
never clear the blocker. Criteria 9/19 test target-change applicability and duplicate
claims under all/quorum/any/supersession: unresolved independent confirmation holds.

**11.a — Exhausted decomposition.** Criterion 11 tests Human-authorized decomposition after lineage exhaustion with an
open High: descendants retain its veto, original base authors and shared exhausted
round/quota counters. A child certificate remains held; relabeling the frozen child
base or creating an autonomous Goal follow-up cannot restore eligibility/budget.

**1.a — Observed parallel admission.** Criteria 1/14/18 prove actual parallel admission with production adapters and reviewed
synthetic contract peers: with M available resource shares and local parallelism M,
every member is observed admitted before any member settles, with overlapping
owned operation lifetimes. This observed admission and overlap is retained
as evidence for two and Triple rosters. Parallelism 1 reports serialized execution,
never parallel success; unavailable shares queue visibly without bypass.

**3.a — Policy relaxation.** Criterion 3 rejects automatic post-opinion all→quorum relaxation; explicit Human
relaxation above floors starts a new retained round. A frozen pre-admission conditional
policy is separately attested, not fabricated after dissent.

**18.e — Concern-linked confirmation.** Criterion 18 allows one
concern-linked inspection confirmation for APPROVE with a potential Medium blocker;
it still requires independent blocker resolution and rejects a second same-tree
autonomous confirmation.

**21.c — Settlement availability metrics.** Criterion 21's #16 handoff must include settlement-expiry
Lost frequency and retained capacity/Task-time, not only inference timeout counts.
Report successful-result normal-completion cleanup expiry separately from
cancellation/timeout cleanup expiry; either can leave an absorbing Lost hold.

**12.e — Human floor preservation.** Human adjudication never substitutes for the mandatory independent approval floor
and never mints a formal Review-Set certificate below F. It may select an eligible
roster or terminate without a certificate. Obligation-preserving decomposition is structural history, not an eligibility/budget recovery exit. No Human-only Workflow gate override is implemented or promised by this Issue; any future override requires its own explicit requirement/ownership gate. Criteria 12/20 test
two-author-family STRICT Triple plus Human adjudication: still no certificate.

**20.b — Zero eligible provider set.** When all three MVP native families are cumulative authors, every class has zero
eligible reviewers within that provider set. QUICK and STRICT reject before input
with named exclusions; allow-self, decomposition and Human opinion never restore
eligibility. Supported exits are an eligible registered non-author provider that
meets the same capabilities/floors, or termination without a certificate. Such a
fourth provider is not promised by this MVP. Criteria 17/20 cover this unready state;
criterion 21 requires #16 to report its frequency and availability/Human impact.

**8.c — Fresh native identity.** Every member in every round requires a freshly created native Session, never an
executor Session, a resumed native history or a prior-round native Session. A new
runtime UUID alone is insufficient: the actual adapter startup contract must
guarantee fresh native creation and reject conflicting resume/continue configuration
before input. Criterion 8 rejects owned executor/history reuse and unsupported
fresh-session capability. Native auto-loaded global/user instruction files, memory
and external history/context channels are distinct residual channels, listed as
declared-by-adapter or unverifiable in result/certificate. Runtime-controlled input
independence never certifies those channels; default auth/hooks/rules are preserved.
Criterion 21's #16 isolation handoff requires observing/testing native auto-injection
separately from passive filesystem/tool-read visibility.

**14.b — Expansion renewal accounting.** Expansion request/slot byte allowances reset only for a newly admitted (round,slot);
per-round aggregate and inherited lineage quota never reset by slot renewal.
Criterion 17 tests that accounting across two rounds.

**9.c — Human self-adjudication attribution.** Human blocker adjudication is
an explicit authority exemption from native author exclusions, but certificate
evidence flags self-adjudication when that Human authored the reviewed delta.
Criteria 9/19 preserve actor/reason/evidence and this flag, never present it as an
independent model confirmation or approval.

**12.f — Activated governing authority.** Policy authority (criteria 3/12/15) resolves from an activated Runtime/registered
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

**10.a — Upstream integration attribution.** Base-sync attribution (criteria 10/12/20) records actual registered-base ancestry
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
policy allows duplicate-agent identities or distinct registered aliases; mandatory
family diversity, if imposed by that policy, still applies. Otherwise use an actually
supported non-author provider or terminate. Decomposition never recovers eligibility
or exhausted quota. Tests distinguish certificate-capable roster selection from
terminal-only exits.

**21.d — Accepted pre-recovery risk.** The conservative pre-#14 settlement-expiry Lost policy can
permanently hold a Task and its capacity even after late owned cleanup; this accepted
Core availability risk must be stated in the public Issue and #16 handoff.

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
frequency separately under 21; actual 21 provider token/cost counters remain nullable.
All retained ReviewSet evidence consumes the same 128 MiB lineage quota, including
transport diagnostics, verification/check logs, adjudication, attention records,
manifests, result/expanded/member frames and claim/exposure records. No uncharged
evidence kind creates unbounded output; references identify externally retained
artifacts and their declared separately owned bounds, never hidden copies here.

**16.c — Read-only proof basis.** Certificate basis is reviewed adapter contract
plus runtime permission construction until #16 real native enforcement evidence;
synthetic peers do not prove native enforcement. Result/certificate records that basis.

**8.d — Prior-round exposure.** A transient same-target retry with one prior
non-result slot and completed peer findings delivers only permitted labelled prior
claims after safe settlement. Its eligible fresh opinions may count F under declared
current-round input independence; certificate records exposure and never claims
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
with retained evidence. Actual trusted ingress/controller succeeds with exact
principal/origin/digest/evidence. Direct same-UID local binary/DB/machine action
outside these APIs is explicitly outside the application guarantee, as in Issue23.
The positive authority is the actual Issue9-owned library ingress/controller port,
not a mock actor field or an assumed future Issue23 implementation. Its evidence
records origin=library-composition and proves that application port, not biological
Human action; no CLI ingress is implemented or implied by this library fixture.

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
specialization with controller/Human provenance is additive and cannot drop core.

**11.c — New Task lineage base.** Registered-base sync retains recorded uncertified
contributions even after upstream reachability. A new obligation may be fresh only
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
settlement and no Lost/partial/failure. No prior approval is carried into counts.
The new required coverage is cumulative baseline→current target delta plus all
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
retention follows 14.c, not a second charge per delivery.

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
waive the certificate floor. The prior unconditional Project-wide ancestry veto was
an availability defect and is replaced by this evidence-bound scope rule.

**14.f — Queue attention bounds.** Queue threshold 0/3601 rejects;1/3600 is accepted
within the remaining policy, default 600 is recorded. Queue expiry never invents a
native timeout or releases an owned operation.

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
its pre-admission activation and exact trigger.

**14.g — Admission reservation and retention.** Reject a round before input when
remaining lineage quota cannot cover its worst-case frozen reservation; an exact-
reservation boundary admits and retains every bounded in-flight result/diagnostic/
expansion. Test numeric diagnostic/model prefix and shared verification/journal
ceilings, unknown suffix and reserved control margin. Held members retain unused
reserved capacity; safe closure releases only unspent reserve, never actual history.

**13.a — Post-certificate veto.** Later actual evidence raising a certificate-bound
Low into the effective blocking set makes that original certificate stale/non-
consumable while preserving it. A downstream actual review-gate consumer rejects
it; already irreversible merge/PR outcomes stay factual for 13 reconciliation, never
fabricated rollback. A new certificate requires applicable current review authority.

**9.f — Clearance availability.** If neither an eligible original finder nor a
pair of diverse non-author confirmers exists, blocker clearance is trusted Human-
only and holds until that adjudication or termination; quorum/allow-self cannot
clear it. #16 measures this Human-interruption frequency/Task impact, distinct
from independent approval-floor availability, under criterion 21's canonical handoff.

**18.g — Normal cleanup bound.** Actual 5/6/7 caller conformance identifies normal
post-result cleanup shape/envelope. At default 30, a known envelope strictly below 30
seconds accepts; at/above 30 or unknown refuses before input. Test 29.9/30/unknown and
a supported configured deadline strictly greater than its known reviewed upper bound,
within 1–3600. Test the one-time normal-result timer start for natural exit and
runtime-requested close, duplicate terminal events without extension, and rejection
of a percentile/typical-only envelope. Repeat
normal-completion expiry through actual owned supervision and expose held ownership.
Before actual 14 recovery and required production native proof, these library/synthetic
certificates are integration evidence, not production merge gate readiness. The MVP
still requires actual native acceptance; synthetic timing does not validate defaults.


**9.g — Actual disposition callback.** A later complete opinion must deliver explicit
prior-finding dispositions through the typed production consumer contract. Prove an
eligible finder’s confirmed-fixed with exact inspected locations/check evidence clears
its prior High after a new full round, while general APPROVE, unknown hash, stale target,
wrong slot/family, malformed disposition or executor-only claim keeps the veto.
Finder still-present versus two diverse confirmed-fixed holds for trusted Human;
post-opinion confirmer redesignation rejects, including an activated designation
between rounds after that agent/slot already expressed its opinion.
Retain every original finding/opinion and disposition independently.

**21.e — Trusted product Human ingress.** Issue9 owns the library authority/ingress
contract and negative native/Workflow/Broker JSON/API/IPC tests. Actual #15 TUI and #24
Goal/controller handlers must publicly own their composition for every Human-only
adjudication, policy/scope/supersession and termination exit, including exact scoped
reason/evidence and principal provenance. No such handler is claimed implemented.
Until available, status exposes integration-pending/nonactionable attention rather
than a fictitious button/CLI escape. Public acceptance quotes this stable ID before 9
closes; product handler completion remains a separate MVP gate, not a9→15 merge cycle.

**21.f — Native contract producer ownership.** Issue9 owns consumer conformance and
runtime declaration checks; actual 5/6/7 adapters or explicitly linked reviewed follow-up
Issues own producer obligations for 8.c fresh no-resume native identity,16.b/18.g normal
cleanup shape/bounds,18.c requested/effective configuration,16.b/16.c runtime read-only
permission construction and 8.a available access traces. Public owner entries identify
exact supported profiles and refusal limits before 9 closes. Existing declarations or
synthetic fixtures do not assert these producers or real native acceptance complete.

**21.g — Fairness with held shares.** ReviewSet retains exact global/Project/provider
shares under Lost; its local parallelism/per-Project cap never releases them or invents
extra capacity. Prove remaining actual capacity progresses a disjoint Project and
report zero-eligible-capacity as held-resource starvation, not runnable scheduling
success. Actual #27 fair admission and #14 trusted recovery must publicly own cross-Project
progress with real held shares, using 12 verification evidence and 16 scoped metrics.
If a Project can consume all global/provider capacity in retained uncertainty, that
cross-Project availability gap is explicit until the reviewed scheduler/recovery
composition;9 library tests/measurements alone cannot satisfy MVP fairness. Tests keep
root-scoped holds while allowing truly disjoint progress when limits permit. Status
and 16 distinguish actionable Human attention from recovery-pending/nonactionable holds.

**8.h — Prior verdict input exclusion.** Runtime member input/expansion excludes
separate prior-round verdict/approval-tally/non-finding peer-opinion fields, including
a same-tree transient retry after other prior slots APPROVEd. Mandatory finding
resolution-status metadata is about finding applicability/adjudication, not vote
counts. Preserve exact original finding claim text/hash and recorded prior delivery:
that prose can incidentally state an opinion and remains labelled attributed claim,
not a promise of semantic blindness. Private retained outcomes/certificates keep
prior verdicts for provenance, never feed them back as vote hints. Actual 16 reports
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

**21.h — Applicability producer and production enablement.** Public12 acceptance
must own the reviewed trusted impact/applicability producer using18/20 scoped exact
inputs and9 typed consumer/checker conformance. Method and known-incompleteness basis
must be explicit; unknown/incomplete or missing producer inherits/refuses, never
fresh-budget production certification. Actual9 fixtures prove the consumer/refusal
boundary only. This is a separately tracked MVP availability/impact gate without
an implementation dependency cycle. Multi-Project production ReviewSet integration
also remains DISABLED until actual14 retained-share recovery and27 fair admission
compose; a synthetic library success or per-Project cap cannot erase held shares.
Public Issue9 states that gate;21.g/#16 owns its actual cross-Project evidence.

The canonical public handoff table below summarizes criterion21 ownership; individual
stable subcriteria still retain their exact stated acceptance. Quoting a gate never
claims it complete or waives actual9 core support prerequisites.

| Stable acceptance keys | Public owner / actual responsibility |
| --- | --- |
| 11.b, 11.e, 21.c/d/g | #14 exact held native/lineage/resource recovery; #27 fair admission; #16 scoped hold/availability measurements |
| 8.g, 10.d | #20 production typed bundle/delta/coverage; #9 consumer contract; #12 impact producer; #18 captured source inputs |
| 11.c/d, 21.h | #12 trusted applicability producer, #18/#20 inputs, #9 consumer; #16 actual disjoint/unknown/resource impact |
| 16.b/c, 18.g, 8.a/c | #5/#6/#7 actual supported native/config/permission/cleanup declarations or tracked reviewed follow-up; #16 real enforcement/trace/default evidence |
| 8.d/h, 16.a, 14.c/d/e, 17.c, 20.a/b, 21.b, 9.f | #16 actual exposure, efficiency, finite delivered versus retained bytes, quotas/retries/eligibility/Human impact; #21 nullable native telemetry |
| 9.d, 21.e | #9 trusted ingress contract; #15/#24 actual scoped product Human handlers |

Every criterion and bold subcriterion ID above is a stable closure-evidence key.
Absent suffixes are intentional reserved IDs; reordered subcriteria retain their
stable IDs rather than renumber historical evidence.

Fixture mechanism, private port layout, hashes/transaction algorithms and controlled
transport barriers belong in Issue9 design; linked evidence must demonstrate these
observable acceptance outcomes through actual production consumers.
