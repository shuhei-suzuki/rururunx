# Issue 43 FM delta R2: recovery_tests producer, and test-only seams left without callers — approved (B, R2-2)

Base: `fc03baf` (FM HOW R3, §8.3 and §9 of `issue-43-fixture-migration-design.md`; delta R1 approved in 6051372522). This delta adds no new class or guard. It answers two questions that the approved R3 rows leave open once they are implemented:

1. Which producer the `recovery_tests` RB rows run on.
2. What happens to `cfg(test)` seams in non-test files whose only callers become R tests.

Measured at `fc03baf`, non-root (fmtest, umask 022, subreaper), toolchain 1.91.1.

## 1. Facts

- **Q1.** All 7 `execution::workflow_source::recovery_tests` (13 scenarios) fail at the same point. `artifact_fixture` makes the legacy-Engine step into Implement (`recovery_tests.rs:122`), and that step is refused by F2 ("managed native binding and private admission are not composed"). Nothing after that point runs today.
- **Q2.** Prototype, not committed: the fm_d2 `committed("codex", Quick)` producer, run to `phase_closed` (Published). Then a real restart:
  - `Runtime::shutdown`;
  - drop the Runtime and the owner;
  - `RuntimeOwner::open` on the same file, which gives a new epoch;
  - `ManagedWorkflowSources::new(owner, the Runtime's config)`.

  After that, the `cfg(test)` port `recover_retained(task)` returns Ok, so the test passes (1/1). The #14 port therefore works on D2-produced retained state.
- **Q3.** Removing the R-converted modules' callers (an experiment with the three modules emptied, then `cargo clippy -p rrx --tests`) leaves these `cfg(test)` items in non-test files dead:
  - `recovery_tests`: `ManagedWorkflowSources::{recover_retained, recover_retained_inner}`, `ReconstructedFrame`, `RecoveryPause` and `RecoveryGuard` (`execution/workflow_source.rs:278-312, 672-760`); `SourceRecovery` and `Store::{begin,accept,abandon}_retained_source_recovery` (`state/execution/source_recovery.rs:55-87, 597-`).
  - `verification_tests`: `ManagedVerifier::{before_commands, before_completion}` (`execution/verification/mod.rs:240-246`).
  - `managed_tests`: nothing in a non-test file.

  `allow(dead_code)` is excluded (STRICT).

## 2. recovery_tests (13 scenarios): producer and RB

**Producer.** The D2 SC1 lane replaces the legacy `artifact_fixture`. It is the existing `fm_d2::committed` harness (accepted ingress, a real Git Project, `config_ref`/`rule_refs` committed before `accept`, the installed issuer, the genuine Driver, the configured protocol fixture), with one variant per row:
- **Published rows:** run to `phase_closed`.
- **Ready row:** hold at `SETTLED_EVALUATION` after capture, exactly as fm_d2 `:470`. The captured artifact is Ready and is not Published.

Then a real restart, as in Q2. There is no legacy `put_goal` (S4) and no legacy `put_task` sibling.

**R part (unchanged from §9).**
- **RB ×13.** On live Unix-peer ingress against a Runtime over the reopened owner, Pause is accepted. Resume then returns `Unavailable { FreshBootstrapRecoveryUnavailable }` (`state/runtime/goals.rs:437-445`). The preimage is the Goal, Task and Workflow rows, plus task_drivers and links unchanged (no job, no link).
- **S5-W ×4** (published, source_claim, final [no epoch], inflight [mode 0]). Their former drift producer, a legacy `put_task` title edit, is refused on accepted rows with "managed Task changes require typed owned control/Workflow transaction". The Task body/version and the audit are unchanged.

