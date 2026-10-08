# Issue 43: FM implementation delta R1 — Native handoff digest defect, D2 lane, managed_* class

- **Status:** R1, approved by Sol (6051372522). §1 and §2 are committed together with the `committed_source_tests` reclassification; §3 is approved and not yet applied.
- **Base:** `777c132` (FM L in-crate, L integration and goal_graph R are committed).
- **Why now:** the first D2 analogs of `workflow::committed_source_tests` (FM HOW R3 §9) fail on the installed lane for a reason in production code, not in the fixture.
- **Constraints (STRICT, unchanged):**
  - no relaxed authority, CAS, owner, marker or #19 check;
  - no test-only authority constructor;
  - no SQL-seeded positive;
  - no disabled or ignored test;
  - no Task.version binding.

## 1. NH-1: the Native handoff compares two digest formats (production defect)

### 1.1 Facts (at `777c132`)

- `CommittedIndex::build` (`context/committed.rs:164`) records every inventory entry's `sha256` with `context::hash` (`context.rs:979`), which returns `sha256:<hex>`.
- `SourceNativePreparationSeal::qualify_git` (`execution/workflow_source/native_handoff.rs`) compares those entries with bare hex at four sites:

  | Line | Check | Bare-hex side |
  | --- | --- | --- |
  | `:141` | "original rule digest differs" | `frame.versions["rules:<path>"]`, from `workflow_source::digest` (`workflow_source.rs:25`, `:1182`) |
  | `:153` | "original config digest differs" | `frame.versions["rules:config"]` (`workflow_source.rs:1174`) |
  | `:178` | "original mandatory bytes/OID correspondence differs" | `digest(original_bytes)` |
  | `:219` | "original Source bytes/hash/size differs" (`qualify_git_corpus`) | `digest(bytes)` |

- These checks can therefore never pass for a Project with **any committed entry that has materialized bytes**:
  - with `rule_refs` or `config_ref`, the rule or config check fails first;
  - otherwise every inventory file fails in `qualify_git_corpus`.
- **Why nothing caught it:**
  - every installed control (SC/BR/EF/DC) registers `register_real_git_project`, whose repository holds only an empty commit, so the inventory is empty;
  - the only corpus unit test (`seal_identity_tests::nongrant_original_corpus_checks_*`) builds a synthetic `InventoryEntry` with bare hex instead of the producer's format.
- **Observed:** the first D2 analog — the SC1 lane plus committed `workflow.toml`/`rules.md` — never binds. Its job reports `refusal: "original rule digest differs"`, `attention: "original preparation Held"`. This happens with and without live rules B.

### 1.2 Proposed fix

One private helper in `native_handoff.rs`, `inventory_sha256(hex) -> "sha256:<hex>"`, gives the bare-hex side the inventory's own format at all four sites.
- Every comparison stays exact equality.
- The skip and absence conditions are unchanged.
- No new authority, input or check is added or removed.
- The two existing unit tests use the producer's format for their synthetic entries.

Rejected alternatives:
- **Strip the prefix from the inventory:** it is part of `context:committed_inventory`'s digest input, so stripping would change persisted versions.
- **Change `context::hash`:** it would change every Context `source_hashes` value.

### 1.3 Controls (run locally on the patch, non-root, CI-like)

| Control | Result |
| --- | --- |
| New `native_handoff::seal_identity_tests::producer_inventory_format_qualifies_its_own_bytes` | Builds the inventory with the real `CommittedIndex::build`. The same bytes qualify; different bytes are refused. Against unfixed `777c132` it **fails** with "original Source bytes/hash/size differs" (causal). |
| Existing corpus and seal unit tests | 2/2 pass. |
| §2's 9 D2 tests | 9/9 pass with the fix. Without it they never bind (the refusal above). |

Per-site revert mutants (run on the submitted source, non-root, then restored with an empty diff):

