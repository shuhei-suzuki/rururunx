# Issue 43: Native readonly Git helper seam (authorization HOW amendment, corrected r2)

## 1. Status, superseded narrowing and corrections

1. This is a proposed, strict and minimal supplement to `issue-43-native-version-helper-design.md` (the version HOW) and `issue-43-native-preparation-integration-design.md` (the parent HOW).
   - It is not a complete specification, a change to the master design or a claim that anything is implemented.
   - Source baseline is `76a58b6e3dfa9cedb7e528296fb59d2fa4725666`. The design tree is `302cabb7`, which adds only this document.
2. It supersedes ONLY these two narrowings, and only far enough to admit the finite `native_phase_git` actions in §3 through the SAME private owned-proof/complete-inventory-CAS route:
   - version HOW §1, "narrowed ONLY for `native_phase_version`";
   - parent HOW §6, "narrows ONLY the `native_phase_version` effect-table exact-permit seam".
   All their other sentences stay in force, including "adds no DDL or permission exemption for input, owner, Session, readiness, Workflow, Driver, Source, quota or any other effect".
3. Unchanged:
   - protected owner/readiness/input/Session/quota permissions;
   - the ordinary validators (`validate_authority`, `driver::validate`, `source_recovery::validate_task`);
   - installed/static admission and the global composition issuer;
   - `SCHEMA_VERSION` 10, the layout fingerprint, trigger/permit slices and the writer-contract catalogue;
   - Task/Workflow/Context/Driver/Source versions;
   - the generic `reserve_execution_helper*` allowlist (`git_helper|native_version|docker_probe`).

   Git is a distinct effect kind and is not disguised as `native_phase_version`. This HOW adds no public selector, authority mode, permission callback or generic marked-scope bypass. It does not override or change any Git config, hook or user setting.
4. None of these authorize this seam: ordinary `git_helper`, the generic allowlist and decoder, or an unprotected `managed_effects` table. Only the actual selected Native start may issue these intents, and only while it consumes the SAME genuine original Source seal (§2.2) and the SAME actor's known settled helper history (§8).
5. Scope is only today's first-Executor selected start (`native_handoff.rs:327–351`: Executor, Preparing, generation 1, no artifact). Reviewer snapshot/artifact qualification and its lease remain later work. A reviewer Unit refuses before any Git intent.
6. The start STILL refuses. The bail at `execution/native/preparation.rs:369–371` stays. Its text may be extended, but it adds no quota, Session, transport, input or prepared-input proof. Native is not enabled. The full MVP, including four-Task independence, remains open.
7. Retained draft "draft2b" is outside this tree.
   - It is not approved source. Its effect kind, action vocabulary and nongrant completion differ from this HOW.
   - It must NOT be enabled unchanged after this design is approved. Its source must be re-derived to this HOW and independently reviewed.
   - Until then, the actual start stays guarded: it captures the original version and refuses before any marked Git intent or spawn.
8. This revision makes these corrections:
   - **R1 (mandatory).** Effective content conversion and filters are qualified BEFORE any convert-capable command (§5). A NEW Git action is added, so the private vocabulary grows from 12 to 13 Git actions. Physical correspondence is now defined precisely, and one consumer enforces it (§6).
   - **"Existing pure ref-format validator" does not exist.** The `git.rs:114,430` checks are child `git check-ref-format` processes. That operand is removed: SourceRoots now uses the pinned original revision, so no ref operand, ref validator or mutable-ref authority remains (§3.3).
   - **Draft2b status** is corrected (§1.7).
   - **Known-ack continuity.** A closed version or Git prefix is never replay-reconciled against later inventory (§8.5).
   - **Aggregate capture.** The previous 12-action formula is 2,818,048 B, not 2,883,584 B. All bounds are recomputed for 13 actions (§12).
   - **Closure.** Same-owner revocation closure is supported. Cross-epoch/restart closure is Held, and the shared snapshot is not weakened (§10).

## 2. Exact private producer lineage and original Source seal

1. Unchanged lineage:
   - `runtime/phase_handoffs.rs:246–319` `Consumer::transfer` builds `PhasePreparationOrigin { source, allocation, ticket: Weak }`.
   - `runtime/phase_supervisor.rs:910–917` builds the only `PhaseLaunchParts`, and `validate_preparation_origin_tx` (`:186–194`) conjoins origin, the current Workflow successor and Driver liveness.
   - `phase_jobs.rs:233–256` → `adapter/native.rs:100–126` → `execution/native.rs:199–214` → `begin_phase_preparation` (`native/preparation.rs:286–372`) commits allocated1→preparing2 under `PhaseEffectAdmission::enter`.
   - `prepare_phase_version` (`native/version.rs:288–359`) runs the version helper through to `close_phase_version_observation` (`version/closure.rs:385–413`).
