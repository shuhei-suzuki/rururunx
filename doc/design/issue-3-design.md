# Issue #3 design

`git::WorktreeManager` exposes create/status/ensure_mutation_allowed/lock_review/verify_review/unlock_review/cleanup. TaskId resolves exact persisted Project/Goal ownership. Git runs directly with argv and scoped CWD, removes inherited repository-routing environment variables, retains native config/hooks, and never uses force or no-verify.

Project root must be canonical exact Git top-level. Supported worktree namespace is a normal descendant of root with no symlink ancestors. Task path is a direct child, has exact top-level and shared Git common-dir, and attached branch matches its durable binding. main/master/configured base are protected. default names are worktree/issue-N + feature/issue-N, or task UUID for Tasks without Issue.

Store validates a typed WorktreeLock {active,revision,worktree,branch,reason} and exact binding. Its immediate transaction checks both existing locks and Executor Session reservations. Starting/Running/WaitingApproval/WaitingHuman are conservative live reservations; terminal/reconciled sessions do not reserve. Assigned Task branch/path cannot silently change.

Creation persists binding/intent, reserves a maintenance lock, invokes git worktree add and releases after success. Failure keeps binding/lock/intents for explicit reconciliation; no deleting a hook-created tree. Cleanup similarly reserves a lock, verifies source and merged ancestry, removes the worktree then locally deletes the merged branch. Partial failure leaves the lock/intent. Records and audit retain provenance after cleanup. Status is a synchronous local operation; future scheduler callers should run it on a blocking worker, not a busy poll.
