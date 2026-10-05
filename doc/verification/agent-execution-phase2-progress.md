# Native Agent execution Phase2 progress

Status: implementation in progress. This is a component checkpoint, not a phase
completion report, native acceptance, or authorization to begin Phase3.
The approved contract is Phase1 commit `9a11d03f1b1676c2c3c8a24b6b3fd328759f7db5`.
Main, legacy native capability advertisements, README Status and licenses remain
unchanged. Work is isolated on `feat/agent-execution-phase2`.

## Preliminary independent review

The two [round3 reports](agent-execution-phase2-preliminary3-reviews.json) examined
immutable source `07bb9cd1e1dd772f9ba5847eef32d3054089f64a` independently, without
peer findings, dirty source, tests, native Agents or account calls. Both requested
changes. Their reports certify neither complete Phase2 nor native compatibility.
Earlier rounds were also preliminary; their findings are not erased by this
checkpoint. A final coordinated source review remains required.

The two [round4 reports](agent-execution-phase2-preliminary4-reviews.json) examined
`061c7914f8b21bc54f58b429a7d6fbe1104e5d8a` independently. Both requested changes;
their earlier reported dispatch/probe/reentry/quota defects are statically closed
for the reviewed paths. The subsequent component correction includes:

- A M1: grants now restore the owner's finite resource namespace, including TMPDIR
  and build output paths, while preserving the shim caller's native authentication,
  settings, Git overlay and inherited PATH after the managed tool prefix.
- B H1: a live same-unit Git chain may forward a canonical candidate index in its
  own Git administration directory. Arbitrary, foreign and stale index contexts
  refuse. Native `commit -a` and partial-commit pre-commit hooks read the actual
  candidate index through the built rrx shim.
- B M1: executor detached checkout/switch refuses before effects until an owned
  detached-state binding is implemented. Readonly reviewer snapshots remain a
  separate explicitly bound detached input.
- B M2: finite nonsecret native failure categories survive status and terminal
  audit persistence. Auth/capability/metadata refusal uses `refused`; malformed
  protocol uses `protocol_error`; genuine missing transport uses `lost`. Work
  remains unknown when no reliable terminal work result exists. No raw native
  error, authentication value or provider output is serialized as a diagnostic.

These corrections still require independent re-review on a fixed commit.

The [round6 reports](agent-execution-phase2-preliminary6-reviews.json) independently
examined `9a0c4768ff361aa142f26e9b8102b4871fb24e4c`. A approved the component scope
with one low test-sensitivity issue; B requested a high correction. Root identified
the existing partial UNIQUE executor constraint in B's initial causal sequence;
B withdrew that sequence and independently confirmed the distinct old
reviewer/verifier case. The correction now explicitly refuses a live executor
replacement and atomically closes prior readonly units before changing generation,
releases their quota/waiters and queues cleanup. Known work and retained artifacts
remain historical facts. A new readonly unit can consume the retained prior SHA
in the new generation concurrently with the new executor. The duplicate-start
control now overlaps two starts during a held version helper, rather than only
calling the second start after the first returns.

Workflow-owned artifact publication now uses the same immediate transaction as
Task revision, passed phase evidence and fresh ContextVersion. It checks the exact
unit/Session/phase/artifact/dependency bindings; standalone publication refuses
when a Workflow owns the Task. A ledger-only causal control exercises publication,
cancellation-first and dependency drift with rollback. On-disk retention controls
remain separate: this ledger control does not attest file/object integrity.
The new candidate native AgentAdapter requires explicit managed input and Session
identity, propagates configured agent aliases and exposes work/cleanup independently
without fabricated exit zero. It is exercised through account-free protocol peers;
Registry/operational CLI routing is still unfinished and no release capability or
native qualification follows from this intermediate component API.

| Finding | Current correction; independent re-review remains pending |
| --- | --- |
| A C-H1: native input/ALLOW retirement race | Private scoped producer journals a digest-only effect intent transactionally before wire dispatch. Cancellation-first rejects admission; intent-first is issued/uncertain and never replayed. Bootstrap checks and cancellation use a persistent timer. |
| A C-M1 / B H1: first Available bucket deletes probe ownership | Bucket recovery retains the live pool probe until its actual lease release/terminal. Backoff resets only when every known exhausted bucket has cleared. |
| B H3: subscription error present only in failed terminal | Exact owned Codex terminal `usageLimitExceeded` is normalized to quota-interrupted/unknown work; ordinary failure remains failure. |
| B H4: Claude last bucket overwrites exhaustion | Session-correlated telemetry updates the ledger; owned error-during-execution consults accepted participating buckets. Budget/turn caps remain failure. |
| B H2: Git hooks wait on their own gate | A live root Git lease permits same-unit command-chain reentry. Nested holders retain the root lock; stale/foreign reentry is rejected. Preparation propagates the same lease. |
| B M1 and root finding: Runtime executes tools outside native sandbox | IPC now carries admission/lifecycle only. The shim executes the actual CLI as its child with inherited native context and stdio; Runtime receives no stdin/stdout/stderr or credentials. Stateful bounded decoding survives partial-frame cancellation. |
| A C-M2: OID-shaped names treated as branch authority | Finite branch/switch/checkout forms separate owned mutable names from detached/start-point OIDs; rename/copy/bulk forms refuse. |
| A C-M3: cleanup invalidates capture authority | Cleanup is an append-only factual projection, with no execution-authority version change. |
| A C-M4: Claude native reviewer role omitted | Reviewer/verifier launches select native plan mode and never grant source writes; Codex selects native read-only. Actual provider/profile conformance remains Phase3. |
| A B-M2-remaining: raw helper namespace/admission | Version helpers and unit-bound capture/snapshot content/digest checks now use unit admission/profile. Retained historical graph inspection and initial source qualification still need coordinated integration; this finding is not closed. |

## Executed controls