2. **Source seal.** The authority is the SAME objects, not equal rows. A private `NativeGitSourceSeal` is built once, outside all locks, at the start of the Git stage. It retains:
   - the SAME `Arc<PhaseLaunchParts>`, and through it the SAME `PhasePreparationOrigin`, the SAME `SourceNativeCustody`, and its `operation`, `pair`, `session` and `invocation`;
   - the SAME origin `NativeAllocation`, whose `ManagedInput` is the full original encoded input;
   - the SAME `Arc<Frame>`, which must satisfy `Arc::ptr_eq(custody.frame, …)` and `frame.revision == unit.base_sha`;
   - two summaries (summaries only, never authority):
     - `input_digest = sha256(canonical encoding of that SAME ManagedInput)`;
     - `inventory_digest = frame.versions["context:committed_inventory"]`.

   The seal also requires `allocation input.revision == frame.revision`. No consumer accepts a Frame, inventory or digest that was re-read, decoded or row-derived in place of these pointers.
3. NEW `prepare_phase_git(custody)` runs inline at `native/preparation.rs:366–369`, after `prepare_phase_version` returns and before the kept bail. It runs on the SAME custody, actor, launch and admission object. It is not a callback, flag or re-entry.
4. NEW crate-private, borrow-only accessors, owned by A/Root and coordinated before editing:
   - `SourceNativeCustody::original_frame(&self) -> &Arc<Frame>`;
   - `Frame::native_git_expectation(&self)`. It returns the revision, OID length, `versions`, and a borrow of the existing `CommittedIndex::inventory()` (`context/committed.rs:235`) of `{oid, sha256, bytes, skipped}`.
   - `read_corpus` (`workflow_source.rs:1182`) gets a named constant `UNSUPPORTED_ENTRY` for its existing skip reason. The accessor uses that same constant to classify non-ordinary entries, so no free-string match is needed.

   None of these accessors constructs a Frame, custody, seal or grant. None is reachable from SQL, DTOs, IDs or `FakeAgent`.

## 3. Finite readonly Git action set (13)

1. The private `PhaseGitAction` enum has exactly 13 variants.
   - Argv comes only from retained marker parent bodies (`project.root`, `project.base_branch` for in-process comparison only, `task.worktree`), the seal revision, and fixed constants.
   - Every variant uses the unchanged `results::git_command_for` (`results.rs:640–664`), the unchanged `ResourceProfile::environment` overlay (`resources.rs:36–64`) and the canonical absolute `real_tools["git"]`.
   - There is no shell, no stdin, and no caller-supplied argv, target or purpose.

| # | Action | Root | Arguments after prefix | Cap | Qualified expectation |
|---|---|---|---|---|---|
| 1 | SourceTop | project | `rev-parse --show-toplevel` | 64 KiB | one line |
| 2 | SourceGitDir | project | `rev-parse --path-format=absolute --git-dir` | 64 KiB | one line |
| 3 | SourceCommon | project | `rev-parse --path-format=absolute --git-common-dir` | 64 KiB | one line |
| 4 | SourceRoots | project | `rev-list --max-parents=0 <R>` | 64 KiB | ≥1 OID lines |
| 5 | TaskTop | worktree | `rev-parse --show-toplevel` | 64 KiB | one line |
| 6 | TaskCommon | worktree | `rev-parse --path-format=absolute --git-common-dir` | 64 KiB | one line |
| 7 | Branch | worktree | `symbolic-ref --quiet --short HEAD` | 64 KiB | one line |
| 8 | Head | worktree | `rev-parse --verify HEAD^{commit}` | 64 KiB | equals R |
| 9 | Config | worktree | `config --null --get-regexp ^(core\.(bare\|worktree\|sparsecheckout\|autocrlf\|filemode)\|index\.sparse\|extensions\.objectformat)$` | 64 KiB | §5.2 |
| 10 | IndexEntries | worktree | `ls-files -z -s -t -v` | 1 MiB | §6.2 |
| 11 | IndexTree | worktree | `diff-index --cached --quiet --no-ext-diff --no-textconv --ignore-submodules=none <R> --` | 64 KiB | exit 0 |
| 12 | ConversionAttrs | worktree | `ls-files -z --cached -- ':(attr:!text !eol !crlf !ident !filter !working-tree-encoding)' ':(attr:-text !eol !crlf !ident !filter !working-tree-encoding)'` | 1 MiB | §5.3 |
| 13 | Status | worktree | `-c core.untrackedCache=false status --porcelain=v1 -z --untracked-files=all --ignored=matching --ignore-submodules=none` | 1 MiB | exit 0, empty stdout, §6 |

