# Issue 6: existing name-only environment admission integration (Design7)

Status: proposed STRICT component integration; design/source qualification pending.
Design1 `6264702`, Design2 `9884ec3`, Design3 `4a60fa2`, Design4 `e85d85b`
Design5 `55369e9` and Design6 `f0fc37c` were not qualified as pairs.
No Rust implementation has begun.
This proposal normally composes main `768f84319cd2a73e14cd39336eb12d99e9be81a7`.
Production Codex availability stays
EMPTY. No native workload backend, managed operation, setup/settlement receipt,
custodian or native-origin provenance producer is introduced.

## Existing authority and actual consumers

Merged Issue51 requirements 1–8 supply the existing policy: bounded names-only
environment selection, a last current Session/environment check before native exec,
own-only checkpoint without environment authority, and opaque rejection without
credential stripping.
Consume `EnvironmentAdmission::new`, `Store::check_environment_admission` and
`Store::put_session_with_environment_if_current`. The DTO contains non-control
baseline names and caller names, never values. The Store streams current foreign
`environment_refs`; it does not decode foreign Project bodies or read their files.
The transaction preserves exact own P/G/T, Session and nullable lock guards.
No shared Store/Grok/Generic/registry/schema implementation changes are planned.

Current Codex consumers are:

- `CodexAdapter::new` and `owned_handles`; the current baseline is instead captured
  later by `prepare_launch` using `vars_os` after Starting/Git preparation.
- `ScopeSnapshot::capture/recheck/recheck_scope`; capture currently retains full
  `Store::projects()` and ordinary recheck compares the complete roster.
- `policy::native_environment/provider_environment`; they strip foreign-declared
  retained credentials and restore arbitrary runtime values named by effective
  selected-provider config without authoritative origin evidence.
- `prepare_launch`: the selected environment reaches three distinct execs:
  native `--version` through `preparation::bounded_git`, discovery through
  `NativeServer::launch_preparing`, and the policy/main-server re-exec through that
  same launcher. Auth, history load and hooks/MCP can run before `turn/start`.
- `Reservation::admit_dispatch` publishes consumed input. `prepare_checkpoint`
  publishes fresh input and replaces the stored request without a native exec.
- `answer_approval`, observations, denial/cancellation, terminal cleanup and
  rollback use already-owned child/own-scope authority; they do not select a new
  credential environment. No foreign environment decision belongs on those paths.

The current caller-map producer is `WorkflowEngine::step/prepare_agent`, which forwards
its map unchanged into `LaunchRequest.environment`; direct public callers can also
supply that map. Current Codex component fixtures construct requests directly;
policy tests exercise both selectors. There is no ready native runtime/CLI producer.
Workflow's earlier context/reservation writes and managed ownership composition
remain separate open gates; this work does not alter Workflow production.

## Exact frozen selection and compatibility impact

Route public `new()` and fixtures through one private constructor consuming an
`(OsString, OsString)` iterator. Production supplies `vars_os()` exactly once;
fixtures supply only isolated synthetic pairs before that same filter/classifier/
bounds/freeze. Do not construct over ambient secrets and overwrite the result.
Keep values private in native OS-string form and share the same immutable snapshot
with owned handles. Frozen/selected containers and CodexAdapter implement NO Debug,
Display or Serialize.
Tests use match/.err()/boolean kind-message checks, never unwrap_err requiring an
Ok-value Debug or value-bearing assertion diagnostics. No process-wide environment
mutation, credential extraction,
different-per-Project native value routing or API-client auth substitution.

Preserve ordered membership: discard non-UTF-8 names and ALL GIT_* names before
applying the whitelist, prefixes or suffix; never compare a lossy name. Baseline
membership then retains the current Codex whitelist: HOME, PATH, SHELL, LANG,
TERM, TMPDIR, TEMP, TMP, NODE_OPTIONS, NODE_PATH, SSL_CERT_FILE, SSL_CERT_DIR,
NODE_EXTRA_CA_CERTS, OPENAI_API_KEY, OPENAI_BASE_URL, OPENAI_API_BASE, XAI_API_KEY,
ANTHROPIC_API_KEY, SSH_AUTH_SOCK, SSH_ASKPASS, EDITOR and VISUAL; plus LC_, XDG_,
CODEX_ prefixes and case-insensitive *_PROXY suffix. Preserve that membership
exactly; USER/LOGNAME remain excluded, with no synthetic missing value. Issue51
requirement 5 and current policy provide no native need for widening it. Historical
native auth without those names is supporting evidence only, not complete native
compatibility. The earlier optional identity delta is deferred outside this scope.

