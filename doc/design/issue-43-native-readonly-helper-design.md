# Issue 43: Native readonly Git helper seam (authorization HOW amendment)

## 1. Status and the single superseded narrowing

1. This is a proposed STRICT, minimal supplement to
   `issue-43-native-version-helper-design.md` (version HOW) and
   `issue-43-native-preparation-integration-design.md` (parent HOW). It is not a
   complete specification, master-design change or implementation claim.
   Source baseline: `76a58b6e3dfa9cedb7e528296fb59d2fa4725666`.
2. It expressly supersedes ONLY these two narrowings, and only far enough to admit
   the finite `native_phase_git` actions in §3 through the SAME private
   owned-proof/complete-inventory-CAS route:
   - version HOW §1: "Parent preparation HOW §6's effect-table exact-permit seam is
     narrowed ONLY for `native_phase_version` …";
   - parent HOW §6: "The version-helper HOW narrows ONLY the `native_phase_version`
     effect-table exact-permit seam …".
   Their other sentences remain in force, including "This exception adds no DDL or
   permission exemption for input, owner, Session, readiness, Workflow, Driver,
   Source, quota or any other effect". The version increment's own statement that
   it "adds no Git helper" stays true for that increment.
3. Unchanged: protected owner/readiness/input/Session/quota permissions; ordinary
   current validators (`validate_authority`, `driver::validate`,
   `source_recovery::validate_task`); installed/static admission and the global
   composition issuer; `SCHEMA_VERSION` 10, layout fingerprint, trigger/permit
   slices and writer-contract catalogue; Task/Workflow/Context/Driver/Source
   versions; the generic `reserve_execution_helper*` allowlist
   (`git_helper|native_version|docker_probe`). Git is a distinct effect kind.
   It is not disguised as `native_phase_version`. No public selector, authority
   mode, permission callback or generic marked-scope bypass is added.
4. Ordinary `git_helper` support, the generic helper allowlist and its decoder do
   NOT authorize this seam. An unprotected `managed_effects` table does not
   authorize it either. Only the actual selected Native start, consuming the
   SAME genuine original Source seal and the SAME actor's known settled helper
   history, may issue these intents.
5. Scope is limited to the first-Executor selected start that exists today
   (`native_handoff.rs:327–351`: Executor, Preparing, generation 1, no artifact).
   Readonly reviewer snapshot/artifact qualification and its lease remain later,
   separately reviewed work. A reviewer Unit refuses before any Git intent.
6. After this stage the start STILL refuses: the deliberate bail at
   `execution/native/preparation.rs:369–371` is kept. Its text is extended, but
   it adds no quota, Session, transport, input or prepared-input proof. This
   amendment does not enable Native. The full MVP goal remains, including
   independent four-Task native work.
7. A separate retained implementation draft exists outside this tree. It is not
   approved source and not a real Native positive. Its selected start captures the
   original version, then refuses before any marked Git intent or spawn. It stays
   that way until this seam has an independent review and reviewed source.

## 2. Exact private producer lineage (baseline)

1. `runtime/phase_handoffs.rs:246–319` `Consumer::transfer` takes the original
   allocation and `PreparationGuard` from `SourceNativeCustody`
   (`workflow_source/native_handoff.rs:50–62,178–195`). It builds
   `PhasePreparationOrigin { source, allocation, ticket: Weak }` (`:52–56`,
   only at `:272–276`).
2. `SourceNativeCustody.frame: Arc<Frame>` is the genuine original Source seal. It
   is built by `Frame::build` (`execution/workflow_source.rs:945–1045`) from
   `read_corpus` (`:1146–1202`): `ls-tree -r -z -l --full-tree <exact OID>`, then
   `cat-file blob` per readable blob. It is bound by `frame.revision ==
   unit.base_sha` (`native_handoff.rs:344`).
3. `runtime/phase_supervisor.rs:910–917` `handoff_phase_marker` builds the only
   `PhaseLaunchParts { marker, retention, origin, preparation: Weak }`. Its
   `validate_preparation_origin_tx` (`:186–194`) conjoins original origin, the
   current Workflow successor and Driver liveness.
4. `phase_jobs.rs:233–256` → `adapter/native.rs:100–126` →
   `execution/native.rs:199–214` → `begin_phase_preparation`
   (`native/preparation.rs:286–372`) produce the actor, the plan and the
   allocated1→preparing2 commit under `PhaseEffectAdmission::enter`
   (`runtime/phase_effect_admission.rs:54–72`).