`R` is the seal revision.

2. After action 8, actions 1–8 feed the unchanged pure `git::validate_worktree_ownership` (`git.rs:42–94`).
   - It runs in process, on the retained marker Project/Task bodies, never on current rows.
   - Its root comparison is exact equality, sorted, against the registered `repository_identity`.
   - A mismatch refuses before the action 9 intent.
3. **SourceRoots uses the pinned revision.** The only operands in the whole set are the exact lowercase-hex `R` (`model::valid_oid`, so it cannot start with `-`) and the fixed pathspec constants after `--`.
   - `refs/heads/<base_branch>` is no longer an operand. No ref-format check, child `check-ref-format` or mutable ref contributes authority.
   - This is valid because the registered identity is the root set of the registered base history, and `R` is the Unit base from that history. The predicate becomes "R's object history in this common dir has exactly the registered roots". That is a stronger binding to the seal.
   - If the base branch later merged an unrelated root history, the result is an honest refusal. Ordinary `project_root` already refuses that case.
   - `protect_branch` still compares the in-process branch string to `project.base_branch`.
4. Order is fixed (1→13) and fail-fast.
   - Each in-process consumer runs before the next intent is planned: ownership after 8, Config after 9, entries after 10, conversion after 12.
   - The first mismatch, parse failure, incomplete capture or refusal settles that action and stops the batch. No later intent is issued, and the start refuses.
   - Any exit code the action allows is settled Confirmed and then interpreted by its own allowlisted consumer. "Settled" never means "prepared".
5. Effects claim, corrected:
   - No action requests a network operation, a fetch or a repository write (refs/objects).
   - `GIT_OPTIONAL_LOCKS=0` disables the opportunistic index refresh write.
   - Actions 1–12 do not read worktree file content through conversion.
   - Action 13 is the only convert-capable action. It runs only after §5 proves that conversion is the identity, which removes the clean/process-filter child path.
   - This HOW does NOT claim that inherited system/global Git configuration launches no other process or writes nothing outside the repository (for example, trace2 targets). Those are preserved and inherited, and they are not qualified here.
   - `HOME` is not substituted. rrx does not read, copy or serialize credentials.
   - Action 9 prints only seven fixed non-credential keys. No `filter.*`, `credential.*`, `include*`, `url.*` or other value is requested.

## 4. In-memory Frame qualification before any Git intent

1. The retained Frame is authoritative for committed content: `revision`, the `code`/`rules:config`/`rules:*`/`context:committed_inventory` versions and the per-path `{oid, sha256, bytes, skipped}` inventory. These are content-addressed by `R`. This stage does not re-run `ls-tree`/`cat-file`.
2. Outside all locks, with no Git action and no file read, all of the following must hold before the action 1 intent. A missing entry or mismatch refuses with zero Git rows.
   - Every `rules:*` path exists in the inventory, is not skipped, and has a matching sha256.
   - `rules:config` equals the retained digest.
   - `code == R`.
   - The OID length matches `R`.
   - No inventory entry is classified `UNSUPPORTED_ENTRY`, i.e. there are no symlinks (120000) or gitlinks (160000).
   - Inventory size is within the Frame's existing bounds (≤4096 entries).
3. Baseline `qualified_content` (`results.rs:777–819`, run at `attempts.rs:538` before the seal) rejects only symlinks, submodules and LFS. It does not qualify physical correspondence, and this stage does not rely on it. Its LFS-pointer semantics stay with that baseline and are not re-claimed here.

## 5. Bounded effective conversion/filter qualification (before Status)

1. **Threat** (primary sources, verified by Sol and Root at Git v2.51.0):
   - `convert.c` `apply_filter` (CAP_CLEAN) and `filter_buffer_or_fd` run `child_process` with `use_shell=1` via `start_command`.
   - `object-file.c` `index_mem` calls `convert_to_git`.
   - `read-cache.c` worktree comparison hashes through that path.
   - `gitattributes(5)`: effective attributes come from worktree/index `.gitattributes`, `$GIT_DIR/info/attributes`, the global and system files, and attr-source settings. clean/process filters and `text`/`eol`/`crlf`/`ident`/`working-tree-encoding` transform bytes on check-in.

   So an ordinary non-LFS filter can run shell-backed processes and normalize changed physical bytes to the original blob. HEAD, index, `H` flags and an empty Status would then all pass.
