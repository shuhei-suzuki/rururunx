# Issue 51 design: Grok environment admission

Risk: STRICT. Proposed design, pending independent approval. No provider or Store
implementation is changed by this head. Requirements3 approved `713ccce`; its four
optional precision findings are adopted in the requirements and below. Source gates
must review the exact helper and its actual start/resume consumers before acceptance.

## Authority and availability

Grok keeps its intentional constructor baseline private and frozen. Selection never
reads another Project's environment values or files. `env_clear().envs(selected)` stays
the actual native child boundary; native cached auth, HOME, hooks and settings remain.
No new CLI flag, permission bypass, global redirect or per-Project credential loader is
introduced. The whitelist stays unchanged. Current #5/#6 source is a comparison input;
normally integrated reviewed behavior must be inspected before sharing its authority.

An ordinary caller value is eligible only when its exact name is declared by the current
owning Project, passes registry policy, is not a control, and is one of LANG, LC_ALL,
LC_CTYPE, TERM, COLORTERM or TZ. A native whitelist name additionally requires an exact
constructor-baseline value. An unsupported declared key does not become pass-through.
All caller RRX_ names are rejected, including previous fake ACP fixture channels.

Any retained non-control baseline name declared by another Project and not by the owner
causes opaque rejection before native spawn. Do not strip a credential and accidentally
select cached/default authentication. A shared declaration permits the single intentional
runtime value; there is no private/shared flag and no distinct credential-value routing.
The availability restriction includes any provider's references under Grok's NODE_,
BUN_ and OPENSSL_ whitelist prefixes. Private references used by Claude/Codex can make
the combined configuration unsupported. Operators must choose undeclared global native
auth or genuinely shared names/values and must not co-declare private foreign credentials.
This guidance is not an enforceable privacy marker. #16 still needs actual 2+ Projects,
4+ Tasks and all three native agents with a supported configuration.

## One registry predicate and one native control predicate

Extract the existing registry environment-name syntax and forbidden-name decision into
crate-private pure helpers in `project.rs`. Public `environment_names` keeps its existing
registered/source validation behavior. Native selection calls only the pure helpers:
no filesystem, Git, rule, config or native work under SharedStore or a SQLite transaction.
Own refs require complete validity and uniqueness; invalid own metadata fails admission.
Foreign records contribute individual syntactically valid names even when another entry
is invalid or duplicate. Include Registered, Blocked and Removed records. Their own
lifecycle does not revoke a retained credential name. Never validate foreign sources.

The Grok control predicate has these components, in precedence order:

1. Existing named identity/routing/shell/TLS/proxy controls admitted by baseline_key:
   HOME, PATH, TMPDIR, BASH_ENV, ENV, SHELL, ZDOTDIR, XDG_CONFIG_HOME, XDG_DATA_HOME,
   XDG_STATE_HOME, XDG_CACHE_HOME, NODE_OPTIONS, SSL_CERT_FILE, SSL_CERT_DIR,
   REQUESTS_CA_BUNDLE, CURL_CA_BUNDLE, HTTP_PROXY, HTTPS_PROXY, ALL_PROXY, NO_PROXY,
   and their already-whitelisted lowercase proxy spellings.
2. LD_/DYLD_ prefixes, plus any baseline_key name forbidden by the shared registry
   predicate. This includes NODE_PATH, NODE_EXTRA_CA_CERTS and every admitted *_PROXY
   spelling. Registry-forbidden proxy routing remains a control even if its URL could
   contain credentials: no valid Project can own such a reference.
3. Reviewed finite additions: NODE_TLS_REJECT_UNAUTHORIZED, OPENSSL_CONF,
   OPENSSL_CONF_INCLUDE, OPENSSL_ENGINES and OPENSSL_MODULES. These govern TLS checking,
   configuration or dynamic engine/provider loading; the sources below support that
   classification. They do not classify entire native prefixes as controls.

For registry-valid names, credential-bearing or locating names cannot be newly classified
as controls. XAI_API_KEY is a policy example, not a claim about installed Grok consumption.
Unknown valid GROK_/XAI_/NODE_/BUN_/OPENSSL_ names stay ordinary native references for
conflict detection. SSLKEYLOGFILE is not expanded into an implicit trusted control.
The same predicate rejects caller controls and protects baseline controls from foreign
declarations. Foreign invalid control refs neither strip nor replace the baseline.
Own control refs fail even where the registry itself would otherwise accept the name.

