# Issue 26: Project registry requirements

Workflow: STRICT (project boundary and persistent activity guards).

Implement persistent registration of two independent repositories through `rrx
project add/list/status/remove`. IDs survive restarts, recovery and soft removal;
Issue numbers are local metadata, never runtime-global identity.

Accept only canonical exact primary Git roots with committed local base history.
Reject subdirectories, bare repositories and linked source roots. Preserve the
Issue 3 identity contract (canonical common Git directory plus sorted base root
commits). Detect missing/moved roots, replacement identity and unsafe references
when registry commands reopen state; persist BLOCKED with a reason. Never rebind
or automatically clear BLOCKED; an explicit repeated add must validate the same
root/identity before recovery. Identical Git metadata and history at the exact
same location are the same identity under this contract.

Project config/rule/source refs must resolve to source files within the owning
root, outside Git metadata, task namespaces and nested repositories. Revalidate
before reads, including symlinks. Store only environment names, reject values,
assignments and routing overrides, and return only the selected project's names.
No credential values are persisted, resolved or forwarded by this issue.

Resolve selectors by stable UUID or an unambiguous active display name; infer CWD
only within registered sources or bound, Git-validated task worktrees. Soft removal
must atomically reject nonterminal Goals/Tasks, all unresolved/live sessions
(including Lost), and active locks. Retain filesystem contents and audit/history.
Reject active writes after removal to prevent a second runtime from racing removal.
Runtime-global default state must remain independent of CWD. Help, version and
config-check keep their existing no-state behavior.

Integration with native agents, scheduler limits/fairness, progressive rules,
Review/Approval Bundle generation and Goal execution remains in dependent issues.