The exact private control predicate is baseline membership AND (invalid registry
name OR registry-forbidden name). Admitted syntactically invalid baseline names
remain unownable global controls, preserved privately and excluded from the DTO.
Do not reject an entire host for a name that no valid Project can own. Foreign
control declarations cannot strip or replace the baseline. Reject owning control
declarations when this provider's membership predicate admits the name, including
when its value was absent. Valid own metadata outside Codex membership is not a
blanket rejection. Registry-valid credential-bearing or credential-locating names
never become controls. HOME, CODEX_*, SSH_AUTH_SOCK, SSH_ASKPASS and *_PROXY are
registry-forbidden global controls; OPENAI_API_KEY, XAI_API_KEY and ANTHROPIC_API_KEY
remain non-control. The reviewed finite additional-control set is exactly EMPTY;
assert it structurally. Registry-valid LC_* and XDG_* names, including XDG_CONFIG_DIRS
and XDG_DATA_DIRS, retain the previously stated non-control classification. They
may select native config/data search paths, but this component never changes or
restores their frozen values; caller non-ordinary keys must match exactly. Foreign
references can refuse a NEW exec, not strip/change a running child's global routing.
This explicit availability coupling is the existing51 conservative subset, not
complete native-config isolation. No new prefix/control eligibility policy is added.
Any future widening needs separate qualified impact; unknown native readiness stays
OPEN. With EMPTY additions, own/control, caller/control and reference/control/GIT_
clauses are subsumed by registry validation: structural duplicates, no independent
mutant credit. The classifier has distinguishing effect on constructor DTO control
exclusion. Assert this equivalence structurally and reopen impact/mutation planning
if additions change. Use that single classifier consistently.

Caller eligibility is the existing Issue51 OR rule: registry-valid, owning-declared,
non-control, and either one of LANG/LC_ALL/LC_CTYPE/TERM/COLORTERM/TZ with its ordinary
value, OR a native-whitelisted key with the exact intentional frozen value. The
ordinary alternative takes precedence and need not equal the baseline. Unknown or
different credential values are explicitly refused; no caller introduction of an
absent non-ordinary native key. The DTO is constructed from every retained
non-control baseline name and every caller name, using the existing 512-name/256-byte/
64KiB baseline and 128-name caller bounds. Enforce bounds in the common constructor
before retaining a usable baseline; never truncate a relevant authority projection.

EVERY retained non-control name participates in the foreign conflict decision,
including LANG, TERM, registry-valid LC_* and XDG_* names, and all three API keys;
the rule is not credential-specific. Any Project, provider or lifecycle can declare
such a name. A foreign-only LANG declaration therefore refuses non-declaring Codex
Projects when LANG is frozen, just as a foreign-only ANTHROPIC_API_KEY does. Do not
reclassify ordinary keys as controls or silently strip them. Supported configurations
keep genuinely global names undeclared, genuinely share the same frozen reference
among affected Codex Projects, or remove the variable from the runtime environment
and reconstruct the adapter. Never co-declare a private credential as a workaround.
Ordinary own scoped caller values can differ under the existing OR rule. This is
the availability limit of Issue51 requirements 2/4/5, not complete per-Project value
routing or waiver of the required multi-Project/native-agent acceptance.

Add explicit UNGATED, pure own-state metadata inspection
`CodexAdapter::environment_candidates(project)` using
`Store::environment_candidates` and this frozen baseline's non-control DTO with
empty caller names. It returns only the requested Project's declared candidate
names, for Registered/Blocked/Removed states; no values, foreign query or filesystem
read. It is never invoked from another Project's rejection. Invalid/unknown own
metadata retains the Store's opaque failure; the method grants no launch authority.
Ordinary EMPTY gates all operational launch/approval/checkpoint routes, not this
bounded synchronous names-only Store inspection; master section 16 must list the
exception explicitly. It runs no Git/native child and reads no foreign inventory.

