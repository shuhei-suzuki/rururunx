# Issue 9 requirements: configurable independent review

Status: proposed requirements. Merged dependencies #2/#4/#8 supply state, adapters and
Workflow. Core production work has not started. Prepared-input integration depends
on reviewed #19, provider typed-input contracts and the record-only binding port #43.
Review Bundle/delta construction #20 depends on this engine. Issue #9 must prove
the typed consumer with actual Git-backed producer fixtures; the full #20 producer and native #16 integration remain explicit MVP gates, avoiding a dependency
cycle or an invented producer-readiness claim.

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

Each slot freezes a unique ID, agent, requested model/effort and independent native
Session. All slots consume the identical frozen Task context and locked revision
without incrementing Task.context_version or owner currency per reviewer. Temporal
concurrency is best effort within permits; queued/serialized members retain the
same input exclusions and residual native filesystem visibility caveat. A reviewed explicit ReviewSet delegation owns the entire roster;
existing single-actor context ownership cannot be bypassed by role or JSON claims.
Executor, Consultant and ApprovalReviewer cannot impersonate a formal slot.

Formal reviewers must support verified read-only native operation; unsupported
capabilities fail before model input. Existing native permissions/auth/hooks/rules
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
manifests before every independent member finishes. Runtime DB/sidecars, peer
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
claims with original author slot, severity and exact text hash. Whole raw peer
outputs/transcripts are never injected. Dismissal adjudication history retains
actor identity. Dismissal claims/rationales remain labelled claims,
not core facts; re-raising a dismissed finding remains possible.

## Verdicts, verification and completion

Slots return APPROVE, REQUEST_CHANGES or ESCALATE, plus typed findings. Native
failure, timeout, malformed output, cancellation and Lost are separate outcomes.
Only completed, well-formed, exact-target APPROVE counts. Required review
instructions/skills must resolve and be deliverable to every designated provider
before any member input; missing, foreign, unresolvable or undeliverable mandatory
content rejects with Human attention. Record delivery in the bundle separately
from native-side activation observed or unverifiable; no invented activation.
REQUEST_CHANGES never
becomes APPROVE merely because its findings were dismissed. ESCALATE requires a
visible human-attention hold. An approval containing a potential blocking finding
cannot satisfy the round until that finding is resolved independently.

Severity is Critical/High/Medium/Low. The immutable policy's blocking set always
includes Critical/High/Medium; configuration may add Low but cannot exempt that
Core floor. Findings are proposals until
verification supplies actor identity (agent/Session or Human), exact target,
inspected/changed locations and actual evidence. No supervisor LLM invents facts.
An executor-only dismissal cannot erase a formal blocker: false-positive dismissal
requires explicit Human judgment OR later-round confirmation by the original
finder who is not a recorded delta author by identity or family, or by at least
two policy-designated slots each excluded from all recorded authors.
Each confirmer must differ from every recorded delta author in registered agent
identity and native provider family, and a two-confirmer pair must also differ from
each other in both registered agent identity and native provider family; alias/same-agent permission cannot create dismissal independence.
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
outcomes and verification/evidence references. Prior-round claims and their original
hashes/actors are in the byte-identical shared core; per-member exposure entries attest that
same delivery, never conceal a different prior claim set. Specialization is separately
declared/hashed and cannot remove this core. It is evidence, not merge authority.

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
cumulative Task/phase-lineage artifact quota prevents unbounded retained output; exhaustion holds
with recorded byte/hash provenance and Human attention. Transport output is
bounded at acquisition, retaining the observed bounded prefix, exact byte count/
rolling hash and overflow reason; unseen suffix content is never treated as safe.
Malformed/partial output needs independent inspection or a fresh successful round
with explicit resolution; an unknown suffix cannot be dismissed as failure-only.
Native-admission startup timeout is 1–3600 seconds, default 60, beginning at
permit acquisition before spawn/handshake/auth/capability checks. The separate
review timeout is 1–3600 seconds after native admission, default 600. A separate
settlement deadline of 1–3600 seconds (default 30) starts at any cancellation,
startup/review expiry or terminal cleanup request. All three are frozen in policy;
missing authoritative terminal/cleanup evidence at settlement expiry becomes a
durable uncertain/Lost Human hold, retaining permits/locks/owned supervisor
bookkeeping. The owned operation remains supervised; expiry never drops its
process-group owner or invents death. Once persisted as uncertain/Lost, this universal absorbing
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

