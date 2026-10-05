# Issue 6: existing name-only environment admission integration (Design4)

Status: proposed STRICT component integration; design/source qualification pending.
Design1 `6264702`, Design2 `9884ec3` and Design3 `4a60fa2` were not qualified as a pair.
This proposal normally composes main `5b4a3147b2b2cd4ed88642741f6b25c4c5f3441b`.
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
with owned handles. No process-wide environment mutation, credential extraction,
different-per-Project native value routing or API-client auth substitution.

Preserve ordered membership: discard non-UTF-8 names and ALL GIT_* names before
applying the whitelist, prefixes or suffix; never compare a lossy name. Baseline
membership then retains the current Codex whitelist: HOME, PATH, SHELL, LANG,
TERM, TMPDIR, TEMP, TMP, NODE_OPTIONS, NODE_PATH, SSL_CERT_FILE, SSL_CERT_DIR,
NODE_EXTRA_CA_CERTS, OPENAI_API_KEY, OPENAI_BASE_URL, OPENAI_API_BASE, XAI_API_KEY,
ANTHROPIC_API_KEY, SSH_AUTH_SOCK, SSH_ASKPASS, EDITOR and VISUAL; plus LC_, XDG_,
CODEX_ prefixes and case-insensitive *_PROXY suffix. Explicit compatibility delta:
USER and LOGNAME join that whitelist to preserve inherited runtime identity when
present. These process-environment values are not
kernel-attested identity, and no native need is claimed from the historical component
evidence, which omitted them. This explicitly requested compatibility addition is
unqualified native behavior until the required defaults/hooks/auth gates pass.
Do not synthesize missing values. No other native prefix/variable is added.

The exact private control predicate is baseline membership AND (invalid registry
name OR registry-forbidden name OR member of the finite additions {USER, LOGNAME}).
Admitted syntactically invalid baseline names remain unownable global controls,
preserved privately and excluded from the DTO. Do not reject an entire host for a
name that no valid Project can own. Foreign control declarations cannot strip or
replace the baseline. Reject owning control declarations when the name satisfies
this provider's membership predicate, including USER/LOGNAME even if their value was
absent from this frozen snapshot. This deliberate metadata-eligibility change means
such own declarations for another provider also refuse Codex; no caller is needed.
Other valid own metadata outside Codex's membership is not a blanket rejection.
Credentials/credential-location prefixes
never become controls; structural tests keep OPENAI_API_KEY, XAI_API_KEY and
ANTHROPIC_API_KEY non-control and assert the finite additions exactly.

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

Add explicit Codex-owned `environment_candidates(project)` inspection using
`Store::environment_candidates` and this frozen baseline's non-control DTO with
empty caller names. It returns only the requested Project's declared candidate
names, for Registered/Blocked/Removed states; no values, foreign query or filesystem
read. It is never invoked from another Project's rejection. Invalid/unknown own
metadata retains the Store's opaque failure; the method grants no launch authority.

As in Issue51, validate own references/control declarations before caller-map syntax,
then malformed/GIT_/NUL caller input uses InvalidInput (a change from Codex's current
OwnershipMismatch). A combined invalid-own plus malformed-caller case therefore uses
the opaque own-reference InvalidConfiguration. Invalid/control own refs,
reserved RRX_, unsupported caller names, undeclared/foreign-only eligible caller
names, different frozen credential values and baseline conflicts use the same
fixed opaque InvalidConfiguration category, `native environment authority
unavailable`, including the Store EnvironmentAuthority mapping. Never emit foreign
IDs, names, versions, values or inventory. Existing own stale/lifecycle/Session/lock
checks retain their kinds and precede selection when they observe the conflict.
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
| Effective config restores an arbitrary runtime key/header | No value restoration; explicit unproven-reference refusal described below |
| Own invalid refs return InvalidInput; undeclared caller returns OwnershipMismatch | Fixed opaque InvalidConfiguration |
| Malformed/GIT_/NUL caller returns OwnershipMismatch | InvalidInput after own-reference validation; combined-invalid order is explicit |
| USER/LOGNAME omitted | Inherited constructor values forwarded; owning declarations and caller overrides refused, including absent-value own declarations; native qualification pending |
| Missing/null selected-provider entry yields zero refs | Same zero-reported-ref outcome; no inferred/default/built-in conformance |
| Empty/over-128-byte selected id or non-object provider/settings can yield zero refs | Explicit Unsupported identifier bound or ParseFailure shape refusal; conservative unqualified-default limitation |
| Public new() never captures environment | Frozen constructor selection can fail opaquely on unsupported bounds; successful construction still leaves ordinary availability EMPTY |
| No extra pre-exec Session publications | Up to three unchanged-content session.saved writes/version increments per fresh/resume attempt; no new consumption/dispatch intent |

