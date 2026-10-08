# Issue 81: rrxd — HOW design H0 (draft for review)

Base: `b2eb694` (#43 merged; Q5 satisfied). Requirements: `doc/design/issue-81-rrxd-requirements.md` (R1, approved 6058083336). Paths are relative to `crates/rrx/src`.

This is a design only. No code exists yet.

## 0. Delivery in four slices

Each slice is its own design-reviewed change, with its own controls, PR and green CI. A slice does not start until the previous one is merged.

| Slice | Requirements | Content |
| --- | --- | --- |
| S1 | R1, R2.1, R2.2, R6.1, R6.2 | Lifecycle (`daemon start/status/stop`), typed stop outcome, protocol 2 |
| S2 | R4.1, R4.5 | Fixed Project Task limit of 1 |
| S3 | R3.4, R3.5 | Project operations through the API; offline exclusion |
| S4 | R2.3, R2.4, R2.6, R2.7, R5, R6.3 | Read surface, attention queue, events, unavailable routing and metrics |

R3.1–R3.3, R4.2–R4.4 and R6.4–R6.5 already hold today. Each slice only has to keep them, and its controls re-assert them.

## 1. S1 — lifecycle

### 1.1 `rrx daemon start`

1. **Probe.** `connect(state)` (`cli/endpoint.rs:248`). If it validates, return `already running` with the identity. Nothing is written.
2. **Spawn.** Start `std::env::current_exe()` as `serve --state <abs> --config <abs> --detached`:
   - stdin is `/dev/null`;
   - stdout and stderr go to `<state>.execution/daemon/serve.log`, opened `O_APPEND|O_CREAT|O_NOFOLLOW`, mode 0600, in a directory created with mode 0700 and the same ownership checks as `control/` (`cli/endpoint.rs:43-52`);
   - the working directory is the state's parent;
   - the environment is inherited unchanged. That is what keeps native auth, settings and hooks the same as `serve` (R1.7).
3. **Detach.** The child's first action under `--detached`, before it opens the owner, is `rustix::process::setsid()`. rustix 1.1 provides it as a safe function, so no `unsafe` and no `pre_exec` is needed. The child is not a process-group leader: it is spawned without `process_group(0)`, so `setsid` succeeds. A `setsid` failure exits with a typed code and opens nothing.
   - **Double fork is not needed.** After `setsid` the child has no controlling terminal, and the parent's exit reparents it to init or the subreaper. HOW alternative D1: also ignore SIGHUP. It is not needed after `setsid`, so it is left out.
4. **Readiness.** The parent polls `connect(state)` every 50 ms for up to 30 s. In parallel it polls `child.try_wait()`.
   - If the endpoint validates, print `running` with the identity and exit 0.
   - If the child exits first, map its exit code to a typed result: `owner busy` for code 75 (owner lock busy) or `start failed` (others), plus the log path.
   - If time runs out, report `start unconfirmed`. The child is not killed: process exit is never cleanup, and the child may be mid-epoch.
   - The parent never takes the owner lock, so a racing second `start` resolves inside `RuntimeOwner::open` (`execution/owner.rs:217`). Exactly one child wins; the others exit 75 → `owner busy`.

### 1.2 `rrx daemon status` (R1.2)

1. If `connect` validates → `running` plus `RuntimeStatus` (`service_running`, `operational`).
2. Otherwise, probe the owner lock with `flock(LOCK_SH|LOCK_NB)` on `owner.lock`, opened `O_RDONLY|O_NOFOLLOW|O_CLOEXEC` and never created, and release it at once.
   - `EWOULDBLOCK` → `owner busy, identity unconfirmed`.
   - Acquired → `not running`.
   - The lock file is missing → `not running` (no state root).
   - The descriptor exists but failed validation, and the lock is free → `not running`. The descriptor is stale; it is reported, not deleted.
   - **Open question O1.** A shared probe held for microseconds can make a concurrent `RuntimeOwner::open`'s `LOCK_EX|LOCK_NB` fail. That `start` then reports `owner busy`, a typed and retriable result: nothing is written, and no wrong state is claimed. Is that acceptable? The alternative is to report `discovery unavailable` without any lock probe, so `status` would never say `not running`.
3. The connection-level failures from step 1 (refused, timeout, identity mismatch) are reported as `discovery unavailable` whenever the lock probe says busy.

### 1.3 `rrx daemon stop` (R1.3)

- `Runtime::shutdown` (`runtime/service.rs:163`) today returns anyhow only. Add a private typed marker, `ShutdownPending { site }`, attached as context at the five "remains pending" exits (`:164-193`). Its only constructor is in `runtime/service.rs`.
- `RuntimeStop` (`runtime/control.rs:428`):
  - `Ok` → `RuntimeStopped { instance, epoch }`, unchanged;
  - an error that downcasts to `ShutdownPending` → new `RuntimeStopPending { instance, epoch, site }`;
  - any other error → `Rejected`, unchanged.
- The CLI maps a response it never receives (I/O timeout or EOF) to `stop unconfirmed`.
- `serve` (`cli/service.rs:97-116`) already returns `Err` on a pending shutdown. Its exit code stays 1. Under `--detached` the log records the pending reason.

### 1.4 Protocol 2 (R2.2)

- `PROTOCOL_VERSION` becomes 2 (`cli/transport.rs:8`), because S1 adds a response variant.
- `Hello` and `Descriptor` keep `deny_unknown_fields` and exact equality. The supported range is `{2}`, which R2.2 allows for MVP.
- A protocol-1 client fails at `Hello::validate_for`, with a typed error and before any request. No range field is added: with `deny_unknown_fields`, a new field would break old peers in an untyped way.

### 1.5 S1 controls

| Control | Assertion |
| --- | --- |
| C-S1a, independence | `daemon start`, then the test kills the invoker's process group with SIGKILL. `status` is `running` with the same epoch. The child's `getsid` differs from the invoker's |
| C-S1b, race | 8 concurrent `daemon start`. Exactly 1 `running`; 7 `already running` or `owner busy`. The epoch counter rose by exactly 1. No second descriptor; one log file |
| C-S1c, status | Each case: stale descriptor with the lock free → `not running`; lock held with no endpoint (test holds an `OwnerLock` only) → `owner busy`; a mismatched identity → `discovery unavailable`. Each run leaves the epoch row and descriptor bytes unchanged |
| C-S1d, stop | completed → `RuntimeStopped`; a held marked job (existing SC fixture) → `RuntimeStopPending`; the client times out → `unconfirmed`. Mutant: map pending to `RuntimeStopped` → fails |
| C-S1e, protocol | A protocol-1 Hello is refused with a typed error, and the request is never decoded. Mutant: accept 1 → fails |
| C-S1f, native unchanged | The installed-lane SC suite runs under `daemon start`. A Task closes with the same links as under `serve` |
| C-S1g, no TCP | The daemon process has no AF_INET or AF_INET6 sockets (`/proc/<pid>/net` and `fd` scan on Linux, `lsof` on macOS CI) |

## 2. S2 — fixed Project limit 1 (R4.1, R4.5)

- Add `const MVP_PROJECT_TASKS: usize = 1` (`config.rs`).
- **Config.**
  - `scheduler.max_tasks_per_project` defaults to 1.
  - Validation (`config.rs:337`) refuses any other value with a typed error: "MVP supports exactly 1 active Task per Project".
  - The Project overlay (`:212`, `:297`) gets the same check.
- **CLI.** `project add --max-tasks N` with N ≠ 1 is refused with the same typed error.
- **Stored `Project.max_tasks > 1`.** The value is never rewritten. Admission skips that Project with a typed attention, `ProjectLimitUnsupported { stored }`. Its status still reads. The fix is an explicit `project add --max-tasks 1`.
- **Consumers.** These pass `MVP_PROJECT_TASKS`, and stop reading config: `candidates.rs:47`, `claim.rs:247/337`, `task_driver.rs:53`, `native.rs:158`, `quotas.rs:519`, `native_phase/quota.rs:507`. `requested_tasks_per_project` in `RuntimeStatus` reports 1.
- **PhaseSupervisor (`phase_supervisor.rs:312/397`).** Its per-project slot counts pending operations, not distinct Tasks (Sol 6057713206 §0). It gets its own constant, today's value `min(4, MAX_PENDING)`, so the reviewer operations of the one active Task are not serialized (R4.3).
  - **Open question O2.** Confirm that decoupling the supervisor slot from the Task limit is right, rather than tying the slot to 1.

| Control | Assertion |
| --- | --- |
| C-S2a | 2 runnable Tasks in 1 Project: the second never gets a Driver while the first is occupied, including a held or unbound occupancy (an existing held fixture) |
| C-S2b | 2 Projects run in parallel (SC10 shape) |
| C-S2c | The active Task's 2 reviewers run concurrently (an existing reviewer fixture) |
| C-S2d | config 2, overlay 2, `--max-tasks 2`: each is a typed refusal with no state change. Stored 4: `ProjectLimitUnsupported`, no Driver, row bytes unchanged |
| Mutant | Pass a limit of 2 to `candidates.rs` → C-S2a fails |

## 3. S3 — Project operations (R3.4, R3.5)

- **New `ControlAction`s:**
  - `ProjectAdd { path, options }` (the existing `AddProject` fields);
  - `ProjectRemove { project, expected_project }` (object-scoped, R2.6);
  - `ProjectList { all }`;
  - `ProjectStatus { project, cwd }`.
- The runtime runs them through the same `ProjectRegistry` (`project.rs:46/150/165/174/243/261`) under the existing `control_admission`. The existing identity, namespace, validation and idle-removal rules apply unchanged.
- **CLI dispatch** (`main.rs:442-538`): `connect` → API. Otherwise use the offline path under `OwnerLock::exclusive` (§3.1). A busy lock without a validated endpoint → typed `owner busy`, nothing written.
- **3.1 `OwnerLock::exclusive(state)` (new, `execution/owner.rs`).** It takes the same `LOCK_EX|LOCK_NB` flock on the same `owner.lock`, with the same checks: NOFOLLOW, CLOEXEC, nlink and symlink refusal. It never begins an epoch or builds a Runtime.
  - It is held from before `Store::open` (schema or migration writes, `state/mod.rs:138-277`) until after the write. A daemon starting meanwhile fails `RuntimeOwner::open` with `owner busy`: there is no interleaving.
  - It mints no Task or Session authority. Its only consumers are the offline Project commands.
  - **Open question O3.** Should the offline path exist at all in MVP? The alternative is "daemon required": every Project command refuses when no daemon runs.

| Control | Assertion |
| --- | --- |
| C-S3a | Daemon running: `project add/remove/list/status` all go through the API. Mutant: the CLI opens the Store directly → the test's open-handle probe sees a second writer and fails |
| C-S3b | Offline write held at a test pause after `Store::open`, then `daemon start` → `owner busy`. After release, start succeeds, and the write is visible exactly once |
| C-S3c | `remove` with a stale `expected_project` → typed refusal, row unchanged |

## 4. S4 — read surface (R2.3, R2.6, R2.7, R5, R6.3)

New read-only `ControlAction`s:
- `ProjectGoals`;
- `TaskReview`;
- `TaskSessions`;
- `AttentionQueue { after, maximum }`;
- `Events { scope, after, maximum }`;
- `Routing { task }`;
- `Metrics { scope }`.

Rules:
- All are bounded like `runtime_goal_task_page` (`state/runtime/goals.rs:518-597`): `maximum` is 1..=128, the cursor is a typed last id, a foreign cursor is refused, and `next` is set only when more rows exist. The packed response stays under `RESPONSE_BYTES` (4 MiB).
- **Events.** A projection of `Store::events` (`state/mod.rs:1671`) with `sequence`, `kind`, `at` and scope only. `data` is never returned (R6.3). A per-kind allowlist of non-secret fields is a later addition.
- **Attention.** A merged, ordered projection of the R5.1 sources. Each item carries `scope`, `reason` and `allowed_actions` (typed). Reading is side-effect free (R5.2).
- **Routing and Metrics.** Typed `Unavailable { RoutingUnavailable | MetricsUnavailable }` in MVP. Neither is recorded today, and R2.3 allows this.
- **Task cancel and retry.** Keep today's `Unavailable { TaskDriverUnavailable }`.
- **R2.6.** Every new write declares itself object-scoped (with an `expected_*` version) or service-scoped. Reads carry the identity only.
- S4 is designed in detail, in its own delta, after S1–S3. This section only fixes its boundaries.

## 5. Open questions for review

- **O1:** the shared-lock status probe (§1.2), or no probe at all.
- **O2:** the PhaseSupervisor slot decoupled from the Task limit (§2).
- **O3:** keep the offline Project path under `OwnerLock`, or require the daemon (§3.1).
- **O4:** the 4-slice delivery order, with one PR each, starting with S1.
