# Issue 6: Native Codex verification in progress

This evidence is partial; Issue 6 is not ready to merge. Native TUI/attach,
final immutable independent delta source review and final
exact-head CI remain required. Authentication/provider compatibility guards are
implemented and tested; native Interactive/Attach remain explicitly unavailable.

## Installed native execution

Installed Codex CLI 0.160.0 used existing authentication/configuration/hooks/trust
and its unspecified native model/effort defaults (`gpt-6.1-sol`, `high`). No OAuth
extraction, direct inference API, global config/auth/trust write, bypass flag,
profile inheritance, native reviewer substitution or foreign user repository was
used. Fixtures are two independent temporary committed Git repositories with
Project/Goal/Task/WorktreeManager ownership and durable SQLite Sessions.

At production commit `bba475b1c0177be22a11372bfc87e77a094b9033`, actual Executor
native UUID `01a100b0-7956-70a1-aff3-e36c1e4f6f86` created and physically verified
`result.txt` containing exactly `NATIVE_EXECUTED\n` in its owning Task worktree.
Foreign Scope was rejected; the other Project had no result artifact. The Session
persisted Exited with PID cleared only after confirmed owned group cleanup.
Observed entire-turn native usage: input 22,603, cached input 10,880, output 269,
total 22,872; monetary cost unavailable. This includes both model/tool cycles,
instead of reporting only the final call. Context Pack payload was 216 bytes.

Actual immutable review and checkpoint/resume used the same rrx Session/native
UUID `01a100b0-763f-7082-83a0-b6239875aa5a`. Supplied synthetic `clamp(x)=x-1`
returned the real witness `x=0 => -1`, then `x=-3 => -4` after resume. The second
turn retained native identity/defaults, rejected foreign Scope and persisted
verified Exited. Observed second-turn usage after subtracting the owned prior
gauge: input 9,663, cached 0, output 37, total 9,700; cost null. Checkpoint version
2 and updated 123-byte input were attributed correctly. These model runs verify
native transport/structured review, not independent implementation-source review.

## Committed Rust controls

At `50c027d`, all 131 workspace tests passed, including 34 Codex tests. Formatting,
all-targets Clippy with warnings denied and locked offline release build passed.
Tests cover actual owning Git roots/worktrees, dirty immutable reviews, directory
replacement with identical HEAD, lock ABA, lifecycle/version/scope conflicts,
native PID/PG authentication and exact socket cleanup, bounded WebSocket frames,
early events, cumulative usage, resume baselines and counter resets.

Unix WebSocket callback regressions verify exact one-time replies and audit
intent, Human-route grant rejection, previous-turn/replayed/cancelled replies,
changed Session owner, paused Goal and injected audit failure before wire delivery.
These are protocol fixtures, not a claim of real installed native approval routing
or completed Cross-Agent Approval dogfood. Native default automatic review is
retained; explicit broker opt-in requires its existing client route and is rejected
before inference otherwise. Persistent or additional permissions are unsupported.

The previous native TUI reconnaissance connected an exact stored UUID to an owned
private server and retained its trust prompt. That establishes transport/native
gating only. Final interactive lifecycle and operation evidence remains pending.


## Workflow integration and latest installed native proof

Workflow #8 merged at `1c44316ee5d4092c3d519782350c6463aa94269f` on latest
product requirements v0.8. Root Codex production bytes were preserved during the
rebase; shared additive structured-start and transport-completion APIs are both
retained. At `cfcbfcc`, all 203 workspace tests and two doctests passed. At
`bba7acc8f16d8a6b43c2d48b6a634c24b20658f5`, 57 Codex regressions and all-targets
Clippy with warnings denied passed. Final full/build/CI gates remain pending.

The installed experimental protocol is conformed against `codex-cli 0.160.0`.
An owned, bounded `--version` subprocess rejects unknown versions before native
thread/model use. Initialize metadata must include bounded absolute native home,
userAgent and the local Unix platform; userAgent is not treated as a binary
version or peer credential. IPC peer/process-group authentication remains separate.

At `bba7acc`, Executor native UUID `01a1011f-4113-7e12-ad70-187c2d866534`, rrx
Session `2534da9f-0574-4888-bb18-f57014983370`, physically created the exact owning
`result.txt` artifact again. Foreign scope/artifact checks passed; verified Exited
has no PID and no fabricated process exit code. Workflow `transport_succeeded`
accepted the private completed native-turn journal after cleanup and persistence.
Native entire-turn usage was input 59,533, cached input 43,648, output 438, total
59,971, cache-write input 0 and reasoning output 32. Context payload 216 bytes,
phase implementation, version 1; cost unavailable. These are observed native
multi-cycle counters, not an efficiency benchmark.