Default formal policy forbids self-review slots and duplicate agent identities.
A self-review slot is the current Task executor agent OR any registered identity
or native family in the cumulative delta-author set (including document drafting
and former executors). It requires explicit allow-self before input, including a
different alias of an author's family; with that permission its opinion never
satisfies the independent floor or any blocker-resolution confirmation.
Explicit policy may allow same-agent independent Sessions;
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

At most one nonterminal Review Set owns a Task/Workflow phase lineage. A new
Set cannot approval-shop by resetting blockers, holds, history, round count or
artifact quota. Explicit Human/Workflow supersession records authority/reason and
inherits unresolved potential/verified blockers, adjudication and cumulative
limits across roster, policy and generation changes. Target change never erases
unresolved findings; verification must explicitly resolve applicability or fixes.
Lineage exhaustion can terminate without certificate or hold for explicit Task
scope decomposition; it cannot reset into a new passing Set. Decomposition requires an explicit
Human actor/reason, never autonomous Goal follow-up creation. Child Tasks inherit unresolved
potential/verified blocker vetoes and original evidence, cumulative author attribution from the
parent frozen base, all remaining budgets and the same parent obligation-lineage identity.
Counters/quota are shared across those descendants, not restarted per child Task. The inherited
findings require the same independent resolution before any child certificate. Decomposition never
certifies the old phase; Goal history alone is insufficient enforcement.

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
verified diagnostic progress, at most two such full-roster retry rounds per lineage
and at most one per provider-family/error-kind cause. They consume the same 64-round
and artifact limits. A repeated proof/cause, exhausted retry allowance, unresolved
partial result or Lost requires Human/recovery rather than another automatic sample.
Repeated blind same-target sampling is not authorized by the lineage ceiling.
A relaxed policy starts a new round or explicitly superseding Set,
retaining prior opinions, obligations and lineage limits; it never manufactures an
APPROVE. ESCALATE/dispute requires authorized Human adjudication,
then a new full-roster round or termination without certificate; malformed/partial
output needs independently recorded inspection/resolution and new round; unknown
suffix requires new successful review rather than pretending it was examined;
quota/round exhaustion requires termination without certificate or explicit Human-authorized scope
decomposition with inherited obligation lineage; Lost remains held until trusted native #14
cleanup. The specific hazard is accepting generic late terminal/cleanup JSON as a claimed
continuously owned supervisor without a private continuity proof bound to the persisted Lost
owner/attempt. That would launder uncertain ownership into capacity release or fresh dispatch.
No such reviewed atomic recovery handoff exists here, so factual late evidence is retained but
cannot authorize release. None of these
exits counts the prior failed/non-approving slot as approval.

Rule, policy, context/Workflow generation or required instruction change during a
round invalidates certificate publication. Findings may carry a typed upward risk
signal to Workflow; no automatic downgrade. An upward signal immediately holds certification;
already admitted peers settle normally as retained evidence, with no new input under obsolete
currency. After settlement, Workflow applies the escalation and a new full-roster round must
meet the escalated floor. Risk-escalation hold exits only through that settled escalation/new
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
restart/native recovery #14 proves cleanup. GitHub irreversible external outcome
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

## Consolidated acceptance

Numbered criteria 1–21, including their mapped case lists and all normative acceptance
paragraphs below, are the complete closing set. Unnumbered MUST/required/reject obligations in
this section require linked evidence under the governing numbered criterion; they are not
optional commentary. Items 1–8 map to the public
Issue #9 checkboxes; items 9–21 add integrity/availability and their explicit cases. Every criterion
needs actual consumer evidence; proposed tests and independent development reviews
are not runtime proof.

1. First-class two-reviewer and one/four-plus rosters run through real Store/Workflow
   delegation, using one fixed Task context, exact locked revision and
   independent member ownership. Reject 0/33 slots, invalid parallelism and duplicate
   Session ownership.
2. Two-of-three quorum counts exact-round APPROVE only; rejectN=0/N>M and quorum
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
   Executor-only dismissal/downgrade and two same-agent/provider-family confirmers
   reject. Independent eligible confirmation or explicit Human judgment records
   evidence; fix candidates/commit/checks require a new full-roster round and explicit
   eligible resolution before a blocker is cleared.
10. Same-target retry and changed-target remediation rerun the full frozen roster,
    retain prior outcomes and never carry approvals. Within-round retry is absent.
    Verify/fix/commit/test/re-review and repeated Human-resolved ESCALATE preserve
    adjudication as labelled history, not model APPROVE.
