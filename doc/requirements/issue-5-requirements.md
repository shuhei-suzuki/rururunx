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
  and original validated runtime authority. New process groups remain private;
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