5. `prepare_phase_version` (`native/version.rs:288–359`) then runs the intent,
   the one-shot spawn, the eager capture and `reconcile`. Reconcile goes through
   settlement and closure to `close_phase_version_observation`
   (`state/execution/native_phase/version/closure.rs:385–413`). That closure
   currently returns `()`.
6. NEW, inline at `native/preparation.rs:366–369`: `prepare_phase_git(custody)`
   runs on the SAME custody, actor, launch and admission object. It runs after
   the version stage returns, and the existing bail still follows it. It is
   private to the selected vtable. It is not a callback, flag or re-entry.
7. NEW crate-private, borrow-only accessors, owned by A/Root and coordinated
   before edit:
   - `SourceNativeCustody::original_frame(&self) -> &Arc<Frame>`, reached from
     `PhaseLaunchParts.origin.source`;
   - `Frame::native_git_expectation(&self)`, which returns revision, OID length,
     the retained `versions` and per-path inventory `{oid, sha256, skipped}`.
   Neither constructs a Frame, custody or grant. Neither is reachable from SQL,
   DTO, IDs or `FakeAgent`.

## 3. Finite readonly Git action set

1. The private `PhaseGitAction` enum has exactly 12 variants. Each argv is
   generated internally from the retained marker parent bodies (`project.root`,
   `project.base_branch`, `task.worktree`) and the retained Frame revision. Every
   variant goes through the unchanged `results::git_command_for`
   (`results.rs:640–664`):
   - prefix `-C <root> -c gc.auto=0 -c maintenance.auto=false
     -c core.fsmonitor=false`;
   - `GIT_OPTIONAL_LOCKS=0`, `GIT_NO_REPLACE_OBJECTS=1`, `GIT_NO_LAZY_FETCH=1`;
   - `GIT_DIR`/`GIT_WORK_TREE`/`GIT_INDEX_FILE` and related variables removed.

   It also uses the unchanged `ResourceProfile::environment` overlay
   (`resources.rs:36–64`: inherited `GIT_CONFIG_COUNT ≤128`, plus 3) and the
   canonical absolute `real_tools["git"]`. There is no shell and no caller
   argv/target/purpose.

| # | Action | Root | Arguments after prefix | Combined cap | Qualified expectation |
|---|---|---|---|---|---|
| 1 | SourceTop | project root | `rev-parse --show-toplevel` | 64 KiB | one line |
| 2 | SourceGitDir | project root | `rev-parse --path-format=absolute --git-dir` | 64 KiB | one line |
| 3 | SourceCommon | project root | `rev-parse --path-format=absolute --git-common-dir` | 64 KiB | one line |
| 4 | SourceRoots | project root | `rev-list --max-parents=0 refs/heads/<base_branch>` | 64 KiB | ≥1 OID lines |
| 5 | TaskTop | worktree | `rev-parse --show-toplevel` | 64 KiB | one line |
| 6 | TaskCommon | worktree | `rev-parse --path-format=absolute --git-common-dir` | 64 KiB | one line |
| 7 | Branch | worktree | `symbolic-ref --quiet --short HEAD` | 64 KiB | one line |
| 8 | Head | worktree | `rev-parse --verify HEAD^{commit}` | 64 KiB | equals Frame revision |
| 9 | Config | worktree | `config --null --get-regexp ^(core\.(bare\|worktree\|sparsecheckout)\|index\.sparse\|extensions\.objectformat)$` | 64 KiB | exit 1 (none), or only `core.bare`/`core.sparsecheckout`/`index.sparse`=false and an object format matching the OID length |
| 10 | IndexFlags | worktree | `ls-files -z -t -v` | 1 MiB | every tag is `H` |
| 11 | IndexTree | worktree | `diff-index --cached --quiet --no-ext-diff --no-textconv --ignore-submodules=none <revision> --` | 64 KiB | exit 0 |
| 12 | Status | worktree | `-c core.untrackedCache=false status --porcelain=v1 -z --untracked-files=all --ignored=matching --ignore-submodules=none` | 1 MiB | exit 0, empty stdout |

2. Actions 1–8 feed the unchanged pure `git::validate_worktree_ownership`. This is
   the same namespace predicate as `git_io.rs:249–310`. It runs in process, on
   the retained marker Project/Task bodies, never on current rows.
3. The only operands are an exact lowercase-hex revision (`valid_oid`) and
   `refs/heads/<base_branch>`. The branch comes from the immutable original
   Project body and must first pass the existing in-process ref-format validator.
   Neither can be parsed as an option.
