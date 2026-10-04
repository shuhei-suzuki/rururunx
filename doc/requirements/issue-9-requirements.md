# Issue 9 requirements: configurable independent review

Status: proposed requirements. Merged dependencies2/4/8 supply state, adapters and
Workflow. Core production work has not started. Prepared-input integration depends
on reviewed19, provider typed-input contracts and the record-only binding port43.
Review Bundle/delta construction20 depends on this engine. Issue9 must prove the typed consumer with actual Git-backed producer fixtures; full20
producer and native16 integration remain explicit MVP gates, avoiding a dependency
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
Session. All slots in a round consume the same Task context/locked revision
concurrently, without incrementing Task.context_version or owner currency per
reviewer. A reviewed explicit ReviewSet delegation owns the entire roster;
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
non-contradiction guarantee is invented. rrx never injects raw peer findings
or executor conversation into a member input during an independent round. Peer
outputs/runtime state stay outside the reviewed source and bundle paths. Native
read-only permissions are retained: the runtime does not claim OS isolation from
same-user-readable runtime files. Certificates state this runtime-input independence
scope and residual filesystem visibility; native16 acceptance checks the actual
input/storage paths and observed independence. Later rounds
may include verified defects/fixes as factual evidence and dismissal adjudication
history with actor identity. Dismissal claims/rationales remain labelled claims,
not core facts; re-raising a dismissed finding remains possible.

## Verdicts, verification and completion

Slots return APPROVE, REQUEST_CHANGES or ESCALATE, plus typed findings. Native
failure, timeout, malformed output, cancellation and Lost are separate outcomes.
Only completed, well-formed, exact-target APPROVE counts. REQUEST_CHANGES never
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
finder when non-executor, or by at least two policy-designated non-executor slots.
Each confirmer must differ from every recorded delta author in registered agent
identity and native provider family, and a two-confirmer pair must also differ from
each other in both registered agent identity and native provider family; alias/same-agent permission cannot create dismissal independence.
Inputs separate original finding and actual repository evidence from the labelled
executor dismissal claim, never presenting that claim as fact. Disputed evidence holds for Human.
A verified fix is recorded with commit/check evidence and requires a new round.
Original outputs, verdicts and verification history are never rewritten.

Completion modes are all (M approvals), quorum (N approvals, 1 <= N <= M), and any
(one approval). Only quorum accepts a quorum parameter. Verified blockers veto
every mode, and unresolved potential blockers prevent a certificate. A
certificate is the immutable Review-Set-satisfied result binding exact Scope,
Workflow phase/generation/active claim and required review-instruction/skill
digest, round/target,
core/specialization hashes, policy/roster, context provenance, individual settled
outcomes and verification/evidence references. It is evidence, not merge authority.

| Slot result | Counting and round consequence |
| --- | --- |
| APPROVE, no unresolved or verified blocker (including unverified Low) | Counts once after safe settlement; Low remains recorded |
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
classified as harmless progress. Numerical modes are equivalent at boundaries:
any = quorum1, all = quorumM; they differ in required approval count.
Default completion waits for every slot to settle. An explicit early-stop policy
may cancel remaining work after numerical eligibility, retaining all partial
output and applying the same blocker verification and safe cleanup conditions.
STRICT/security review disables early stop. Other explicitly enabled early stop
never permits discarding inconvenient findings. Within-round retry is disabled in Core: a retry needs a new immutable
round, includes prior failure/attempt history and consumes the round limit.
Each new round reruns the full frozen roster; approvals never carry across rounds
or changed targets. Same-target retry/dismissal rounds consume a typed identity
delta (baseline == new) with prior-round provenance; changed-target rounds consume
an actual revision delta. Certificates contain outcomes from that exact round.

Lost/uncertain rounds surface a durable Human attention item with exact ownership
and blocker evidence. Trusted native recovery14 must produce authoritative
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
Timeout is 1–3600 seconds per slot after native admission, default600, frozen in
policy. Resource-queue wait is a separate visible state with 1–3600-second
attention threshold, default600; exceeding it holds for Human without pretending
a model timed out. Existing Runtime,
Project and agent resource limits can lower local parallelism, never be bypassed.

