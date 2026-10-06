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
- the SAME captured compatibility qualification of the selected installed adapter's nonsecret declaration against that version observation and the SAME original Frame;
- a SAME-custody `PreparedPhaseNoCurrentDispatch`;
- a known-committed admission in the bounded private quota transaction.

That quota transaction counts every active lease, marked or legacy. Its fairness does not treat a refusal as absence. Parking is the only alternative to admission. It writes only the exact Unit, waiter, lease, pool and readiness images, under the SAME stop admission. It is resumed only by the SAME in-task exact due-claim of the original operation, pair, input and helpers, with Task, Workflow, Context, Driver and Source rows and versions unchanged.

Prepared is consumed once by the approved transport registration, which never uses allocated-v1 or generic marked validators. Composition stays refused until G1–G5 are real. Every other path is no-effect refusal, Held, or nongrant closure.

## 2. Revalidated source map at a913203f (V)

Paths are relative to `crates/rrx/src/`.

| Item | Actual fact at the pinned commit |
|---|---|
| Selected start | `adapter/native.rs:100–126` → `execution/native.rs:200–217` → `native/preparation.rs:344–431`. The steps are: `claim_start` one-shot (`:46–57`); same-Unit `try_lock_owned` gate (`:366–389`); actor; ≤128 preparation index (`:394–406`); readiness allocated1→preparing2 under admission (`:410–423`); `prepare_phase_version` (`:424`); `prepare_phase_git` (`:425`). The start then **refuses** at `:426–430`: "original Native readonly Git observations retained; full preparation and transport unavailable". The later bail at `native.rs:214–216` is unreachable |
| Absent types | `PreparedNativePhase`, `PreparedPhaseNoCurrentDispatch`, `NativeQuotaPlan`, `NativeTransportStartPlan` and `NativePreCoreCustody` do not exist (zero grep hits). `NativePhaseStart::Launched`/`::Waiting` are never constructed; they are only matched in `runtime/phase_jobs.rs:259,263` |
| Version | `NativeVersionObservation::qualified_profile` (`native/version.rs:138–153`) passes **untrimmed** stdout. Codex `verify_native_version` requires exact equality with `"codex-cli 0.160.0"` (`codex/protocol.rs:24–34`), so output ending in a newline never qualifies. Claude uses its first whitespace token `2.1.283` (`execution/claude_wire.rs:26–37`). Legacy paths trim (`results.rs:702–704`). Closed settlement: `NativeHelperSettlementCommit` (`state/execution/native_phase/version.rs:323–327`), issued only by `close_phase_version_observation` (`version/closure.rs:385–415`) |
| Git batch | `prepare_phase_git` (`native/version.rs:460–538`) returns `NativeReadonlyHelperCompletion` (`:15–23`, documented as nongrant) after `confirm_phase_helper_history`. That function requires 14 linked entries, current inventory == last `after`, all confirmed (`state/.../version.rs:393–475`), and `reserve_git_batch` ≤242 rows (`:280–298`). Seal/actions: `native/readonly.rs:31–61,194–302,465–552` |
| Source handoff | First Executor only: `workflow_source/native_handoff.rs:448–519` requires generation 1, Unit generation 1, no artifact and Preparing. `qualify_git` refuses an artifact frame with "actual readonly artifact lease unavailable" (`:102–110`). `driven_initial.rs:56–59` refuses continuation after initial Evidence |
| Reviewer | `ResultSnapshot` (`results.rs:66–80,521–626`) is created by `prepare_snapshot_inner` (`attempts.rs:604–711`). It is retained only in the Engine's in-memory `managed_snapshots` map (`workflow.rs:560–562,1749–1752`). `GitLease` is process-local (`execution/owner.rs:33–37,292–326`) |
| Registration | `plan_phase_registration`/`register_phase_session`/`validate_phase_preparation` require allocated v1 (`state/execution/native_phase.rs:982–1000,1128,1409`) and call generic `validate_authority` (`:1407,1462`). `NativeOwnerPlan::validate_tx`/`validate_terminal_tx` use it at `:360,373`. Only caller is `start_with_launch(…, None)` (`native.rs:195`) |
| Readiness DDL | `managed_binding/schema.sql:110–120`: `state ∈ {allocated, preparing, parked, registered, deferred, held, closed}`; nullable positive `parking_version`; body ≤4096. Permit-gated triggers (`:173–175`); core trigger `version=OLD+1`, monotonic flags (`:178`); `binding_admission_not_parked` and the parked/consumed exclusion (`:179–181`). Nothing writes `parked` or a non-NULL `parking_version` today |
| Quota DDL | `execution.sql:117–142`. `quota_pools` (PK provider+account_key; `next_probe_at`, `probe_unit`, `backoff ∈[60000,1800000]`, `last_role`), `quota_windows` (PK +bucket; `observed_at`, json body), `quota_leases` (PK unit_id; provider, account_key, role, epoch>0, active∈{0,1}), `quota_waiters` (PK unit_id; provider, account_key, reason, next_due, fairness_sequence, resume_state). **No version column on any quota table**; no triggers other than the writer-contract guards |
| Quota code | `state/execution/quotas.rs:149–370` uses generic `validate_authority` (`:172`). Its COUNTs are unfiltered (`:207–213,534`), so they include every active lease. History read is unbounded (`:239–268`). Due waiters are materialized unbounded (`:278`). The fairness loop **skips** a candidate whose generic validation fails (`:286–300`). `fairness_sequence` is an epoch-ms timestamp. `resume_state` is the snake_case `UnitState`. Account key is always `"unknown"` (`execution/quota.rs:60–68,130,220`). Caps: `QuotaScheduler::new` 6/2/3/4 (`quota.rs:15–23`); `NativeLimits::configured` (`native.rs:114–153`) only lowers them; Store `ensure!` ≤1024 |
| Unit | `execution_units` (`execution.sql:6–23`): version CAS via `write_unit` (`state/execution.rs:314–321`). Body `ExecutionUnit` (`execution/model.rs:~199–221`) has `state`, `wait_reason`, `capacity_retry_at`. `UnitState` includes `Preparing` and `WaitingQuota` (`model.rs:91–101`) |
| Permit catalogue | `managed_binding/permits.rs:23–180` covers workflow_native_contracts, managed_phase_operations, managed_marker_bodies, managed_phase_owners, managed_phase_inputs, managed_phase_admissions, managed_phase_readiness, task_drivers, source_recoveries, records, audit. `ExactRowMutation` (`:185–232`) and `with_exact_permit` (`:243–350`) handle 1..=128 rows. `execution_units`, `managed_effects`, `quota_*`, `native_invocations` and `session_units` are **not** permit tables; they carry only writer-contract10 guards (`state/execution.rs:124–138`) |
| Epoch | `begin_execution_epoch` deactivates all leases, nulls `probe_unit` and deletes all waiters (`state/execution.rs:589–591`). `fence_task_tx` (`:340–345`) and terminal (`native_phase/terminal.rs:736–737`) release leases and delete waiters |
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
| S1 | (P) `qualify_compat_static`, outside all locks | none | nothing; refuses before S2 when undeclared |
| S2 | (A) version helper | `native_phase_version` | closed qualified observation (nongrant) |
| S3 | (A, Executor) 13-action Git batch / (P, Reviewer) §9 batch | `native_phase_git` | `NativeReadonlyHelperCompletion` (nongrant) |
| S4 | (P) `qualify_compat_observed` | none | `NativeCompatQualification` (nongrant) |
| S5 | (P) `issue_no_current_dispatch` | none (one read-only Immediate) | `PreparedPhaseNoCurrentDispatch` |
| S6 | (P) quota plan → Admit, or Park ⇄ due-claim loop | quota/Unit/readiness | `NativeQuotaAdmitted` (known commit) |
| S7 | (P) `issue_prepared` | none | `PreparedNativePhase` (one per operation) |
| S8 | (approved transport HOW §7) | registration + transport | per that HOW, requires G2–G4 |