2. **Config predicate (action 9).**
   - Parse `--null` records `key\nvalue\0`, or `key\0` for a valueless (true) key.
   - Booleans use a closed case-insensitive set: true = `true|yes|on|1` or valueless; false = `false|no|off|0|` (empty). Any other spelling is unparsed and refuses.
   - Every occurrence of a key (including multi-valued keys) must satisfy:

     | Key | Required |
     |---|---|
     | `core.bare` | false, or absent |
     | `core.worktree` | absent |
     | `core.sparsecheckout` | false, or absent |
     | `index.sparse` | false, or absent |
     | `extensions.objectformat` | absent ⇒ OID length 40; `sha1` ⇒ 40; `sha256` ⇒ 64 |
     | `core.autocrlf` | false, or absent (`true`/`input` refuse) |
     | `core.filemode` | true, or absent |

   - Exit 1 (no keys) means all keys are absent.
   - Raw values are never persisted. The receipt carries only the stdout hash and the expectation.
3. **Effective attribute predicate (action 12).**
   - This uses Git's own attribute resolver in the check-in direction, the same one `convert_to_git` uses. It therefore covers every effective source: worktree `.gitattributes` (tracked or untracked), the index fallback, `info/attributes`, `core.attributesFile`/XDG global, the system file, macros (`binary`, user `[attr]`), and `attr.tree`/`GIT_ATTR_SOURCE`.
   - The two OR'ed positive pathspecs match a tracked path only if `filter`, `ident`, `eol`, `crlf` and `working-tree-encoding` are all unspecified, AND `text` is unspecified or unset (`-text`, `binary`).
   - **Expectation:** exit 0, and the NUL-separated output, with no duplicates, equals EXACTLY the inventory path set (byte equality).
   - Any path that has a filter, a text set/auto/value, eol, crlf in any state, ident or working-tree-encoding is missing from the output, so the batch refuses before Status.
   - No `filter.*` config is read: with no path naming a driver, `convert.c` performs no driver lookup and starts no child.
   - Together with `core.autocrlf` false or absent, `convert_to_git` is the identity for every tracked path.
4. **Fail-closed analysis.**
   - Missing pathspec magic (old Git) or a literal-pathspec environment (`GIT_LITERAL_PATHSPECS`) makes Git error or match nothing. The set mismatch refuses.
   - A silently ignored attr requirement is not credited as safe. The mandatory controls in §14 (filter/text/eol/crlf/ident/encoding/macro/untracked/info/global) must show refusal on the pinned qualified Git.
   - Pathspec `attr` semantics (gitglossary "pathspec", `attr:`) are cited here but were NOT primary-source verified in this revision. That verification, against v2.51.0 `pathspec.c`/`dir.c`/`attr.c`, is a required source-review step.
   - The `ls-files -s -t` record format (`show_ce`) must be verified at the same step.
5. Setup and race limits:
   - The attribute sources and config can change between action 12 and action 13. That is the existing cooperative race limit and is not tamper resistance.
   - Refusal on unsupported attributes or config is the intended product behavior. rrx never overrides attributes, config, hooks or `HOME` to make a repository pass.

## 6. Precise physical correspondence and its enforcing consumer

1. Let `S` be the seal (§2.2), `R` its revision and `I` its inventory, in which every entry is an ordinary blob (§4.2).
2. **Index binding (action 10).** Each NUL record `<tag> <mode> <oid> <stage>\t<path>` must have:
   - tag `H` (not `h`, `S`, `M` or any other tag);
   - stage 0;
   - mode `100644` or `100755`.

   The exact set `{(path, oid)}` must equal `{(p, I[p].oid)}`, with no extra or missing path. This binds the physical index directly to the retained OIDs.
   - Under genuine provenance this is redundant with 11 + 8, so no independent mutant kill is claimed for it. It catches a Frame/`read_corpus` defect.
   - Action 11 additionally proves that the index tree equals `tree(R)`, including the mode of each entry.
