# Issue 6: existing name-only environment admission integration (Design1)

Status: proposed STRICT component integration; requirements/design/source gates
pending. Base is the limited Stage A merge `47830b0ab97a095d0c387089bf5442452e060572`.
Production Codex availability stays EMPTY. No native profile, workload owner,
managed operation, setup/settlement receipt or backend is introduced.

## Existing contract and concrete consumers

The merged Issue51 contract already supplies `EnvironmentAdmission::new`,
`Store::check_environment_admission` and
`Store::put_session_with_environment_if_current`. The DTO contains bounded,
preclassified non-control baseline names and caller names, never values. The
identical pure decision checks current owning references and streams foreign
`environment_refs` through SQLite; it neither fully decodes foreign Projects nor
reads their files. Initial selection and final exact Session/P/G/T/nullable-lock
CAS can use this existing authority. This is an environment policy decision, not
process ownership, delivery, cleanup, settlement or configuration provenance.

Current Codex consumers still use a different path:

- `ScopeSnapshot::capture` retains `Store::projects()`. `recheck_authority` compares
  whole foreign Project JSON, including unrelated metadata, and can fail on
  foreign non-reference corruption.
- `prepare_launch` captures mutable process environment after Git preparation;
  `native_environment` silently removes a retained credential if only another
  Project declares its name. This can select cached/default authentication.
- `provider_environment` treats merged `config/read` names as authority to restore
  any matching runtime variable, without authoritative layer provenance.
- Initial, checkpoint and consumed-input publications use ordinary Session CAS,
  without the existing in-transaction environment decision. Approval replies must
  not become dependent on foreign reference changes after a child was selected.

The integration changes only own Codex modules and their component tests. Shared
Store/Grok/Generic/registry behavior and schema are unchanged. It consumes the
already reviewed APIs rather than creating a competing environment transaction.

## Frozen baseline and selection

Capture the intentional native baseline once in pure adapter construction, keep
values private, and reuse that exact baseline for fresh preparation, checkpoint
and resume. Preserve existing trusted native auth/config/hooks/proxy/TLS routing
and OS login identity. Project references/caller values cannot override native
controls, including HOME/PATH/config selectors and trusted USER/LOGNAME. Existing
native whitelist/prefix routing is documented exactly; no foreign-name conflict
silently drops a credential or changes authentication to fit a profile.

Use the existing registry control classifier plus the finite OS identity additions
USER/LOGNAME. Do not classify provider credential prefixes as controls or broaden
the baseline to all runtime variables. Frozen values retain their native OS string
representation privately; unsupported names or caller comparisons fail rather
than truncating an authority projection or logging a value.

Adopt the existing Issue51 name policy for non-control keys: owning references
must be valid; ordinary locale/terminal keys may take explicitly declared caller
values. A retained native non-control caller key requires its intentional frozen
value, not a different credential/loader setting. Undeclared or unsupported caller
keys fail explicitly. No different-per-Project value routing or API-client auth
substitution is introduced. Values never enter the DTO, durable state, audit,
public candidate projection or error diagnostics.

Construct the bounded DTO from the actual selected non-control baseline and
caller names. Its initial Store decision runs after ordinary availability refusal
and own-scope validation, before any component process/model effect. An owning
invalid/control declaration or relevant foreign reference conflict fails with the
fixed opaque environment category; irrelevant foreign metadata/lifecycle/control
or nonmatching reference changes do not revoke selection. All foreign lifecycle
states and unknown reference projections follow the existing Store contract.

Remove the whole-Project roster from ScopeSnapshot. Own Project/Goal/Task and exact
nullable scoped locks remain strict currency. Do not refresh their versions to
hide changes. An already-selected child's approval/observation authority remains
own-scope-only; foreign roster edits are not a second grant ledger or a stop cause.

## Selected native provider references: no inferred provenance

Keep actual native settings and mandatory rules unchanged. A merged effective
`config/read` value is not a trusted reference-origin certificate. This slice
removes runtime-value restoration from arbitrary effective provider `env_key` or
`env_http_headers` names. The selector may validate a configured reference already
in the private selected environment; it cannot introduce an arbitrary new runtime
value just because native config names it. A required reference outside that
explicit selection fails Unsupported before re-exec/model dispatch, rather than
silently selecting cached auth or disabling/changing native config.

A reference name alone does not prove that an environment value is required or
that an absent value changes native authentication. Missing/optional reference
semantics remain native readiness/configuration concerns; this integration never
extracts a replacement value or infers an authentication fallback from that name.

This is a bounded refusal for unproven reference expansion, not complete IPC02.
Full effective-layer/source/endpoint/header provenance and custom-provider native
compatibility remain required open gates. No `includeLayers` native probe, file
parser pretending to be native effective config, token extraction or replacement
API client is added. Ordinary production still refuses before discovery because
its actual workload backend/dispatch producer remains absent.

## Actual publication boundary

Retain the private DTO with the same preparation/attempt that selected its values.
Use the existing environment-aware exact CAS at fresh input/checkpoint/resume
admission, with no await or new observer between the final decision and existing
component dispatch boundary. A relevant reference replacement across another
Store connection is rechecked in that Immediate transaction. Own P/G/T/Session
and complete exact lock guards remain unchanged. Failure preserves actor/version,
consumed-input facts and conservative Lost/Unknown cleanup semantics.

Do not apply a foreign environment recheck to denial/stop/drain of a previously
owned child or claim that SQLite and OS exec are atomic. Post-admission changes
are not retroactive physical fencing. Every future enabled launch still needs
actual Stage B/#19/#58/#60/#14 ownership, all-job closure and current settlement.

## Finite verification and source gate

Use synthetic isolated component controls with existing same-route test inputs;
no real native auth/model/kernel/profile probes and no unrelated repository data.
Component fixtures receive an explicit private synthetic baseline input so real
runtime credentials are never forwarded to fake children. That test seam changes
data only, never availability/routing, and does not mutate process-wide environment.
Observe actual entry/process sentinels and Store/audit/version/frame state before
classification strings. Include:

- Two Projects, foreign-only and shared API-key names, intentional native control
  preservation, malformed own refs, unsupported/different caller credential values
  and opaque errors that never echo synthetic canary values.
- Irrelevant foreign body corruption/non-reference/version/lifecycle changes do not
  affect own capture/selection; unknown or relevant reference authority does deny.
- Original selection, explicit higher-version checkpoint and resume: actual second
  Store replacement windows before initial/final admission, including a relevant
  name after a long roster. Exact own P/G/T/Session/lock guards stay causal.
- Native effective config cannot add an arbitrary runtime-secret reference;
  already selected references remain unchanged. This is synthetic reference-policy
  evidence, not native layer or custom-provider acceptance.
- Actual consumer mutants omit selection/final environment CAS or restore arbitrary
  reference values, and must fail effect/canary assertions. Redundant survivors
  are explicitly classified without kill credit; no timing/concurrency relaxation.

Commit clean stages before the scoped checks. Run appropriate default full
workspace debug/release, fmt, denied-warning all-target Clippy/builds and current
both-OS actual-checkout/tree/blob gates. Independent native zero-tool reviews use
public bytes only, normal native defaults/auth/rules/hooks and properly retained
owned review guardians. The historical failed review wrappers and old worktree
remain held separately; this new worktree never adopts their numeric identities.
No successful component test or names-only DTO opens native availability or closes
Issue6/51/19/58/60/14, IPC02, Task attach or four-plus MVP acceptance.
