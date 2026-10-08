# Issue 81: rrxd — HOW H1 (H0 revised per Sol 6058699019)

Base: `b2eb694` (#43 merged; Q5 satisfied). Requirements: `doc/design/issue-81-rrxd-requirements.md` (R1, approved 6058083336). Paths are relative to `crates/rrx/src`. This is a design only; no code exists yet.

The H0 review (6058699019) was C/H/M/L = 0/0/7/1, with 3 optional items. O1–O4 were supported with conditions. Every item is addressed below and indexed in §6.

## 0. Delivery in four slices (O4: supported)

The slices run in order: S1, then S2, then S3, then S4. Each slice is its own change with its own controls, PR and green CI, and a slice starts only after the previous one is merged. A completed slice is a limited completion, not MVP completion. Any API a slice exposes keeps the existing identity checks and non-secret projection (R6.3) from that slice onward, not only from S4.

| Slice | Requirements | Content |
| --- | --- | --- |
| S1 | R1, R2.1, R2.2, R6.1, R6.2 | Lifecycle (`daemon start/status/stop`), typed stop outcome, protocol 2 |
| S2 | R4.1, R4.5 | Fixed Project Task limit of 1 |
| S3 | R3.4, R3.5 | Project operations through the API; offline exclusion |
| S4 | R2.3, R2.4, R2.6, R2.7, R5, R6.3 | Read surface (detailed in a later delta) |

R3.1–R3.3, R4.2–R4.4 and R6.4–R6.5 already hold today. Each slice keeps them and re-asserts them in its controls.

## 1. S1 — lifecycle

### 1.1 `rrx daemon start`

1. **Probe.** Run the status procedure (§1.2).
   - `running` → return `already running` with the identity. Nothing is written.
   - `owner busy` or `discovery unavailable` → return that typed result. Nothing is written.
2. **Spawn.** The parent runs `current_exe()` as `serve --state <abs> --config <abs> --detached`.
   - stdin is `/dev/null`.
   - stdout is a **pipe owned by this parent** (the readiness channel, M1).
   - stderr is the log `<state>.execution/daemon/serve.log`, opened with `O_APPEND|O_CREAT|O_NOFOLLOW|O_CLOEXEC`, mode 0600. It lives in a 0700 directory checked like `control/` (`cli/endpoint.rs:43-52`).
   - An existing log file must be a regular file owned by the UID, with no group or other bits and nlink 1, as `private_file` checks (`cli/endpoint.rs:69-89`; OL2). Otherwise the result is typed `start failed`.
   - The working directory is the state's parent. The environment is inherited unchanged (R1.7).
3. **Detach.** The child's first action under `--detached`, before it opens the owner, is `rustix::process::setsid()`. rustix 1.1.5 provides it as a safe function, so no `unsafe` code, no `pre_exec` and no double fork is needed. The child is spawned without `process_group(0)`, so it is not a group leader and `setsid` succeeds. If `setsid` fails, the child exits with a typed code and opens nothing.
4. **Readiness (M1).**
   - After `ControlEndpoint::bind` succeeds and before it accepts connections, the child writes **one** JSON line `{"instance","epoch"}` to stdout. It then points stdout at `/dev/null` with `rustix::stdio::dup2_stdout`, a safe function, so a later write cannot hit a closed pipe.
   - The parent waits up to 30 s for that line, while polling `child.try_wait()`. It returns `running` only when two things hold:
     1. its **own** child sent the line;
     2. `connect(state)` then validates a live identity with the **same** `instance` and `epoch`.
   - A child that exits without the line maps by exit code: 75 → `owner busy`; anything else → `start failed`, with the log path.
   - A winner's endpoint that the parent's own child never announced is never success. It is `owner busy` or `already running`, decided by re-running §1.2.
   - Timeout → `start unconfirmed`. The child is not killed: process exit is never cleanup.
   - A PID is never used as authority. The pipe ties the parent to its own child, and the identity ties it to the published endpoint.

### 1.2 `rrx daemon status` — the decision table (M2, O1)

Inputs:
- **D**, the endpoint:
  - `valid`: the descriptor is read and validated, and the connect and `Hello` identity checks pass;
  - `absent`: no descriptor file;
  - `invalid`: present, but malformed, mismatched, refused, timed out, or an old protocol.
- **L**, the owner-lock probe:
  - the probe takes `flock(LOCK_SH|LOCK_NB)` on `owner.lock`, opened `O_RDONLY|O_NOFOLLOW|O_CLOEXEC` and never created, and releases it at once;
  - `free`: acquired, or the lock file is absent (an owner always holds an existing file);
  - `busy`: `EWOULDBLOCK`;
  - `error`: any other failure.

| D \ L | free | busy | error |
| --- | --- | --- | --- |
| valid | `running` | `running` | `running` |
| absent | `not running` | `owner busy, identity unconfirmed` | `discovery unavailable` |
| invalid | `discovery unavailable` (stale descriptor, reported, not deleted) | `discovery unavailable` | `discovery unavailable` |

- Evaluation order: D first. L is probed only when D is not `valid`.
- `not running` requires both an absent descriptor and a free lock. That is R1's necessary condition, and it never follows from a discovery failure (R1 AC).
- `status` writes nothing: no epoch, no descriptor, no state.
- **O1 (supported).** The shared probe can make a concurrent `RuntimeOwner::open` fail. That `start` reports `owner busy`: typed, with no epoch begun, nothing written, and retriable. The lock is released immediately.
- A missing lock file is **not** treated as a missing state root. A state database without a lock file is simply `free`.

### 1.3 `rrx daemon stop` (M4)

**Pending sites.** All six shutdown "remains pending" exits are classified. Each attaches the private marker `ShutdownPending { site }` as context. The marker's only constructors are at these sites.

| # | Site | Condition |
| --- | --- | --- |
| P1 | `runtime/service.rs:164-167` | Admission lock not acquired within 5 s |
| P2 | `:172-182` | Supervisor join not complete within 5 s |
| P3 | `:183-190` | Driver poll deadline |
| P4 | `phase_supervisor.rs:826-828` | Accepted originals retained |
| P5 | `phase_jobs.rs:1069-1071` | Retained Native jobs |
| P6 | `phase_handoffs.rs:510` | Retained Source handoffs |

Real failures are **not** pending and never get the marker: the poison errors at `phase_supervisor.rs:825`, `phase_jobs.rs:1068` and `phase_handoffs.rs:507`, and any other error. Those stay `Rejected`.

`RuntimeStop` (`runtime/control.rs:428`) answers as follows:

| Result | Response |
| --- | --- |
| `Ok` | `RuntimeStopped { instance, epoch }` (unchanged) |
| Error carrying `ShutdownPending` | `RuntimeStopPending { instance, epoch, site }` (new) |
| Any other error | `Rejected` (unchanged) |

**Failure retention in `serve`.** Today `cli/service.rs:73-83` records `stop_failed` from `response.is_err()`. That changes to "the stop response was not `RuntimeStopped`". Then a pending stop still exits 1, even if a later internal shutdown retry succeeds. The log records the site.

**The CLI** maps a response that never arrives (I/O timeout or EOF) to `stop unconfirmed`.

### 1.4 Protocol 2 and the replacement of an old descriptor (M3)

- `PROTOCOL_VERSION` becomes 2. The live range is `{2}`, with exact equality, as R2.2 allows for MVP.
- `Hello::validate_for` and the descriptor check used by **discovery** (`cli/endpoint.rs:29-40`, `transport.rs:42-50`) stay exact. A protocol-1 peer or descriptor is refused, typed, as `discovery unavailable`, before any request.
- **Replacement inside `ControlEndpoint::bind` (`cli/endpoint.rs:174-178`).** The current code validates a leftover descriptor with the exact validator, so a protocol-1 descriptor left behind by a crash would block a new owner after its epoch began. Under the exclusive owner and endpoint locks, `bind` now uses `read_replaceable_descriptor`. It accepts:
  - `protocol ∈ {1, 2}`;
  - the **same canonical state** and the same strict shape (`deny_unknown_fields`, size bound, private-file checks).

  It then replaces that descriptor. A foreign-state, malformed or oversize descriptor is still refused. No old-protocol communication or negotiation is added.

### 1.5 S1 controls

| Control | Assertion |
| --- | --- |
| C-S1a, independence | `daemon start` while an installed-lane Task is in flight. The test then SIGKILLs the invoker's process group and closes its terminal (pty). `status` stays `running` with the same epoch, the child's `getsid` differs from the invoker's, and the in-flight Task closes with native children owned by the daemon (OL3) |
| C-S1b, race and own-child readiness | 8 concurrent `daemon start`: exactly 1 `running`; 7 `already running` or `owner busy`; the epoch counter rises by 1; one descriptor. **M1:** a loser's child is held at a test pause just before `RuntimeOwner::open`, the winner publishes, then the loser's parent polling resumes. That parent never returns `running`, and it ends with a typed refusal. Mutant: the parent drops the own-child line or identity match → the loser returns `running` → FAIL |
| C-S1c, status table | One case per non-`valid` cell of §1.2: stale descriptor + free → `discovery unavailable`; absent + busy (the test holds the owner lock only) → `owner busy`; mismatch + busy → `discovery unavailable`; absent + free → `not running`. The epoch row and descriptor bytes are unchanged in each case. Mutant: discovery failure maps to `not running` → FAIL |
| C-S1c2, probe vs start | `status` and `start` run concurrently in a loop. Any failed `start` is typed `owner busy`, writes nothing, and the next `start` succeeds |
| C-S1d, stop | completed → `RuntimeStopped`; each pending site P1–P6, driven through an existing held fixture or a site-scoped test pause → `RuntimeStopPending { site }`; a poison error → `Rejected`; pending first, then a retry that succeeds → `serve` exits 1; the client times out → `unconfirmed`. Mutants: remove the marker at any one site → FAIL; keep `stop_failed` on `is_err()` → FAIL |
| C-S1e, protocol | A protocol-1 `Hello` is refused, typed, before the request is decoded. Mutant: accept 1 for live discovery → FAIL |
| C-S1e2, old descriptor | A valid protocol-1 descriptor for the same state with the lock free: the new owner starts and publishes its protocol-2 identity. A foreign-state descriptor is still refused. Mutants: remove the replacement path → FAIL; accept a foreign state → FAIL |
| C-S1f, native success and refusal unchanged | The installed-lane SC suite runs under `daemon start`. Success links equal those under `serve`. An auth or compat refusal case is refused identically (OL3) |
| C-S1g, no TCP | Every socket FD of the daemon process maps to its inode. On Linux, `/proc/<pid>/fd` socket inodes are matched against `/proc/<pid>/net/{tcp,tcp6,udp,udp6}`, which list the network namespace, so the match is by inode and not by namespace membership. On macOS, `lsof -p`. No AF_INET or AF_INET6 socket is owned. The control listener is checked separately as AF_UNIX (OL3) |

## 2. S2 — fixed Project limit of 1 (R4.1, R4.5; O2: supported)

- `const MVP_PROJECT_TASKS: usize = 1` (`config.rs`).
- Config: the default is 1, and any other value is refused, typed ("MVP supports exactly 1 active Task per Project"; `config.rs:337`). The same applies to the Project overlay (`:212`, `:297`) and to `project add --max-tasks N` with N ≠ 1.
- A stored `Project.max_tasks > 1` is never rewritten. Admission skips that Project and raises a typed attention, `ProjectLimitUnsupported { stored }`. Its status still reads. The fix is an explicit `--max-tasks 1`.
- Consumers pass `MVP_PROJECT_TASKS`: `candidates.rs:47`, `claim.rs:247/337`, `task_driver.rs:53`, `native.rs:158`, `quotas.rs:519`, `native_phase/quota.rs:507`. `requested_tasks_per_project` reports 1.
- **PhaseSupervisor (O2).** Its slot counts operations, not distinct Tasks, so it keeps its own constant: today's value, `min(4, MAX_PENDING)`. The same Task's reviewer operations are not serialized (R4.3). The distinct-Task limit is enforced at candidate selection and at the final claim.

| Control | Assertion |
| --- | --- |
| C-S2a, candidate | With 2 runnable Tasks in 1 Project, `ready_driver_candidates` never lists the second while the first is occupied, including a held or unbound occupancy |
| C-S2a2, final claim (L1) | The second Task's claim reaches the claim transaction directly, after candidate selection, through a test seam that only parks: `claim.rs:335-338` refuses it, typed |
| C-S2b | 2 Projects run in parallel (the SC10 shape) |
| C-S2c | The active Task's 2 reviewers run concurrently |
| C-S2d | Config 2, overlay 2 and `--max-tasks 2` are each refused, typed, with no change. A stored 4 → `ProjectLimitUnsupported`, no Driver, row bytes unchanged |
| Mutants (L1) | Candidate limit 2 → C-S2a FAIL. Claim limit 2 → C-S2a2 FAIL. Each limit is changed separately |

## 3. S3 — Project operations (R3.4, R3.5; O3: supported)

### 3.1 API (M5, M7)

New `ControlAction`s. Each declares its R2.6 scope.

| Action | Scope | Contract |
| --- | --- | --- |
| `ProjectRegister { path, options }` | object, new | Creates a Project. Refused, typed, if the path or name is already registered |
| `ProjectUpdate { project, expected_project, options }` | object | Updates an existing registration. The final write checks the client's `expected_project` (id + version) |
| `ProjectRemove { project, expected_project }` | object | The expected version is checked **before** any reconcile, and again inside the final write transaction. A stale expected version is refused, typed, with no row or audit change |
| `ProjectList { all }` and `ProjectStatus { project, cwd }` | read | The implicit reconcile goes through §3.2 |

- **Paths (M7).** The CLI resolves `path` (and `cwd` for status) against the **client's** working directory to an absolute canonical path before sending. The runtime refuses a relative `path`, typed, then re-canonicalizes and verifies the identity: an existing directory and the same Git work tree. Source-relative config, rule and namespace references keep their existing source-root meaning.
- `project add` keeps its CLI shape. It sends `ProjectRegister` when the path is new. When the path is already registered, it reads the current version and sends `ProjectUpdate` with that version.

### 3.2 Preflight outside the Store lock (M6)

The `ProjectRegistry` holds `&mut Store` and reaches synchronous Git (`git.rs:590`), so the daemon never runs it as-is. Each operation splits into two steps, following the master contract (`doc/design/master/multi-project-runtime.md:240`).

1. **Snapshot.** A short Store lock reads the Project row and version.
2. **Preflight.** Git and filesystem validation (reconcile's checks, the source-root and work-tree identity) run in `spawn_blocking`, **without** the Store lock or `control_admission`. They are bounded by a 10 s timeout and the Runtime's `stopping` flag. A timeout gives a typed `project preflight unavailable` and writes nothing.
3. **Commit.** A short `control_admission` and Store lock re-check the identity, the version (expected and snapshot), and that the Runtime is not stopping. Then the write happens, with a CAS on the version. A changed row → typed stale refusal.

Reconcile's `Blocked` write follows the same commit rule.

### 3.3 Offline path (R3.5)

`OwnerLock::exclusive(state)` (new, `execution/owner.rs`) takes the same `LOCK_EX|LOCK_NB` flock on the same `owner.lock`, with the same NOFOLLOW, CLOEXEC, nlink and symlink checks. It never begins an epoch, builds a Runtime, or mints Task or Session authority.

- It is held from before `Store::open` (schema or migration writes, `state/mod.rs:138-277`) until after the last write. A daemon starting meanwhile fails `RuntimeOwner::open` with `owner busy`.
- The offline commands use the same path resolution and expected-version rules as §3.1.
- Their only consumers are the offline Project commands.

### 3.4 S3 controls

| Control | Assertion |
| --- | --- |
| C-S3a | With a daemon running, `project add/remove/list/status` all go through the API. Mutant: the CLI opens the Store directly → the second-writer probe FAILs |
| C-S3b | An offline write is held at a test pause after `Store::open`; `daemon start` → `owner busy`. After release, start succeeds, and the write is visible exactly once |
| C-S3c (M5) | Clients A and B observe version v. A updates. Then B's `ProjectUpdate` and B's `ProjectRemove` are each refused, typed, with the row and audit unchanged. Mutant: replace the client expected version with the latest → FAIL |
| C-S3d (M6) | Project A's Git preflight is held at a test pause. Project B's `ProjectStatus` and a `RuntimeStop` still complete within 5 s. Mutant: hold the Store lock or `control_admission` across the preflight → FAIL |
| C-S3e (M7) | The client's working directory is repository A, the daemon's is repository B, and the client runs `project add .`. Only A is registered. Mutant: drop the client-side resolution → FAIL (B, or a refusal) |

## 4. S4 — read surface (boundaries only; detailed in a later delta)

S4 adds these read-only `ControlAction`s: `ProjectGoals`, `TaskReview`, `TaskSessions`, `AttentionQueue`, `Events` and `Routing`/`Metrics`.

- **Bounds.** Pages follow `runtime_goal_task_page` (`state/runtime/goals.rs:518-597`): `maximum` 1..=128, a typed cursor, foreign cursors refused, `next` only when more rows exist, and a total under `RESPONSE_BYTES`.
- **Events** project `sequence`, `kind`, `at` and scope only. `data` is never returned.
- **Metrics (OL1).** The existing Usage storage (`state/mod.rs:1482`) is legacy and not qualified (`doc/design/issue-21-legacy-scope.md:26`). In MVP, Metrics is typed `Unavailable { MetricsUnqualified }`. The S4 delta decides when recorded usage becomes a qualified metric.
- **Routing** is typed `Unavailable { RoutingUnavailable }`. Task cancel and retry stay `Unavailable { TaskDriverUnavailable }`.

## 5. Decisions (6058699019)

| # | Decision |
| --- | --- |
| O1 | The shared-lock probe is adopted, with the §1.2 table and C-S1c2 |
| O2 | Operation slots are separate from the distinct-Task limit |
| O3 | The offline path is kept under `OwnerLock` for the whole write |
| O4 | S1 → S2 → S3 → S4 in order, each a limited completion |

## 6. H0 findings → H1

| Finding | H1 |
| --- | --- |
| M1: a racing loser returned success | §1.1 step 4 (own-child pipe plus identity match); C-S1b |
| M2: the status table was inconsistent | §1.2 table; C-S1c, C-S1c2 |
| M3: an old descriptor blocked a restart | §1.4 replacement; C-S1e2 |
| M4: incomplete pending sites; `stop_failed` | §1.3, P1–P6 plus failure retention; C-S1d |
| M5: no client optimistic version | §3.1 Register/Update/Remove; C-S3c |
| M6: Git ran under the Store lock | §3.2; C-S3d |
| M7: path resolved against the daemon's working directory | §3.1 paths; C-S3e |
| L1: the candidate-only mutant was not detected | C-S2a2 plus separate mutants |
| OL1: Metrics | §4 |
| OL2: log file checks | §1.1 step 2 |
| OL3: control mapping | C-S1a, C-S1f, C-S1g |