Primary runtime references are [Node command-line environment documentation](https://nodejs.org/api/cli.html#node_tls_reject_unauthorizedvalue)
and [OpenSSL environment documentation](https://docs.openssl.org/3.0/man7/openssl-env/).
They explain the selected runtime keys, not whether a particular Grok build consumes
every one. Installed native auth/config values are never read for this design or tests.

## Name-only snapshot and atomic pre-spawn boundary

Introduce a crate-private environment roster DTO containing sorted pairs of ProjectId
and the persisted raw `environment_refs` list. It contains no display names, roots, settings,
credential values or Project activity beyond those reference lists. Retain invalid and
duplicate strings for exact change detection; selection separately applies per-name
syntax/control rules. Snapshot all records, not just the current owner. New Project,
removed Project, or reference replacement is a roster change. A foreign display-name,
context-pointer or other non-reference metadata update is not an environment change.
The owning full Project/Goal/Task version guards still apply separately.

Proposed additive Store boundary is `put_session_with_environment_if_current` taking
the existing Session/version, expected P/G/T versions, exact WorktreeLock IDs/versions,
and the private roster DTO. Factor the existing `put_session_if_current` transaction
body into one private implementation with an optional roster check; the existing method
keeps identical behavior. In the same Immediate transaction, compare exact P/G/T versions,
ownership/activity, nullable scope, exact lock set and the current roster, then perform
the ordinary Session CAS/guards/audit. Any mismatch rolls back all writes. No schema
marker change or persisted global inventory record is required. Root and shared Store
owners must coordinate the exact additive helper before implementation; the full helper,
cross-connection fixtures and both provider consumers are source-review inputs.

At start, capture own ScopeSnapshot and name-only roster together under SharedStore.
Other Store connections are not excluded by that process-local mutex. Require the
roster's owning raw refs to equal the captured owning Project refs, select from this
roster, and rely on the final Immediate transaction for coherent admission. Inconsistent
read snapshots fail closed; a process-local lock alone is never a cross-connection fence.
Select private environment against that snapshot and reserve Starting using the existing
scoped Session boundary. Git/filesystem/profile preflight continues outside the lock.
Immediately before `command.spawn`, after all preflight, encode a bounded native-spawn
intent in Session recovery, check cancellation, and call the new atomic helper with the
captured roster and current owned Session version. A failed admission cannot start a
child, send ACP initialize/authenticate/load/prompt, or publish a consumed-input intent.
Environment errors are identical opaque bounded categories; do not include foreign
names/IDs, values, roster contents or detailed guard internals in A's error or audit.
The native-spawn intent records only this scoped operation, not environment data; it is
distinct from the existing authoritative model-prompt dispatch/consumed-input intent.

After successful admission, release the Store lock and spawn with the already-selected
map. SQLite admission and OS spawn are not atomic. A concurrent reference change after
admission does not retrospectively revoke the child's environment; disclose that window.
Existing pre-prompt and approval scope checks remain. Checkpoint's former full-Project
roster equality is narrowed to the exact reference roster, without weakening its own
P/G/T/lock checks; unrelated foreign metadata never grants or revokes environment
authority. If post-admission scope checks reject an admitted child,
retain factual spawn/audit outcome and verified cleanup or Lost uncertainty; never assert
that auth/config initialization did not run. A spawn intent does not establish model
completion or prove native process death. This fix never changes Lost reservation,
unknown dispatch, cancellation or recovery authority.

Resume captures current own refs and the full current roster again, selects against the
same frozen baseline, then uses the same last pre-spawn helper before initialize/auth/load.
Checkpoint updates PreparedInput only. Removed caller authority makes continuation fail
opaquely even if cached launch metadata once permitted it. Existing fresh input version,
same native UUID/private supervisor, verified death and no implicit replay gates remain.
Foreign roster updates after admission do not become permission-grant authority; native
permission checks keep their exact own scope/session/lock CAS.

## Operator diagnosis without a reverse leak

Provide an explicit state-only Project-scoped policy inspection method. It reports only
the selected Project's own referenced baseline names and unsupported/control declarations;
it reads no values and reports no foreign IDs, launches, counts or timestamps. Invoke it
only through an explicit operator check or independent inventory maintenance, never as a
side effect of another Project's blocked admission. A's error/audit remains opaque.

Explain two remedies: if the declaring source remains valid, use existing clear refs /
reactivation commands without skipping removal guards; if it is gone or otherwise cannot
reactivate, remove the variable from the intentional runtime environment and reconstruct
the adapter/runtime. Do not clear state automatically, rewrite another Project or fall
back to cached authentication. A new public CLI surface is optional future integration;
the explicit scoped inspection contract and regression are required here.

## Producer inventory and fixtures

Current source inventory finds caller environment in WorkflowEngine's supplied map and
Grok's direct LaunchRequest consumers. No production RRX_ producer was found at this
head; any later integrated producer reopens requirements rather than adding an exception.
`crates/rrx/tests/grok.rs` currently carries RRX_DATABASE, RRX_FOREIGN, RRX_MODE,
RRX_SPAWN_OBSERVED, RRX_PAUSE, RRX_PROMPT_OBSERVED and RRX_EXPECT_INPUT. They are fixture
metadata, not production authority. Move them into generated executable-owned configuration
next to each fake ACP script. The fixed native argv remains unchanged. Each invocation
loads its private configuration at startup; mode/input changes for a later continuation
update the fixture file before its new owned process starts. Preserve existing actual
Store/preflight/permission/continuation tests rather than deleting failing consumers.

New canary fixtures re-execute a dedicated test-binary child under env_clear with known
synthetic HOME/PATH/tool paths, intentional native keys and private TempDir. The child
constructs the real GrokAdapter with `new`; no ambient set_var, copied selector or actual
native config/auth reads. Fake ACP capture compares only known synthetic marker values.
Unexpected values fail before persistence/hash/printing; mismatch messages never show
actual values. Historical observation is a transcription, not a reproducible raw byte
capture or binary-revision attestation. New build/source provenance is labelled honestly.

## Required causal verification

- Actual start and fresh-checkpoint/resume negatives: registry-valid foreign TZ caller,
  undeclared ordinary key, reserved RRX_, own control, foreign retained baseline reference,
  and mixed invalid foreign refs containing a valid conflicting name. Compare opaque
  foreign/undeclared errors without exposing names. Assert no spawn marker first.
- Actual positives: own TZ, own/shared XAI_API_KEY synthetic exact runtime value,
  undeclared global native auth, protected baseline unchanged despite invalid foreign
  control declaration, absent foreign baseline name, and foreign non-reference metadata
  update. Preserve actual ACP identity/lifecycle rather than synthesize exit0.
- Per-invocation private hooks after snapshot and immediately before final admission use
  a second Store connection to mutate own refs or register/replace a foreign reference.
  Assert no child/native wire/consumed-input intent and rolled-back admission; unchanged
  controls prove this exact prepared path starts. Cover reference changes between
  checkpoint and resume and between its new snapshot and pre-spawn transaction.
- Store tests independently prove cross-connection new/deleted/replaced refs are seen,
  unchanged foreign metadata is eligible, and failed roster CAS leaves Session/audit
  unchanged while existing P/G/T/lock/activity/Blocked/Lost guards remain effective.
- Compiled actual-consumer mutants remove caller ownership, baseline conflict and final
  roster check individually. They must fail on child canary/spawn state; helper-only,
  masked, uncompiled or irrelevant failure receives no consumer credit. Restore exact
  source/control, kill/reap owned groups before leader reap, then normal worktree cleanup.
- Immutable two independent design/source gates, verified fixes/rereviews, commit before
  scoped tests and required default debug/release workspace, fmt/all-target clippy/build,
  exact Linux/macOS CI. Preserve unrelated #46 timeouts/Unknown/Lost and failed CI;
  do not relax budgets/latches or serialize acceptance. Installed native #16 acceptance
  remains separate from fake ACP environment wiring.

## Implementation and integration order

After approved design: pure predicates and native selector; coordinated additive Store
helper plus second-connection regressions; actual start/resume wiring and fixture migration;
causal mutants/restored controls; immutable source reviews; final evidence/master/README
and exact final CI. Issue46 shared cleanup has priority. Normally integrate reviewed
shared helpers and native6 updates without overwriting peer authority or force-pushing.
Root alone merges/closes. No ACP/tool/approval/filesystem, factory/TUI, global credential
extraction, schema or native sandbox change belongs to this issue.