Old strip/pass-through/custom-reference policy assertions migrate to these declared
outcomes. Existing own currency/cancellation, native
defaults/auth/hooks/config, consumed-input/Lost facts and shared cleanup semantics
retain their current purpose. All eighteen unit-test new() calls in session.rs
must migrate to the common injected iterator constructor, including ordinary
unavailable tests; no test captures ambient credentials then overwrites a baseline.
The two external integration calls in tests/codex_unavailable.rs are already inside
an env_clear re-executed child with synthetic HOME/PATH. Add a pre-constructor
expected-name/value boolean guard there; no unknown value is printed or captured
by the adapter. No example or production factory currently constructs CodexAdapter.
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
no new Session/audit/consumption publication or process entry sentinel.

Retain the selected private environment and names DTO in that exact fresh/resume
attempt. Immediately before EACH exec receiving those values, invoke the existing
environment-aware exact CAS on the current reservation Session as the last state
check: native version, discovery server, main/policy re-exec. Each successful CAS
increments the actual Session record version and appends session.saved while keeping
logical Session content and dispatch_intent unchanged (null for this new unconsumed
attempt). Update reservation version, Control's published exact snapshot and watch
coherently; migrate fixed version/audit assertions to these explicit publications.
Irrelevant foreign changes add no different event kind/count. The initial check is
not this final exec fence and performs no publication.

Use an additive private synchronous before-spawn callback at the actual selected
exec sites in `preparation::bounded_git` and `NativeServer::launch_preparing`, after
their command/local setup and directly before spawn. The callback is a statically
owned reservation Store CAS, not caller code, a background job, a native lifetime
lease or a new public API. Use `Preparation::publish` semantics under its admission
mutex: Cancelled/Failing short-circuit, failed CAS latches Failing, and successful
pre-exec CAS leaves Preparing unchanged. These callers require Preparing; consume
and checkpoint are not pre-exec primitives. Only admit_dispatch transitions this
launch attempt to Consumed. Stop can still cancel after a successful pre-exec CAS.
Only the existing cancellation check may intervene between successful CAS and spawn;
no production await/new Store observer. Plain Git preflight uses its existing Git
environment and ownership gate, not this native DTO. Its helper currently captures
ALL ambient OS pairs except nine Git routing names; it is not credential-free or
covered by this selection. That shared Git environment/ownership acceptance remains
OPEN under #51/#60 and ordinary EMPTY blocks these Codex preparation effects. No
shared helper/new/Drop/custody implementation is changed or declared safe here.

At each selected exec site, a cfg(test)-only synchronous post-CAS hook precedes the
unchanged final preparation.check()/spawn. A multi-thread runtime rendezvous drives
the actual adapter.stop() concurrently and waits only until cancellation is visible,
then releases the hook so stop can complete; waiting for stop's full outcome inside
the hook would deadlock cleanup. Production has no hook or await in that window.

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
checks. Removed caller keys now fail opaquely before any new native exec. Own edits
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
to discovery config/read before main re-exec AND the main server's effective
config/read BEFORE account/read, environment/status, thread start/resume/history
and consumed publication. Unpermitted main references refuse after actual main
cleanup with no current model frame/consumption. A different selected provider with
only permitted references does not supply origin evidence; full provenance stays
OPEN. Selected values pass unchanged; no values are added.

Deterministic extraction reads only model_provider and the selected entry of
model_providers, then env_key and env_http_headers VALUES (never header names as
environment names). No inferred includeLayers origin or built-in-provider expansion:

| Reported shape | Component outcome |
| --- | --- |
| Config not an object; non-string non-null selection | Fixed opaque ParseFailure |
| Absent/null model_provider | Zero reported references; no default/provider/auth conformance inferred |
| Absent/null model_providers; selected entry absent/null | Zero reported references, same as absent selection; no invented built-in/default entry |
| Empty or over-128-byte selected id | Fixed opaque UnsupportedCapability identifier limitation |
| Non-null non-object model_providers or selected entry | Fixed opaque ParseFailure shape limitation |
| Selected object, absent/null env_key and env_http_headers | Zero reported references |
| Non-string non-null env_key, non-object non-null headers, non-string header value | Fixed opaque ParseFailure |
| Reported valid references all present and non-control | Same frozen selection; no restoration |
| Invalid/control/absent reference or extraction overflow | Fixed opaque UnsupportedCapability |

Selected identifiers are nonempty UTF-8 strings bounded to 128 bytes. Each reference
preserves the existing selector's 1–128 ASCII-byte predicate: first alphabetic or
underscore, remaining alphanumeric or underscore; additionally registry-valid,
not registry-forbidden, not GIT_ and not this baseline's control predicate. Invalid
names/control names refuse even if HOME/PATH or another value is present. Limit the
raw env_key-plus-header reference occurrences to 128 and their aggregate name bytes
to 16KiB, checking before copying/collecting; repeated names still count. Unsupported
overflow never truncates. This reuses the existing caller-count envelope and native
reference length as an explicit component bound, not measured native availability.
Header iteration is bounded before retaining names; unrelated config fields are
already bounded by the existing 4MiB native RPC frame, not the 1MiB review harness.

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
with only known synthetic markers. Observe entry sentinels, canary presence
booleans, Store/watch/audit/frame facts before error labels. Include:

- Constructor freeze/filter/control/bounds, shared API-key and foreign-only collision,
  exact/different native caller values, ordinary-value precedence, OS identity,
  invalid admitted global-control names and malformed owning refs; opaque errors.
  Assert all named routing/identity whitelist members, CODEX_* and *_PROXY stay
  controls through the shared registry predicate; API keys stay non-control.
  Assert USER/LOGNAME own and caller refusals with both present and absent values;
  these are inherited runtime metadata, not proof of native or kernel identity.
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
  Initial own edits after capture are separately observed before publication without
  claiming atomic preflight/error-kind precedence. Earlier selected
  children may have run and must clean up factually. Post-CAS real stop uses the
  specified hook: assert no child, real stop outcome and coherent Store/watch.
  Matching controls prove each path reaches its exec unchanged, still Preparing
  after each pre-exec CAS, with the later consumed CAS independently succeeding.
  Assert unchanged Session/dispatch intent and identical scoped event kind/count
  under irrelevant roster changes; pre-exec success is a durable versioned write.
- Final consumed CAS conflict yields no current turn/start frame/consumed publication,
  without pretending prior execs never happened. Already-owned approval/stop remains
  independent of later foreign metadata.
- Relevant foreign declaration/own caller-key removal before checkpoint permits
  checkpoint, then resume fails pre-exec; post-checkpoint own change keeps StateConflict.
  Irrelevant roster changes permit actual initial/checkpoint/resume controls.
- Config selected/absent/ambient-only/control/invalid reference cases reach the actual
  consumer. Unknown references never restore a runtime canary; discovery cleanup
  precedes refusal, with no main exec/model frame. A second effective config with an
  unpermitted ref denies before account/thread/model/consumption after real main
  cleanup; identical config is positive. Cover EVERY extraction table row, synthetic
  reported default entry, raw duplicate/count/byte overflow and exact restored source.
  No native optionality/default-provider proof claimed.

Compiled actual-consumer mutants cover constructor filtering/control/freeze/bounds,
DTO baseline/caller population, exact-value comparison, initial check omission,
EACH pre-exec CAS omission, post-CAS cancellation check omission, wrongly consuming
pre-exec publication, discovery reference-check omission killed on no-main-entry,
main reference-check omission killed on no-account/thread/model/consumption,
consumed CAS omission,
arbitrary reference restoration,
and reintroduced roster equality at start/checkpoint/resume. Bind each to its precise
canary/entry/Store assertion. Checkpoint is not an environment-CAS mutation target.
Redundant/masked/surviving operators earn no kill credit; restore exact source bytes.

Implementation must update master agent-adapter section 16 (Codex component boundary)
and its environment-admission current-state text: frozen/filter/control/DTO facts,
all-non-control availability and own candidate inspection, each pre-exec publication,
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
