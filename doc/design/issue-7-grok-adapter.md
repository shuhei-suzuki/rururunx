# Issue 7: Grok ACP adapter design

Status: implemented; installed-native acceptance passed; independent code review and final CI pending.

`adapter::grok::GrokAdapter::new(agent, absolute_executable, SharedStore)` is an explicit
provider constructor compatible with Registry.register. It implements AgentAdapter and
exposes structured launch/decision output through an additive provider method until the
shared structured-start contract lands. No provider inference or config-schema change.

## Native profiles and launch

Create protected runtime-owned temporary profile files with frontmatter only. Both
profiles declare nonempty exact `toolConfig.tools` IDs `GrokBuild:read_file` and
`GrokBuild:search_replace`, `injectDefaultTools: false`, and the corresponding recognized
allowlist. This curated native harness prevents optional/default tool injection and remote
harness substitution. Executor retains those two file tools; decision explicitly denies
both. Both explicitly deny `search_tool`, `use_tool`, `web_search`, `x_search`, and
`web_fetch`; hosted search is not included in ACP bridge tool counts. Use strict sandbox
for executor and read-only for decision. Native sandbox labels alone do not establish
Task isolation: executor tools delegate to the supervisor's scoped filesystem.

Executor preserves native default permission mode (`permissionMode: default` in its
curated profile); it adds no acceptEdits/auto/bypass/always-approve grant. Ordinary native
file edits can be auto-allowed by existing native policy; if policy asks, the adapter denies
and the tool fails. Thus execute capability does not guarantee every proposed edit succeeds.
Decision uses dontAsk. Default system prompt/rule/skill discovery, native hooks and
inherited MCP startup remain. Decision-only means no model tool entrypoint; it does not
mean native hooks/config discovery have no effects. A before/after worktree observation
must detect unexplained effects before any completed transport result is accepted.

Support is initially pinned to native `initialize._meta.agentVersion == 1.0.46`, protocol
1 and advertised cached-token/load capabilities. Record the verified safe native version,
profile digest and capabilities. ACP offers tool counts but no exhaustive pre-prompt
names API: exact names are authorized by the tested curated profile contract, not inferred
from count alone. Unknown versions or missing identity/contract fields fail before prompt.
The public source snapshot supports this contract but is not binary-source identity proof.

Both use `agent --no-leader --agent-profile <owned-file> stdio`. Exact argv is constructed
internally, not shell-interpolated. Explicit intentional native baseline environment is
retained; project leakage and Git overrides are rejected. Native authentication/config,
loader and permission override variables may only equal the constructor-captured intentional
runtime baseline; per-launch replacement HOME/GROK_HOME/config/auth/library injection or
native safety overrides fail. Additional ordinary explicit environment remains scoped.
Exact baseline-only names: HOME, PATH, TMPDIR, XDG_CONFIG_HOME, XDG_DATA_HOME,
XDG_STATE_HOME, XDG_CACHE_HOME, NODE_OPTIONS, SSL_CERT_FILE, SSL_CERT_DIR,
REQUESTS_CA_BUNDLE, CURL_CA_BUNDLE, HTTP_PROXY, HTTPS_PROXY, ALL_PROXY, NO_PROXY
(and lowercase proxy equivalents). All GROK_*, XAI_*, DYLD_*, LD_* variables are
baseline-only, including unknown future suffixes. GIT_* is always rejected. The
constructor captures these intentional baseline values; launch cannot alter/add them. Preserve native auth home;
never create an empty replacement config/auth home. No native trust bypass is introduced.

Fresh lifecycle: reserve persisted Starting → async immutable Git/owner preflight → owned
child spawn → initialize ACP version 1 → authenticate existing cached_token → session/new
with exact CWD → apply/verify explicit model then effort → native session/info gate →
persist owned UUID/PID/metadata with CAS → durably consume exact input version and unique
prompt ID with CAS → send prepared payload. A failed or uncertain send consumes that input;
no durable dispatch marker means no prompt. Unknown outcomes remain reserved.
Before dispatch, one Immediate Store transaction checks Project/Goal/Task versions, exact
lock ID/version set, Session version/ownership/reservation and input consumption, then
writes the marker through existing Session guards. Buffer the complete NDJSON frame before
this transaction; no other I/O/preflight await lies between commit and its bounded pipe
write. External mutation cannot be made atomic with SQLite: actor checks ownership after
send, journals the actual outcome, and fails conservatively on changed authority.