Default formal policy forbids the Task executor agent as reviewer and duplicate
agent identities. Explicit policy may allow same-agent independent Sessions;
that permission and provider/model diversity are visible in the result. A single
Session cannot occupy two slots. The Triple preset ships the Claude/Codex/Grok roster and all-of-three policy,
without an implicit self-review exception. With an MVP executor in that roster,
default selection rejects before input. An explicit visible allow-self independent-
Session policy makes that same preset usable; the author slot is never eligible
as a dismissal/downgrade confirmer. Acceptance exercises both default rejection
and actual supported explicit-policy execution, rather than calling the unresolved
default a working configuration. Formal policies and the triple preset default require explicit
resolved model/
effort configuration before input; unresolved None rejects. An explicit advisory
policy can inherit native defaults but cannot silently replace formal settings.
Record requested, native-reported and unavailable values separately; optional
require-verified-effective policy rejects unavailable native measurements.
Report actual provider model/effort when exposed, otherwise an explicit reason. A known mismatch to requested Some fails the slot;
Advisory requested None may bind the native default without an invented
measurement; formal policy does not silently inherit it.

Non-scope: Review Bundle/delta construction20, Approval Broker10, general trusted
recovery14, global scheduler/recovery14/27, complete CLI/TUI15 and actual native
dogfood16. This engine integrates their typed evidence/ownership contracts rather
than fabricating them. Reviewer scheduling obeys available resource authority.
Native two/triple dogfood is mandatory for the MVP Goal in16; Issue9 core can prove
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
scope decomposition; it cannot reset into a new passing Set. Decomposition never
certifies the old phase and must preserve unresolved obligations in Goal history.

Hold exits are explicit: persistent nonblocking REQUEST_CHANGES or too few approvals
requires an authorized Human/Workflow choice of a new full-roster round, explicit
policy relaxation above the frozen mandatory floor, or termination without a
certificate. A relaxed policy starts a new round or explicitly superseding Set,
retaining prior opinions, obligations and lineage limits; it never manufactures an
APPROVE. ESCALATE/dispute requires authorized Human adjudication,
then a new full-roster round or termination without certificate; malformed/partial
output needs independently recorded inspection/resolution and new round; unknown
suffix requires new successful review rather than pretending it was examined;
quota/round exhaustion requires termination without certificate or explicit scope
decomposition; Lost remains held until trusted native14 cleanup. None of these
exits counts the prior failed/non-approving slot as approval.

Rule, policy, context/Workflow generation or required instruction change during a
round invalidates certificate publication. Findings may carry a typed upward risk
signal to Workflow; no automatic downgrade.

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
all recorded agents/native provider families that modified the reviewed Task delta,
not only the current Task.executor. Unknown authorship is surfaced and requires
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
restart/native recovery14 proves cleanup. GitHub irreversible external outcome
reconciliation13 is distinct. These future ports are not fabricated by this engine.

APPROVE with only nonblocking Low findings may count even before verification;
those findings remain recorded and can be raised to a blocker on actual evidence.
REQUEST_CHANGES with
empty or nonblocking findings never counts. This preserves the slot's opinion
while quorum/any tolerate explicit nonapproval; mode never removes blocker checks.
Independent partial/malformed inspection uses the same non-executor confirmer or
Human eligibility as dismissal. Mandatory core/specialization overflow rejects
before model input with actual size/hash/NeedsContext evidence and Human attention;
no semantic truncation or success through a byte cap is permitted.

Policy minimums are explicit resolved inputs, frozen before round admission. Core
baseline requires at least one approval for QUICK/STANDARD and two for STRICT,
with distinct reviewer identities/provider families for those two, Critical/High/
Medium blocking, clean/read-only/lock safeguards and explicit reviewer config.
Runtime/Project configuration may strengthen this floor; implicit missing policy
or unsupported configuration rejects. The preset can raise count/diversity but
cannot weaken class minimums or reset lineage requirements.

All Lost/uncertain member reservations continue to consume their actual provider,
Project and runtime resource shares and remain visible in status. Unrelated Tasks
can use remaining permits; exhaustion is reported as held capacity, never silently
ignored or bypassed. No operator release exists before trusted native14 recovery.
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