| Reverted site | Killed by |
| --- | --- |
| rule `:141` | `fm_d2_committed_source_631_codex_quick_front` ("normal Bound absent") |
| config `:153` | same |
| mandatory `:178` | same |
| corpus `:219` | `producer_inventory_format_qualifies_its_own_bytes` ("original Source bytes/hash/size differs") |

The negative half of the producer control (`b"abd"` refused) covers a changed inventory byte.

## 2. FM D2 lane (HOW R3 §9)

New module `runtime/installation/tests/fm_d2.rs`.
- **Fixture:** the installed protocol fixture in commit mode, plus a recording of the outgoing Native input.
- **Project:** a real Git Project with committed `config_ref` (class policy, `repo_map_tokens = 321`) and `rule_refs` (rule A), registered before `accept`. Live B (STRICT, 777, rule B) is written uncommitted before preparation.
- **Producers:** the installed issuer and the genuine Driver.
- **Shared helpers:** the existing `success.rs` helpers become `pub(super)` (test-only).

| Test | §9 D2 part |
| --- | --- |
| `fm_d2_committed_source_631_{claude,codex}_quick_front` | Committed policy over live B. Context A (revision = base, both hashes, budget, payload has A not B). Wire payload = Context data. One `native_input`. Same single Unit. Published, verified, Unit Success and closed. claude tail: S4-W refusal with Goal, artifact and input unchanged. codex tail: SC-N (deferred). |
| `fm_d2_committed_source_631_{claude,codex}_{standard,strict}_front` | Committed class. Requirements binds the same Unit. Context A. Wire payload = Context. Deferred: Requirements evidence integration. |
| `fm_d2_committed_source_301_{claude,codex}_gate_receipts` | 2 `managed_workflow_gate_v1` receipts (Worktree gate + Implement), each with a 64-hex context digest. They survive a reopen. A survivor worktree edit leaves the artifact verifiable. Deferred (SC-N): Commit, 3rd receipt, Tests wait. |
| `fm_d2_committed_source_470_corrupted_manifest_is_never_published` | Hold at `SETTLED_EVALUATION`, corrupt the captured artifact's manifest, release. Result: claim and observed, no closure, Implement not completed, nothing Published, Task unchanged. |

When §1 lands, the legacy tests these replace move to their §9 classes in the same commit:
- `:301` and `:631`: removed (their tail is deferred, or covered in D2);
- `:470` terminal: R (F2).

## 3. `tests/managed_tools.rs` / `tests/managed_docker.rs`: proposed D2 → L

- **Approved class:** HOW R3 §9 says D2 through `serve` plus the Driver, with the Git/Docker calls run inside the Native phase.
- **Fact:** their consumer is the legacy public preparation route `AttemptManager::prepare`. On legacy rows it runs exactly as the in-crate `phase_supervisor` L tests (§8.5) do.
- **L prototype (uncommitted):**
  - Project registration;
  - the Goal with both Tasks planned (the sibling moves into the plan, as group C says);
  - then the ordered historical migration;
  - `managed_tools` passes 1/1, with all its candidate-index, hook and owner-resource assertions unchanged.
- **D2 would need:** a configured compat agent that performs the commits inside the Native phase. The tests' assertions (the Unit's `ResourceProfile`, cookie and hook context) would also have to be re-derived from internal objects.
- **Proposal:** classify both as L. What L does not prove: accepted Source/marker/Native eligibility; that remains SC1/SC-N.
- **Approved** (6051372522). With it the §9 D2 count is 9 (the `fm_d2` tests), down from 11.

## 4. Flag already reported (`e593698`)

`runtime::phase_supervisor::tests::actual_publication_drop_and_shutdown_preserve_unknown_slot_until_proven_rollback`:
- Its `shutdown().unwrap()` predates `12304a9`, after which retained phase jobs keep shutdown pending.
- The test had not run past setup since then.
- It now asserts the reported "Native phase shutdown remains pending with retained jobs". Every later custody assertion is unchanged and passes.
