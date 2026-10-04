# Issue 5: native Claude Code adapter

Source: [Issue #5](https://github.com/shuhei-suzuki/rururunx/issues/5).
Depends on merged adapter foundation #4; this work includes the merged #8 context
transport completion contract. Product v0.8 puts native Claude/Codex/Grok support
on the MVP Core path. Authentication, permissions, process ownership and shared
runtime state make this a STRICT change.

## Required behavior

- Run the installed native Claude Code CLI as Executor in an exact persisted
  Project/Goal/Task worktree. Native hooks, mandatory rules, trust, authentication,
  provider routing and default model/effort remain authoritative. Pass explicit
  model/effort only when requested. No login, token extraction, API substitution,
  bare/safe/restricted mode, system-prompt replacement or permission bypass.
- Accept already-prepared ContextPack/ReviewBundle inputs. Select no repository
  context and add no supervisor inference. Preserve exact input Scope, revision,
  version and source bindings. Shared Store locks cover only snapshots/CAS/audit;
  native filesystem/Git/process/protocol operations run outside them.
- Supply actual noninteractive Review/ApprovalReviewer sessions with no model
  operation tools, including inherited MCP entrypoints. Fail explicitly when the
  installed native capability/managed policy cannot honor the restriction.
  Decision roles use factual bundles, never an Executor's conversation history.
- Provide an actual native interactive consultation through an owned terminal.
  Preserve its native trust/permission UI. Make live attachment connect only to
  the adapter's verified owned terminal. Native `claude attach` manages background
  sessions and must not be used to adopt arbitrary/global native processes.
- Resume only a confirmed exact native UUID with the same Project/workspace/role
  and original validated runtime authority. Every continuation requires an explicit new higher-version checkpoint input; never replay cached mutating payload. New process groups remain private;
  restored native history and provider defaults remain native-owned. Do not use
  most-recent/picker/name heuristics, replay stale approvals, or treat a PID or
  UUID alone as reconnection authority after runtime restart.
- Surface supported native `can_use_tool` requests as scoped, correlated pending
  events. Partial callback coverage is explicit: native hooks/deny/allow/automatic
  policy can decide before host callbacks. No universal interception claim.
  Broker replies cannot add permanent grants or rewrite operation input; unknown,
  foreign, replayed or stale requests fail closed. ESCALATE remains pending.
- Normalize unavailable/auth/startup/protocol/native-error/timeout/cleanup outcomes.
  A successful process exit is not a successful native turn. Private completion
  evidence must back `transport_succeeded`, including exact terminal identity and
  verified owned-group cleanup. Uncertain cleanup retains Lost reservations.
- Report actual nullable input/output/cache/cost by Scope, phase and review round.
  Deduplicate native message IDs and handle resumed cumulative gauges without
  stale or fabricated zeros. Missing metrics do not make native operation unusable.

## Verification

Meaningful protocol/ownership/approval/telemetry/PTY regressions and reversible
mutations accompany native fixtures. Actual installed native Execute, zero-tool
Review, interactive prompt/response/attachment, and exact owned Resume must be
exercised with public code or private synthetic targets. Auth failures observed
only inside the tool sandbox must be distinguished from real account readiness.
Full relevant Rust/fmt/clippy/build/CI and independent native immutable-source
reviews precede acceptance. Other providers and workflow/approval engines remain
separate consumers, not prerequisites on unmerged provider implementations.

Acceptance remains pending the reviewed shared session-isolation helper and actual native interactive consultation/attachment. The current explicitly opted-in PTY trust-screen prototype is not an advertised production capability. Independent source review must verify the fixed immutable head; prior request-changes rounds are not approvals.

Source-review acceptance also requires explicit Task reviewer assignment,
validated injected terminal outcomes, preserved human result output, an exact
fail-safe denial for concurrent native permission requests, no phantom DENY
publication after failed CAS, and conservative Lost persistence on a Blocked
Project. Native baseline selector controls must reject Project overlay while
foreign Project environment names remain excluded. Direct scoped keys and trusted
OS login identity keep their documented isolation rules. Staged continuation and
explicit evidence release semantics must be visible to consumers.

Permission wait/cancel and non-broker automatic denial must preserve the last durable Session on publication failure, persist conservative Lost with verified PID cleanup, and audit exact automatic denial intent before dispatch. Rejecting broad Project overlay classes must not forward unrelated ambient Node or Google credentials.

Post-spawn PID publication uses a candidate and current scoped CAS. If publication fails, the runtime verifies owned group cleanup and reap before returning an error, including only owned PID and verified/unverified cleanup diagnostics. Recovery derives from the last committed metadata; an unsubmitted print attempt may fail only after verified cleanup, while the opted-in native UI with uncertain input remains Lost. No stale Starting/WaitingHuman record is manufactured by a never-durable PID.

A correlated native decision result does not establish current authority. Successful decision completion uses current Project/Goal/Task and the exact full scoped lock-set CAS; revoked or changed authority preserves the observed Exited result and actual usage but marks final_authority_current=false and withholds private completed decision proof, checkpoint/resume and transport_succeeded. Executor native outcome remains separate from a nullable final authority observation. Unknown Lost recovery retains last durable metadata for conservative publication. A completed proof is an attempt observation; consumers still require their own current operation grant fence.

StructuredOutput in this implementation denotes validated native stream-envelope collection, not arbitrary caller JSON Schema enforcement. A textual JSON request can produce non-JSON model text. The actual synthetic reviewer fixture accepts only a complete JSON document or exact JSON code fence, keeps format/usage metadata before validation, and never extracts fragments from prose. Native `--json-schema` is a separate documented facility ([CLI reference](https://code.claude.com/docs/en/cli-reference)); a future explicit schema API must consume its authoritative structured result and is not claimed by this adapter today.

Final decision authority is a nullable observation: true requires the atomic scoped publication, false requires a recognized typed Store authority/lock guard failure, and an untyped storage error records null while withholding proof. Native Exited outcome and actual usage survive a storage error if the Session-only fallback publishes. Failed PID publication appends bounded claude.launch_failure diagnostics (Session ID, launch-owned PID/PGID, cleanup proof and error kind), including on a Blocked Project; unavailable audit is explicit in the returned failure. Audit PIDs are forensic identities only and never authorize signals after recovery.

Workflow integration must preserve this strict scope currency. Current Store workflow bookkeeping increments Task.version when binding a running Session, which intentionally revokes a decision snapshot. A separately reviewed Record-only binding transaction must resolve that integration conflict; weakening provider authority or documenting away real Task writes is not a fix. Durable workflow success must also require the decision proof marker, rather than Exited alone. These shared-consumer gates remain pending, outside this adapter change.

Fresh continuation keeps the prior privately proved native UUID in the first Starting publication; that identity does not establish the current attempt's completion. The newly checkpointed input metadata and exact previous-terminal restore SHA-256 remain pinned before native initialization, and the separate current dispatch CAS consumes the fresh input. A causal SQL trigger checks this actual resume caller boundary, then native initialization failure restores the previous terminal Session byte-for-byte. This prepares the #19 immutable owner contract; typed ContextStore source gates still require the separately reviewed shared integration.

Scoped publication has nine production consumers: initial Starting reservation, print PID publication, initial model-input intent, checkpoint Starting, decision completion, broker ALLOW intent, and the opt-in prototype terminal-start/PID/input intents. All keep the same structured AdapterError mapping. Final decision publication additionally retains the typed Store observation: a conflict on the own Session record is unknown, not a parent/lock revocation; its failed fallback never overwrites the independent writer.

An untyped ALLOW storage failure is fatal before any permission response reaches native. The already-dispatched model turn lacks an authoritative terminal outcome after cleanup, so Lost retains the last durable pending hash and executor reservation; no durable ALLOW intent, private pending grant, automatic retry or cached-input replay is permitted. Verified Git preflight Timeout in ALLOW follows the same conservative stop/Lost policy. This is a documented tradeoff, not a claim that an operation was granted: database/turn uncertainty cannot become an implicit retry grant. A causal ALLOW-only SQL abort proves no native response, preserved durable pending metadata, verified owned cleanup and no grant audit.