11. Supersession retains unresolved obligations and lineage floors across executor,
    roster, policy and generation changes. Cumulative 64-round/128-MiB limits never
    reset: supersession at round 63 permits only one further round. Scope decomposition
    preserves the same obligation-lineage vetoes, author attribution and budgets in descendants; Human authorization is required and no child certifies the prior phase.
12. Class/Project policy floors reject missing/weaker policies (including blocking
    {Critical,High} without Medium), early-stop under STRICT/security and absent
    required configuration. Pairwise independence/delta-author exclusion and unknown
    authorship decisions are visible; same-agent permission does not authorize
    dismissal by an author.
13. Certificate binds exact phase/generation/claim, target/source/core/member hashes,
    mandatory instruction/skill/context/checkpoint provenance and settled evidence.
    Source/rules/policy/risk/context drift and cross-phase replay cannot pass.
    Certificate is not merge permission.
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

A native provider family is the trusted registered adapter's declared native
implementation family (for example claude, codex or grok), not an agent alias or
caller-supplied output. Undeclared/unknown family cannot establish independence.
Eligibility excludes every recorded native delta-author identity AND family;
unknown authorship needs explicit recorded disposition before formal input.
Human-authored changes are separately attributed, not invented native families.

Each member's queue/startup/review/settlement timings and usage carry exact
Project/Goal/Task, phase, Set/round/slot, Session and agent attribution. Provider
token/cache/cost measurements remain nullable with reasons, separate from runtime
byte/token estimates; source bytes and reviewer amplification are attributable.
Acceptance criterion 16 includes this fixture-level attribution, without inventing native
measurements or replacing the #16 efficiency comparison.