As in Issue51, validate the request's own references/control declarations before
caller-map syntax. Traverse the caller BTreeMap in lexical key order; for each key,
check malformed/GIT_/NUL syntax (InvalidInput), then supported non-control shape/value
(opaque InvalidConfiguration). As in the actual Grok loop, DO NOT check declaration
in that loop: after all keys pass, construct the bounded caller DTO, then the Store
decision verifies current own declarations/conflicts. Thus undeclared ordinary
COLORTERM followed by GIT_DIR yields InvalidInput; A_UNSUPPORTED followed by GIT_DIR
yields InvalidConfiguration. Eligible maps exceeding the 128-name/256-byte/32KiB
caller DTO bounds fail opaquely AFTER the per-key traversal; a malformed later key
wins before that bound. Test these distinguishing combinations and both sort orders.
A combined invalid-request-own plus malformed caller uses the opaque own-reference
InvalidConfiguration first. Unsupported/RRX_/control/different credential values,
undeclared/foreign-only eligible callers, current invalid refs and baseline conflicts
use fixed opaque InvalidConfiguration, `native environment authority unavailable`,
including Store EnvironmentAuthority mapping. Never emit foreign IDs, names,
versions, values or inventory. Existing own stale/lifecycle/Session/lock checks
retain their kinds and precede selection when they observe the conflict.
The initial read-only scope check and environment check are separate SQLite reads,
not a new transaction: an own edit between them can be observed as an opaque
environment refusal. That refusal remains before effects/publication; do not claim
one atomic initial snapshot or deterministic stale-kind precedence across that race.
Each later pre-exec/consumed CAS retains the existing transactional own-version
precedence; explicit own edits at those windows must be StateConflict.

Intentional changes and test migrations:

| Current Codex behavior | Proposed outcome |
| --- | --- |
| Baseline captured after preparation at each launch | Frozen once by the common constructor; handles and continuation reuse it |
| Foreign-only retained non-control name silently stripped | Opaque refusal before the relevant new exec, including LANG/TERM/LC_*/XDG_* and API keys; no default/auth substitution |
| Any valid owning-declared arbitrary caller value passed | Existing51 ordinary OR exact frozen native value rule; unsupported/different values refused |
| Full foreign Project decode/equality | Own Project equality, Goal/Task/locks remain exact; foreign refs alone determine environment policy |
| Effective config restores an arbitrary runtime key/header | No value restoration; reference predicate before discovery/main policy checks; combined-invalid reference precedence and main non-object ParseFailure |
| Own invalid refs return InvalidInput; undeclared caller returns OwnershipMismatch | Fixed opaque InvalidConfiguration |
| Malformed/GIT_/NUL caller returns OwnershipMismatch | InvalidInput after own-reference validation; combined-invalid order is explicit |
| USER/LOGNAME omitted | Remain omitted; no identity whitelist/control delta |
| Public policy::native_environment and private provider_environment | Removed; one private frozen selector/classifier and pure reference predicate replace them, with no roster parameter |
| Missing/null selected-provider entry yields zero refs | Same zero-reported-ref outcome; no inferred/default/built-in conformance |
| Empty/over-128-byte selected id or non-object provider/settings can yield zero refs | Explicit Unsupported identifier bound or ParseFailure shape refusal; conservative unqualified-default limitation |
| Public new() never captures environment | Frozen constructor selection can fail opaquely on unsupported bounds; successful construction still leaves ordinary availability EMPTY |
| No extra pre-exec Session publications | Up to three session.saved writes/version increments per fresh/resume attempt; main pre-exec publication also clears verified-dead discovery PID; no new consumption/dispatch intent |

