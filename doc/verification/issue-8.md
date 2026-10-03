# Issue 8 verification

Issue: https://github.com/shuhei-suzuki/rururunx/issues/8
PR: https://github.com/shuhei-suzuki/rururunx/pull/36
Risk: STRICT, because this change owns durable workflow authority and native/side-effect reservations.
Reviewed production head: `8f46135c2bc9e9c103bbdac29d610536236815ec`.
Integrated base: merged Issue 18 (`a43afd8`), including its guarded audit API and rustix filesystem support.

## Acceptance and boundaries

Requirements/design/master/README were updated in the same PR. QUICK, STANDARD and
STRICT execute their actual phase plans through separate fake executor and reviewer
AgentAdapter implementations and explicit evidence ports. Commit precedes tests and
review per the current user Goal §35. No process exit, CI stub, PR creation or native
turn completion fabricates a review verdict, merge approval, test result or Task completion.

Atomic Task + Workflow + ContextVersion transitions enforce owner/version CAS, exact
ContextVersion/source/phase identity, monotonic workflow/risk, ordered prerequisites,
immutable factual history, live/Lost native reservations and review locks. Schema marker
3 has ordered atomic v1→v2→v3 migration; old snapshot clients reject new authority.
Ordinary Store APIs cannot rewrite workflow fields, publish its context pointer or forge
its factual audit event. Raw workflow transitions and observers are crate-private.

Each Evaluating round claims its exact prior observation count. The private observer
journals one exact-scope/phase/generation/Session/ContextVersion/claim outcome. Prior
Waiting/Failed rounds cannot resolve a resumed in-flight operation during concurrent
poll, restart, cancellation, raw closure or generation invalidation. Completion needs
the actual current Passed evidence. Compact observation metadata omit Context Pack text.

Irreversible Pr/MergeGate/Cleanup outcomes hold durably after drift, retain references
for same-attempt idempotent reconciliation and cannot restart into duplicate operations.
Cleanup resumes frozen pre-disposal source/class/phases. QUICK PR-created is nonterminal;
actual accepted MergeGate/Cleanup evidence is required for Completed. Cancel/fail retain
reservations; narrow release keeps decision/Task/context/evidence immutable, requires
persisted owned native termination and rejects unknown outcomes/unbound dispatch.
Project removal checks remaining active Workflow reservations.

## Local gates

All gates ran on the clean committed head using Rust 1.91.1 with committed Cargo.lock.
Git/process fixtures use temporary isolated repositories and owned process groups.

