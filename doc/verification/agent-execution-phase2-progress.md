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
