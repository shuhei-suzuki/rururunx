# Context Efficiency Design

**Status:** Draft
**Scope:** MVP token/context efficiency

## 1. Goal

Parallel agents and multi-reviewer workflows can multiply the same repository and conversation context across model calls. rururunx must reduce redundant context without reducing required review independence or safety evidence.

The Context Efficiency Layer optimizes **what context is sent**, **when it is refreshed**, and **what can be reused**, while native provider caching remains the provider's responsibility whenever possible.

## 2. Principles

1. **Retrieve, don't dump** — do not inject the entire repository or full task history by default.
2. **Progressive disclosure** — load detailed rules/skills/files only when relevant.
3. **Stable context + dynamic delta** — separate reusable stable context from frequently changing task context.
4. **Share artifacts, not chats** — reviewers receive durable task/review artifacts rather than another agent's full conversation transcript.
5. **Condense history** — bound long-running session context while preserving goals, decisions, critical files, failures, and next actions.
6. **Prefer deltas for re-review** — later review rounds focus on changes since the previously reviewed revision plus unresolved findings.
7. **Preserve independence** — token reduction must not cause reviewers to inherit another reviewer's conclusions before performing independent review.
8. **Measure everything** — token/cached-token/cost estimates should be recorded when adapters expose them.

## 3. Context Pack

Each Task maintains a versioned Context Pack containing concise durable context.

A separate Goal Context Pack contains only cross-Task state and must not duplicate every Task history.

Task Context Pack contents include:

- project ID / repository identity
- goal ID
- task purpose and acceptance criteria
- workflow/risk class
- relevant project rules
- architecture/design references
- relevant repository map slice
- changed files/symbols
- impact-analysis summary
- current diff/revision identifiers
- test/verification results
- unresolved findings
- current blockers/next actions

The Context Pack is not a replacement for source code. Agents may retrieve source files as needed.

Task Context Packs are versioned by Task state/revision so stale information can be detected.

### Goal Context Pack

Goal-level context includes:

- objective / completion criteria
- constraints / non-goals
- source-of-truth refs
- Task DAG summary
- cross-Task decisions
- Goal blockers
- next runnable work
- aggregate review/security state
- aggregate metrics where useful

Goal context references Task Context Packs rather than copying them wholesale.

## 4. Repository Map / Context Index

Issue 18 implements the local Task-scoped `RepositoryContext` library and
`repository-context` inspection example. It indexes exact bound worktrees with
SHA256 content/ignored-evidence/rule/config freshness, deterministic lexical
ranking and expansion, and scoped selection evidence. Entire mandatory rules,
evidence and requested expansions must fit; otherwise it returns `NeedsBudget`.
`utf8_bytes_v1` is a labeled packing estimate, with nullable provider measurement.
Source and Git scans run outside the shared Store mutex and recheck state versions
before audit mutation. See [bounded retrieval design](../issue-18-design.md).
Uncertain native Git cleanup remains fail-closed. Diagnostics preserve the failing
call's native error chain and distinguish an uncertainty latch set by a concurrent
call, which discards the current call's result without attributing the failure to it.
Consumers should render the full error chain (`{:#}` or Debug).
Cancellation may latch uncertainty without returning a diagnostic; verified cleanup
and recovery remain Issue 14 responsibilities.

Issue 19's `ContextPacks` service publishes typed Task and Goal artifacts atomically
with their durable pointers and audit events. It captures primary Project sources
read-only for Goal packs and retains terminal Task context refs as non-launchable
history after cleanup. Goal Task refs carry explicit source-validation-required status during concurrent
work; immutable finalized refs remain historical. Task preparation selects current
sources under the complete
rendered budget; dirty files, rules/config and state changes invalidate stale packs.
Database/pointer versions are CAS guards, separate from phase-stable physical and
instruction hashes. The actual Workflow Engine phase publication port owns launch
ContextVersions; standalone publishers reject workflow ownership and live/Lost
launch contexts. Goal semantic/lifecycle versions remain unchanged for typed
pointer-only publication; the consecutive ContextVersion head is separate atomic
authority. Stale generic Goal writes cannot restore an older typed pointer.
See [durable pack design](../issue-19-design.md).

Maintain a compact repository index inspired by repository-map approaches:

- file paths
- important symbols/signatures
- imports/dependencies
- references/call relationships where cheaply available
- optional language-specific parser data

A token/size budget limits what is injected into an agent prompt.

Selection should prioritize context relevant to:

- task text
- changed symbols/files
- dependency/reference graph
- review focus

The full index may exist locally; only selected slices are sent to agents.

## 5. Progressive Rule and Skill Loading

Do not send all global/project workflow documentation on every call.

Maintain lightweight metadata for available rules/skills and load full content only when triggered by task/phase/file scope.

Mandatory safety/project rules are never omitted merely to save tokens.

## 6. Conversation Condensation

Long sessions may be periodically condensed into a durable checkpoint.

Checkpoint must preserve at least:

- user/task goal
- confirmed requirements/decisions
- work already completed
- current revision/worktree
- critical files/symbols
- commands/tests already run and outcomes
- unresolved errors/findings
- next intended action
- important safety constraints

Recent events remain verbatim for a configurable window.

Condensation must be auditable and must not overwrite authoritative requirements/design artifacts.

Issue 19 implements deterministic condensation over typed, consecutive events.
Goals, decisions, completed work, failures, findings, next actions, constraints,
critical references and verification remain intact across checkpoints. Only
explicitly transient events leave the configurable serialized-byte tail, with
count/sequence-range/rolling-digest evidence. Typed checkpoint records are immutable
and same-Task preparation/publication binds the current chain head or explicit
absence. Own history is automatically included and cannot be omitted by a caller;
it is separate from adopted consultation snapshots. Cross-Task
consultation promotion is an explicit immutable snapshot, independent of later
source checkpoint appends. Generic Context updates cannot move a typed pack pointer
backward or replace its authority. Mandatory
state overflow fails explicitly; no summarizer guesses which text is safe to drop.
Checkpoint refs bind exact Session/Task/Goal provenance and retain historical source
metadata. Cross-Task Consultant promotion requires the same Project/Goal and copies
Consultant-origin semantic facts without its recent transcript, including when
the original Task chain also contains Executor events. Native event normalization and
checkpoint scheduling remain caller/transport integrations, not measured runtime
model behavior.

## 7. Review Bundle

For each review phase, construct a deterministic Review Bundle rather than forwarding executor chat history.

Typical contents:

- project ID / repository identity
- goal ID / task ID
- immutable commit/revision
- requirement/design artifacts required for phase
- relevant project rules
- compact repo-map slice
- diff or changed-file list
- impact-analysis artifact
- test/verification evidence
- review instructions

All reviewers in the same independent review round should receive equivalent factual inputs unless reviewer specialization explicitly requires additional material.

Reviewer findings are hidden from peer reviewers until independent reviews complete, unless the workflow explicitly defines a consensus/reconciliation phase.

## 8. Delta Re-review

For review round N > 1, avoid resending all prior transient context when possible.

Provide:

- baseline reviewed revision
- new revision
- delta/diff
- prior verified unresolved findings
- fixes claimed
- new test/verification evidence
- stable requirements/design references

Reviewer may request broader context if needed.

## 9. Prompt / Context Cache Awareness

Adapters may expose provider-native cache capabilities and usage telemetry.

rururunx should:

- keep stable instructions/tool definitions ordered consistently when it controls prompts
- keep reusable repository/project context in stable prefixes when provider semantics make this useful
- avoid invalidating stable prefixes with volatile data unnecessarily
- use provider-native explicit cache only when the adapter/provider supports it and policy allows it
- never depend on cache presence for correctness

Caching reduces provider computation/cost; it does not necessarily reduce logical context-window token count. Therefore caching and context minimization are separate optimizations.

## 10. Token Budgets

Configurable budgets may exist per:

- task
- phase
- review
- reviewer
- context-pack/repo-map injection

Example:

```yaml
context:
  repo_map_tokens: 2000
  review_context_tokens: 12000
  recent_history_tokens: 8000
  condensation_threshold: 0.70
```

Exact defaults are implementation decisions and must be validated empirically.

## 11. Context Request / Expansion

An agent must be able to request more context rather than being forced to guess.

Examples:

- fetch file
- expand symbol
- include caller/callee
- include design section
- include previous review evidence

Context expansion is logged for observability.

## 12. Cost and token telemetry

When exposed by native agents/providers, record:

- input tokens
- cached input tokens
- output tokens
- context size
- estimated cost
- context-pack size
- repo-map injected size
- condensation events
- context expansion events

Metrics should be attributable by Task, phase, and agent.

## 13. Invalidation and freshness

Context artifacts must carry enough metadata to detect staleness.

At minimum consider:

- repository
- branch/worktree
- HEAD/commit
- workflow phase
- source artifact hashes/versions
- generation timestamp
- Context Pack version

Invalidation examples:

- HEAD changed → diff/review bundle becomes stale
- requirements/design changed → dependent Context Pack sections become stale
- workflow escalated → newly mandatory rules/evidence must be added
- project rule file changed → relevant rule snapshot must refresh

A stale artifact must not be silently reused for a gate whose correctness depends on freshness.

Context artifacts from one Project must never be reused in another Project merely because file paths, Issue numbers, or symbols look similar.

## 14. Context selection pipeline

Logical pipeline:

```text
Task + Phase + Target Revision
          ↓
Mandatory rules/evidence
          ↓
Context Pack
          ↓
Repository-map candidate retrieval
          ↓
Relevance ranking / graph expansion
          ↓
Budget packing
          ↓
Agent-specific formatting
          ↓
Native Agent
```

Mandatory items are packed before discretionary repository context.

## 15. Baseline mode

For measurement and debugging, rururunx should support a baseline/disabled mode that bypasses optional context-reduction behavior where practical.

This allows dogfood comparison of:

- token use
- cost
- latency
- review findings
- task success
- safety/quality regressions

Baseline mode must still preserve mandatory safety/project rules.

## 16. MVP acceptance

MVP must demonstrate:

- repository-map/context-index generation
- token-budgeted relevant context selection
- versioned Context Pack
- progressive loading of optional rules/skills
- session condensation/checkpointing
- deterministic Review Bundle
- delta-based re-review
- provider cache-awareness hooks
- per-task/per-agent token telemetry when available

A dogfood comparison should measure the same representative workflow with and without Context Efficiency features and report token/cost/time differences plus any quality regressions.

The bounded core supports 4096 retained events and 1 MiB per Task checkpoint,
128 Tasks/4096 edges per Goal summary. It provides no semantic deduplication or
checkpoint reset; mandatory overflow requires decomposition into another Task.
Native-source draft DTOs carry no Adapter launch version. Complete rendered pack
estimates are audited independently of optional repository slice estimates.

Issue 19's actual Workflow source port binds typed phase artifacts to exact
Project/Goal/Task scope, owned HEAD/content hashes, payload digest and selected
phase/budget. Engine alone publishes the phase ContextVersion and preserves the
reserved artifact through actual Cleanup. Optional repository sections obey the
discretionary budget; mandatory Task/Goal/checkpoint metadata and rules are counted
separately, once, in total UTF-8 byte/token estimates. The complete typed native
input remains hard-capped at 1 MiB; standalone explicit budgets cap their complete
rendered input. Measurements remain nullable. Restart restores durable scoped
facts/references, and Goal summaries atomically compare all owned Task membership
and versions, including Tasks outside the DAG. Ordered v3→v4 checkpoint-head
index migration fences older writers; native Starting/Running reservations compare
the prepared own-head digest or explicit `none` before launch.

Irreversible PR/merge/Cleanup evaluation claims and first/new consumed native
dispatch intents recheck the scoped indexed checkpoint head atomically. Same
admitted input remains pinned through approval/human/Lost observations; terminal
continuation requires a higher version and an immutable exact historical restore
proof before wire delivery. Current checkpoint source invalidation is conservative:
it can restart a pre-effect generation or hold post-PR work for Issue 13/23
reconciliation. These core proofs do not claim automatic post-PR recovery or
actual double/triple native runtime dogfood.

Ordered schema 4→5 adds private immutable prepared-frame authority for standalone
selection. Preparation publishes scope/context version/HEAD/source versions/UTF-8
byte count/lowerhex SHA256 of the exact complete PreparedInput.payload in the
same Immediate owner/source/head CAS and audit transaction. Up to 128 variants
per context version are retained; prompt bodies are not duplicated. Generic audit
or Record writes cannot fabricate this authority. Workflow authority uses its
immutable ContextVersion.data.payload, including mandatory Engine rule prefix.
Store::validate_context_input checks actual request bytes; Session admission pins
input_sha256 and consumed intent must match that hash. Provider-specific RPC
envelopes/fixed prefixes are a distinct transport digest, never this input hash.
Older public schema 4 writers refuse schema 5. Existing schema4 standalone packs
must be prepared again under the new private publication contract before launch.

