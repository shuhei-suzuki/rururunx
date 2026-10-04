# Issue19 checkpoint append identity source evidence

Component only. Approved design1 at47401485581d94f0b79e9f6ae4ac879c045d4084,
native d52f3139-db9e-4947-89ec-bbf709a11efa, zero tools/MCP/read-only PUBLIC
packet57866 bytes/SHA256d2ef17089859aa3e2740da0333fd491ed6b768bed2326875659db1adabe8f93f.
No blockers; no new schema/native/lifecycle/permission contract.

Source912928b adds7 explicit exact identity/predecessor/retained-prefix checks in
existing append transaction. Existing constant visibility is crate-only. The sole
service producer already satisfies these checks; accepted bytes/digests unchanged.
Source1122430 corrects only test SQLite diagnostic layer: the injected custom
message is on the outer rusqlite Error, root source reports SQLite1811. Initial
failed fixture result remains saved; no production predicate was relaxed.

Actual in-crate owned repository/current Store/source snapshot fixture runs first
and consecutive private append, load_checkpoint, then actual checkpoint service
successor. Factual initial terminal Consultant history is UNVERIFIED provenance,
not model/native cleanup/admission/settlement proof. Both previous directions,
inner format/scope/session (missing and real second owner)/role/worktree, missing/
reordered/rewritten text/kind/session mandatory prefix refuse atomically; the outer
P/G/T/Session versions are current. Every refusal leaves record/head/audit unchanged.
Owned injected audit failure also leaves no partial record/head/audit. No positive
raw SQL seeding. Reader-only validation residuals are stated in the design.

Committed focused consumer test and clippy passed. Restored17 tests (16 original
encoder controls plus1 separate checkpoint identity test) passed. Full default-
parallel regression and exact final pushed-head CI are pending, never assumed.

## Compiled causal proof

Each mutant was committed separately in an owned detached worktree before actual
consumer test. Every mutant compiled and failed at the explicit inconsistent-body
ACCEPTANCE panic (exit101), not parser/compiler/other late errors.

| Predicate mutant | Exact committed head | Result |
| --- | --- | --- |
| format | `ea5d003d12d24b3c2f9188802b5d65c9627cad3a` | compiled; consumer-acceptance killed |
| scope | `c23a2a9dc4f090be317f3acd90b0c5d36c185205` | compiled; consumer-acceptance killed |
| session | `f41d04d283722f964d32f4473ab1f2b4f64185bb` | compiled; consumer-acceptance killed |
| role | `7ec8dd6ade891d29910893b2927a98aef978d87d` | compiled; consumer-acceptance killed |
| worktree | `d42feb59125025affe247be44752d36a034afd79` | compiled; consumer-acceptance killed |
| predecessor | `c0a1f25f282e87e6cdcf71a02e39d3db0175370b` | compiled; consumer-acceptance killed |
| mandatory-prefix | `a5de0af021a1db0dfdd41cdc2d1871efea30cefb` | compiled; consumer-acceptance killed |
| prefix-length-only | `9d219bd57d3fd1b4334de88926fb06cc448c1dce` | compiled; consumer-acceptance killed |
| prefix-first-only | `daaf5af814a5625301198a1b50639356d876d05b` | compiled; consumer-acceptance killed |

Restored clean detached `a53d12b99b81367fa9f5065dc1b4cbae2bf1b706` passed all17 controls.
Private saved mutation manifest/log paths: `/private/tmp/rururunx-issue19-checkpoint-identity-mutations.json`
and `rururunx-issue19-checkpoint-identity-mutant-*.txt`. These are not native authority.

Source1 independent component review pending. Whole PR39 remains DRAFT/UNAPPROVED.
Main schema3 has no CPP consumer; component schema5 is unmerged. All proposed6,
native/legacy/19-23-43-60 integration and complete parent CPP source gates remain OPEN.
