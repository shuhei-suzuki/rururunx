# Issue 81: rrxd local runtime daemon — requirements (WHAT), approved

| Stage | Comment | Result |
| --- | --- | --- |
| R0 | 6056376746 | Sol 6057713206: M1–M3 mandatory, 2 optional |
| R1 | 6057882635 | Sol 6058083336: **APPROVE LIMITED, 0 findings** |

Requirements baseline: `51b79b9`. The HOW starts after #43 merged (Q5): PR #80 merged as `b2eb694`. Code paths are relative to `crates/rrx/src`. Normative words are MUST and MUST NOT.

## 0. Facts at the baseline

| Area | Today | Gap |
| --- | --- | --- |
| Service | `rrx serve` runs in the foreground: owner open → Runtime → endpoint bind → start (`cli/service.rs:29-34`) | It cannot be detached, and there is no `rrx daemon start/status/stop` |
| Exclusivity | A non-blocking `flock` on `owner.lock`. Each open is a new epoch (`execution/owner.rs:169-251`) | None; it is reused |
| Endpoint | A private Unix socket, a descriptor with identity, peer UID checked on both sides, an exact `PROTOCOL_VERSION=1` handshake | The supported range is a single version |
| Control | Resolve, Goal create/propose/status/tasks/lifecycle, Runtime status/stop. Task retry/cancel → `Unavailable` | Lists, attention queue, Review/Approval, sessions, routing, events, metrics |
| Concurrency | `max_tasks_per_project` defaults to 4 | MVP needs a fixed 1 |
| Direct Store use | `rrx project *` opens the Store without the owner, and reconcile writes (`project.rs:150`) | Must become runtime-authoritative |

## 1. Requirements

### R1. Lifecycle

- **R1.1** `rrx daemon start` starts one service per canonical state root, detached. It is the same implementation as `rrx serve`. After the invoker, its terminal and its process group exit:
  - the same epoch keeps running;
  - active Tasks continue;
  - native child ownership stays with the daemon.

  The command returns only after the endpoint is published and validated, or with a typed failure.
- **R1.2** `rrx daemon status` reports exactly one of:
  - `running`, with the validated identity `{state, instance, epoch, protocol}` and `service_running`. `operational` (Native readiness) is reported separately from service readiness.
  - `owner busy, identity unconfirmed`: the lock is held but no endpoint validates.
  - `discovery unavailable`: the descriptor is stale, malformed or mismatched; the connection was refused or timed out; or the identity does not validate.
  - `not running`: only when the owner lock is free. The probe neither takes the lock into service nor begins an epoch.

  An unresponsive endpoint is never evidence that the daemon has stopped. `status` is read-only and never auto-starts.
- **R1.3** `rrx daemon stop` is a `RuntimeStop` request through the endpoint, never a PID signal. The result distinguishes:
  - completed;
  - pending, with owned work held;
  - failed;
  - unconfirmed.

  Only "completed" counts as stopped. None of them counts as cleanup success, and process exit is not evidence of cleanup.
- **R1.4** A second `start` fails typed and writes nothing: no epoch, no descriptor, no state. The refusal is one of:
  - `already running`, with the validated identity;
  - `owner busy, identity unconfirmed`, including a start that races before the first endpoint is published.

  The flock is the sole arbiter. PID files are not authority.
- **R1.5** `rrx serve` (foreground) stays, for debugging and supervisors.
- **R1.6** For every client (CLI, Desktop and `status`), discovery failure is typed and distinct from "stopped".
- **R1.7** The detached service keeps the standard providers' (Claude, Codex) native auth, settings, rules, hooks and permissions, with success and refusal conditions equal to `serve`.

### R2. Control API

- **R2.1** A private Unix socket with peer-UID checks and the identity echo. No TCP listener in MVP.
- **R2.2** The `Hello` advertises the protocol and its supported range. MVP MAY use a single version.
  - These fail closed with a typed error and no state change: out of range, a stale `instance`/`epoch`, unknown fields, an oversize frame.
- **R2.3** The read surface includes:
  - Runtime status;
  - Projects → Goals → Tasks;
  - the human-attention queue (R5);
  - Review/Approval per Task;
  - Agent/reviewer session status;
  - non-secret attach/session routing metadata, or typed unavailable;
  - logs/event summaries, paged per scope;
  - token, cost and cache metrics when recorded, otherwise explicitly unavailable.

  No read returns native refs, credentials, environment values or authority.
- **R2.4** The write surface keeps every existing operation: Project resolve; Goal create, propose, Pause, Resume, Cancel and Fail; Runtime status and stop. It adds:
  - Task cancel and retry, typed `Unavailable` until #14/#75;
  - live Project add and remove (R3.4).
- **R2.5** CLI and Desktop use the same API. No client-only state is authoritative.
- **R2.6** Every request carries `request_id` and the service identity `instance`/`epoch`. Object-scoped writes also carry the target's optimistic version. Service-scoped writes (`RuntimeStop`) are bound by the service identity only. Each new write declares which of the two it is.
- **R2.7** Reads are pull-only and paged, bounded in count, bytes and work. Each response distinguishes continue, end and stale cursor. Subscription is out of MVP scope.