New native admissions of typed Workflow frames require the current active Running,
dispatch-started attempt with the exact context, phase and generation. Session
publication also binds its owned Session ID. Inactive owners, terminal Tasks,
frozen final packs and EvidencePort phases are non-launchable. Already-consumed
observations retain their historical input instead of re-admitting it. Stable
Project scoped references and Goal/Task instruction hashes are source authority;
Goal criterion satisfaction, DAG progress and raw row counters are bookkeeping.

Optional source sections use the smaller of the selected discretionary budget and
the remaining absolute 1 MiB capacity after mandatory pack metadata and Engine
rule bytes. Zero remaining optional bytes is valid; mandatory overflow still fails
closed. The selected phase budget remains unchanged and reported. New generic
Context writes reject reserved task_pack/frozen_task_pack envelopes. Readers
classify historical legacy envelopes by the explicit typed format. Pointer-only
Goal contention is a typed SnapshotChanged on goals.context_version. Reserved
preparation/index/selection audit events require their private producer paths.

Blocked-owner Lost updates may add only native_dispatch_unobserved=true and clear
PID while preserving exact actor, scope, input, restore proof and consumed intent.
This flag is conservative uncertainty, not a new dispatch admission. New
checkpoints record their configured transient window; historical checkpoints with
no recorded policy retain an explicit unknown value.

A successful native actor acknowledgement may refresh only its Project/Goal CAS
rows after sibling bookkeeping changes. It first verifies the exact unchanged
Task/Workflow versions, active attempt/context/generation and stable semantic
instruction hashes, with active lifecycle guards. Session ID publication then
uses the new CAS in the same existing atomic Workflow transition. Changed
constraints, scoped references, Task/attempt authority or paused owners remain
fenced; acknowledgement is not another model dispatch. Legacy sources retain
strict Project/Goal version equality.

### Private native input admission

The reviewed-before-code schema6 contract adds one private Task-scoped admission
digest per Session, atomically with validated Running persistence/audit. Production
remains schema5 until that implementation is validated. Pending waiting input
revalidates live frame/head; Lost is absorbing under generic writes; uncertainty is not consumption. New consumed intent always revalidates.
Nonterminal admitted input cannot visit Starting to restore an older terminal
frame. Exact prewire restore is denied after admission/consumption/uncertainty;
owned higher-version terminal continuation binds fresh authority first. These are
Store input-admission facts, separate from Workflow actor Session binding and
provider wire-delivery evidence. Caller JSON/generic writes cannot assert them.

Semantic instruction projection is versioned and exhaustively classifies each
domain field; adding fields requires explicit authority review. Physical HEAD is source authority;
Engine effective workflow/risk/budget belongs to immutable phase authority.
Standalone frames additionally render and bind their Task policy/directive digest and admit
only the exact Executor agent/role/worktree. Explicit idle consecutive republish
provides migration/terminal-continuation versions without changing active input.
Consultant history remains supported; live consultation needs a separate scoped
frame port. Approval review uses an operation-free decision Task.

Ordered5-to-6 migration installs a private registered SQLite writer function and
additive INSERT/UPDATE/DELETE fences on every application table, retaining existing
domain triggers. Thus old5 connections already open before migration also refuse
writes, including metadata/audit. Actual compiled live-connection and fresh-open
old-writer fixtures must prove refusal without state change. No historical input
admission rows are synthesized. See Issue19 design for field inventory, exact
transition predicates, canonical hashing and native proof requirements.

All typed Workflow actors bind the actual Task worktree and one private Session
allocation per immutable single-actor phase context. Future parallel review uses
its own scoped roster authority. Semantic instruction changes are versioned;
projection definitions never change silently. Persistence upgrade requires typed
active/nonterminal unfinished Workflow, all live/Lost Session and active lock ownership to drain under the compatible old runtime
before any migration mutation. Finalized immutable history remains available.

Fresh admissions require permitted standalone Task states or the exact typed
phase Task state/key. Task holds before first consumption/Running block admission;
historical observations retain pins. Typed context publication cannot transform a
live legacy Session's protection contract. Forced identical publication requires
private validated preparation/admission and exact terminal input, not raw history
JSON. Terminal non-owning cancelled Workflow history may migrate; active claims
remain operational ownership even on terminal Tasks.


Initial native actor binding is monotonic per input: requested None may bind one
effective model/effort value, explicit Some cannot silently change, and a known
native_ref remains immutable across the Session UUID. Exact admitted metadata
pins and private same-attempt binding updates remain atomic. Higher-input fresh
continuation resets requested model/effort before Starting and keeps exact prior
terminal rollback proof. Claude5, Codex6 and Grok7 require reviewed complete prepared-frame preflight,
input SHA/source pins before Starting, atomic consumed admission before wire,
native binding compatibility and causal typed-frame caller fixtures before
integrated acceptance.