3. **Conversion identity** (§5.2–5.3) holds for every `p ∈ I`.
4. **Status (action 13)** must exit 0 with empty stdout. Git then has, for every index entry, either:
   - found the stat data unchanged (stat-cache; no re-hash), or
   - hashed the raw worktree bytes through identity conversion to the index OID, with mode and typechange agreeing.

   Status also proves there are no untracked, ignored or unmerged paths. `--ignore-submodules=none` recurses into nothing, because the index has no gitlinks.
5. **Correspondence statement.**
   - For every `p ∈ I`, the worktree at the TaskTop path holds a regular non-symlink file whose raw bytes `B_p` satisfy:
     - `GitBlobOID(B_p) == I[p].oid`;
     - for non-skipped entries, `sha256(B_p) == I[p].sha256` and `len(B_p) == I[p].bytes`, by content addressing of the retained blob;
     - the exec bit matches the tree mode.
   - No non-empty path exists outside `I` apart from Git metadata.
   - Exceptions, which are explicit cooperative limits: entries Git skipped by stat-cache; the race after the last check (including attribute/config changes between 12 and 13); same-user tampering; and path-name folding under `core.ignorecase`/`core.precomposeunicode`. Empty directories are not observed.
   - rrx does not re-hash files itself.
6. **Enforcing consumer.** The private `NativeGitPhysicalCorrespondence` can be minted only by the action-13 qualifier, `qualify_status`. That qualifier requires the SAME batch's:
   - qualified ownership value;
   - `ConfigQualified`;
   - `IndexMatchesSeal`;
   - IndexTree confirmation;
   - `ConversionIdentity`.

   Each of these is an owned value produced only by its action's parser from that batch's retained observation, and each must be `Arc::ptr_eq` to the SAME seal.

   The qualifier's own rules:
   - Status is not planned at all without `ConversionIdentity`. The intent planner for action 13 takes it as a required argument.
   - The minted value retains the SAME seal and the ordered acks.
   - It is stored in the preparation custody `git` slot. It is a nongrant fact, never prepared input, a permission or NoCurrentDispatch.
   - Today's start still bails after storing it. Any later prepared-input port must take this SAME value as a mandatory conjunct. Rows, receipts or a fresh Status cannot construct it.

## 7. Effect tuple and nine-column writes

| | Old generic tuple (`UnitGit::run_command`, unchanged) | New private tuple |
|---|---|---|
| kind | `git_helper` | `native_phase_git` |
| idempotency_key | `helper-{id}` | `native-git-{id}` (≤256 B) |
| expected_target | absolute root path | `git:{action}:{sha256(operation:pair:epoch:profile:R:inventory_digest:input_digest:action)}` (≤4096 B) |
| producer | `reserve_execution_helper_pinned` + `HelperGuard` | private `reserve_phase_git_intent` only; never `HelperGuard` |

1. The intent INSERT writes the existing nine columns. The body is exactly the nine-field `ManagedEffect` (`execution/model.rs:281–292`, `deny_unknown_fields`) with an empty receipt, ≤8 KiB, and no raw environment, output, path, config value or error.
2. Settlement UPDATE sets all nine columns `WHERE` all nine `IS` the saved preimage. Only these change:
   - `state`: pending→confirmed|unknown;
   - `body`: state, receipt and version;
   - `version`: 1→2, with a checked add.
3. Receipt: ≤16 entries; keys ≤64 B; values ≤256 B; no control characters. Fields are `action, creation, capture, exit, bytes, stdout_sha256, expectation (match|mismatch|unparsed), hygiene`. Raw stdout/stderr, paths and config values are never persisted.
4. Persisted rows are bookkeeping, never authority.

## 8. Same original qualified history, complete inventory CAS and known-ack continuity

1. `close_phase_version_observation` returns a private `NativeVersionSettledCommit { settlement: Arc<NativeHelperSettlementPlan> }`. It is issued only after its Immediate observed `current == settlement.after` and committed. The custody retains it.
2. The Git stage starts only if:
   - that SAME ack exists;
   - its effect is `Confirmed`;
   - its SAME owned observation has `qualified_profile().is_some()`.

   A missing ack, Unknown or unsupported version, or a re-planned or row-derived settlement refuses before any Git intent.