**Decision R2-1 (for review): the #14 port on these rows.**
- **(A) Recommended.** After the RB and S5-W assertions, each row keeps exercising the `cfg(test)` #14 port on the same D2-produced, restarted state. Kept: exact Published frame reopen, exclusive claim, future drop, bad manifest, caller DTO drift, foreign DTO, Ready never selected.
  - These are port mechanics only. They are **not counted** as D2, and they are not a recovery proof: the consumer is #14 (the §9 deferred tails stay at 13).
  - Two drifts move to legitimate producers (§8.2), because the generic Task write is now refused (S5-W):
    - final [no epoch] and inflight [mode 0]: a Project registry `rule_refs` change, which is in `governing_digest`;
    - source_claim: the DTO currency check uses the changed Project DTO.
  - The two epoch halves (final [epoch], inflight [mode 1]) need a mid-flight `begin_execution_epoch`, which §8.2 excludes. Their port half is retired (deferred #14), and each keeps only RB.
  - The legacy-only tail of `published_recovery` (Engine steps Commit, then "typed transition advances pins") is SC-N, as in §9.
- **(B)** Delete the `cfg(test)` #14 port listed in Q3 (about 300 lines in `workflow_source.rs` and `source_recovery.rs`), and leave RB/S5-W only. #14 reintroduces the port with its production consumer.

(A) keeps the existing port evidence alive with no non-test change. (B) shrinks test-only scaffolding but discards #14 groundwork.

**Placement (A).** The tests stay in `recovery_tests.rs`. The fm_d2 producer and restart become one `cfg(test)` `pub(crate)` helper (`runtime::installation::tests::fm_d2::restarted(provider, published) -> Restarted { dir, owner, runtime, config, task, artifact, ingress }`). The only non-test-file change is the `cfg(test)` module visibility on that path (§8.6.2).

## 3. verification_tests (12) and managed_tests (5): D4 R

Decided: D4 (a), convert to R now. Each test keeps its setup (profile admission, verifier, engine) up to the refused step into Implement inside `publish()`. It asserts the typed `ManagedBindingUnavailable` and the §8.3 F2 preimage. Its doc comment names the former subject and its owner, SC-N (the Commit/Tests/Review continuation).

**Decision R2-2 (approved, 6052644734).** `ManagedVerifier::{before_commands, before_completion}` and their `cfg(test)` fields and read sites (`execution/verification/mod.rs:210-246, 483, 527`) lose all callers in the R conversion chosen here.
- **Decided:** delete these `cfg(test)` items. SC-N re-adds a pause seam with its D2 Tests consumer.
- The setters need no Artifact or authority, so an R test could still call them before F2 without `allow(dead_code)`. This conversion keeps no such caller, because a seam that is set but never reached proves nothing (L1).

## 4. Counts

Classes and boundaries are unchanged. R stays at 21 scenarios / 25 boundary assertions (§9), and the 20 deferred tails stay the same. The D4 R conversions (verification 12, managed 5) were already counted in §7.2 R≈66.

## 5. Review outcome (Sol 6052644734) and implementation

| Item | Verdict | Implemented |
| --- | --- | --- |
| R2-1 A (keep the #14 port) | ChangesRequired (M1) | — |
| R2-1 B (delete the port) | Approve | `ef522bc` |
| R2-2 (delete the verifier seams) | Approve (L1 wording, fixed above) | `eae7052` |
| Counts and placement | Approve | — |

- **M1 (confirmed at `source_recovery.rs:142`).** Resume returns before it writes the Goal back, so the Goal stays Paused. The port claim requires `GoalState::Running`, so A could not reach the port after RB.
- **B scope.** Removed: ReconstructedFrame, RecoveryPause, RecoveryGuard, `recover_retained(_inner)`, SourceRecovery, `Store::{begin,accept,abandon}_retained_source_recovery`, and the begin-only `Snapshot.{project,goal}`. Kept: SourceReadBinding, `validate_source_read`/binding validation, the marker-bound refusal, the schema, epoch invalidation, and the Driver Source7 consumer.
- **Placement.** The R tests are `runtime::installation::tests::fm_rb` (same 7 names), next to the fm_d2 producer, so no module visibility changes. The restart is `ControlFixture::restart` (test module): shutdown, Runtime and owner custody released and awaited, `RuntimeOwner::open` with a new epoch, a new Runtime with the same config, and a new Unix-peer pair.
- **Ready row.** The settled evaluation is held, the captured manifest is corrupted, then released. The gate observes it without closure. The artifact is still Ready after the reopen, and this is asserted.
- **Scenario accounting under B.** Each test function runs the RB once per distinct producer. Loop variables that only selected a retired port mode (final [epoch], inflight [modes 1/2], foreign [×3]) are collapsed, because they no longer change the setup.
  - Executed RB scenarios: 8 (7 Published, 1 Ready).
  - S5-W assertions: 4.
  - In §9 units: R recovery 13 → 8 scenarios; boundary assertions RB 13 → 8, S5-W 4.
  - Deferred #14 tails: 13, unchanged (the subjects are not proven).