Reviewer requires the exact immutable active review lock; Executor requires no active lock.
Consultant requires no live/Lost executor on the Task and may share a stable review lock.
Multiple independent decision Sessions use existing scoped Session records, which exclude
only reserved Executors, not other reviewers; every reviewer shares the same lock snapshot.
ApprovalReviewer is explicitly Unsupported in this baseline until Issue 10 supplies exact
requester/pending-request authority. Review/consult output itself grants no native approval.
Concurrent reviewer hook effects invalidate affected reviews rather than fabricate evidence.
Failure after actual native creation is recorded factually, even if owner activity changes;
conservative Session updates preserve Blocked ownership metadata guards.

Inventory gate requires expected agent identity, native Session ID/CWD and the tested curated profile contract plus exact tool
count (executor two, decision zero). Missing/malformed fields or mismatches terminate the
owned process without prompting. Fresh and loaded Sessions use the same gate. Native
model/effort current values must match explicit requested values before prompting.

## Actor and native I/O

One Tokio actor owns each private process and NDJSON channel. It correlates exact RPC IDs,
bounds every frame/output/pending request, drains stderr concurrently, and rejects
unknown/wrong-session reverse requests. It preserves structured native result/error and
scoped factual audit. Prepared text receives a fixed non-command first-line envelope; its
content remains intact beneath that line. Native human-catalog leading slash commands
cannot turn a Context Pack into a permission/workflow control operation. Arbitrary content
metadata is never forwarded. Public supplied recovery JSON cannot create completion authority.

Executor client FS supports ACP read/write only. Resolve relative paths against the exact
Task root and walk pinned directory descriptors with no-follow flags; check directory/file
identity, regular-file type and hard-link count. Reject foreign paths, every dot-prefixed
component (including .git/.grok/.claude/.codex/.agents/.rrx/.github), AGENTS.md/CLAUDE.md/GROK.md
case-insensitively, and exact configured Project rule/config paths and their inode aliases.
The baseline accepts ASCII path components only, so Unicode/case-normalization ambiguity
cannot grant authority on macOS. Read and write share this conservative baseline; non-secret
dotfile/rule reads are intentionally unsupported, not an accidental full-repository claim.
Pinned discovery includes AGENTS.md/CLAUDE.md plus vendor dot directories; GROK.md is
conservatively protected even though it is not a verified discovered rule name. Deny FIFOs/devices/hardlinks/symlinks. Text reads/writes
are UTF-8 and bounded; unsupported ranges or absent parents fail explicitly. New files use
exclusive no-follow creation under the pinned parent. Existing files preserve their mode
and are only written after identity/type/content checks; partial failed effects remain
factual failures, never rolled back or accepted. Recheck root/parent identity and scoped
owner versions around operations. Bound content and line/range requests. Perform
I/O off SharedStore, recheck owner/source snapshots before action, and journal actual
effects before post-operation policy checks. A denied foreign request is a tool failure,
not evidence that the enclosing Task passed or native process died.

Decision Sessions expose no FS/terminal model capability and reject all unexpected file,
terminal and permission requests. For executor native permission callbacks, accept no
persistent grants; unsupported requests receive native reject_once/cancelled. Universal
interception is not advertised: ordinary native file edits may omit permission callbacks,
but the tested file tools still delegate their file I/O through ACP FS. Every permission
callback is rejected with reject_once/cancelled; no approval grant path is advertised.
Native extension hooks are known to fail open on some failures and are not isolation gates.

Native assistant text updates feed bounded output. Schema launch sends native
`session/prompt._meta.outputSchema`; native terminal structuredOutput is retained and
validated locally against the same bounded supported schema contract as well as native
success/error/size. The supported JSON Schema subset has no dialect/ref resolver: type (single object/array/
string/integer/number/boolean/null), properties, required, boolean additionalProperties,
enum, items, minLength/maxLength, minItems/maxItems, minimum/maximum, and annotation-only
title/description. Schema <=16KiB, depth <=16, properties/required/enum <=128, result <=1MiB.
Unsupported schema keywords/dialects fail before inference; native
metadata alone does not validate a result. Post-turn identity/count checks are mandatory;
decision toolCallCount must remain zero, and any FS/terminal callback invalidates decision
success. Any live tool_call/tool_call_update invalidates a decision; executor native tool
metadata must name exactly read_file/search_replace and correlate each successful tool
with its requested-path ACP callbacks. A native tool that resolves a directory/stat or
fallback locally without callbacks cannot establish completed executor transport. Native
search/tool invocation fields invalidate decision success if present. Missing post-turn
counts fail; old history/cumulative counts are not new activity. Executor must retain its
two-tool contract. Notifications replayed during load
are excluded from live output/evidence until the load response has been verified. Review Engine owns verdict/evidence policy. Native
`_meta.usage` aggregate counters are preferred over last-call fields; absent telemetry
and uncontracted cost conversion remain null.

