# Owned native answer acquisition and durable receipts

Status: proposed STRICT implementation supplement; documentation only. Actual
source baseline `37c5cc434d8f9c46d4a0f5f86a0285421a773899`. The approved
[ReviewEngine requirements](../requirements/review-engine-integration-requirements.md)
and [design](review-engine-integration-design.md) already require this component.
This supplement makes their acquisition/persistence interfaces concrete; it does
not reopen roster policy or qualify an implemented ReviewEngine, native account,
four-Task operation, OS compatibility or final Phase2.

## 1. Contract mapping and current gap

| Approved obligation | Actual baseline / proposed connection |
| --- | --- |
| R2 exact immutable input and mandatory source identity | `NativeSessions::start_inner`, `ManagedInput`, actual PreparedInput and native-input effect registration supply the frozen invocation binding. No model-echoed identifiers mint it. |
| R3 actual owned answer, strict acquisition, durable-before-watch | Codex currently returns only `turn/completed.params.turn` (`native.rs:1149–1170`), while Claude returns an owned result envelope. Add provider collectors and typed receipt persistence; project `NativeStatus.result` from the saved receipt. |
| R4 native failure content is not approval | Work/disposition, acquisition validity and content are independent fields; Failure plus valid APPROVE text remains Failure and zero approval authority. |
| Design §6 exact answer/terminal transaction and draft retention | Replace the separate work-terminal and Session-close writes in `Core::run` with one typed native-completion transaction and a fenced diagnostic-only append path. |
| Approved result-protection policy | Preserve known work separately from cleanup, logical currency, fresh attempts and independent retained artifacts. A receipt does not reopen grants or certify process death. |

Implement under `execution/native_result.rs` (private producer/collector proofs),
provider protocol bridges and `state/execution/native_results.rs` (typed SQL).
`execution/native.rs` owns the actual Core/stdio paths; adapter vtables consume
their normal managed status projection. The legacy custody adapter remains
unchanged. Review round/member grants, strict review-envelope parsing, readonly
completion, certificate counting and full-round Workflow publication are later
components; acquisition is useful for every managed native role and is never an
approval proof. No broad fixture PhaseGate may consume raw answer JSON as success.

## 2. Protocol evidence and supported acquisition profiles

The existing Codex bridge admits only `codex-cli 0.160.0`; retain that check. On
2026-10-06, the installed `/opt/homebrew/bin/codex --version` reported that version.
The account-free command `codex app-server generate-json-schema --experimental
--out /private/tmp/rrx-native-results-schema-0160` produced `ServerNotification.json`
(208255 bytes, SHA256
`28a42039eee1c3f45c92b6cf07111bacf6de2716f4d3365ff53ad65507fef00c`), byte-identical
to the earlier saved `/private/tmp/rrx-phase2-codex-schema` schema. Its definitions
include ItemCompletedNotification `{threadId,turnId,item,completedAtMs}`,
agentMessage `{id,text,phase?,delivery?,questions?,memoryCitation?}`, MessagePhase
`commentary|final_answer`, and terminal Turn `{id,status,items,itemsView?}`.
`delivery=async` is a separate delivery mode; absent/null phase is explicitly
unknown, not evidence of final-answer semantics. These are local generated schema
observations, not authenticated native output conformance. Temporary files are
not required at install/run time; commit bounded representative shapes and their
provenance with the source controls, without generated dependency/license files.

