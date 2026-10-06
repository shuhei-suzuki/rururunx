# Issue 43: Native prepared-input completion (G1) and pre-Session quota contract

## 1. Status, fixed source and categories

1. This is a proposed authorization/contract HOW supplement. It refines, and does not replace:
   - the approved [preparation HOW](issue-43-native-preparation-integration-design.md) §§2, 3, 5 and 6;
   - the approved [transport HOW](issue-43-native-transport-integration-design.md) at `9dafdb0256e49568e02c5f910dfb91b61082e1f0`, specifically §§5.1, 5.3, 6, 13 and gate G1 in §16;
   - the approved [readonly HOW](issue-43-native-readonly-helper-design.md) at `d92d63824cd480dd39b876bd69a3f03019e96f86`, specifically §§8.6 and 15;
   - the [version HOW](issue-43-native-version-helper-design.md);
   - [managed binding design](issue-43-managed-binding-design.md) §§2.1, 3.1 and 5.2;
   - [agent-execution design](agent-execution-design.md) §§5, 6 and 8.

   It is not a requirements change. It does not update the master design, and it does not claim that anything is implemented.
2. **Exact source.** `a913203f0a0e7cdb8b4c3534a24082c0f16a59c8`, clean. All line references are to this commit.
   - This commit contains the new first-Executor readonly Git batch. That batch's source tests and independent source review are still pending.
   - The batch is a component. It does not qualify Native execution.
   - The G1 delta was re-read at `0ca61b37a4bdc52e59143d71867a8284088bc401`, whose only change from `a913203f` is this document, so every source line reference is unchanged.
3. Categories are kept strictly separate:
   - **A (Actual):** present in source at the pinned commit.
   - **P (Proposed):** this contract. Not implemented.
   - **V (Verified):** only what reading source at the pinned commit shows.

   No build, test, lint, Agent or OS run was executed for this HOW. Full regression and Clippy remain RED open gates. The following all remain unqualified:
   - rrx Native one-Task and four-Task runs;
   - macOS and Linux;
   - authentication and hooks;
   - quota exhaustion and recovery.
4. **Out of scope (unchanged):**
   - credentials, HOME and provider roots, root, VM/container, a separate user;
   - license; ps46/60; CLI bugs unrelated to this lane; inherited Stop findings; GitHub publishing; Tier custody; full MVP tracking.
   - The following are never authority: public actor constructors, callback capabilities, boolean/optional protected bypasses, FakeAgent admission, row/DTO/receipt/digest-created ownership, any Task.version-changing Session binding.
   - rrx is not a security sandbox, makes no process-death claim, and Native Agents and hooks keep the user's host authority.

### 1.1 One-line acceptance condition (P)

The actual selected Native start, consuming only the SAME retained original Source/Driver allocation, marker, launch and actor, issues exactly one `PreparedNativePhase`, and only after all of the following, all in the SAME preparation custody:
- the SAME qualified version observation with known closed settlement;
- the SAME completed 13-action Git history and physical correspondence (Executor), or the SAME Reviewer seal batch;
- the SAME captured compatibility qualification of the selected installed adapter's nonsecret declaration, for the SAME allocation role, against that version observation and the SAME original Frame;
- the SAME retained transport command, built once from the SAME actor/allocation and closed preparation facts and checked by that qualification before any quota write;
- a SAME-custody `PreparedPhaseNoCurrentDispatch`;
- a known-committed admission in the bounded private quota transaction.