S1 runs before any further process effect, so an undeclared alias wastes no helper. S4 needs the SAME version observation.

### 3.2 Types (all crate-private, non-Clone, non-Serialize/Deserialize, private fields, no row/ID/DTO constructor)

```rust
// execution/native/compat.rs (new)
struct NativeCompatDeclaration { /* parsed, canonical, bounded; digest */ }
struct NativeCompatQualification {
    declaration: Arc<NativeCompatDeclaration>,       // SAME Arc as installed adapter
    version: Arc<NativeVersionHelperCustody>,        // SAME qualified closed observation
    project_hooks: Box<[QualifiedHookRef]>,          // borrowed facts from SAME Frame inventory
    frame: Arc<Frame>,                               // SAME original Frame (via seal)
}
// execution/native/prepared.rs (new)
struct PreparedPhaseNoCurrentDispatch { custody: Weak<NativePreparationCustody>,
    completion: Arc<NativeReadonlyHelperCompletion>, readiness: PairRowImage, unit: UnitImage }
struct NativeQuotaAdmitted { plan: Arc<NativeQuotaPlan>, unit_after: UnitImage,
    readiness_after: PairRowImage }                   // only from Store known commit / exact confirm
struct PreparedNativePhase { actor: Arc<NativePreparationActor>,
    known: Arc<NativePreparationCommit>, version: Arc<NativeVersionHelperCustody>,
    completion: Arc<NativeReadonlyHelperCompletion>, compat: Arc<NativeCompatQualification>,
    quota: Arc<NativeQuotaAdmitted>, readiness: PairRowImage /* preparing, P */,
    unit: UnitImage /* Preparing, U */ }
// state/execution/native_phase/quota.rs (new) + state/execution/quota_policy.rs (factored)
struct NativeQuotaPlan { /* SAME actor, SAME no-dispatch, snapshot, decision, exact images */ }
enum NativeQuotaOutcome { Admitted(Arc<NativeQuotaAdmitted>), Parked(Arc<NativeParkedPhase>) }
struct NativeParkedPhase { plan: Arc<NativeQuotaPlan>, parking_version: u64, due: i64,
    reason: WaitReason, waiter: WaiterImage }
```

### 3.3 Ownership graph

