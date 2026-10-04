# Issue 6: existing name-only environment admission integration (Design2)

Status: proposed STRICT component integration; design/source qualification pending.
Design1 `6264702` was not qualified. This correction normally composes main
`cf8a1e7c4ab74367ad175bedcfb60e28cb6bfc8b`. Production Codex availability stays
EMPTY. No native workload backend, managed operation, setup/settlement receipt,
custodian or native-origin provenance producer is introduced.

## Existing authority and actual consumers

Merged Issue51 requirements 3 and 6–8 supply the existing policy: bounded names-only
environment selection, a last current Session/environment check before native exec,
effect-free own-only checkpoint, and opaque rejection without credential stripping.
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

The current caller-map producer is `WorkflowEngine::step/start_agent`, which forwards
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

Baseline membership retains the current Codex whitelist: HOME, PATH, SHELL, LANG,
TERM, TMPDIR, TEMP, TMP, NODE_OPTIONS, NODE_PATH, SSL_CERT_FILE, SSL_CERT_DIR,
NODE_EXTRA_CA_CERTS, OPENAI_API_KEY, OPENAI_BASE_URL, OPENAI_API_BASE, XAI_API_KEY,
ANTHROPIC_API_KEY, SSH_AUTH_SOCK, SSH_ASKPASS, EDITOR and VISUAL; plus LC_, XDG_,
CODEX_ prefixes and case-insensitive *_PROXY suffix. Explicit compatibility delta:
USER and LOGNAME join that whitelist to preserve trusted OS login identity when
present. Do not synthesize missing values. No other native prefix/variable is added.

The exact private control predicate is baseline membership AND (invalid registry
name OR registry-forbidden name OR member of the finite additions {USER, LOGNAME}).
Admitted syntactically invalid baseline names remain unownable global controls,
preserved privately and excluded from the DTO. Do not reject an entire host for a
name that no valid Project can own. Foreign control declarations cannot strip or
replace the baseline. Reject owning control declarations only when the name is in
this provider's baseline membership. Other valid own metadata not forwarded by
Codex is not a blanket Project rejection. Credentials/credential-location prefixes
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

Malformed/GIT_/NUL caller input preserves InvalidInput. Invalid/control own refs,
reserved RRX_, unsupported caller names, undeclared/foreign-only eligible caller
names, different frozen credential values and baseline conflicts use the same
fixed opaque InvalidConfiguration category, `native environment authority
unavailable`, including the Store EnvironmentAuthority mapping. Never emit foreign
IDs, names, versions, values or inventory. Own stale/lifecycle/Session/lock errors
retain their existing precedence and kinds.

Intentional changes and test migrations:

| Current Codex behavior | Proposed outcome |
| --- | --- |
| Baseline captured after preparation at each launch | Frozen once by the common constructor; handles and continuation reuse it |
| Foreign-only retained credential silently stripped | Opaque refusal before the relevant new exec; no cached-auth substitution |
| Any valid owning-declared arbitrary caller value passed | Existing51 ordinary OR exact frozen native value rule; unsupported/different values refused |
| Full foreign Project decode/equality | Own Project equality, Goal/Task/locks remain exact; foreign refs alone determine environment policy |
| Effective config restores an arbitrary runtime key/header | No value restoration; explicit unproven-reference refusal described below |
| Own invalid refs/undeclared caller errors have different kinds | Fixed opaque environment category, preserving malformed caller and stale-own precedence |
| USER/LOGNAME omitted | Present trusted constructor values forwarded; own/caller override refused |

Old strip/pass-through/custom-reference policy assertions migrate to these declared
outcomes. Existing ordinary unavailable tests, own currency/cancellation, native
defaults/auth/hooks/config, consumed-input/Lost facts and shared cleanup semantics
remain unchanged. Synthetic fixture constructors migrate to the common injected
constructor so no real credentials enter fake children. The new OS identity
forwarding is tested as an explicit compatibility change, not described as unchanged.

## Initial selection, every new exec, and consumed input

After ordinary EMPTY refusal and exact own scope/resume currency validation, select
values and run the initial Store decision in the synchronous preparation context,
BEFORE `Reservation::persist` or any Starting/Git/native effect. Deterministic initial
refusal therefore yields FreshUnpublished or exact RestoredBeforeAdmission, with
no new Session/audit/consumption publication or process entry sentinel.

