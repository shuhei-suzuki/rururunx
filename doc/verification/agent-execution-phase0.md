# Native Agent execution Phase0 inventory

Status: Phase0 static investigation. Human approval is required before Phase1.
No production code, dependencies, licenses, README status, or existing master
requirements/design have been changed. No native Agent acceptance is claimed.

## Scope and provenance

The latest user request replaces the earlier whole-workload custody proposal.
Process cleanup is best effort; it must not overwrite a known work result or
prevent a new attempt solely because an old process/worktree remains. rururunx
is not a security sandbox. Official logged-in host CLIs remain responsible for
authentication, settings, hooks, and their own permission/sandbox enforcement.
No VM, containerized Agent, separate user, privileged service, credential
inspection/copying, or additional outer sandbox is proposed.

The source baseline is main
`6146b1639bf03f244d1c5b716304dc1d20146194`. Claude inventory additionally refers to
the unmerged issue-5 candidate
`f9b671fc110bcabce66d9d4d9c01e098d69d8076`; its behavior is not available on main.
Source references below are relative to those commits, not a changing checkout.
The full contributor inventories are retained in the
[invariant inventory](agent-execution-phase0-invariants.md) and
[refusal inventory](agent-execution-phase0-refusals.md), with adjacent source
manifests. These are investigation inputs, not implementation approval.

Repository instructions were sought at the root (`AGENTS.md`, `CLAUDE.md`);
neither exists at this baseline. README development/workflow conventions, all
seven `doc/design/master/` documents, and issue-4, issue-46, issue-60 reader and
inspection requirements/design were inspected. Their historical ownership
requirements are not silently amended by this investigation. Phase1 must
explicitly supersede the conflicting requirements and preserve unrelated guards.
DB schema and native launch/public API changes require STRICT treatment;
requirements and design must be committed and independently reviewed before
implementation. Phase0 review is a static document review, not a native-provider
acceptance review or a claim that existing open implementation gates passed.

Investigation used source reads and targeted `rg` searches across adapter,
Codex, Git, State, Context, workflow, Goal/project code and the separate Claude
candidate, plus the primary documents linked below (checked 2026-10-05 JST).
No real Agent, quota request, quota-exhaustion experiment, Docker workload,
systemd scope, or process-escape test was run for this Phase0. Prior custody
spikes and prior CI runs do not establish acceptance of this new policy.

## Protection mapping

User requirement **2** means actual review/test inputs come from the recorded
commit SHA, not simply attaching a SHA to a mutable checkout. **3** means a fresh
worktree and branch identity for every execution attempt/retry, with abandoned
ones recorded instead of reused. **4** means Task resource namespaces and an
explicit shared-daemon/Git-maintenance policy. These are cooperative execution
rules; environment variables do not enforce isolation of arbitrary host commands.