Before native startup and after shutdown, observe exact HEAD, current branch ref, index,
status, and bounded Task file inventory/content hashes, including ignored files. Initial bounds are 4096 entries, 1MiB per file, 16MiB aggregate,
32 directory levels and a five-second observation deadline. Repositories with large ignored
build trees must use a clean isolated Task worktree; no weak metadata fallback is claimed.
Over-budget is explicit observation failure. Executor
changes must match recorded ACP writes; decision changes must be empty. Missing/over-budget
observations or unexplained hook/native/concurrent effects fail completion. Preserve hooks
and report evidence rather than silently roll back effects. Other Project/native-global
lifecycle effects are outside the model tool authority claim. Owned group cleanup includes
ordinary inherited descendants; processes deliberately escaping the group remain an
explicit native lifecycle limit, not a claim of whole-machine supervision.

## Lifetime and resume

Use the shared owned ProcessGroup primitive: observe natural exit without reaping,
terminate the owned group before leader reap, then preserve actual OS exit status.
Prompt end_turn is a separate native success fact. On native completion, close only the
owned session and shut down/reap its private server. Success is privately authorized only
when exact native UUID/prompt/Scope and durable terminal Session match. Stop/cancel/timeouts
do not synthesize success or zero exit. Only native end_turn can establish turn completion;
cancelled/max_tokens/refusal/unknown reasons do not. Once stop is requested, reject new
file/permission work while cleaning up. Uncertain cleanup/persistence or a dispatched prompt without authoritative native terminal
outcome remains Lost/reserved even after verified process death and PID clearing. A known
correlated native terminal failure can be Failed; cancellation never supplies that evidence.

Private registry state prevents duplicate live launches or concurrent same-Session resume.
Checkpoint accepts explicit scope/version/source-bound fresh PreparedInput, persists only
authority metadata, and retains payload/environment in memory. Resume requires a higher
input version than the last dispatched turn, unchanged owning path/provider/role and
verified prior termination. It restores conversation through session/load without
--restore-code, applies current explicit config, rechecks inventory and dispatches the new
input once. The 0400 profile in a 0700 owned runtime directory is immutable while running;
its exact digest/contract is checked before dispatch and matched on load. After the final
setter, verify both model and effort together from returned options. Native per-prompt
aggregate usage after load is collected once; unknown fields stay null. No picker/title/continue heuristics and no automatic old-prompt replay.

Release requires owned verified terminal state; Lost cannot be released. After runtime
restart, absent private ownership returns SessionLost until Issue 13 supplies explicit
verified reconstruction. PTY/attach/shell/native Goal support remain typed Unsupported.

## Evidence and dependencies

Real installed Grok 1.0.46 reconnaissance used existing cached auth, an owned private ACP
server and isolated temporary files. Two tools were reported before actual edit; own
read/write arrived through ACP FS and the foreign read was denied. Native end_turn and
aggregate four-call usage were observed under a curated strict profile; the actual server stopped with OS status 143.
Separate pre-prompt zero-tool validation yielded a schema-constrained DENY, post-turn
toolDefinitionsCount/toolCallCount zero and no callbacks. This is actual invocation proof,
not an inference from empty inventory metadata. The implemented adapter additionally
passed the installed-native owned edit, fresh-input same-UUID continuation and constrained
DENY acceptance test. A second curated decision profile explicitly
removed its two declared tools and hosted/MCP entrypoints; it yielded zero bridge tool
calls and native DENY. ACP model/effort setters returned the exact requested current values.
A new private server loaded the exact prior native UUID and CWD, then acted only on a fresh
continuation prompt; loaded history is not live evidence.

First-party references: [sandbox](https://docs.x.ai/build/features/sandbox),
[permissions](https://docs.x.ai/build/features/permissions),
[settings](https://docs.x.ai/build/settings/reference), and
[native source snapshot](https://github.com/xai-org/grok-build/tree/2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8).
The snapshot is supporting evidence, not proof of exact installed-binary source identity.

Issue 9 owns real parallel Review Sets; Issue 10 owns broker aggregation; Issue 11 owns
runtime configured provider factories and CLI/TUI; Issues 13/14 own verified recovery and
cleanup integration. Keep shared ProcessGroup extraction additive and behavior-identical
with concurrent native Codex/Claude work. No Store schema change is planned.
