# Issue 5: native Claude Code session design

Status: implementation and scoped fixture verification in progress; immutable source review pending. Installed baseline:
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
primary Git root. Review and ApprovalReviewer check their own clean immutable locked Task revision. Approval consumers can use a separate same-Project DecisionTask and factual operation bundle; the broker must independently bind the original operation before granting. The provider never treats live mutable executor cwd as a safe approval-review source.
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
[official Python SDK query transport](https://github.com/anthropics/claude-agent-sdk-python/blob/68db221ebe29c1d82b0001ae80fa71e10d57d80a/src/claude_agent_sdk/_internal/query.py).
The [CLI reference](https://code.claude.com/docs/en/cli-reference) documents native
print/streaming, exact UUID resume and background-session attachment.
[Permission semantics](https://code.claude.com/docs/en/agent-sdk/permissions)
explain callback coverage after native policy evaluation. Prior installed-version
metadata-only evidence and redacted readiness are retained in the private
reconnaissance notes; they are prerequisites, not completed adapter acceptance.


## Installed fixture evidence and remaining gates

The native account stays usable only when scoped launch retains trusted macOS
login/keychain context (`USER`, `LOGNAME`, `SECURITYSESSIONID`,
`__CF_USER_TEXT_ENCODING`, `XPC_SERVICE_NAME`, `XPC_FLAGS`). A metadata-only
comparison observed logged-in true under normal/identity-preserving launch and
false when those names were omitted. Project references/values cannot override
these identities; another Project's API-key references are still excluded.

Rustix allocates real stdin/stdout/stderr PTY descriptors and launches the native
CLI directly in `process_group(0)`. There is no helper process, controlling-tty
claim, unsafe pre-exec, shared leader or global attach adoption. A bounded native
probe reached Claude's actual trust/safety screen and verified owned cleanup.
No trust selection or global configuration mutation was performed. This proves
native UI/PTY feasibility; an actual consultation response is still gated by the
native trust prerequisite, so Interactive/Attach capabilities remain unadvertised.
Provider-specific input/resize APIs operate only on the existing scoped owned UI.
PTY closure/exit never produces structured turn-completion or usage evidence.
Interactive startup never puts prepared context in argv. The unadvertised UI prototype is an explicit opt-in and starts without submitting context; owned terminal input is separately bounded and fenced. A caller with a controlling host tty fails closed until the shared session-isolation helper is implemented and reviewed; this is an interim closed boundary, not the final interactive capability.

Actual native zero-tool source consultation and exact UUID resume succeeded
through the Rust adapter. `result.usage` tokens/cache are invocation observations;
cost/API-duration gauges may accumulate or reset on resume. No resumed gauge is attributed to a phase without an explicit cumulative/reset marker: monotonicity alone proves neither. Resumed cost/API duration stay null; raw native reported gauges remain separate observations. Malformed optional telemetry is nullable and never erases a correlated operation outcome. A validated successful run closes stdin to let native
history flush, observes its unreaped leader exit with a five-second bound, then
terminates remaining owned group members before reaping. Stop/failure kill the
owned group directly. Source review and atomic scope-version dispatch fencing
remain required before acceptance.

The pinned [first-party callback option helper](https://github.com/anthropics/claude-agent-sdk-python/blob/68db221ebe29c1d82b0001ae80fa71e10d57d80a/src/claude_agent_sdk/types.py)
selects the native `--permission-prompt-tool stdio` route. CLI native policy may
allow or deny before that partial host callback. In-memory `pending_operation`
returns exact parameters to a trusted broker; audit/recovery contain only bounded
identities, tool name and operation digest, never parameter values.


The actual native Executor fixture read a random nonce that was absent from its
prompt from its owned Task's `proof.txt`. A fresh Reviewer UUID then processed a
factual bundle on the clean locked revision with zero tools/MCP. These are backend
operation/role evidence, not a verdict on this adapter's source. Native default
model and effort, ordinary auth/hooks/rules and trusted OS identity were retained.


The trusted baseline also retains native cloud authentication/routing names and
TLS/cache settings. The [Bedrock](https://code.claude.com/docs/en/amazon-bedrock),
[Google](https://code.claude.com/docs/en/google-vertex-ai) and
[Foundry](https://code.claude.com/docs/en/microsoft-foundry) docs identify these
native credential chains. Registered other-Project reference exclusion happens
before provider-prefix retention, and explicit current-Project values must be
validated declarations. No cloud login/credential chain was replaced or exercised
as evidence for an unconfigured account.


PTY transport uses Tokio `AsyncFd` readiness, with a bounded output queue and
serialized writes/deadlines; an idle terminal performs no periodic read syscall.
Output flood/cancel fixtures verify backpressure and prompt group cleanup.
Resize requests pass through the sole PTY supervisor. It applies the tty window
size and signals SIGWINCH only while owning the unreaped ProcessGroup, so cleanup
cannot recycle its PID/group between an alive check and a cached signal.
Actual native Claude trust screen was preserved and redrew after the owned
40x100 resize. This proves redraw and cleanup, not a consultation response or
controlling-tty support; Interactive/Attach remain unadvertised.

Mark input uncertainty before attempting its wire write. If dispatch may have
occurred but no correlated native terminal is observed, timeout/protocol failure
and pre-supervisor cancellation or explicit stop retain Lost even after confirmed group death
(clear PID only after cleanup). A validated native negative terminal is Failed.
Lost Executors retain their reservation and cannot silently start another attempt.
An early human result with unfinished native background-agent tasks retains its
actual metrics but is not aggregate terminal proof; stop/timeout keeps Lost.
Malformed result markers and injected background/channel/peer turn results never
complete the prepared input or contribute its usage; only native unassigned or
human-attributed correlated terminals do so, matching the pinned SDK origin contract.

## Atomic dispatch and final native fixture evidence

The additive shared `Store::put_session_if_current` compares exact optional
Project/Goal/Task versions and the complete scoped lock ID/version set in the
same Immediate transaction as Session CAS and `session.saved` audit. Project-only
consultation uses absent-owner version zero. It retains existing lifecycle,
worktree-lock and live/Lost Executor guards; no schema change is needed.
Fully encode/bound input and permission reply bytes before the transaction.
Only a successful CAS replaces the private candidate Session; publish the scoped
`dispatch_intent` metadata before wire I/O. Audit projects just this intent and
existing Session identities, never full recovery, prompt, operation parameters
or credentials. Actor-owned PTY input also uses a bounded queue, cancellation
check, canonical workspace recheck and per-input scoped CAS; raw terminal bytes
remain in memory. PTY raw UI input is not a universal policy interception claim.

A causal input fixture pauses after final Git/scope preflight and frame encoding,
then changes Task through a separate Store connection. The CAS rejects it without
native dispatch or consumed intent. Replacing that consumer's scoped CAS with
ordinary Session CAS is killed by the fixture assertion. Injected-result and
post-dispatch Lost-reservation mutations are also killed and sources restored.

After this boundary, actual native fixtures passed with ordinary existing auth
and default model/effort: Executor read a random private Task nonce absent from
the prompt; a different fresh immutable Reviewer UUID returned a zero-tool result;
Project-only Consultant resumed the exact UUID; native trust UI resized/redrew
40x100 and its private group terminated. For this measured run, Execute reported
4 input / 138 output tokens, 47,792 cache-read / 26,416 cache-create tokens,
USD 0.2236624 and 3,180 ms API duration. Consultant resume reported 2 input /
10 output tokens, 12,611 cache-read / 153 cache-create, USD 0.0039542 delta and
1,538 ms API delta. These are fixture observations, not estimates or promises;
other missing/reset gauge cases remain null. Native trust selection and actual
interactive assistant response were not performed. Backend fixture results are
not immutable implementation review verdicts.


## Reviewed continuation and resource boundaries

An owned completed UUID is not permission to replay a previous mutation. Resume requires an explicit higher-version `checkpoint` PreparedInput; the adapter pins its revision, source versions, byte count and exact payload SHA-256 before the first Starting publication. Checkpoint verifies ownership/Git without inference and retains the prior consumed Session as the usage authority. Starting reservations include the SHA-256 of the exact prior terminal Session; failure before input consumption restores that Session byte-for-byte. After input may reach native, an unobserved outcome remains Lost even when every owned group is known dead. Protected typed Context/checkpoint publication remains the separate #19 shared Store integration.

A private admission semaphore reserves registry capacity before Store/native side effects. Completed evidence is retained until explicit release and is never evicted to make room. Scope snapshots collect only exact-scope locks; Project/Goal consultation does not inherit descendant Task lock versions. A cancelled broker reply cannot consume ALLOW authority. Durable intent and the watch state are published before wire awaits; exact DENY uses Session-only CAS because it grants no operation authority, while ALLOW keeps full current Scope/Git/lock fences. Correlated native terminal outcome remains distinct from later parent metadata edits; final Session CAS protects publication.

Initialization buffers bounded known non-control metadata while correlating the response ID and draining bounded stderr. Background-injected turns never replace the sole human-input result/output/usage. After an early human result, the last task termination must be followed by an observed final native idle state and finished injected turn. Failed/interrupted native tasks do not become successful aggregate proof; unavailable aggregate attribution stays nullable. Missing final idle/turn completion remains Lost.

PTY master/slave allocation and duplicated descriptors use atomic CLOEXEC. Read/write supervision uses AsyncFd readiness; the owner continues draining output and observing stop while an input write is backpressured. Resize remains synchronized with unreaped process-group ownership. The shared hidden setsid helper proposal must separately verify session/controlling-tty identity with a bounded READY/GO handshake before this prototype can advertise Interactive/Attach. No permanent requirement for a detached user supervisor is intended.

Installed native provider routing/TLS aliases and OS login identity are trusted baseline controls, never scoped application credential replacements. Another Project's reference cannot disable trusted OS login identity. Other foreign-declared environment names are excluded even when native supports them; routing values that are meant to be global must remain undeclared as Project references. Current Project API-key references still receive normal exact Project scoping. Sources: [native environment variables](https://code.claude.com/docs/en/env-vars), [gateway routing](https://code.claude.com/docs/en/llm-gateway-connect), [network settings](https://code.claude.com/docs/en/network-config).

## Verified source-review corrections

Decision agents must be explicitly listed in Task.reviewers. A provider may be
both the configured executor and an explicitly assigned reviewer; independent
review still uses a fresh role-owned native UUID and factual ReviewBundle.
Injected terminals receive outcome/schema validation before their payload is
excluded. Failed injected work invalidates aggregate success; malformed injected
terminals leave the operation outcome unknown. Mid-turn native feedback does not
create a separate post-result turn. Nested or injected assistant text never
changes the recorded human result after its terminal.

Only one operation is exposed to the broker at a time. An additional correlated
permission request receives a durable exact DENY, retaining the original pending
request; it never receives implicit approval. DENY candidates replace in-memory
state only after Session CAS succeeds. If denial publication fails, the owned
native is stopped immediately; unobserved operation outcome remains Lost. Lost
retains the last durable pending hash as forensic metadata, with the private
permission ledger cleared. This allows the shared conservative Blocked-Project
lifecycle update without manufacturing an unsent denial audit.

Project declarations cannot override native cloud/model/credential-file/process
selectors in CLAUDE_, ANTHROPIC_, AWS_, AZURE_, CLOUDSDK_, GOOGLE_, GCLOUD_,
CLOUD_ML_, VERTEX_ or NODE_ classes. An explicit direct-key allowlist remains
scopable, including ANTHROPIC_API_KEY and OPENAI_API_KEY. Any name declared by a
foreign Project is excluded unless also declared by the current Project, apart
from trusted OS login identity; a native prefix never restores a foreign secret.
Undeclared trusted baseline settings remain available. Installed2.1.283 static
control-name metadata and [official native environment documentation](https://code.claude.com/docs/en/env-vars)
confirm credential-file, regional and model selectors; no credential values were
read for verification.

Filesystem admission exhaustion is a retryable Locked result before dispatch,
not a native turn timeout. Terminal-start audit explicitly says prepared input
was not submitted. ContextCheckpoint stages a higher-version continuation for
later owned resume; it does not deliver context to a live turn. Retained native
evidence requires explicit release after the consumer records its result.

Every permission wait, cancellation and automatic denial uses a Session candidate: the runtime journal changes only after durable CAS. An automatic denial records its exact dispatch intent before wire output. If Project activity changes before publication, Lost is derived from the last committed recovery, preserving only a durably published pending hash. The broad forbidden overlay class does not expand native baseline passthrough: unrelated ambient NODE_/GOOGLE_/GCLOUD_/VERTEX_ secrets remain excluded.

Post-spawn PID publication uses a candidate and current scoped CAS. If publication fails, the runtime verifies owned group cleanup and reap before returning an error, including only owned PID and verified/unverified cleanup diagnostics. Recovery derives from the last committed metadata; an unsubmitted print attempt may fail only after verified cleanup, while the opted-in native UI with uncertain input remains Lost. No stale Starting/WaitingHuman record is manufactured by a never-durable PID.

A correlated native decision result does not establish current authority. Successful decision completion uses current Project/Goal/Task and the exact full scoped lock-set CAS; revoked or changed authority preserves the observed Exited result and actual usage but marks final_authority_current=false and withholds private completed decision proof, checkpoint/resume and transport_succeeded. Executor native outcome remains separate from a nullable final authority observation. Unknown Lost recovery retains last durable metadata for conservative publication. A completed proof is an attempt observation; consumers still require their own current operation grant fence.