| Invariant / surviving-process failure | Current consumer | Protection and remaining gap |
| --- | --- | --- |
| Review decisions and verification describe one stable tree | `adapter.rs:141,630`; `workflow.rs:574,1142`; `context.rs` captures live repository content | 2. Materialize a new verification/review snapshot from the exact commit, or read Git objects directly. A frozen ReviewBundle/revision field does not change the Agent's CWD or subsequent file reads. |
| Reviewed changes are the changes published/merged | `git.rs:201,246` review lock/verification; workflow evidence revision/source checks and external publication ports | 2 plus Runtime publication of the exact accepted SHA. Branch names and live HEAD must not select a later unreviewed commit. Expected-old-ref publication is required new design, not an existing commit/merge implementation in WorktreeManager. |
| Retry does not consume writes from the previous attempt | `git.rs:110` creates issue/task-named bindings; `workflow.rs:2054` retries; State Task binding checks | 3. Current Task worktree binding is durable/immutable, not attempt-specific. Introduce attempt identity; do not merely remove a reservation fence and reuse the path. |
| Review executor cannot change an active executor's workspace | `git.rs:359` executor reservation; `state/mod.rs:432,556`; workflow ReadOnly/Mutating admission | 2/3. Review a separate pinned snapshot; keep identity and CAS admission. Existing advisory review lock protects Runtime operations only. |
| Context, repo maps, rules and budgets describe their claimed source | Context live scans; `workflow.rs:574,2155,2202` | 2 for source snapshots, 3 for executor workspace. Capture rules/config versions and commit input together; changing external/user rules still requires explicit version handling. |
| Terminal callbacks/events cannot complete a newer attempt | Session IDs, native thread/turn checks; workflow/State record versions and CAS | Not provided by 2–4. Keep exact scope/session/attempt/generation and CAS; add attempt fencing where worktrees/resources are rebound. Never equate process cleanup with callback retirement. |
| Runtime crash does not replay an unknown side effect | workflow native dispatch marker and unbound recovery; `state/mod.rs:490`; workflow gate recovery | 3 protects a new local workspace only. Keep durable dispatch/outcome reconciliation. Unknown PR/merge/deploy/external tool effects must not be blindly replayed. |
| Work result survives cleanup uncertainty | `adapter.rs:1590,1596`; `codex/session.rs:590–660`; Claude candidate `session.rs:1350–1438` | Separate axes, outside 2–4. Preserve known native completion/failure; record cleanup unknown/leftovers separately. Lost may still mean genuinely unknown work, but not merely failed reclamation. |
| Failed disposal does not stall later attempts | `git.rs` cleanup locks/removal; workflow Cleanup and terminal recovery | 3 plus a durable cleanup backlog. Do not release/delete an unrelated attempt's resources; do not drop dirty artifacts silently. Cleanup diagnostics cannot demote accepted work. |
| Process cancellation does not stop another Task | owned child/process group; native socket/thread handles | 4 and exact attempt ownership. Retain safe child/handle lifecycle; no machine-wide daemon kill or numeric-PID resurrection. Cookie collection is incomplete and PID reuse needs identity checks. |
| Port use, temp files and build outputs do not collide | Project max_tasks defaults to 4; launch receives environment, no resource allocator found | 4. Durable leases and attempt-specific temp/output/cache paths. Assigned port ranges are advisory: commands must use them; bind conflicts must be handled. Do not recycle uncertain leases immediately. |
| Docker cleanup affects only Task resources | no Task Docker project/label manager found | 4. Unique Task/attempt labels/project names plus inventory. Env does not automatically label `docker run`; raw socket/global prune/shared bind mounts are uncovered. No blanket daemon cleanup. |
| Build-daemon shutdown/cache writes do not affect siblings | no Gradle/Bazel/sccache lifecycle policy found | 4. Disable daemons or select private homes/output bases/cache dirs/endpoints. Runtime-owned shared service is never killed by one Task. Tool-specific propagation must be tested. |
| tmux/SSH persistent helpers do not share stop targets | no Task socket/ControlPath policy found | 4. Private tmux socket, private SSH ControlPath or disable multiplexing. `kill-server`/global connection teardown remains unsafe against shared services. |
| Git maintenance/ref/config/locks do not affect sibling worktrees | `git.rs:78` deliberately accepts the common repository; management uses native Git | 4 can disable inherited automatic gc/maintenance/fsmonitor for Task commands and serialize Runtime publication. Linked worktrees still share ordinary refs/config/object storage; direct global Git mutations and stale lockfiles are not solved by 2/3. Never delete another owner's lock based only on age. |
| Published commit objects remain available | same shared Git object database; revision/evidence stored as strings | 2 needs retained reachable commits and durable result evidence. A SHA fixes content identity, not object availability; GC/corruption/deletion can still remove it. A result store independent from executor write paths requires Phase1 choice. |
| Host user settings/auth/hooks remain intact | adapters use explicit environment/`env_clear`; native permission flows | Not provided by 2–4. Preserve intentional native HOME/config/auth/settings/hook behavior and test it. Do not isolate caches by changing HOME/CODEX_HOME/Claude configuration roots or disable mandatory hooks. |
| Quota exhaustion does not become Task failure or retry storm | no quota state in ErrorKind/Task/session/workflow; error mappings below | First-class shared provider/account quota state, separate waiting reason, reset/backoff and bounded admission. Token usage/context budgets are not subscription capacity. |
| Shared filesystem, external services and resource exhaustion cannot damage other results | host-native commands, hooks, external ports | Not covered by 2–4 alone. Explicitly classify shared writes, absolute paths, symlinks, database migrations, CPU/RAM/disk pressure and external mutations. No sandbox means universal cross-Task exclusion cannot be promised. |

The companion inventories enumerate concrete consumers and refusal families;
the table covers source-level invariant families, not every possible command an
unrestricted CLI could execute. Unknown dependencies/plugins/hooks must be
declared or reported as unsupported resource behavior, not inferred safe.

## Refusal and hold disposition

No guard is removed in Phase0. The following is the proposed Phase1 decision
boundary; enabling effects before its replacement exists would violate the new
request as well as the current implementation contract.

