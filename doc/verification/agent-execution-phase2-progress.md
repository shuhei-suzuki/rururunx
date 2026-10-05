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

- Full current-source/retained-graph qualification and production source/evidence
  ports; Runtime orchestration and operational CLI.
- Required native/Git settings and hooks, dependency qualification and remaining
  scoped preparation/retained-inspection helpers; full finite tool profiles and
  installed-binary tool mediation tests.
- Optional Linux scope, Docker label reconciliation, filesystem/port release
  policy and complete crash/restart/legacy reconciliation.
- Full fixture/migration/fairness/permission/cancellation coverage, installation,
  both-OS CI and final independent coordinated STRICT source approvals.
- Phase3 genuine Claude/Codex four-Task, crash/escape, auth/settings/hooks and
  subscription waiting/recovery matrix, after explicit phase approval.

No work/cleanup guarantee, OS conformance, native readiness, release or MVP
completion follows from this checkpoint. Human approval is still required only
at the requested completed phase boundary; Phase2 work is already authorized.