Retain the selected private environment and names DTO in that exact fresh/resume
attempt. Immediately before EACH exec receiving those values, invoke the existing
environment-aware exact CAS on the current reservation Session as the last state
check: native version, discovery server, main/policy re-exec. It updates the private
Session version/watch snapshot coherently but does not publish consumed input or a
new environment dispatch-intent. The initial check is not this final exec fence.

Use an additive private synchronous before-spawn callback at the actual selected
exec sites in `preparation::bounded_git` and `NativeServer::launch_preparing`, after
their command/local setup and directly before spawn. The callback is a statically
owned reservation Store CAS, not caller code, a background job, a native lifetime
lease or a new public API. The admission/cancellation critical section preserves
the existing first-cause rules. Only the existing cancellation check may intervene
between successful CAS and spawn; no await/new Store observer. Plain Git preflight
uses its existing Git environment and ownership gate, not the native credential DTO.
No shared helper/new/Drop/custody implementation is changed or declared safe here.

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

## Effect-free checkpoint and fresh resume

Remove the foreign roster from ScopeSnapshot entirely. Capture still exact-compares
the owning persisted Project with request.project; Goal/Task/complete exact nullable
locks remain strict. Checkpoint refreshes own metadata under immutable repository/
worktree identity and higher-version prepared input, using existing own-only
`put_session_if_current` in its atomic registry/control/Store request replacement.
It selects no values, checks no foreign refs and confers no environment authority.

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
explicit selected-provider reference must be syntactically supported, non-control,
and already present in that private selection. Otherwise refuse
UnsupportedCapability after actual discovery cleanup and BEFORE main re-exec or
input consumption. Selected references pass unchanged; no values are added.

This deliberately treats absent and ambient-only excluded references alike as an
unproven configuration route. It makes no assertion that an absent value is required,
that auth is unavailable, or that native optional-header behavior would fail. Optional/
absent custom-provider compatibility and full effective-layer/source/endpoint/header
provenance remain OPEN native qualification gates. No native config/defaults are
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
- Valid foreign JSON with malformed NON-reference fields, unrelated refs/version/
  lifecycle/control/registration/deletion remains eligible. Invalid JSON, missing/
  non-array refs and non-string elements are unknown authority and deny opaquely.
  A relevant name after a long roster cannot be skipped.
- Actual second-Store replacement hooks immediately before version/discovery/main
  exec: relevant change denies that particular NEW child/canary; earlier selected
  children may have run and must clean up factually. A stop after CAS before spawn
  causes no child. Matching controls prove each path reaches its exec unchanged.
- Final consumed CAS conflict yields no current turn/start frame/consumed publication,
  without pretending prior execs never happened. Already-owned approval/stop remains
  independent of later foreign metadata.
- Relevant foreign declaration/own caller-key removal before checkpoint permits
  checkpoint, then resume fails pre-exec; post-checkpoint own change keeps StateConflict.
  Irrelevant roster changes permit actual initial/checkpoint/resume controls.
- Config selected/absent/ambient-only/control/invalid reference cases reach the actual
  consumer. Unknown references never restore a runtime canary; discovery cleanup
  precedes refusal, with no main exec/model frame. No native optionality proof claimed.

Compiled actual-consumer mutants cover constructor filtering/control/freeze/bounds,
DTO baseline/caller population, exact-value comparison, initial check omission,
EACH pre-exec CAS omission, consumed CAS omission, arbitrary reference restoration,
and reintroduced roster equality at start/checkpoint/resume. Bind each to its precise
canary/entry/Store assertion. Checkpoint is not an environment-CAS mutation target.
Redundant/masked/surviving operators earn no kill credit; restore exact source bytes.

Commit clean stages before default checks: full workspace debug/release, fmt,
all-target denied-warning Clippy/builds and current both-OS checkout/tree/blob proof.
Independent native zero-tool reviews consume public bytes only with normal native
defaults/auth/rules/hooks and retained review guardians. Historical failed wrappers
and their old worktree remain held separately. No component success, DTO or callback
opens EMPTY availability or closes #6/51/19/58/60/14, IPC02, Task attach or four-plus MVP.
