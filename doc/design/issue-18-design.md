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
submodules, symlinks, hard links, newline filenames, untracked nested repositories
and oversized source files require future scoped handling. Noncanonical path
aliases and case-folded aliases of one inode fail explicitly.

A scanner runs on Tokio's blocking pool. It opens canonical roots as directory
FDs before native Git observation, records device/inode identities, compares
them around scans and on reuse, traverses each component with `openat(NOFOLLOW)`, rejects nested `.git`
markers, and opens final regular files with `NOFOLLOW|NONBLOCK`. This prevents
ancestor/final symlink races below the anchored root and FIFO blocking. Root
opening compares metadata with the opened descriptor; ownership/identity checks
around collection detect observed root moves/replacements. These are local
observations, not an atomic transaction with native Git. It excludes actual common Git
directory and Project namespace paths, even with nonstandard Git layouts.

Limits are 4096 files, 256 KiB/file, 16 MiB source+rule bytes, 128 additional refs,
8 KiB query text, 128 changed/evidence refs, 64 lexical definitions/imports and
512 identifier references per file, 128 edges/file, 100,000 total graph visits,
512 query terms and 2 MiB graph edge paths.
Lexical/graph truncation is visible on each file; emitted index metadata is
limited to 32 MiB. Git commands share a five-second deadline per collection;
the existing process-group cleanup also bounds retained output. Raw NUL lists
preserve whitespace paths; revision arguments are separated from paths, Git is
resolved through absolute runtime PATH entries, and optional index writes are
disabled. Filesystem jobs and scanner/graph work have five-second deadlines.
An uncertain Git cleanup latches a process-wide context launch block, including
when cancellation drops the native preflight future; no later context result can
be published through another helper call. Automatic recovery of that uncertainty
is not implemented by this retrieval library.
Two process-wide filesystem slots are retained by workers until they exit,
including after timeout: hung filesystem syscalls cannot be forcibly cancelled,
but they cannot create unbounded detached scan workers or publish a result.
Expansions are
limited to 128 files and require a narrower request when fanout exceeds it.

SHA256 hashes include every indexed source, admitted ignored source, primary
rule/config and missing marker. Inventory hash, HEAD, exact worktree and all
three state versions form `Freshness`. Initial indexing scans twice; ownership
and HEAD are checked around each scan. Reuse rescans source contents, then
rechecks state versions. Selection verifies again before scoped audit mutation.
`Store::audit_if_current` checks Registered ownership and all three expected
versions inside the same Immediate transaction that appends the event, including
when another runtime/CLI uses a separate SQLite connection.
Filesystem snapshots are observations, not atomic filesystem transactions;
callers must validate immediately before use and rebuild after changes.

Definitions use a cheap line/token heuristic for common Rust/Python/JS/TS
declaration keywords. Imports and identifier mentions connect file definitions
and file stems. Callers include lexical references, possibly definitions,
comments and strings; callees are file dependency neighbors. Import/definition
identifiers are retained first, then frequent identifiers with deterministic
ties. Completeness is
never claimed. Deterministic scoring prioritizes changed paths/symbols, task-text
matches, then one-hop graph neighbors; equal scores sort by source-relative path.

Packing renders JSON-lines frames with typed section kind, scoped path,
content hash and escaped body; source/evidence text cannot create additional
authoritative frames. Packing renders the full mandatory header/rules/evidence first. Each optional
entry includes its wrapper and JSON metadata in the budget. A byte-count estimate
(`utf8_bytes_v1`) is deliberately labeled as an estimate rather than provider
tokens; byte and estimated-token limits both apply to the entire rendered payload.
It is not a model-specific tokenizer or a guarantee of a provider's context-window
usage. Measured tokens remain `None`. `NeedsBudget` has no payload or adapter-input
conversion. Explicit expansion files are mandatory in that request and cannot
be silently omitted. Optional omissions identify further expansion needs.
Audit `required` names describe requested mandatory content; `selected` is empty
on `NeedsBudget`. Worktree/rule keys use distinct prefixes. Index audit includes
admitted ignored paths and manifest hash; source bodies are absent from audit.

Maps/slices have private integrity fields, serialize for inspection, and do not
deserialize as trusted inputs. `ContextSlice::prepared_input` validates freshness
again before producing an existing adapter `PreparedInput`; callers own durable
ContextVersion publishing. Audit events expose factual selection metadata without
copying source bodies. The local `repository-context` example inspects existing
Store Task bindings and never starts an agent.