The concurrent independent review/resume fixture used native UUID
`01a1011f-4116-7210-aed5-9de54c974358`, rrx Session
`a83f21b1-0a1c-4e77-917c-b99856f4c6bd`. Real mathematical witnesses were
`clamp(0)=-1` and `clamp(-3)=-4`. The second turn retained the same identities and
native `gpt-6.1-sol`/`high` defaults, with an explicit fresh checkpoint version 2.
Its input was 13,088, cached input 0, output 37, total 13,125, payload 123 bytes,
phase review, round 2; cost unavailable. Workflow private completion accepted
both turns and all owning groups were confirmed cleaned before persisted Exited.
These transport fixtures do not substitute for independent source review.

Fresh continuation input is mandatory before resume: version must advance beyond
the last persisted attempt. Cached mutating prompts are never implicitly replayed.
A failed pre-inference attempt with confirmed cleanup restores the old terminal
record/watch and may retry the explicit continuation. Post-inference uncertainty
remains Lost; a known failed attempt is not silently retried. Explicit checkpoint
can refresh mutable own Project metadata, while root/repository/base/namespace
identity cannot change. Observations hold the registry owner and compare persisted
turn/journal under short locks so resume cannot mix old counters/new context.

## Independent native source review, findings verified

Public immutable baseline `c854c4acbd96b029d6fcd31c2aa9fc88ef7339ab` was reviewed
by two independent real native Claude CLI zero-tool runs, using only public source
bundles, requirements/design and verification facts. Neither reviewer received
another reviewer's findings before completing. Native defaults/auth/settings were
retained; no executor transcript, private credentials, target-operation tools,
permission bypass, explicit model or effort override was supplied. Owned process
cleanup was confirmed for every counted terminal run. This is current manual
source verification, not completed Issue 16 multi-provider Review Engine dogfood.

| Review | Native session | Result | Native API time / observed cost |
| --- | --- | --- | --- |
| Lifecycle | `e926d6ed-722f-4825-87e4-ffaae114af18` | request changes, 1 High / 4 Medium / 5 Low | 554,366 ms / USD 1.738104 |
| IPC / grants | `1f3a6a55-2009-4c93-9ffe-b85ae9f599bb` | request changes, 1 High / 3 Medium / 4 Low | 579,872 ms / USD 1.573696 |

An earlier oversized IPC bundle exceeded its 600-second deadline. That invocation
was stopped with confirmed owned group cleanup and contributes no review verdict.
Both counted reviewers completed independently before findings were shared.

Verified lifecycle fixes: serialize resume/checkpoint/release/eviction transitions;
restore terminal authority/watch on failed pre-inference resume; keep Lost/PID and
scoped diagnostic when **any** owned Git/native group cleanup remains uncertain;
keep transient rejected grants pending/deniable; unrelated Project registration
must not revoke another Project's grant; unavailable stopped/resume gauge baseline
and replay mismatch stay null; no native server exit-zero becomes model-turn
success; checkpoint monotonicity and real Git validation precede replacement;
repeated stop is idempotent; explicit checkpoint refreshes own mutable metadata.

Verified IPC fixes: bind grants to reviewed command/CWD or bounded native planned
file patch, method/arguments/content digest, Task workspace and current identities;
reject path traversal, outside/symlink/hard-link targets and unknown patch facts;
retire resolved/completed native requests without runtime grants; reject unknown
native human-input/other requests explicitly with a bounded protocol error and
scoped audit; preserve native retryable errors and validate embedded start IDs;
remember high-water token counters across missing snapshots; verify initialize
shape and native binary compatibility before inference. Native commandActions are
best-effort display metadata, not proof of every shell target; the native scoped
sandbox and denied escalation remain the command boundary.

Residual limitations: production native Interactive/Attach and its exclusive
Human-versus-broker grant route are pending, so no concurrent TUI/client grant
capability is advertised. Native user-input replies are unavailable in the current
noninteractive route. Commit `38d1cad` addresses the Low startup alias readiness
and cleanup failure findings: a dangling alias retries only while its target is
absent within the existing startup deadline; wrong UID/mode/namespace/peer remains
fatal. An unverified cleanup retains the original sanitized failure as well as
the cleanup failure. All 59 Codex regressions passed at this commit. A final delta source
review must include full shared ProcessGroup inspect/cleanup/reap/Drop helpers;
the original IPC excerpt supplied only part of that helper. Final Linux/macOS CI
is still required; no merge approval is claimed here.

