# Issue 7: native Grok adapter requirements

Issue: https://github.com/shuhei-suzuki/rururunx/issues/7
Risk: STRICT: native authentication, process lifetime, filesystem authority and durable Sessions.
Sources: product requirements v0.8, Agent Adapter / Approval / Workflow / Multi-Project masters.

The adapter uses the installed native Grok CLI's private ACP `agent --no-leader stdio`
transport. It is a native process supervisor, not a new coding agent or a planner.
The runtime registers `GrokAdapter` explicitly; names and arbitrary argv never select
a provider. Native model inference, instructions, rules, skills, hooks and existing
authentication remain authoritative. No leader, global configuration replacement,
credentials extraction, bypass mode, always-approve flag or snapshot restoration.

## Acceptance

- Native execution accepts scoped prepared input and actually edits an owned Task file.
  The first implementation exposes only native `read_file` and `search_replace` tools,
  routed through ACP filesystem callbacks. It does not advertise shell execution.
- Consultant and Reviewer Sessions are decision-only: profile identity
  and a tested curated tool contract plus native zero-tool inventory are checked before every fresh/resumed prompt. Empty
  strings, unknown tool names or prompt instructions are never security controls.
- Native `outputSchema` produces collected structured results. Missing/invalid or locally schema-invalid native
  structured output is a typed failure, never a fabricated review verdict.
- Explicit model/effort are applied through ACP configuration setters and verified from
  returned current values before inference. Unspecified values preserve native defaults.
- Executable missing, auth unavailable, unsupported protocol/capability, parse failure,
  launch/state conflict, timeout and lost Session are normalized. Executable presence
  alone does not establish authentication or executor capability.
- Concurrent independent reviewer Sessions work through the existing AgentRegistry.
  Issue 9 still owns actual Review Set policy/scheduling/aggregation.
- Every launch is Task-scoped with exact persisted Project/Goal/Task/worktree ownership.
  Mutating launches reserve Starting before async preflight; executor launches fence persisted review locks,
  base/main/master branches, inactive owners and live/Lost executors remain fenced.
- Filesystem callbacks require the exact current native Session ID and owned Task root.
  Traversal, symlinks, hard-link aliases, foreign Projects, Git/native metadata and
  rule/config authority writes are rejected. ASCII-only no-follow paths and case-insensitive
  protected names reject platform aliases; unsupported UTF-8 file-name/range operations fail. Filesystem/Git I/O runs outside SharedStore.
  Structural identity and version checks preserve concurrent owner changes.
- Process groups, RPC deadlines/frames/output and pending work are bounded. Stop and
  cancellation affect only owned private children. Unknown termination keeps Lost
  reserved; completion cannot be inferred from cancellation or process exit alone.
- Native prompt completion and actual OS exit status remain separate. Unexplained native/hook
  worktree changes invalidate completion; decision-only covers model tool authority. A successful
  native turn requires private owned completion evidence, verified process cleanup and
  successful terminal persistence before `transport_succeeded` can return true.
- Resume preserves the owned native UUID and rururunx Session ID; it requires an explicit
  fresh checkpoint input version, never repeats an old mutating prompt. Missing in-memory
  ownership after restart remains an explicit recovery limitation pending Issue 13.
- Native aggregate token/cache telemetry is collected only when present. Absent values
  remain null; last-model-call counters and projected fallback zeros are not totals.

ApprovalReviewer/requester binding pending Issue 10, interactive PTY/attach, shell/terminal
operations, universal permission interception,
native Goals and cross-process recovery are unsupported in this baseline. Unknown reverse
callbacks fail closed. Native one-time permission callbacks may be denied conservatively;
native default-mode ordinary auto-approved edits do not prove an interception capability. All permission
callbacks are denied; native cached-token authentication has no interactive fallback. Future extensions
must prove those native authority boundaries independently before advertising support.

## Verification

Use fake native ACP subprocesses to cover actual wire/config/schema/auth behavior,
concurrent ownership/resume/cancellation races, malformed data and resource limits.
Use isolated real Git repositories to prove file effects and foreign-scope denial. Native
installed-CLI tests preserve normal auth/config and never touch unrelated repositories.
Required clean committed fmt/clippy/debug/release/workspace gates, meaningful compiled
assertion mutants, immutable independent review/fix/rereview and exact Linux/macOS CI.

Observed reconnaissance establishes only the named capabilities: installed Grok 1.0.46
completed an owned two-tool ACP edit and denied a foreign fixture through host callbacks;
a zero-tool session produced native schema-constrained DENY with zero tool calls. The
native `--reasoning-effort low` flag alone still reported xhigh, so a flag is not sufficient
proof of applied effort. ACP setter verification is required.
