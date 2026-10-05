# Native Agent execution requirements

Risk: STRICT. Native launch, shared scheduling, persisted schema, Git evidence,
and public adapter contracts change. Requirements and design must be committed
and independently reviewed before implementation. Status: Phase1 proposal;
not implemented or native-qualified. Stop for human approval after Phase1.

## Authority and evidence

The latest request replaces whole-workload custody with protection of accepted
results and best-effort cleanup. This document supersedes conflicting ownership
prerequisites in issue-4, issue-46, issue-60 and the master design **for this new
execution profile when implemented**. Historical verification remains historical;
no existing implementation gate is asserted to have passed.

Source baseline: main `6146b1639bf03f244d1c5b716304dc1d20146194` and unmerged Claude
candidate `f9b671fc110bcabce66d9d4d9c01e098d69d8076`. See the reviewed
[Phase0 report](../verification/agent-execution-phase0.md),
[invariants](../verification/agent-execution-phase0-invariants.md) and
[refusals](../verification/agent-execution-phase0-refusals.md), including their
exact source manifests. Main cannot currently execute production Codex, and
Claude is not on main. Removing a check alone cannot satisfy these requirements.

The guarantees are cooperative and bounded: surviving ordinary detached helpers
using their assigned paths/resources must not change accepted commit/review
content or a new attempt's inputs. Native Agents and hooks have the user's host
authority. rururunx is **not a security sandbox**. Arbitrary sibling-path writes,
raw Docker API calls, global daemon administration, repository destruction,
shared external database changes, and disk/RAM exhaustion are not contained.
Unsupported shared operations must be identified as such; declared dependencies
that prevent the protections below cause refusal before their effects. Environment
variables alone do not enforce compliance. An unknown undeclared dependency
cannot be claimed isolated merely because a Task was admitted.

## R1. Host-native execution and admission

Launch, native completion detection, cancellation and bounded output must work
for logged-in official Claude Code and Codex on macOS and Linux. Use the existing
subscription through the official CLI. Do not read, duplicate or forward its
credentials; do not change HOME or provider configuration roots, disable required
hooks, replace login, or wrap execution in a VM/container/separate user/outer
security sandbox. No root, entitlement, system extension or special service is
required. Optional systemd/Docker facilities cannot be required to install rrx.

Refuse before native launch or preparation effects if a fresh attempt, durable
result capture, pinned review/test input, or required resource policy cannot be
provided (R2–R4). Retain ordinary configuration, permission, risk, protected-base,
scope, protocol-version, bounded-capacity and external-outcome checks. Process
ownership proof is no longer an admission precondition. Report unsupported CLI
versions/capabilities honestly; fixtures cannot advertise production execution.
Preparation helpers and verifier/reviewer helpers receive their own recorded
execution identity and resource namespace, not an untracked global launch path.
Initial base/context preparation must be registered before its first Git effect.
It must not invent Agent success or a retained result to initialize Workflow.
The first native phase may consume the same genuinely prepared namespace only
through exact, atomic Workflow/Context/Task authority checks; recovered rows alone
are not preparation provenance. Preparation-only authority cannot admit native
input. Old writers must not bypass the resulting grant restriction (R8).

## R2. Accepted results and review inputs

Accept execution output only by capturing a complete commit object graph at an
exact full SHA and its base into a Runtime result repository independent of the
executor's shared Git storage. Do not depend on object alternates or executor
hardlinks. Retain reachable references and durable, hashed evidence until the
configured retention policy releases them. SHA identity without retained objects
is insufficient. Uncommitted work is a draft, never reviewed/published evidence.

Each reviewer and verifier materializes a new snapshot from the retained commit,
with an isolated index/workspace and output directory. Record actual materialized
SHA, base, rule/config source versions, tool versions and relevant dependency
manifests. They must not read the executor's changing worktree as authoritative
input. Their own test/build writes must not select a different reviewed revision.
Define explicit submodule/LFS and external-rule handling; refuse unsupported
required content rather than silently review incomplete files. Native settings
and hooks remain supported inputs; arbitrary external mutation is a limitation.

Reviews/verification findings and approval decisions bind to artifact identity,
SHA and exact source versions. Findings do not automatically become facts.
Publish/merge the accepted SHA with an expected-old-target check, serialized by
the Runtime; never select a later branch HEAD. Review result persistence must
survive executor-worktree deletion and Runtime SIGKILL after publication.
Cancellation racing result publication has one transactional winner; record a
late result as draft/diagnostic, never as the newer attempt's accepted output.

## R3. Attempts, worktrees and fencing

Add an immutable attempt identity and monotonically increasing Task generation.
Every execution retry, including quota-driven restart after a terminal native
turn, uses a newly named worktree, branch and resource namespace. Review retries
also receive new snapshots. Never reassign an abandoned path to another attempt.
Task's current binding is an atomic projection; historical Session bindings stay
attached to the attempt that actually launched them.

Fence the old attempt before permitting a new local attempt. Scope, attempt,
generation, Session/native-turn identity and record-version CAS remain required
on callbacks, grants, approvals, result commits and workflow transitions. A
cleanup failure alone must not block subsequent work. Unknown external PR,
merge/deploy or service mutations require idempotent reconciliation; fresh
worktrees do not authorize blind replay. Unknown local work may be retried in a
new namespace with an explicit unknown prior work result.