Private phase allocation covers initial INSERT and fresh terminal-to-Starting
UPDATE; Engine binding/closure must use that exact allocated owner. Ordered
migration preflight covers every older supported version, and an old Lost owner
without verified recovery remains an explicit upgrade limitation. Existing
checkpoint-v1 and terminal restore hash encodings remain byte-compatible. Backup
rollback is permitted only before any post-upgrade application/external effect.

The private admission index stores separate validated-preparation and admitted
pairs. Starting prepares current input without asserting delivery; only validated
Running/consumption admits it. Historical actor binding can update an admitted
pair solely for permitted initial None-to-Some fields, never change its input.
These facts cannot be synthesized from Session recovery JSON or generic audit.


A typed irreversible claim must carry an explicit own checkpoint head; absence
fails closed. Single-actor ownership survives the gap before Engine Session
binding: an allocated live Reviewer prevents claim closure. The native dispatch
intent denotes actual input delivery; an own-checkpoint append requires fresh
higher-input continuation before another delivery. Historical observation alone
sends no new model input. Variant-cap exhaustion supports explicit idle consecutive
republish without discarding history. Idle unfinished Workflow hot-upgrade is not
supported; compatible-runtime completion or explicit terminal cancellation is
required, with remaining external effects reconciled.


An allocated native owner is tied to its exact prepared input, not merely Session
identity. Historical prewire restoration cannot certify a fresh phase. Pending
preparation can bind actor identity; success also needs that input's private
admission/consumption and Exited outcome. Semantic authority changes require both
projection and persistence/writer-fence version changes.


Persisted Lost is absorbing under generic writes; uncertain ownership cannot
become terminal through generic history writes; trusted owned recovery remains Issue14. Private admission proves exact
input, while native adapters retain their separate registry/terminal authority.
Unregistered historical UUID/native_ref cannot authorize native resume. Pending
initial actor binding refreshes preparation only after latest full-frame/head and
lifecycle validation in the same transaction. Checkpoint facts remain explicitly
caller-classified historical coordination evidence; Consultant facts are preserved
without representing them as native transcript or completion proof. New claims
retain live-head checks; already admitted claims retain their frozen provenance.
Migration verifies every legacy typed checkpoint reference before mutation.


Protected actor pins are checked on every Session update, not only launches.
Historical initial binding requires the exact private admitted digest and updates
its pair atomically; it cannot recover Lost. Admission encoding has explicit
entry/string/aggregate bounds. Fresh and migrated schema authority constraints
must match. Writer compatibility SQL fences state writes; old runtime/native/Git
activity must be stopped and drained for supported upgrades.


Universal absorbing Lost intentionally changes legacy history settlement, including
Goal/Project-only Sessions; Goal publication/removal stays held until trusted #14
recovery. Initial terminal Consultant facts remain allowed. A typed dispatch UUID
is consumed once per Session/input version and privately indexed before wire.
A second delivery at that version rejects; identical intent is observation only.
Exhaustive Session field classification and common preparation/admission bounds
prevent silent authority drift. Native callers share canonical restore hashing
and compare exact private registry snapshots before continuation.


Schema6 reserves private preparation/admission/allocation mutations to the native
Session owner/version/complete-lock CAS port; generic history cannot seed them.
Consumed DTO versions must equal current checked owner versions. Current-input
binding and restored-not-admitted failure closure use distinct predicates, keeping
an already bound owner immutable while refusing historical success. Input indices
prove currency, not native terminal truth; Workflow retains adapter-owned outcome
evidence. All complete typed native frames share a 1 MiB cap before selection or
publication. Protected Session restore hashing is byte/depth/node bounded. The
single-delivery count concerns PreparedInput frames; fixed protocol replies remain
separate, and live free-text instruction shortcuts are unsupported.


Exact prewire restoration precedes intent novelty checks. An older restored owner
is frozen while its allocated attempt is open, keeping failure closure possible;
separate diagnostic records remain allowed. Historical binding skips live head
comparison while existing active-owner/lock CAS fences remain. All fresh Workflow
native phases require actual typed authority, including generic/Fake/native caller
migration and default-absent admission capability. Allocation applies only to
Workflow native phases; standalone frames allow separately guarded new Sessions,
and evidence/opaque history is never no-admission proof.
