# Issue 51 requirements: scoped Grok native environment references

Risk: STRICT. MVP-blocking follow-up to #7 and actual integration acceptance #16.
Goal §6 separates credentials/environment references by Project. Requirements/design
approval precedes native/shared implementation. No Grok, Store or provider source is
changed by this requirements-only head.

## Verified problem and evidence limits

The executor reports using public GrokAdapter source from main4851fcd (unchanged at
docs-only d56f2bf); the compiled binary's revision was not independently attested. It was
launched with an exclusively synthetic constructor environment and a real isolated
fake ACP child. Corrected fixture7356fa4 persisted Project B with TZ and
XAI_PROJECT_B_CANARY references and verified them through public environment_names;
Project A declared neither. Caller ordinary TZ value synthetic-caller-B and constructor
XAI_ value synthetic-baseline-B both reached A's child. The capture precedes ACP.
Probe1/1 passed1.88s, terminal Exited/failure null/actual OS exit_code null.

Earlier fixture9085422 instead inserted forbidden RRX_PROJECT_B_CANARY directly
through Store.put_project; ProjectRegistry would reject that reference. Preserve its
result only as invalid-record and unconditional reserved caller RRX_ passthrough
characterization, not registry-valid foreign ownership. Initial compile failure of
that fixture was corrected/committed before execution and earns no test/mutant credit.
The global XAI_GLOBAL_AUTH_CANARY, declared by no Project, remained in both child
captures: intentional global native baseline, not a defect. No installed Grok, actual
credentials/private configuration or production worktree was used. Child propagation
is proven; native inference/auth/general provider acceptance is not.

Grok constructor baseline_key correctly avoids inheriting all ambient variables.
environment() then admits ordinary RRX_/locale keys without owning refs and does not
fence known foreign baseline references; supervise uses env_clear().envs directly.
Normative authority comes from Goal§6, #7 and this document only. Current unmerged
#5/#6 policies are non-normative comparison inputs pending normally integrated reviewed
contracts, not universal provider safety evidence.

## Required behavior

1. Resolve the persisted exact owning Project and name-only foreign reference inventory.
   Fully validate own Project/reference policy before selection; malformed, forbidden
   or duplicate own refs fail closed. Existing activity/owner/session/lock/version
   guards remain. Foreign inventory is a syntactic name-only read within admission,
   including inactive records; never scan foreign filesystem/Git/rules/config. Foreign
   malformed/forbidden records cannot grant authority or alter global controls.
   Inventory semantics are per name: an invalid mixed record still contributes each
   exact non-control baseline name to conflict detection; a forbidden control name
   falls under requirement 3. Skipping a record never grants its references.
   Provide a separately Project-scoped operator diagnostic for that Project's own
   conflicting reference names only; never copy it into A's context/audit or expose
   values/foreign identity in A's error. This diagnostic comes from explicit state-only
   operator evaluation or inventory maintenance, never another Project's launch activity,
   counts or timestamps. Explain clear/reactivate when the declaring source remains valid,
   or removing the variable from the runtime environment and reconstructing the adapter
   when that source is gone; lifecycle guards remain authoritative.
2. Preserve the registry's RRX_ reserved-name prohibition. Caller RRX_ keys and routing/
   loader/native-control names are rejected, even with identical baseline values.
   No new runtime bypass/channel is added. Existing fake ACP metadata must move to the
   generated fixture executable or other fixture-owned data, not LaunchRequest
   environment. The adapter's fixed argv is not a fixture channel. Analyze all current
   producers/harnesses before implementation; any production RRX_ producer reopens
   requirements rather than receiving a design-time exception.
   Admissible caller keys are only registry-valid own references that were already
   allowed by the Grok policy: ordinary LANG/LC_ALL/LC_CTYPE/TERM/COLORTERM/TZ, or native-
   whitelisted non-control names with exactly their constructor baseline value. A
   declared non-ordinary/non-whitelisted key remains unsupported; no widened passthrough.
   Undeclared and foreign-only keys return one identical opaque bounded category.
3. Separate immutable global native identity/routing/config/safety controls from scoped
   references. Controls cannot be replaced, removed, or made Project-local by foreign
   declaration. Protected baseline controls include the existing named baseline routing/
   identity/TLS/proxy/shell keys and forbidden LD_/DYLD_ loader prefixes; the design must
   enumerate a reviewed finite set of additional runtime controls such as
   NODE_TLS_REJECT_UNAUTHORIZED, never classify all NODE_/GROK_/XAI_ names as control.
   A protected key in own refs fails admission; a foreign declaration (including an
   invalid persisted control reference) neither strips nor replaces its baseline value.
   Caller rejection and baseline protection use the same reviewed set. Include every
   registry-forbidden name that can enter the baseline, including prefix-only
   NODE_EXTRA_CA_CERTS/NODE_PATH. Membership is a predicate, not an exhaustive finite
   list: named controls, LD_/DYLD_ prefixes, baseline names forbidden by the registry,
   and the reviewed finite additions. Registry-forbidden names have control precedence
   because no valid Project can own them. The credential-bearing/locating prohibition
   applies to registry-valid names; existing HOME/XDG identity controls remain global.
   Ambiguous registry-valid names cannot be silently stripped as credentials or accepted
   as controls. Reuse the registry predicate or verify drift explicitly.
4. For any non-control baseline name referenced by another Project but not A, reject
   A's pre-spawn admission if the retained baseline contains it. Do not silently remove
   an auth key and fall back to cached/default auth/config. This includes example names
   XAI_API_KEY and unclassified XAI_PROJECT_B_CANARY (not a claim about which environment
   names installed Grok consumes); no A child is launched
   with either. Names absent from the existing baseline do not add any value. Explicit
   shared means declared by A and at least one other Project; it remains eligible only
   under existing exact-value/non-control policy. Own/shared XAI_API_KEY and own TZ are
   concrete positive controls. A declaration can revoke a formerly global credential
   for future A admissions by requiring this opaque failure, not rewriting native controls.