The [round5 reports](agent-execution-phase2-preliminary5-reviews.json) independently
examined immutable `74f484844119b6d3adcc16366ebb71f7626fd7c0`: A approved the
reviewed components, B requested two medium corrections. Neither is a final
Phase2 approval. The subsequent correction separates typed Claude authentication
and unsupported-capability errors from ordinary work failure, and adds an
unclassified capacity terminal: unknown work, closed native/finalization flags,
capacity wait and a durable 60-second local recheck for a fresh attempt. It never
fabricates confirmed subscription exhaustion or closes a sibling Task's pool.
Finite installed Codex schema discriminators and owned Claude assistant/result
indications select the bounded category; intermediate native retries are not
terminals, and budget/turn caps remain failures. Claude structured error types
are described in the [official SDK message reference](https://code.claude.com/docs/en/agent-sdk/python).
That reference is a decoder-policy input, not real version/account conformance.
New fields have nullable compatibility defaults for existing development rows.

All initial source Git helpers now follow unit/resource registration and the
common-Git gate. A missing explicit base is resolved once through that admitted
helper and bound before native Session launch. Cancellation/epoch/source fences
are checked during helper waits. Dropping preparation closes its own semantic
unit/Session identity and marks its pending helper uncertain without replay;
an independently admitted winning Session cannot be adopted by a competing start.
These corrections require immutable independent re-review. Retained historical
inspection and coordinated Workflow publication remain outstanding below.

On this macOS host, owned temporary Git repositories and explicitly synthetic
Claude/Codex protocol peers exercise actual new component producers. They do not
use provider accounts, authentication, models or network. The synthetic peers
advertise pinned protocol versions for decoder testing; this does not qualify
those versions of the real CLIs.

The checkpoint adds controls for multi-window pool recovery and a live-probe
competitor, Codex terminal-only exhaustion and retry-then-terminal exhaustion,
Claude other-bucket/stale Available events, unrelated budget failure and foreign
Session telemetry. Other controls cover partial IPC reads, actual native Git
post-commit-hook reentry followed by sibling Git progress, OID-shaped foreign
refs, cleanup during held result staging, digest-only durable dispatch admission,
and reviewer role arguments through actual fixture launch.

Four simultaneous peers (two Claude/two Codex) remain an account-free protocol
control: one is cancelled, three finish and retain independently verifiable
committed graphs. This is not the requested real four-Agent acceptance matrix,
an installed-shim test, or native sandbox/hook compatibility evidence.

The native plan-mode flag is visible in installed Claude `--help`. Current
[official CLI reference](https://code.claude.com/docs/en/cli-reference) and
[permission documentation](https://code.claude.com/docs/en/agent-sdk/permissions)
describe plan mode routing writes to permission handling. The fixture verifies
selected arguments, source permissions and transport; actual pinned provider
behavior with user settings and required hooks remains unverified.

The subsequent macOS regression run passed 371 library tests with zero failures
and 24 ignored test-child entry points. Six initial failures were corrected by
giving current-writer corruption fixtures the existing connection-local writer
contract, and constructing an actual historical SQL layout for schema-v2
migration rather than relabelling a v4 database. The old-writer negative controls
remain active; the production writer guards were not relaxed.

An additional built-binary integration control uses the actual rrx Git entry point,
not the protocol peer: an unstaged FORBIDDEN change is refused by a managed
pre-commit hook for both `commit -a` and partial commit. A SAFE change commits.
The hook's temporary file and build target use its own Task's paths even when the
caller supplies sibling paths; the sibling's worktree and output remain intact.
This is not a `cargo install`, authenticated Agent, native sandbox, or both-OS
conformance claim.

New controls cover bootstrap cancellation for both protocol fixtures and seven
separate Codex bootstrap failure categories without retaining a private error
sentinel. Component selection passed 38 tests after those additions. Restart
recovery now validates indexed identities before mutation, preserves known work,
fences managed Sessions including already-retired units, marks unacknowledged
effects unknown without replay, clears old quota waiters/leases, and admits fresh
attempt identities. Pure SQLite controls exercise successful reconciliation and
full rollback on a redirected body; Runtime SIGKILL and escaped processes remain
Phase3 cases. Native quota decoding uses the installed CLI-generated int32 schema;
out-of-range fractions remain unknown, with percentages above 100 retaining an
exhaustion observation rather than a fabricated capacity fraction.

After the restart/quota corrections, `cargo test --workspace --no-fail-fast
--offline` passed on this macOS host: 374 library tests, 112 primary integration
tests and two doctests, with zero failures. Child-entry test runs are not counted
twice. The 27 primary ignored cases include explicit child-only fixture entry
points and opt-in real native tests; they are not acceptance evidence. Subsequent
targeted built-binary testing also passed a SAFE partial commit, so the candidate
index control covers both rejection and successful consumption. Formatting,
review JSON parsing and diff whitespace checks passed. Three dead-code warnings
still reflect unconnected finalization/reclamation APIs. Earlier failed full
regression attempts remain acknowledged: integration corruption canaries needed
the current-writer contract, and schema-v1 Project migration needed a genuine
historical SQL layout. The append-only/immutable-record and old-writer negative
controls remain enabled.

## Required work still outstanding

The next immutable candidate adds a generation fence for live readonly units,
atomic managed-artifact/Task/Workflow/Context publication, and a private native
adapter carrying exact managed Session identities. A replacement executor closes
old-generation reviewers/verifiers and releases their capacity in the same
transaction, preserving known work and retained artifacts. An active executor
still prevents replacement. The duplicate-start control now overlaps the two
starts while the first admitted helper is held; only the winner launches.

On this macOS host the stable subsequent workspace regression passed 384 library
tests, 112 primary integration tests and two doctests, with zero failures and 27
primary ignored cases. Synthetic adapter-vtable controls exercise admission,
configured aliases, typed cancellation and independent work/cleanup reporting.
Atomic publication controls exercise cancellation and dependency drift rollback.
An earlier run observed a transient doctest visibility error during concurrent
editing; the stable rerun passed after correcting visibility. A sandboxed scoped
run could not spawn a legacy Grok observation helper; the authorized stable full
run passed. These are fixture/component results, not account, installation or
both-OS acceptance evidence. Registry, managed Workflow preparation and CLI
construction are still unconnected at this candidate.

Round6 independent reports are preserved in
[the review record](agent-execution-phase2-preliminary6-reviews.json). A approved
the reviewed components with one low observation. B withdrew the original
executor-replacement finding after confirming the partial UNIQUE index, and
reported the live readonly-generation lease issue instead. The corrections above
require a new immutable review; neither report constitutes final Phase2 approval.

The subsequent [round7 reports](agent-execution-phase2-preliminary7-reviews.json)
requested retained-content verification at coordinated publication and identified
the same readonly lease issue through the existing stop/retire path. Executor
retirement now closes affected readonly authority and capacity before advancing
generation. Workflow publication requires a nonserializable proof minted by
scoped Git/manifest verification, binds the full artifact snapshot and unit
authority, and rechecks both in its atomic SQL transaction. Evidence strings and
Ready metadata cannot construct that proof.

The managed Registry now selects explicit `claude`/`codex` provider configuration,
independent of configured aliases, and shares one NativeSessions/tool server under
the same Runtime owner as Workflow. Workflow reserves an exact pre-Session unit
identity with its Task projection and phase claim in one transaction before
preparation helpers. Native inputs must match that phase's immutable
ContextVersion. Admission waits remain resumable without duplicate input; terminal
quota/capacity interruption keeps work unknown, releases the phase for a fresh
namespace and reports typed Task waiting. Explicit local retry still rejects
unknown evidence/external outcomes. Native status uses the unit's owned worktree,
including separate readonly review inputs, rather than the executor projection.
Readonly provenance is currently retained in the owning WorkflowEngine instance;
missing provenance refuses advancement and requires a fresh attempt.

Account-free controls now exercise the real managed Registry/Workflow/adapter
path with owned Git repositories and synthetic native peers. Intact retained
graphs publish together with Task/Workflow/Context pointers. Missing/corrupt
manifests, missing retained refs, cancellation and a metadata-version change after
verification all refuse publication. Native quota/capacity terminals for both
providers wait with fresh resources instead of Task failure. A readonly reviewer
uses the published SHA despite changes to the old executor path; changing the
review input rejects its result. The evidence/source ports in these controls are
test integrations, not a shipped Runtime gate policy or native review verdict.

The stable macOS workspace run passed 387 library tests, 112 primary integration
tests and two doctests, zero failures and 27 primary ignored cases. The initial
run failed one ledger fixture because it omitted the newly required typed managed
status; that fixture now supplies its explicit ledger-only identity and confirms
metadata-only publication is refused. Formatting and diff whitespace checks
passed. CI/native conformance, operational CLI and final independent review are
still required; no new release/OS/account claim follows from these controls.

The [round8 independent reports](agent-execution-phase2-preliminary8-reviews.json)
confirmed the prior retained-publication and stop-generation corrections. Each
reported one Medium: concurrent due-wait callers could overwrite the winning
Workflow version; subscription updates omitted the durable wait reason. The
wait consumer now claims the exact due record before asynchronous source/native
work, never adopts a competing in-flight record, and leaves CAS losers without
write or retirement authority. A barrier control overlaps two Workflow engines
sharing Registry/Store and checks both successful Session binding and a renewed
quota wait. The subscription control holds a native retry open and compares its
watch update with status, then checks the terminal wait reason. Both controls
passed on this host; neither exercises a genuine subscription.

Readonly completion now requires a private provenance proof reverified against
its immutable source tree and retained graph. The exact unit authority and full
Published artifact snapshot are rechecked in the same Task/Workflow/Context SQL
transaction that closes readonly finalization. Intact review, changed input,
after-verification artifact mutation and cancellation controls passed; known work
and the executor's accepted SHA remain independent of rejected review acceptance.

Managed Registry construction now applies configured concurrency ceilings to
transactional admission leases, including leases before Session registration.
Unknown-capacity policy caps global sessions at six and provider executors/total
at two/three. Lower configured global limits apply; configured per-agent alias
caps conservatively share the strictest cap for that provider, rather than
promise independent account balances. Project capacity counts distinct active
Tasks and applies the lower scheduler/registered-Project limit; another readonly
unit for an already admitted Task does not consume another Task slot. Global,
provider alias and Project cap controls wait before native spawn. These are
component tests, not completed production scheduling/installation qualification.

The subsequent stable macOS workspace regression passed 390 library tests,
112 primary integration tests and two doctests, zero failures and 27 primary
ignored cases. The first full run caught a misplaced test-only publication hook;
the second caught a short reset deadline expiring under parallel test load.
The hook now runs only for actual managed completion proofs, and the competing
wait control advances its fixture's observation window explicitly before a
bounded capacity recheck. Both negative/control cases remain enabled. Clippy
with `--workspace --all-targets -- -D warnings` passed after correcting formatting,
unnecessary unit bindings, explicit test mutex scopes and finite enum storage.
No authentication, real model operation, Linux run or release qualification was
performed by these checks.

Round9 [independent component reports](agent-execution-phase2-preliminary9-reviews.json)
confirmed the readonly completion proof, due-wait claim and configured admission
caps. A approved that fixed component source with no findings. B requested a
Medium correction: supervisor Drop used durable Cancelled but published Lost,
and omitted the retained wait reason. Abandoned supervision now retires as Lost
and projects the actual unit/Session fields; known terminals and explicit
cancellation are preserved. A dedicated Tokio executor is destroyed during an
ordinary running turn and a held native quota retry. Both owned synthetic
controls confirm closed permissions, durable/session/watch Lost agreement and
one input dispatch. This is supervisor lifecycle evidence, not Runtime SIGKILL
or authenticated Agent qualification.

An independent `rrx-process-tracker` crate now contains the public libproc/sysctl
FFI. rrx's workspace unsafe forbid is unchanged. Only this crate's macOS module
permits audited unsafe; it depends on OS wrappers and no rrx/SQLite APIs. Its
bounded same-user enumeration returns exact cookie matches, birth identities and
coverage categories; no argv/environment values or cookie appear in public
results/debug output. Temporary environment buffers are filtered for the cookie. Explicit buffer
cleanup is a source-level attempt, not compiler-proof secure memory erasure. macOS never signals a discovered raw PID.
Linux opens a pidfd before reading the process environment and checks both birth
identity and the fd's current process association after the read; signals use
only that retained handle and report send/denial/unknown separately from exit.

On this Mac, three tracker unit controls and one owned-child integration control
passed. Two same-user children carry distinct cookies: only the exact match is
returned, discovered-PID termination reports Unsupported without signaling it,
and the sibling stays live. The child-only entry is ignored in the parent run.
The Linux implementation has not been built or executed on this host. No claim
of Linux pidfd conformance or complete enumeration follows. The tracker is not
yet connected to a Runtime cleanup backlog or native stop path.

The stable subsequent workspace run passed 391 rrx library tests, three tracker
unit tests, 112 rrx primary integration tests, one tracker child-discovery control
and two doctests, with zero failures. The primary ignored count is 28 including
the new child-only tracker entry. Workspace Clippy with all targets and warnings
as errors passed. The first full run encountered a legacy Context Git output
cleanup timeout, which latched that integration process and failed 13 cases;
the unchanged Context-only rerun passed all 16, and the unchanged full rerun passed.
This observation is retained as a limitation of the frozen legacy path, not a
reason to extend its ps-based observations. Production retained-source integration
must use the scoped authority path instead of that global latch.

Round10 [independent reports](agent-execution-phase2-preliminary10-reviews.json)
requested two distinct Medium corrections and the same Low correction. Linux
now reads the effective UID from bounded `/proc/<pid>/status`, rather than
procfs inode ownership. Non-dumpable processes can have root-owned procfs files
without changing effective UID; this behavior is documented in
[Linux proc_pid(5)](https://man7.org/linux/man-pages/man5/proc_pid.5.html).
Unavailable metadata is reported as coverage uncertainty. Exit observation now
polls the retained pidfd without reaping; exit readiness and PID disappearance
are distinct, as documented in
[Linux pidfd_open(2)](https://man7.org/linux/man-pages/man2/pidfd_open.2.html).
The Linux fixture leaves its terminated child unreaped until readiness is
observed. A same-user non-dumpable child control checks actual UID classification
and, for unprivileged execution, denied environment coverage. These Linux controls
have not been compiled or run on this Mac; they are not OS conformance evidence.
A private temporary-buffer guard attempts cleanup on normal, over-limit and
partial-read-error exits. The latter injects an error after bytes are read and
checks the actual error category; no physical secure-erasure claim follows.

NativeSessions now owns a periodic historical CleanupWorker, also usable through
an explicit bounded sweep. The worker holds only a weak RuntimeOwner between
sweeps; its Drop aborts the scheduler. Each sweep claims at most four closed units
using the current owner epoch and cleanup-job version CAS before OS observations.
Both native and result-finalization permissions must be closed, so a broad cookie
scan cannot stop retained-result capture helpers. Claims have a durable retry
reservation; expired/replayed/old-epoch observations cannot amend the backlog.
Each observation quarantines only its unit's unreleased leases. It never changes
Task/Workflow/evidence, unit work, accepted artifact or the unit authority version.
Scans retain aggregate unavailable/changed/limited coverage, not environment or
argv contents. Kernel calls are not subject to a hard interruption deadline.
Linux termination uses only retained pidfds; macOS discovered-PID termination
remains Unsupported. Docker and filesystem/port release are still pending, so an
empty observation does not claim all tracked resources reclaimed. Last-seen
matches that cannot be stopped are reported separately from unknown coverage.

Two transactional controls passed: finalization-open jobs remain unclaimed, and
cleanup preserves successful work plus a sibling's authority/leases; expired
claim and epoch replay are refused without partial observations. Two additional
service controls passed on macOS: a real retained/published commit survives a
cookie sweep while its sibling child stays live, and an idle worker does not
retain the owner lock. The first matched child remains live and reports leftovers
on this Mac, consistent with unsupported discovered-PID signaling. These are
owned synthetic child/component controls, not authenticated Agent acceptance.

The full macOS workspace regression passed 395 rrx library tests, four tracker
unit tests, 112 rrx primary integration tests, one tracker child-discovery control
and two doctests, zero failures and 28 primary ignored cases. A preliminary
broad test filter unintentionally selected legacy process/socket controls under
the restricted shell, which reported permission failures; the actual new ledger
fixture also initially used an inadmissible Active initial lease and was corrected
to Reserved. The precise ledger controls and subsequent authorized full run
passed. Clippy initially caught a redundant buffer borrow and test-module
placement; both were corrected. No frozen ps observation, credential behavior,
real subscription or Linux runtime qualification was added by these checks.

Round11 [independent reports](agent-execution-phase2-preliminary11-reviews.json)
closed both tracker corrections statically, and each requested the same Medium:
the cleanup consumer discarded termination and exit-observation categories.
Cleanup observations now include a bounded typed action list with safe resource
identity, requested action, termination outcome and independent confirmation.
Denied, Unsupported, Unknown, skipped budget, unavailable exit observation and
observation errors remain distinct in persisted history. Remaining IDs describe
last-seen candidates; only an actual retained-handle exit observation marks them
exited. Raw OS error strings are not copied into these receipts. A bounded public
history reader exposes those observations without changing authority; historical
rows missing the new action field deserialize with an empty list. This is an
additive JSON receipt change, not a schema or license change.

The macOS retained-commit/sibling control now compares the persisted receipt with
its returned observation and checks Unsupported/Unavailable for its live match.
A fallible-observation control distinguishes Denied/still-observed from
Unknown/observation-error, tests serialization and old-row defaults, and confirms
raw injected diagnostics are absent. All three cleanup-service controls passed;
all 20 execution-ledger controls also passed. The previous full workspace run belongs to the
prior checkpoint; these action corrections do not inherit a later Linux/native
or release acceptance. Formatting and current-action workspace/all-target Clippy with warnings as
errors passed.

## Docker component checkpoint

The [round12 reports](agent-execution-phase2-preliminary12-reviews.json) each
approved the typed cookie-action correction at `6350c9f`, without C/H/M/L
findings. These were static component reviews, not provider/OS acceptance.

Docker qualification and historical cleanup were added at `39f077f`. A first
account-free protocol run passed one control and failed six because the host's
ambient `DOCKER_HOST` refused qualification before the fixture could run. The
private fixture-only program now ignores that ambient endpoint prerequisite;
production still refuses it. At `52b43e6`, seven protocol controls, a full macOS
workspace run and warnings-denied Clippy passed. An independent temporary-worktree
mutation removed initial namespace-label verification; the actual foreign-action
oracle failed, and the restored source passed. No actual Docker daemon was used.

The [round13 reports](agent-execution-phase2-preliminary13-reviews.json) requested
changes. Both identified Runtime/shim Docker configuration divergence and mutation
acknowledgement timeout losing exact action identity. One additionally identified
changing inventories exceeding the composed persistence bound. All three defects
were verified against their consumers. The routing issue was High in one report
and Medium in the other; both original severities are retained. Corrections at
`a3ee6fc` bind a nonsecret configuration-directory digest in the real shim request,
Runtime qualification, grant recheck and historical target; register exact
container/engine action intents before spawn; create typed Unknown actions before
await; and reserve 128 Docker plus 896 cookie remaining identities. Every partial
return applies the report cap and marks limited coverage on truncation. Configuration
and authentication files are not read/copied/substituted.

Eleven Docker controls passed on that source, including kill and remove each
performing a synthetic mutation and blocking before acknowledgement. Their exact
Unknown target/action survives both durable intent and persisted cleanup receipt.
A changing-inventory control observes 32 initial and 32 different final containers,
32 networks and 32 volumes through actual registered helper processes. It bypasses
the aggregate five-second timeout to exercise the legal worst-case inventory on a
host that starts 128 Python helpers; this is a bound/composition control, not a
latency or service-budget qualification. The actual outer composition and Store
persist those identities with a controlled maximum process receipt. The process
IDs in that composition control are synthetic; no 896-process scan is claimed.

A separate actual Cargo-built rrx/IPC test uses two synthetic Docker configuration
roots with the same context name and distinct synthetic engines. A differing config
or shim-only DOCKER_HOST refuses before probes/creation; the matching config reaches
the recorded engine and creation identity. Its child environment is isolated without
changing this process's global environment. This is installed-entry wiring, not
actual Docker or authenticated Agent compatibility. At `cd7b620`, the producer loop,
coverage and composition control share the actual cookie limit; at `e067578`, the
control calls Store persistence before the length assertion.

The [round14](agent-execution-phase2-preliminary14-reviews.json) and
[round15 reports](agent-execution-phase2-preliminary15-reviews.json) each have two
independent component approvals with zero C/H/M/L. Round14 covers `a3ee6fc`;
Round15 covers the final small budget/control delta at `e067578`. The initial
review findings are closed statically; final coordinated Phase2 approval is pending.
These reviewers executed no tests, native Agents, Docker, authentication or OS actions.

The full macOS workspace run at `cd7b620` passed 407 rrx library tests, four tracker
unit tests, 113 primary rrx integration tests, one tracker discovery control and
two doctests: 527 primary passes, zero failures and 29 primary ignored cases.
Nested reexecuted witnesses are not counted again. Workspace/all-target Clippy with
warnings as errors passed on that same source. The later `e067578` only moves a
test assertion after persistence; its targeted actual persistence control passed.
The prior full-run tree is not relabelled as a later tree or Linux/native qualification.

Four distinct compiled causal mutations are retained in the
[checkpoint ledger](agent-execution-phase2-docker-checkpoint.json), with exact
mutant/restored commits, patches, local log hashes and failure oracles. Removing
the Runtime/caller configuration check let the actual built shim accept a different
synthetic engine; the entry refusal oracle failed. Removing the exact mutation
intent target failed the timeout receipt's required target lookup. Increasing the
real cookie producer cap to 928 combined with the changing 128 Docker identities
caused the actual Store persistence call to reject the 1056-entry observation.
Each corresponding restored source and selected test passed. The earlier label
mutation failed its actual foreign-action oracle and also passed after restoration.
These are account-free component mutation controls, not real Docker or native tests.

`cargo install --path crates/rrx --locked --offline` at `e067578` succeeded into an
owned temporary install root and produced one optimized rrx binary. Version, help
and config-check passed without starting native Agents. This is local-source macOS
installation evidence, not crates.io distribution or Linux installation. The
installed help still exposes Project/configuration commands; operational run/Goal
commands remain pending. No PATH, license, README Status, user Docker configuration
or global installation was changed.

The [staged master profile](../design/master/agent-execution.md#6-staged-docker-command-profile)
records finite Docker versions and refusal/coverage limits. Actual Docker 28
templates/CLI behavior, remote contexts, Compose, image-declared anonymous volumes,
network/volume creation/attachments and complete delegation coverage are unqualified
or unsupported. Observed unexpected networks/volumes are reported, not deleted.
The overall cleanup remains unknown or leftovers; ports/worktrees stay quarantined,
and an empty Docker list is not a complete collection or release certificate.

## Historical retained-result inspection checkpoint

The public ResultStore verification path now registers each finite read-only Git
helper before spawn. Its authority is the current Runtime epoch plus the complete
indexed Ready/Published artifact snapshot, independent of the producing unit's
retired generation or closed native/finalization permissions. Each operation has
a distinct cookie, finite safe target, bounded capture, periodic epoch/artifact
checks and transactional receipt revalidation. Historical executor/profile/temp
paths are not needed, and read failures never overwrite work or cleanup outcomes.
Public publication rechecks the complete verified artifact in its existing CAS.
Manifest reads are bounded at 128 KiB plus a sentinel. Generic unregistered Git/
capture helpers now compile only for owned test fixtures.

At `1ac7700`, ten result controls passed and one failed because the corruption
fixture assumed loose-object storage. A pack-independent correction at `10e9abc`
passed all eleven. The twenty existing execution-ledger controls also passed.
Initial Clippy exposed the now-unused production helpers; test-only compilation
and imports were corrected without allowing dead code.

The [round16 independent reports](agent-execution-phase2-preliminary16-reviews.json)
requested changes: inherited GIT_COMMON_DIR could substitute another common store,
and shallow ancestry could hide missing required parents. Both were verified with
actual public-path negative controls in owned temporary repositories, at `8543974`.
An additional graft-metadata audit reproduced the same ancestry truncation. All
three oracles failed before correction. Internal Runtime Git now clears inherited
repository/common/object/namespace/ancestry routing; retained capture/inspection
refuses canonical aliases, commondir, alternates, shallow and info/grafts. Required
native settings/hooks and HOME are not replaced. Direct GIT_SHALLOW_FILE/GIT_GRAFT_FILE
environment behavior is unestablished; it is not credited as an exploit reproduction.

The [round17](agent-execution-phase2-preliminary17-reviews.json) reviews approved
the corrected component at `ccae042`, each with zero C/H/M/L. A full run found one
cleanup fixture still passing the pre-publication Ready DTO after publication.
Its two-line test-only correction uses the actual Published DTO at `187cc63`;
the targeted cleanup control and full workspace then passed. The
[round18](agent-execution-phase2-preliminary18-reviews.json) reviewers independently
approved that test delta with zero findings and unchanged production bytes.
These are static component approvals, not coordinated Phase2 or native acceptance.

On exact `187cc63`, the macOS workspace run passed 415 rrx library tests, four
tracker unit tests, 113 primary rrx integrations, one tracker discovery control and
two doctests: 535 primary passes, zero failures and 30 primary ignored cases.
Nested reexecuted witnesses are not counted again. Current workspace/all-target
Clippy with warnings denied, formatting and diff checks passed. Explicit verification
PATH selected Git 2.49.0, although the interactive shell selects Apple Git 2.54.0;
only the former is credited to these controls. Neither is a general tool-version
qualification. Linux and actual native-provider compatibility remain unverified.

The [retained inspection ledger](agent-execution-phase2-retained-checkpoint.json)
preserves exact sources, all six compiled mutation patches/commits and restoration
trees, plus 23 local log hashes. Omitting common-directory sanitation, shallow
refusal, graft refusal, intent registration, exact artifact equality or publication
snapshot equality fails its actual selected oracle. Each restored source passes,
with a tree identical to the corrected mutation baseline. Owned mutation worktrees
were normally removed after commands ended; their commit chain remains under a
local verification ref. No push, actual account/Docker call, license or README
Status change, current-source installation or Phase3 acceptance occurred.

## Live quota waiting and recovery checkpoint

The native producer now connects owned Claude plan-window exhaustion to the same
unit's durable WaitingQuota and watch/status, retaining its native input and
active lease. A telemetry-only rejection does not mint recovery-probe authority;
Codex's owned willRetry=true subscription error remains a separate explicit
retry producer. Foreign Sessions and unknown windows cannot create confirmed
subscription waiting. Unrelated Available windows do not erase exhaustion.

When every participating window is Available in the accepted pool ledger,
transactional same-unit authority/lease validation restores Running/no wait.
The watch carries the same authority and Workflow polling restores the active
phase's Task state. The same native Session, generation, worktree and input remain
in place. A reliable native success remains successful even after quota waiting.
Cancellation closes its authority without replaying input or changing a sibling.
This is a staged component, not an operational CLI or real quota qualification.

The [round19 records](agent-execution-phase2-preliminary19-reviews.json) preserve
two independent medium findings at fixed `f63048ab...`. A's accepted live recovery
finding was reproduced at `5a89957d...`: the pool accepted a new Available window,
but the preterminal watch remained Quota. B's telemetry/probe coupling was
independently reproduced by the full f63048a regression: same-window Allowed
incorrectly reopened exhaustion, changing the native terminal and Workflow result.
The initial missing live-wait producer was separately reproduced at `904b037...`.

Corrections at `a75629ee...` and `d30a9bec...` separate waiting/probing and connect
accepted recovery through Store, native watch and Workflow. Both
[round20 reviewers](agent-execution-phase2-preliminary20-reviews.json) approved
the fixed component with no C/H/M/L findings. Clippy then required lexical scopes
for two test MutexGuards rather than explicit drop calls; production/design bytes
were unchanged. Both [round21 reviewers](agent-execution-phase2-preliminary21-reviews.json)
approved this final test delta at `48ed158e824aeab749b5b121d78438e4fd506891`, again
with no C/H/M/L findings. These are static independent component approvals only.

The [checkpoint ledger](agent-execution-phase2-live-quota-checkpoint.json) binds
source/blob identities, all original review records and 16 local log digests.
Owned local protocol controls use an ordered permission notification after quota
frames, before terminal, rather than assuming a sleep proves stream consumption.
Cases cover live rejection, another allowed bucket, accepted new-window recovery,
foreign Session, unknown window, successful terminal, held Workflow waiting and
cancellation. Actual input-intent counts remain one and sibling unit snapshots
remain unchanged. Fixture source/evidence ports do not qualify production gates.

On macOS 26.6.2/aarch64 with Rust/Cargo 1.91.1 and explicit test PATH Git 2.49.0,
the full fixed d30a9be workspace passed 537 primary tests, zero failures and 30
primary ignored entries: 417 rrx library, 113 rrx integrations, four tracker unit,
one tracker discovery and two rrx doctests. Nested child entries are not counted
again. Final 48ed158 has identical production bytes; its 24 live-consumer controls,
workspace/all-target Clippy with -D warnings, fmt and diff checks passed. No
current-source full workspace or installation run at 48ed158 is fabricated.

Four final committed/compiled mutations fail actual runtime assertions:

| Mutation | Consumer failure |
| --- | --- |
| Omit Claude live-wait dispatch | Preterminal watch is None instead of Quota. |
| Allow telemetry wait to mint a probe | Actual held unit has forbidden recovery-probe ownership. |
| Omit accepted live-recovery dispatch | Accepted Available pool leaves the watch at Quota. |
| Omit Workflow recovered Task projection | Workflow Task remains WaitingQuota instead of Implementing. |

Each revert tree equals fixed d30a9be. The final restored 24 controls pass, and
the mutation chain remains at the scoped local verification ref in the ledger.
These are runtime failures after compilation, not compiler rejection; omission
mutants can produce unused-method warnings without substituting them for the
assertion oracle. Initial failing regression and Clippy logs remain preserved.

No provider account, native auth/settings/hooks, actual subscription recovery,
four authenticated Agents, Linux build/run, Docker engine or current installation
was tested. Cleanup guarantees, Phase2 completion and MVP completion do not follow
from this checkpoint. Operational source/evidence/Runtime integration remains
dependency-ready work; no additional component approval is requested.

## Registered initial-source component checkpoint

This checkpoint advances the production source-bootstrap prerequisites. It does
not complete production Workflow source/evidence ports or native acceptance.
The [machine-readable checkpoint](agent-execution-phase2-bootstrap-checkpoint.json)
retains exact commits, source/log hashes, original failure evidence, mutation
chains, host/tool versions and limitations. The separate
[review ledger](agent-execution-phase2-bootstrap-reviews.json) preserves all five
independent design/source rounds and their original findings.

Design `a732376` made initial preparation distinct from successful Agent work and
specified private same-unit adoption. Design1 review A found that the actual
Workflow rule/config producer remains separate from Sources.capture. Inspection
of `inputs`/`load_rules` confirmed that finding: even a committed repository index
would otherwise receive live tracked rules. `9b4ad6a` explicitly specifies the
prepared committed rule/config frame at the actual policy/hash/payload consumer,
external-origin handling and an A-versus-B input control. Both Design2 reviewers
approved the component contract. That Workflow frame connection is still pending.

Source `6be63c8` adds `AttemptManager::prepare_workflow_source` and a non-Clone,
non-serializable PreparedExecutor with private producer fields and an abandonment
guard. It reuses the actual registered attempt/UnitGit preparation path, before
base-resolution Git. It remains Preparing with no Session, work result or artifact.
Its bounded ordinary-file reader resolves the exact base tree entry and blob,
using literal relative paths and registered helpers; it does not read later HEAD
or live worktree bytes. This is a file reader, not yet a complete RepositoryMap.

Native start rejects this phase before version helpers or preparation abandonment
effects. Session registration, native capacity, non-Git helper and delegated/native
intent transactions independently reject it. Ordinary preparation cannot mint the
reserved phase. Schema 5 replaces connection-local contract-4 write guards with
contract-5 guards without changing table layout; unrelated append-only triggers
remain. Dropping the capability closes exact owned authority and records unknown
work, retaining paths/resources for best-effort cleanup rather than fabricating
success or disposal.

The initial full regression passed 540 primary tests. Initial Clippy rejected the
new test's explicitly dropped MutexGuard lifetime; `5efdc89` uses a lexical block
and also rejects the reserved phase at the public snapshot preparation entry.
Its new reader fixture incorrectly included a symlink before preparation. Actual
regression and Source2 review A both showed preparation refusing that unsupported
content before the size/literal reader assertions. `da85933` separates ordinary
file reader controls from a later symlink-preparation refusal. Source2 review B's
earlier approval is retained, including its Source3 acknowledgement of the missed
reachability issue. The qualification guard was not relaxed.

Final source `da85933a3d2c5c8c692d5f906f6fb2e00be63b7c` passed the full macOS
workspace regression: 421 rrx library, 113 primary rrx integration, four tracker
library, one tracker discovery and two doctests, **541 primary passes**, zero
failures and 30 primary ignores. Internal filtered child runs are not counted
again. Clippy all-targets with `-D warnings`, build, formatting and diff checks
passed. Both final component reviewers approved with zero findings; these are
static component reviews, not coordinated Phase2/native approval.

Twelve final mutations each compiled and failed at an actual consumer assertion:
Native, Session, quota, non-Git helper and delegated grant restrictions; exact
commit origin; abandonment guard; contract-4 write fence; snapshot phase entry;
blob size and type; literal path handling. Every restoration tree equals the
final source tree. Four selected controls passed on the final restored tree.
Both mutation chains are retained under the checkpoint's named verification refs;
all three newly owned temporary worktrees were normally removed after clean/tree
checks. No outstanding command handles remain.

An actual CLI check created an owned empty schema-5 database with the current
debug build. The previously recorded `e067578` installed schema-4 binary refused
`project list` with `unsupported state schema 5, supported 4`, exit 1, while its
logical database dump remained unchanged. Already-open contract-4/cached-writer
refusal and current-writer success passed separately through actual SQLite.
This is not a current-source installation or CLI run/Goal qualification.

The same-unit adoption transaction, committed Workflow mandatory-rule/config
frame, complete committed context index and real source/evidence ports are the
next integration. Runtime run/status/stop/resume, verifier/reviewer policies,
recovery and Linux/native qualification still remain. No new human decision is
needed for this dependency-ready Phase2 work. The full Phase2-to-Phase3 human
approval gate remains; README Status and licensing are unchanged.

## Committed Workflow inputs and first-unit adoption

This component advances the preceding bootstrap prerequisites into actual Workflow
consumers. The final immutable source is
`32e0ca18f76ee2932b30b93a36a945775f009c24`, with clean checks and independently
scoped source reviews. See the [checkpoint](agent-execution-phase2-source-wiring-checkpoint.json)
and [original review records](agent-execution-phase2-source-wiring-reviews.json).
This is still Phase2 work, not shipped Runtime/CLI or native qualification.

ManagedWorkflowSources registers its private preparation capability before
initialize reads the Task projection. Its complete committed tree/blob corpus
supplies the bounded lexical CommittedIndex. Workflow.inputs and prepare_pack now
consume one private committed policy/rule/input frame, bypassing the legacy live
rule/config loader for this managed integration. Unsupported untracked/external
mandatory refs explicitly refuse. The index retains all inventory/content/OID/skip
metadata; the existing bounded Workflow/artifact dependency map receives its
complete-inventory digest, code, instruction and mandatory-rule/config digests.
Goal/Task instructions and rules precede discretionary repository selection.

The first actual Executor adopts the same Unit ID, generation, branch, worktree,
profile and resource identity in an Immediate transaction. It retains the existing
Task/Workflow/Context/P/G/epoch/projection checks, disarms the actual guard only on
success, and compares both private source text and the complete actual serialized
Context envelope. Subsequent successful native terminals produce genuinely
captured/published commits, never a fabricated bootstrap work-success artifact.
Cached frames also validate accepted P/G semantic instructions after capability
consumption. A changed frame refuses before a new native effect; automatic fresh
input recovery is still unfinished.

RetainedGit extends its finite, current-epoch artifact inspection to the exact
commit tree and ordinary bounded blob OIDs observed there. It registers each
intent before spawn and verifies current artifact/epoch before receipt. The actual
control retires the producer, removes its historical profile and changes surviving
worktree bytes, then reads the captured commit bytes and refuses an unrelated old
blob. No historical native/finalization grant is reopened.

Independent Source1 reviews found the native Context envelope/raw-text mismatch
(CP-M1/C-H1), a legitimate negative LFS grep exit misclassified as unsettled
(CP-M2), and cached retained inputs missing current P/G instruction validation
(C-H2). These were verified against actual source consumers and corrected. Fatal
qualification/reader errors still prevent private frame creation. Adoption rejects
unresolved helper states and non-Git effects; an observed no-match grep is valid
qualification, not a fabricated successful helper. A separate test-fixture error
attempted to downgrade a persisted Standard Task before its intended guard; the
control now constructs a fresh Quick Task. Original failures remain in the record.

At final Source2, C independently reviewed the whole component; A reviewed the
producer/adoption/Workflow/readers/tests, excluding its own Index; B independently
reviewed the Index/registered producer contract, excluding its own Workflow tests.
All final reviewed scopes report zero findings. These are static component
approvals, not final Phase2 or provider/OS acceptance. Peer findings were not
shared before independent completion.

| Provider | Workflow | Actual account-free consumer result |
| --- | --- | --- |
| Claude fixture | QUICK | Same initial Unit; complete fixed native input; terminal, commit capture/publication and graph verification passed. |
| Claude fixture | STANDARD | Same result through the first Requirements Executor after initial Issue/Worktree gates. |
| Claude fixture | STRICT | Same first Requirements Executor result; later STRICT gates are not attested. |
| Codex fixture | QUICK | Same initial Unit; complete fixed native input; terminal, commit capture/publication and graph verification passed. |
| Codex fixture | STANDARD | Same result through the first Requirements Executor after initial Issue/Worktree gates. |
| Codex fixture | STRICT | Same first Requirements Executor result; later STRICT gates are not attested. |

These six cases are sequential local protocol fixtures. Early evidence gates are
also fixtures. They do not use subscriptions, models, authentication or network,
and do not qualify real Claude/Codex, four parallel native Tasks or Linux.
Actual initialize/prepare_pack/outgoing native bytes distinguish committed rule/
config A from modified live Project B. Actual published-artifact consumers reject
changed Goal objectives or Project rule refs without another native input or
artifact mutation. Other controls target public payload injection, stored Context
corruption, stale Task CAS, altered envelope generation, pending helpers and
initial cancellation/failure/last-source-owner Drop.

On macOS 26.6.2/25G83 arm64 with Rust/Cargo 1.91.1 and Git 2.49.0, final clean
source32 with `RUST_TEST_THREADS=4` passes **554 primary tests, 0 failures and 30
primary ignored**: 434 rrx library, 113 integration, 4 tracker library, 1 tracker
discovery and 2 doctests. An internal 44-test observer subprocess and one filtered
Codex child are excluded as duplicate execution. Workspace Clippy/all-targets with
`-D warnings`, build and fmt pass on that same fixed source. This build is neither
a new cargo-install qualification nor a Linux build.

The initial unrestricted-parallel source32 run failed three existing fixtures:
two Grok synthetic child watchdogs expired near 60 seconds and one Docker report
lacked an expected unregistered volume. Its new Workflow controls passed. The
same-source bounded-parallel full run passed all three and the entire workspace.
Scheduling/load sensitivity is an inference; the precise cause and unrestricted
repeatability are not established. Production and test deadlines were unchanged;
the failed run is recorded and is not counted as successful regression.

Six parent compiled mutations remove complete-envelope equality, exact Task CAS,
unsettled-helper refusal, retained blob membership, actual Workflow committed-
frame wiring or cached instruction validation. Every actual consumer fails with
an assertion (exit 101), not a compile error. Final restored controls pass on exact
baseline trees: two negative/reader controls for f429 and four Workflow controls
for source32. The first four and last two chains are retained under
`refs/rrx/verification/agent-execution-phase2-source-wiring/`, and their clean owned
temporary worktree was normally removed. They are not claimed as controls executed
after every restoration. The delegated pure index records seven debug/release
controls and ten separate compiled mutations; their scope is not native authority.

Unverified or unsupported boundaries remain explicit: automatic existing-Workflow/
changed-instruction recovery, external mandatory refs, large disabled-context
baselines beyond 128 requested files, actual native serialization/provider limits,
production review/verifier gates, Runtime/CLI owner construction, both-OS CI and
actual settings/hooks/subscription compatibility. Byte packing estimates are not
provider tokens. Process collection remains best effort, and rururunx is not a
security sandbox. There is no new human decision for dependency-ready Phase2
integration; the whole Phase2-to-Phase3 approval gate remains in place.

## Initial production Workflow gate checkpoint

Source2 `b6e21db36310ebf233865b24e8aa7a24c6ef834f` implements a concrete
ManagedWorkflowGates library port. Requirements/design were fixed at
`1c17f8470465a9ffacb6cb4aaefdfc3058b8f2bf` and independently approved by A and C
before source. A and B independently approved Source1 `85c13d27` and the final
test-only Source2 delta, excluding their own earlier Index/tests. Peer findings
were withheld until independent completion. These are component approvals only.
The [checkpoint](agent-execution-phase2-production-gates-checkpoint.json) pins
sources, checks, hashes, controls and limitations; the
[review ledger](agent-execution-phase2-production-gates-reviews.json) preserves all
six original static reports.

Issue checks the registered local Task specification; it does not claim a remote
Issue was created. Issue/Worktree retain the genuine initial capability, validate
its actual profile, exact registered unit and leased Git ownership/base/clean
namespace, and record observations without consuming adoption or inventing work.
Implement requires the exact owned successful terminal and retained Ready artifact.
The three commit gates inspect the corresponding Executor's Published artifact,
never current branch HEAD or uncommitted bytes. Existing private atomic Workflow
publication remains the only accepted-result transition. Receipts pin the actual
Evaluating claim, Context data digest, launch/observed revisions, source frame and
checked unit/artifact. They are diagnostic records, not native grant authority.

| Account-free control | Observed result on macOS |
| --- | --- |
| Claude and Codex QUICK protocol peers, separately | Production Worktree → same initial Unit → Implement → retained publication → Commit passed; Tests explicitly waited. |
| STANDARD initial Issue/Worktree | Registered local specification and actual preparation passed without native input. |
| Dirty preparation / missing acceptance criteria | No successful initial receipt or native input. |
| Forged native work outcome / corrupted retained manifest | Implement held, no phase success or Published artifact. |
| Separate Store reopen / surviving executor writes | Three scoped receipts remained readable; retained accepted graph remained valid. |

These are three test functions with seven fixture cases, not real native provider
or four-Task qualification. Requirements/Design/ImpactAnalysis content policy,
ReviewEngine, verifier commands and PR/merge/cleanup evidence still return named
Waiting outcomes. RequirementsCommit/DesignCommit lack qualified real predecessor
milestone policies. No operational Runtime/CLI wiring is supplied by this port.
Every foreign/stale claim and graph-corruption variant is not individually tested
here; lower-level controls remain in the full regression, and remaining acceptance
must not be inferred from these seven cases.

At final fixed clean Source2, macOS 26.6.2/25G83 arm64, Rust/Cargo1.91.1,
Git2.49.0, `RUST_TEST_THREADS=4`: **557 primary passed, 0 failed, 30 ignored**.
Workspace all-target Clippy `-D warnings`, build and fmt passed. Raw602 passes
exclude44 observer-child tests and one duplicate Codex child. The first restricted
sandbox control run failed before gate invocation because local Unix sockets were
unavailable; unchanged Source1 passed all three controls with fixture permissions.
Source1 full bounded regression also passed, but Clippy rejected the added test's
explicit-drop guard lifetime. Source2 uses a lexical guard scope and adds the
separate Store reopen control; final full checks passed. The initial failures are
retained, not credited as successful checks.

Two compiled mutations at Source1 remove dirty-namespace refusal and the native
work-success check. Each fails its actual Workflow test assertion (exit101), not
compilation. Restored three controls pass after both changes on the exact baseline
tree. Source2 changes tests only; all four related production files are byte-identical
to the mutated baseline. The mutation chain remains on the owned test branch;
no mutated code enters Phase2. Cleanup is best effort and this is not a security
sandbox. No Linux/current-install/native/default-unrestricted acceptance follows;
the earlier unrestricted source32 failures remain unresolved.

## Production gate claim and next integration checkpoint

Fixed clean source `a79a3051d41bc122e1e7e88cb7399c3cdfb83787` adds two actual
production-gate controls, covering eight altered invocation inputs and replay of a
closed claim. A independently approved the appended test source `99b71c38`; no
production bytes changed from the prior gate checkpoint. The parent read the real
assertion logs and verified all five delegated log hashes. Task/full-Context guard
omissions compiled and failed their intended assertions; restored controls passed
on the exact baseline tree. Closed replay is rejected by the earlier stale-owner
check and does not qualify a separate Evaluating-predicate omission.

Current bounded full checks passed: **559 primary passed, 0 failed, 30 ignored**;
all-target Clippy `-D warnings`, build and fmt also passed. Raw604 passes exclude
the same45 duplicate child passes. The [additive claim checkpoint](agent-execution-phase2-gate-claims-checkpoint.json)
preserves the delegated controls, mutations, raw review and exact parent checks;
the previous checkpoint is retained unchanged. No actual provider, four-Task,
Linux or unrestricted-parallel qualification is inferred.

The [Runtime/Scheduler design](../design/runtime-scheduler-integration-design.md)
and [ReviewEngine design](../design/review-engine-integration-design.md) passed
independent requirements/design reviews after corrections. The parent verified
missing restart Sources authority, inherited finite channel/cohort admission and
failure-content counting predicates against pinned actual consumers. Revised
private recovery and collective approval contracts close these design findings;
they do not establish available production ports. The
[integration design ledger](agent-execution-phase2-integration-design-reviews.json)
retains original findings and all re-reviews instead of dropping earlier failures.

The [native result supplement](../design/native-result-receipts-design.md) also
passed independent A/C static reviews. It specifies actual acknowledged input and
native identity, bounded answer acquisition, durable receipt and terminal updates
before watch publication, historical-only fenced drafts, and ordered schema6.
Schema6 is assigned to that implementation; existing source still uses schema5.
Published-frame recovery is assigned7 after Native6; future ReviewRound moves
to8. Fresh pre-input recovery remains a later integration, explicitly unsupported.
The local Codex-generated schema is protocol-shape evidence, not authenticated
model/native qualification. Review counting/member grants and Runtime construction
remain separate source work. These component approvals permit continued Phase2
implementation, not Phase2/MVP completion or Phase3 account use.

## Bounded raw JSON content checkpoint

Requirements/design `6bd0d9d` received C's independent approval before source.
Fixed clean source `bff8b27b72ae68c664764b6e51b549e860266bea` implements the pure
shared strict decoder; A and C independently approved source with no findings.
Original bytes are checked for duplicate decoded keys at every object, one complete
value and finite inclusive frame/depth/node/string/entry budgets. Errors retain
only categories. The normal Value it returns remains untrusted content.

Eight controls passed. Two compiled mutants omit duplicate and depth guards;
actual expected-error assertions fail, then all eight controls pass on the exact
restored tree. Mutation history is retained and the owned temporary worktree was
normally removed. At fixed clean source with four test threads: **567 primary
passed, 0 failed, 30 ignored**; workspace all-target Clippy `-D warnings`, build
and fmt passed. Raw612 passes exclude the same45 duplicate child passes. The
[checkpoint](agent-execution-phase2-strict-json-checkpoint.json) preserves raw
reviews, command/log hashes, source hashes and exact mutation/restore identities.

This component does not alter native/Review/Workflow/Store consumers. B's actual
native acquisition must invoke it before Value conversion, and ReviewEngine still
needs strict DTO/identity/semantic validation plus owned receipt/read-only/member
proofs. Parsed APPROVE never supplies approval authority. Both-OS/current-install,
actual native four-Task and unrestricted regression remain unqualified; earlier
unrestricted Source32 failures remain retained.

## Remaining Phase2 and acceptance work

- Qualified milestone content, verifier/reviewer/external gate consumers and
  Runtime orchestration/operational CLI construction of the new source/gate ports.
  Initial committed source, same-unit adoption and initial production observations
  are covered above; existing-Workflow/changed-input
  recovery still needs fresh provenance rather than reconstructed ownership.
- Required native/Git settings and hooks, dependency qualification and remaining
  scoped bootstrap/preparation helpers; full finite tool profiles and
  installed-binary tool mediation tests.
- Optional Linux scope, actual Docker qualification, filesystem/port release
  policy and complete crash/restart/legacy reconciliation.
- Full fixture/migration/fairness/permission/cancellation coverage, installation,
  both-OS CI and final independent coordinated STRICT source approvals.
- Phase3 genuine Claude/Codex four-Task, crash/escape, auth/settings/hooks and
  subscription waiting/recovery matrix, after explicit phase approval.

No work/cleanup guarantee, OS conformance, native readiness, release or MVP
completion follows from this checkpoint. Human approval is still required only
at the requested completed phase boundary; Phase2 work is already authorized.
