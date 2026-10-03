# Issue 18 design

`RepositoryContext` captures registered Project, Goal and bound Task snapshots
under `SharedStore`, then releases the mutex. Existing bounded native Git process
supervision collects ownership facts, HEAD and NUL-delimited inventory/diff data.
The same exact ownership validator used by adapters checks canonical primary and
Task roots, common Git directory, registered repository identity and Task branch.
Native Git configuration and hooks remain authoritative; inherited Git routing
variables are removed by the existing native environment helper.

Tracked and nonignored untracked files enter the index. `additional_paths` admits
specific ignored sources/evidence; build trees and unrelated ignored files are
not scanned. A missing tracked/admitted file gets an explicit missing hash marker.
Binary files remain hashed with an explicit skipped-text reason. Oversized or
unsafe sources fail closed. This initial index supports small bounded repositories;
submodules, symlinks and oversized source files require future scoped handling.

A scanner runs on Tokio's blocking pool. It opens canonical roots as directory
FDs, traverses each component with `openat(NO FOLLOW)`, rejects nested `.git`
markers, and opens final regular files with `NOFOLLOW|NONBLOCK`. This prevents
ancestor/final symlink races and FIFO blocking. It excludes actual common Git
directory and Project namespace paths, even with nonstandard Git layouts.

Limits are 4096 files, 256 KiB/file, 16 MiB source+rule bytes, 128 additional refs,
8 KiB query text, 128 changed/evidence refs, 64 lexical definitions/imports and
512 identifier references per file, 128 edges/file and 2 MiB graph edge paths.
Lexical/graph truncation is visible on each file; emitted index metadata is
limited to 32 MiB. Git commands share a five-second deadline per collection;
the existing process-group cleanup also bounds retained output. Expansions are
limited to 128 files and require a narrower request when fanout exceeds it.

SHA256 hashes include every indexed source, admitted ignored source, primary
rule/config and missing marker. Inventory hash, HEAD, exact worktree and all
three state versions form `Freshness`. Initial indexing scans twice; ownership
and HEAD are checked around each scan. Reuse rescans source contents, then
rechecks state versions. Selection verifies again before scoped audit mutation.
Filesystem snapshots are observations, not atomic filesystem transactions;
callers must validate immediately before use and rebuild after changes.

Definitions use a cheap line/token heuristic for common Rust/Python/JS/TS
declaration keywords. Imports and identifier mentions connect file definitions
and file stems. Callers include lexical references, possibly definitions,
comments and strings; callees are file dependency neighbors. Completeness is
never claimed. Deterministic scoring prioritizes changed paths/symbols, task-text
matches, then one-hop graph neighbors; equal scores sort by source-relative path.

Packing renders the full mandatory header/rules/evidence first. Each optional
entry includes its wrapper and JSON metadata in the budget. A byte-count estimate
(`utf8_bytes_v1`) is deliberately labeled as an estimate rather than provider
tokens; byte and estimated-token limits both apply to the entire rendered payload.
It is not a model-specific tokenizer or a guarantee of a provider's context-window
usage. Measured tokens remain `None`. `NeedsBudget` has no payload or adapter-input
conversion. Explicit expansion files are mandatory in that request and cannot
be silently omitted. Optional omissions identify further expansion needs.

Maps/slices have private integrity fields, serialize for inspection, and do not
deserialize as trusted inputs. `ContextSlice::prepared_input` validates freshness
again before producing an existing adapter `PreparedInput`; callers own durable
ContextVersion publishing. Audit events expose factual selection metadata without
copying source bodies. The local `repository-context` example inspects existing
Store Task bindings and never starts an agent.
