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

### 2.1 S2 implementation delta (D1–D8, for review before code)

The design above stays as it is. This delta fixes the open implementation points.

| # | Point | Decision |
| --- | --- | --- |
| D1 | Typed config refusal | New `ProjectTaskLimitRefused { origin, requested }`, where `origin` is `RuntimeConfig`, `ProjectOverlay` or `CliFlag`. The message stays "MVP supports exactly 1 active Task per Project". `Config::validate` raises it for a value other than 1 (`config.rs:337`). So does `apply_project`, before any field is mutated (`:297`). `SchedulerConfig::default()` becomes `MVP_PROJECT_TASKS`, and an explicit `= 1` is still accepted. A value of 0 keeps its existing "scheduler limits must be positive" refusal |
| D2 | `project add` | `--max-tasks N` with N ≠ 1 is refused with `ProjectTaskLimitRefused { origin: CliFlag }` before `reconcile` or any write. A new Project stores `MVP_PROJECT_TASKS`, and the `Project::new` default becomes 1. Re-adding an existing Project without `--max-tasks` keeps the stored value byte for byte. `--max-tasks 1` is the only rewrite, and it is the explicit fix |
| D3 | Stored > 1 at admission | Nothing is written: no Project row, no `scheduler_tasks.attention`, no Driver. A Project whose stored `max_tasks` is not 1 is excluded in the `ready_driver_candidates` SQL by joining `projects` on `json_extract(body,'$.max_tasks') = MVP_PROJECT_TASKS`, so it never uses up the 32-evaluation budget. `plan_initial_driver` re-reads the Project in its snapshot and refuses with a typed `ProjectLimitUnsupported { stored }` as a second guard |
| D4 | The "typed attention" | It is derived when status is read, not stored. `UnavailableReason` gains `ProjectLimitUnsupported`. `GoalFacts` and `GoalProposalFacts` report it when the Project's stored limit is not 1, and status still reads. The stored value goes in a new optional `project_limit_stored` field, so the existing unit-enum encoding of `UnavailableReason` keeps its shape |
| D5 | Final-claim refusal (L1) | `claim.rs:337-338` changes from a string `ensure!` to a typed `DriverCapacityUnavailable { scope: Global \| Project }`, and the Project bound is `MVP_PROJECT_TASKS`. The seam for C-S2a2 is `cfg(test)` only. It parks admission after candidate selection and before planning for a named Task, and it never constructs authority or skips a check. The test lets Task A claim, then releases B's parked evaluation, and asserts the typed `Project` refusal with no `task_drivers` row for B |
| D6 | Consumers | `MVP_PROJECT_TASKS` replaces the config value at `admission.rs:62`, `task_driver.rs:53`, `control.rs:459` (reports 1), `native.rs:158`, `quotas.rs:519` and `native_phase/quota.rs:507`. The last two use `min(MVP, stored)`, which is 1 for every admitted row. `PhaseSupervisor` (`runtime/mod.rs:67`) gets its own `PHASE_SLOTS_PER_PROJECT = 4` (O2) |
| D7 | Existing rows and tests | Projects registered before S2 store `4`, the old default. Each is reported as `ProjectLimitUnsupported` until `project add <path> --max-tasks 1`; this is the intended R4.5 behaviour and is not migrated. Test fixtures that set `max_tasks_per_project` directly change as follows. `activation/pages.rs:8` uses 128 Projects with one Task each, so it drops the line and keeps its meaning. The `phase_supervisor/tests.rs` fixture drops its `per_project` argument. Its per-Project pending refusal test (`:376`, today limit 1 with 2 operations) becomes `PHASE_SLOTS_PER_PROJECT` operations admitted and the next one refused, so it asserts the same bound. `tests/cli.rs:63-64` and `tests/project.rs:350` become refusal controls under C-S2d, and the overlay merge assertions keep their other fields |
| D8 | Effective config (Sol 6070180593 M1) | `project::effective_config` (`project.rs:567-576`) no longer copies the stored `Project.max_tasks` without a check. A stored 1 returns `MVP_PROJECT_TASKS`. Any other stored value returns the typed `ProjectLimitUnsupported { stored }` without touching the row, and status still reads. The README configuration example (`README.md:366`) changes to `max_tasks_per_project = 1`, so `readme_configuration_example_is_valid` (`tests/cli.rs:132`) still runs it through the real loader. Control C-S2e: a stored 4 gives the typed refusal with the row, version and audit unchanged; an explicit `--max-tasks 1` repair then gives 1. Mutant: restoring the unchecked copy makes C-S2e fail |

