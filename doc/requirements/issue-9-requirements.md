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
non-contradiction guarantee is invented. Raw peer findings
and executor conversation are hidden during an independent round. Later rounds
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
includes Critical and High and defaults to Critical/High/Medium; configuration
may add severities but cannot exempt Critical/High. Findings are proposals until
verification supplies actor identity (agent/Session or Human), exact target,
inspected/changed locations and actual evidence. No supervisor LLM invents facts.
An executor-only dismissal cannot erase a formal blocker: false-positive dismissal
requires explicit Human judgment OR later-round confirmation by the original
finder when non-executor, or by at least two policy-designated non-executor slots.
A confirmer must differ from the executor in registered agent identity and native
provider family; alias/same-agent permission cannot create dismissal independence.
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
| APPROVE, no unresolved or verified blocker | Counts once after safe settlement |
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

Policy bounds: 1–32 slots, 1–32 local parallel launches, 64 rounds per Set, 256
findings per result, 8192 UTF-8 bytes per finding and 1 MiB per factual bundle/result
envelope. Limits are simultaneous ceilings: maximum count does not promise every
maximum-size text fits the envelope. No semantic truncation occurs. A 128 MiB
aggregate Set artifact quota prevents unbounded retained output; exhaustion holds
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
Session cannot occupy two slots. Triple preset must not silently override a
self-review prohibition; incompatibility fails before input or requires explicit
policy selection. Formal policies and the triple preset default require explicit
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

Prove one reviewer, all-of-two, two-of-three, triple preset, any and a four-plus
roster through the real Store/Workflow delegation. Validate 0/32/33 slots, N=0/N>M,
non-quorum parameter rejection, timeout/quota limits and reopening. Show parallel
same-context consumption, equivalent core hashes, explicit specialization, hidden
peer output, self-review policy and exact native configuration. Exercise independent
coordinators, source/dirty/lock drift, Reviewer mutation and immutable active packs.

Decision-table tests include contradictory approval/blocker, executor-only false
positive rejection, independent dismissal, non-approving slot preserved, Human
hold, tolerated empty failure, partial/malformed/overflow blocker retention,
early safe cancellation and Lost ownership. Exercise verify/false-positive/fix/
commit/test/re-review history, new-round retry and quota exhaustion without reset.

Round N>1 must consume an actual Git-backed typed delta with baseline/new committed
targets,
physical/source hashes, prior verified unresolved facts and fix/check references.
Preserve provenance; reject stale, foreign or relabeled old bundles. This consumer
port is required before Issue9 closes; full production20 producer
readiness remains separately required for the MVP. Meaningful mutation controls, independent native
requirements/design/source review and exact-head Linux/macOS CI are required.
Synthetic adapters prove orchestration only. Real two/triple native outputs,
resource/safety/measurement evidence remain separately mandatory16 Goal proof.


At most one nonterminal Review Set owns a Task/Workflow phase lineage. A new
Set cannot approval-shop by resetting blockers, holds, history, round count or
artifact quota. Explicit Human/Workflow supersession records authority/reason and
inherits unresolved potential/verified blockers, adjudication and cumulative
limits across roster, policy and generation changes. Target change never erases
unresolved findings; verification must explicitly resolve applicability or fixes.
Lineage exhaustion can terminate without certificate or hold for explicit Task
scope decomposition; it cannot reset into a new passing Set. Decomposition never
certifies the old phase and must preserve unresolved obligations in Goal history.

Hold exits are explicit: ESCALATE/dispute requires authorized Human adjudication,
then a new full-roster round or termination without certificate; malformed/partial
output needs independently recorded inspection/resolution and new round; unknown
suffix requires new successful review rather than pretending it was examined;
quota/round exhaustion requires termination without certificate or explicit scope
decomposition; Lost remains held until trusted native14 cleanup. None of these
exits counts the prior failed/non-approving slot as approval.

Rule, policy, context/Workflow generation or required instruction change during a
round invalidates certificate publication. Findings may carry a typed upward risk
signal to Workflow; no automatic downgrade. Acceptance includes inherited-blocker
supersession, executor/provider-alias dismissal rejection, same-target full-roster
identity-delta rounds, cross-phase/generation replay rejection, context/risk drift,
queue attention and diagnostics-only versus partial-findings timeout/cancellation.
Real registered native adapter capability negotiation is a closing non-model test:
generic Execute-only CLI and unsupported read-only reviewers reject before input,
while verified read-only native declarations are accepted. This capability proof
does not substitute for actual two/triple native model outputs in16.