Remove the public `rrx::codex::policy::native_environment` helper and its private
`provider_environment` restoration helper; no compatibility strip/pass-through shim
or second whitelist remains. This is an intentional pre-1.0 public API break.
At the read-only publication check
(2026-10-05), first-party crates.io/api/v1/crates/rrx returned 404; repository has no
Git tags/release-publish workflow, and README documents local --path installation.
The 0.1.0 manifest remains publishable, so do not claim publish=false or published
API compatibility. Require a README migration note in the source PR; before any
future registry release containing this removal, a 0.2.0-or-higher breaking minor
release/version qualification is mandatory. No publishing/version mutation is part
of this component. Current
workspace source consumers are prepare_launch, provider_environment and policy unit
tests; repository CLI/examples/other providers have no call to this Codex helper.
The separate shared git::native_environment is unchanged and not this API. Future
external consumers must migrate rather than retain the rejected roster/stripping path.
Retarget old policy structural tests to the one private selector/classifier; record
the source/API inventory and ensure no second definition survives.
Old strip/pass-through/custom-reference policy assertions migrate to these declared
outcomes. Existing own currency/cancellation, native
defaults/auth/hooks/config, consumed-input/Lost facts and shared cleanup semantics
retain their current purpose. All eighteen unit-test new() calls in session.rs
must migrate to the common injected iterator constructor, including ordinary
unavailable tests; no test captures ambient credentials then overwrites a baseline.
The two external integration calls in tests/codex_unavailable.rs are already inside
an env_clear re-executed child. Supply synthetic HOME/PATH and an owning-declared
OPENAI_API_KEY marker; an ungated own candidate inspection must report that key after
real public new(), proving the production iterator wiring. In a distinct child, one
registry-valid LC_ member over 256 bytes makes new() fail opaquely at construction,
while its bounded matching control constructs successfully. Before either constructor,
check the exact permitted synthetic OS-name/value pairs with boolean-only failures;
never capture an unknown value or print one. A new()->empty-iterator mutant is killed
by the candidate assertion, and lazy capture/bounds mutants by constructor refusal.
No example or production factory currently constructs CodexAdapter.
Ordinary EMPTY behavior remains, but this construction migration is intentional.

Current caller producer inventory: Workflow prepare_agent forwards its input map
unchanged. Codex ownership::Fixture creates an empty LaunchRequest.environment;
all session fixtures inherit that empty map. Policy unit tests alone provide
OPENAI_API_KEY and CUSTOM_NATIVE_KEY synthetic maps: differing OPENAI_API_KEY moves
to opaque refusal; arbitrary CUSTOM_NATIVE_KEY is unsupported, never restored.
Other policy names (ONLY_A, CUSTOM_HEADER, UNSELECTED_SECRET, FOREIGN_SECRET,
ARBITRARY_SECRET) are synthetic baseline/config/metadata, not caller keys.
RRX_SYNTHETIC_NATIVE_DIRECTORY is exported by the generated fixture executable;
RRX_EMPTY_FIXTURE is a sanitized test-child locator. Neither is a LaunchRequest
caller channel. No production or Codex-fixture caller RRX_ producer was found.

## Initial selection, every new exec, and consumed input

After ordinary EMPTY refusal and exact own scope/resume currency validation, select
values and run the initial Store decision in the synchronous preparation context,
BEFORE `Reservation::persist` or any Starting/Git/native effect. Deterministic initial
refusal therefore yields FreshUnpublished or exact RestoredBeforeAdmission, with
no new Session/audit/consumption publication or process entry sentinel. Inside
the synchronous context closure, a cfg(test)-only initial-selection hook runs AFTER
Scope capture/resume currency validation and BEFORE selection/initial Store decision,
outside the Store mutex, without await. It edits only via a second Store connection.
Removed caller refs/current invalid own refs refuse opaquely before publication;
neutral/enlarging edits can pass this read and get stale-own StateConflict after
Starting/Git at the existing recheck. This seam is separate from BeforeInitialPersist.

Retain the selected private environment and names DTO in that exact fresh/resume
attempt. Immediately before EACH exec receiving those values, invoke the existing
environment-aware exact CAS on the current reservation Session as the last state
check: native version, discovery server, main/policy re-exec. Each successful CAS
increments the actual Session record version and appends session.saved while keeping
dispatch_intent unchanged (null for this new unconsumed attempt). The discovery
PID is cleared to None only after its actual verified shutdown and before the main
pre-exec CAS; that publication must not advertise the reaped discovery PID. All other
logical content remains unchanged. Unknown shutdown keeps the existing hold/uncertainty
and cannot reach this clear or main exec; clearing is not a new death-proof authority.
Update reservation version, Control's published exact snapshot and watch
WHILE the same Store guard used by the successful CAS is still held, mirroring
persist_unchecked; current()/status cannot observe a Store/watch gap. Then release
that guard before any test hook. Assert PID None from the second-connection persisted
Session at that CAS version, watch and Control at the main post-CAS hook. The
session.saved audit has no PID field and earns no PID-clear assertion credit.
Migrate fixed version/audit assertions to these explicit publications.
Irrelevant foreign changes add no different event kind/count. The initial check is
not this final exec fence and performs no publication.