5. Preserve undeclared global native auth/config/hooks/safety baseline and the whitelist.
   Constructor baseline stays frozen deliberately; no all-ambient inheritance, secret
   evidence capture, global config/HOME redirect, hooks/auth skipping or control bypass.
   Name-level isolation only: distinct per-Project values under the same variable name
   remain unsupported; a shared declaration shares the single intentional runtime value.
   Availability is narrower than name-only isolation: if one frozen runtime baseline
   contains distinct A-only and B-only declared native names, both Projects' Grok
   admissions conflict. Declaring a previously undeclared global name blocks other
   non-declaring Grok Projects. Supported multi-Project setups retain intentional
   undeclared global native auth, or explicitly share the same runtime references/value
   among all Grok-using Projects; ordinary own scoped caller values remain isolated.
   Here native/baseline names include every baseline_key name declared by any Project
   for any provider, including NODE_/BUN_/OPENSSL_ names used privately by Claude/Codex.
   Such private references can make a multi-Project Grok configuration unsupported.
   Do not co-declare another Project's private credential as a workaround: this is
   operator configuration guidance, not an enforceable private/shared flag in the
   existing plain environment_refs list. Distinct
   Project-native credential/value sources need a separate reviewed future contract.
   #16 still requires actual 2+ Projects/4+ Tasks/all three native agents under supported
   configurations; this limit does not waive its multi-Project acceptance criterion or
   imply complete per-Project credential-value separation.
6. Final environment authority admission is the last state check BEFORE actual child
   exec/spawn, including resume initialize/authenticate/load, not merely prompt dispatch.
   Snapshot owning versions plus foreign name/reference inventory; owning P/G/T CAS
   alone cannot detect another Project registering a formerly global credential name.
   Deterministic per-invocation hooks/second Store mutate own refs and foreign inventory
   after snapshot/before admission. Define exact atomic admission/dispatch-intent fence
   in design; Git/filesystem/native work runs outside SharedStore. SQLite+OS exec cannot
   be atomic: changes after successful admission are explicitly non-retroactive. A
   pre-prompt recheck may reject/stop an already admitted child, preserving native
   cleanup/outcome uncertainty; it cannot retroactively claim its environment never ran.
7. Checkpoint publishes fresh input, not environment authority. Resume derives current
   own/inventory authority afresh at its new pre-spawn admission; launch-time caller
   keys removed from own refs remain non-resumable with the same opaque category.
   Native identity/fresh input/no implicit mutating replay and Lost reservations stay
   authoritative. No uncertain native outcome is cleared by this environment fix.
8. Diagnostics remain exact owning scoped and bounded; never log/persist environment
   values, full environment/private config/credentials. Foreign-only/undeclared errors
   and A audit are indistinguishable, without foreign identities/reference inventory.
   Baseline reference conflicts can fail with a generic environment-authority category;
   no B identity/value is exposed. Cleanup #46 uncertainty is not relaxed for a pass.

## Acceptance and failure attribution

- Actual fake-child start/resume tests reject registry-valid foreign TZ and foreign
  retained XAI_ baseline conflicts before spawn; reserved RRX_ and invalid persisted
  foreign records are separate cases. Own TZ, own/shared XAI_API_KEY, undeclared global
  native auth and foreign-declared protected control canaries remain positive controls.
  Also cover undeclared caller
  key rejection, immutable native control replacement denial, GIT_/invalid refs and a
  foreign mixed invalid record containing a valid conflicting baseline reference.
- Constructor environment must be isolated by sanitized test-binary re-exec or a reviewed
  injection path actually used by new(); no ambient credential capture or parallel
  in-process set_var. Fail closed before capture unless every captured value is an
  expected synthetic marker; unknown values are never written, hashed or printed.
  Compare marker matches/booleans without assert diagnostics that print values.
  All result/output artifacts stay inside owned TempDir on Linux/macOS.
- A causal second-Store mutation after snapshot/before final admission changes owning
  reference authority and separately registers/replaces a foreign reference formerly
  treated as global. Assert no native spawn/dispatch or stale consumed intent from
  rejected admission, before matching error wording. Control proves the same prepared
  path actually starts when authoritative refs remain unchanged.
- Exercise actual Grok start and continuation consumers, not a copied environment
  helper. Compiled caller-ownership/foreign-baseline-conflict admission mutants must reach that
  consumer and die on spawn/child canary state; restored controls pass. Record exact
  compiled patch/base/head and masked/unit-only outcomes honestly.
- Independent immutable requirements/design/source gates and verified fix→rereview;
  commit before tests, required fmt/clippy/debug/release/regressions and exact final
  Linux/macOS CI. Distinguish installed native acceptance from fake protocol tests,
  default versus constrained concurrency, and preserve every unrelated cleanup failure.

## Boundaries and dependencies

Environment references/native child isolation only. No ACP, filesystem/tool/approval
policy, factory/TUI/attach, auth extraction, global settings/config bypass or schema
redesign. An additive authority helper, if needed, requires separately reviewed exact
consumer/transaction design coordinated with shared Store/provider owners. Existing
native permission/hooks/auth controls remain authoritative. Dependencies #7; reviewed
#5/#6 policies may be reused only after normal integration; #16 must dogfood the fixed
provider isolation. Root owns merge/close. Issue46 cleanup implementation has priority
when its formal design gate approves; this worktree contains proposed documentation only
until its own design gate approves.
