# Issue 51 design: Grok environment admission

Risk: STRICT. Proposed design, pending independent approval. No Issue51 provider or
Store implementation exists; reviewed main Issue46 is normally integrated. Requirements7
approved `1688a85`, preserving Requirements5's explicit credential prohibition and
Requirements6's SSLKEYLOGFILE/fresh-decision/checkpoint ordering clarification.
The earlier optional precision findings are adopted in the requirements and below. Source gates
must review the exact helper and its actual start/resume consumers before acceptance.

## Authority and availability

Grok keeps its intentional constructor baseline private and frozen. Selection never
reads another Project's environment values or files. `env_clear().envs(selected)` stays
the actual native child boundary; native cached auth, HOME, hooks and settings remain.
No new CLI flag, permission bypass, global redirect or per-Project credential loader is
introduced. The whitelist stays unchanged. Current #5/#6 source is a comparison input;
normally integrated reviewed behavior must be inspected before sharing its authority.

A caller name is eligible only when declared by the current owning Project, registry-
valid and non-control, and either (a) LANG, LC_ALL, LC_CTYPE, TERM, COLORTERM or TZ with
its own ordinary value, or (b) a native-whitelisted name with exact constructor-baseline
value. These are alternatives; native keys need not be ordinary. An unsupported declared key does not become pass-through.
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
3. Reviewed finite addition: NODE_TLS_REJECT_UNAUTHORIZED, a TLS certificate-checking
   control documented below. It is not a credential value or credential-location key.
   Do not classify entire native prefixes as controls. Registry-valid credential-bearing
   or locating keys can never enter the additions; a drift test retains examples of
   native credential and ambiguous native-reference names as non-controls.

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
every one. OpenSSL describes config/module path overrides; their existence alone does
not grant a registry-valid name protected-control status. No additional OPENSSL_ name
is newly trusted here: absent foreign references the existing global baseline remains,
while declared non-control names obey the same conflict rule. Installed native auth/config
values are never read for this design or tests.

## Name-only selection and atomic pre-spawn boundary

Keep a private frozen-baseline name set and bounded caller-key set for environment
admission; no values, foreign identities, Project versions or raw roster are persisted
or emitted. The selected environment remains private. Remove ScopeSnapshot.projects: capture/checkpoint/Actor.owner read only own Project/
Goal/Task/lock authority, never fully decode or retain foreign Project bodies/roots/
display names. Launch/resume selection alone obtains a temporary environment_refs
projection from persisted Project JSON; use a bounded per-name projection, not
store.projects() full Project decoding. Malformed unrelated foreign fields do not block
A. Malformed/missing reference projection has no reference authority; valid string
entries in mixed lists still contribute per-name under the existing rule;
foreign record order, identities and versions are not authority tokens. Final admission
reevaluates the same pure environment decision against current references inside SQLite.
Only an actual relevant conflict denies A. Registering/removing/replacing unrelated names,
foreign control-only changes, lifecycle or non-reference metadata must remain eligible
and must not produce an A error/audit correlated with those changes. Own P/G/T versions
remain exact independent guards. Include all foreign lifecycle states and per-name
validity in this evaluation, with no foreign filesystem/config/Git work.

Proposed additive Store boundary is `put_session_with_environment_if_current` taking
existing Session/version, expected P/G/T versions, exact WorktreeLock IDs/versions and
one crate-private bounded EnvironmentAdmission DTO of frozen non-control baseline names
and caller names. Pure policy constructors enforce syntax/control classification and
bounds; no environment values or mutable callback enter the transaction. One shared
value-free pure name-decision function serves initial selection and final transaction,
with preclassified baseline/caller names and shared registry pure helpers; Store has
no dependency on Grok control functions. Only initial selector compares actual native
values with the private frozen baseline. Never duplicate caller/conflict logic. Removing
only in-transaction own-ref validation is equivalent under exact own-Project CAS and
earns no mutant credit; shared decision operators and transaction-call omission supply
the meaningful targets. Factor existing
`put_session_if_current` into one private transaction body with an optional pure
environment decision. Existing consumers retain identical behavior. In the same Immediate
transaction, validate exact own versions/ownership/activity/nullable scope/lock set,
validate expected Session version/ownership/activity/exclusion before the
environment decision so existing stale Session guards retain precedence,
fully validate current owning refs, read current foreign refs name-only, reevaluate
caller ownership and foreign retained-baseline conflicts, then perform ordinary Session
CAS/guards/audit. A relevant conflict returns one fixed opaque typed guard without
foreign IDs, names, values or inventory/version details. All writes roll back on failure.
No schema change or global inventory record. Coordinate helper with shared Store owners
before implementation; full helper and actual consumers enter source reviews.

