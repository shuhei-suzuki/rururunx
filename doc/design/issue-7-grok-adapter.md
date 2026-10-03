# Issue 7: Grok ACP adapter design

Status: committed design; implementation and acceptance gates pending.

`adapter::grok::GrokAdapter::new(agent, absolute_executable, SharedStore)` is an explicit
provider constructor compatible with Registry.register. It implements AgentAdapter and
exposes structured launch/decision output through an additive provider method until the
shared structured-start contract lands. No provider inference or config-schema change.

## Native profiles and launch

Create protected runtime-owned temporary profile files with frontmatter only. Executor
profile permits exactly `read_file` and `search_replace`; exact disallowed IDs remove
`search_tool` and `use_tool`. No shell, terminal, web, MCP discovery entrypoint, subagent,
scheduler or native Goal tools are exposed to the model. Use workspace sandbox plus
ACP client filesystem capabilities; native file tools delegate to the supervisor's scoped
filesystem, independently of the sandbox's broad read policy.

Decision profile permits recognized WebSearch while `--disable-web-search` makes it
unavailable, and explicitly removes both MCP meta-tools. Use native read-only sandbox
and dontAsk. This preserves default system prompt/rule/skill discovery and native hooks;
native inherited MCP startup may still occur, but no model tool entrypoint remains.

Both use `agent --no-leader --agent-profile <owned-file> stdio`. Exact argv is constructed
internally, not shell-interpolated. Explicit intentional native baseline environment is
retained; project leakage and Git overrides are rejected. Preserve native auth home;
never create an empty replacement config/auth home. No native trust bypass is introduced.

Fresh lifecycle: reserve persisted Starting → async immutable Git/owner preflight → owned
child spawn → initialize ACP version 1 → authenticate existing cached_token → session/new
with exact CWD → apply/verify explicit model then effort → native session/info gate →
persist owned UUID/PID/metadata with CAS → send unique prompt ID and prepared payload.
Every async boundary is followed by necessary owner checks before irreversible dispatch.
Failure after actual native creation is recorded factually, even if owner activity changes;
conservative Session updates preserve Blocked ownership metadata guards.

Inventory gate requires expected agent identity, native Session ID/CWD and exact tool
count (executor two, decision zero). Missing/malformed fields or mismatches terminate the
owned process without prompting. Fresh and loaded Sessions use the same gate. Native
model/effort current values must match explicit requested values before prompting.

## Actor and native I/O

One Tokio actor owns each private process and NDJSON channel. It correlates exact RPC IDs,
bounds every frame/output/pending request, drains stderr concurrently, and rejects
unknown/wrong-session reverse requests. It preserves structured native result/error and
scoped factual audit. Public supplied recovery JSON cannot create completion authority.

Executor client FS supports ACP read/write only. Resolve relative paths against the exact
Task root and walk pinned directory descriptors with no-follow flags; check directory/file
identity, regular-file type and hard-link count. Reject foreign paths and protected Git,
native config, rules and runtime metadata. Bound content and line/range requests. Perform
I/O off SharedStore, recheck owner/source snapshots before action, and journal actual
effects before post-operation policy checks. A denied foreign request is a tool failure,
not evidence that the enclosing Task passed or native process died.

Decision Sessions expose no FS/terminal model capability and reject all unexpected file,
terminal and permission requests. For executor native permission callbacks, accept no
persistent grants; unsupported requests receive native reject_once/cancelled. Universal
interception is not advertised: native auto-allowed file edits can bypass callbacks and
native extension hooks are known to fail open on some failures.

Native assistant text updates feed bounded output. Schema launch sends native
`session/prompt._meta.outputSchema`; native terminal structuredOutput is retained and
validated for success/error/size. Review Engine owns verdict/evidence policy. Native
`_meta.usage` aggregate counters are preferred over last-call fields; absent telemetry
and uncontracted cost conversion remain null.

## Lifetime and resume

Use the shared owned ProcessGroup primitive: observe natural exit without reaping,
terminate the owned group before leader reap, then preserve actual OS exit status.
Prompt end_turn is a separate native success fact. On native completion, close only the
owned session and shut down/reap its private server. Success is privately authorized only
when exact native UUID/prompt/Scope and durable terminal Session match. Stop/cancel/timeouts
do not synthesize success or zero exit. Uncertain cleanup/persistence remains Lost/reserved.

Private registry state prevents duplicate live launches or concurrent same-Session resume.
Checkpoint accepts explicit scope/version/source-bound fresh PreparedInput, persists only
authority metadata, and retains payload/environment in memory. Resume requires a higher
input version than the last dispatched turn, unchanged owning path/provider/role and
verified prior termination. It restores conversation through session/load without
--restore-code, applies current explicit config, rechecks inventory and dispatches the new
input once. No picker/title/continue heuristics and no automatic old-prompt replay.

Release requires owned verified terminal state; Lost cannot be released. After runtime
restart, absent private ownership returns SessionLost until Issue 13 supplies explicit
verified reconstruction. PTY/attach/shell/native Goal support remain typed Unsupported.

## Evidence and dependencies

Real installed Grok 1.0.46 reconnaissance used existing cached auth, an owned private ACP
server and isolated temporary files. Two tools were reported before actual edit; own
read/write arrived through ACP FS and the foreign read was denied. Native end_turn and
aggregate four-call usage were observed; the actual server stopped with OS status 143.
Separate pre-prompt zero-tool validation yielded a schema-constrained DENY, post-turn
toolDefinitionsCount/toolCallCount zero and no callbacks. This is actual invocation proof,
not an inference from empty inventory metadata. Adapter implementation must reproduce it.

First-party references: [sandbox](https://docs.x.ai/build/features/sandbox),
[permissions](https://docs.x.ai/build/features/permissions),
[settings](https://docs.x.ai/build/settings/reference), and
[native source snapshot](https://github.com/xai-org/grok-build/tree/2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8).
The snapshot is supporting evidence, not proof of exact installed-binary source identity.

Issue 9 owns real parallel Review Sets; Issue 10 owns broker aggregation; Issue 11 owns
runtime configured provider factories and CLI/TUI; Issues 13/14 own verified recovery and
cleanup integration. Keep shared ProcessGroup extraction additive and behavior-identical
with concurrent native Codex/Claude work. No Store schema change is planned.
