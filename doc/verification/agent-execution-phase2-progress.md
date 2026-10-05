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

- Workflow/Context publication and full current-source CAS; public adapter and
  Registry integration; Task/reservation/Runtime orchestration and operational CLI.
- Required native/Git settings and hooks, dependency qualification and remaining
  scoped preparation/retained-inspection helpers; full finite tool profiles and
  installed-binary tool mediation tests.
- Cookie/platform cleanup, optional Linux scope, Docker label reconciliation,
  safe cleanup backlog and complete crash/restart/legacy reconciliation.
- Full fixture/migration/fairness/permission/cancellation coverage, installation,
  both-OS CI and final independent coordinated STRICT source approvals.
- Phase3 genuine Claude/Codex four-Task, crash/escape, auth/settings/hooks and
  subscription waiting/recovery matrix, after explicit phase approval.

No work/cleanup guarantee, OS conformance, native readiness, release or MVP
completion follows from this checkpoint. Human approval is still required only
at the requested completed phase boundary; Phase2 work is already authorized.