Conditions from the review (6070180593), applied to the controls:

- **D1/D2.** The overlay refusal runs before the first field mutation in `apply_project`. The CLI refusal runs before `reconcile` and before `Store::open`'s initialisation writes. The controls cover these cases separately:
  - `origin` and `requested`;
  - an explicit 1 succeeds;
  - a config or overlay 0 keeps the positive-limit refusal;
  - a re-add with no argument keeps a stored 4;
  - only an explicit 1 repairs it.
- **D3.** The SQL keeps a fixed query with bound values, and the existing cursor, order and occupancy conditions. The `projects` join on Project ID and the fixed-1 filter apply before the eligible `LIMIT`. C-S2a3 mixes many stored-4 Projects with supported ones and asserts that only the supported ones are listed. The snapshot guard in `plan_initial_driver` gets its own control: a typed refusal with nothing written.
- **D4.** An accepted Goal and an inert proposal both report the reason and the stored value. The answer is the same before and after `reconcile_runtime_attention`. The read changes no row, version or audit. After the repair the extra reason is gone. The plain `goal status` renderer (`cli/goal_facts.rs:105`) also prints the reason, the stored value and the `--max-tasks 1` repair (the optional item, taken).
- **D5, the harness.** `admit_ready_tasks` holds `control_admission` and the cursor from candidate selection through evaluation, so B is not parked inside it. The test:
  1. reads B's key through the production reader;
  2. waits while holding no admission, cursor or Store guard;
  3. lets A claim through the normal path;
  4. takes the normal admission guard and passes B's key to the same production evaluator, as the existing sweep controls do (`installation/tests/sweep.rs:9`).

  Global capacity is free and A's occupancy remains. B must reach the `Project` variant, with no new Driver row and no new claim audit for B. The deciding mutant changes only the claim's Project bound to 2. A candidate check or an early refusal alone does not pass.
- **D6/D7.** Both quota boundaries keep `min(MVP, stored)` and the own-Task check. The pending-bound control keeps global headroom. It still tells apart four operations followed by the next per-Project refusal, another Project's progress, the global refusal, and the retry of the original objects.

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

### 3.5 S3 implementation delta (D1–D10, for review before code)

§3.1–§3.4 stay as approved. This delta settles the implementation points they leave open. Round 1 (Sol 6077182128) is applied below: M1 → D1, M2 → D3, M3 → D10, M4 → D8, L1 → C-S3g, Q1 → D1, Q2 → D8. Round 2 (Sol 6077529665): L1 → C-S3d3/C-S3d4, with C-S3g's mutant branches named. Paths are relative to `crates/rrx`.

