# Issue 5: native Claude Code session design

Status: requirements/design first, implementation pending. Installed baseline:
Claude Code 2.1.283. Current web docs include flags newer than this installed
version; local help and actual native wire evidence govern capability claims.

## Authority and process lifecycle

Implement `ClaudeAdapter` against the provider-neutral `AgentAdapter` contract.
Copy the proven provider-neutral scope snapshot logic into a Claude-owned module
short-term, with Claude's typed error helper; do not depend on unmerged #6.
Expose only existing crate-private `ProcessGroup`/bounded native Git cleanup APIs
needed by both providers. A later shared-module extraction requires equivalence
regressions, rather than cross-agent mutation during active native reviews.

Capture persisted Project/Goal/Task versions, registered environment-reference
names and review locks. Validate canonical native Git common-directory, owning
worktree branch/HEAD and input bindings outside Store locks; CAS-check snapshots
again before spawn/dispatch. A source-only consultation uses its registered
primary Git root. Review checks its clean immutable locked Task revision.
Only a private direct child with `process_group(0)` is a cleanup authority; keep
its leader unreaped until group termination. Maintain separate sticky uncertainty
for each Git/native child. Lost reservations are not cleared by another completed
child, a stale PID, a saved UUID, or a successful native-looking result alone.

## Streaming native protocol

Use native print mode with stream-json stdin/stdout and verbose structured output.
Initialize the bidirectional channel before supplying the factual user input;
keep stdin available for exact correlated native control responses. No SDK/API
model client is introduced. Bound each line, queues, assistant/stderr tails,
prepared inputs, pending callback count and initialization/turn/shutdown waits.
Correlate initialization response IDs, assigned native UUID, native message IDs,
permission request/tool-use IDs and terminal records. Unknown or malformed native
requests terminate safely. Read/write protocol work does not hold Store locks.

Native hook discovery stays enabled; initialization adds no replacement hooks,
agent system prompts, workflow policy, or instruction-source suppression. Native
turn completion and process completion are distinct. Retain the validated terminal
and known usage before stopping the still-owned process group; publish terminal
state only after cleanup. Private completed evidence, not server exit code,
backs the merged `transport_succeeded` consumer contract.

## Roles and decision restrictions

Executor preserves the native permission mode and configured rules; the runtime
never supplies broad allowed-tool grants or bypass flags. Decision roles launch
with an empty built-in toolset, all tool definitions disallowed and strict empty
MCP configuration; unanswered prompts deny. These are capability restrictions,
not replacement native user settings. Managed incompatibility is a typed failure.
Verify installed supported controls and emitted native init inventory; expose no
claim that `--tools ''` alone removes inherited MCP. A permission request or tool
use from a decision session is an unexpected protocol/policy failure, never a
request the same reviewer may approve. Use fresh role-owned native sessions.

Interactive consultation is a real privately owned PTY, retaining native trust
and UI semantics. Its initial prepared input supplies scoped repository facts;
read-only consultation has no operation grant. PTY input/output and resize remain
separate from structured host callbacks. Capability claims follow actual installed
native prompt/response evidence. A trust prompt or alive process alone is not
proof of consultation. Live attach uses that same owned PTY; do not start/adopt
`--bg`, a shared daemon or unrelated native session. Missing owned transports after
restart produce SessionLost instead of fabricated attachment.

## Permission callbacks and pending state

Handle native host `can_use_tool` requests only where available. Store bounded
original parameters and a deterministic operation hash together with exact native
UUID/request/tool-use IDs. Publish WaitingHuman/WaitingApproval as appropriate.
Default integration can deny/cancel and surfaces the pending question; a trusted
runtime broker opt-in may grant only the exact one-shot native operation, retaining
native policy. Recheck scoped lifecycle/native workspace before a grant; audit
intent and fence Session CAS before the wire reply. No persistent permission
updates, modified tool input, remembered allow-all, duplicate grants or synthetic
ESCALATE-as-allow responses. Stop cancels pending work and terminates the owned
group. Native auto-approved operations may never reach this callback; interception
is explicitly partial and cannot be the sole universal security boundary.

## Resume and metrics

Keep confirmed terminal UUIDs and role/workspace/model input provenance. Exact
owned resume restores native history; it does not adopt external sessions or
change an Executor's history into a decision-only review bundle. Revalidate the
original ownership and revision and reject stale approvals on every transition.
Capture native terminal cost/usage and message IDs as observations. Missing or
reset resumed gauges remain unknown; never sum duplicated cumulative counters or
invent missing tokens/cache/cost. ContextPack byte size/version is separately
observable and is not a tokenizer estimate. Usage belongs to the requested
Project/Goal/Task, phase and review round.

## Verification and primary references

Synthetic protocol/PTY targets prove failure/correlation/limits without inference.
Actual private native fixtures prove installed account, Executor, zero-tool Review,
interactive prompt/response and exact owned Resume. Immutable native reviewers
receive public source/factual bundles, preserving existing native rules and the
read-only-session exception. Full code verdicts remain distinct from init/auth
capability probes. Mutations must defeat relevant real assertions and restored
source must pass; unsupported capabilities remain typed and honest.

First-party wire authority is the
[official Python SDK query transport](https://github.com/anthropics/claude-agent-sdk-python/blob/main/src/claude_agent_sdk/_internal/query.py).
The [CLI reference](https://code.claude.com/docs/en/cli-reference) documents native
print/streaming, exact UUID resume and background-session attachment.
[Permission semantics](https://code.claude.com/docs/en/agent-sdk/permissions)
explain callback coverage after native policy evaluation. Prior installed-version
metadata-only evidence and redacted readiness are retained in the private
reconnaissance notes; they are prerequisites, not completed adapter acceptance.