### R3. Ownership

- **R3.1** A client disconnect, crash or exit never cancels, pauses or cleans up anything, and is never cleanup proof.
- **R3.2** Reconnect never mints Task, Session or Workflow authority.
- **R3.3** The daemon owns the Agent child processes. Clients never spawn or own native provider processes.
- **R3.4** While a daemon owns the state root, every state-changing Project operation goes through the control API: `project add` (including an update), `project remove`, and the reconcile reached from `project list/status/resolve`. The runtime applies the existing identity, namespace, validation and idle-removal rules. A client never supplies Store writes, owner, epoch or native authority.
- **R3.5** An offline write MUST NOT race a daemon's ownership for its whole duration. That covers Project writes, reconcile, and Store initialization or migration on open. Offline writes never mint Task or Session authority.

### R4. Concurrency

- **R4.1** At most one active Task execution authority per Project, enforced by the runtime.
  - The limit is a fixed 1. A value above 1 from any source is refused with a typed error: runtime config, Project overlay, `project add --max-tasks`, or a stored `Project.max_tasks`. Stored values are never silently rewritten, and status reads keep working.
  - The authoritative count is the distinct-Task union of driving Driver, open Unit and open managed operation, held or unbound included.
  - A Task limit is not a session limit.
- **R4.2** Multiple Projects run concurrently, subject to `global_max_sessions`. Cross-Project interference never ends another Project's session (the SC10 fix, `9dc2648`).
- **R4.3** Reviewer and Approval sessions of the active Task MAY run concurrently.
- **R4.4** Same-Project multi-Task execution and same-Project namespace retry are out of MVP scope.
- **R4.5** Every consumer applies the same fixed 1: admission, initial-driver claim, PhaseSupervisor, status projection, native limits and quota, and Project registration and effective config.

### R5. Human attention and Review/Approval

- **R5.1** One queue, derived from durable state: Goal attention, workflow waits, Review/Approval pending, and held Unknown non-success. Each item names its scope and its allowed typed actions.
- **R5.2** Reading the queue has no side effects.

### R6. Safety

- **R6.1** The endpoint is never public.
- **R6.2** The identity is verified before every control action.
- **R6.3** No credentials, environment values, tokens or native refs are returned in status.
- **R6.4** Clients cannot bypass native auth, rules, hooks, trust or permissions.
- **R6.5** A restart is a new epoch. Fencing turns undetermined work into Unknown and keeps known Success/Failure. It never creates a new Task success or cleanup success. A Goal with history still resumes as `FreshBootstrapRecoveryUnavailable` until #14/#75.

## 2. Acceptance → controls

| AC | Control |
| --- | --- |
| Runs independently | End the invoker, its terminal and its process group. Status stays `running` with the same epoch, and an in-flight Task continues with native children owned by the daemon |
| CLI status and control | A socket integration test for every R2.3/R2.4 request |
| Desktop uses the same API | A contract doc and a second client fixture: same identity, read/control and disconnect conditions. This does not count as the Desktop app being complete |
| Close the client, the Task keeps running | Kill the client while a Task is in flight. The Task closes normally |
| One daemon per root | N concurrent `start`, including a race before the endpoint exists. Exactly 1 runs; the other N-1 get a typed refusal and leave nothing changed |
| Status is not liveness | Stale, mismatched, refused or timed out → `discovery unavailable`. A held lock with no endpoint → `owner busy`. `status` changes nothing |
| Reconnect vs restart | Reconnect reads the same state. A restart is verified separately (R6.5) |
| Versioned, rejects stale requests | Mutants: protocol off by one, stale epoch, unknown field. Each gets a typed refusal and the preimage is unchanged |
| Attention and Review state | A queue entry for each R5.1 source |
| No public listener | No AF_INET or AF_INET6 socket is bound |
| Stop ≠ cleanup | The four outcomes are distinct. After a restart, undetermined → Unknown, known outcomes are kept, and no new success appears |
| Offline writes | A daemon starting during an offline write or Store open does not interleave with it. Reconcile goes through the daemon while it runs |
| API scope | Every kept operation, routing (or unavailable) and summaries. No secret is returned |
| One active Task per Project | Fixed 1; a value above 1 is refused, typed, with no rewrite. Cases: the second Task waits, including during a held or unbound occupancy; reviewers of the same Task run in parallel; two Projects run in parallel. Mutant: a limit of 2 fails |

## 3. Decisions

| # | Decision |
| --- | --- |
| Q1 | Fixed 1, plus a typed refusal |
| Q2 | Live add/remove through the API, plus the offline exclusion |
| Q3 | Detach the same service; no separate binary in MVP |
| Q4 | Pull-only, paged and bounded |
| Q5 | The HOW starts after #43 merged |