3. **Chain.** `before₁` is the SAME `settlement.after` object, and `beforeₖ₊₁` is action k's SAME settled `afterₖ`. No stage reads current rows to refresh history. A coherent planning snapshot that differs from the retained expected set is Held.
4. Every intent Immediate conjoins, in one transaction under one `InventoryBudget`:
   - `admission.validate_for(launch)`, from a NEW guard on the SAME `PhaseEffectAdmission`;
   - `actor.validate_open()`;
   - `NativePreparationCommit::validate_version_ready`, i.e. `validate_common` plus the pair readiness postimage;
   - an explicit read-only `runtime_epoch.epoch == facts.epoch`;
   - `Inventory::read == beforeₖ`;
   - absence of the planned id/idempotency;
   - `INSERT == 1`.

   The 50 ms fence repeats the actor, readiness, epoch and `pendingₖ` conjunction. Settlement accepts `current == afterₖ` (idempotent) or `current == pendingₖ`, followed by the exact nine-column CAS. Anything else is Held.
5. **Known-ack continuity.** Each settled action yields `NativeGitSettledCommit` (k). The custody keeps an append-only `NativeGitHistory { version: Arc<NativeVersionSettledCommit>, acks: Vec<Arc<NativeGitSettledCommit>> }`, where each `beforeₖ` is `ptr_eq` to the previous `after`.
   - Once an ack exists, that helper's `reconcile` is terminal and memoized. It returns the SAME ack without SQL and never re-plans closure or re-reads inventory. This applies to the version helper's `reconcile` (`version.rs:165–178`) and to each Git action.
   - Only the current tail action can be planned, settled or closed against current rows.
   - Later rows, including later Git rows and any future registration/transport rows, therefore never invalidate the closed prefix. A replay of the version or an earlier Git prefix against the later inventory is forbidden; the comparison would be false by construction.
   - Future consumers validate `current == final after` or their own chain from it.
6. **Row headroom.** `before₁ ≤ 242` rows, so the postimage after action 13 is ≤255 rows, keeping the reserved 256th native-input slot.
   - The version stage's own baseline limit of ≤254 is unchanged. A version baseline of 242–254 qualifies the version stage, but this stage then refuses honestly before any Git intent.
   - Later registration/transport intents get no headroom from this stage.

## 9. Raw child, capture and observation before parsing or SQL

1. For each action:
   - plan outside locks;
   - retain helper custody;
   - build the eager `CaptureOwner` before spawn or first poll;
   - take a NEW admission guard;
   - run the intent Immediate (known commit);
   - set `attempted = true`;
   - call `Command::spawn` once;
   - immediately `RetainedRawProcess::adopt(child)`.

   This is the same pattern as `native/version.rs:302–346`. A spawn `Err` is attempted-without-handle, never NoChild, and is never retried. Admission is dropped before any capture await.
2. Capture retains a bounded `NativeGitObservation` (stdout up to the cap, discarded stderr counted against the combined cap, exit, completeness, hygiene) in helper custody BEFORE any parsing or Store access. A persistence error cannot discard it.
3. These apply unchanged:
   - process group;
   - null stdin;
   - 50 ms `MissedTickBehavior::Delay` fence;
   - ≤30 s action deadline;
   - ≤2 s drain;
   - ≤10 s best-effort stop/reap;
   - the leader stays unreaped until group signaling, and descendant death is never inferred.
4. `owner.git_lease(unit, None)` is held outside Store across the whole batch, with `RRX_GIT_GATE_TOKEN` as in `git_io.rs:150–154`. It is process-local serialization only.

## 10. Factual nongrant closure: same-owner only, cross-epoch Held

1. Each action's closure mirrors the version closure:
   - SAME settlement and observation;
   - `actor.validate_original`;
   - `launch.validate_preparation_original`;
   - the latest full Unit CAS (including `owner_epoch`, `closure.rs:7,129`);
   - inventory equal to `pendingₖ` or `afterₖ`.

   The closure Immediate omits only the current/Driver/admission/new-intent conjunctions, so that a revoked actor can record work it actually did.
2. **Closure planning reuses the shared `snapshot`** (`state/managed_binding/snapshot.rs:29–58`). That snapshot requires `runtime_epoch` instance and epoch to equal the SAME `RuntimeOwner`.
   - Supported: same-owner revocation closure, within the SAME Runtime owner and epoch, after actor revocation, Driver stop, admission closure or a successor change.
   - NOT supported: cross-epoch or restart closure. After a process exit or owner retirement, the in-memory plan, observation and acks are gone, and startup `fence_epoch_effects` turns pending into unknown. The action stays Held under the Driver Task-wide pending/unknown gates (`driver/preparation.rs:349,546`).
   - The shared snapshot, ordinary permissions and validators are NOT weakened. No universal epoch-free closure planning is claimed.
