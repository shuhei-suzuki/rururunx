# Issue #3 requirements

Risk/workflow: STRICT, because Git ownership and immutable review are shared safety boundaries.

Each Task gets an independent project-local worktree and branch (Issue numbers remain scoped by Project). The Git manager must reject protected or detached development, noncanonical paths, repository mismatches, existing branch/path adoption and unsafe removal. Task branch/path binding persists before native Git side effects, allowing inspection of interrupted creation.

Dirty state includes tracked, untracked and ignored files. Review requires clean exact HEAD; logical locks block runtime executor reservations/continuation. A persisted Starting executor counts as reserved, even when liveness is unknown. Review lock and executor reservation must serialize across SQLite connections. Cleanup requires exact ownership, clean files, no reserved executor/review and HEAD merged into the configured local base. No force, hook bypass or assumed successful rollback.

Logical locks cannot prevent a user's external editor/Git command. Verification must detect changed HEAD/files; unresolved locks and failed create/cleanup intents remain explicit recovery evidence. Workflow/CLI/recovery integration is handled by dependent Issues, not claimed here.