Use an additive private synchronous before-spawn callback at the actual selected
exec sites in `preparation::bounded_git` and `NativeServer::launch_preparing`, after
their command/local setup and directly before spawn. The callback is a statically
owned reservation Store CAS, not caller code, a background job, a native lifetime
lease or a new public API. Add a private pre-exec primitive under the existing
admission mutex: Cancelled/Failing return their latched cause; any other non-Preparing
state (Consumed/CheckpointCommitted) returns StateConflict without running the
closure. Preparing success stays Preparing; CAS error latches Failing before cleanup.
Existing publish alone does not enforce this invariant and must not be reused
verbatim. Unit and actual caller controls prove the phase guard; consume and
checkpoint are not pre-exec primitives. Only admit_dispatch transitions this
launch attempt to Consumed. Stop can still cancel after a successful pre-exec CAS.
Only the existing cancellation check and a pure local deadline recheck may
intervene between successful CAS and spawn; no await/new Store observer. bounded_git
can spend its 5s budget inside the SQLite CAS, so expired version deadline must latch
Timeout without spawn. Do not extend/restart the existing budget. Native startup
timeout keeps its existing semantics; no new native deadline weakening.
Plain Git preflight uses its existing Git
environment and ownership gate, not this native DTO. Its helper currently captures
ALL ambient OS pairs except nine Git routing names; it is not credential-free or
covered by this selection. That shared Git environment/ownership acceptance remains
OPEN under #51/#60 and ordinary EMPTY blocks these Codex preparation effects. No
shared helper/new/Drop/custody implementation is changed or declared safe here.