| Gate | Result |
| --- | --- |
| `cargo fmt --all --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS: 149 checks |
| `cargo build --workspace --locked` | PASS |
| `cargo build --workspace --locked --release` | PASS |

The 149 checks include 49 private workflow regressions, 19 real adapter/subprocess
regressions, both public API doctests and the integrated config/context/Git/project/state
suites. A subsequent test-only variant commit `d45abe7` covers both Waiting and Failed holds;
its focused regression and clippy pass. Production bytes are unchanged from the reviewed
head. The privacy check requires E0624 and has a positive accessible Store control.
Class checks cover all three preset launches, scoped source/owner/record/context CAS,
formal dependency freshness, rule/config drift, unsupported review, actual commit HEAD,
transport success separate from verdict, second-round claim concurrency/restart/cancel,
wrong/duplicate observer claims, irreversible holds, post-disposal Cleanup, terminal
rejected-review release, blocked owners, native status mismatch and audit spoofing.
The generic subprocess test verifies terminal status equals the already persisted Session.

Logs retained locally under `/private/tmp/rururunx-issue8-r6-final-{clippy,tests,debug,release}.log`.

## Independent review loop

Reviews used native Claude's configured default model, medium effort and this normal
read-only restriction: `claude -p --tools '' --strict-mcp-config --mcp-config <empty-MCP-json> --effort medium --output-format json`.
Normal hooks, global/project rules, auth and model configuration were preserved. Inputs
contained exact byte-verified public source/diffs, factual verification and prior independent
findings, with no executor chat, private configuration or credentials. The repository was
verified PUBLIC. Source remained clean and immutable until each review finished.
The manual read-only workflow exception applies; this is not the later Issue 16 Triple
Review dogfood and does not claim three-provider review evidence.

| Round | Outcome and verified remediation |
| --- | --- |
| 1, `3cb4d49` | Native tool-use exhaustion, `is_error=true`; no completed assessment, not credited. |
| 2, `8e7f12d` | Completed, 4 High/6 Medium/5 Low. Fixed factual failures, owner metadata preservation, committed HEAD, raw authority shape, stale pre-gate rejection, dependency freshness and same-Session resume. |
| 3, `97f3049` | Completed, 3 High/4 Medium/3 Low. Fixed equal-class rebinding, audited terminal/finalization boundaries, preclaim/predispatch CAS recovery, factual postgate observations and private raw authority; clarified reserved default and trusted native termination. |
| 4, `7794a8a` | Completed, 3 Medium/6 Low, no Critical/High. Fixed known-outcome postgate replay, irreversible drift holds, inactive-owner cancellation/terminal release, refreshed blocker handling, audit kind parity, frozen Cleanup, privacy controls and final-window CAS regression. |
| 5, `b97007d` | Completed, 1 High/1 Medium/5 Low. Verified resumed-round race, fixed exact claim/one observation fences, ordinary Failed terminal release, durable holds, validation-failure blockers, frozen Cleanup policy, payload duplication and status diagnostics. |
| 6, `b53377e` | Completed, 2 Low/1 Info, no Critical/High/Medium. Confirmed all Review 5 findings resolved; fixed hold blocker refresh and observer-only factual append; documented fail-closed unpublished snapshots. |
| 7, `8f46135` | Scoped delta completed, no Critical/High/Medium/Low; both Low fixes resolved. Optional Failed-arm test gap covered with a test-only variant and assertion-killed narrowed-guard mutation. Redundant observation bound remains harmless. |

Successful rounds have `is_error=false` and no permission denials. Actual findings were
verified against source, committed, tested and sent back for independent delta review.
Full native artifacts remain locally as `/private/tmp/rururunx-issue8-reviewN-{input.txt,output.json,result.txt}`.

## Semantic mutation evidence

Every credited mutant compiled and failed an actual test assertion. No compilation
failure, interrupted process or timeout is counted. Tests and production were restored
between cases. Counts are per round; repeated guard targets are not distinct mutants.

| Round | Assertion kills | Excluded trials |
| --- | --- | --- |
| Initial invariants | 27 / 27 | None |
| Review 2 fixes | 37 / 38 | Harmless diagnostic wording survived |
| Review 2 semantic delta | 3 / 4 | Reservation removal remained blocked by independent Store native fence |
| Review 3 fixes | 10 / 10 | None |
| Review 4 fixes | 11 / 12 | Removed hold remained enforced by independent invalidation hold |
| Review 5 exact claims | 9 / 9 | Initial unbounded fixture wait interrupted, excluded; corrected deadline rerun killed by assertion |
| Review 6 Low fixes | 2 / 2 | None |
| Review 7 optional Failed-arm control | 1 / 1 | None |

Review 5 mutations cover stale prior-round knowledge, missing claim advancement,
wrong-claim and duplicate observers, ordinary Failed terminal release, unstripped payload,
non-durable hold, weakened Project risk mapping and unknown raw closure. The earlier
privacy mutation made both method and argument access public, causing the expected
compile-fail doctest to fail; the positive consumer control compiled.
JSON/per-case logs are retained as `/private/tmp/rururunx-issue8-*-mutations.json` and
`/private/tmp/rururunx-issue8-{M*,R3*,K4*,E5*}.log`.
Restored final baseline: 49 workflow tests and both privacy doctests PASS.
The exclusive detached mutation worktree was removed normally after clean restoration.

## CI and pending integrations

Reviewed exact-head Linux/macOS CI: [run 37109940133](https://github.com/shuhei-suzuki/rururunx/actions/runs/37109940133), both SUCCESS at `8f46135`.
Final test/documentation-head CI is tracked by PR 36 exact-head checks; root must verify
both Linux and macOS SUCCESS before merge.

Final run 37110578255 failed in the macOS merged Issue 18 context suite: a native
Git cleanup uncertainty latched subsequent scans closed. Workflow and adapter checks
passed; Linux completed every check but received the matrix cancellation conclusion.
The unchanged local native context suite passed all 16 tests. The original diagnostic
discarded the underlying native error, so a narrow diagnostic follow-up preserves that
error chain while retaining the uncertainty latch and rejecting successful output when
ownership remains uncertain. Its regression covers both failure and successful-output
cases. No timeout or safety-boundary change is inferred from the initial failure.

Issue 9 owns actual multi-review scheduling/verdict reconciliation; Issue 12 owns typed
approval/blocker routing; Issue 13 owns actual unknown-outcome/crash reconciliation;
Issue 14 owns native cleanup; Issue 19 owns production Context Pack publication. GitHub,
commit, test, mutation/browser/staging and merge ports remain explicit integrations.
Missing evidence waits. Raw terminal Session writes remain a trusted verified native
provider/recovery boundary, never proof inferred from cancellation or caller JSON.
Identical foreign blocker text remains ambiguous until Issue 12 supplies typed ownership.
`workflow.default` is reserved for Issue 11 Task creation; Task::new currently uses STANDARD.