Initial selection and Starting reservation preserve current authority. Git/filesystem/
profile preflight stays outside Store. At the last state boundary before spawn, Actor's
existing capture-based Actor.owner() remains immediately before the new private
`admit_environment`; preserve its non-executor reserved-Session checks, with a Task-
Consultant executor-reserved-during-preflight no-spawn regression. `admit_environment`
takes entry.transition and follows Actor.publish's exact
ownership protocol: build candidate from its current session without mutating private
state first; check stopping; execute full atomic helper using current Actor.version and
captured own authority; on success update Actor.version and retained watch Session to
that exact saved candidate while transition is held. On failure keep previous private
version/session/watch unchanged. This Session CAS is the admission fence; no new
native-spawn recovery intent or audit schema is added. Ordinary Session.saved records
this exact scoped write and retains its existing field set; prior model-prompt
consumed-input/dispatch intent remains unchanged. Admission is neither prompt consumption
nor completion nor process-death evidence.

After releasing transition/Store, check entry.stopping again immediately before
command.spawn with no intervening await. A stop recorded after admission must not start
a child; a deterministic hook tests this exact boundary. Stop/entry authority cannot
observe a mismatched persisted Session and private/watch snapshot. SQLite admission and
OS exec are not atomic; a reference mutation or stop after this last check is an
explicit non-retroactive window, not atomic revocation. Successful spawn keeps existing
factual process_spawned audit and normal supervision. Post-admission scope checks may
reject/stop an admitted child; retain factual native outcomes and verified cleanup or
Lost uncertainty, never claim initialize/auth/config did not run. No Lost reservation,
unknown dispatch, cancellation or recovery authority is weakened.

Checkpoint refreshes current own Project under immutable identity/full own authority
and explicitly higher PreparedInput version, as current source does; remove its foreign
whole-Project roster comparison entirely. Actor/pre-prompt/approval checks continue exact
own scope guards. Resume uses that explicit checkpoint's Project snapshot, captures
current own authority and reevaluates live references during the same fresh pre-spawn
admission. If own metadata changed after checkpoint, preserve current ScopeSnapshot's
stale-Project StateConflict and require another explicit fresh checkpoint. If refs were
removed before latest checkpoint, selector/admission fails opaquely. No implicit Project
refresh or cached-authority exception. Fresh input, same private native UUID, verified
death and no implicit prompt replay remain authoritative.

### Error precedence and structural drift

Malformed/duplicate/forbidden own refs follow owning configuration rejection before
selection; existing malformed/GIT_ input remains InvalidInput. Reserved RRX_, caller
controls and unsupported caller names are InvalidConfiguration, regardless of identical
baseline value. Eligible caller names use an OR: one of the six own ordinary names,
or an own native-whitelisted non-control name with exact constructor-baseline value;
ordinary values need not equal that baseline. A declared unsupported name not actually
passed by this provider is metadata for other providers, not a blanket own-Project
rejection. Undeclared/foreign-only eligible-shape caller and baseline conflict, including
final transaction conflict, share the same fixed opaque InvalidConfiguration category.
Add fixed-display EnvironmentAuthority guard; map it to InvalidConfiguration in BOTH
adapter.rs::state_error (used by Grok save_current/assert_saved) and
grok/ownership.rs::state_error, with the same fixed message and no foreign IDs. Actual
async consumers must assert that category. Do not expose SnapshotChanged foreign IDs.
Own stale Project/parent/Session/lock/lifecycle errors retain current precedence/kinds.