That quota transaction counts every active lease, marked or legacy. Its fairness does not treat a refusal as absence. Parking is the only alternative to admission. It writes only the exact Unit, waiter, lease, pool (including a cold pool's complete default image) and readiness images, under the SAME stop admission, each against the SAME known readiness lineage. It is resumed only by the SAME in-task exact due-claim of the original operation, pair, input and helpers, with Task, Workflow, Context, Driver and Source rows and versions unchanged.

Prepared is consumed once by the approved transport registration, which never uses allocated-v1 or generic marked validators. Composition stays refused until G1–G5 are real. Every other path is no-effect refusal, Held, or nongrant closure.

### 1.2 Correction delta (P)

This revision corrects only two verified defects, B1 and B2. The closed G1 findings R1–R5 and R4a stay closed and are not reopened. The source facts for both corrections were read at `fd4a6a160fc1d20f1ab3f41f2ac15b76246c5d83` (clean). Every file cited for B1 and B2 is identical at `a913203f` and `fd4a6a16`, except `state/execution/native_phase/version.rs`, whose only change (`:441–449`) lies outside the cited planner lines `:280–311`. So every line reference in this document keeps its meaning. Nothing below is implemented, built or tested.

- **B1 effect arithmetic.** Replaced assumption: a ≤252 pre-registration baseline plus transport 1, setup ≤2 and input 1 (transport HOW §13; here §§8, 9.1, 11.1 and C-1). The actual pre-input setup is provider-specific:
  - Codex journals six `native_setup` effects before its `turn/start` `native_input` (`execution/native.rs:2035`): `initialize`, `initialized`, `account/read`, `environment/status`, `account/rateLimits/read` and `thread/start` (`:1963,1968,1973,1980,1987,2020`). `boot_call` maps an absent kind to `native_setup` for a protected phase (`:1487–1507`).
  - Claude journals one, `initialize` (`:2123`), before its input (`:2149`).

  The unchanged `plan_phase_dispatch` gate admits a dispatch only while the complete inventory is `<256` (`state/execution/native_phase.rs:1293–1298`). At the allowed 252 baseline, Codex is therefore refused at its fourth setup, before input. The correction is a provider+role admission budget, checked before every helper effect (§11.4). Changed: §§8 item 2, 9.1, 11.1, 11.2, 11.4 (new), 12, 13, 14 (C-1, C-9) and 15.
- **B2 Legacy candidate validation under the private lock.** Replaced assumption: §6.4 let the existing generic validator decide a Legacy candidate "inside the Immediate as today", while §11.3 forbade hashing and encoding under SharedStore. Actual `validate_authority(…, native=true, …)` (`state/execution.rs:458–497`) runs all of the following inside that transaction:
  - `governing_digest(Project, Goal)`: JSON construction, serialization, SHA-256 and hex formatting (`:498–511`);
  - `runtime::driver::validate` (`state/runtime/driver.rs:318–353`): Driver body decode, a pins snapshot with digests, a serialized comparison and the `rrx_live_task_driver` liveness callback;
  - `source_recovery::validate_task` (`state/execution/source_recovery.rs:500–509`): a pins snapshot that includes `governing_digest`, and a serialized comparison.

  The correction is one precisely limited lock-contract exception, E-1 (§6.4 item 5). A two-branch plan keeps policy, image building and private encoding out of the lock. Changed: §§6.2, 6.3, 6.4, 11.2, 11.3, 12, 13, 14 (C-10) and 15.

## 2. Revalidated source map at a913203f (V)

Paths are relative to `crates/rrx/src/`.

| Item | Actual fact at the pinned commit |
|---|---|
| Selected start | `adapter/native.rs:100–126` → `execution/native.rs:200–217` → `native/preparation.rs:344–431`. The steps are: `claim_start` one-shot (`:46–57`); same-Unit `try_lock_owned` gate (`:366–389`); actor; ≤128 preparation index (`:394–406`); readiness allocated1→preparing2 under admission (`:410–423`); `prepare_phase_version` (`:424`); `prepare_phase_git` (`:425`). The start then **refuses** at `:426–430`: "original Native readonly Git observations retained; full preparation and transport unavailable". The later bail at `native.rs:214–216` is unreachable |
| Absent types | `PreparedNativePhase`, `PreparedPhaseNoCurrentDispatch`, `NativeQuotaPlan`, `NativeTransportStartPlan` and `NativePreCoreCustody` do not exist (zero grep hits). `NativePhaseStart::Launched`/`::Waiting` are never constructed; they are only matched in `runtime/phase_jobs.rs:259,263` |
| Version | `NativeVersionObservation::qualified_profile` (`native/version.rs:138–153`) passes **untrimmed** stdout. Codex `verify_native_version` requires exact equality with `"codex-cli 0.160.0"` (`codex/protocol.rs:24–34`), so output ending in a newline never qualifies. Claude uses its first whitespace token `2.1.283` (`execution/claude_wire.rs:26–37`). Legacy paths trim (`results.rs:702–704`). Closed settlement: `NativeHelperSettlementCommit` (`state/execution/native_phase/version.rs:323–327`), issued only by `close_phase_version_observation` (`version/closure.rs:385–415`) |
| Git batch | `prepare_phase_git` (`native/version.rs:460–538`) returns `NativeReadonlyHelperCompletion` (`:15–23`, documented as nongrant) after `confirm_phase_helper_history`. That function requires 14 linked entries, current inventory == last `after`, all confirmed (`state/.../version.rs:393–475`), and `reserve_git_batch` ≤242 rows (`:280–298`). Seal/actions: `native/readonly.rs:31–61,194–302,465–552` |
| Preparation lineage | `NativePreparationPlan` freezes `current: CurrentWorkflowSuccessor`, `owner_before`, `readiness_before` and `readiness_after` (`state/execution/native_phase/preparation.rs:9–16`); `readiness_after` is fixed to `preparing`/version 2 (`:108–117`). `NativePreparationCommit::validate_version_ready` (`:24–27`) always runs `validate_common` against that original successor and then that v2 image. `validate_common` (`:39–49`) calls `validate_preparation_origin_tx` (`runtime/phase_supervisor.rs:186–194`) → `validate_current_tx` (`managed_binding/successor.rs:261–281`), which requires the complete Unit row (version, body and index columns, `marker_plan.rs:141–151`) to equal the successor's Unit. Helper consumers: `state/.../version.rs:371,444,490`. Any later Unit or readiness version change therefore makes these validators refuse permanently |
| Source handoff | First Executor only: `workflow_source/native_handoff.rs:448–519` requires generation 1, Unit generation 1, no artifact and Preparing. `qualify_git` refuses an artifact frame with "actual readonly artifact lease unavailable" (`:102–110`). `driven_initial.rs:56–59` refuses continuation after initial Evidence |
| Reviewer | `ResultSnapshot` (`results.rs:66–80,521–626`) is created by `prepare_snapshot_inner` (`attempts.rs:604–711`). Its checkout ends with `readonly_tree(&unit.worktree, true)` (`results.rs:602–626`), so the Reviewer's source tree is readonly. It is retained only in the Engine's in-memory `managed_snapshots` map (`workflow.rs:560–562,1749–1752`). `GitLease` is process-local (`execution/owner.rs:33–37,292–326`) |
| Registration | `plan_phase_registration`/`register_phase_session`/`validate_phase_preparation` require allocated v1 (`state/execution/native_phase.rs:982–1000,1128,1409`) and call generic `validate_authority` (`:1407,1462`). `NativeOwnerPlan::validate_tx`/`validate_terminal_tx` use it at `:360,373`. Only caller is `start_with_launch(…, None)` (`native.rs:195`) |
| Readiness DDL | `managed_binding/schema.sql:110–120`: `state ∈ {allocated, preparing, parked, registered, deferred, held, closed}`; nullable positive `parking_version`; body ≤4096. Permit-gated triggers (`:173–175`); core trigger `version=OLD+1`, monotonic flags (`:178`); `binding_admission_not_parked` and the parked/consumed exclusion (`:179–181`). Nothing writes `parked` or a non-NULL `parking_version` today |
| Quota DDL | `execution.sql:117–142`. `quota_pools` (PK provider+account_key; `next_probe_at` default 0, `probe_unit` nullable, `backoff` default 60000 ∈[60000,1800000], `last_role` default `'reviewer'`), `quota_windows` (PK +bucket; `observed_at`, json body), `quota_leases` (PK unit_id; provider, account_key, role, epoch>0, active∈{0,1}), `quota_waiters` (PK unit_id; provider, account_key, reason, next_due, fairness_sequence, resume_state). Windows, leases and waiters each have a FOREIGN KEY to `quota_pools(provider,account_key)` (`:124–142`), so none can exist before the pool row. **No version column on any quota table**; no triggers other than the writer-contract guards |
| Quota code | `state/execution/quotas.rs:149–370` uses generic `validate_authority` (`:172`) and creates the pool only after it (`:178–181`, `INSERT … ON CONFLICT DO NOTHING`); `observe_quota_inner` creates it the same way (`:60`). Its COUNTs are unfiltered (`:207–213,534`), so they include every active lease. History read is unbounded (`:239–268`). Due waiters are materialized unbounded (`:278`). The fairness loop **skips** a candidate whose generic validation fails (`:286–300`). An exhausted pool makes every Unit wait while `at < next_probe_at` or another Unit is `probe_unit` (`:302–305`). `release_quota_tx` (`:515–521`) deactivates the lease and, only where `probe_unit` is that Unit, sets `probe_unit=NULL, next_probe_at=MAX(next_probe_at, now+backoff)`. `fairness_sequence` is an epoch-ms timestamp. `resume_state` is the snake_case `UnitState`. Account key is always `"unknown"` (`execution/quota.rs:60–68,130,220`). Caps: `QuotaScheduler::new` 6/2/3/4 (`quota.rs:15–23`); `NativeLimits::configured` (`native.rs:114–153`) only lowers them; Store `ensure!` ≤1024 |
| Unit | `execution_units` (`execution.sql:6–23`): version CAS via `write_unit` (`state/execution.rs:314–321`). Body `ExecutionUnit` (`execution/model.rs:~199–221`) has `state`, `wait_reason`, `capacity_retry_at`. `UnitState` includes `Preparing` and `WaitingQuota` (`model.rs:91–101`) |
| Permit catalogue | `managed_binding/permits.rs:23–180` covers workflow_native_contracts, managed_phase_operations, managed_marker_bodies, managed_phase_owners, managed_phase_inputs, managed_phase_admissions, managed_phase_readiness, task_drivers, source_recoveries, records, audit. `ExactRowMutation` (`:185–232`) and `with_exact_permit` (`:243–350`) handle 1..=128 rows. `execution_units`, `managed_effects`, `quota_*`, `native_invocations` and `session_units` are **not** permit tables; they carry only writer-contract10 guards (`state/execution.rs:124–138`) |
| Epoch | `begin_execution_epoch` deactivates all leases, nulls `probe_unit` and deletes all waiters (`state/execution.rs:589–591`). `fence_task_tx` (`:340–345`) and terminal (`native_phase/terminal.rs:736–737`) release leases through `release_quota_tx` and delete waiters |
| Config | `config.rs:52–61` `AgentConfig {provider, command, model, effort, max_concurrent}`. There is no compatibility, version or hook representation. Project overlay may set only agent `model`/`effort` (`:133–138`). Frame `config` is the runtime config merged with the committed project text (`workflow_source.rs:1051–1079`). Only project bytes are digested (`rules:config`). Committed provider/command must equal the Runtime registry |
| Launch environment | Managed launch and version helper inherit the full environment (no `env_clear`). The overlay is `ResourceProfile::environment` (`resources.rs:36–104`), which adds three `GIT_CONFIG_*` entries (gc/maintenance/fsmonitor) and no hooks override. Claude argv carries `--settings {"forceLoginMethod":"claudeai"}`. Initialize sends `"hooks":null` (`claude_wire.rs:38–45`), which registers no SDK hooks |
| Composition | `Runtime::installed_driver_composition` always bails (`state/managed_binding/composition.rs:70–75`) and has no callers. `require_managed_native_binding_composed` always errors (`workflow.rs:529–534`). `PreparedInputAdmission` is never advertised (`adapter/native.rs:263–271`). `AgentRegistry::from_managed_config` has only a test caller (`execution/phase.rs:307`) |
| Root jobs | `runtime/phase_jobs.rs:19–60,233–304`: ≤128 jobs; custody preallocated in `reserve` (`:140`); `InvocationObservation::Waiting` is derived only from a returned `Waiting`. No `rrx.private.workflow.phase_closed` producer exists (the kind is only declared, `managed_binding/schema.rs:37,397,408`) |

## 3. Stage order and ownership graph (P)

### 3.1 Inline stage order of the SAME selected start

The bail at `native/preparation.rs:426–430` is replaced by S4–S8. There is no re-entry, callback, flag or second `claim_start`.

| Stage | Producer | Effect | Grants |
|---|---|---|---|
| S0 | (A) actor + readiness allocated1→preparing2 | readiness | nothing |
| S1 | (P) `qualify_compat_static`, outside all locks, role-specific (§4.2) | none | nothing; refuses before S2 when undeclared or role-incompatible |
| S2 | (A) version helper | `native_phase_version` | closed qualified observation (nongrant) |
| S3 | (A, Executor) 13-action Git batch / (P, Reviewer) §9 batch | `native_phase_git` | `NativeReadonlyHelperCompletion` (nongrant) |
| S4 | (P) `qualify_compat_observed` | none | `NativeCompatQualification` (nongrant) |
| S4b | (P) `plan_native_command`, outside all locks, then `compat.check_command` | none | retained `Arc<NativeTransportCommand>` (nongrant data) |
| S5 | (P) `issue_no_current_dispatch` | none (one read-only Immediate, Initial lineage) | `PreparedPhaseNoCurrentDispatch` |
| S6 | (P) quota plan → Admit, or Park ⇄ due-claim loop | quota/Unit/readiness; pool default INSERT when cold | `NativeQuotaAdmitted` (known commit) with its known lineage |
| S7 | (P) `issue_prepared` | none | `PreparedNativePhase` (one per operation), retaining the SAME S4b command |
| S8 | (approved transport HOW §7, as corrected by C-7 and C-8) | registration + transport | per that HOW, requires G2–G4 |

S1 runs before any further process effect, so an undeclared alias or a role-incompatible hook declaration wastes no helper. S4 needs the SAME version observation. S4b runs once, before any quota write, so a doomed command never holds a lease; nothing rebuilds it later.

### 3.2 Types (all crate-private, non-Clone, non-Serialize/Deserialize, private fields, no row/ID/DTO constructor)

```rust
// execution/native/compat.rs (new)
struct NativeCompatDeclaration { /* parsed, canonical, bounded; digest */ }
struct NativeCompatQualification {
    declaration: Arc<NativeCompatDeclaration>,       // SAME Arc as installed adapter
    version: Arc<NativeVersionHelperCustody>,        // SAME qualified closed observation
    role: SessionRole,                               // SAME allocation f.role (§4.2)
    project_hooks: Box<[QualifiedHookRef]>,          // borrowed facts from SAME Frame inventory
    frame: Arc<Frame>,                               // SAME original Frame (via seal)
}
// execution/native/prepared.rs (new)
fn plan_native_command(owner: &Arc<RuntimeOwner>, actor: &Arc<NativePreparationActor>,
    completion: &Arc<NativeReadonlyHelperCompletion>, compat: &Arc<NativeCompatQualification>)
    -> Result<Arc<NativeTransportCommand>>;          // sole protected command producer (C-7)
struct PreparedPhaseNoCurrentDispatch { custody: Weak<NativePreparationCustody>,
    completion: Arc<NativeReadonlyHelperCompletion>,
    issued: Arc<NativeReadyLineage> }                 // the Initial lineage it was issued under
struct NativeQuotaAdmitted { plan: Arc<NativeQuotaPlan>,
    lineage: Arc<NativeReadyLineage> }                // only from Store known commit / exact confirm
struct PreparedNativePhase { actor: Arc<NativePreparationActor>,
    known: Arc<NativePreparationCommit>, version: Arc<NativeVersionHelperCustody>,
    completion: Arc<NativeReadonlyHelperCompletion>, compat: Arc<NativeCompatQualification>,
    command: Arc<NativeTransportCommand>,            // SAME S4b command
    quota: Arc<NativeQuotaAdmitted> }                // its lineage: readiness (preparing,P,NULL), Unit Preparing U
// state/execution/native_phase/preparation.rs (extended)
enum NativeReadyLineage {
    Initial(Arc<NativePreparationCommit>),           // (preparing,2,NULL) + original Unit; A validator
    Quota { commit: Arc<NativePreparationCommit>,    // SAME original known commit
            current: CurrentWorkflowSuccessor,       // SAME original successor; Unit = known postimage
            readiness: PairRow, unit: Body<ExecutionUnit> },
}
// state/execution/native_phase/quota.rs (new) + state/execution/quota_policy.rs (factored)
struct NativeQuotaPlan { /* SAME actor, SAME no-dispatch, SAME pre-lineage, snapshot, decision, exact images */ }
enum NativeQuotaOutcome { Admitted(Arc<NativeQuotaAdmitted>), Parked(Arc<NativeParkedPhase>) }
struct NativeParkedPhase { plan: Arc<NativeQuotaPlan>, lineage: Arc<NativeReadyLineage>,
    parking_version: u64, due: i64, reason: WaitReason, waiter: WaiterImage, pool: PoolImage }
```

### 3.3 Ownership graph

```text
Root PhaseJobs Entry -> Job -> JobState.preparation -> NativePreparationCustody (A)
  slots (each one-time, no history lists):
    actor, plan, known, helpers[<=32] (A); completion (A)
    compat: Option<Arc<NativeCompatQualification>>            (P)
    command: Option<Arc<NativeTransportCommand>>               (P, set once at S4b)
    no_dispatch: Option<Arc<PreparedPhaseNoCurrentDispatch>>   (P; cleared by S8 install)
    quota: QuotaSlot{ current: Option<Arc<NativeQuotaPlan>>,
                      lineage: Option<Arc<NativeReadyLineage>>,  // latest known lineage
                      parked: Option<Arc<NativeParkedPhase>>,
                      admitted: Option<Arc<NativeQuotaAdmitted>> } (P)
    prepared: Option<Arc<PreparedNativePhase>>                 (P, set once)
    transport: Option<Arc<NativeTransportCustody>>             (transport HOW)
    parked_level: watch::Sender<ParkedLevel>                   (P, nongrant observation)
Prepared -> actor, known, version, completion, compat, command, quota (siblings, all strong)
NativeQuotaPlan -> actor, no_dispatch, pre-lineage; never -> custody/Prepared
NativeReadyLineage -> NativePreparationCommit only (no actor/custody edge beyond A)
PreparedPhaseNoCurrentDispatch -Weak-> custody
NativeTransportCommand: immutable data, no edges
NativeCompatDeclaration <- NativeAdapter (installed) and <- compat qualification
```

- No new strong edge returns to Runtime, PhaseJobs, JobState, NativeSessions, the NativeAdapter registry or a JoinHandle. The declaration Arc and the command Arc are immutable data with no back-edges.
- Hashing, parsing, encoding, profile/filesystem work and awaits happen outside the Store, custody, queue and child mutexes. A custody mutex is held only to install a slot (pointer-checked, exactly once; the `lineage` slot is replaced only by the Store method's known value for the SAME plan).
- No caller-supplied authority callback exists. `parked_level` is a nongrant `watch` level that Root only reads.

### 3.4 Known readiness lineage (P)

1. **Initial.** Created once from the SAME known `NativePreparationCommit`. Its validator is the unchanged A `validate_version_ready` (original successor, readiness `(preparing,2,NULL)`). It is used only while those original images are current: S5 issuance, Admit-first and the first Park. The helper validators (`state/.../version.rs:371,444,490`) keep using it unchanged; they run only before S5.
2. **Quota.** Created only inside the quota Store method, or `confirm_phase_quota`, for the SAME plan Arc, after a known commit or an exact-postimage confirm. Its readiness and Unit are that plan's own planned postimages; versions are never recomputed. Its successor comes from a new crate-private `CurrentWorkflowSuccessor::with_known_unit(&self, Body<ExecutionUnit>)`. That constructor:
   - keeps the SAME original marker plan, Workflow body and ledger count/head of the predecessor lineage's successor;
   - re-applies the immutable-identity predicate of `current_unit` (`successor.rs:63–99`) to the new body, requiring a strictly greater version;
   - reads no row.

   Its validator runs `validate_common` factored as `validate_common_with(tx, &current)`: actor original, selected DB, `validate_preparation_origin_tx` with the lineage successor, Unit facts and governing digest, `registration_unit`, `no_registration` and owner v1. It then validates the complete lineage readiness image. The original successor is never re-applied after the first quota write.
3. **Transitions.** Exactly these:

   | Transaction | Preimage lineage | Postimage lineage |
   |---|---|---|
   | Admit-first | Initial | Initial (no readiness/Unit change) |
   | Park | Initial: `(preparing,2,NULL)`, Unit U₀ | Quota: `(parked,3,3)`, U₀+1 |
   | Re-park | Quota parked `(parked,3,3)`, Uₖ | SAME lineage when the reason is unchanged; otherwise Quota `(parked,3,3)`, Uₖ+1 |
   | Due-claim admit | Quota parked | Quota: `(preparing,4,NULL)`, Uₖ+1 |
   | Closure | latest lineage readiness image only; the Unit is compared as its latest factual image, not a lineage image (§7.4) | none (closed) |

   A due-claim admit always proceeds to S7, so no Park ever starts from a Quota `preparing` lineage.
4. **Not authority.** Rows, IDs and equal-looking images never create or advance a lineage. A lineage only fixes the preimages that the SAME actor's next write must match. The nongrant closure's factual Unit read (§7.4) is no exception: it is compared, never turned into a lineage.

## 4. Compatibility declaration (P)

### 4.1 Location and schema

The runtime config gains `AgentConfig.compatibility: Option<NativeCompatConfig>`:

```toml
[agents.claude-main]
provider = "claude"
command  = ["claude"]
[agents.claude-main.compatibility]
profile      = "rrx-native-inherited-v1"   # only supported value
cli_version  = "2.1.283"                   # claude pin; codex: "codex-cli 0.160.0"
settings     = "inherited"                 # only supported value
[[agents.claude-main.compatibility.user_hooks]]
label     = "fmt-on-edit"                  # 1..64 B, [A-Za-z0-9._-]
reference = "/Users/me/.claude/settings.json#hooks.PostToolUse"  # opaque, never read
writes    = "worktree"                     # "worktree" | "none"
```

The committed project config gains a list of required hooks, merged into a new `Config.native.required_hooks`:

```toml
[[native.required_hooks]]
path   = "<repo-relative path>"
writes = "none"                            # "worktree" | "none"
```

- **Bounds.** `user_hooks` and `required_hooks` each hold at most 16 entries. Each `reference` and path is 1..1024 B, UTF-8, with no NUL or control characters.
  - A project path is relative and normalized: no `.` or `..` segment, no leading `/`, no backslash. Entries are unique by path.
  - `writes` is a closed, nonsecret enum of exactly two values. An unknown value, including any output-only or common-repository value, refuses at load.
  - The canonical declaration encoding is ≤16 KiB. Every struct uses `deny_unknown_fields`.
- **`writes` meaning (cooperative statement, not verified by rrx).**
  - `none`: the hook declares no filesystem writes.
  - `worktree`: the hook may write tracked or untracked files inside the SAME Unit worktree (the Agent's source tree).
  - No vocabulary exists for a derived-output profile. agent-execution design §5 requires one to be separately qualified, and none is provided here.
- **Ownership.**
  - `compatibility` is runtime-only. `ProjectAgentOverlay` is unchanged, so a project cannot set it; `deny_unknown_fields` already refuses it.
  - `native.required_hooks` is project-only. `Config::load` refuses a non-empty runtime-file `[native]` before the project is applied.
- **Defaults and semantics.**
  - An absent `compatibility` means the alias is **undeclared**. A managed Native start refuses at S1 with typed `UnsupportedCompatibility("native compatibility undeclared")`.
  - There is no default profile, no default `writes` value and no universal compatibility. An absent project `[native]` means no project-declared required hooks.
  - Legacy, unprotected adapters ignore both fields.
  - The fields are not a claim flag. They are cooperative user evidence, not a security proof.

### 4.2 Equality and captured references

1. **Installation.** The installed `NativeAdapter` is built from the runtime registry entry. It captures `Arc<NativeCompatDeclaration>` immutably, beside `program` and `origin_id`.
   - The declaration is created only by the registry constructor from parsed config. No setter exists.
   - `selected_adapter()` pointer identity, which already holds (`adapter/native.rs:78–96`), therefore implies the SAME declaration.
2. **S1 static checks (outside all locks).** All must hold:
   - The declaration exists.
   - `profile` and `settings` are the supported values.
   - `cli_version` equals the provider pin: claude `2.1.283`, codex `codex-cli 0.160.0`.
   - The SAME Frame's merged `config.agents[alias].compatibility` encodes to the SAME canonical digest as the installed declaration. This connects the declaration to the original Source pin. Projects cannot alter it, and it is runtime-frozen just like `command`/`provider` (`workflow_source.rs:1068–1079`).
   - Each `Config.native.required_hooks` path exists in the SAME Frame's `CommittedIndex::inventory()` as an ordinary, non-skipped blob. Its `{path, oid, sha256, bytes}` is borrowed, never re-read. Because `rules:config` is digested, the list and each `writes` value are pinned by `R`.
   - **Role rule.** The role is the SAME allocation `f.role`, cross-checked against the original Unit kind (Executor↔Executor, Reviewer↔Reviewer). One function, `qualify_role_hooks(role, declaration, required_hooks)`, applies one rule to every user hook and every project required hook:
     - Executor: each `writes` is `worktree` or `none`.
     - Reviewer: each `writes` is `none`. Any `worktree` entry refuses with typed `UnsupportedCompatibility("reviewer source is readonly; source-writing hook declared")`. This happens before S2, so no helper runs. The Reviewer source is readonly (`results.rs:602–626`), and agent-execution design §5 requires source-writing hooks to use a separately qualified derived-output profile or refuse before effects. Native never falls back to writable source, never disables or overrides a required hook, and never substitutes a derived-output profile.

     The result is captured as `NativeCompatQualification.role`; S4 and S4b reuse it and never re-decide it.
3. **S4 observed check.** The SAME closed version observation's `qualified_profile()` must equal the label derived from `cli_version` (`claude-2.1.283` / `codex-cli-0.160.0`). Provider, program and origin must equal the SAME allocation facts (`allocation.facts().provider/program`, `launch` origin). A mismatch refuses with no further effect.
4. **Command producer and guard (S4b; re-checked in S7 and S8).**
   - `plan_native_command(owner, actor, completion, compat)` is the sole protected producer of `NativeTransportCommand` (C-7). It runs once, outside all locks, after S4 and before S5. It builds the transport HOW §6 vector from the SAME actor's allocation facts only (`f.program`, `f.path`, `f.role`, `f.model`, `f.effort`, `f.provider`). It requires the SAME closed completion whose seal worktree is `f.path`, and it reuses the version helper's bounded physical profile qualification. The Claude native session UUID is generated here, once. It takes no Prepared value, row, ID, callback or DTO, and its output grants nothing. The result is installed once in the custody `command` slot.
   - `NativeCompatQualification::check_command(&NativeTransportCommand)` then requires all of:
     - argv equals the approved transport HOW §6 vector exactly. The `--settings` JSON is exactly `{"forceLoginMethod":"claudeai"}`, with no `hooks`, `disableAllHooks` or `permissions` key. No `--setting-sources`, `--dangerously-skip-permissions` or `--bare`. No Codex `-c` or `--config` override. `--permission-mode plan` is present exactly when `compat.role` is not Executor.
     - Environment overlay keys are a subset of the fixed `ResourceProfile::environment` key set.
     - None of `HOME`, `CODEX_HOME`, `CLAUDE_CONFIG_DIR`, `XDG_CONFIG_HOME`, `GIT_CONFIG_GLOBAL`, `GIT_CONFIG_SYSTEM` or `GIT_CONFIG_NOSYSTEM` is set.
     - `GIT_CONFIG_*` pairs are exactly `{gc.auto, maintenance.auto, core.fsmonitor}`; never `core.hooksPath`.
   - S7 re-runs `check_command` on the SAME retained command (pointer-equal to the slot). S8 re-runs it, together with transport HOW §6's physical program/cwd recheck, on that SAME command before `plan_prepared_transport`, and never rebuilds it.

   This is a mutation-detection conjunct over rrx's own command. It does not prove the Agent's behavior.
5. **Persistence.** The canonical declaration digest is added to the transport HOW §4.3 `command_digest` material (correction C-2). No raw reference, path or settings value is persisted, logged or hashed beyond that digest. rrx never opens, stats or hashes a `user_hooks.reference`, a user settings file, hook storage or credential storage.

### 4.3 Honest supported/unsupported behavior

| Case | Behavior |
|---|---|
| User-level settings/hooks (`~/.claude/settings.json`, `~/.codex/config.toml`) | Inherited by the official CLI through the unchanged environment and HOME. rrx does not parse them. `user_hooks` only records the user's cooperative statement |
| Committed project settings/hooks (e.g. tracked `.claude/settings.json`, tracked hook scripts) | Present in the fresh worktree with bytes proven equal to `R` by the Status correspondence (readonly HOW §6). Declared paths are checked against the SAME inventory |
| Untracked or ignored local project settings (e.g. `.claude/settings.local.json`) | Not present in a fresh worktree; Status refuses if they appear. Hooks configured only there are **unsupported** for managed runs. Declaring them is impossible (not in the inventory) |
| Hooks declared `worktree` on a Reviewer (user or project required) | Refused at S1 before any helper effect. The Reviewer source is readonly; no derived-output hook profile exists in this HOW |
| Hooks writing only to an output directory, a mutable common repository or sibling paths | No `writes` value exists for them, so declaring them refuses at load. A derived-output hook profile needs its own HOW and conformance. Undeclared behavior is uncovered and reported if discovered |
| Undeclared hooks that write the Reviewer's source | Uncovered. The readonly tree makes such writes fail cooperatively; this is not proof that no write was attempted |
| Git hooks used by the Agent's own Git commands | Inherited through the common directory or `core.hooksPath`. rrx sets no hooks override. rrx's own readonly actions run no hook |
| Other CLI versions | Refused as `UnsupportedCapability`. Extending pins needs its own conformance fixtures and review |
| Native permission mechanisms | Preserved: Claude `--permission-prompt-tool stdio` routing and `--permission-mode plan` for non-Executor; Codex app-server permission framing. Nothing is skipped or downgraded |

## 5. `PreparedPhaseNoCurrentDispatch` (P)

1. **Issuer.** Only `issue_no_current_dispatch(custody)` creates one, inline in the SAME start after S4b. It requires all of:
   - the actor is open;
   - the known readiness commit and its Initial lineage;
   - the SAME `NativeReadonlyHelperCompletion` (Executor, or the §9 Reviewer equivalent), whose history is 14 (Reviewer: 10) linked, Confirmed and qualified helpers;
   - the `transport` slot is empty and no `KnownTransportRegistration` was ever produced;
   - no delegated effect is admitted in this lane (none exists);
   - the count of attempted helpers equals the count of settled helpers (from the custody helper manifest, not rows).
2. **Read-only Immediate.** Under a NEW guard from the SAME `PhaseEffectAdmission`, it checks all of:
   - the Initial lineage validator, i.e. the unchanged `NativePreparationCommit::validate_version_ready` (which includes `validate_preparation_origin_tx` against the original successor), with readiness exactly `(preparing, 2, parking_version NULL)`, and `actor.validate_open`;
   - the negative conjuncts `validate_negative_tx`:
     - Session record absent for `allocated_session_id`;
     - owner v1 with `validated=0` and `native_invocation_id NULL`;
     - no `native_invocations` row for the invocation ID;
     - no `managed_phase_admissions` row for the pair;
     - the complete effect inventory, under `InventoryBudget`, equals the completion's SAME final `after`;
   - the exact Unit image (`Preparing`, `session_id None`, `wait_reason None`), which here is the original successor's Unit.

   A row-absence result alone never issues the value; the local conjunct comes first.
3. **Lifetime.** It is stored once in custody. It is invalidated, by clearing the slot, when the transport custody is installed. The negative conjuncts are lineage-independent. A parked→preparing due-claim re-runs them in its own Immediate under the SAME parked Quota lineage validator (§3.4); the issued Initial images are never used as a later preimage. Restart, epoch change, or a missing custody means none can exist. It is no permission, input or success. Consumers:
   - the S6 quota plan (both Admit and Park);
   - the nongrant preparation closure (§7.4);
   - Root's typed non-success closure (absent prerequisite RN-1, §15).

## 6. Pre-Session quota (P)

### 6.1 Pool, caps and policy

- **Pool.** Exactly one selected pool: `(provider, "unknown")`, with `provider` taken from the SAME allocation facts. This is the existing conservative provider-wide pool (`execution/quota.rs:60–68`). No account alias or credential-derived identity is used. A future official native account identifier needs its own HOW.
- **Caps.** Taken from the SAME selected `NativeSessions`' configured `NativeLimits` and `QuotaScheduler` values, which are immutable after installation: `global_total ≤ 6`, `provider_total`, `provider_executor`, and project = `min(configured, current Project.max_tasks)`. Also the existing Store bound ≤1024.
- **Four-Task consequence (V by reading).** Executor slots are `global_total − 2 = 4` globally and `provider_executor = 2` per provider. So four concurrent Executor Tasks need two Claude and two Codex. Configuration can only lower caps. This matches agent-execution design §8 and is stated as an honest limit.
- **Policy factoring.** The decision logic of `quotas.rs:160–312` moves into a pure `quota_policy::decide(&QuotaSnapshot, at) -> Decision { Admit { probe }, Wait { reason, due } }`, shared by the legacy and private routes. It covers: caps; the high-utilization executor throttle; exhausted/probe/backoff; same-Task capacity due; role alternation by `last_role`; first eligible candidate. Legacy behavior is unchanged except for §6.4 classification. For an absent pool, `decide` receives the default image D (§6.3) and no windows.

### 6.2 Bounded snapshot (two-pass, limit+1, outside SharedStore)

| Inventory | Selection | Limit (+1 sentinel) | Bytes |
|---|---|---|---|
| Active leases (global) | `quota_leases WHERE active=1` scalar columns joined `execution_units(project_id, task_id, kind)` | 4096 | scalar index ≤4 MiB total for all scalar inventories |
| Due waiters in pool | `quota_waiters` scalar columns joined Unit `kind, project_id, task_id, native_effects_open, version`, plus marked-class columns (§6.4) | 4096 | (shared 4 MiB) |
| Same-Task capacity history | `execution_units WHERE task_id=? AND id<>?` | 256 | bodies each ≤16 KiB |
| Legacy candidates ahead of self | full bodies only for those ahead in fair order; used for the capacity predicates and branch planning, never for a validity verdict (§6.4 item 5) | ≤4096 | all candidate+history bodies ≤72 MiB |
| Windows | selected pool | 64 | body ≤8192 each |
| Pool / own waiter / own lease | exact row, or exact absence | 1 each | ≤8192 each |
| Own Unit / readiness | exact, equal to the SAME pre-lineage images | 1 | ≤16 KiB / ≤4096 |
| Whole plan | — | — | ≤8 MiB excluding borrowed manifest |

- The first pass uses `typeof`/`length(CAST(… AS BLOB))` with checked arithmetic before copying. The second pass extracts and compares only after a complete first pass, in the SAME read transaction, under the existing `InventoryBudget`.
- **Cold pool.** An absent selected pool is a normal preimage: a genuine first start for that provider has no observation and no legacy admission. Absence is recorded as `PoolImage::Absent`, together with zero windows, zero leases and zero waiters for that pool (all FK-dependent and compared).
- An overflow, a VM interrupt or an oversize value is a refusal with no effect. A refusal is never truncated success, and nothing is silently pruned.
- Cost: one admission is O(due waiters ahead + leases), as in legacy. A large unrelated history can cause an honest refusal.

### 6.3 Transactions and exact images

Every private quota Immediate runs under a NEW guard from the SAME launch admission, and its common conjunct is all of:
- `admission.validate_for(launch)` and `actor.validate_open()`;
- the SAME pre-lineage validator (§3.4): Initial for Admit-first and Park; the parked Quota lineage for Re-park and Due-claim admit. This includes `validate_preparation_origin_tx` (current successor + Driver-live) with that lineage's successor, and the Unit authority facts, effect-open, parent activity and governing digest, exactly as in transport HOW §9 (never generic `validate_authority` for the own unit);
- `no_registration`, owner v1, and the complete inventory == completion `after`;
- the SAME no-dispatch value and its negative conjuncts;
- planned `at` within 5 s of now; otherwise replan;
- for Admit-first, Park, Re-park and Due-claim admit whose decision depends on the head, the E-1 head walk (§6.4 item 5).

The Closure row uses the §7.4 nongrant conjunct instead of the list above.

Images below are complete columns, with full body bytes where present. Every write is `WHERE` all columns `IS` the preimage, and the rowcount must be exactly 1. An INSERT requires exact absence. Every compare and the E-1 head walk complete before the first write of the Immediate; a mismatch or an E-1 abort rolls back with no write.

The pool default image is D = `(provider, 'unknown', next_probe_at 0, probe_unit NULL, backoff 60000, last_role 'reviewer')`, which equals the DDL defaults (`execution.sql:117–123`).

| Transaction | Readiness (permit, `ExactRowMutation`) | `quota_waiters` | `quota_leases` | `quota_pools` | `execution_units` |
|---|---|---|---|---|---|
| **Admit-first** (never parked) | unchanged `(preparing,2,NULL)`, compared | absent, compared | INSERT `(unit, provider, 'unknown', role, owner_epoch, 1)`, or UPDATE of an exact inactive own row | cold: INSERT D (exact absence), then the UPDATE below against D in the SAME Immediate; existing: complete image compared. UPDATE `last_role`, and probe fields if `probe` (as `quotas.rs:329–335`). A cold pool has no windows, so it is never exhausted and never probes | unchanged `(Preparing, U₀, wait_reason None)`, compared |
| **Park** | `(preparing,2,NULL)` → `(parked,3,3)`; body keys unchanged except state/version/parking_version | INSERT `(unit, provider, 'unknown', reason, due, fairness_sequence=at, resume_state='preparing')` | none, compared | cold: INSERT D (postimage D); existing: complete image compared, unchanged | `wait_reason None→Some(reason)`, `version U₀→U₀+1`, `updated_at`; `state` stays `Preparing` |
| **Re-park** (due, still waiting) | unchanged `(parked,3,3)`, compared | UPDATE `reason, next_due` only (fairness kept) | compared | exists (waiter FK); complete image compared | `wait_reason` updated only if the reason changes (+1) |
| **Due-claim admit** | `(parked,3,3)` → `(preparing,4,NULL)` | DELETE exact image | as Admit-first | exists; as Admit-first for an existing pool | `wait_reason Some→None`, +1 |
| **Closure** (§7.4) | latest lineage `(parked,3,3)` or `(preparing,2\|4,NULL)` → `(closed, V+1, start_ended=1, parking_version NULL)` | DELETE exact image, or accept absence only when the actor is revoked | `active 1→0` on an exact own image, or accept absent/inactive | own probe (`probe_unit` = own unit): `probe_unit→NULL`, `next_probe_at→MAX(pre.next_probe_at, at+pre.backoff)`, `backoff` and `last_role` unchanged (as `release_quota_tx`, `quotas.rs:515–521`). Foreign or NULL probe: complete image compared, unchanged. Absent pool: absence compared | untouched; its latest complete factual image is compared by exact CAS (§7.4) and may be the `fence_task_tx` postimage (Unit retirement belongs to the trusted cancel/fence owner) |

1. **Unit state stays `Preparing`.** Because of this, the transport HOW §5.3 `registration_unit` predicate is unchanged. Prepared's Unit image is exactly the `NativeQuotaAdmitted` lineage Unit: U₀ after Admit-first, or the due-claim postimage. It is never recomputed.
2. **Readiness lineage.** P is 2 when never parked, or 4 after one Park to `(parked,3,3)`, any number of Re-parks at version 3, and one due-claim admit (§3.4). `parking_version` is non-NULL only in `parked`, where it equals that row's version. This is the actual representation of the approved "readiness parkingVersion". The existing `binding_admission_not_parked` trigger keeps consumption impossible while parked.
3. **Approved "waiter version" (correction C-6).** `quota_waiters` has no version column. The approved exact waiter/version promise is represented by three things together:
   - (a) complete seven-column image CAS;
   - (b) the Unit version, which advances on every park and reason change;
   - (c) the readiness `parking_version`.

   No DDL is added. A delete-and-recreate of an identical image by another writer is not an authority, because authority is the in-memory custody plus the readiness row. It is a cooperative limit.
4. **Known commit.** `NativeQuotaAdmitted`, `NativeParkedPhase` and the next `NativeReadyLineage` are issued only by the Store method for the SAME plan Arc, after the commit returns `Ok`. An uncertain commit, for any transaction above including Closure, leads to `confirm_phase_quota(plan)`:
   - exact postimage of every planned row, including the pool default or postimage → issue for the SAME plan;
   - exact preimage of all rows, including pool absence → definitive rollback → replan allowed (Closure: the SAME closure plan may be retried), because no external effect exists;
   - anything else → Held.

   Rows never construct any of these values.
5. **Contention.** A foreign change to the compared lease, waiter, pool or window inventory is a conflict detected before any write. This includes a pool created by legacy admission or `observe_quota` (`quotas.rs:60,179`) after a cold snapshot. It is a definitive pre-write refusal, so it may replan with the SAME actor. At most 8 replans per wake are allowed, then 100 ms–5 s capped backoff. This costs liveness, not safety.
6. **Probe.** A marked unit may become the pool probe through `decide`. Admit with `probe` sets `probe_unit` to the own unit, `next_probe_at = at+backoff` and `backoff = MIN(backoff*2, 1800000)`, as `quotas.rs:329–331`. Recording a probe observation (`observe_quota_from_probe`, generic authority) stays refused for marked units until G2 (§10.2). Non-probe `observe_quota` (pool metadata, no authority) is unchanged. Before registration, the only release of an own probe is the Closure row. It clears the own probe with the same cooldown as `release_quota_tx`, so a sibling genuine Unit may obtain the probe once `at ≥ next_probe_at` (`quotas.rs:302–305`). It never clears or changes a foreign probe. After registration, terminal `release_quota_tx` (`terminal.rs:736–737`) is unchanged.

### 6.4 Candidate classification (both routes) and marked-sibling accounting

1. **Lease accounting is a union and stays unfiltered.** Every active lease (legacy or marked) counts in global, provider, executor and project counts. No validator outcome removes a lease from a count.
2. **Classifying a due waiter `w`.** Classification uses one bounded join on `managed_phase_operations(unit_id, phase_open=1)` and `managed_phase_readiness`. There are three classes:
   - **Legacy.** No open operation exists for the unit. The legacy route decides it with the unchanged generic validator inside its own Immediate, as today (`quotas.rs:286–301`). The private route decides it only through E-1 (item 5). In both routes a Legacy that the unchanged validator refuses is skipped, which is the unchanged legacy semantics for its own class.
   - **MarkedParked.** An open operation exists and all of these hold:
     - readiness `parked` with `parking_version = version`;
     - Unit `Preparing` with `wait_reason` equal to `w.reason`;
     - `w.resume_state = 'preparing'`;
     - pool equal;
     - `w.next_due ≥ at − 30 s`.

     It **holds its fair position**. Generic validation is never applied to it.
   - **MarkedStalled.** Marked, but any MarkedParked predicate fails. It holds no position (it cannot be resumed), is reported as attention, and is never deleted by a generic writer. Its lease, if any, still counts.
3. **Head selection.** The head is the first candidate in the existing fair order that passes the capacity predicates and is either a validated Legacy or MarkedParked.
   - The legacy route admits only if the head is itself. Otherwise it waits `Capacity at+1000`, now including the case where the head is a marked sibling.
   - The private route admits only if the head is its own unit, or if the own unit has no waiter and no due candidate precedes it.
4. **Why stalls drop out.** A live MarkedParked owner re-checks within 1 s (Capacity) or at its due time (Quota). An unclaimed waiter more than 30 s overdue implies a held or absent owner. Excluding it bounds starvation. This is fairness, not safety; capacity safety comes from the lease union.
5. **E-1: Legacy validation inside the private Immediate (the only lock-contract exception, §11.3).**
   - **Why an exception and not a preplan.** The exact verdict of `validate_authority(tx, &candidate.authority(), true, false)` includes the Driver liveness conjunct `rrx_live_task_driver` (`driver.rs:330–335`). That SQL function reaches the Runtime `DriverRegistry` only on the Store writer connection: the writer-contract registration installs it with an empty `Weak` (`state/mod.rs:92`, `driver.rs:289–317`), and only `attach_runtime_drivers` attaches the registry (`driver.rs:355–357`, `execution/owner.rs:255–274`). The query-only planning connection (`managed_binding/snapshot.rs:29–58`) registers neither, so a preplanned verdict cannot be the exact verdict. Preplanning the other conjuncts would also mean retaining, and re-comparing under the lock, each candidate's Project (≤1 MiB), Goal (≤4 MiB), Task (≤1 MiB), Workflow record (≤8 MiB), Context, Driver and Source images (the Driver snapshot bounds, `driver.rs:97–99`, and its Workflow read). That does not fit the unchanged ≤8 MiB plan bound (§6.2), and it would be a second implementation of the Driver and Source pin predicates.
   - **Scope.** Only the private Admit-first, Park, Re-park and Due-claim admit Immediates, and only when the plan's decision depends on the head. Never the Closure, S5, helper, registration or any other private Immediate. Never for the own unit, which keeps the §6.3 lineage conjuncts. Never for a MarkedParked or MarkedStalled candidate.
   - **Two precomputed branches.** `decide` runs only in the planner. Its result depends on the head only when the own unit has no exhaustion or probe wait, no same-Task capacity due and no capacity cap (`quotas.rs:302–312`). In that case the plan carries both complete image sets, built outside SharedStore: `own_head` (Admit-first, or Due-claim admit) and `other_head` (Park, or Re-park, with reason `Capacity` and due `at+1000`, as `quotas.rs:308–309`). Otherwise the plan carries one image set and no walk runs. The walk only selects a branch. It never runs `decide`, never builds or encodes an image and never computes a private digest.
   - **Head walk (inside the Immediate, after every compare, before any write).** Walk the due candidates in the existing fair order (`quotas.rs:278`). The walk ends at the own waiter (Re-park, Due-claim admit), or at the end of the due list when the own unit has no waiter (Admit-first, Park). For each candidate:
     1. One shape query requires the candidate's `execution_units.body` ≤16 KiB, `projects.body` ≤1 MiB, `goals.body` ≤4 MiB and `tasks.body` ≤1 MiB, each present with `typeof` text. These are the §6.2 Unit bound and the Driver snapshot bounds. A failure aborts the Immediate.
     2. `unit_tx(&tx, id)` and `project_capacity_blocked(&tx, &candidate, project_max)`, exactly as `quotas.rs:287–288`. An error aborts the Immediate, just as its `?` aborts the legacy transaction. A capacity-blocked or executor-throttled candidate is passed over, as `quotas.rs:288–296`.
     3. MarkedParked: it is the head; stop. MarkedStalled: pass over.
     4. Legacy: call the unchanged `validate_authority(&tx, &candidate.authority(), true, false)`, the SAME call the legacy route makes at `quotas.rs:297`. `Ok`: it is the head; stop. `Err`: pass over, which is the unchanged legacy semantics for its class.

     Reaching the own waiter, or the end of the list with no head, selects `own_head`. Any head selects `other_head`.
   - **Limit.** At most 8 validator calls per Immediate (`LEGACY_HEAD_CALLS = 8`). If the walk would need a 9th call, the Immediate aborts. A candidate that was not evaluated is never treated as refused, absent or passed over.
   - **Abort.** Every abort in the walk is a definitive pre-write refusal: no waiter, lease, pool, Unit or readiness write, no helper and no registration. It replans within §6.3 item 5 and reports the attention `legacy head unresolved`. The start stays cancellable through §7.4. This costs liveness, never safety: a malformed or over-bound candidate is never passed over, so it can never let the own unit move ahead in the fair order.
   - **Never trusted.** The plan holds no Legacy verdict, validity boolean, governing digest or Driver/Source result, and the Immediate reads none from it. A verdict from one Immediate is never carried to another. An `Ok` only makes another unit the head, so the own unit waits. An `Err` only passes over that candidate. Neither grants the own unit anything; the own unit's authority is only its §6.3 lineage conjuncts.
   - **Compared identity/CAS evidence.** These complete plan images are compared before the walk; any difference is a definitive pre-write conflict and replans:
     - the own unit: every §6.3 common conjunct;
     - the selected pool image or its absence, including `last_role` (the fair-order key), `next_probe_at`, `probe_unit` and `backoff`, and the complete window inventory (≤64);
     - the complete global active-lease inventory joined to Unit `project_id, task_id, kind`, which feeds every count;
     - the complete due-waiter inventory of the pool: all seven waiter columns, the joined Unit `kind, project_id, task_id, native_effects_open, version`, and the class columns (an open `managed_phase_operations` row for the unit; readiness `state, version, parking_version`);
     - the own waiter and own lease images, or their absence;
     - the same-Task capacity history (≤256).
   - **Evaluated live under the lock, never compared to a plan value.** The shape query, `unit_tx`, `project_capacity_blocked` (current `Project.max_tasks`, as in legacy) and the validator call. As today, the validator reads the runtime epoch, `task_execution.generation`, the Unit, Project, Goal and Task, `execution_context.governing_digest`, `goal_authority`, `task_drivers` with the Driver liveness callback, the Driver snapshot's Workflow and Context records, and `source_recoveries`.
   - **Race.** If a Legacy verdict changes between the plan and the Immediate, the walk selects the other precomputed branch, which is still an exact-preimage CAS. A change to the fair order, a class or a count is a compared-inventory conflict before any write.
   - **Legacy route.** Unchanged. It keeps calling the validator inside its own Immediate, with no E-1 limit or shape query (`quotas.rs:286–301`).

### 6.5 Writer/consumer matrix (who may change these rows)

| Writer | Quota/Unit/readiness behavior | Private consumer result |
|---|---|---|
| Private route (this HOW) | exact-image writes above, including a cold pool's D INSERT and the Closure own-probe release | — |
| Legacy admission (other Tasks) | own unit only; may create the pool; changes pool/lease inventory | conflict → replan |
| `observe_quota` (non-probe) | may create the pool; writes windows | conflict → replan |
| `fence_task_tx` / trusted cancel | deletes Task waiters, releases leases (including an own probe, through `release_quota_tx`) | revoked actor → closure accepts absence and compares the fenced Unit's latest factual image (§7.4); open actor → Held |
| `begin_execution_epoch` (new epoch) | deletes all waiters, deactivates leases, nulls every probe | in-memory custody is gone; readiness remains `parked`/`preparing` → Held visible (cross-epoch gate) |
| Terminal (`terminal.rs:736–737`) | releases own lease and own probe | after registration only |
| Old10 cached/new-open binary | generic validation refuses marked candidates → skip (fairness only); may self-admit legacy units | conflict → replan; never a grant |
| Raw same-user SQL | outside the cooperative model | — |

**Permission layout, stated honestly.** Readiness writes use `with_exact_permit` (permit table). The quota tables and `execution_units` are not permit tables. Their protection is the private producer, exact-image CAS and the writer-contract10 compatibility guard. The guard is not authority, and these writes are not called ExactRowMutation. No DDL, trigger, permit slice, catalogue or `SCHEMA_VERSION` change is required. Existing complete images suffice: readiness `parking_version`, Unit version, the seven waiter columns, the four pool columns and the DDL pool defaults. A DB-level fence against compatible writers would need its own ordered Binding11 migration and is not delivered here.

## 7. Parking, due-claim, cancel and restart (P)

### 7.1 In-task parking (Root handoff)

- The parked wait runs inside the SAME Runtime-owned start task, which Root's PhaseJobs entry already retains independently of the Engine future.
- A protected start never returns `NativePhaseStart::Waiting`. Returning it would end the future, and `claim_start` is one-shot with no resume consumer (correction C-4).
- **Root responsibilities.**
  - The PhaseSupervisor slot and the PhaseJobs entry stay charged while parked: ≤128 total, plus per-Project allowance. Driver distinct-Task counting already includes `phase_open=1` operations (`driver/claim.rs:337–338`).
  - `InvocationObservation::Waiting` is derived by Root from the custody's nongrant `parked_level` watch while `outcome` is `None`.
  - Root never claims, starts or resumes from that observation.
- **Derived status.** Workflow, CLI and Driver status read `readiness.state='parked'` + Unit `wait_reason` + waiter `next_due` and report WaitingQuota or WaitingCapacity. No Task or Workflow row is written. This follows the binding design.

### 7.2 Exact due-claim and resume

1. The loop runs `select!(sleep_until(due.min(now+cap)), custody.revocation())`.
   - `cap` is 1 s for Capacity.
   - For Quota, `due` follows the pool's `next_probe_at`. It uses the reported reset when present, otherwise the existing 60 s–30 min backoff. No busy loop.
2. On wake it makes a fresh bounded snapshot and a plan for the SAME actor, no-dispatch and parked value, with the SAME parked Quota lineage as pre-lineage. The Immediate validates that lineage (readiness `(parked,3,3)`, its Unit image and successor), the exact waiter image, the complete pool image and the no-dispatch negative conjuncts. Then:
   - `decide → Admit`: the due-claim admit row of §6.3;
   - `decide → Wait`: re-park.
3. The claim consumes the parked record once. Readiness leaves `parked`, so a second claim fails the CAS.
4. **What resume reuses.** The SAME actor, pair, Session/invocation IDs, input bytes, version observation, Git completion, compat qualification and S4b command. No helper is relaunched and no command is rebuilt. Uncertain helpers can never reach here, because S5 requires all settled. No new Unit, Context, marker, Session, owner or PhaseAttempt is created. `claim_start`, `prepare_managed` and the Workflow Waiting path are not re-entered.
5. Due changes never reset the custody's retained `first_parked_at`. No new expiry is invented. Waits are cancellable.

### 7.3 Original elapsed bound and Unknown

- An unknown capacity classification never turns into Admit.
- Quota exhaustion before a Session is not work failure. No terminal and no Lost is written.
- A Held parked operation stays visible.

### 7.4 Cancel and closure

1. Root's trusted stop (G3 `request_stop`) revokes the actor and notifies the parked loop. The SAME task then plans the nongrant **preparation closure**, using the readonly HOW §10 same-owner/epoch snapshot rules. It omits the current/Driver/admission conjuncts, and it keeps:
   - `validate_preparation_original`;
   - the latest complete factual Unit image as an exact CAS. The Unit is not changed by closure. The image is read and checked only by the existing `LatestUnitImage` port (`version/closure.rs:11–153`): `read` copies all 13 columns under the bounded snapshot of the SAME selected owner (pointer-equal `RuntimeOwner` and selected DB, as `plan_phase_version_closure`, `version/closure.rs:349–382`); `validate_original` requires the SAME allocation's original immutable identity (id, scope, kind, generation, `owner_epoch`, phase, provider, worktree, branch, base, profile, cookie, `created_at`; version ≥ original); `validate_tx` re-checks the complete image in the Immediate. It is not the lineage Unit image, because a trusted `fence_task_tx` (`state/execution.rs:323–356`) may already have retired the Unit (`Retired`, `Cancelled`, both open flags false, absent work → `Unknown`, version+1) before releasing the own lease, waiter and probe; a lineage Unit CAS would then always be stale. A foreign identity or another owner/epoch refuses with no write;
   - the latest known lineage's complete readiness image as the preimage;
   - the inventory == completion `after`;
   - the no-dispatch value (still valid: no registration).

   This factual Unit read exists only for this nongrant closure. It never issues or advances a `NativeReadyLineage`, `NativeQuotaAdmitted`, `NativeParkedPhase`, Prepared, permission or grant. Admit-first, Park, Re-park, Due-claim admit, S7, `issue_prepared` and registration keep the known lineage Unit image and never read the Unit row to refresh it.
2. It writes the Closure row of §6.3, including the own-probe release. It then releases the same-Unit gate once, outside all locks (preparation HOW §3.2 "trusted cancel/genuine closure"). It offers the SAME no-dispatch value to Root's typed non-success closure (RN-1). Until RN-1 exists, the operation stays Held after the factual closure.
3. **Invariants.** Closure issues no permission, input or success. Unknown is never upgraded. Known work is not overwritten. A foreign probe is preserved.
4. **Failures.** A Store error keeps the SAME plan. A definitive pre-write image mismatch, such as a fence committed after the closure snapshot, writes nothing and replans the closure from a fresh bounded snapshot within the §6.3 item 5 limits. An uncertain closure commit runs `confirm_phase_quota` on the SAME closure plan (§6.3 item 4). At most one exact probe runs per wake, with 100 ms–5 s backoff.
5. The same closure applies after Admit but before S8. That path releases the own lease and, if held, the own probe.

### 7.5 Restart and lost owner

- Nothing is reconstructed from rows. After Runtime exit or a new epoch, waiters and leases are fenced by `begin_execution_epoch`. Readiness stays `parked` or `preparing`, and the operation is Held under the existing Driver Task-wide pending/unknown and cross-epoch gates.
- An explicit qualified closure or fresh-attempt producer is a separate open gate, not provided here.

## 8. `issue_prepared` (P)

1. **Sole issuer.** `issue_prepared(custody) -> Result<Arc<PreparedNativePhase>>`, inline in the SAME start after S6 returns `Admitted`. The `prepared` slot is set once, pointer-checked. A second call returns the SAME Arc or refuses; it never makes a new one.
2. **Conjuncts.** All are SAME-pointer checks on custody slots:
   - actor open;
   - known commit;
   - the version helper `closed()` with `qualified_profile` Some;
   - the completion's history whose first link is that version ack (C1 of the transport HOW);
   - compat whose `version` is `Arc::ptr_eq` to that helper, and whose `role` equals `f.role`;
   - the SAME S4b command, `Arc::ptr_eq` to the custody `command` slot, and `compat.check_command` re-run on it;
   - quota `Admitted` whose plan holds the SAME no-dispatch value;
   - readiness and Unit images taken from the `Admitted` lineage: `(preparing,2,NULL)` with U₀ (Initial), or `(preparing,4,NULL)` with the due-claim Unit postimage (Quota); never recomputed;
   - inventory manifest = the completion's final `after` (quota adds no effect row);
   - inventory ≤ the §11.4 prepared bound of the SAME allocation provider, Codex ≤248 and Claude ≤252 (transport HOW §13 as corrected by C-9);
   - role ↔ Unit kind.
3. **Not a source of authority.** It cannot be built from rows, IDs, a version string, an exit status, a completion alone, a command alone or `NativeQuotaAdmitted` alone. It grants nothing until the transport HOW's registration consumes it once.
4. **Fault rule.** A fault between S7 and the registration known commit leaves Prepared retained and the lease held. Closure (§7.4) is the only release.

## 9. Reviewer, later Executor and retry (P for Native; prerequisites marked)

### 9.1 Native-side generalization (defined here)

- `NativeGitSourceSeal::new(actor)` becomes `NativeSourceSeal::from_custody(&SourceNativeCustody)`. It returns an enum over the kind that the SAME Source custody carries:
  - `Executor { frame_R, worktree }`, today's seal for any generation ≥1. The Unit is whatever the Source lane genuinely prepared;
  - `Reviewer { frame_A, snapshot: &ResultSnapshot, artifact }`.
- Native never chooses the kind and never accepts an artifact ID from a caller.
- **Reviewer batch** (`PhaseReviewerGitAction`, 9 actions, same intent→spawn→capture→CAS route, kind `native_phase_git`):

  | Action | Requirement |
  |---|---|
  | TaskTop | equals `snapshot.source` |
  | TaskGitDir | equals `<source>/.git` |
  | TaskCommon | equals TaskGitDir, i.e. an independent non-linked repository |
  | Head | equals `A` |
  | Config, IndexEntries, IndexTree, ConversionAttrs, Status | as readonly HOW §§5–6 |

  The project-ownership actions 1–4 and Branch do not apply, because the snapshot is deliberately independent of project storage (agent-execution design §5).
- Snapshot independence is qualified by the SAME `ResultSnapshot` object's creation checks: `--no-local`, no alternates, read-only tree, retained `manifest_sha256` (`results.rs:521–626`). It is not re-derived.
- **Bounds.**
  - Total helpers are 10 (version + 9), which is ≤32.
  - Capture is version 64 KiB + 6×64 KiB (TaskTop, TaskGitDir, TaskCommon, Head, Config, IndexTree) + 3×1 MiB = 3,604,480 B.
  - Inventory (§11.4, C-1 arithmetic): for Claude, version baseline ≤242 and Git `before₁` ≤243, so the post-batch inventory is ≤252; for Codex, ≤238 and ≤239, so it is ≤248.
- The Reviewer command keeps `--permission-mode plan` (keyed on `role != Executor`). The snapshot remains the only source tree the Reviewer is launched in.
- **Reviewer hooks.** The Reviewer uses the SAME §4.2 role rule, not the Executor qualification. Every declared user hook and every project required hook must be `writes = "none"`, or S1 refuses before any helper effect. No derived-output hook profile is offered.

### 9.2 Retained readonly lease

- `ReviewerArtifactLease` is a Source-owned, non-Clone object. It retains:
  - the SAME `ResultSnapshot`, moved from its creator (never cloned from the Engine map);
  - artifact ID, revision `A` and `manifest_sha256`.
- It lives in `SourceNativeCustody` for the operation's life. The Engine `managed_snapshots` map stays legacy-only and is never protected custody.
- The process-local `GitLease` remains serialization, never qualification. An artifact ID, SQL bytes or a successful Git command never qualify an artifact.

### 9.3 Prerequisites (missing; owned by A/Root; not satisfied by this HOW)

1. **PR-1:** Driver continuation after initial Evidence. Today `driven_initial.rs:56–59` refuses it.
2. **PR-2:** Source offers for a later Executor, a retry and a Reviewer. These replace the first-Executor-only predicates at `native_handoff.rs:448–519`. A retry uses a NEW Unit, worktree, branch and namespace (`attempts.rs:352–470`, `resources.rs:294–297`), and a new operation, pair, Session and marker. The prior operation must be closed or Held. It is never reused.
3. **PR-3:** a Reviewer Frame at artifact revision `A`, built from retained objects. Also `ResultSnapshot` creation moved into the Source lane (`attempts.rs:604–711`) with the `ReviewerArtifactLease` handed into Source custody.
4. **PR-4:** artifact retention release must treat an open operation whose custody references the artifact as a live dependency.

Until PR-1 to PR-4 exist, a Reviewer or retry Unit refuses before any Git intent, exactly as today.

## 10. Transport integration, composition and code disposition (P)

### 10.1 Transport integration

- Prepared is the only input to transport HOW §7 step 1. Step 1 takes the SAME Prepared command and never builds another (C-7). It re-runs transport HOW §6's physical program/cwd recheck and `check_command` on that command, outside all locks, and derives `command_digest` from it.
- Registration's preimage readiness and Unit are Prepared's lineage images, `(preparing, P, NULL)` with P ∈ {2, 4}. Its `validate_preparation_origin_tx` uses the lineage successor. It never calls `validate_version_ready`, and it never re-applies the original successor once a quota write exists (C-8).
- A parked or claimed waiter is impossible at registration: the parked state refuses, and an admitted lease is held.
- The registration Immediate additionally compares:
  - the own active lease image;
  - absence of an own waiter;
  - the SAME `NativeQuotaAdmitted` postimage.
- No obsolete allocated-v1 producer and no generic `validate_authority` participates.
- After the registration known commit, `PreparedPhaseNoCurrentDispatch` is cleared. Closure after that point uses the transport HOW §§8.2 and 10, not §7.4.

### 10.2 G2 (registered owner predicates) is required in the same or an earlier increment

It replaces generic `validate_authority` at `native_phase.rs:360,373`, used for dispatch, ACK, projection and terminal. It also adds private registered-actor routes for live `mark_execution_quota_wait/retry/resume` and `observe_quota_from_probe`, keyed by the actor's `Live` state and its exact active lease. Until G2 exists, these stay refused for marked units and composition stays refused.

### 10.3 Composition

- `installed_driver_composition` stays a deliberate refusal until a final composition increment. In that increment:
  - (a) G1–G4 and RN-1 are implemented and independently reviewed;
  - (b) the issuer takes the actual Runtime-owned objects by reference: PhaseSupervisor, PhaseJobs, the Driver registry's native claim producer, the binder, and the NativeSessions built by the real `AgentRegistry::from_managed_config` with installed declarations. It verifies pointer identity within the SAME Runtime;
  - (c) `PreparedInputAdmission` is advertised only by an adapter whose installed declaration passes S1-static and which belongs to that composed Runtime. The capability list is derived from the composition object, not from a static list.
- There is no `#[cfg(test)]`, feature-gated or static issuer. Positive tests must reach composition through the same production constructor. Setup refusal is recorded as such.

### 10.4 Code disposition

| Code | Disposition |
|---|---|
| bail `native/preparation.rs:426–430` | replaced by S4–S8 |
| `start_with_launch` launch parameter and protected branches; `plan_phase_registration`, `register_phase_session`, `validate_phase_preparation` (allocated v1) | removed (transport HOW §14.1) |
| `NativePhaseStart::Waiting`, `RetainedStart::Waiting` | removed; protected start never returns Waiting; the observation is derived (§7.1) |
| `quotas.rs` decision code | factored into `quota_policy`; legacy fairness gains §6.4 classification; legacy unbounded reads remain legacy-only (the private route never uses them) |
| `NativePreparationPlan::validate_common` (`preparation.rs:39–49`) | factored to `validate_common_with(tx, current)`; every existing caller passes the original successor, so behavior is unchanged |
| `CurrentWorkflowSuccessor` | gains crate-private `with_known_unit` (§3.4); `plan_current_phase` unchanged |
| transport HOW `plan_transport_command(owner, prepared)` | replaced by `plan_native_command` (S4b, C-7) |
| Workflow Waiting persist/resume (`workflow.rs:1848–1870,1933–1990`) | frozen for protected scopes (already refused by preflight); legacy unchanged |
| Engine `managed_snapshots` | frozen to legacy; never protected custody |
| `qualified_profile` Codex comparison | fixed (C-3) |

## 11. Effects, state/error transitions and bounds (P)

### 11.1 Effect inventory per original operation

| Kind | Count | Notes |
|---|---|---|
| `native_phase_version` | 1 | |
| `native_phase_git` | 13 (Executor) or 9 (Reviewer) | |
| `native_phase_transport` | 1 (transport HOW) | |
| setup (pre-input) | Codex 6, Claude 1 | Codex: `initialize`, `initialized`, `account/read`, `environment/status`, `account/rateLimits/read`, `thread/start` (`execution/native.rs:1963–2020`). Claude: `initialize` (`:2123`) |
| input | 1 | Codex `turn/start` (`:2035`); Claude user message (`:2149`) |

- Quota, compat and the S4b command add **no** `managed_effects` rows, process or network effects. Rows that are not effects: readiness, Unit, waiter, lease and pool.
- Total helpers ≤32. Aggregate capture: Executor 3,866,624 B; Reviewer 3,604,480 B; both ≤8 MiB.
- The pre-input setup sequence is fixed. Each call admits its journaled effect exactly once, and any error ends Core through `?`; there is no setup retry. A pre-input approval request is answered with an error, never with a dispatch (`native.rs:1521,2142`). So no further pre-input reservation is needed. Post-input `native_permission` dispatches (`:2108,2242`) are not reserved (§11.4).

### 11.2 State table (preparation custody)

| Observation | Next |
|---|---|
| S1 undeclared/mismatch/role-incompatible hook | refuse; custody Held per approved §3.2 (no helper ran) |
| §11.4 budget exceeded before the version intent, the Git batch, S7 or registration | refuse with no new effect, quota or registration row; Held per approved §3.2 |
| S4 version/declaration mismatch | refuse; Held; observations retained |
| S4b command build or `check_command` fails | refuse; Held; observations retained; no quota row |
| S5 negative check fails | Held; never a retry or new helper |
| Quota conflict (including a pool created after a cold snapshot) | replan ≤8/wake, then backoff |
| E-1 abort: shape-query failure, a candidate `unit_tx` or capacity error, or a 9th validator call needed | no write; replan ≤8/wake, then backoff; attention `legacy head unresolved`; cancellable (§7.4) |
| Quota Admit known | S7 |
| Quota Park known | parked loop with the new Quota lineage |
| Quota commit uncertain (any transaction, including Closure) | `confirm_phase_quota`; Held otherwise |
| Due-claim admit known | S7 |
| Stop while parked/preparing | §7.4 closure → gate release → RN-1 or Held |
| Stop after S7 before registration | §7.4 including lease and own-probe release |
| Epoch/restart | Held; no reconstruction |
| S7 conjunct fails | Held; lease kept until closure |

### 11.3 Locks and time

- Snapshot planning, hashing, policy, command building and encoding happen outside SharedStore. The only exception is E-1 (§6.4 item 5): inside the private Admit-first, Park, Re-park and Due-claim admit Immediates, at most 8 calls of the unchanged `validate_authority(…, true, false)` for Legacy candidates, each after its bounded shape query. Those calls hash and encode exactly as the legacy route does today. Nothing else of the private route hashes, encodes, builds images or runs `decide` under SharedStore.
- Each Immediate is synchronous with no await, FS or process. Admission is acquired asynchronously before it and dropped right after.
- The parked loop holds no lock across its sleep.
- At most 128 parked or preparing operations exist. Per operation, a Capacity wake is ≤1/s and a conflict wake follows the 100 ms–5 s backoff.

### 11.4 Provider+role effect admission budget (P)

The existing per-dispatch gate is unchanged and is not widened: `admit_phase_dispatch` admits a setup, input or permission dispatch only while the complete inventory is `<256`, so no dispatch leaves more than 256 rows (`state/execution/native_phase.rs:1293–1298`).

With S the pre-input setup count of §11.1, the first input is admitted iff `prepared + 1 (transport) + S ≤ 255`, i.e. `prepared ≤ 254 − S`. That is 248 for Codex. For Claude it is 253, and the existing conservative 252 is kept. Subtracting the helpers (version 1 plus Git: Executor 13, Reviewer 9) gives the bounds that each protected planner checks before its own intent:

| Provider | Role | Version baseline (before the version intent) | Git `before₁` (before the batch) | Prepared and registration | Inventory before the input dispatch | After input |
|---|---|---|---|---|---|---|
| Codex | Executor | ≤234 | ≤235 | ≤248 | 248+1+6 = 255 | 256 |
| Codex | Reviewer | ≤238 | ≤239 | ≤248 | 248+1+6 = 255 | 256 |
| Claude | Executor | ≤238 | ≤239 | ≤252 | 252+1+1 = 254 | 255 |
| Claude | Reviewer | ≤242 | ≤243 | ≤252 | 252+1+1 = 254 | 255 |

- **Keying.** One crate-private pure table, `native_effect_budget(provider, role)`, keyed on the SAME allocation `f.provider` and `f.role` (the role cross-checked with the Unit kind, §4.2). Any other provider or role refuses before the version intent. The table is not configuration and has no override.
- **Consumers.** The protected version intent (replacing `with_version_intent`'s `≤254`, `state/execution/native_phase/version.rs:305–311`), the protected Git batch (replacing `reserve_git_batch`'s `≤242`, `:280–297`), `issue_prepared` (§8 item 2), and transport HOW §7 step 1 and §9 (registration plan and Immediate) through transport HOW §13. Legacy helper planners are unaffected.
- **Before helper effects.** A start whose baseline already exceeds its bound refuses before the version intent. No helper, quota, registration or transport row is written for a start that could not reach its input.
- **First input only.** The budget guarantees only that the fixed pre-input setup and the first input fit. Later dispatches are not reserved. Post-input permission replies (Codex `native.rs:2108`, Claude `:2242`) use what remains: at the maxima, none for Codex and one for Claude. Beyond that, the unchanged gate refuses with "Native effect admission profile exhausted", which is the existing, legitimate later exhaustion. No later dispatch is claimed as guaranteed.
- **Not changed.** Every authenticated and setup RPC keeps its own journaled `native_setup` effect; none is omitted, merged or unjournalled. No history row is pruned. The 256-row, 2 MiB, 8192 B body and VM bounds are not widened. Native authority, permission routing and the helper allocation proofs are unchanged.

## 12. Impact analysis

| Changed / consumed | Consumers checked (V reading) | Impact / handling |
|---|---|---|
| `AgentConfig.compatibility`, `Config.native` (`required_hooks` as `{path, writes}` tables), `ProjectOverlay.native` | `config.rs` load/apply/validate/tests; `workflow_source.rs:1051–1079` (merged Frame config); `workflow.rs:3911–3917` legacy loader; `project.rs:567–577` `effective_config`; struct literal at `execution/phase.rs:301` (test) | Additive optional fields with `deny_unknown_fields` and a two-value `writes` enum. Older binaries refuse configs that use them, which is intended. The legacy loader ignores them. `rules:config` digest changes only for projects that add `[native]` (byte pin, same algorithm). The runtime config is not digested, so no Context digest changes. **No new Context version key**; the declaration connects through the SAME Frame config and the installed adapter pointer |
| Role-specific hook rule (`qualify_role_hooks`) | S1 only; result captured in `NativeCompatQualification.role` and consumed by S4b, S7 and `check_command` (`--permission-mode plan`); Reviewer source readonly (`results.rs:602–626`) | Executor accepts `worktree`/`none`; Reviewer accepts only `none`. Legacy adapters unaffected |
| `NativeAdapter` (declaration field), `AgentRegistry::from_managed_config` | `adapter.rs:330–411`, `adapter/native.rs:22–126,263–271` | Selected identity unchanged; capability advertisement derived (§10.3) |
| `qualified_profile` (C-3) | `native/version.rs:86–153` (sole consumers: version stage, receipt) | Codex newline accepted exactly; no new leniency for Claude |
| `plan_native_command` (C-7) replacing transport `plan_transport_command` | transport HOW §3.1 signature, §4.3 `command_digest`, §6 vector, §7 steps 1 and 4d, `NativeTransportCustody.command` (now `Arc`, SAME as Prepared's) | One producer, run once before quota. S8 rechecks and digests the SAME command; no second command can exist |
| `NativePreparationCustody` slots, `abandon` | `phase_jobs.rs:140,195–228,280–284`; `native/preparation.rs:9–69,104–184` | Added `command` and `lineage` slots; abandon also notifies the parked loop; teardown is not closure |
| `NativePreparationPlan::validate_common` → `validate_common_with`; `NativeReadyLineage` | `begin_native_preparation`, `confirm_native_preparation`, `validate_version_ready` (`preparation.rs:24–27,137–180`); helper txs `state/.../version.rs:371,444,490` | Existing callers keep the original successor and the v2 image, so helper behavior is unchanged. Only quota, due-claim, closure (readiness image only), S7 and registration use the Quota lineage |
| `CurrentWorkflowSuccessor::with_known_unit` (new) | `validate_current_tx` (`successor.rs:261–281`), `validate_preparation_origin_tx` (`phase_supervisor.rs:186–194`); other `plan_current_phase` callers (`binding.rs:320`, `native_phase.rs:243,1114`, `native.rs:242`) | Other callers neither see nor build it. No row read; identity predicate identical to `current_unit` |
| `quotas.rs` → `quota_policy` + classification | all quota callers: `execution/quota.rs`, `native.rs:399–423` (legacy start), `workflow.rs` legacy waits, `terminal.rs:736–737`, `state/execution.rs:340–345,589–591,686`, `driver/executor.rs:212` | Legacy decisions identical except a marked head is no longer skipped; adoption gates are unaffected (they run before the marker) |
| `quota_pools` cold INSERT and Closure own-probe release (private route) | FK dependants (`execution.sql:124–142`); legacy creators `quotas.rs:60,179` (`ON CONFLICT DO NOTHING` tolerates an existing row); probe readers `quotas.rs:302–305,386`; `release_quota_tx` callers (`fence_task_tx`, terminal); `begin_execution_epoch` | The inserted D equals the DDL defaults, so legacy readers see a normal pool. Closure probe semantics equal `release_quota_tx`; foreign probes are untouched |
| `state/execution/native_phase/quota.rs` (new) | readiness permit route (`preparation.rs:108–161` pattern); `write_unit` CAS; `InventoryBudget` | No new table/permit/trigger; writer-contract10 unchanged |
| Readiness `parked`/`parking_version`/`closed` | readers: `native_phase.rs:311–330,982–1000`, `terminal.rs:302–324`, transport HOW §§4.2, 8.2; triggers `schema.sql:173–181` | New states are written only by private routes. Old readers expecting v1/v2/v3 refuse them, which is conservative Held |
| Unit `wait_reason` with `Preparing` | legacy quota (`quotas.rs:401–486`), status/CLI readers, `registration_unit` | Same encoding as legacy Capacity waits (which already keep state); registration sees `None` after admit |
| Transport registration lineage (C-8) | transport HOW §§7 step 1, 9 (`validate_preparation_origin_tx`, prepared readiness P, prepared full Unit CAS) | Registration uses Prepared's lineage successor and images; no other transport check changes |
| Root `InvocationObservation::Waiting` derivation | `phase_jobs.rs` observers, Engine waiters | Nongrant level only |
| Finalization/artifact retention | `artifacts.rs:392–570`, `results.rs:101–135` | Unchanged here; PR-4 is a prerequisite for Reviewer |
| Admission/cleanup | `fence_task_tx`, `begin_execution_epoch`, cleanup intents | Unchanged writers; the private consumer detects and Holds, except that a revoked nongrant closure compares the fenced Unit's latest factual image (§7.4) |
| `LatestUnitImage` (`version/closure.rs:11–153`), reused by the §7.4 closure | version-observation closure (`version/closure.rs:349–415`); transport HOW §8 `close_transport_observation` (same port) | Read-only reuse with unchanged predicates. Used only by nongrant closure; never by a grant, lineage, Prepared or registration path |
| C-1 planner constants, now the §11.4 provider+role table | `state/.../version.rs:280–311` (`reserve_git_batch`, `with_git_intent`, `with_version_intent`); `issue_prepared` (§8); transport HOW §§7 step 1, 9, 13 and 14.3; the unchanged dispatch gate (`native_phase.rs:1293–1298`) | Protected-only; honest refusal before the version or Git intent; a Codex start refuses at a lower baseline than a Claude start; legacy unaffected; no bound widened |
| E-1 (§6.4 item 5): the unchanged `validate_authority` called for Legacy candidates in the private Immediate | `validate_authority` (`state/execution.rs:458–497`) and its legacy callers (`quotas.rs:64,172,297,409,455`); `governing_digest` (`:498–511`; also `:891` and the Source pins snapshot); `runtime::driver::validate` (`driver.rs:318–353`; also `state/execution.rs:783`); `source_recovery::validate_task` (`source_recovery.rs:500–509`; also `state/execution.rs:152`); `rrx_live_task_driver` (`state/mod.rs:92`, `driver.rs:289–317,355–357`) | None of these functions or their other callers change. Only the private route gains bounded calls under E-1. The legacy routes keep their existing unbounded behavior |
| Master current behavior | `master/agent-execution.md` §5 last paragraph, `master/workflow-engine.md` | **No master edit in this HOW.** The source PR updates master only with implemented, verified current facts |

Not affected: Task/Workflow/Context/Driver/Source rows and versions; Session binder; audit kinds (none added); Grok/Codex legacy adapters; `SCHEMA_VERSION` 10 and layout catalogue; user settings and hook storage (never read).

## 13. Controls and qualification gates

1. **Primitive controls (compiled; not lifecycle proof):**
   - declaration parse, bounds and defaults; runtime/project placement refusals; canonical digest stability;
   - `writes` enum: unknown/output-only value refused at load; role rule table (Executor `worktree`/`none` accepted; Reviewer `none` accepted, `worktree` refused) for both user and project required hooks;
   - `check_command` deny cases, including `--permission-mode plan` absent for a Reviewer;
   - C-3 Codex parser: `"codex-cli 0.160.0\n"` accepted; `"codex-cli 0.160.0 \n"`, `"\n\n"` and other versions refused;
   - `quota_policy::decide` equivalence with legacy on a table of snapshots, including an absent pool given D;
   - classification of Legacy, MarkedParked and MarkedStalled including the 30 s boundary;
   - limit+1 sentinels at 4096/4097, 256/257 and 64/65, plus byte boundaries;
   - image-CAS refusal for each changed column, including pool absence vs presence and each pool column;
   - `with_known_unit` refuses an identity change or a non-increasing version;
   - §11.4 budget table, per provider and role: the version baseline at its bound is accepted and bound+1 is refused with zero new rows (Codex Executor 234/235, Codex Reviewer 238/239, Claude Executor 238/239, Claude Reviewer 242/243); Git `before₁` at its bound is accepted and +1 refused with no Git row (235/236, 239/240, 239/240, 243/244); `issue_prepared` and the registration plan accept the prepared bound and refuse +1 (Codex 248/249, Claude 252/253); any other provider refuses before the version intent;
   - E-1 head walk on Store fixtures, checked for equivalence with the legacy loop on identical rows: a Legacy refused by the unchanged validator is passed over and `own_head` is selected; a validated Legacy ahead selects `other_head`; a MarkedParked ahead selects `other_head` with no validator call; a needed 9th call, an over-bound or absent Unit/Project/Goal/Task body and a `unit_tx` error each abort with no write and are never passed over; the validator is never called for the own unit, for a marked candidate or in the Closure Immediate; a head-independent decision runs no walk; the plan type holds no Legacy verdict, digest or Driver/Source result.
2. **Genuine actual-producer controls.** These run only after the real Goal/Driver/Source/marker/job/actor chain reaches S4. Otherwise, record SETUP refusal; it is neither a pass nor a mutant kill.
   - S1 refusal with zero helper rows.
   - S4 version/declaration mismatch, with observations retained and no quota row.
   - S4b command refusal with no quota row; Prepared's command is pointer-equal to the S4b slot, and S8 digests that SAME command.
   - **Cold first start:** no `quota_pools` row for the provider, no observation and no legacy admission. Admit-first reaches `Admitted` and Prepared once, with pool D+`last_role`, and no FK failure or setup refusal. A cold Park, forced by capacity, inserts D and the waiter.
   - Admit-first, then Prepared once, with Task/W versions unchanged.
   - **Park lineage:** Park on a forced exhausted pool → `(parked,3,3)`/U₀+1; a Re-park with a reason change → U+1; then due-claim admit → `(preparing,4,NULL)`. Each step validates against the known postimage, not the original v2 image or successor. The operation, pair, Session, input and helpers are identical, with no second version/Git row and no dispatch.
   - **Probe closure:** probe admit on an exhausted pool, then cancel before registration. The closure clears the own probe with `next_probe_at = MAX(pre, at+backoff)`. A different genuine Unit obtains the probe after the cooldown. A closure under a foreign probe leaves the pool image unchanged.
   - A marked parked head blocks a later legacy candidate; a stalled one does not.
   - Marked plus legacy lease union at caps.
   - Foreign lease change, or a pool created by another writer, between plan and Immediate gives replan without a write.
   - **Legacy verdict race (E-1):** after the private plan, a real Driver invalidation or Goal change makes the validated Legacy head refuse before the Immediate. The walk then selects the precomputed `own_head` with no replan, and the reverse order selects `other_head`. A waiter, lease or pool change instead replans without a write.
   - Commit-uncertain confirm, both postimage and rollback, for Admit (cold and existing pool), Park and Closure.
   - Cancel while parked: closure rows, gate released once, no Session.
   - Cancel after Admit: lease released.
   - `fence_task_tx` race gives absence accepted only when revoked.
   - **Fenced revoked closure:** an actual Park, and separately an actual Admit-first, reaches its known lineage. Then the actual `fence_task_tx` commits and Root's stop revokes the actor, and only then the closure runs. It CASes the exact latest bounded same-owner factual Unit image (the fence postimage, not the lineage Unit), leaves the fenced Unit row byte-identical, writes readiness `closed` once, and releases the same-Unit gate exactly once. Waiter, lease and own probe are accepted as already released by the fence. An injected Unit row with a changed immutable identity or another `owner_epoch` refuses with no write, and the operation stays Held. If the real chain cannot reach Park or Admit, or the actual fence cannot run, this is a SETUP refusal, neither a pass nor a kill.
   - Epoch change while parked gives Held and no reconstruction.
   - Weak probes show no custody/actor/plan cycle.
   - Another Task proceeds.
   - Reviewer with a declared `worktree` hook: S1 refusal with zero helper rows (SETUP refusal until PR-1 to PR-4 exist).
   - **Effect budget (requires G2 dispatch):** a genuine Codex Executor start with version baseline 234 dispatches its six setups and its input through the unchanged gate, which leaves 256 rows; at baseline 235 it refuses before the version intent with zero new rows. A Claude Executor start at 238 leaves 255 rows after its input. Until G2 allows dispatch, record SETUP refusal.
3. **Required compiled mutants** (each must fail its intended assertion):
   - generic `validate_authority` for the own unit;
   - skipping MarkedParked in fairness;
   - filtering marked leases from counts;
   - Unit state set to WaitingQuota on park;
   - `parking_version` not set or not cleared;
   - waiter DELETE without image;
   - due-claim without readiness CAS;
   - re-running the version helper on resume;
   - returning `NativePhaseStart::Waiting`;
   - Prepared without compat, without quota, without the S4b command, or with a re-planned manifest;
   - NoCurrentDispatch from rows only;
   - declaration from the Frame instead of the installed Arc;
   - `check_command` dropping `CODEX_HOME`/`HOME`;
   - C-3 using `trim()`;
   - C-1 constants restored;
   - the Codex bound equal to Claude's (prepared 252; killed by the Codex 235/236 and 248/249 controls), the budget keyed on the role only, or the provider read from a row instead of the SAME allocation;
   - the budget checked only at registration instead of before the version and Git intents;
   - a pre-input `native_setup` dispatch removed, merged or unjournalled, or the dispatch gate widened beyond `<256`;
   - E-1: a Legacy verdict or governing digest computed in the planner and trusted in the Immediate; the validator applied to the own unit or a marked candidate; an over-bound or malformed candidate passed over instead of aborting; the walk continuing past 8 calls; `decide` or image encoding run under SharedStore; a validator `Err` aborting instead of passing over (breaks legacy equivalence);
   - cold start requiring an existing pool, or creating it with `ON CONFLICT DO NOTHING` instead of exact absence;
   - closure leaving the own probe set, or clearing a foreign probe;
   - closure Unit CAS against the lineage Unit image instead of the latest factual image (killed by the fenced revoked closure control); `validate_original` skipped for the closure Unit; the closure's factual Unit image feeding a lineage, `NativeQuotaAdmitted` or Prepared;
   - Initial `validate_version_ready` or the original successor re-applied after Park;
   - a lineage built from current rows;
   - S8 rebuilding the command, or Prepared issued before `check_command`;
   - role-blind hook rule (Reviewer `worktree` accepted), or project required hooks excluded from the role rule.

   Compile or setup failure is never a kill.
4. **Both-host official qualification** (user-approved, later; reported separately from fixtures):

   | Gate | Required observation |
   |---|---|
   | N1 | Claude and Codex, macOS and Linux, one Task: actual start, terminal, commit and review identities |
   | N4 | ≥4 overlapping Tasks (2 Claude + 2 Codex Executors under default caps), then reviewer progression; distinct worktrees, leases and results |
   | H | A committed project hook and a user-level hook observed firing in a managed Executor run, with no rrx override; a Reviewer with a declared source-writing hook refused before helper effects |
   | Q | Actual exhaustion wait and resume, or recorded unverified |
   | Install | `cargo install` |

   Supported versions are exactly the pins. An installed CLI outside them is an honest UnsupportedCapability, not a pass.
5. **Review plan.**
   - This HOW: independent Sol high review. Re-review covers only the delta of confirmed findings.
   - Source increment: the initial review runs three roles in parallel at high: Sol on the implementation diff, Opus on the security/contract diff, Grok on reconciliation only. The fix is made by the side that did not raise the finding. Re-review is by the finding side only, on the fix delta.
   - Grok authors nothing. Same-evidence re-reviews are not repeated.

## 14. Explicit corrections to approved documents (minimum, evidence-backed)

| ID | Correction | Evidence |
|---|---|---|
| C-1 | Protected planners guarantee the §11.4 provider+role pre-registration bound before any doomed effect. Executor: version baseline Claude `≤ 238` / Codex `≤ 234` (was 254) and Git batch `before₁` Claude `≤ 239` / Codex `≤ 235` (was 242). Reviewer: version baseline Claude `≤ 242` / Codex `≤ 238`, and Git `before₁` Claude `≤ 243` / Codex `≤ 239` | readonly HOW §8.6 allows a postimage of 255; transport HOW §§9 and 13 as corrected by C-9 |
| C-2 | Transport HOW §4.3 `command_digest` material additionally includes the canonical compat declaration digest. §6 additionally runs `check_command` | Requirement to connect profile identity to consumers |
| C-3 | Codex phase version accepts exactly the pin with at most one trailing `\n` (optionally preceded by `\r`) | V: `native/version.rs:147–149`, `codex/protocol.rs:27`, `results.rs:702–704` |
| C-4 | A protected start never returns `NativePhaseStart::Waiting`; parking is in-task | V: `claim_start` one-shot (`preparation.rs:46–57`), no resume consumer (`phase_jobs.rs:259–263`) |
| C-5 | Readiness `parking_version` = parked row version, NULL otherwise | V: DDL `schema.sql:117`, only NULL writers today |
| C-6 | Approved "waiter version" is represented as complete seven-column image CAS + Unit version + readiness `parking_version`; no DDL | V: `execution.sql:136–142` has no version column |
| C-7 | Transport HOW §3.1's `plan_transport_command(owner, &Arc<PreparedNativePhase>)` is replaced by `plan_native_command(owner, actor, completion, compat)` (S4b), run once before S5 from the SAME actor/allocation and closed preparation facts. `PreparedNativePhase` retains the resulting `Arc<NativeTransportCommand>`. Transport §7 step 1 "Build `NativeTransportCommand` (§6)" becomes "take Prepared's SAME command; recheck §6 program/cwd and `check_command`; derive `command_digest`". `NativeTransportCustody.command` holds that SAME Arc. The §6 vector, bounds, UUID-once rule and digest material are otherwise unchanged | §8 needs `check_command` before Prepared, but the approved producer consumes Prepared (transport HOW §3.1:216–217, §7 step 1) |
| C-8 | Transport registration (§7 step 1 plan, §9 registration column) validates `validate_preparation_origin_tx` with Prepared's lineage successor and compares Prepared's lineage readiness `(preparing,P,NULL)` with P ∈ {2,4} and its Unit image. It never calls `NativePreparationCommit::validate_version_ready` | V: `preparation.rs:24–27,39–49,108–117`; `successor.rs:261–281` compares the full Unit |
| C-9 | Transport HOW §13's pre-registration bound "≤252 rows (transport + up to 2 setup + 1 input ≤256)" becomes the §11.4 provider bound: Codex ≤248 (transport 1 + setup 6 + input 1 → 256), Claude ≤252 (transport 1 + setup 1 + input 1 → 255). Transport HOW §9's "≤252 rows" and §14.3's "satisfied by the ≤252 baseline" change the same way. The `<256` dispatch gate and the complete ≤256 rows, 2 MiB, body and VM bounds are unchanged | V at fd4a6a16: `execution/native.rs:1487–1507,1963–2035,2123,2149`; `state/execution/native_phase.rs:1293–1298` |
| C-10 | §11.3's lock contract gains exactly one exception, E-1 (§6.4 item 5): at most 8 unchanged `validate_authority(…, true, false)` calls for Legacy candidates in the private Admit-first, Park, Re-park and Due-claim admit Immediates, each after a bounded shape query, selecting one of two precomputed branches. The legacy routes are unchanged | V at fd4a6a16: `state/execution.rs:458–511`; `state/runtime/driver.rs:289–357`; `state/execution/source_recovery.rs:500–509`; `state/mod.rs:92`; `managed_binding/snapshot.rs:29–58` |

No requirement is changed. Agent-execution R5's Task WaitingQuota stays satisfied as derived status, per the already-approved binding design §5.2.

## 15. Unresolved prerequisites and open gates

- **RN-1:** Root typed non-success operation closure (`phase_open 1→0`, `phase_closed` audit). It consumes the no-dispatch value. Absent.
- **G2:** registered owner predicates and live quota routes. **G3:** Root `request_stop` targeting the SAME custody. **G4:** transport child cell. **G5:** composition.
- **PR-1 to PR-4:** Reviewer, later Executor and retry Source lane.
- A qualified derived-output hook profile (needed before any source-writing or output-only hook can run in a Reviewer). Absent; such declarations refuse.
- Cross-epoch closure and restart fresh-attempt producer.
- Primary-source verification of pathspec `attr` and `ls-files -s -t` at the qualified Git (readonly HOW §5.4).
- Source review and tests of the a913203f Git batch.
- Full regression and Clippy RED.
- Installed CLI versions outside the pins.
- N1/N4/H/Q/Install qualification on both hosts.
- E-1 liveness limit: more than 8 refused Legacy candidates ahead keep a private start unresolved (attention, no write) until they are fenced or become valid.
- Post-input dispatch headroom is not reserved by §11.4; a Codex start at its maximum baseline has none.

None of these is satisfied by this HOW. Each corresponding effect stays refused or Held until its real producer exists.
