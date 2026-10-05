# Issue #19 curated preparation custody integration

Status: selected composition controls verified; full regression failed and
independent composition review pending. Not merge-ready. Whole Issue #19 and
native readiness remain open.

## Immutable inputs and scope

Base main: `768f84319cd2a73e14cd39336eb12d99e9be81a7`.
Selected approved source: `857950094ae9732d31d6c11d773e7fe96adbae02`.
Historical source evidence: `a8aa95108bfe2fc098f5333f7bb5cf445c4c74a9`.

Only five code paths are imported: Codex attempt.rs, custody.rs,
custody_tests.rs, mod.rs and session.rs. Every imported code blob is identical
to the selected source. Main attempt/session/mod blobs match the approved
Design5 baseline `3a12ef787e6bcf1cdb5f6732fdc18b2c482b783a` exactly.
Thus the main-to-source code diff is the selected mechanical component.
The broad parent PR #39's other 29 paths, including Context Pack, Context,
Store and schema prototypes, are excluded. Three selected design/evidence
documents are retained; README and master adapter design describe this limited
composition. No source approval of the broad parent is inferred.

Two independent native reviewers approved selected Source2 with no
Critical/High/Medium findings. Their bounded pending hardening limits and
rejected prior rounds remain in the [historical evidence](../design/issue-19-lifetime-component-evidence.md).
The current Create factory is test-only. Its unreproduced actual worker-spawn
error plus pending Note combination and hypothetical future pre-created handler
panic require qualification before future factory/handler activation.

## Regression history and required gates

Exact selected-source CI 37256266809 failed on macOS: seven library failures,
including three independent inspection deadline observations and four subsequent
Context uncertainty refusals; the eighteen selected mechanics passed. Linux
debug passed and release build was cancelled by matrix fail-fast. The retained
last-poll facts do not establish child status or output at the deadline, because
the loop checks its deadline before the next poll/read. Cause and component
contribution remain unknown. Earlier Grok/watchdog/inspection failures remain
recorded, and no same-head retry or timeout/parallelism/latch change is used.

This smaller composition must compile and pass its own focused Debug/Release
controls, workspace regression, fmt, warnings-denied all-target clippy,
Debug/Release builds, relevant compiled mutation controls, two independent
composition reviews and exact final-head macOS/Linux CI before a limited merge.
Any later green run records its own observation and does not explain old failures.

No ordinary native capability, ready backend, native cleanup certificate, private
producer, schema upgrade or full parent acceptance is added. Production custody
selection remains None; the closed fixture creates no native/Git subprocess.

## Actual curated code checks

Clean code head `94350fbb55d35f305cbdc3ff6b35d3c6a74e23f3`, tree
`02df0958f193266e321f91615db62e9ae5dd8149`, was committed before execution.
Rust 1.91.1, locked dependencies, default test parallelism and original internal
deadlines were used. The first sandboxed attempt passed16 controls and failed2
checkpoint setup controls with an explicit inspection spawn PermissionDenied.
That log is retained separately. A host-authorized execution subsequently passed
all18 selected Debug controls in3.70s and all18 Release controls in2.75s.
The broader affected Release Codex suite passed121, ignored1, in49.42s.

The host full default Debug run failed: library282 passed,7 failed,23 ignored,
73.63s; later workspace targets were not run. All18 selected mechanics passed.
The failed Grok consumers were:

- live_reference_change_rejects_actual_start_before_spawn: expected environment
  denial, received Timeout: Git ownership preflight timed out.
- live_reference_change_rejects_actual_resume_before_spawn and
  owning_reference_change_after_snapshot_rejects_resume: actual preflight Timeout,
  with nondispatched/not-attempted receipt facts.
- owning_reference_change_after_snapshot_rejects_start: expected StateConflict
  prefix failed; the underlying failure string was not printed.
- dispatched_clean_receipt_reaches_actual_supervise_stages: Failed versus expected
  Lost, with nondispatched/not-attempted receipt; underlying cause unknown.
- unrelated_foreign_changes_do_not_revoke_checkpoint: original60001ms watchdog,
  last finite boundary1/case6/resumed_terminal; actual child SIGKILL/reap.
- unrelated_foreign_changes_do_not_revoke_resume: original60002ms watchdog,
  last finite boundary2/case6/checkpoint; actual child SIGKILL/reap.

Those stage facts do not establish the blocking operation or complete nested
settlement. Cause and contribution of this component remain unknown. This head
was not rerun to seek a green full regression. Fmt, all-target clippy with warnings
denied, Debug build and Release build passed separately. Full Release workspace
tests were not run; the affected Release controls are a separate scoped result.
Full Debug log SHA256:
`c0e67616f2b86a37c163cb61e5d7dbd045fb9cc616c0b3f3b2e7e1ea57452ce3`.
Affected Release log SHA256:
`2a1c5273f56170517e5775ac25c895be3151a682303ed8fa9d148542589bea6e`.
The private check manifest is `/private/tmp/rururunx-root-custody-host-checks.json`;
failed sandbox artifacts remain separate and are not counted as passing gates.

## Actual composition mutation controls

An independently owned detached worktree based on the exact94350f code executed
the same16 distinct boundary operators against this smaller assembly. Each mutant
was normally committed before testing, compiled, and failed at its intended
consumer assertion. These are16 operators verified on this composition, not16
new operators in addition to the historical16. The mutants remove/alter:

| Boundary | Actual assertion consequence |
| --- | --- |
| Armed caller revocation | Caller Drop leaves jobs unrevoked |
| Endpoint generation | Foreign endpoint note succeeds |
| Handler revocation check | Revoked queued notes change the count |
|4096-byte metadata cap | Encoded +1 metadata succeeds |
|64-entry inbox cap | Over-capacity note succeeds |
| NotCreated worker credit | Actual no-created completion cannot replenish |
| Registry sweep | Same adapter reaches32 retained entries |
| Unknown worker accounting | Required unresolved slot count is not observed |
| Worker Begin wait | File opens before installed Begin |
| Held entry retention | Resource-owning error loses the registry entry |
| Late worker observer | Pending worker observer is discarded |
| Opaque creator error accounting | Unproved3-slot reservation refunds |
| Bookkeeping versus resource hold | Original zero-effect removal is stranded |
| Reply after retirement | Actor freezes uncreated Held instead of Fresh |
| Metadata versus effect request | Queued fixed Note freezes genuine no-work |
| No-work registry restoration | Exact previous Control is not restored |

After all16 compiled assertion kills, the owned mutation worktree returned clean
to94350f and the identical complete tree02df0958; all18 restored controls passed
in3.18s. All owned mutation cargo commands exited and were waited; no Root
mutation command remained live before normal worktree removal. This is not a
certificate of all delegated/native resource settlement. The per-mutant
commit/test/exit/log-hash record is retained privately at
`/private/tmp/rururunx-root-custody-composition-mutation-verified.json`.
This verifies selected mechanical test sensitivity, not native backend acceptance.

## Later parent observation

The parent evidence-only head a8aa951 has unchanged approved selected code.
Its distinct exact CI37256989270 also failed on macOS: library315 passed with
23 ignored (all18 mechanics passed), followed by context5 passed/11 failed.
Three context consumers reported inspection deadline observations301350/301853/
300488us; eight subsequently refused on the conservative context uncertainty
latch. The saved Pending/WouldBlock facts are prior polls, not facts at the
deadline. Mac builds were skipped; Linux every CI step succeeded. No same-head
retry or source correction was performed. This observation remains separate
from the original857 failure and the curated94350f full regression.