3. Closure issues no permission, intent, retry, prepared fact or NoCurrentDispatch. It cannot turn Unknown into success or reopen the actor.

## 11. Interference gives Held, not DB fencing

Each of the following writers or changes is detected by the private consumer. The observation is retained, nothing is replayed, no later intent is issued and the result is Held:
- generic `reconcile_managed_effect(_pinned)` (`effects.rs:129–171`);
- epoch `fence_epoch_effects` (`effects.rs:3–28`);
- an old10 cached or newly opened writer;
- delete, replace or body/index drift.

Existing10 has no schema fence against these writers, and this HOW claims none.

## 12. Bounds and their consumers

| Bound | Value | Consumer |
|---|---|---|
| Git intents per original operation | 13 (14 with the version helper) ≤ parent 32 | `PhaseGitAction` plan |
| Capture per action | 64 KiB (1–9, 11); 1 MiB (10, 12, 13) | capture accounting |
| Aggregate preparation capture | version 64 KiB + 10×64 KiB + 3×1 MiB = 11×65,536 + 3×1,048,576 = 720,896 + 3,145,728 = **3,866,624 B** ≤ parent 8 MiB | batch custody |
| Deadlines | 30 s/action, 2 s drain, 10 s stop/reap; 180 s batch from the first Git intent (binding cap) | capture task / batch custody |
| Inventory rows | before₁ ≤242; each postimage ≤255; read sentinel at 257 | `Inventory::with_git_intent`, `read` |
| Inventory bytes | body ≤8 KiB; whole charged set ≤2 MiB, re-checked per action; planning charges 13 maximal rows | plan, intent, settlement |
| SQLite VM work | existing `InventoryBudget` (1000-op interval, interrupt at 1000 callbacks) per routine | every snapshot/Immediate |
| Receipt | ≤16 entries, ≤64/256 B | `effects.rs:137–143` reader |
| Inherited Git config overlay | ≤128 entries + 3 | `ResourceProfile::environment` |
| Config keys queried | exactly 7 fixed keys | action 9 |
| Frame inventory | ≤4096 entries (existing) | §4, §6 |

For reference, the earlier 12-action formula was 64 KiB + 10×64 KiB + 2×1 MiB = 2,818,048 B, not 2,883,584 B.

Overflow, timeout or a VM interrupt gives an incomplete observation or Held. It never qualifies an action.

## 13. Impact analysis

| Affected | Effect | Handling |
|---|---|---|
| `native/preparation.rs:366–371` | new inline stage before the bail | bail kept |
| `version/closure.rs:385–413` (sole consumer `native/version.rs:174–178`) | returns a private ack | reconcile memoized after the ack; no other caller |
| `NativePreparationCustody` | adds `git` slot, history, correspondence, abandon, Drop | same eager-guard rules |
| `native_handoff.rs`, `workflow_source.rs` Frame, `read_corpus` skip constant | borrow-only accessors and named constant | coordinated with A/Root; skip text unchanged, so no digest change |
| `context/committed.rs:235` `inventory()` | reused read-only | none |
| `git.rs` (`validate_worktree_ownership`, `repository_identity`) | predicate reused; no `check-ref-format` reliance | unchanged |
| `results.rs` `git_command_for`, `qualified_content` | reused unchanged / not relied on | shared consumers unaffected |
| `effects.rs` allowlist, `reserve_effect_tx` | unchanged | `native_phase_git` not added |
| Driver pending/unknown gates | block on native git pending/unknown, including cross-epoch | intended |
| `driver/executor.rs:210–216`, `state/execution.rs:685–690` | non-`git_helper` refuses | runs before the marker; no regression |
| `managed_binding/snapshot.rs` | reused unchanged | no epoch weakening |
| Schema10, permits, Task/Workflow/Context/Driver/Source/Session/owner/readiness/quota rows | unchanged | no permission change |

Legacy `execution/native.rs:303–319` stays unreachable. Command-only verifier and bootstrap Units refuse first.

## 14. Verification controls and their limits

1. Positive controls use the actual Goal/Driver/Source/allocation/marker/launch/job/actor chain. Rows, DTOs, `FakeAgent`, SQL-seeded actors and static issuers do not count.
   - A setup refusal is recorded as such. It is never a positive or a mutant kill.
   - Test fixtures may use a hermetic `HOME`/`XDG_CONFIG_HOME`/`GIT_CONFIG_GLOBAL` for global attributes. That is test isolation, not a product override.