Structural tests assert finite additions exactly {NODE_TLS_REJECT_UNAUTHORIZED}, every
named protected baseline key registry-forbidden, and the registry-valid named baseline
exception SSLKEYLOGFILE non-control. XAI_API_KEY, valid GROK_*, OPENSSL_CONF and BUN_OPTIONS
remain non-controls. Syntactically invalid admitted baseline names are unownable global controls, with the
same precedence as registry-forbidden names; preserved constructor values cannot be
replaced by callers and invalid foreign strings grant/conflict with no reference.
Explicit bounded name-admission failure is opaque InvalidConfiguration, never truncation:
max512 retained names, each max256 UTF8 bytes, aggregate max64KiB; enforce in new()
before retaining a baseline, never log offending names/values. DTO caller bounds match
existing max128names/max256bytes per name. This resource bound does not widen whitelist
or pass-through; oversize runtime configuration is unsupported/fails closed. Boundary
tests assert rejection and no silent dropped conflict. Existing undeclared valid native
baseline remains intentional global runtime input. Assert unchanged
native whitelist separately; membership never implies protected status.

## Operator diagnosis without a reverse leak

Provide an explicit state-only Project-scoped policy inspection method. It reports only
the selected Project's own referenced non-control baseline candidate names;
it reads no values and reports no foreign IDs, launches, counts or timestamps. Invoke it
only through an explicit operator check in the runtime that owns the frozen baseline, never as a
side effect of another Project's blocked admission or inventory/registration change.
Registered, Blocked and Removed Projects all use pure state reads, without calling
public environment_names that requires a registered source/FS. Its candidate set is
derived only from that Project's own declarations and
the retained baseline name set, independent of other Projects' co-declarations. A's
error/audit remains opaque.

Explain two remedies: if the declaring source remains valid, use existing clear refs /
reactivation commands without skipping removal guards; if it is gone or otherwise cannot
reactivate, remove the variable from the intentional runtime environment and reconstruct
the adapter/runtime. Do not clear state automatically, rewrite another Project or fall
back to cached authentication. A new public CLI surface is optional future integration;
the explicit scoped inspection contract and regression are required here.

## Producer inventory and fixtures

Current WorkflowEngine.step(task_id, environment) accepts a caller-supplied map and
forwards it unchanged into LaunchRequest; it does not resolve refs or construct a native
provider-safe map (workflow.rs750/753/998). Direct Grok consumers also supply maps.
Current CLI has no LaunchRequest producer. Shared Generic fixtures use caller PATH
(adapter.rs1775/1795 and1830/1902); those remain Generic-only. Grok in-crate unit requests
currently use empty maps. Both installed acceptance tests use external Fixture::new
and currently retain RRX_DATABASE/RRX_FOREIGN; their ignored status does not excuse
breakage. Migration must remove all reserved metadata from that shared request, including
installed tests, with a nonignored empty/reserved-free caller-map assertion. Preserve the
Generic contract separately: callers supply its intentional baseline; Grok derives its
immutable control/auth baseline in new() and rejects caller HOME/PATH/config controls.
Add actual WorkflowEngine-to-Grok forwarding coverage with own-declared ordinary inputs
and unsupported caller controls; do not claim the engine builds or filters them.