| Current guard family | Disposition under the new policy |
| --- | --- |
| Codex production `Availability::require` always fails; no advertised executable capabilities (`codex/availability.rs:51–69`) | Replace the whole-workload-owner prerequisite with actual production dispatch, requirements 2–4 admission and native capability/conformance checks. Flipping fixture inputs or deleting this check does not implement launch. |
| Repeated availability gates at preparation/transport/RPC/input/grant/checkpoint/resume/attach | Replace the same ownership-only gate consistently; retain operation-specific supported capabilities and permission/approval checks. |
| Codex and Claude terminal cleanup failure converts known completion to Lost | Replace with independent work/cleanup outcomes and persist both. Transport failure before a reliable terminal still produces unknown work. |
| Generic process uncertainty and reservation Drop create Lost | Replace cleanup-only coupling. Keep a genuinely uncertain launch/native outcome distinct; Drop cannot invent successful work. |
| Lost executor prevents review/commit/cleanup/retry (`git.rs:359`; State and workflow fences) | Replace cleanup-only exclusion with attempt/snapshot/resource admission. Keep exclusion against simultaneous writers of the same attempt and uncertain external outcome. |
| Review worktree freeze requires executor termination | Replace with SHA snapshot and isolated reviewer/verifier paths. Keep scope, evidence freshness, protected-base and expected-ref rules. |
| Context/Git-reader Unknown and retained reader capacity (#60) | Freeze ps-based diagnostics. Keep bounded I/O, actual child ownership/reaping and capacity bookkeeping. Change only consumers that interpret reader/workload cleanup as global Task failure; do not discard unknown work/rules/input facts. |
| Unknown native-dispatch marker/unbound reservation (#14/#41) | Preserve durable facts and exact CAS. Allow a new local attempt only after fencing the old attempt and deciding how partial effects are reconciled; do not broadly release every unknown reservation. |
| Unknown external EvidencePort/PR/Merge/Cleanup outcome | Retain outcome reconciliation/idempotency guards. Local worktree replacement does not prove external effects absent. Cleanup backlog policy must distinguish resource disposal from publication. |
| Foreign Project/Goal/Task, stale revision/input/source/rule versions, terminal generation mismatch | Retain; these protect result attribution and admission, not proof of process death. |
| Authentication unavailable, required hook/MCP/settings failure, unsupported native protocol/version, invalid environment/Git root/base branch, unauthorized permission grant | Retain native compatibility/safety checks. Unsupported version needs defined conformance policy; it is not fixed by abandoning process ownership. |

Current main has no complete `rrx run`/Goal scheduler capable of this acceptance
scenario. Claude's adapter is a separate unmerged candidate. Phase2 therefore
needs actual public CLI/runtime wiring and supported provider selection in
addition to changing refusal predicates; constructor/protocol fixtures do not
constitute actual four-Task execution.

## Subscription limits: observable facts and gaps

Codex's documented app-server exposes `account/rateLimits/read` and
`account/rateLimits/updated`. Windows provide usage percentage, duration and
Unix reset time; optional multiple buckets must not be collapsed into a global
balance. The error documentation lists `UsageLimitExceeded` and failed turns.
Treat this as a documented category, not an established wire spelling for the
installed `codex-cli 0.160.0`; Phase1 must pin its generated schema and versions.
A commit review target exists, but its full file-read behavior is not proven by
the request's SHA field. [Official app-server reference](https://learn.chatgpt.com/docs/app-server).

`codex exec --json` documents `turn.failed`/`error` events. Its documented token
usage is consumption, not remaining subscription allowance; saved CLI login is
reused by default. The pages inspected do not establish a quota-specific process
exit code. [Official non-interactive reference](https://learn.chatgpt.com/docs/non-interactive-mode).

Current Codex `native_turn_error` (`session.rs:2795`) classifies authentication,
then every other failure as ProcessFailure. RPC errors retain only a numeric code
(`protocol.rs:143,376`). Rate-limit metadata/notifications are not handled. Adding
quota classification requires preserving structured upstream detail and handling
account-wide events separately from strict per-thread/turn events, without
relaxing attribution for ordinary results and permission requests.

Claude's documented `system/api_retry` provides an error category, HTTP status
and retry delay/attempt counts; `rate_limit` is a retryable API error, not proof
that a subscription window is exhausted. Required hook/MCP startup can be
observed separately. [Official programmatic execution reference](https://code.claude.com/docs/en/headless).

Claude status-line input documents optional five-hour/seven-day usage percentages
and reset times, available after a response for eligible subscriptions. That is
not confirmation of the same metadata in headless stream output. Do not replace
the user's status line or mandatory hooks to obtain it without a compatible
opt-in design. [Official status-line reference](https://code.claude.com/docs/en/statusline).

Claude candidate `protocol.rs:361` accepts but discards `rate_limit_event`.
Its unsuccessful result becomes AuthenticationUnavailable or ProcessFailure
(`protocol.rs:323–348`). The exact installed CLI event payload, exhaustion text,
reset detail, exit status and native waiting behavior have not been reproduced.
Neither provider has quota waiting/resume or remaining-capacity scheduling in
the current runtime. A generic nonzero exit, timeout, 429, context-window limit,
spend cap or authentication failure must not automatically become quota wait.

Phase1 needs KnownAvailable / Exhausted(until?) / Unknown / Stale observations,
account/provider/model bucket identity without credential storage, timestamp and
source provenance. Unknown balance cannot be invented or silently mean zero.
Concurrent sessions share an allowance, including sessions outside rrx; a local
reservation cannot guarantee future provider availability. Reserve reviewer
capacity, limit retries, handle cancellation during waiting, and recheck after
reset. Resume unfinished work from a recorded attempt/checkpoint; don't replay
already completed commits or external effects, or rerun a native still-waiting
session. Authentication refresh stays inside the official CLI.

## Shared resource policy candidates

Use Task and attempt identities together. Ports, temp/build outputs and Docker
labels/names need durable allocation before dispatch and crash reconciliation.
Keep the old attempt's leases/backlog until its resource state is resolved;
allow unrelated Tasks to continue. Finite disk/port exhaustion may still refuse
a new launch before effects: nonblocking leftovers cannot mean infinite capacity.

Gradle: no daemon or private Gradle user home; Bazel: batch mode or private
output-user-root/output-base; sccache: disabled or private cache/server endpoint.
Exact tool/version flags are Phase1 research and Phase3 conformance items, not
verified support. tmux/SSH similarly need private control sockets. Runtime-owned
shared daemons require leases, and per-Task cancellation must not stop them.

For Git, pass scoped configuration to Runtime and task commands rather than
changing user/global config: disable automatic gc/maintenance/fsmonitor where
supported, serialize shared publication, pin result retention, and preserve
repository hooks. This does not cover commands that clear their environment,
explicitly request shared maintenance/ref/config changes, or start global
services. Linked worktrees share ordinary refs and default repository config;
they are not independent repositories. [Official Git worktree reference](https://git-scm.com/docs/git-worktree).

Docker Compose project names do not label arbitrary `docker run` automatically.
Require an explicit supported launch path/wrapper or declare raw delegation
outside the resource contract. Docker is optional; missing daemon must not
prevent plain native Agent execution. Never use global prune/shutdown to clean
one Task. No Docker/cgroup containment guarantee is claimed.

Linux user scope is an optional best-effort cleanup aid; probe actual service
availability/delegation before use and fall back when unavailable. A scope is
not a mandatory `cargo install` prerequisite. macOS retains process groups plus
best-effort cookie discovery. Cookie inheritance/visibility can be lost, APIs
can deny environment reads, snapshots race and PIDs can be reused. Cleanup must
report observations and limitations; absence in a scan is not proof that every
delegated process died. #46/#60 ps observation is frozen, not extended.

## Assumptions requiring a decision before Phase1

1. Accept a cooperative accident model with declared Task resource paths and
   known limits, rather than universal exclusion for arbitrary host commands.
   Same-user native processes can access sibling paths, common Git state and
   external services; SHA/unique paths/env alone cannot prevent that.
2. Decide how accepted results survive common Git mutation/object removal:
   Runtime-retained reachable objects plus durable artifact copies, or an
   independent result repository/store. No same-UID storage is an OS security
   boundary; the contract must state its accidental-mutation assumptions.
3. Require actual pinned review/verification snapshots; define rule/config,
   submodule/LFS/dependency inputs and prohibit using the executor's mutable
   build output as evidence. Do not freeze only the label on a live checkout.
4. Choose a quota-Unknown scheduling policy and a Claude metadata channel that
   preserves existing settings/hooks. Automatic recovery needs confirmed
   signals and bounded rechecks; exact remaining tokens are not guaranteed.
5. Define external-effect reconciliation and shared daemon/Docker opt-out
   behavior. Blindly retrying an unknown PR/merge/tool effect is not authorized
   by making cleanup best effort.

Runtime SIGKILL necessarily interrupts the supervisor for all its Tasks.
The requested test should assert correctness/durability of other Tasks'
commits/reviews and restart reconciliation; continuous scheduling while the
Runtime is dead is not claimed. Single-Task cancellation separately must leave
other active Tasks running. Native auth/settings/hook compatibility and both-OS
four-Task behavior remain Phase3 tests, not findings of this inventory.

## Phase exit

Confirmed by static reads: ownership-only admission and cleanup/result coupling,
Task-level reusable binding, mutable review/context inputs, existing scope/CAS
and external-effect fences, absent Task resource/quota allocation, and the main
versus unmerged Claude distinction. Confirmed from primary documentation:
Codex quota metadata/category, Claude retry/status-line signals, and Git sharing.

Not confirmed: actual CLI quota payload/exit semantics, quota recovery, auth and
mandatory hooks, real >=4 Task parallelism, cancellation/crash correctness,
escape cleanup, Docker or systemd operation, both-platform installation and
native compatibility. The future matrix must record these as not-run/unknown
until executed; unrelated old green CI or fixtures cannot fill those cells.

Stop after this document and its independent static review. Human approval is
required to proceed to Phase1 requirements/design. README Status changes remain
deferred until Phase3, as requested.