4. Order is fixed (1→12) and fail-fast. The first mismatch, parse failure,
   incomplete capture or refusal ends the batch. That action is settled, no later
   intent is issued and the start refuses. A nonzero exit that the action allows
   is settled Confirmed, then interpreted by its allowlisted consumer. "Settled"
   never means "prepared".
5. None of these commands contacts a remote, writes refs/index/objects or runs a
   repository hook. Action 9 queries only fixed, non-credential keys. Inherited
   system/global config and hooks stay as they are; `HOME` is not substituted.
   rrx does not read, copy or serialize credentials.

## 4. Retained Source corpus versus physical observation

1. The retained Frame is authoritative for committed content. That covers
   `revision`, the `code`/`rules:config`/`rules:*`/`context:committed_inventory`
   versions and the per-path `{oid, sha256, size, skipped}` inventory, plus the
   text it already retains. These are content-addressed by the exact original
   commit OID. Re-running `ls-tree`/`cat-file` per file would only re-read the
   same immutable objects, so this stage does not do it.
2. Mandatory config/rules are checked in memory, outside all locks, with no Git
   action and no file read. Each `rules:*` path must exist in the retained
   inventory, not skipped, with a matching sha256. `rules:config` must equal the
   retained digest. `code` must equal the revision. A missing entry or a mismatch
   refuses before any Git intent.
3. Full physical-tree equality is the conjunction of these observations:
   - Head (8): HEAD equals the revision;
   - IndexFlags (10): no skip-worktree, assume-unchanged or unmerged entries;
   - IndexTree (11): the index tree equals the revision tree;
   - Status (12): the worktree equals the index, with no untracked and no ignored
     paths and no submodule drift;
   - Config (9): no bare, external-worktree or sparse mode, and an object format
     matching the OID length.
   rrx does not re-hash every physical file. Git's stat-cache comparison and the
   race after the last check remain cooperative limits. This is not tamper
   resistance against the same user.
4. The Frame retains no tree OID, and this stage adds none. Tree identity is
   checked by action 11 against the commit OID. No `write-tree` or other
   object-writing command is used.

## 5. Effect tuple and nine-column writes

| | Old generic tuple (`UnitGit::run_command`, unchanged) | New private tuple |
|---|---|---|
| kind | `git_helper` | `native_phase_git` |
| idempotency_key | `helper-{id}` | `native-git-{id}` (≤256 bytes) |
| expected_target | absolute root path | `git:{action}:{sha256(operation:pair:epoch:profile:revision:action)}` (≤4096 bytes) |
| producer | `reserve_execution_helper_pinned` plus `HelperGuard` | private `reserve_phase_git_intent` only; never `HelperGuard` |

1. The intent INSERT writes the existing nine SQL columns: `id, unit_id,
   project_id, goal_id, task_id, idempotency_key, state='pending', body,
   version=1`. The body is exactly the nine-field `ManagedEffect`
   (`execution/model.rs:281–292`, `deny_unknown_fields`) with an empty receipt.
   The body is ≤8 KiB and contains no raw environment, output, path or error.
2. The settlement UPDATE sets all nine columns `WHERE` all nine are `IS` the
   saved preimage. Only `state` (pending→confirmed|unknown), `body` (state,
   receipt, version) and `version` (1→2, checked add) differ. Identity, scope and
   idempotency columns are byte-identical.
3. The receipt has ≤16 entries, keys ≤64 bytes and values ≤256 bytes, no control
   characters. Fields: `action, creation, capture, exit, bytes, stdout_sha256,
   expectation (match|mismatch|unparsed), hygiene`. Raw stdout/stderr is never
   persisted.
4. Persisted rows are bookkeeping, never authority. Any reader that decodes them
   gets no Git-qualified, prepared or NoCurrentDispatch fact.

## 6. Same original qualified predecessor history and complete inventory CAS

1. `close_phase_version_observation` returns a private
   `NativeVersionSettledCommit { settlement: Arc<NativeHelperSettlementPlan> }`.
   It is issued only after its Immediate observed `current == settlement.after`
   and committed, and the preparation custody retains it.
2. The Git stage starts only if that SAME ack exists, its effect is `Confirmed`
   and its SAME owned observation has `qualified_profile().is_some()`. Unknown or
   unsupported versions, a missing ack and a re-planned or row-derived settlement
   all refuse before any Git intent. A current confirmed row cannot stand in.
3. The history is chained: `before₁` is the SAME `settlement.after` Inventory
   object, and `beforeₖ₊₁` is action k's SAME settled `after`. No stage reads
   current rows to refresh history. A coherent planning snapshot that differs
   from the retained expected set leaves the batch Held.