Compatibility impact: reusing one Generic HOME/PATH/config baseline map on a Grok phase
now fails even for identical values; stricter control rejection remains required. The
current step API accepts a new map each invocation, and public snapshot()/Phase.actor()
allow a caller to prepare a phase-specific map; it does not mandate one fixed map for
an entire mixed-provider Task. Such caller preparation is not an atomic provider-bound
input token or an implemented runtime driver. A driver reusing one baseline map across
providers is unsupported. Add a real Generic-executor/Grok-reviewer forwarding negative:
Grok rejects InvalidConfiguration before any native reservation/child; Workflow already
created its attempt reservation/dispatch_started marker and its start-error branch
returns StepResult::Failed with no Session binding. Assert that actual outcome, never
claim a typed AdapterError is returned directly by engine.step or that no Workflow
reservation existed. A fresh appropriate per-phase map has a separate forwarding
positive: assert actual StepResult::Started/native Session binding, not complete native
review or transport success. Current unchanged-Task binding increments raw Task version
and can invalidate the running provider (#43); full mixed native Workflow completion
remains blocked on that independently reviewed binding integration. Do not weaken
provider CAS or claim this forwarding positive closes #43.
Future runtime/provider-aware driver integration must reconcile normally merged #5/#6
caller contracts before #16; native constructor-owned baseline plus empty/eligible
ordinary maps may avoid this Generic-specific conflict, but no native3 support is
inferred until actual integration proves it. No production RRX_ producer was found at this
head; any later integrated producer reopens requirements rather than adding an exception.
`crates/rrx/tests/grok.rs` currently carries RRX_DATABASE, RRX_FOREIGN, RRX_MODE,
RRX_SPAWN_OBSERVED, RRX_PAUSE, RRX_PROMPT_OBSERVED and RRX_EXPECT_INPUT. They are fixture
metadata, not production authority. Move them into generated executable-owned configuration
next to each fake ACP script. The fixed native argv remains unchanged. Each invocation
loads its private configuration at startup; mode/input changes for a later continuation
update the fixture file before its new owned process starts. Preserve existing actual
Store/preflight/permission/continuation tests rather than deleting failing consumers.

Define cfg(test)-only per-adapter/per-Actor awaitable hooks: before admission runs
after final preflight but BEFORE entry.transition acquisition; after admission runs
after transition/Store release BEFORE final stopping load; checkpoint runs after
capture/verify_git BEFORE recheck and its Session CAS; retained invocation-private handles only,
not process-global state, public runtime fields, environment or argv channels. Put new
actual-consumer tests in Grok in-crate unit module so cfg(test) hooks are visible (integration
binaries compile the library without cfg(test)). Reuse fixture-owned fake ACP helpers
there. Move the fake ACP source into one shared fixture file included by both in-crate
and integration tests; integration-crate constants cannot be imported by library tests.
Use file-backed Store/second connection rather than the shared memory-only preflight
fixture. Current-thread Tokio hooks must never synchronously wait on the test task. The
after-admission hook coordinates concurrently spawned adapter.stop(), waits only for
the stopping flag notification, then lets Actor publish terminal; awaiting stop completion
inside Actor would deadlock. Assert actual stop returns Stopped, not Timeout, plus no
spawn and coherent Actor/Store/watch. No-await applies only final stopping load to spawn.
Keep external tests and migrate metadata to sibling JSON. Hooks run outside SharedStore and never accept callbacks inside the SQLite transaction.

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
  control declaration, absent foreign baseline name, and irrelevant foreign registration/deletion/reference/control/lifecycle/non-reference
  update. Include own/shared/foreign SSLKEYLOGFILE exact-value cases as non-control.
  Inject every irrelevant change inside that actual consumer's own window: start capture
  to admission, checkpoint capture to Session CAS, resume capture to new admission.
  Compare owning A scoped event kinds/counts against the unchanged-reference control,
  ignoring identities/timestamps and preserving normal operation audit. One compiled
  raw-roster-equality operator per boundary must die on that actual-consumer positive. Preserve actual ACP identity/lifecycle rather than synthesize exit0.
- Per-invocation private hooks after snapshot and immediately before final admission use
  a second Store connection to mutate own refs or register/replace a foreign reference.
  Assert no child/native wire/new-version consumed-input intent and rolled-back admission;
  for resume preserve prior dispatch_intent/prompt_id/dispatch_state byte-for-byte and
  assert no new input_version intent, rather than requiring all historical intent absent; unchanged
  controls prove this exact prepared path starts. Cover own reference changes before checkpoint (opaque resume denial), after checkpoint
  (stale-Project StateConflict until another checkpoint), and between new snapshot/admission.
  After-admission hook requests stop and asserts no spawn, coherent Actor/Store/watch
  versions, unchanged prompt intent; no Session death inferred from stop alone.
- Store tests independently prove cross-connection new/deleted/replaced refs are seen,
  irrelevant foreign roster changes are eligible, and relevant decision conflict leaves Session/audit
  unchanged while existing P/G/T/lock/activity/Blocked/Lost guards remain effective.
- Compiled actual-consumer mutants remove caller ownership, baseline conflict and final
  fresh environment decision individually. They must fail on child canary/spawn state; helper-only,
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
