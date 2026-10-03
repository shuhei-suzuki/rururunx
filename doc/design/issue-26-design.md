# Issue 26: Project registry design

`project::ProjectRegistry` borrows `Store` and exposes add, reconcile, list,
resolve, status and remove. Durable UUIDs and existing scoped foreign keys bind
Project → Goal → Task. `ProjectStatus` enumerates only the owning project's
Goals/Tasks/sessions. Duplicate names are allowed but ambiguous lookup fails;
UUID-shaped selectors always mean identity. Removed rows reserve their identity
and namespace for auditable, same-ID reactivation.

Registration canonicalizes the supplied path, checks exact Git top-level and
non-bare primary root, infers origin's local default/main/master/current branch
(or takes `--base`), and calls Issue 3's repository identity helper. Git commands
strip inherited Git routing variables while preserving native hooks/config.
References are normalized after containment and repository ownership checks;
worktree namespaces stay below the owning source root. Registration does not
create worktrees or read rule contents. Repeated add preserves existing options
unless explicitly supplied; `--max-tasks` overrides the stored concurrency policy.
An initial project overlay supplies its per-project task default.

Reconciliation validates all REGISTERED entries before commands. Invalid source
or reference paths persist BLOCKED and a JSON-compatible `blocked_reason` field
(default None for older snapshots, no SQL migration required). BLOCKED is sticky
until explicit validated add. Existing immutable identity/base cannot change.
`effective_config`, `scoped_file`, and `environment_names` require REGISTERED and
revalidate source identity before returning selected input. The environment API
returns reference names only; native launch isolation is a dependent adapter duty.

CLI `--state` chooses a shared database. Default resolution is absolute
`RRX_STATE_PATH`, then `XDG_STATE_HOME/rururunx/state.sqlite3`, then
`$HOME/.local/state/rururunx/state.sqlite3`. Only project commands open/create it.
`--project-config` on project add is source-relative; other project commands use
the saved reference and reject arbitrary overlay injection. `list --json` and
`status --json` provide machine-readable state. `list --all` includes tombstones.

Removal is a version-checked state update. Store checks Goals, Tasks, typed
sessions and locks within the same SQLite IMMEDIATE transaction as REMOVED;
active Goal/Task/session/lock writes also require REGISTERED under their write
transaction. Terminal updates can reconcile/cancel blocked work. Locks and Lost
sessions must be explicitly reconciled. No source/worktree/branch is deleted.

Impact: CLI, Project JSON, configuration composition, transactional Store active
write guards; existing Git safety consumers retain identity semantics and names.
Tests use isolated temporary repositories and the actual binary, include same
Issue 42 in separate projects, CWD inference and adversarial path/config/env
routing. Full existing CLI/state/Git regressions and Rust lint/build checks apply.
Mutation proofs target identity matching, reference containment and idle removal.

Limitations: each validation still traverses base history (Issue 3 contract); no
shared cross-database coordination or existing-worktree adoption is promised.
Completed Task bindings retain names, so reuse of an old issue worktree name
requires a later explicit recovery/naming policy. Rule contents, env values and
native context/bundle construction are handled by dependent workflows.