Dirty or undisposable worktrees are recorded in a cleanup backlog. Never erase
unaccepted drafts without the retention/user policy. Keep their paths and uncertain
resources unavailable for reuse. Resource pressure may cause cancel-aware bounded
waiting or refusal; it must not be disguised as subscription exhaustion.

## R4. Resource independence

Before effects, allocate durable leases for each execution unit (executor attempt,
reviewer or verifier): temp/output/cache paths, a disjoint port range, Docker
project/name/labels, and supported persistent-tool endpoints. Namespace includes
Runtime instance, Project, Task and execution-unit identity. Propagate the same
profile to preparation and helper commands. Reservations survive restart.

Ports are advisory allocations, not pre-bound ownership guarantees. Handle bind
failure and never immediately recycle an uncertain orphan range. Temp paths must
not alias sibling or shared result/state paths. Native HOME/provider roots remain
unchanged. Git auto-gc/maintenance/fsmonitor are disabled for managed Task Git
commands; do not disable required hooks or mutate global user Git configuration.
Common Git ref updates and known maintenance are Runtime-serialized.

Provide real managed invocation policies, not just environment hints, for Docker
run/create/compose labeling and inventory; Gradle/Bazel/sccache daemon shutdown or
private instances; tmux private sockets; SSH ControlMaster disable/private paths.
Task cancellation must never issue global prune, daemon-wide shutdown or remove
another Task's lock/resource. Supported tools/versions and policy propagation
must be explicit. Direct absolute tool invocation, raw sockets, overrides and
shared bind mounts remain unsupported unless declared and reviewed. A user may
explicitly allow a shared operation with its scope and guarantee exception recorded;
this does not make it isolated. An undeclared escape discovered afterward is
reported, not retrospectively prevented or asserted safe.

Linked executor worktrees may share a common Git directory. Define stale-lock
handling without deleting a lock based only on age or a cached PID. If its owner
cannot be established, wait/refuse the affected common-Git mutation while other
independent snapshots/results remain usable. No universal shared-repository or
host-resource isolation claim is permitted.

## R5. Subscription capacity and waiting

Track provider/account/model or limit-bucket capacity separately from consumed
token Usage, ContextBudget, Session concurrency and worker/disk/port capacity.
Use official native metadata only, or a nonsecret user-provided account alias.
Unknown identity conservatively shares one provider account pool; aliases alone
cannot imply different subscriptions. Never inspect credential storage.

Confirmed subscription exhaustion sets the affected scheduling pool and Task to
`WaitingQuota`, not work failure, review rejection or risk escalation. Preserve
native-turn continuation if still retrying; after a terminal turn, checkpoint
only what is supported, fence it and resume in a fresh attempt. Other accounts
or providers may progress. Recover using reported reset/fresh official metadata
or bounded retry with backoff; absence of a reset cannot cause a busy loop.

Do not treat every HTTP 429, transient rate limit, transport/auth error or text
substring as subscription exhaustion. Version-pin structured observations and
report unknown classification. Codex account metadata may expose windows/reset
times; Claude headless exposes retry events but equivalent remaining subscription
capacity is not established. Do not modify statusline or hooks to obtain it.

Admission must consider fresh known capacity, existing executor/reviewer leases,
fairness and configurable review headroom. With unknown capacity, use explicit
bounded concurrency/backoff and report `Unknown`, not fabricated remaining tokens
or indefinite blanket refusal. Support a configured four-executor mixed-provider
run when host/account capacity permits. A quota observation received after a
lease is released may update its exact account window but not complete another
Task. Waiting Tasks can be cancelled; quota wait is not failure. Review fallback
cannot silently remove required reviewers or switch to paid API billing.

## R6. Best-effort reclamation

On cancellation, retire logical authority first, then attempt direct-child/group
stop, cookie discovery and owned-resource cleanup. Preserve owned unreaped child
identity and PID-start identity; cached numeric PID/PGID never authorizes a signal.
Cookie visibility and scans are incomplete and non-atomic. Do not broaden or
extend issue-46/60 ps-based ownership observations. Optional Linux user scopes
may aid collection when usable; non-systemd/container environments use the same
non-guaranteeing fallback. A cookie is provenance, not a security boundary.

Attempt Docker cleanup by exact managed labels/recorded IDs. Resource observations
record coverage, timestamp, attempted action, remaining IDs and bounded errors.
No Docker availability means its scope is unavailable/unknown, not globally empty.
Runtime crash recovery reconciles ledger and owned resources on restart; cleanup
continuity while Runtime is dead is not promised. Do not signal historical cached
handles or adopt unrelated legacy workloads. Unknown locks/resources remain held
or operator-reconcilable; cleanup must be bounded and independently schedulable.

## R7. Independent work and cleanup outcomes

Persist both axes per execution unit and aggregate them per Task:

| Axis | Values | Meaning |
| --- | --- | --- |
| Work | success / failure / unknown | Reliable native work terminal and/or accepted workflow evidence; native exit zero alone does not pass tests/review/merge gates. Cancellation is a separate disposition and may leave work unknown. |
| Cleanup | reclaimed / leftovers / unknown | Observed status of tracked resources at the recorded time. Reclaimed never means complete absence of arbitrary descendants or delegated effects. |

Cleanup never overwrites work or accepted evidence. A successful Task may have
leftovers; a failed Task may be fully reclaimed. Unavailable inspection is unknown.
Native completion and workflow success are distinct. Persist observations even
when Task advances; late cleanup updates affect only their historical unit.

## R8. Durable transitions and migration

Extend existing Session/Task/reservation/workflow with attempts, result artifacts,
resources, quota pools/leases/waits and cleanup observations. Register intent
before each managed effect; use explicit pending/confirmed/unknown reconciliation
across SQLite/file/native boundaries. Never keep a SQL transaction locked while
awaiting native I/O. Commit publication, cancellation and generation advancement
use one authoritative CAS transaction. Restart fences old scheduling authority
without confusing lease timeout with physical death.

Define a schema-version migration with scope FKs, unique live leases, append-only
audit, rollback on malformed data and future-version refusal. Legacy Lost/unknown
records remain unknown; they cannot be promoted to success or safely signaled by
reinterpretation. Restart must not issue duplicate unresolved external operations.
Old readers must reject the newer schema rather than silently write it.

## R9. Replacement boundary and delivery

Remove ownership-only availability refusal and known-work-to-Lost coupling only
after their replacement admission, artifact, attempt and resource paths exist.
Replace cleanup-only whole-Task exclusion in Git/State/workflow; retain exclusion
for competing writes to the same execution unit, stale evidence, unknown external
effects, invalid scope/CAS, protected branches, permissions and resource ceilings.
Freeze the old process observer rather than expand it. Separate actual reader I/O
or unknown work facts from reader reclamation diagnostics. Preserve safe child
reaping, bounded output/reader capacity, permission routing and independent review.

Implement enough Runtime scheduling and CLI run/status/stop/resume/report plumbing
to exercise this contract; the current project/config CLI is insufficient. Keep
the workspace `unsafe_code = "forbid"`; any necessary OS unsafe boundary belongs
to an independent crate without an rrx dependency. Do not change license files
or descriptions. README Status changes only after Phase3, with verified behavior
and known limitations. `cargo install` must install all required rrx helpers with
no privileged setup; official logged-in CLIs remain external prerequisites.

## Acceptance and phase gates

Phase1 requires requirements review, then design review, both against immutable
commits, with verified findings and re-review after fixes. Static review is not
native-provider acceptance. Stop after each phase for explicit human approval.

Phase2 sequence: native adapter dispatch, commit capture/fresh snapshots and
attempts, resources, quotas, reclamation. Do not release an intermediate adapter
that bypasses R2–R4. Commit before appropriate STRICT verification/re-review.
Phase3 records actual OS/CLI versions and results, including limitations:

| Case | Required observation / coverage |
| --- | --- |
| Native concurrent run | Same Runtime, at least four overlapping Tasks, both Claude and Codex; actual start/terminal/commit/review identities and distinct leases on macOS and Linux. |
| Cancel one Task | Siblings continue; accepted commits/review hashes remain correct; cancelled unit cannot submit a late accepted result or approval. |
| Runtime SIGKILL | Previously durable results survive; restart fences prior sessions and resumes/reconciles safely. No claim of scheduler continuity during SIGKILL. |
| Detached helpers | nohup, setsid where available, detached Docker managed run; survivors write their old namespace, never become retry input; cleanup observations and unsupported paths reported. |
| Reviewer/verifier mutation | Change executor after snapshot creation; every reviewer/test still consumes its declared SHA. Include concurrent snapshot writes and result retention after executor disposal. |
| Resource policy | Port collision, dirty worktree, orphan leases, common Git lock, Gradle/Bazel/sccache/tmux/SSH policies, Docker inventory before create and Task-only deletion. Unsupported tool versions reported. |
| Quota | Confirmed exhaustion waits without failure; no retry storm, sibling account progress, reset/recheck and fresh-attempt resume, reviewer capacity/fairness, cancelled waiter and unknown metadata. Injected protocol fixtures and actual exhaustion observations reported separately. |
| Compatibility | Actual logged-in Claude/Codex auth, user/project settings, required hooks and their own sandbox/permission behavior; no credential inspection. |
| Crash boundaries | Intent-before-spawn, native launch before acknowledgement, capture before DB publication, cancel/publication race, generation replacement, Docker creation acknowledgement lost, quota lease and restart. |
| Installation | Clean `cargo install` on each OS, helper resolution, default no-systemd fallback; no preinstalled hidden helper or privileged service. |

GitHub-hosted CI can validate synthetic adapter/protocol/crash/resource cases and
available native OS APIs. Hosted runners do not establish the user's authenticated
subscription/hook compatibility; Docker/systemd/tool availability must be probed
and skips named. Controlled actual-account runs supply the remaining native rows.
An unmet or skipped row is recorded as unverified/unsupported, never a pass.