These are the closing criteria. Items1–8 map to the public Issue9 checkboxes;
items9–16 are explicit added integrity/availability requirements. Every criterion
needs actual consumer evidence; proposed tests and independent development reviews
are not runtime proof.

1. First-class two-reviewer and one/four-plus rosters run through real Store/Workflow
   delegation, concurrently using one fixed Task context, exact locked revision and
   independent member ownership. Reject0/33 slots, invalid parallelism and duplicate
   Session ownership.
2. Two-of-three quorum counts exact-round APPROVE only; rejectN=0/N>M and quorum
   parameters on all/any. All, quorumM and any/quorum1 have equivalent boundary
   outcomes and retain every nonapproval/blocker.
3. All-of-two requires both approvals. Exercise empty/nonblocking REQUEST_CHANGES,
   persistent nonapproval and audited Human relaxation/new-round or termination
   without fabricated approval.
4. All-of-three Triple preset resolves actual registered production Claude5/Codex6/
   Grok7 read-only declarations and supported explicit model/effort configurations.
   With each MVP executor-in-roster, prove default self-review rejection and explicit
   independent-Session permission success. Unsupported/unregistered configurations
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
8. Prove identical mandatory core hashes, additive specialization/expansion, no peer
   output in runtime-controlled inputs and storage outside source/bundle paths.
   Changed and same-target rounds consume actual Git-backed typed revision/identity
   deltas with prior unresolved facts and fix/check provenance; stale/foreign/relabelled
   bundles reject. Production20 producer and representative native16 independence/
   result/efficiency dogfood remain separate mandatory MVP gates.
9. APPROVE with a potential blocker holds; verified blockers veto every completion
   mode. Preserve unverified nonblocking Low and immutable original severity.
   Executor-only dismissal/downgrade and two same-agent/provider-family confirmers
   reject. Independent eligible confirmation or explicit Human judgment records
   evidence; verified fix/commit/checks requires a new full-roster round.
10. Same-target retry and changed-target remediation rerun the full frozen roster,
    retain prior outcomes and never carry approvals. Within-round retry is absent.
    Verify/fix/commit/test/re-review and repeated Human-resolved ESCALATE preserve
    adjudication as labelled history, not model APPROVE.
11. Supersession retains unresolved obligations and lineage floors across executor,
    roster, policy and generation changes. Cumulative64-round/128-MiB limits never
    reset: supersession at round63 permits only one further round. Scope decomposition
    preserves Goal obligations and cannot certify the prior phase.
12. Class/Project policy floors reject missing/weaker policies (including blocking
    {Critical,High} without Medium), early-stop under STRICT/security and absent
    required configuration. Pairwise independence/delta-author exclusion and unknown
    authorship decisions are visible; same-agent permission does not authorize
    dismissal by an author.
13. Certificate binds exact phase/generation/claim, target/source/core/member hashes,
    mandatory instruction/skill/context/checkpoint provenance and settled evidence.
    Source/rules/policy/risk/context drift and cross-phase replay cannot pass.
    Certificate is not merge permission.
14. Bounds0/1/32/33,256 findings,8192-byte text,1-MiB envelopes, mandatory overflow,
    quota/round exhaustion, timeout1–3600 and queue-attention exits retain evidence
    and fail visibly without semantic truncation or hidden resource bypass.
15. Lost retains actual resource shares and native locks even after Set termination
    without certificate. Remaining permits still serve unrelated Projects/Tasks;
    exhausted capacity is explicitly reported. Reviewer-mutation, drift, NeedsContext
    and queue holds require their recorded authorized exits. No fictitious cleanup.
16. Actual reviewed19/43 shared typed-input/member delegation and native5/6/7 caller
    contracts compose with the real Workflow consumer. Incompatible old writers,
    generic history/JSON authority fabrication and stale concurrent updates reject
    without evidence loss. Meaningful boundary mutants, independent requirements/
    design/source gates and exact-head Linux/macOS CI complete core verification.
    Real two/triple model results are separately required by16; actual declarations/
    supported preset consumer proof are required here.