4. Every intent Immediate conjoins all of the following, in one transaction under
   one `InventoryBudget`:
   - `admission.validate_for(launch)`, from a NEW guard on the SAME
     `PhaseEffectAdmission`;
   - `actor.validate_open()`;
   - `NativePreparationCommit::validate_version_ready`, i.e. `validate_common`
     (selected database, the Source origin/current successor/Driver-live
     conjunction, Unit authority facts, `native_effects_open`, parent activity,
     governing digest, `registration_unit`, `no_registration`, pair owner image)
     plus the pair readiness postimage;
   - an explicit read-only `runtime_epoch.epoch == facts.epoch`;
   - `Inventory::read == beforeₖ`;
   - absence of the planned id/idempotency;
   - `INSERT == 1`.
5. The epoch conjunct is added on purpose. The version stage checks the epoch in
   its snapshot and through Unit row images. That does not prove an independent
   Immediate epoch CAS, and the existing indirect detection through
   `native_effects_open` is not relied on alone.
6. The 50 ms fence repeats the actor, readiness, epoch and `pendingₖ`
   conjunction. Settlement accepts `current == afterₖ` (idempotent) or
   `current == pendingₖ`, followed by the exact nine-column CAS. Anything else is
   Held.
7. Row headroom: `before₁` must have ≤243 rows, so the postimage after action
   12 has ≤255 rows. That keeps the version HOW's reserved 256th native-input
   slot. Later registration/transport intents get no headroom from this stage.
   They must check their own and refuse honestly.

## 7. Raw child, capture and actual observation before parsing or SQL

1. For each action, the pattern is: plan outside locks; retain helper custody;
   build the eager `CaptureOwner` before spawn or first poll; take a NEW
   admission guard; run the intent Immediate (known commit); set
   `attempted = true`; call `Command::spawn` once; and immediately
   `RetainedRawProcess::adopt(child)` before PID, pipe or reader work. This is
   the same as `native/version.rs:302–346`.
2. Admission is dropped before any capture await. A spawn `Err` is
   attempted-without-handle. It is not NoChild and is never retried.
3. Capture retains a bounded `NativeGitObservation` (stdout up to the cap,
   discarded stderr counted, exit, completeness, hygiene) in helper custody
   BEFORE UTF-8/NUL/line parsing, expectation matching or any Store access.
   Parsing reads only that retained observation, and a persistence error cannot
   discard it.
4. Process group, null stdin, 50 ms `MissedTickBehavior::Delay` fence, ≤30 s
   action deadline, ≤2 s drain and ≤10 s best-effort stop/reap all apply. The
   leader stays unreaped until group signaling. Leader exit, ESRCH or empty pipes
   do not show that descendants died.
5. `owner.git_lease(unit, None)` is held, outside Store, across the whole batch,
   with `RRX_GIT_GATE_TOKEN` set as in `git_io.rs:150–154`. It is process-local
   serialization, not a durable fence or grant.

## 8. Factual nongrant closure, distinct from permission

1. Each action has a closure plan that mirrors the version closure. It needs the
   SAME settlement and observation, `actor.validate_original`,
   `launch.validate_preparation_original`, the latest full Unit CAS, and
   inventory equal to `pendingₖ` or `afterₖ`. It deliberately skips the
   current/Driver/admission/epoch conjunctions so that a revoked actor can record
   work it actually did.
2. Closure issues no permission, intent, retry, prepared fact or
   NoCurrentDispatch. It cannot turn Unknown into success or reopen the actor.
   Only the §6.4 intent conjunction permits a new spawn.

## 9. Interference gives Held, not DB fencing

1. Compatible writers stay able to write; a new private consumer detects the
   change. In each case the observation is retained, nothing is replayed, no
   later intent is issued and the result is Held:
   - generic `reconcile_managed_effect(_pinned)` (no kind allowlist,
     `effects.rs:129–171`);
   - epoch `fence_epoch_effects` (pending→unknown, `effects.rs:3–28`);
   - an old10 cached or newly opened writer;
   - delete, replace or body/index drift.
2. Existing10 has no schema fence against these writers, and this design claims
   none.

## 10. Bounds and their consumers