Inherited MVP obligations are linked explicitly: [#20 acceptance](https://github.com/shuhei-suzuki/rururunx/issues/20)
requires production deterministic bundle/delta/expansion integration and the same
mechanism for two/Triple; [#16 acceptance](https://github.com/shuhei-suzuki/rururunx/issues/16)
requires actual native two/Triple results, isolation and efficiency comparison.
[#14 recovery](https://github.com/shuhei-suzuki/rururunx/issues/14) must consume
ReviewSet Lost/uncertain member holds and reconcile exact native ownership, locks
and resource permits with trusted evidence. Issue #9 never releases them by opinion
or reset. Core closure reports these still-open MVP obligations explicitly.


Authorship is cumulative from the lineage's frozen Task base revision to the
reviewed target. Later identity/short revision deltas never erase earlier native
contributors or former executor families. Independent approval counts eligible
Sessions, not distinct families: two separately owned Sessions from the remaining
non-author family can meet STRICT when the frozen roster/duplicate-agent/diversity
policy permits them. A fixed Triple roster with Claude and Codex as recorded
authors has only one eligible slot and rejects before input, naming excluded
families and the unmet floor; neither supersession nor relaxation can lower it.
An explicitly allocated eligible custom roster can satisfy it, without treating
an author alias as independent. Test both outcomes.

Items 1–4 require actual runtime consumer execution through registered production
Claude/Codex/Grok capability resolution and provider adapter contracts. Deterministic
stand-ins may replace only the native process/transport peer. Conformance evidence must identify
the exact reviewed #5/#6/#7 adapter contract fixture source commit/path/digest and the replayed
or derived frame/error/exit shapes: startup and capability negotiation, requested/effective
configuration, input, permission denial, auth/stderr-only failures, partial/overflow output,
terminal response and stalled cancellation. Unsupported shapes reject rather than idealizing
successful startup/read-only/settlement; the production registry, scoped Store,
private input ports and Workflow consumer remain real. Every such evidence record
states native_execution=synthetic, the stand-in identity and contract-fixture source digest.
Native read-only enforcement and real transport/model acceptance remain unverified by these
peers and are mandatory #16 evidence; production runtime permission construction/denial can be
tested here. Declaration-only or
fixture-only adapter success cannot close these items. This proves deterministic
runtime plumbing, not real native-model acceptance reserved for #16.

Expansion has frozen per-slot request limit 1–64 (default 8) and cumulative byte
budget 1–8 MiB (default 1 MiB), with at most 1 MiB per returned artifact and 32 MiB
aggregate expansion bytes per round. Every expansion artifact/manifest counts
against the 128-MiB lineage quota. Exhaustion holds NeedsContext with recorded
count/bytes; no semantic truncation or context-pointer update hides it. Independent
expansion input retains the same guarded core/target/provenance exclusions.

Where a provider exposes read/tool traces, observed access to current-round peer
output or runtime-private DB/artifact paths is an independence violation holding
certification; record exact scoped trace evidence. Missing native read visibility
is explicitly unverifiable, never a proof that no read occurred. Real isolation
confirmation remains #16. Parallel remediation in another worktree is not run by
this engine: external fixing follows product-requirements §24's separate-worktree rule and
requires safe round settlement plus a verified committed target before new review.

17. Additional mandatory cases mapped to criteria 1–4 and 8–16:

- Two-reviewer matrix: QUICK/STANDARD non-author pair, explicit allow-self author
  plus one non-author, and STRICT non-author pair succeed when their mode settles;
  STRICT author plus one eligible slot rejects before input. Ordinary same-family
  eligible Sessions remain allowed under the frozen policy. Multi-family authorship
  rejects an insufficient Triple but permits an eligible custom STRICT roster.
- Early-stop cannot fire on author-only approval, below-floor counts or potential
  blockers. It preserves diagnostics/partial output and holds on partial findings
  until independent resolution. STRICT/security rejects enabling it.
- New concern-linked inspected verification authorizes only one autonomous confirmation round per lineage/tree. Sequential fresh inspections of different Low findings after nonblocking dissent reject a second automatic same-tree round; unrelated inspections and repeated rationale without new evidence reject. Two safely settled transient retry rounds consume
  lineage budgets; repeated causes/exhaustion/partial/Lost do not automatically retry.
- Missing/unresolvable/undeliverable required review/security instructions or skills
  reject before any member input. Delivery and native activation measurements
  remain distinct. A policy adding Low treats an unverified Low as a potential
  blocker rather than using the table's nonblocking-Low exception.
- Every per-member entry within the certificate lists prior structured claims seen, author/text hashes,
  trace-observed independence violations or native visibility unavailable. Test
  expansion request/byte/aggregate and lineage-quota boundaries with unchanged
  core/ContextVersion and retained evidence. The process stand-in flag accompanies
  actual registered-adapter two/quorum/Triple consumer fixtures.


Cumulative authorship conservatively includes Task document-drafting families as
well as implementation/fix families, including paths outside a phase's narrowly
reviewed slice. It does not silently narrow author exclusions by artifact. A
Claude document drafter plus Codex implementation author therefore leaves only
one eligible fixed Triple slot for STRICT; reject with named exclusions or use an
explicit eligible custom roster. Human authorship remains separately attributed.

18. Additional mandatory cases mapped to criteria 1, 4, 10, 14–16:

- An empty commit with identical tree stays same-target for retry authority;
  unrelated or unexplained whitespace deltas cannot trigger another autonomous
  sample after dissent. A checked formatting repair linked to the retained
  concern can authorize a changed-target round; prior opinions never carry over.
- Freeze and test startup/review/settlement deadline minima/maxima. After a stalled
  cancellation expires, status exposes exact uncertain member ownership and
  durable Human attention while the owned operation is still supervised and all
  permits/locks remain held. Later original-supervisor cleanup is retained as factual evidence but neither grants an approval nor releases the absorbing hold before trusted #14 recovery. Repeat with restart/broken supervisor continuity; ownership remains held in both cases.
- Custom two-reviewer and four-plus-reviewer rosters deliver distinct explicit
  model/effort per slot. Known effective mismatch fails only that slot without
  counting approval; require-verified-effective rejects unavailable measurements
  before inference (safe startup may occur to resolve them). Formal None rejects;
  advisory None records native defaults/unavailable facts separately. Result and
  certificate retain requested, reported and unavailable fields distinctly.
- Parallelism below roster size queues slots while preserving identical core
  hashes, Task context and runtime-input exclusions. Document-only plus code author
  families yield the stated named STRICT Triple rejection or custom-roster result.


19. Blocker fix-resolution remains independent: in 2-of-3, an executor fix claim
    plus check artifacts, finder diagnostic-only timeout and two general APPROVEs
    holds without certificate. Explicit eligible independent fix confirmation plus
    approvals can pass. Preserve original finding, fix claim and confirmation
    separately in the input/certificate; author or same-family alias cannot clear it.
    Under any/all/supersession no unresolved prior blocker disappears.
20. Default QUICK Triple with a document-drafting family and a different code-author
    family rejects self slots before input; explicit allow-self can collect their
    opinions, while only eligible non-author approvals meet the floor. A distinct
    agent alias of an author family follows the same rule. STRICT with one eligible
    slot rejects; a permitted custom non-author roster remains usable. After every
    1/2/4+ member round, including queued slots, Task.context_version and native owner
    currency have not changed per member; all share one context/core, and an attempted
    per-member context clone/bump rejects.
21. Before closing this Issue, closure evidence must quote/link the public acceptance
    entries (Issue bodies or linked requirements) that own the specific inherited
    obligations: #14 exact ReviewSet Lost/uncertain member ownership/locks/permits;
    #20 production deterministic two/Triple bundle/delta/expansion; #16 real native
    two/Triple isolation/results/efficiency plus timeout/retry/partial-output and settlement-expiry Lost hold
    frequencies, retained capacity/Task-time and Human-interruption impact. If a public owner does not carry
    its obligation, this Issue cannot close until that tracking is concrete. This
    requires traceability, not completion of #20 or a cyclic merge dependency.
    Status/Human attention must state that no in-runtime Human action can release
    a Lost hold before trusted #14 recovery.

Safe retry/timeout defaults are retained until real #16 measurements justify a
separately reviewed policy change. Native dogfood reports how often partial-output
cancellation, retry exhaustion and long reviews require Human attention; early
stop is expected to hold when cancelled members emitted unexamined content.
This does not invent an automatic weak-policy fallback to reduce interruptions.

Criterion 2 also rejects outvoting a potential Medium finding: two approvals plus
that dissent stay held, while safely settled Low-only dissent outside the blocking
set may be tolerated. Independent resolution is recorded before any certificate.

Criterion 8 excludes raw-output/transcript copies by recorded provenance as well as storage
location: known exact output-hash copies and paths declared by the scoped artifact policy are
excluded with a visible note, or hold before input if mandatory. A committed copy under
doc/verification does not become eligible merely because Git tracks it. Unknown content is not
claimed to be exhaustively recognized; native/source visibility limits are reported. Test
committed copies, explicit requests and symlink aliases.

Criterion 12/20 tests forged Git author/trailers: runtime-owned Codex dispatch/delta attribution
keeps Codex excluded despite a commit string naming Human/Grok. Uncovered delta bytes require
explicit recorded disposition before formal input. Criterion 9/10 class-wide verification
records the checked locations and N inspected / M changed, preserving scope and check artifact
provenance; synthetic facts do not claim exhaustive native inspection.

STRICT fixed Triple is unavailable when document drafting and implementation have contributed
two native families. Supported alternatives are an eligible custom roster meeting the same
floor, or explicit Human adjudication/termination; allow-self never supplies the missing
independent approval. #16 must exercise this availability limit alongside supported Triple
configurations.

Every disposition removing a potential/verified blocker from the effective veto
uses the same explicit Human/original eligible finder/two diverse eligible confirmer
rule. Fix, false-positive dismissal, severity downgrade, not-applicable, duplicate
and superseded-by-target-change are typed dispositions; unknown dispositions reject.
Actual source removal/checks are facts, but executor-only applicability conclusions
never clear the blocker. Criteria 9/19 test target-change applicability and duplicate
claims under all/quorum/any/supersession: unresolved independent confirmation holds.

Criterion 11 tests Human-authorized decomposition after lineage exhaustion with an
open High: descendants retain its veto, original base authors and shared exhausted
round/quota counters. A child certificate remains held; relabeling the frozen child
base or creating an autonomous Goal follow-up cannot restore eligibility/budget.

Criteria 1/14/18 prove actual parallel admission with production adapters and reviewed
synthetic contract peers: with M available resource shares and local parallelism M,
a causal peer barrier keeps every member unsettled until all M admissions occur.
Observed admission-before-any-settlement plus owned operation overlap is retained
as evidence for two and Triple rosters. Parallelism 1 reports serialized execution,
never parallel success; unavailable shares queue visibly without bypass.

Criterion 3 rejects automatic post-opinion all→quorum relaxation; explicit Human
relaxation above floors starts a new retained round. A frozen pre-admission conditional
policy is separately attested, not fabricated after dissent. Criterion 18 allows one
concern-linked inspection confirmation for APPROVE with a potential Medium blocker;
it still requires independent blocker resolution and rejects a second same-tree
autonomous confirmation. Criterion 21's #16 handoff must include settlement-expiry
Lost frequency and retained capacity/Task-time, not only inference timeout counts.
