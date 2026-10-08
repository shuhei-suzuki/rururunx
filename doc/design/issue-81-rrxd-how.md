# Issue 81: rrxd — HOW H2 (H0 per Sol 6058699019, H1 per Sol 6059215697)

Base: `b2eb694` (#43 merged; Q5 satisfied). Requirements: `doc/design/issue-81-rrxd-requirements.md` (R1, approved 6058083336). Paths are relative to `crates/rrx/src`. Approved as H2 (Sol 6059595718). S1 is implemented (§7); S2–S4 are design only.

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

1. **Probe.** Read the D and L inputs of §1.2. `start` acts on them, not on the `status` label (H1 M3):

   | D \ L | free | busy | error |
   | --- | --- | --- | --- |
   | valid | `already running`, with the identity | `already running` | `already running` |
   | absent | spawn | `owner busy` | `discovery unavailable` |
   | invalid | **spawn** | `discovery unavailable` | `discovery unavailable` |

   - Spawning on `invalid` + `free` is what lets a crashed service's leftover descriptor be replaced. It applies to a protocol-1 descriptor and to a protocol-2 descriptor alike.
   - The spawned child takes the exclusive owner, and `ControlEndpoint::bind` then applies §1.4: it replaces a known-shape descriptor for the same canonical state, and refuses a foreign or malformed one with a typed `start failed` and the log path.
   - `status` still reports `invalid` + `free` as `discovery unavailable` (§1.2). Only `start`, which ends up holding the exclusive owner, acts on it.
   - Every refusal in this table writes nothing.
2. **Spawn.** The parent runs `current_exe()` as `serve --state <abs> --config <abs> --detached`.
   - stdin is `/dev/null`.
   - stdout is a **pipe owned by this parent** (the readiness channel, M1).
   - stderr is the log `<state>.execution/daemon/serve.log`, opened with `O_APPEND|O_CREAT|O_NOFOLLOW|O_CLOEXEC`, mode 0600. It lives in a 0700 directory checked like `control/` (`cli/endpoint.rs:43-52`).
   - An existing log file must be a regular file owned by the UID, with no group or other bits and nlink 1, as `private_file` checks (`cli/endpoint.rs:69-89`; OL2). Otherwise the result is typed `start failed`.
   - The working directory is the state's parent. The environment is inherited unchanged (R1.7).
3. **Detach.** The child's first action under `--detached`, before it opens the owner, is `rustix::process::setsid()`. rustix 1.1.5 provides it as a safe function, so no `unsafe` code, no `pre_exec` and no double fork is needed. The child is spawned without `process_group(0)`, so it is not a group leader and `setsid` succeeds. If `setsid` fails, the child exits with a typed code and opens nothing.
4. **Readiness (M1).**
   - After `ControlEndpoint::bind` succeeds and before it accepts connections, the child writes **one** JSON line `{"instance","epoch"}` to stdout. It then points stdout at `/dev/null` with `rustix::stdio::dup2_stdout`, a safe function, so a later write cannot hit a closed pipe.
   - The parent waits up to 30 s for that line, while polling `child.try_wait()`. It returns `running` only when two separately checked conditions hold:
     1. **own-child line:** its own child sent the line;
     2. **identity match:** `connect(state)` then validates a live identity whose `instance` and `epoch` equal the line's.
   - If the line arrived but the live identity differs (H1-L1), for example because the child retired and a new epoch was published in between, the parent never reports it as its own start. It re-runs the probe: a valid endpoint → `already running` with that other identity, otherwise `start unconfirmed`.
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
| C-S1b, race and own-child line | 8 concurrent `daemon start`: exactly 1 `running`; 7 `already running` or `owner busy`; the epoch counter rises by 1; one descriptor. **M1:** a loser's child is held at a test pause just before `RuntimeOwner::open`, the winner publishes, then the loser's parent polling resumes. That parent never returns `running`, and it ends with a typed refusal. Mutant: drop the **own-child line** condition → the loser returns `running` → FAIL |
| C-S1b2, identity match (H1-L1) | The parent has received its own child's line A. A test pause in the parent, after the last `try_wait` and before `connect`, holds it while the test stops A and starts a new epoch B that publishes. The parent resumes. It never returns `running` as its own start: it returns `already running` with identity B. Mutant: drop **only** the identity comparison → the parent returns `running` for B → FAIL |
| C-S1c, status table | One case per non-`valid` cell of §1.2: stale descriptor + free → `discovery unavailable`; absent + busy (the test holds the owner lock only) → `owner busy`; mismatch + busy → `discovery unavailable`; absent + free → `not running`. The epoch row and descriptor bytes are unchanged in each case. Mutant: discovery failure maps to `not running` → FAIL |
| C-S1c2, probe vs start | `status` and `start` run concurrently in a loop. Any failed `start` is typed `owner busy`, writes nothing, and the next `start` succeeds |
| C-S1d, stop | completed → `RuntimeStopped`; each pending site P1–P6, driven through an existing held fixture or a site-scoped test pause → `RuntimeStopPending { site }`; a poison error → `Rejected`; pending first, then a retry that succeeds → `serve` exits 1; the client times out → `unconfirmed`. Mutants: remove the marker at any one site → FAIL; keep `stop_failed` on `is_err()` → FAIL |
| C-S1e, protocol | A protocol-1 `Hello` is refused, typed, before the request is decoded. Mutant: accept 1 for live discovery → FAIL |
| C-S1e2, old descriptor, from the `daemon start` entry (H1 M3) | Through `daemon start`, not `bind` alone. A valid protocol-1 descriptor for the same state with the lock free → `running` with the new protocol-2 identity. A leftover protocol-2 descriptor from a crash → the same. A foreign-state or malformed descriptor → typed `start failed`, nothing published. Mutants: the probe returns on any `invalid` → FAIL; remove the `bind` replacement path → FAIL; accept a foreign state → FAIL |
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
2. **Preflight.** Git and filesystem validation (reconcile's checks, the source-root and work-tree identity) run in `spawn_blocking`, **without** the Store lock or `control_admission`. They are bounded by a 10 s timeout and the Runtime's `stopping` flag.
   - A timeout gives a typed `project preflight unavailable` and writes nothing.
   - Aborting a started `spawn_blocking` does not stop its work (Tokio 1.53.1, noted in 6059215697). So each Git child runs in its own process group and is owned. On timeout or stop, the group is killed and reaped before the preflight handle is released.
   - The timeout response never claims that the work has ended, and no commit runs while a preflight for the same Project is still unreaped.
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
| H1 M3: the start probe blocked replacement | §1.1 step 1 start table; C-S1e2 from the `daemon start` entry |
| H1-L1: the identity-only mutant was not detected | §1.1 step 4, two separate conditions; C-S1b2 |
| H1 note: M6 timeout is not termination | §3.2 preflight: owned process group, killed and reaped |
| OL1: Metrics | §4 |
| OL2: log file checks | §1.1 step 2 |
| OL3: control mapping | C-S1a, C-S1f, C-S1g |

## 7. S1 outcome (implementation)

Branch `claude/adoring-archimedes-7eehnw`, on top of PR #82's head (`1d7843e`). Paths are relative to `crates/rrx`.

| Design item | Implementation |
| --- | --- |
| §1.1 probe and start table | `src/cli/daemon.rs` `start_with`. It acts on D (`endpoint::discover`) and L (`probe_lock`), and spawns on `absent+free` and on `invalid+free` |
| §1.1 spawn and log | `serve --detached` from `current_exe()`. stdin is `/dev/null`, stdout a parent-owned pipe, stderr `<state>.execution/daemon/serve.log` (0700 directory; 0600 file opened `NOFOLLOW` and checked like `private_file`) |
| §1.1 detach | `main.rs` runs `service::detach()` (`rustix::process::setsid`) as the first action under `--detached`. On failure it exits 71 and opens nothing |
| §1.1 readiness | `service::announce` writes one `{"instance","epoch"}` line after `bind`, then `dup2_stdout(/dev/null)`. The parent needs the own-child line and a separate live identity match. Exit 75 (owner busy) re-runs D. Timeout gives `start unconfirmed` without killing the child |
| §1.2 status table | `daemon::status`: D first, L only when D is not valid. The exit codes are 0 running, 3 not running, 4 owner busy or discovery unavailable |
| §1.3 stop | `src/runtime/stop.rs` defines `ShutdownPending { site }`. It is constructed only at P1–P6, and the message text is unchanged. `RuntimeStop` answers `RuntimeStopPending { instance, epoch, site }`. `serve` keeps any stop that is pending or stopping without `RuntimeStopped` as a failure. In the CLI, EOF or timeout after sending is `stop unconfirmed` |
| §1.4 protocol 2 | `PROTOCOL_VERSION = 2`, with exact live discovery and `Hello`. `bind` uses `read_replaceable_descriptor`: protocol ∈ {1, 2}, the same canonical state and the strict shape |
| Feature | The workspace `rustix` gains `stdio` for the safe `dup2_stdout`. `Cargo.lock` is unchanged |

### Controls

| Control | Test | Result |
| --- | --- | --- |
| C-S1a | `tests/daemon_native.rs` `c_s1a_daemon_survives_invoker_group_and_terminal`: a pty invoker; an in-flight implement held at the peer; SIGKILL of the invoker group plus a closed master | pass. Same identity; the daemon is its own session leader, in a session other than the invoker's; the native peer descends from the daemon; links `session_bound → gate_claim → gate_observed → phase_closed` |
| C-S1b | `src/cli/daemon/tests.rs` `c_s1b_concurrent_starts_one_runs_and_held_loser_never_claims_running`: 8 concurrent starts, plus a loser whose own child is held by its exec wrapper before `RuntimeOwner::open` | pass. 1 running, the rest `already running`/`owner busy`; epoch 1, then 2 on the next start; one descriptor |
| C-S1b2 | `c_s1b2_identity_match_refuses_a_replacement_epoch`: a parent pause after the line, before the match; A stopped, B published | pass. `already running` with B |
| C-S1c | `tests/daemon.rs` `c_s1c_status_table_cells_and_no_effect` | pass. Every non-valid cell; epoch row and descriptor bytes unchanged |
| C-S1c2 | `c_s1c2_concurrent_status_never_corrupts_start` (10 rounds) | pass |
| C-S1d | P1 `c_s1d_p1_held_admission…` (held admission), P2 `c_s1d_p2_parked_loop…` (site-scoped loop park), P3 `ca4g…`, P4 `c_s1d_p4_accepted_original…`, P5 `actual_publication_drop…`, P6 `original_empty_reservation…`, all through `RuntimeStop`; poison `c_s1d_poisoned_handoffs…`; `c_s1d_serve_exits_unsuccessfully_after_a_pending_stop`; `c_s1d_stop_without_response_is_unconfirmed_and_refusal_is_failed` | pass |
| C-S1e | `src/cli/endpoint.rs` `c_s1e_protocol_1_hello_is_refused_before_any_request` | pass. The peer reads no request |
| C-S1e2 | `tests/daemon.rs` `c_s1e2_daemon_start_replaces_known_leftovers_only` (through `daemon start`, including a SIGKILL crash) and `endpoint` `c_s1e2_bind_replaces_known_leftovers_and_refuses_foreign` | pass |
| C-S1f | `tests/daemon_native.rs` `c_s1f_native_success_and_refusal_equal_under_serve_and_daemon` | pass. Success links equal; the compat refusal projection (links, `native_invocations`, Session records, readiness) is equal and empty |
| C-S1g | `tests/daemon.rs` `c_s1g_daemon_owns_no_inet_socket` (Linux `/proc` inodes; macOS `lsof`) | pass |

### Mutants (each restored; tree clean)

| Mutant | Detected by |
| --- | --- |
| MB: an endpoint the own child never announced is reported as `running` | C-S1b FAIL |
| MB2: the identity comparison is dropped, and only that | C-S1b2 FAIL |
| MC: a discovery failure maps to `not running` | C-S1c FAIL |
| MD1–MD6: the marker is removed at P1 / P2 / P3 / P4 / P5 / P6 | the matching C-S1d site test FAILs (P2 also fails the `serve` exit test) |
| MDF: `stop_failed` is kept on `is_err()` | `c_s1d_serve_exits_unsuccessfully_after_a_pending_stop` FAIL |
| ME: protocol 1 is accepted for live discovery | C-S1e FAIL |
| ME2A: the probe returns on any `invalid` | C-S1e2 (`daemon start`) FAIL |
| ME2B: the `bind` replacement path is removed | C-S1e2, both tests FAIL |
| ME2C: a foreign-state leftover is accepted | C-S1e2, both tests FAIL |

### Source review

| Round | Result |
| --- | --- |
| 6061653755 → Sol 6062003954 | REQUEST CHANGES, required 0/0/1/0. M1: a FIFO `owner.lock` held the probe in `open(2)` |
| `34cd2e7`, `69337b6` → Sol 6062762783 | **APPROVE LIMITED, 0 findings**. The probe opens `O_NONBLOCK` and requires a regular file before `flock`. Control `owner_lock_fifo_is_a_typed_probe_error_for_every_command`: all three commands return typed `discovery_unavailable` within a deadline. The mutant without `O_NONBLOCK` FAILs |

The endpoint test failure seen once (`actual_endpoint_connects…`, rebind line) remains unexplained. It passed 15 consecutive runs under the same filter set. It is recorded as an open observation, not as a flake.

### Notes

- After a pending stop, `serve` itself retries shutdown and exits 1. This is the existing behaviour, and it is now asserted: the process exit is not counted as cleanup.
- The P2 park (`runtime::stop::park`) is `cfg(test)` only. It holds the real loop, so the join genuinely misses its deadline. It never constructs the marker.