2. **Ordinary positives.** In each case the correspondence is minted and the start still bails, with Task/Workflow versions unchanged:
   - a clean tree with no attributes;
   - `*.bin binary`;
   - `*.txt -text`;
   - an executable file;
   - a file over 256 KiB (OID-only correspondence).
3. **R1 negatives.** Each must refuse at action 12 or 9, with no action-13 intent and no Status spawn. A canary marker written by any filter must be absent.

   **Non-LFS filters:**
   - `* filter=canary` with a repo-local `filter.canary.clean` that writes a marker and normalizes modified worktree bytes back to the original blob;
   - the same through a long-running `filter.canary.process`.

   **Where the attribute comes from:**
   - committed `.gitattributes`;
   - an untracked worktree `.gitattributes`;
   - `info/attributes`;
   - a hermetic global attributes file;
   - a user macro `[attr]m filter=canary`.

   **Content conversions:**
   - `text=auto` with injected CRLF;
   - `eol=crlf`;
   - `crlf`/`-crlf`;
   - `ident` with an expanded `$Id$`;
   - `working-tree-encoding=UTF-16`.

   **Config:**
   - `core.autocrlf=true`/`input` with CRLF injected under unspecified `text`;
   - `core.filemode=false` with a chmod drift.

   **Mutant kills.** Each compiled omission must fail its intended assertion:
   - drop the `ConversionIdentity` requirement on Status planning: the canary runs and the correspondence is minted;
   - drop any single attribute term (one control per term);
   - drop the autocrlf predicate;
   - drop the filemode predicate.
4. **Factual corrections:**
   - SourceRoots on `R`: retargeting or deleting `refs/heads/<base>` after the seal does not change the outcome. A mutant using the ref changes it. A revision whose roots differ refuses.
   - Version reconcile after Git rows exist returns the SAME ack without SQL. A mutant that re-reads inventory errors.
   - Same-owner revocation mid-action: closure records facts.
   - Cross-epoch restart: no closure; Held under the Driver gate.
5. **Retained controls:**
   - a missing or unqualified version ack;
   - a fresh-read `before`;
   - drift at 242/243 rows and at 2 MiB;
   - each dropped epoch/Source/Driver/pair/readiness conjunct;
   - spawn before intent; a retried spawn; an error between spawn and adopt; an unpolled capture Drop;
   - HEAD, untracked/ignored files, skip-worktree, assume-unchanged, index entry/tree, sparse, bare or object-format mismatch;
   - output at the 64 KiB and 1 MiB boundaries (including action 12);
   - action and batch timeouts;
   - generic reconcile and old10 cached-writer interference;
   - another Task continuing.
6. **What primitive tests cannot prove:**
   - producer causality;
   - intent-before-spawn under real races;
   - retention across Drop;
   - compiled old10 behavior;
   - process-group and descendant death;
   - per-version Git semantics beyond the pinned qualified Git;
   - stat-cache races;
   - same-user tamper resistance.

   The in-memory non-ordinary-entry check and the per-path index↔seal check are primitive-only. Upstream baseline refusals make them unreachable through a genuine chain, so no mutant kill is claimed for them.

## 15. Acceptance condition and unresolved gates

**Acceptance (one line):** the real selected start, holding only the SAME qualified known-settled version history and the SAME original operation/pair/input/Frame seal, runs at most the 13 fixed readonly Git actions. Each is committed intent-before-one-shot-spawn under the full current/Source/Driver/Unit/pair/epoch conjunction and the exact complete-inventory CAS. Status runs only after effective conversion is proven to be the identity. The start retains every observation and the §6 correspondence, and still refuses, with no schema, guard, permission, config/hook or grant change.

1. Unresolved and out of scope:
   - primary-source verification of pathspec `attr` and the `ls-files -s -t` format at v2.51.0 (§5.4);
   - reviewer snapshot/artifact lease;
   - hooks/settings qualification;
   - quota;
   - Native transport/registration/input/ACK;
   - the installed composition issuer and `cargo install` usability;
   - full regression and Clippy (RED);
   - macOS/Linux real-OS qualification;
   - four-Task independence;
   - cross-epoch closure;
   - rewriting draft2b to this HOW;
   - the Sol delta review of this revision, the later source review and the matching master delta.

   All existing source/native1/native4/realOS/hooks/quota/reviewer/full-regression/Clippy gates remain.
2. The same non-dispatch limits apply: no NoCurrentDispatch, best-effort process cleanup, and cooperative work, not a sandbox or a process-death guarantee. Full Prepared remains separate and absent.