## Semantic mutations at the current production baseline

At immutable `38d1cad942485a74dcf14d950df5356c64f87250`, 13 independently
compiled mutants were killed by assertion failures in the existing causal tests:

| Mutation | Removed guarantee | Causal regression |
| --- | --- | --- |
| M601 | private native completion authority | caller recovery cannot fabricate completed turn |
| M602 | observed turn identity | stale telemetry/pending approvals rejected |
| M603 | fresh continuation version | cached prompt cannot be implicitly replayed |
| M604 | original failure after uncertain cleanup | both sanitized causes retained |
| M605 | operation-content digest | changed operation cannot claim reviewed approval |
| M606 | hard-link ambiguity rejection | native planned patch cannot grant a linked foreign target |
| M607 | uncertainty from any owning process group | Lost retained when a separate Git group is unverified |
| M608 | exact native version conformance | unknown native version rejected before a turn |
| M609 | exclusive transition claim | release and another resume cannot race a claimed attempt |
| M610B | counter high-water memory | missing snapshot cannot conceal a later native reset |
| M611 | native CWD binding | foreign native thread CWD rejected |
| M612 | effective native permission profile binding | writable profile cannot replace decision restriction |
| M613 | actual native tool inventory verification | surviving tools and discovery errors fail closed |

The first M610 attempt matched no source anchor and receives **no mutation credit**;
M610B is the separately compiled semantic replacement that actually failed its
assertion. Every mutation ran in an owned detached temporary worktree, with
byte-exact restoration in `finally`; all 59 original Codex tests passed after
each batch's restoration. Compiler errors, runner failures and timeouts receive
no kill credit. No production branch was mutated by these runs.

## Native TUI gateway reconnaissance

At `4d2aa5e343c1f1e158f819711ad8d8f50fa6d98c`, the complete workspace passed
216 Rust tests and 2 doctests; fmt, all-targets clippy with warnings denied and
release build passed. Nine newly compiled semantic mutants were killed by
assertions: actual dispatch caller CAS (M616), unknown native outcome after
loss/stop (M617), abandoned consumed resume (M618B), complete JSON wire bound
(M619), each parent version (M620C), lock ABA (M621), Session CAS (M622), atomic
intent audit (M623), and pre-wire consumed boundary (M624). Originals were restored
byte for byte and all 65 Codex regressions passed after every batch. Initial
ambiguous M618/M620/M620B attempts have no credit; only their separately compiled
replacement variants count. This is regression evidence, not native TUI acceptance.

Linux CI passed at that head. macOS CI run `37119853652` failed the existing
same-HEAD replacement test's successful fresh Git check. That assertion hid the
returned cause; replace it with `unwrap` to preserve sanitized adapter diagnostics.
The cause remains unverified, and no process-ownership guard, cleanup latch or
deadline has been relaxed. Independent TUI design re-reviews still request changes
to live human-turn state, native-home isolation, trust/policy and PTY cleanup;
no implementation or merge verdict is implied by completed design reviews.

At `7075fcd`, two further compiled semantic mutants (M614/M615) removing cleanup
uncertainty handling from discovery success or setup failure were killed by
assertions. The original 60 Codex regressions passed after byte-exact restoration.
The subsequent atomic dispatch/source pinning/Lost changes require fresh checks,
mutations and full independent source review before merge; these earlier results
do not validate later edits.

An actual installed native CLI connected through a temporary private Unix gateway
to stored mathematical-fixture UUID `01a10137-f77b-76c3-aa2c-407dd1ba0050`.
Observed native client methods were `initialize`, `initialized`, `account/read`,
`thread/read` and `config/read`. The real terminal remained alive at the native
trust gate; the owning native UUID was unchanged, and both owning process groups
were confirmed dead before reaping their leaders. No trust choice, target
operation, config/auth write or additional permission was supplied.

This proves native metadata transport and retained native gating only. The
temporary gateway is reconnaissance, not production authorization or a reviewed
scope implementation. In particular, metadata allowlisting does not establish
safe configuration projection, actual conversation, new-turn attribution or
exclusive Human/broker authority. Production Interactive/Attach remain unavailable.