The [official app-server documentation](https://developers.openai.com/codex/app-server/)
describes completed items as final item state, agentMessage text with optional
phase, and commentary/final-answer values. Its moving version is supporting
context; the exact local schema/version, owned wire producer and actual controls
govern this profile. The public tag/source URL was unavailable during this task;
no fetched tag authenticity or live model behavior is inferred from its name.

The [official Claude structured-output documentation](https://code.claude.com/docs/en/agent-sdk/structured-outputs)
places structured_output in the result message when structured output is requested.
This does not establish equality with free-form result text or pin CLI 2.1.283.
Preserve the current versioned `claude_wire::RunState` identity, origin and
background/final-idle qualification; new extraction controls must exercise the
actual Claude managed path. This component initially qualifies text acquisition;
structured-output content may be retained as a separate bounded representation,
but cannot silently replace text or count as a validated Review envelope. A later
explicit structured-only profile needs its own duplicate-key/exact-wire controls.

### 2.1. Codex collector

Create a collector only from the private invocation acknowledged by the actual
`turn/start` RPC: owned thread ID from thread/start, returned turn ID, immutable
launch binding and registered input operation. It consumes only exact matching
threadId AND turnId notifications; missing/foreign identity cannot populate it.
RPC boot queues are replayed only after that acknowledgment, under the same bounds.

Only `item/completed` with `item.type=agentMessage` supplies a completed candidate.
Require bounded nonempty item ID, text string and supported phase fields; async
delivery/questions do not qualify an ordinary final answer. Commentary and tool,
reasoning, plan, review-mode and user events never supply final content. Deltas may
provide bounded draft evidence, keyed by exact thread/turn/item, but completed
text is authoritative and is not reconstructed by concatenating arbitrary deltas.
The initial exact profile requires one completed `phase=final_answer` candidate.
Null/absent phase remains Unsupported/Ambiguous acquisition; do not choose the
last unknown-phase message. This may preserve native work Success with unusable
answer, and formal review remains held. Supporting legacy unknown-phase models
requires a separately qualified extraction profile; changing native work to
Failure merely because acquisition is unsupported is prohibited.

Keep a bounded item-ID identity/digest inventory. Identical completed redelivery
is idempotent. Same ID with changed text/phase/type, duplicate final candidates
with different IDs, malformed owned candidate or contradictory terminal item
content poisons acquisition and retains bounded diagnostic content. Do not merge
multiple final answers or repair them. Matching commentary before one final is
allowed. After an ambiguous/overflow flag, later well-formed text cannot clear it.

On exact `turn/completed` bind to that turn's native status. Terminal full items
may corroborate previously observed completed candidates; they never mint an
unobserved final item. Where itemsView is partial/notLoaded, absence from the
terminal list does not retract an observed candidate. A full view containing the
same final ID with changed content or extra final candidates is ambiguous.
Do not require terminal.items to contain all observed items when the pinned
schema permits unloaded views. A matched failed/interrupted terminal preserves
acquired answer/draft and the existing quota/capacity/auth classification. Missing
or premature terminal, EOF or malformed protocol yields Unknown acquisition/work
under existing terminal semantics, never an approval.

### 2.2. Claude collector

Consume only the current owned Session's result accepted by RunState, with
supported ordinary input origin and the existing background/final-idle conditions.
Retain accepted failure-result content before the unsuccessful terminal branch
returns. Authentication error text, errors arrays, tool output and stderr are
protocol diagnostics, not model answers, and must not be persisted as answer
content. An owned ordinary result's textual `result` is the text candidate when
its native subtype/profile identifies it as model content; unsupported/error-only
representations remain named unavailable diagnostics. This requires explicit
fixture controls for error_during_execution with model content versus auth/error
diagnostics, rather than blanket retention of all result strings.

No assistant/subagent/background/channel transcript is concatenated into a final
answer. Identical repeat result is idempotent; changed or multiple ordinary final
representations are ambiguous. A simultaneous nonempty text and structured_output
is retained as two representations with Ambiguous/Unsupported acquisition, unless
a separately qualified profile supplies an exact equivalence rule. No preference
for whichever representation parses as APPROVE. Raw wire duplicate-key checks
precede Value conversion where structured content could later be interpreted;
Value reserialization cannot prove original duplicate-key absence.

## 3. Bounded acquisition and privacy

Initial finite limits: native wire frames retain their existing 4MiB ceiling
(including framing); bound before JSON allocation. Keep at most 256 observed
answer-related item identities, 1024 relevant candidate/delta events and 1MiB
decoded answer bytes per invocation. Completed response parsing has depth32 and
finite provider field/ID bounds. At overflow stop collecting/stop the owned native
invocation best effort; classify acquisition Overflow and do not keep reading an
unbounded suffix to discover its full length. Preserve any already known native
terminal separately; if unread, work remains Unknown. Do not discard a previously
known Success because hygiene or result persistence later fails.

Receipt complete encoded body is at most 2MiB, including all metadata/source
maps/representations/JSON escaping; maxima need not fit simultaneously. Complete
content must fit both decoded and encoded limits. Invalid/partial/overflow paths
retain at most 64KiB total safe model-content UTF-8 prefix (trim incomplete scalar),
prefix SHA256, exact observed bytes or explicit lower bound, unseen_suffix=true
and bounded reason codes. Never hash a prefix and label it the full answer hash.
For oversized/malformed raw wire frames retain only bounded length/hash/category
observations, not raw prefixes containing credentials or transport payloads.
Raw auth/RPC errors/stderr/credential streams stay unpersisted. Model content may
itself contain sensitive user output: use the protected state-store boundary and
do not mirror it into generic events, logs or public status by default. Receipt
IDs, validity, hashes and finite diagnostics suffice for ordinary status.

## 4. Frozen invocation and SQL schema6

Root assigned this component **schema6**. Subsequent coordination assigns
Published-frame recovery schema7, first Verifier delivery schema8 and future
ReviewRound schema9; Goal/Runtime uses the next coordinated migration. The original independently reviewed supplement
named ReviewRound7 before those dependencies were identified. Update SCHEMA_VERSION,
fresh install and ordered migration5→6 in one STRICT source delivery. No parallel reuse of6.

| Table | Identity and ownership |
| --- | --- |
| native_invocations | Private invocation UUID, UNIQUE Unit and UNIQUE Session; exact Project/Goal/Task scope, generation/owner epoch/provider, native profile/version, initial Unit/Session versions, launch Context identity/version/hash, complete source_versions map/digest, target/base SHA and optional artifact ID/version, exact PreparedInput payload hash, outgoing frame hash/input effect ID when actually reserved, native thread/session and turn IDs when acknowledged, lifecycle/version. |
| native_results | UNIQUE invocation FK; immutable receipt UUID/version1, same indexed scope/Unit/Session/generation/epoch/provider/thread/turn binding, acquisition status (Complete/Missing/Partial/Ambiguous/Unsupported/Overflow), bounded answer representations/prefix evidence, normalized native terminal WorkOutcome/disposition/failure category, authority classification (OwnedTerminal/HistoricalDraft), terminal provenance/hash and capture time. |

Invocation metadata is capped64KiB complete encoded; source maps retain existing
128-entry/name/value limits. Identity/body/index equality, positive checked i64
versions, FK ownership, immutable content triggers and unique receipt constraints
apply before reads/writes. SQL length/count projections bound decoding; scoped
paginated history explicitly reports completeness. One invocation per current
Session corresponds to the baseline one-turn-per-managed-Session path. No new
continuation/multi-turn API is implied. Future multi-turn changes require ordinal
and admission design; existing willRetry continuation stays the same invocation.

Private `OwnedInvocation` is created from the actual admitted Core, current exact
Session/Unit and PreparedInput checks. Its initial row is registered in the SAME
Immediate Session-registration transaction before child spawn, with input state
NotDispatched. This covers bootstrap/spawn failure without fabricating an input
effect or model answer. Fill the input effect ID and outgoing frame hash in the
SAME Immediate native_input intent transaction before stdin effects; persist current Context body hash when
managed Workflow Context exists, and explicit standalone input identity otherwise.
Standalone input is not mislabelled a durable Workflow Context. NotDispatched
terminals receive Missing acquisition and no turn/answer authority. Link actual
operation/frame hashes rather than the model's echoed input. A pending invocation
cannot be rebuilt into send authority from a row. Actual successful turn/start
reply then atomically acknowledges thread/turn under exact invocation/current
Unit/Session/epoch CAS before collector acceptance. Claude uses its actual issued
session ID and an internal one-shot input token; missing native turn ID is explicit,
not a fabricated provider turn ID. Origin rules remain the provider-owned check.

Collectors create non-Clone/private-field `OwnedNativeCompletion` or bounded
`HistoricalNativeDraft` proofs, not serializable caller capabilities. Public DTOs,
generic Record JSON, TaskTool IPC or raw SQL body labels cannot invoke those
mutation ports. Add both tables to all writer guards; reinstall existing tables'
guards against version6 so already-open schema5 writers with their connection-
local function returning5 fail INSERT/UPDATE/DELETE. Schema5 binaries reject new
open6. Refused migration rolls back tables/guards/version/audit together and never
labels old unpersisted NativeStatus.result as an authoritative receipt.

## 5. Native terminal, cancellation and watch transaction

Introduce `Store::complete_native_invocation(proof)` using Immediate, factoring
existing finish_execution and close_execution_session internals into TX helpers.
Recheck exact invocation/binding/version, source/Context input pins, current
Runtime epoch, Unit/Session identity/current semantic authority and original
launch grant. Quota telemetry may legitimately advance Unit/Session versions;
the Core updates exact observed versions through those existing typed transitions,
without refreshing an unrelated source/Task/Workflow authority. Store does not
guess identity from the Unit's latest Session after await.

In one transaction: insert immutable receipt; record native known work/disposition;
close native effects; preserve result_finalization when applicable; close exact
Session terminal; release its quota lease/waiter using existing rules; enqueue
cleanup and append bounded noncontent audit. Owned Success remains Success with
Missing/Unsupported answer. Failure with Complete APPROVE text remains Failure.
The receipt neither publishes an artifact nor completes Workflow/Task or review.
Cleanup runs afterwards and cannot alter receipt work/content. Watch terminal
publication occurs only after this transaction, reads the saved typed receipt and
publishes its ID/validity plus bounded content only through the intended result
accessor. Add a typed immutable receipt-reference field to NativeStatus.
`NativeStatus.result` becomes a compatibility projection with versioned shape
`{schema: "native_answer_v1", receipt_id, acquisition, authority_class,
work, disposition, text?, structured_output?, prefix_evidence?}` read from that
receipt, not a second mutable truth or opaque turn object. It may carry bounded
failure content; every future consumer still checks receipt/native/snapshot
authority. Public logs/status render only the reference and diagnostics by default.

Cancellation/replacement/epoch race has one winner. If authority is already fenced,
use a separate historical-only append checking the original immutable invocation,
exact original Session/Unit/native identity and private live producer proof. Store
current epoch is used only to write history, never to reauthorize old work. Mark
HistoricalDraft, keep acquired content and observed native outcome separately from
authoritative Unit work; do not replace Cancelled/Lost, reopen finalization, mutate
Task/Workflow, release another lease or publish completion. Newer attempt remains
unaffected. Identical receipt replay is idempotent; changed content/second terminal
for the same invocation refuses without replacing the original. A conflicting
late terminal may add a finite diagnostic event, never overwrite the receipt.

Persistence failure publishes no authoritative result/terminal. Preserve actual
owned supervisor/guard and bounded acquisition state, report a finite persistence
attention, and attempt exact conservative retirement without claiming saved
content. No watch success may race ahead of receipt commit. If restart/crash loses
uncommitted in-memory content, classify Missing/Unknown truthfully. Core Drop
preserves its current terminal/draft behavior; it must never mint Complete or native
Success. Bounded owned cleanup and resource quarantine remain independent.

Restart reads exact saved receipts by immutable invocation identity under current
Runtime read authority; checks complete body/index/source/terminal hashes and
classification. HistoricalDraft remains inspection-only. It cannot reconstruct
native handles, original send grants, readonly proof, member identity, ReviewSet,
round opinion or artifact publication. A future ReviewEngine requires all its
independent current proof predicates in addition to this receipt.

## 6. Source controls and remaining limits

Source ownership will be coordinated with Root after independent supplement
review. No source/API promised here exists yet. Required account-free controls:

| Control | Actual consumer / refusal |
| --- | --- |
| Codex commentary → exact final item → exact terminal | Managed NativeSessions actual stdio/vtable returns acquired final text; separately reopened Store receipt equals it and binds actual sent Context/source/input operation. |
| Tool/delta/foreign/missing thread or turn/previous item | Cannot supply final answer; changed same-ID and two final IDs poison acquisition. Null phase and async content stay explicitly unsupported. |
| Failed terminal with valid APPROVE / potential Medium text | Durable content retained, authoritative work Failure; no approval/gate proof obtainable. Future round counting control remains mandatory. |
| Claude ordinary text, foreign/injected background result, error-only/auth, mixed text+structured | Actual owned terminal/origin and final-idle checks govern acquisition; no diagnostic string or representation preference becomes model approval. |
| Byte/depth/event/encoded escaping boundaries | Exact boundary accepted, overflow prefix length/hash/unseen flag correct; no full hash for partial content or unbounded allocation/drain. |
| Cancel/epoch/new generation before terminal and receipt TX | Only historical draft or original exact owned completion wins; siblings/Task/new Unit/Context/artifacts/leases unchanged. |
| Receipt failure/CAS rollback and watch order | No orphan receipt, half-closed Unit/Session or watch result before commit; known work never replaced by cleanup. |
| Idempotent replay/change/reopen/tamper | Exact replay unchanged; changed terminal/body/index/hash refuses; generic Record input cannot create native-result authority. |
| Ordered 1→…→6 and schema5 old-writer matrix | Actual already-open5 cached read then generic and native-result INSERT/UPDATE/DELETE fail after migration; old5 new open rejects6; current6 typed producer works; failed migration preserves old bytes/version. |

Use positive actual protocol producers plus compiled causal mutants that reach
the receipt/native consumer assertions: omit thread/turn/input binding, accept a
changed duplicate, take unknown last message, treat terminal object as answer,
publish watch before receipt commit, overwrite cancellation, preserve a mutable
second answer, omit an old-table writer guard. Setup refusal/compilation failure
does not count as a killed causal mutant. Run applicable Store/native/Workflow
regressions, fmt/Clippy/build and later exact-head Linux/macOS CI, then immutable
independent source review. Synthetic schema/fixture observations do not prove real
model output phase, subscription recovery, authentication/settings/hooks or full
N-member review. Actor private grants, native qualification/admission envelopes,
readonly review completion, full-round integration and final acceptance remain
separate required work.
