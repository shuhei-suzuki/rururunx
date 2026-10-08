# Native Review Engine integration requirements

Risk: STRICT. Status: proposed requirements only; no implementation, formal-review
availability or native qualification is claimed. Requirements and design require
independent review before implementation. This delivery changes no production code.

## Authority and baseline

Source baseline: `ed3c1078a0f5a9646a8c4991881179887b57b28e`. Follow
[Agent execution requirements](agent-execution-requirements.md),
[Workflow](../design/master/workflow-engine.md),
[Context efficiency](../design/master/context-efficiency.md),
[Goal Runtime](../design/master/goal-runtime.md), and
[Issue #9](https://github.com/shuhei-suzuki/rururunx/issues/9), read on 2026-10-06.
The Issue names its detailed normative requirements at
[`0514473`](https://github.com/shuhei-suzuki/rururunx/blob/05144737514f30a90aa03a5d837554b34a4a7f49/doc/requirements/issue-9-requirements.md).
These requirements supplement that consumer contract for the new result-protection
profile; they do not close #9, #16, #20 or inherited acceptance obligations.

The latest user request supersedes conflicting custody/death-proof, live-executor
worktree locking, and cleanup-dependent ownership release prerequisites **for this
new profile**. Exact retained input, logical fencing and independent snapshots
replace them. Cleanup uncertainty alone neither rejects an otherwise valid opinion
nor blocks a fresh round; its uncertain resources remain quarantined. Unknown
review content, stale input, unsupported native read-only/instruction behavior,
and unknown irreversible effects still prevent acceptance. No current legacy gate
is weakened by writing this document. All other #9 policy, independent-floor,
finding-resolution, attribution, bounded lineage and public-evidence obligations
remain; unresolved obligations retain their existing IDs and stay unticked.

## Observed implementation gap

`workflow.rs` selects `Task.reviewers.first()` and has one `PhaseAttempt.unit`,
Session and agent. `prepare_managed` reserves one reviewer snapshot and
`adapter/native.rs` checks that single Workflow binding. A second independently
reserved reviewer cannot reuse it. `state/execution.rs::checked_workflow_binding`
requires an empty single unit slot. `ReadonlyCompletion` and its SQL consumer
close one successful reviewer only alongside one Workflow completion.

`SessionStatus.execution` carries `NativeStatus.result`, but Codex currently saves
the `turn/completed` turn object, not an accumulated assistant answer; Claude saves
the provider result envelope. Neither object is a Review verdict. Managed adapter
stdout is empty; parsing it or exit zero cannot produce review evidence. Native
result content is currently in watch state, so durable replay needs an owned receipt.
No `ReviewEngine` module or `WorkflowReservationGrant` type exists at this baseline;
the private implemented reservation type is `execution::model::WorkflowReservation`.
The new member grant must have a real producer and real adapter/Session consumers.

## R1. Review sets and immutable rounds

Support 1–32 uniquely keyed slots, including first-class two-reviewer and 4+
configurations; requested parallelism 1–32, effective concurrency bounded by actual
Runtime/Project/provider permits. A slot freezes registered alias/provider identity,
requested model/effort and specialization; each invocation has a fresh Session.
Defaults disallow duplicate agents and self review. Explicit allow-self never
makes an author eligible for the independent floor or blocker clearance. Cumulative
runtime-recorded delta authors include document authors/former executors; aliases,
Git author fields and new Set IDs cannot reset that exclusion. Unknown attribution
holds for supported deterministic evidence/trusted Human disposition.

Freeze completion `all`, `quorum(N)` or `any`, roster, security flag, blocking
severity set, author set, activated governing policy/rubric/instruction hashes,
Task/Workflow/Context/artifact identity and common factual core before any input.
Only quorum accepts N (1 <= N <= M). Critical/High/Medium always block; Low may
additionally block. QUICK/STANDARD need at least one eligible non-author approval;
STRICT needs two. Thus STRICT any still needs two; mode and floor both apply.
Triple is an all-of-three Claude/Codex/Grok preset, not a constant in aggregation.
Preserve #9's explicit author-visible variant and default author exclusion.
Absent supported Grok/member profile makes Triple unavailable, not a two-slot preset.
Counts above three stay supported policy shapes, with actual capability readiness
reported separately; refusal is not 4+/repeated-family acceptance evidence.

Freeze the actual registered native-channel qualification and its finite admission
envelope: source/key scope, load/write timing, queue delay, start skew and allowed
parallel/permit schedules (Issue9 I9-AC-8.k). Serialization is permitted only by
genuine SERIALIZED_CAPABLE profile evidence for the exact roster/view and original
qualified envelope. Best-effort concurrency or equivalent prompt bytes alone do
not qualify it. Before every queued member's actual input, revalidate elapsed
schedule, current permits and original qualification. An expired/out-of-envelope
schedule forbids further input and makes the whole round noncertifying; retain
already admitted owners' evidence and close their logical grants normally.
Any retry uses genuine full-roster retry authority, not a refreshed qualification
timestamp or within-round replacement.

A CONCURRENT_ONLY profile needs its complete qualified cohort's global/Project/
provider permits atomically before any member input. Queued cohorts hold zero
partial member permits; a wholly unadmitted queued Set owns no round worktree lock.
Use an actual private all-or-nothing cohort producer, not sequential per-member
leases or a capability label. If that producer/qualification is absent, refuse
before member preparation/native effects. A startup failure or admission-envelope
violation makes the whole round noncertifying, never silently serializable.
Capacity reservation does not guarantee physical native startup success. Queue
policy must be finite and compatible with the qualified native channel envelope.

## R2. Inputs, governing rules and independence

Every member receives byte-identical factual core and exact retained target/base,
immutable context and mandatory evidence. Additive specialization has a separately
recorded hash and may not omit mandatory rubric, rules, claims or blocking severities.
Fresh independent read-only Git snapshots/output/temp/resources belong to each
member. No executor worktree or another member's writable output is authoritative.
The declared review profile must support actual native read-only operation without
changing HOME, credentials, required hooks or native user settings. Do not add an
outer security sandbox. Unsupported source-writing hooks/instruction activation
refuse before member input; read-only chmod checks do not prove same-user containment.

Exclude raw executor transcripts, current-round peer findings/output and Runtime
state/sidecars from all Runtime-supplied inputs and expansions. Current results stay
hidden until the full roster logically closes. Prior-round unresolved claim text,
original slot/severity/hash, verified facts and attributed resolution claims are
delivered under the #9 history/exposure contract. Native visibility of same-user
files and auto-injected user settings is a limitation, not proof of blindness.

Activate governing review policy/rules independently of the reviewed author's
target: Task changes to policy/rules are proposals to review, not authority to
weaken their own review. Preserve explicit approved activation/digest provenance.
The current committed Workflow source frame supplies exact repository/config
bytes; it alone is not a ReviewSet activation, author-attribution or independent
bundle capability. Conflicting native auto-discovery that cannot deliver the
activated governing view refuses the formal profile. No live-rule fallback.

## R3. Actual native result and strict validation

Obtain final model answer through the actual registered provider protocol under
exact Unit/Session/native-turn identity. Preserve work success/failure/unknown,
quota/capacity wait and cleanup as separate observations. Native success, exit zero,
transport EOF, no findings, or an arbitrary JSON object is never APPROVE.

Use a versioned, strict Review result envelope. Require exact scope, set, round,
slot, generation, target artifact/SHA, core/policy/input digests; verdict APPROVE,
REQUEST_CHANGES or ESCALATE; typed findings and required prior-finding dispositions.
Reject duplicate object keys, unknown fields, missing fields, nonfinite numbers,
wrong identities, invalid locations/evidence references and ambiguous multiple
answers. A model's echoed identifiers only select a schema; actual native ownership
and frozen receipt comparison supply authority. No repair LLM or text-substring
fallback. Invalid/partial/overflow content never counts as a tolerated no-finding
failure; retain bounded acquisition evidence and hold for inspection/new round.

Persist the exact bounded acquired answer and native terminal receipt before watch
publication. Retain protocol-owned diagnostics separately, without credential/auth
streams. Provider extraction must distinguish model content from tool/transport
events and must not discard content merely because native terminal is failure.
Restart may parse a durable current receipt; it cannot reconstruct an answer from
the generic native turn object or manufacture provenance from caller JSON.

## R4. Opinions, blockers and verify/fix/re-review

Countable opinion requires the exact owned native WorkOutcome::Success terminal,
verified readonly snapshot completion, validated current receipt/envelope and
logical worker closure, AND verdict APPROVE. Independent floor counting additionally
requires eligible non-author identity. Failure/Unknown, cancellation, timeout or
lost invocation contributes zero mode/floor approvals even if its retained answer
is a well-formed APPROVE. Preserve such content/findings for inspection and blocker
resolution; content validity and counting are separate predicates. A potential
blocking finding vetoes certification until independently resolved; a verified
blocker vetoes all/quorum/any. REQUEST_CHANGES never becomes APPROVE because its
findings were dismissed. ESCALATE/disputed verification creates Human attention.
Failure, timeout, cancellation, partial output, malformed output and unknown work
remain distinct. Quorum/any may tolerate a diagnostic-only failure after review
inspection establishes there is no unexamined result; they do not hide blockers.
Default waits for all roster slots; this initial integration disables early stop.

Verify each proposed finding against exact retained target/evidence and actual
actor/location/check references. Do not turn executor fix/dismissal claims into
facts. Preserve the #9 clearance rules: trusted Human, eligible non-author original
finder in a later round, or predesignated eligible pair distinct in agent/family
from authors and each other. Contradictory eligible evidence holds for Human.
Explicit per-finding disposition is required; general approval clears nothing.
Trusted Human ingress is separate from native/Workflow/Broker JSON, not a claimed
biological-Human authenticator or same-UID security boundary.

A fix runs as a genuinely fresh Executor attempt, records bounded concern-linked
commit/delta and actual author attribution, captures/publishes a new artifact and
starts a new full-roster round with new snapshots/Sessions. Approvals never carry.
Same-tree retry uses an identity delta and concern-linked new evidence or explicit
trusted Human authority; no automatic repeated sampling. Preserve shared 64-round,
retention and confirmation/transient retry lineage limits across supersession and
decomposition. Within-round native re-launch/retry is disabled. A still-live native
quota retry is the same invocation and receives no duplicate input.

## R5. Durable member authority and atomic publication

Use indexed Set/round/member state with unique identity and exact foreign keys,
immutable input/result/history rows, CAS and append-only audit. Generic Review
Records/metadata are diagnostics, not execution or certificate authority. Register
round and member before their preparation effects, bind each Unit once, and register its
Session before native input. Concurrent members do not occupy the single legacy
PhaseAttempt.unit or increment Task.context_version per reviewer.
Bundle object inspection before round activation uses separately registered
current-Runtime retained-read operations; it grants no reviewer/Session/input
authority and never performs unregistered Git or live executor filesystem reads.

New member reservation grants must be minted from actual Workflow reservation,
frozen round, current owner epoch, exact artifact and actual bundle provenance.
Adapter start, version/helpers, quota admission, Session registration and native
input dispatch revalidate member authority, preventing public ManagedInput or a
Task hint from bypassing it. Cancellation/replacement fences every member before
native stop; stale/foreign callback results remain historical diagnostics only.
No SQL transaction spans native/Git awaits. No duplicate member spawn/input from
two coordinators or an ambiguous dispatch acknowledgement.

Persist result inspection and logical member finalization separately from round
certificate. Aggregate publication atomically compares Task/Workflow/Context,
Project/Goal/governing digests, generation/epoch, full artifact snapshot, policy,
roster/core/member results and verification dispositions; writes certificate,
Workflow completion, Task projection and next Context together. Failure rolls
back all. Replace the singular Session evidence assumption with exact multi-member
certificate authority; never pick the first Session as the collective attester.
Goal advancement consumes this accepted Workflow evidence, not member counts.

## R6. Capacity, cancellation and recovery

Use existing durable quota pools/leases/probe admission and configured caps per
member. Wait does not mean rejection or work failure. Quota notifications affect
only their exact owned account/bucket/member, not siblings' verdicts. Live retry
preserves Unit/Session/input; terminal quota interruption closes that invocation
without an opinion, then uses a fresh full-roster round after bounded recovery
and authorized retry. Never launch a duplicate within the same round or silently
remove/switch reviewers to fit capacity. Unknown capacity uses bounded concurrency
and cooldown; auth/transport/resource wait is not subscription exhaustion.

All live native slots remain individually visible. Task WaitingQuota is a derived
projection only when no member is runnable/running and unresolved members are
quota-waiting; mixed running/waiting review remains Reviewing. Cancellation is
possible from every wait. Logical close and late cleanup update only exact
historical members. Known accepted review/evidence survives cleanup failure.
Restart fences previous epoch/dispatch authority, retains results and unknown
operations, and requires exact recovery or a fresh authorized round, not Session
reattachment from a cached PID. Resource quarantine cannot become blocker clearance.

## R7. Bounds, migration and honest acceptance

Preserve #9's 1 MiB complete prepared member frame AND complete acquired result,
256 findings/result, 8192 encoded bytes/complete finding entry, 64 KiB diagnostics,
bounded expansions and cumulative lineage limits. Account for actual retained
core/member/frame/row/receipt/verification bytes and provider overhead before input;
the 128 MiB retained allowance is not a token limit. Missing calculable bounds
refuse the affected profile. No mandatory semantic truncation. Startup, opinion,
queue/capacity and logical-close deadlines are named and finite, with quota
waiting charged/reported separately. Do not promise a physical cleanup deadline.

Schema migration is STRICT, ordered/transactional, preserves prior execution,
Session, Workflow, Context and audit history, and fences incompatible open/reopened
old writers. Legacy single-review evidence does not become a new certificate.
Current public readiness stays unavailable until all actual producer/consumer
guards compose and independent source review passes. Official native qualification,
#16 representative two/Triple runs, #20 producer obligations and latest user's
two-OS four-Task scenarios remain distinct acceptance evidence. Fixtures do not
qualify authentication, hooks, settings, read-only enforcement or profile independence.

## Required controls and reporting

| Area | Required actual consumer control |
| --- | --- |
| Roster/policy | 1, all-of-2, 2-of-3, any plus STRICT floor, 4+ and repeated-family shapes; Triple default author refusal and explicit author-visible handling; unsupported Grok reported. |
| Native result | Claude owned final text and Codex owned agentMessage plus exact turn terminal; exit-zero missing answer, forged identity, duplicate keys, malformed/partial/overflow, auth/transport failure never approve. |
| Independence | Byte-identical core, additive focus, peer/output/transcript exclusion, prior exposure; governing rule edits remain proposals; actual conflicting auto-discovery refuses. |
| Authority | Concurrent start/CAS, public adapter/Session/input bypass attempts, stale epoch/round/artifact/Context/Task, sibling result and wait isolation, failure rollback, reopen old writers. |
| Qualified admission | Finite-window serialized-capable profile delays second input beyond its envelope: zero second input/certificate; concurrent-only absent capacity: zero input/partial permits; two 2-member cohorts competing for capacity3: one gets2, the other0, then progresses on returned permits. Unqualified serialized/cohort producer refuses before effects. |
| Opinion counting | Owned native Failure/Unknown with syntactically valid APPROVE contributes zero mode/floor votes; failure with a potential Medium finding preserves inspection/veto rather than tolerated no-finding failure. Successful current readonly receipt is the separate positive. |
| Resolution | Potential Medium veto in quorum/any, required dispositions, self/alias author refusal, disputed clearance, trusted Human route, fresh fix/re-review, bounded same-tree retries and inherited budgets. |
| Snapshot/result | Four concurrent readonly members, cancel one, mutate executor after capture, distinct snapshot/output resources, retained graph loss refuses, approved artifact/commit/evidence unchanged. |
| Quota/recovery | Live wait/recovery same invocation, terminal wait fresh full round, all-wait vs mixed Task projection, durable result-before-crash, ambiguous dispatch no replay, cleanup backlog independent. |

Report source-qualified, fixture-qualified, and actual official-native-qualified
behavior separately with exact commits, OS/profile/evidence and known limits.
Requirements/design delivery ends with independent review and human decision;
no source implementation, native account use, Issue closure or README claim follows
from approving these documents.
