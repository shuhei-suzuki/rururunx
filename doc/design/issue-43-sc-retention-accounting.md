# Issue 43: success-stage retention accounting (SC §12 addendum, SOL-L01)

- **Status:** proposed amendment to the approved SC HOW `a40355d` §12 "Success stage" figure. It answers SOL-L01 (Issue #43 comments 6040615918 and 6042027870).
- **Pin:** as implemented at `88b4f08`. Paths are relative to `crates/rrx/src/`.
- **Method:** encoded bytes of owned copies at the existing bounds (`BODY_BYTES` W = 8 MiB). `Body<T>` owns raw + canonical, so a decoded Workflow counts 2W; parsed values are heap only and are not counted. These are not RSS figures (M1 measures).
- **No new cap and no Task limit is proposed.** The existing guards are unchanged.

## 1. What each retained object owns (as implemented)

| Object | Lifetime | Owned (encoded) |
| --- | --- | --- |
| In-flight `GateClaimPlan` / `GateObservedPlan` (`state/managed_binding/gate.rs` `LinkAdvance`) | from planning until the write is Known (kept while Uncertain/RolledBack for the SAME-plan confirmation) | currency successor Workflow `Body` ≤2W, Unit ≤16 KiB ×2, head link ≤4 KiB, `after` `Body` ≤2W, link data ≤4 KiB ≈ **4W ≈ 32 MiB**. The full-image record mutation is no longer retained; it is materialized per write. |
| Known `GateClaimAcknowledgment` | until the observation is Known (then released) | postimage raw ≤W, count ≈ **W = 8 MiB** |
| `SettledGateCompletion` | from evaluation until the observation is Known (then released) | observation ≤64 KiB, gate receipt record and `SourceSnapshot` (small) |
| Known `GateObservedAcknowledgment` | until the job is released | postimage raw ≤W, observed facts (evidence, receipt) ≈ **W = 8 MiB** |
| `SuccessClosurePlan` | from closure planning until the job is released | its currency successor Workflow `Body` ≤2W, Context raw ≤W (8 MiB stored-decoder bound), publication (small); shares the observed acknowledgment ≈ **3W = 24 MiB** |
| `SuccessClosureMaterial` | from materialization (before admission) to the end of one write or confirmation turn | transient, counted separately in §6 (SOL-L03); not covered by the old §12 16 MiB figure |

## 2. Per-stage peaks

| Phase of the stage | Retained | Peak |
| --- | --- | --- |
| Claim in flight | claim plan | ≈32 MiB |
| Observation in flight | Known claim + observed plan + completion | ≈40.1 MiB |
| Closure in flight / Known until release | Known observed + closure plan | ≈32 MiB |

Plus `SettledTerminalImages` ≤ about 6.3 MiB (§12, unchanged). The maximum is ≈ **46.4 MiB** per active success stage at the bounds, instead of the approved "≈8.06 MiB".

## 3. Proposed amendment

1. Replace §12's "Success stage: compact plans … ≈ 8.06 MiB" with §1–§2 above (peak ≈40.1 MiB plus the sealed images).
2. Update the worst case accordingly: ≤128 jobs × (56.2 + 40.1 + 6.3) MiB ≈ **12.8 GiB** encoded. With the #81 rule of one active Task per Project, the practical bound is the number of concurrently active Projects.
3. As §12 already does for the binding plan, list a further reduction as a policy question, not a PR1 requirement: digest-only in-flight plans that materialize the before and after images inside the write and confirmation from the SAME inputs. This would change the shared `SettledCurrency` / `CurrentWorkflowSuccessor` (used by BR and binding) and is outside SOL-L01's minimal scope.

## 4. Invariants kept

- SAME proof, inputs, `at` and digests.
- Exact CAS and two-branch confirmation.
- The completion is retained until the observation is Known (SOL-M02).
- The claim and completion are released at Known on both the Driver and the Root paths.

## 5. M1 status (as of `ccaef5f`)

- **M1a (measurement only, `e4190cd`):** a document encoded exactly at `BODY_BYTES` is owned as 8 388 608 raw + 8 388 608 canonical = 16 MiB; at `SESSION_BYTES`, 8 MiB. One byte over each bound refuses. These are the 2W figures used in §1.
- **M1b:** not yet measured (needs a reachable large genuine closure Context and retained/per-turn instrumentation).
- **M1c: SETUP (unreachable precondition).** It asks for legitimate `Store::put_context` history in the Task scope before Goal acceptance. That precondition cannot be produced:
  - `context_versions` has foreign keys to the Goal and the Task (`state/schema.sql:40–50`);
  - Goals and Tasks exist only through accepted ingress (generic Goal creation refuses);
  - after acceptance, `put_context` refuses ("accepted Goal Context requires genuine owned publication", `state/mod.rs:1443–1458`).

  The `gate_claim` headroom check therefore remains defense-only, as the SC design already allows.

## 6. Transient ownership (SOL-L03, as implemented after the SOL-M03 fix)

The old §12 figure ("≤16 MiB, one at a time by Store serialization") is not inherited. A Driver or Root worker builds its material before control admission and the Store guard. Workers of different Projects can therefore hold materials at the same time; only the Store-held part is serialized.

Units are encoded owned bytes, W = 8 MiB. Parsed values are heap only, as in §1.

| Step | Store held | Owned during the step | Peak |
| --- | --- | --- | --- |
| Materialization `images()` + `material_digest` | no | decoded SAME preimage `Body` 2W (dropped on return); the new Workflow serialization moved into its `Body` 2W; the digest's JSON copy plus its serialization ≤2W | ≈ **5W ≈ 40 MiB**, momentary |
| Held material, waiting for admission | no | Workflow postimage `Body` 2W; Task ≤1 MiB, Unit ≤16 KiB, artifact, operation and link data (small); identity head of the preimage (six columns, small) | ≈ **2W ≈ 16 MiB** per waiting worker |
| Writer W1–W8 (`close_phase_success`) | yes, serialized | material 2W; permit old image (a raw copy of the retained preimage) W; permit new image W. No decode inside the Immediate since SOL-M03 | ≈ **4W ≈ 32 MiB**, one at a time |
| Confirmation (`confirm_phase_success`) | yes, serialized | material 2W; the current Workflow row read ≤W | ≈ **3W ≈ 24 MiB**, one at a time |

Per active success stage, the bound is the §2 retained peak (≈40.1 MiB) plus the transient. The transient is ≤5W at materialization, then 2W while waiting. The serialized writer adds ≤2W once system-wide.

Worst case, counting every waiting worker at its momentary materialization peak: ≤128 jobs × (56.2 + 40.1 + 6.3 + 40) MiB ≈ **17.8 GiB** encoded, plus one serialized writer increment (16 MiB). As in §3, the practical bound under the #81 rule of one active Task per Project is the number of concurrently active Projects. No new cap or Task limit is proposed. These are encoded-byte bounds, not RSS figures; M1b remains the measurement.

SOL-M03 source fix (`success_closure.rs`):
- `images()` records the preimage's six identity columns (`record_head`), id and version, from the decode it already performs outside the Store.
- The writer builds the permit old image from that head plus the retained raw (`record_image_from_head`), and the W6 CAS from the same id, version and raw.
- The SAME inputs, the fixed `at`, the digests and the two-branch confirmation are unchanged.