```text
Root PhaseJobs Entry -> Job -> JobState.preparation -> NativePreparationCustody (A)
  slots (each one-time, no history lists):
    actor, plan, known, helpers[<=32] (A); completion (A)
    compat: Option<Arc<NativeCompatQualification>>            (P)
    no_dispatch: Option<Arc<PreparedPhaseNoCurrentDispatch>>   (P; cleared by S8 install)
    quota: QuotaSlot{ current: Option<Arc<NativeQuotaPlan>>,
                      parked: Option<Arc<NativeParkedPhase>>,
                      admitted: Option<Arc<NativeQuotaAdmitted>> } (P)
    prepared: Option<Arc<PreparedNativePhase>>                 (P, set once)
    transport: Option<Arc<NativeTransportCustody>>             (transport HOW)
    parked_level: watch::Sender<ParkedLevel>                   (P, nongrant observation)
Prepared -> actor, known, version, completion, compat, quota   (siblings, all strong)
NativeQuotaPlan -> actor, no_dispatch; never -> custody/Prepared
PreparedPhaseNoCurrentDispatch -Weak-> custody
NativeCompatDeclaration <- NativeAdapter (installed) and <- compat qualification
```

- No new strong edge returns to Runtime, PhaseJobs, JobState, NativeSessions, the NativeAdapter registry or a JoinHandle. The declaration Arc is immutable data with no back-edges.
- Hashing, parsing, filesystem work and awaits happen outside the Store, custody, queue and child mutexes. A custody mutex is held only to install a slot (pointer-checked, exactly once).
- No caller-supplied authority callback exists. `parked_level` is a nongrant `watch` level that Root only reads.

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
writes    = "worktree"                     # only supported value
```

The committed project config gains `[native] required_hooks = ["<repo-relative path>", …]`. This is merged into a new `Config.native.required_hooks`.

- **Bounds.** `user_hooks` and `required_hooks` each hold at most 16 entries. Each `reference` and path is 1..1024 B, UTF-8, with no NUL or control characters.
  - A project path is relative and normalized: no `.` or `..` segment, no leading `/`, no backslash. Entries are unique.
  - The canonical declaration encoding is ≤16 KiB. Every struct uses `deny_unknown_fields`.
- **Ownership.**
  - `compatibility` is runtime-only. `ProjectAgentOverlay` is unchanged, so a project cannot set it; `deny_unknown_fields` already refuses it.
  - `native.required_hooks` is project-only. `Config::load` refuses a non-empty runtime-file `[native]` before the project is applied.
- **Defaults and semantics.**
  - An absent `compatibility` means the alias is **undeclared**. A managed Native start refuses at S1 with typed `UnsupportedCompatibility("native compatibility undeclared")`.
  - There is no default profile and no universal compatibility. An absent project `[native]` means no project-declared required hooks.
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
   - Every hook entry has `writes == "worktree"`.
   - The SAME Frame's merged `config.agents[alias].compatibility` encodes to the SAME canonical digest as the installed declaration. This connects the declaration to the original Source pin. Projects cannot alter it, and it is runtime-frozen just like `command`/`provider` (`workflow_source.rs:1068–1079`).
   - Each `Config.native.required_hooks` path exists in the SAME Frame's `CommittedIndex::inventory()` as an ordinary, non-skipped blob. Its `{path, oid, sha256, bytes}` is borrowed, never re-read. Because `rules:config` is digested, the list itself is pinned by `R`.
3. **S4 observed check.** The SAME closed version observation's `qualified_profile()` must equal the label derived from `cli_version` (`claude-2.1.283` / `codex-cli-0.160.0`). Provider, program and origin must equal the SAME allocation facts (`allocation.facts().provider/program`, `launch` origin). A mismatch refuses with no further effect.
4. **Command guard (consumed in S7 and S8).** `NativeCompatQualification::check_command(&NativeTransportCommand)` requires all of:
   - argv equals the approved transport HOW §6 vector exactly. The `--settings` JSON is exactly `{"forceLoginMethod":"claudeai"}`, with no `hooks`, `disableAllHooks` or `permissions` key. No `--setting-sources`, `--dangerously-skip-permissions` or `--bare`. No Codex `-c` or `--config` override.
   - Environment overlay keys are a subset of the fixed `ResourceProfile::environment` key set.
   - None of `HOME`, `CODEX_HOME`, `CLAUDE_CONFIG_DIR`, `XDG_CONFIG_HOME`, `GIT_CONFIG_GLOBAL`, `GIT_CONFIG_SYSTEM` or `GIT_CONFIG_NOSYSTEM` is set.
   - `GIT_CONFIG_*` pairs are exactly `{gc.auto, maintenance.auto, core.fsmonitor}`; never `core.hooksPath`.

   This is a mutation-detection conjunct over rrx's own command. It does not prove the Agent's behavior.
5. **Persistence.** The canonical declaration digest is added to the transport HOW §4.3 `command_digest` material (correction C-2). No raw reference, path or settings value is persisted, logged or hashed beyond that digest. rrx never opens, stats or hashes a `user_hooks.reference`, a user settings file or credential storage.

### 4.3 Honest supported/unsupported behavior

| Case | Behavior |
|---|---|
| User-level settings/hooks (`~/.claude/settings.json`, `~/.codex/config.toml`) | Inherited by the official CLI through the unchanged environment and HOME. rrx does not parse them. `user_hooks` only records the user's cooperative statement |
| Committed project settings/hooks (e.g. tracked `.claude/settings.json`, tracked hook scripts) | Present in the fresh worktree with bytes proven equal to `R` by the Status correspondence (readonly HOW §6). Declared paths are checked against the SAME inventory |
| Untracked or ignored local project settings (e.g. `.claude/settings.local.json`) | Not present in a fresh worktree; Status refuses if they appear. Hooks configured only there are **unsupported** for managed runs. Declaring them is impossible (not in the inventory) |
| Hooks needing a mutable common repository or sibling writes | `writes` vocabulary admits only `worktree`, so they are refused when declared. Undeclared behavior is uncovered and reported if discovered |
| Git hooks used by the Agent's own Git commands | Inherited through the common directory or `core.hooksPath`. rrx sets no hooks override. rrx's own readonly actions run no hook |
| Other CLI versions | Refused as `UnsupportedCapability`. Extending pins needs its own conformance fixtures and review |
| Native permission mechanisms | Preserved: Claude `--permission-prompt-tool stdio` routing and `--permission-mode plan` for non-Executor; Codex app-server permission framing. Nothing is skipped or downgraded |

## 5. `PreparedPhaseNoCurrentDispatch` (P)

1. **Issuer.** Only `issue_no_current_dispatch(custody)` creates one, inline in the SAME start after S4. It requires all of:
   - the actor is open;
   - the known readiness commit;
   - the SAME `NativeReadonlyHelperCompletion` (Executor, or the §9 Reviewer equivalent), whose history is 14 (Reviewer: 10) linked, Confirmed and qualified helpers;
   - the `transport` slot is empty and no `KnownTransportRegistration` was ever produced;
   - no delegated effect is admitted in this lane (none exists);
   - the count of attempted helpers equals the count of settled helpers (from the custody helper manifest, not rows).
2. **Read-only Immediate.** Under a NEW guard from the SAME `PhaseEffectAdmission`, it checks all of:
   - `validate_preparation_origin_tx`, `actor.validate_open`, and `NativePreparationCommit::validate_version_ready` with readiness exactly `(preparing, P, parking_version NULL)`;
   - Session record absent for `allocated_session_id`;
   - owner v1 with `validated=0` and `native_invocation_id NULL`;
   - no `native_invocations` row for the invocation ID;
   - no `managed_phase_admissions` row for the pair;
   - the complete effect inventory, under `InventoryBudget`, equals the completion's SAME final `after`;
   - the exact Unit image (`Preparing`, `session_id None`, `wait_reason None`).

   A row-absence result alone never issues the value; the local conjunct comes first.
3. **Lifetime.** It is stored once in custody. It is invalidated, by clearing the slot, when the transport custody is installed. A parked→preparing due-claim re-validates it in the same Immediate. Restart, epoch change, or a missing custody means none can exist. It is no permission, input or success. Consumers:
   - the S6 quota plan (both Admit and Park);
   - the nongrant preparation closure (§7.4);
   - Root's typed non-success closure (absent prerequisite RN-1, §15).

## 6. Pre-Session quota (P)

### 6.1 Pool, caps and policy

- **Pool.** Exactly one selected pool: `(provider, "unknown")`, with `provider` taken from the SAME allocation facts. This is the existing conservative provider-wide pool (`execution/quota.rs:60–68`). No account alias or credential-derived identity is used. A future official native account identifier needs its own HOW.
- **Caps.** Taken from the SAME selected `NativeSessions`' configured `NativeLimits` and `QuotaScheduler` values, which are immutable after installation: `global_total ≤ 6`, `provider_total`, `provider_executor`, and project = `min(configured, current Project.max_tasks)`. Also the existing Store bound ≤1024.
- **Four-Task consequence (V by reading).** Executor slots are `global_total − 2 = 4` globally and `provider_executor = 2` per provider. So four concurrent Executor Tasks need two Claude and two Codex. Configuration can only lower caps. This matches agent-execution design §8 and is stated as an honest limit.
- **Policy factoring.** The decision logic of `quotas.rs:160–312` moves into a pure `quota_policy::decide(&QuotaSnapshot, at) -> Decision { Admit { probe }, Wait { reason, due } }`, shared by the legacy and private routes. It covers: caps; the high-utilization executor throttle; exhausted/probe/backoff; same-Task capacity due; role alternation by `last_role`; first eligible candidate. Legacy behavior is unchanged except for §6.4 classification.

### 6.2 Bounded snapshot (two-pass, limit+1, outside SharedStore)

| Inventory | Selection | Limit (+1 sentinel) | Bytes |
|---|---|---|---|
| Active leases (global) | `quota_leases WHERE active=1` scalar columns joined `execution_units(project_id, task_id, kind)` | 4096 | scalar index ≤4 MiB total for all scalar inventories |
| Due waiters in pool | `quota_waiters` scalar columns joined Unit `kind, project_id, task_id, native_effects_open, version`, plus marked-class columns (§6.4) | 4096 | (shared 4 MiB) |
| Same-Task capacity history | `execution_units WHERE task_id=? AND id<>?` | 256 | bodies each ≤16 KiB |
| Legacy candidates ahead of self | full bodies only for those ahead in fair order | ≤4096 | all candidate+history bodies ≤72 MiB |
| Windows | selected pool | 64 | body ≤8192 each |
| Pool / own waiter / own lease | exact rows | 1 each | ≤8192 each |
| Own Unit / readiness | exact | 1 | ≤16 KiB / ≤4096 |
| Whole plan | — | — | ≤8 MiB excluding borrowed manifest |

- The first pass uses `typeof`/`length(CAST(… AS BLOB))` with checked arithmetic before copying. The second pass extracts and compares only after a complete first pass, in the SAME read transaction, under the existing `InventoryBudget`.
- An overflow, a VM interrupt or an oversize value is a refusal with no effect. A refusal is never truncated success, and nothing is silently pruned.
- Cost: one admission is O(due waiters ahead + leases), as in legacy. A large unrelated history can cause an honest refusal.

### 6.3 Transactions and exact images

Every private quota Immediate runs under a NEW guard from the SAME launch admission, and its common conjunct is all of:
- `admission.validate_for(launch)` and `actor.validate_open()`;
- `validate_preparation_origin_tx` (current successor + Driver-live);
- Unit authority facts, effect-open, parent activity and governing digest, exactly as in transport HOW §9 (never generic `validate_authority` for the own unit);
- `no_registration`, owner v1, and the complete inventory == completion `after`;
- the SAME no-dispatch value;
- planned `at` within 5 s of now; otherwise replan.

Images below are complete columns, with full body bytes where present. Every write is `WHERE` all columns `IS` the preimage, and the rowcount must be exactly 1. An INSERT requires exact absence.

| Transaction | Readiness (permit, `ExactRowMutation`) | `quota_waiters` | `quota_leases` | `quota_pools` | `execution_units` |
|---|---|---|---|---|---|
| **Admit-first** (never parked) | unchanged `(preparing,P,NULL)`, compared | absent, compared | INSERT `(unit, provider, 'unknown', role, owner_epoch, 1)`, or UPDATE of an exact inactive own row | UPDATE `last_role`, and probe fields if `probe` (as `quotas.rs:329–335`) | unchanged `(Preparing, U, wait_reason None)`, compared |
| **Park** | `(preparing,P,NULL)` → `(parked,P+1,P+1)`; body keys unchanged except state/version/parking_version | INSERT `(unit, provider, 'unknown', reason, due, fairness_sequence=at, resume_state='preparing')` | none, compared | compared | `wait_reason None→Some(reason)`, `version U→U+1`, `updated_at`; `state` stays `Preparing` |
| **Re-park** (due, still waiting) | unchanged `(parked,PV,PV)`, compared | UPDATE `reason, next_due` only (fairness kept) | compared | compared | `wait_reason` updated only if the reason changes (+1) |
| **Due-claim admit** | `(parked,PV,PV)` → `(preparing,PV+1,NULL)` | DELETE exact image | as Admit-first | as Admit-first | `wait_reason Some→None`, +1 |
| **Closure** (§7.4) | `(parked\|preparing, V, …)` → `(closed, V+1, start_ended=1, parking_version NULL)` | DELETE exact image, or accept absence only when the actor is revoked | `active 1→0` on an exact own image, or accept absent/inactive | untouched | untouched (Unit retirement belongs to the trusted cancel/fence owner) |

1. **Unit state stays `Preparing`.** Because of this, the transport HOW §5.3 `registration_unit` predicate is unchanged. Prepared's Unit image is exactly the `NativeQuotaAdmitted` postimage. Its version U is whatever that known commit wrote; it is never recomputed.
2. **Readiness lineage.** P = 2 + 2k after k park/claim cycles. `parking_version` is non-NULL only in `parked`, where it equals that row's version. This is the actual representation of the approved "readiness parkingVersion". The existing `binding_admission_not_parked` trigger keeps consumption impossible while parked.
3. **Approved "waiter version" (correction C-6).** `quota_waiters` has no version column. The approved exact waiter/version promise is represented by three things together:
   - (a) complete seven-column image CAS;
   - (b) the Unit version, which advances on every park and reason change;
   - (c) the readiness `parking_version`.

   No DDL is added. A delete-and-recreate of an identical image by another writer is not an authority, because authority is the in-memory custody plus the readiness row. It is a cooperative limit.
4. **Known commit.** `NativeQuotaAdmitted` and `NativeParkedPhase` are issued only by the Store method for the SAME plan Arc, after the commit returns `Ok`. Uncertain commit leads to `confirm_phase_quota(plan)`:
   - exact postimage of every planned row → issue for the SAME plan;
   - exact preimage of all rows → definitive rollback → replan allowed (no external effect exists);
   - anything else → Held.

   Rows never construct either value.
5. **Contention.** A foreign change to the compared lease, waiter, pool or window inventory is a conflict detected before any write. It is a definitive pre-write refusal, so it may replan with the SAME actor. At most 8 replans per wake are allowed, then 100 ms–5 s capped backoff. This costs liveness, not safety.
6. **Probe.** A marked unit may become the pool probe through `decide`. Recording a probe observation (`observe_quota_from_probe`, generic authority) stays refused for marked units until G2 (§10.2). Non-probe `observe_quota` (pool metadata, no authority) is unchanged. Lease release on terminal (`terminal.rs:736–737`) or closure bumps `next_probe_at` as today.

### 6.4 Candidate classification (both routes) and marked-sibling accounting

1. **Lease accounting is a union and stays unfiltered.** Every active lease (legacy or marked) counts in global, provider, executor and project counts. No validator outcome removes a lease from a count.
2. **Classifying a due waiter `w`.** Classification uses one bounded join on `managed_phase_operations(unit_id, phase_open=1)` and `managed_phase_readiness`. There are three classes:
   - **Legacy.** No open operation exists for the unit. The existing generic validator decides it, inside the Immediate as today. A refused Legacy is skipped, which is the unchanged legacy semantics for its own class.
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

### 6.5 Writer/consumer matrix (who may change these rows)

| Writer | Quota/Unit/readiness behavior | Private consumer result |
|---|---|---|
| Private route (this HOW) | exact-image writes above | — |
| Legacy admission (other Tasks) | own unit only; changes pool/lease inventory | conflict → replan |
| `fence_task_tx` / trusted cancel | deletes Task waiters, releases leases | revoked actor → closure accepts absence; open actor → Held |
| `begin_execution_epoch` (new epoch) | deletes all waiters, deactivates leases | in-memory custody is gone; readiness remains `parked`/`preparing` → Held visible (cross-epoch gate) |
| Terminal (`terminal.rs:736–737`) | releases own lease | after registration only |
| Old10 cached/new-open binary | generic validation refuses marked candidates → skip (fairness only); may self-admit legacy units | conflict → replan; never a grant |
| Raw same-user SQL | outside the cooperative model | — |

**Permission layout, stated honestly.** Readiness writes use `with_exact_permit` (permit table). The quota tables and `execution_units` are not permit tables. Their protection is the private producer, exact-image CAS and the writer-contract10 compatibility guard. The guard is not authority, and these writes are not called ExactRowMutation. No DDL, trigger, permit slice, catalogue or `SCHEMA_VERSION` change is required. Existing complete images suffice: readiness `parking_version`, Unit version, the seven waiter columns. A DB-level fence against compatible writers would need its own ordered Binding11 migration and is not delivered here.

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
2. On wake it makes a fresh bounded snapshot and a plan for the SAME actor, no-dispatch and parked value. The Immediate requires readiness `(parked, PV, PV)`, the exact waiter image and the exact Unit image. Then:
   - `decide → Admit`: the due-claim admit row of §6.3;
   - `decide → Wait`: re-park.
3. The claim consumes the parked record once. Readiness leaves `parked`, so a second claim fails the CAS.
4. **What resume reuses.** The SAME actor, pair, Session/invocation IDs, input bytes, version observation, Git completion and compat qualification. No helper is relaunched. Uncertain helpers can never reach here, because S5 requires all settled. No new Unit, Context, marker, Session, owner or PhaseAttempt is created. `claim_start`, `prepare_managed` and the Workflow Waiting path are not re-entered.
5. Due changes never reset the custody's retained `first_parked_at`. No new expiry is invented. Waits are cancellable.

### 7.3 Original elapsed bound and Unknown

- An unknown capacity classification never turns into Admit.
- Quota exhaustion before a Session is not work failure. No terminal and no Lost is written.
- A Held parked operation stays visible.

### 7.4 Cancel and closure

1. Root's trusted stop (G3 `request_stop`) revokes the actor and notifies the parked loop. The SAME task then plans the nongrant **preparation closure**, using the readonly HOW §10 same-owner/epoch snapshot rules. It omits the current/Driver/admission conjuncts, and it keeps:
   - `validate_preparation_original`;
   - the latest full Unit CAS (unchanged);
   - the inventory == completion `after`;
   - the no-dispatch value (still valid: no registration).
2. It writes the Closure row of §6.3. It then releases the same-Unit gate once, outside all locks (preparation HOW §3.2 "trusted cancel/genuine closure"). It offers the SAME no-dispatch value to Root's typed non-success closure (RN-1). Until RN-1 exists, the operation stays Held after the factual closure.
3. **Invariants.** Closure issues no permission, input or success. Unknown is never upgraded. Known work is not overwritten.
4. **Failures.** A Store error keeps the SAME plan. At most one exact probe runs per wake, with 100 ms–5 s backoff.
5. The same closure applies after Admit but before S8. That path releases the own lease.

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
   - compat whose `version` is `Arc::ptr_eq` to that helper;
   - quota `Admitted` whose plan holds the SAME no-dispatch value;
   - readiness `(preparing, P, NULL)` and Unit image from the `Admitted` postimage;
   - inventory manifest = the completion's final `after` (quota adds no effect row);
   - inventory ≤252 rows (transport HOW §13);
   - role ↔ Unit kind;
   - `compat.check_command` on the planned transport command.
3. **Not a source of authority.** It cannot be built from rows, IDs, a version string, an exit status, a completion alone or `NativeQuotaAdmitted` alone. It grants nothing until the transport HOW's registration consumes it once.
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
  - Inventory: version baseline ≤242 and Git `before₁` ≤243, so the post-batch inventory is ≤252 (C-1 arithmetic).
- The Reviewer command keeps `--permission-mode plan` (keyed on `role != Executor`). The snapshot remains the only source tree the Reviewer is launched in.

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

- Prepared is the only input to transport HOW §7 step 1. Registration's preimage readiness is `(preparing, P, NULL)` and the Unit image is Prepared's.
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
| setup | ≤2 | |
| input | 1 | |

- Quota and compat add **no** `managed_effects` rows, process or network effects. Rows that are not effects: readiness, Unit, waiter, lease and pool.
- Total helpers ≤32. Aggregate capture: Executor 3,866,624 B; Reviewer 3,604,480 B; both ≤8 MiB.

### 11.2 State table (preparation custody)

| Observation | Next |
|---|---|
| S1 undeclared/mismatch | refuse; custody Held per approved §3.2 (no helper ran) |
| S4 version/declaration mismatch | refuse; Held; observations retained |
| S5 negative check fails | Held; never a retry or new helper |
| Quota conflict | replan ≤8/wake, then backoff |
| Quota Admit known | S7 |
| Quota Park known | parked loop |
| Quota commit uncertain | `confirm_phase_quota`; Held otherwise |
| Due-claim admit known | S7 |
| Stop while parked/preparing | §7.4 closure → gate release → RN-1 or Held |
| Stop after S7 before registration | §7.4 including lease release |
| Epoch/restart | Held; no reconstruction |
| S7 conjunct fails | Held; lease kept until closure |

### 11.3 Locks and time

- Snapshot planning, hashing, policy and encoding happen outside SharedStore.
- Each Immediate is synchronous with no await, FS or process. Admission is acquired asynchronously before it and dropped right after.
- The parked loop holds no lock across its sleep.
- At most 128 parked or preparing operations exist. Per operation, a Capacity wake is ≤1/s and a conflict wake follows the 100 ms–5 s backoff.

## 12. Impact analysis

| Changed / consumed | Consumers checked (V reading) | Impact / handling |
|---|---|---|
| `AgentConfig.compatibility`, `Config.native`, `ProjectOverlay.native` | `config.rs` load/apply/validate/tests; `workflow_source.rs:1051–1079` (merged Frame config); `workflow.rs:3911–3917` legacy loader; `project.rs:567–577` `effective_config`; struct literal at `execution/phase.rs:301` (test) | Additive optional fields with `deny_unknown_fields`. Older binaries refuse configs that use them, which is intended. The legacy loader ignores them. `rules:config` digest changes only for projects that add `[native]` (byte pin, same algorithm). The runtime config is not digested, so no Context digest changes. **No new Context version key**; the declaration connects through the SAME Frame config and the installed adapter pointer |
| `NativeAdapter` (declaration field), `AgentRegistry::from_managed_config` | `adapter.rs:330–411`, `adapter/native.rs:22–126,263–271` | Selected identity unchanged; capability advertisement derived (§10.3) |
| `qualified_profile` (C-3) | `native/version.rs:86–153` (sole consumers: version stage, receipt) | Codex newline accepted exactly; no new leniency for Claude |
| `NativePreparationCustody` slots, `abandon` | `phase_jobs.rs:140,195–228,280–284`; `native/preparation.rs:9–69,104–184` | Added slots; abandon also notifies the parked loop; teardown is not closure |
| `quotas.rs` → `quota_policy` + classification | all quota callers: `execution/quota.rs`, `native.rs:399–423` (legacy start), `workflow.rs` legacy waits, `terminal.rs:736–737`, `state/execution.rs:340–345,589–591,686`, `driver/executor.rs:212` | Legacy decisions identical except a marked head is no longer skipped; adoption gates are unaffected (they run before the marker) |
| `state/execution/native_phase/quota.rs` (new) | readiness permit route (`preparation.rs:108–161` pattern); `write_unit` CAS; `InventoryBudget` | No new table/permit/trigger; writer-contract10 unchanged |
| Readiness `parked`/`parking_version`/`closed` | readers: `native_phase.rs:311–330,982–1000`, `terminal.rs:302–324`, transport HOW §§4.2, 8.2; triggers `schema.sql:173–181` | New states are written only by private routes. Old readers expecting v1/v2/v3 refuse them, which is conservative Held |
| Unit `wait_reason` with `Preparing` | legacy quota (`quotas.rs:401–486`), status/CLI readers, `registration_unit` | Same encoding as legacy Capacity waits (which already keep state); registration sees `None` after admit |
| Root `InvocationObservation::Waiting` derivation | `phase_jobs.rs` observers, Engine waiters | Nongrant level only |
| Finalization/artifact retention | `artifacts.rs:392–570`, `results.rs:101–135` | Unchanged here; PR-4 is a prerequisite for Reviewer |
| Admission/cleanup | `fence_task_tx`, `begin_execution_epoch`, cleanup intents | Unchanged writers; the private consumer detects and Holds |
| C-1 planner constants | `state/.../version.rs:280–311` | Protected-only; earlier honest refusal; legacy unaffected |
| Master current behavior | `master/agent-execution.md` §5 last paragraph, `master/workflow-engine.md` | **No master edit in this HOW.** The source PR updates master only with implemented, verified current facts |

Not affected: Task/Workflow/Context/Driver/Source rows and versions; Session binder; audit kinds (none added); Grok/Codex legacy adapters; `SCHEMA_VERSION` 10 and layout catalogue.

## 13. Controls and qualification gates

1. **Primitive controls (compiled; not lifecycle proof):**
   - declaration parse, bounds and defaults; runtime/project placement refusals; canonical digest stability;
   - `check_command` deny cases;
   - C-3 Codex parser: `"codex-cli 0.160.0\n"` accepted; `"codex-cli 0.160.0 \n"`, `"\n\n"` and other versions refused;
   - `quota_policy::decide` equivalence with legacy on a table of snapshots;
   - classification of Legacy, MarkedParked and MarkedStalled including the 30 s boundary;
   - limit+1 sentinels at 4096/4097, 256/257 and 64/65, plus byte boundaries;
   - image-CAS refusal for each changed column.
2. **Genuine actual-producer controls.** These run only after the real Goal/Driver/Source/marker/job/actor chain reaches S4. Otherwise, record SETUP refusal; it is neither a pass nor a mutant kill.
   - S1 refusal with zero helper rows.
   - S4 version/declaration mismatch, with observations retained and no quota row.
   - Admit-first, then Prepared once, with Task/W versions unchanged.
   - Park on a forced exhausted pool, then due-claim admit with identical operation/pair/Session/input and no second version/Git row.
   - A marked parked head blocks a later legacy candidate; a stalled one does not.
   - Marked plus legacy lease union at caps.
   - Foreign lease change between plan and Immediate gives replan without a write.
   - Commit-uncertain confirm, both postimage and rollback.
   - Cancel while parked: closure rows, gate released once, no Session.
   - Cancel after Admit: lease released.
   - `fence_task_tx` race gives absence accepted only when revoked.
   - Epoch change while parked gives Held and no reconstruction.
   - Weak probes show no custody/actor/plan cycle.
   - Another Task proceeds.
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
   - Prepared without compat, without quota, or with a re-planned manifest;
   - NoCurrentDispatch from rows only;
   - declaration from the Frame instead of the installed Arc;
   - `check_command` dropping `CODEX_HOME`/`HOME`;
   - C-3 using `trim()`;
   - C-1 constants restored.

   Compile or setup failure is never a kill.
4. **Both-host official qualification** (user-approved, later; reported separately from fixtures):

   | Gate | Required observation |
   |---|---|
   | N1 | Claude and Codex, macOS and Linux, one Task: actual start, terminal, commit and review identities |
   | N4 | ≥4 overlapping Tasks (2 Claude + 2 Codex Executors under default caps), then reviewer progression; distinct worktrees, leases and results |
   | H | A committed project hook and a user-level hook observed firing in a managed run, with no rrx override |
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
| C-1 | Protected planners guarantee transport HOW §13's ≤252 pre-registration inventory before any doomed effect. Executor: version baseline `≤ 238` (was 254) and Git batch `before₁ ≤ 239` (was 242). Reviewer: version baseline `≤ 242` and Git `before₁ ≤ 243` | readonly HOW §8.6 allows a postimage of 255; transport HOW §§9 and 13 require ≤252 |
| C-2 | Transport HOW §4.3 `command_digest` material additionally includes the canonical compat declaration digest. §6 additionally runs `check_command` | Requirement to connect profile identity to consumers |
| C-3 | Codex phase version accepts exactly the pin with at most one trailing `\n` (optionally preceded by `\r`) | V: `native/version.rs:147–149`, `codex/protocol.rs:27`, `results.rs:702–704` |
| C-4 | A protected start never returns `NativePhaseStart::Waiting`; parking is in-task | V: `claim_start` one-shot (`preparation.rs:46–57`), no resume consumer (`phase_jobs.rs:259–263`) |
| C-5 | Readiness `parking_version` = parked row version, NULL otherwise | V: DDL `schema.sql:117`, only NULL writers today |
| C-6 | Approved "waiter version" is represented as complete seven-column image CAS + Unit version + readiness `parking_version`; no DDL | V: `execution.sql:136–142` has no version column |

No requirement is changed. Agent-execution R5's Task WaitingQuota stays satisfied as derived status, per the already-approved binding design §5.2.

## 15. Unresolved prerequisites and open gates

- **RN-1:** Root typed non-success operation closure (`phase_open 1→0`, `phase_closed` audit). It consumes the no-dispatch value. Absent.
- **G2:** registered owner predicates and live quota routes. **G3:** Root `request_stop` targeting the SAME custody. **G4:** transport child cell. **G5:** composition.
- **PR-1 to PR-4:** Reviewer, later Executor and retry Source lane.
- Cross-epoch closure and restart fresh-attempt producer.
- Primary-source verification of pathspec `attr` and `ls-files -s -t` at the qualified Git (readonly HOW §5.4).
- Source review and tests of the a913203f Git batch.
- Full regression and Clippy RED.
- Installed CLI versions outside the pins.
- N1/N4/H/Q/Install qualification on both hosts.

None of these is satisfied by this HOW. Each corresponding effect stays refused or Held until its real producer exists.
