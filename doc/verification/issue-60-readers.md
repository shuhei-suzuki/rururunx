# Issue60 selected Git reader lifetime verification

Status: requirements candidate UNAPPROVED; no implementation or acceptance.
Base2c6ae9d includes diagnostic partialPR61 and environment partialPR53. Their
independent approvals do not authorize this reader/owner-driver change. Parent60
and native16/F1/runtime-workload settlement remain OPEN.

Historical18c/b90/555 failures and every later CI failure remain FAILED/RED with
cause AND regression UNKNOWN. A reader JoinHandle source gap is verified
separately; no historical EOF writer, scheduling/inspection cause or regression
is identified by that observation.

Preparation: exclusive worktree issue-60, branchfix/issue-60-readers, from clean
main2c6ae9d. No source changes. Agent6 explicitly confirms approved StageA has no
common ProcessGroup/Drop/retained-child implementation/approval; StageB remains
required OPEN. Shared impact is consequently part of this independent design,
not an assumed dependency or silent helper edit.

Primary Tokio contract check: repository Cargo.lock resolves1.53.1. The fetched
locked crate's runtime/task/join.rs documents task detachment on dropped handle
and that cancellation must complete before the finished state is observed;
SHA256 `85484895de0c7f38a6ce49cb42ed4ba7b680b674deaf27aeb77467dc987b3ee7`. Native blocking work already
running is not cancellable by a Tokio abort request. These are source/API facts,
not observed evidence that a particular old failure left a live task.
[Published Tokio JoinHandle documentation](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html)
was checked2026-10-05JST; that page currently labels1.53.2. The baseline statement
is independently checked against locked1.53.1 source; no dependency/version change
is proposed or inferred from the latest-page label.