| Bound | Value | Consumer |
|---|---|---|
| Git intents per original operation | 12 (13 including the version helper) ≤ parent 32 | `PhaseGitAction` plan |
| Combined capture per action | 64 KiB (1–9, 11); 1 MiB (10, 12) | capture accounting |
| Aggregate preparation capture | 64 KiB + 10×64 KiB + 2×1 MiB = 2,883,584 B ≤ parent 8 MiB | batch custody |
| Deadlines | 30 s per action, 2 s drain, 10 s stop/reap; 180 s batch from first Git intent | capture task / batch custody |
| Inventory rows | before₁ ≤243; each postimage ≤255; read sentinel at 257 | `Inventory::with_git_intent`, `read` |
| Inventory bytes | body ≤8 KiB; whole charged set ≤2 MiB, re-checked per action; planning also charges 12 maximal rows | plan, intent, settlement |
| SQLite VM work | existing `InventoryBudget` (1000-op interval, interrupt at 1000 callbacks) per routine | every snapshot/Immediate |
| Receipt | ≤16 entries, ≤64/256 bytes | `effects.rs:137–143` reader |
| Inherited Git config overlay | ≤128 entries + 3 | `ResourceProfile::environment` |

1. Overflow, timeout or a VM interrupt means an incomplete observation or Held.
   It never qualifies an action.

## 11. Impact analysis

| Affected | Effect | Handling |
|---|---|---|
| `native/preparation.rs:366–371` | new inline stage before the bail | bail kept |
| `version/closure.rs:385–413` (sole consumer `native/version.rs:174–178`) | returns a private ack | no other caller |
| `NativePreparationCustody` | adds `git` slot, abandon, Drop | same eager-guard rules |
| `native_handoff.rs`, `workflow_source.rs` Frame | borrow-only accessors | coordinated with A/Root |
| `effects.rs` helper allowlist / `reserve_effect_tx` | unchanged | `native_phase_git` not added |
| Driver pending/unknown gates (`driver/preparation.rs:349,546`) | block on native git pending/unknown | intended |
| `driver/executor.rs:210–216`, `state/execution.rs:685–690` | non-`git_helper` refuses | these run before the marker; no regression |
| `version.rs:337–345` baseline | unchanged | Git runs after the single version intent |
| `results::git_command_for`, `ResourceProfile::environment`, `git::validate_worktree_ownership`, `RetainedRawProcess` | reused unchanged | shared consumers unaffected |
| Schema10, permits, Task/Workflow/Context/Driver/Source/Session/owner/readiness/quota rows | unchanged | no permission change |

1. Legacy `execution/native.rs:303–319` is unchanged and unreachable from the
   protected start. Command-only verifier and bootstrap Units refuse first.

## 12. Verification controls and their limits

1. Positive controls must use the actual Goal/Driver/Source/allocation/marker/
   launch/job/actor chain. Rows, DTOs, `FakeAgent`, SQL-seeded actors and static
   issuers do not count. A setup refusal is recorded and is not a positive or a
   mutation kill.
2. Each must fail its intended assertion when exercised:
   - a missing or unqualified version ack;
   - a fresh-read `before`;
   - drift at 243/244 rows and at 2 MiB;
   - a dropped epoch, Source, Driver, pair or readiness conjunct;
   - spawn before intent, and a retried spawn;
   - an error between spawn and adopt;
   - an unpolled capture Drop;
   - HEAD, untracked or ignored files, skip-worktree or assume-unchanged entries,
     index/tree, sparse, bare or object-format mismatch;
   - output at the 64 KiB/1 MiB boundary, and action/batch timeouts;
   - generic reconcile and old10 cached-writer interference;
   - another Task continuing;
   - Task/Workflow versions unchanged.
3. Primitive tests (argv builders, parsers, `Inventory` helpers) cannot prove
   producer causality, intent-before-spawn ordering under real admission races,
   retention across Drop, compiled old10 behavior, process-group and descendant
   death, per-version Git flag semantics, stat-cache races or same-user tamper
   resistance.

## 13. Acceptance condition and unresolved gates

**Acceptance (one line):** the real selected start, holding only the SAME
qualified known-settled version history and original Source seal, runs at most the
12 fixed readonly Git actions, each committed intent-before-one-shot-spawn under
the full current/Source/Driver/Unit/pair/epoch conjunction and exact
complete-inventory CAS, retains every observation, and still refuses, with no
schema, guard, permission or grant change.

1. Unresolved and out of scope:
   - Reviewer snapshot/artifact lease;
   - hooks/settings qualification;
   - quota;
   - Native transport/registration/input/ACK;
   - installed composition issuer and `cargo install` usability;
   - full regression and Clippy (currently RED);
   - macOS/Linux OS qualification;
   - four-Task independence;
   - the independent Sol high review of this HOW, the later source review and
     the matching master delta.
2. Same non-dispatch limits apply: this stage produces no NoCurrentDispatch.
   Process cleanup remains best effort.