| # | Point | Decision |
| --- | --- | --- |
| D1 | Wire and protocol | **Actions.** New `ControlAction`s: `ProjectLookupRoot { root }`, `ProjectRegister { path, options }`, `ProjectUpdate { project, expected_project, options }`, `ProjectRemove { project, expected_project }`, `ProjectList { all }` and `ProjectStatus { selector, cwd }`. `options` is a new serde type, `ProjectOptions`. It mirrors today's `AddProject` field for field (name, base, rules, env refs, worktree root, `max_tasks`, clears, `config_ref`). D3 sets which of these paths are absolute.<br><br>**Responses (M1).** Every response is an explicit, non-secret projection. `ProjectView` carries the `Project` row's own fields (id, name, state, version, root, base, worktree root, blocked reason, refs). `ProjectStatusFacts` carries the `ProjectView`, the active goal and task counts, and `sessions: Vec<SessionView>`. `SessionView` is `{ id, task, agent, provider, role, state, started_at }`. `native_ref`, `pid`, `recovery`, `worktree`, `model` and `effort` are never returned. No response serializes a `Goal`, `Task` or `Session` body.<br><br>**Output.** The plain-text output keeps today's lines. `project status --json` becomes this projection on both the API and the offline paths, so it is no longer byte-identical to today's raw `ProjectStatus`. That change is intended.<br><br>**Typed reasons.** New `UnavailableReason`s: `ProjectPreflightUnavailable`, `ProjectPreflightInFlight`, `ProjectCurrencyChanged` and `ProjectPathNotAbsolute`.<br><br>**Protocol (Q1).** `PROTOCOL_VERSION` becomes **3**. The exact live set is `{3}` and the replaceable set is `{1, 2, 3}`. Replacement still happens only under the exclusive owner and endpoint locks, for the same canonical state and the strict shape (S1 §1.4). A live protocol-2 descriptor or `Hello` is `discovery unavailable`. It never implies a stop or permission for an offline write. The S1 controls that pin protocol 2 (C-S1e, C-S1e2) move to 3, and protocol 2 becomes their refused case |
| D2 | CLI routing (R3.4, R3.5) | `rrx project …` decides once per command, in this order:<br>(a) D (`endpoint::discover`) is `Valid`, so the command goes through the API.<br>(b) D is `Absent`, so the CLI takes `OwnerLock::exclusive` (D7) and runs offline.<br>(c) The exclusive lock is busy, so the command fails typed `owner busy`, exit 4, with no write.<br>(d) D is `Invalid`, so the command fails typed `discovery unavailable`, with no write.<br><br>There is no direct-Store fallback after any API error. `main.rs:502` is the only offline `Store::open` call in the CLI (checked), and it moves behind (b) |
| D3 | Paths (M7, M2) | **Client side.** The CLI canonicalizes `path` and `cwd` against its own working directory before sending them. `--project-config`, rules and env refs keep their source-relative meaning. They are sent exactly as given, relative or absolute, and resolved by the existing `resolve_file` against the Project's source root, with its containment and Git-ownership checks. A client CWD outside the source root therefore selects the same `<root>/project.toml` as today (`tests/project.rs:717`).<br><br>**Runtime side.** The runtime refuses a relative `path` or `cwd` with typed `ProjectPathNotAbsolute`. It re-canonicalizes both in the preflight (D5) and checks the work-tree identity |
| D4 | Registry split (M6) | `ProjectRegistry::add/remove/reconcile/status/list` become three steps over the existing code:<br>1. `snapshot(&Store) -> ProjectSnapshot`.<br>2. `preflight(&ProjectSnapshot, &request, &GitRunner) -> ProjectPlan`. It holds no `Store`. It runs today's `source_root`, `default_base`, `repository_identity`, `validate_inputs` and `with_project_file`.<br>3. `commit(&Transaction, &ProjectPlan)`.<br><br>`Store::put_project` is factored into a transaction-level writer, `put_project_tx`. It keeps every existing check: the `write_snapshot` CAS, `ensure_project_idle`, the namespace and identity rules, and the `project.saved` audit. `put_project` wraps it, so D6's checks and the write share one transaction. The offline path calls the same three steps, in order, under the exclusive lock. No validation is copied, dropped or relaxed |
| D5 | Bounded preflight | **Runner.** A new `GitRunner` serves `git.rs`'s `command`, `git` and `git_text`. In the daemon, each child is spawned through `OwnedProcess::spawn`, with its own process group and `kill_on_drop`. Its pid is registered in a per-preflight `PreflightGuard`.<br><br>**Deadline and stop.** The preflight runs in `spawn_blocking` with a 10 s deadline, and it also stops when `stopping` is set. On either, the guard kills each registered group, reaps it, and only then awaits the blocking task. The response is typed `ProjectPreflightUnavailable` and never claims that the work ended.<br><br>**In-flight set.** A per-Project in-flight set holds the Project until that task has joined. While the Project is in it, a commit for that Project is refused typed `ProjectPreflightInFlight`. Other Projects are unaffected.<br><br>**Offline.** The runner keeps today's synchronous behaviour |
| D6 | Commit (M5) | The commit takes `control_admission` and then the Store lock, briefly. Inside one Immediate transaction, it checks:<br>1. the owner epoch (`owner_current`);<br>2. that the Runtime is not stopping;<br>3. that the Project is not in flight;<br>4. that the current row's version equals both the client's `expected_project` (for update and remove) and the snapshot version;<br>5. that identity, root and base still match the plan.<br><br>It then writes with `put_project_tx`. Any mismatch is typed `ProjectCurrencyChanged`, with no row or audit change.<br><br>`ProjectRemove` also checks `expected_project` against a snapshot before any preflight, so a stale remove does no Git work. `ProjectRegister` has no expected version. A duplicate root, name or identity is refused by the existing checks |
| D7 | `OwnerLock::exclusive` (R3.5) | In `execution/owner.rs`, `RuntimeOwner::open` and the new `exclusive` share one helper that opens and checks the lock file:<br>- absolute path;<br>- `create_dir_all(<state>.execution)`;<br>- symlink refused;<br>- `NOFOLLOW` and CLOEXEC;<br>- `nlink == 1`.<br><br>`exclusive` then takes `NonBlockingLockExclusive`, and `WOULDBLOCK` is typed `OwnerBusy`. It returns a guard and does nothing else: no epoch, no Runtime, no authority. The CLI takes it before `Store::open`, so every schema, migration and WAL write on open happens under it. It is dropped after the last write. A daemon that starts meanwhile fails `RuntimeOwner::open` with the existing `owner busy` (exit 75) |
| D8 | Reconcile (M4, Q2) | **`ProjectStatus`.** It resolves its one Project read-only, either by selector or by the CWD routing reader. It then reconciles **only that Project**, not every Registered Project as today's `status` does.<br><br>**`ProjectList`.** It runs one bounded `git --version`. It then starts every Registered Project's preflight concurrently under one 10 s deadline. A Project already in flight from another request is **not waited on and not refused**: it is reported as its current row, marked `reconcile: skipped_in_flight`. A Project whose own preflight times out is reported as its current row, marked `reconcile: unavailable`.<br><br>**`Blocked` writes.** Each goes through D6's commit, with the snapshot version as the CAS. A write that loses the CAS is skipped: row and audit are unchanged, and the response reports the re-read current row (state, version, reason). A row that still reads Registered is never described as verified by the stale check.<br><br>Neither action holds the Store lock or `control_admission` across Git. Holding A's preflight never delays or fails B's `ProjectStatus` |
| D9 | Out of scope | No change to Goal, Task, Driver or Native paths, or to the S2 limit. `ResolveProject` keeps its read-only route check. The S2 `--max-tasks` refusal stays on the client, before routing |
| D10 | Add lookup (M3) | `project add` first sends `ProjectLookupRoot { root }` with the client-canonicalized root. The runtime answers `ProjectLookup { found: Option<{ id, version, state }> }` from an exact-root match over **all** rows, Removed included, under a short Store lock. This is the same match `add` uses today (`project.rs:55`).<br><br>- `found: None` means the root is genuinely unregistered. The CLI sends `ProjectRegister`.<br>- `found: Some` means the CLI sends `ProjectUpdate` with that id and version. For a Removed row, this reactivates the same ID, as today.<br>- A lookup failure (for example a non-absolute root) is a typed error. It is never read as "unregistered".<br>- A version that goes stale between the lookup and the write is refused by D6 with no change, and the CLI does not retry |

