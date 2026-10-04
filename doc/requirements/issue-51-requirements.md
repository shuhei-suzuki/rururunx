# Issue 51 requirements: scoped Grok native environment references

Risk: STRICT. MVP-blocking follow-up to #7 and actual integration acceptance #16.
Goal §6 separates credentials/environment references by Project. Requirements/design
approval precedes native/shared implementation. No Grok, Store or provider source is
changed by this requirements-only head.

## Verified problem and evidence limits

The actual public GrokAdapter from main4851fcd (unchanged at docs-only d56f2bf) was
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
   malformed/forbidden records cannot grant authority or alter global controls. An
   operator diagnostic, if emitted, is scoped to its owning foreign Project, never
   copied into A's context/audit; no values or foreign identity in A's error.
2. Preserve the registry's RRX_ reserved-name prohibition. Caller RRX_ keys and routing/
   loader/native-control names are rejected, even with identical baseline values.
   No new runtime bypass/channel is added. Existing fake ACP metadata must move to the
   generated fixture executable/arguments or other fixture-owned data, not LaunchRequest
   environment. Analyze all current producers/harnesses before implementation.
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
   Ambiguous names cannot be silently stripped as credentials or accepted as controls.
4. For any non-control baseline name referenced by another Project but not A, reject
   A's pre-spawn admission if the retained baseline contains it. Do not silently remove
   an auth key and fall back to cached/default auth/config. This includes known credential
   names such as XAI_API_KEY and unclassified XAI_PROJECT_B_CANARY; no A child is launched
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
   #16 cannot claim complete per-Project credential-value separation from this contract.
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
  key rejection, immutable native control replacement denial, GIT_/invalid refs.
- Constructor environment must be isolated by sanitized test-binary re-exec or a reviewed
  injection path actually used by new(); no ambient credential capture or parallel
  in-process set_var. All result/output artifacts stay inside owned TempDir on Linux/macOS.
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
when its formal design gate approves; this worktree stays requirements-only meanwhile.