At EACH selected exec site, a cfg(test)-only synchronous PRE-CAS hook runs after
command setup but before the transaction, outside both admission and Store mutexes.
For replacement cases it performs relevant/own edits only through a second Store
connection, so no earlier post-Git gate masks that CAS omission. A separate PRE-CAS
real-stop case at EACH site waits until cancellation is visible, then releases the
hook: assert no extra pre-exec session.saved/version increment, unchanged exact
watch/Control publication before cleanup, Cancelled first cause and no entry sentinel.
Bind the Cancelled-guard omission mutant to that extra-write assertion; the final
preparation.check alone prevents spawn and cannot earn this kill. Consumed and
CheckpointCommitted phase rejection is UNIT-ONLY because actual pre-exec consumers
do not reach those phases. Failing rejection is also UNIT-ONLY, masked by entry
preparation.check(); mutate it separately from reachable concurrent Cancelled. No
actual-consumer mutant credit for those branches.
A distinct cfg(test) post-primitive outcome observer runs outside both mutexes before
any result propagation/cleanup, including a Cancelled refusal, to assert those exact
unchanged snapshots/events. On success, it is also the post-CAS hook preceding the
unchanged final preparation.check()/spawn. An explicit tokio test runtime
(flavor="multi_thread", worker_threads=2 or more) drives the actual adapter.stop()
concurrently and waits only until cancellation is visible,
then releases the hook so stop can complete; waiting for stop's full outcome inside
the hook would deadlock cleanup. Every synchronous hook rendezvous has a finite
deadline and a distinct fail-closed
harness error; one-worker/default-CPU scheduling is not acceptable evidence.
At each selected site, attempt-scoped cfg(test) spawn observers increment an attempt
counter immediately BEFORE Command::spawn, and a success counter immediately AFTER
it returns an actual Child, before ProcessGroup wrapping/await. They carry no values
or PID/kill authority. Post-CAS real-stop controls assert BOTH counters zero; bind
EACH final-check omission mutant to this deterministic observation. Fixture entry
sentinels are supplementary only: a spawned child can die before writing one. A
Tokio Err is not proof no OS child existed (open #60 wrapping gap); counter/evidence
does not grant production no-effect/cohort/cleanup authority. Any uncertain fixture
spawn/cleanup fails closed, never earns acceptance.
Production has no test hooks, counters or await in that window.

Keep a separate environment-aware CAS at `Reservation::admit_dispatch` immediately
before the buffered `turn/start` wire. This is the consumed-input fence. Its failure
does not prove that earlier discovery/auth/history/hooks never ran. Preserve exact
current-input pins, prior intent rollback, conservative cleanup and Lost/Unknown.
Ordinary Starting/PID/observation/terminal/rollback publications retain their current
purpose; only the named exec and consumed-input boundaries add environment checks.
Approval reply (including Approve/Deny/Cancel) remains own-scope-only and is not a
new selected-environment exec. The ordinary EMPTY approval guard is unchanged.

SQLite and OS exec are not atomic. After successful last admission/cancellation
check, later foreign-reference edits cannot retroactively fence an already-selected
child. Subsequent new exec/current-input boundaries recheck current authority;
cleanup/outcome remains factual. This mechanism never certifies full workload death,
MCP/hooks containment, native provenance or physical revocation.

## Own-only checkpoint and fresh resume

Remove the foreign roster from ScopeSnapshot entirely. Capture still exact-compares
the owning persisted Project with request.project; Goal/Task/complete exact nullable
locks remain strict. Checkpoint refreshes own metadata under immutable repository/
worktree identity and higher-version prepared input, using existing own-only
`put_session_if_current` in its atomic registry/control/Store request replacement.
It performs no selected-native environment decision, checks no foreign refs and
confers no environment authority. It is NOT effect-free: existing preparation
publishes Starting and executes Git with the unselected ambient helper environment.
That #51/#60 residual remains OPEN; its own-only checkpoint positive proves currency
semantics only, never credential isolation, resource custody or native settlement.

A relevant foreign change or caller-key removal before checkpoint must not make
that checkpoint fail. Resume uses the explicitly checkpointed Project and fresh
own-scope snapshot, then performs its own initial/pre-exec/consumed-input environment
checks. Relevant foreign conflicts/removed caller keys must be refused by resume
INITIAL selection, with exact RestoredBeforeAdmission, unchanged persisted Session
version/events, no Starting, Git or version/native entry, and opaque InvalidConfiguration.
Bind resume-initial-check omission to those pre-publication assertions, not merely
no native exec (the later version CAS would mask it). Own edits
after checkpoint still produce stale-own StateConflict and require another explicit
checkpoint; no implicit metadata refresh or cached-authority exception. Fresh input,
known UUID identity and no implicit mutating replay remain unchanged. Managed
Fresh/Continue and standalone checkpoint readiness still require actual #19 integration.

## Configured provider references: deterministic bounded refusal

Reuse the IDENTICAL selected environment for discovery and main/policy re-exec.
Never retain the complete ambient environment for later restoration. Effective
`config/read` env_key/env_http_headers names are not an origin certificate or a
grant to obtain any runtime value. Define a deterministic component rule: every
reported selected-provider reference must be syntactically supported, non-control,
and already present in that private selection. Apply the IDENTICAL pure predicate
immediately after discovery config/read, BEFORE DecisionPolicy::from_native/
for_executor, and immediately after main config/read BEFORE verify_configuration,
account/read, environment/status, thread start/resume/history and consumption.
The reference predicate wins when policy and references are both invalid; native
policy checks remain unchanged and execute only after the predicate passes. This
changes main non-object error precedence from policy UnsupportedCapability to
fixed reference ParseFailure. Combined policy-invalid + reference-invalid cases at
EACH consumer pin this precedence. Discovery non-object step1 can still be
independently subsumed by the policy object check if omitted: UNIT-ONLY step1 kill
credit there, never a claimed independently distinguishing actual-consumer kill.
Unpermitted main references refuse after actual main
cleanup with no current model frame/consumption. A different selected provider with
only permitted references does not supply origin evidence; full provenance stays
OPEN. Selected values pass unchanged; no values are added.

Extraction reads only model_provider and its selected model_providers entry, then
env_key and env_http_headers VALUES, never header names as environment names. Apply
this ordered FIRST-FAILURE procedure at both actual config consumers:

1. Config must be an object, else fixed opaque ParseFailure.
2. Absent/null model_provider returns zero references immediately, without inspecting
   model_providers. Otherwise selection must be a string, else ParseFailure.
3. Empty or over-128-byte selected identifier returns fixed opaque UnsupportedCapability
   BEFORE looking at the providers table, including when that table is absent.
4. Absent/null model_providers returns zero references. Non-null non-object is
   ParseFailure. Absent/null selected entry returns zero; non-null non-object is ParseFailure.
5. env_key absent/null is no occurrence; otherwise require string, else ParseFailure.
   Next, headers absent/null is empty; otherwise require object, else ParseFailure.
6. Count raw occurrences (present env_key plus header map length), including duplicates.
   More than 128 returns UnsupportedCapability BEFORE header-value type/name checks.
7. Inspect ALL header values in lexical header-key order; any non-string is ParseFailure.
   Only bounded borrowed references are retained. This ordering does not depend on
   serde_json preserve_order. No name semantics are checked before all shapes pass.
8. Validate env_key first, then headers in lexical order: each reference is 1–128
   ASCII bytes, first alphabetic/underscore, remaining alphanumeric/underscore, and
   registry-valid, not registry-forbidden/GIT_/this classifier's control. Otherwise
   UnsupportedCapability. Validate lengths before copying.
9. Every reference must already be present in the private selection, else
   UnsupportedCapability. Return the IDENTICAL frozen selected values, never restoration.

Zero reported refs do not infer built-in/default/auth conformance. There is no
includeLayers origin or built-in expansion. The count/per-name bounds imply at most
16KiB name bytes; no redundant aggregate check or independent byte-overflow mutant
credit is claimed. Existing native RPC bounds unrelated config fields to 4MiB;
the review harness 1MiB cap is separate.

Combined actual-consumer cases pin precedence: null selection plus wrong-type table
is zero; empty/oversize id plus absent table is Unsupported; invalid env_key name
plus malformed header map is ParseFailure; 129 occurrences plus a non-string header
value is Unsupported; within-bound non-string value plus invalid env_key name is
ParseFailure. Test selected-table/entry missing and null separately. Compile targeted
check-order mutants and bind independently distinguishable kills to the exact
consumer/error/no-main or no-account/thread/frame sentinel, with no masked/redundant credit.

This deliberately treats absent and ambient-only excluded references alike as an
unproven configuration route. It makes no assertion that an absent value is required,
that auth is unavailable, or that native optional-header behavior would fail. Optional/
absent custom-provider compatibility and full effective-layer/source/endpoint/header
provenance remain OPEN native qualification gates. Default/built-in effective
config/read shapes have not been observed here: the deterministic table applies
equally to reported default-provider fields, but neither a synthetic pass nor absence
of reported fields qualifies ordinary native defaults. Required default/native
compatibility must still be proven; this conservative subset cannot waive it.
No native config/defaults are
changed or disabled to pass; no includeLayers probe/parser/token/API substitution.
The refusal is an explicit component limitation, not whole native MVP acceptance.

## Causal verification and finite gates

Use isolated synthetic iterator input through the actual common constructor and
fail-closed expected-marker booleans; no ambient credential capture, global set_var
or value-printing assert. A sanitized env_clear re-exec checks real public new()
with only known synthetic markers and the public-new capture/bounds observables above.
The native fixture writes a version-entry sentinel and canary-presence booleans
BEFORE its --version early exit, distinct from discovery/main entry. Observe entry
sentinels, canary presence
booleans, Store/watch/audit/frame facts before error labels. Include:

- Constructor freeze/filter/control/bounds, shared API-key and foreign-only collision,
  exact/different native caller values, ordinary-value precedence, OS identity,
  invalid admitted global-control names and malformed owning refs; opaque errors.
  Assert all named routing/identity whitelist members, CODEX_* and *_PROXY stay
  controls through the shared registry predicate; API keys and registry-valid
  LC_*/XDG_* (including XDG_CONFIG_DIRS/DATA_DIRS) stay non-control. Exact additional
  control set is EMPTY; no whole-native compatibility inferred.
  Assert USER/LOGNAME never join membership; their metadata alone does not refuse
  Codex, and non-ordinary/non-whitelisted caller values remain unsupported.
  Assert non-UTF-8/GIT_PROXY exclusion before suffix matching, raw OS values, bounds
  constructor refusal and ordinary EMPTY after valid construction. Explicit own
  candidate inspection reads only that Project's names and never another rejection.
  Foreign-only LANG/TERM denies before effects; shared LANG plus own ordinary override
  reaches the unchanged selected child. Include registry-valid LC_*/XDG_* conflicts.
- Valid foreign JSON with malformed NON-reference fields, unrelated refs/version/
  lifecycle/control/registration/deletion remains eligible. Invalid JSON, missing/
  non-array refs and non-string elements are unknown authority and deny opaquely.
  A relevant name after a long roster cannot be skipped.
- Actual second-Store replacement hooks immediately before version/discovery/main
  exec: both foreign relevant changes and own-ref edits deny that particular NEW
  child/canary; own edits assert transactional StateConflict and coherent versions.
  Initial rejecting own edits after capture refuse before publication with the observed
  opaque category. Neutral or enlarging own-ref edits can pass the separate selection,
  publish Starting/run Git, then fail the existing post-Git recheck or first native CAS
  as stale-own StateConflict. Record that complementary path; do not claim every own
  edit is fenced by initial selection or add a new transaction. Earlier selected
  children may have run and must clean up factually. Post-CAS real stop uses the
  specified hook: assert zero spawn-attempt/success counters, supplementary no entry,
  real stop outcome and coherent Store/watch.
  Matching controls prove each path reaches its exec unchanged, still Preparing
  after each pre-exec CAS, with the later consumed CAS independently succeeding.
  Assert coherent Session/dispatch intent (including verified-dead discovery PID None
  at its main post-CAS persisted/watch/Control snapshots), and identical scoped
  event kind/count
  under irrelevant roster changes; pre-exec success is a durable versioned write.
- Final consumed CAS conflict yields no current turn/start frame/consumed publication,
  without pretending prior execs never happened. Already-owned approval/stop remains
  independent of later foreign metadata.
- Relevant foreign declaration/own caller-key removal before checkpoint permits
  checkpoint, then resume refuses at initial selection with the exact no-publication/
  no-Git shape above; post-checkpoint own change keeps StateConflict.
  Inject start/resume irrelevant changes at initial-selection hook AFTER capture or
  a pre-exec hook BEFORE the second recheck. For checkpoint, use its existing
  BeforeInitialPersist gate AFTER capture/before Git/recheck, and a second Store for
  BOTH relevant and irrelevant foreign changes. Checkpoint commits with identical
  own-scope event kinds/counts to unchanged control. Bind EACH reintroduced roster
  equality mutant to its own windowed actual consumer; pre-call edits earn no credit.
- Config selected/absent/ambient-only/control/invalid reference cases reach the actual
  consumer. Unknown references never restore a runtime canary; discovery cleanup
  precedes refusal, with no main exec/model frame. A second effective config with an
  unpermitted ref denies before account/thread/model/consumption after real main
  cleanup; identical config is positive. Cover extraction branches with actual-consumer
  controls where independently reachable/distinguishing, unit-only credit where
  explicitly subsumed; include combined policy/reference-invalid cases and synthetic
  reported default entry, combined first-failure cases, raw duplicate/count and per-name
  bounds, and exact restored source. No redundant aggregate-bound kill credit.
  No native optionality/default-provider proof claimed.

Compiled actual-consumer mutants cover real public-new iterator/eager-bounds wiring,
constructor filtering/control/freeze/bounds,
DTO baseline/caller population, exact-value comparison, initial check omission,
EACH pre-exec CAS omission (version killed by its pre-exit sentinel), Cancelled pre-exec
guard omission killed by extra-write evidence, with non-reachable phase branches
unit-only; EACH post-CAS cancellation check omission killed by deterministic spawn
counters (entry sentinel alone is racy and earns no kill credit), wrongly consuming
pre-exec publication, discovery reference-check omission killed on no-main-entry,
main reference-check omission killed on no-account/thread/model/consumption,
consumed CAS omission,
verified-dead discovery PID-clear omission bound to persisted/watch/Control snapshots,
arbitrary reference restoration,
and reintroduced roster equality at start/checkpoint/resume. Bind each to its precise
canary/entry/Store assertion. Checkpoint is not an environment-CAS mutation target.
Redundant/masked/surviving operators earn no kill credit; restore exact source bytes.

Implementation must update master agent-adapter section 16 (Codex component boundary)
and its environment-admission current-state text: frozen/filter/control/DTO facts,
all-non-control availability and ungated pure own candidate metadata inspection,
removed public legacy helper/API migration, each pre-exec publication,
own-only checkpoint, both effective-config predicates and conservative subset limits.
Keep ordinary EMPTY and all missing ownership/native producers explicit, with no
issue history in those master paragraphs. Root #19 lifetime component remains a
separate coordinated consumer; these callbacks do not acquire resource custody.

Commit clean stages before default checks: full workspace debug/release, fmt,
all-target denied-warning Clippy/builds and current both-OS checkout/tree/blob proof.
Independent native zero-tool reviews consume public bytes only with normal native
defaults/auth/rules/hooks and retained review guardians. Historical failed wrappers
and their old worktree remain held separately. No component success, DTO or callback
opens EMPTY availability or closes #6/51/19/58/60/14, IPC02, Task attach or four-plus MVP.