Controls are §3.4 C-S3a–e, with these additions:

| Control | Assertion |
| --- | --- |
| C-S3a, projection (M1) | A real Session is written by its production producer with `native_ref` and `pid` set. Through the API and offline, the key set of every session object in `project status --json` equals the `SessionView` allowlist. The output contains no `native_ref` or `pid` value, and the plain-text output is unchanged |
| C-S3e2 (M2) | With the client CWD outside the source root, `--project-config project.toml` selects `<root>/project.toml`, both through the API and offline. A reference outside the root is still refused |
| C-S3h (M3) | `project remove` A, then `project add` A through the API: A is reactivated with the **same ID**. A second client whose looked-up version went stale gets `ProjectCurrencyChanged`, with row and audit unchanged |
| C-S3d2 (M4; Sol 6077529665 L1) | A's preflight is held by a Git child that sleeps past the deadline. B's `ProjectStatus` returns B's facts within 5 s. Mutant: status reconciles every Registered Project → FAIL |
| C-S3d3 (List does not wait) | The test first observes A in the in-flight set. While A is still held, it starts `ProjectList`. The List must complete within 5 s, with A still in flight when it returns: A is reported as `skipped_in_flight` and B as reconciled. Mutant: List waits for, or joins, an existing in-flight preflight → FAIL, on the deadline and on the finish order |
| C-S3d4 (List's own timeout) | A separate case. No preflight is in flight beforehand. List's own preflight for A runs past the deadline, so A is reported as `unavailable`, B is reconciled, and A's process group is reaped before the response. `unavailable` and `skipped_in_flight` are asserted separately |
| C-S3f (D5) | A preflight whose Git child is held past the deadline returns `ProjectPreflightUnavailable`. The child's process group is gone (reaped) before the in-flight entry clears. A commit for the same Project in that window is `ProjectPreflightInFlight`, while another Project commits. Mutant: release the in-flight entry before the reap → FAIL |
| C-S3g (D2, L1) | Three cases:<br>(i) D `Invalid`, from a corrupt descriptor.<br>(ii) The exclusive lock is held by a test probe.<br>(iii) **D `Valid` with a real API refusal.** The daemon runs with a `git` wrapper first on its `PATH` that sleeps past the deadline, so each of `project add`, `remove`, `list` and `status` receives typed `ProjectPreflightUnavailable` (list: every row `unavailable`, and the CLI exits non-zero).<br><br>In every case, each subcommand fails typed with its own reason, and the state's row, version and audit are unchanged. Two mutants are named, each with the branch it takes:<br>- (M-a) revive the old direct-`Store` fallback after an API error. The CLI's own `git` succeeds, so the command writes or answers instead of returning the API's typed reason → FAIL.<br>- (M-b) fall back through D7's offline path after an API error. The live daemon holds the lock, so the command returns `OwnerBusy` before any Git. That is not the API's typed reason → FAIL.<br><br>Both are caught because each command must return the original API reason |

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

## 8. S2 outcome (implementation)

Branch `claude/adoring-archimedes-7eehnw`, on top of PR #83's merge (`9aed3bf`). Paths are relative to `crates/rrx`. Protocol stays 2: the response changes are additive (`project_limit_stored` is optional and skipped when absent).

| Delta | Implementation |
| --- | --- |
| D1 | `src/config.rs`: `MVP_PROJECT_TASKS`, `LimitOrigin`, `ProjectTaskLimitRefused`, `ensure_mvp_project_tasks` (Ok for 0, which then keeps the positive-limit refusal, and for 1). `apply_project` checks the overlay before any field is mutated; `validate` checks the runtime value; the default is MVP |
| D2 | `src/main.rs` refuses `project add --max-tasks N≠1`, 0 included (`CliFlag`), before `Config::load` and `Store::open`; `src/project.rs` `add()` checks first. `ensure_mvp_project_tasks` exempts 0 only for `RuntimeConfig` and `ProjectOverlay`, which keep the positive-limit refusal (Sol 6071558338 M1). `Project::new` stores MVP |
| D3 | `state/runtime/driver/candidates.rs`: the fixed query joins `projects` on `json_extract(body,'$.max_tasks')=?11` (bound to MVP) before the eligible `LIMIT`. `claim.rs` `plan_initial_driver` re-reads the Project in its snapshot and refuses with `ProjectLimitUnsupported { stored }`; `task_driver.rs` maps it to `SkipReason::ProjectLimitUnsupported` |
| D4 | `runtime/control.rs`: `UnavailableReason::ProjectLimitUnsupported` and the optional `project_limit_stored` on `GoalFacts` and `GoalProposalFacts`, read by `project_limit_stored` (`state/runtime/goals.rs`, `proposals.rs`). `cli/goal_facts.rs` prints the reason, the stored value and the `--max-tasks 1` repair |
| D5 | `claim.rs`: typed `DriverCapacityUnavailable { scope: Global \| Project }`; the Project bound is MVP |
| D6 | `admission.rs`, `task_driver.rs` (parameter removed), `control.rs` (RuntimeStatus reports 1), `execution/native.rs` (`project: MVP`), `state/execution/quotas.rs` (`configured.min(MVP).min(stored)`, distinct Tasks, own-Task exemption). `runtime/phase_supervisor.rs` `PHASE_SLOTS_PER_PROJECT = 4` |
| D7 | Fixtures that ran several Tasks of one Project at once now plan each sibling in its own Project through the accepted control ingress (`runtime/tests.rs` `LegacySibling`, `legacy_fixture_with_siblings`, `legacy_store_with_siblings`). No Goal is written outside that ingress. CA2 uses four Projects. The native cap test keeps its same-Project sibling for the `project` case only |
| D8 | `project::effective_config` refuses a stored value ≠ 1 with `ProjectLimitUnsupported`; README example `max_tasks_per_project = 1` |

### Controls

| Control | Test | Result |
| --- | --- | --- |
| C-S2a | `runtime/installation/tests/project_limit.rs` `c_s2a_candidate_never_lists_second_task_while_first_is_occupied` | pass |
| C-S2a, retained Driver alone | `c_s2a_retained_driver_alone_keeps_second_task_out`: A's Source preparation fails (the root is not a Git repository), its Unit is retired and its Driver retained; occupancy `(driver, unit, operation) = (1, 0, 0)` | pass. B never listed or claimed |
| C-S2a, Unit alone | `runtime/tests.rs` `c_s2a_legacy_unit_alone_keeps_same_project_accepted_task_out`: a migrated legacy Task's Unit is reserved by the production `AttemptManager::prepare`, with no Driver and no operation; occupancy `(0, 1, 0)`. On the same owner and epoch an accepted Task B of the same Project and C of another Project are created through control ingress | pass. B is not a candidate; C is |
| C-S2a, held operation alone | `c_s2a_held_phase_operation_alone_keeps_second_task_out`: A parked before transport; A's Goal paused through the lifecycle writer (Unit retired, Driver invalidated); the non-success consumer keeps the plan Held; occupancy `(0, 0, 1)` | pass. B (same Project, another Goal) never listed across reconciliations; a Task of another Project is admitted beside it |
| C-S2a2 | `c_s2a2_final_claim_refuses_second_task_typed`: B's key read through the production reader, A claims, B evaluated under the normal admission guard | pass. `DriverCapacityUnavailable { Project }`, no Driver row or claim audit for B |
| C-S2a3 | `c_s2a3_stored_legacy_limits_are_excluded_and_never_rewritten`: 70 legacy-limit Tasks in a Project that sorts before the supported one (more than the eligible `LIMIT 65`). Two registered Projects are assigned by observed ID order, so the setup never depends on a random search (Sol 6072320376 L2) | pass. The supported Task is the only key on the first page and is claimed; stored rows unchanged |
| D3 guard | `d3_plan_guard_refuses_stored_legacy_limit_without_writes` | pass |
| D4 | `d4_status_reports_stored_legacy_limit_read_only_until_repair` | pass. Accepted Goal and proposal both report. Around every read, the `projects`, `goals`, `tasks`, `records`, `scheduler_*`, `task_drivers`, `execution_units` and `audit` rows are unchanged. The answer is the same after the production `reconcile_runtime_attention` runs to its end, before and after start; gone after repair |
| C-S2b | `success.rs` `sc10_four_projects_close_independently`; `activation/composition.rs` `ca2_four_tasks_two_projects_retain_own_scope_and_roster` (four Projects) | pass |
| C-S2c | `state/execution/tests.rs` `c_s2c_active_task_reviewers_run_concurrently_under_project_limit_of_one`: two Reviewer leases for the active Task, another Project admitted beside them, a second same-Project Task waits on `Capacity` | pass |
| C-S2d | `config.rs` `c_s2d_runtime_limit_other_than_one_is_refused_typed`, `c_s2d_overlay_limit_other_than_one_is_refused_before_mutation`; `tests/cli.rs` `c_s2d_limits_other_than_one_are_refused_without_state` (`--max-tasks` 0 and 2: typed `CliFlag`, no state directory or database); `tests/project.rs` `c_s2d_registration_refuses_other_limits_and_only_explicit_one_repairs` (0 and 2 typed `CliFlag`, on an empty Store and on an existing row whose row, version and audit stay unchanged) | pass |
| C-S2e | `tests/project.rs` `c_s2e_effective_config_refuses_stored_legacy_limit_until_explicit_repair` (legacy row through the production `put_project`) | pass |

### Mutants (each restored; tree clean)

| Mutant | Detected by |
| --- | --- |
| Claim Project bound 2 | C-S2a2 FAIL |
| Candidate bound 2 | C-S2a FAIL |
| Unchecked copy restored in `effective_config` | C-S2e FAIL |
| Quota own-Task exemption dropped | C-S2c FAIL |
| Native configured global cap ignored | `configured_global_provider_alias_and_project_caps_wait_before_native_spawn` (global) FAIL — the sibling is in another Project, so the Project limit cannot hide it |
| CLI 0 exempt again (M1) | `tests/cli.rs` C-S2d and `tests/project.rs` C-S2d FAIL |
| Candidate Driver branch dropped | `c_s2a_retained_driver_alone_keeps_second_task_out` FAIL |
| Candidate phase-operation branch dropped | `c_s2a_held_phase_operation_alone_keeps_second_task_out` FAIL |
| Legacy exclusion moved after the eligible `LIMIT` | C-S2a3 FAIL |
| Status read stores an attention row | D4 FAIL ("status read changed state") |
| Candidate Unit branch dropped | `c_s2a_legacy_unit_alone_keeps_same_project_accepted_task_out` FAIL at the candidate assertion (Sol 6072320376 L1: the legacy `AttemptManager` path leaves a Unit open on its own) |

### Source review

| Round | Result |
| --- | --- |
| 6071357590 → Sol 6071558338 | REQUEST CHANGES, required 0/0/1/3. M1: a CLI `--max-tasks 0` passed the early check and was refused only after `Store::open`. L1–L3: missing single-occupancy, pre-`LIMIT` and read-only controls |
| `cafc547` → Sol 6072320376 | M1, L1 (Driver and operation alone), L2 (original), L3 closed. REQUEST CHANGES 0/0/0/2: L1 Unit alone through the legacy path; L2 setup relied on a bounded random search |
| `90e6a0d`, `d6ca604` → Sol 6072686320 | **APPROVE LIMITED, required 0/0/0/0.** L1 (Unit alone) and L2 (deterministic order) closed; M1, L1 (Driver, operation), L3 stay closed. The earlier adapter 18/19 stays an open observation |

### Verification

Non-root (fmtest, umask 022, subreaper), all 19 test binaries: rrx lib 738 passed, 0 failed (20 ignored, unchanged from before S2); every integration binary passed. `cargo clippy --workspace --all-targets` and `cargo fmt --check` are clean. An earlier run was discarded: the test binary was rebuilt while it ran, and `execution/resources.rs` re-executes `current_exe()`, so the installation tests failed on the replaced executable. The clean re-run above had no build alongside it.

Review-fix run (`cafc547`, non-root, no build alongside): rrx lib 740 passed, 0 failed (20 ignored). Every integration binary passed except `adapter` 18/19: `runtime_shutdown_terminates_native_group_and_preserves_uncertain_reservation` failed once with "fixture descendant still running: R". The S2 diff touches no adapter or process code, and the failure did not reproduce in 40 single runs or 10 whole-binary runs. The failing path is the `GenericCliAdapter`'s `ProcessGroup::drop` (`adapter.rs:604`, corrected by Sol 6072320376), which signals the group without waiting, and `assert_process_dead` (`tests/adapter.rs:564`) samples `ps` once, so a SIGKILLed descendant can still read `R` while it exits. This is recorded as an open observation, not as a flake. The proposed fix, a bounded wait for the process to be gone or a zombie before asserting, is outside S2.
