# Issue 51 requirements: scoped Grok native environment references

Risk: STRICT. MVP-blocking follow-up to #7 and actual integration acceptance #16.
Goal §6 separates credentials/environment references by Project. Requirements/design
approval precedes native/shared implementation. No Grok, Store or provider source is
changed by this requirements-only head.

## Verified problem and evidence limits

The actual public GrokAdapter from main4851fcd (unchanged at docs-only d56f2bf) was
launched with an exclusively synthetic constructor environment and a real isolated
fake ACP child. Project B was persisted with RRX_PROJECT_B_CANARY and
XAI_PROJECT_B_CANARY references; Project A declared neither. Caller ordinary RRX_
value synthetic-caller-B and whitelisted constructor XAI_ value synthetic-baseline-B
both reached A's child. The model-independent child capture precedes ACP processing.

The global XAI_GLOBAL_AUTH_CANARY declared by no Project also reached the child; this
is an intentional native baseline control, not a defect. Probe1/1 passed1.82s,
terminal Exited/failure null/actual OS exit_code null. No installed native Grok, actual
credentials/private configuration, production worktree or audit secrets were read.
The fixture compiled only after removing an invalid private helper call; that initial
compile failure is not test/mutation credit. This is actual child propagation proof,
not native inference, successful auth or general provider safety acceptance.

Grok currently filters constructor variables through baseline_key, correctly avoiding
ambient inheritance of all variables. environment() subsequently accepts ordinary
RRX_/locale keys without checking own references and does not filter known foreign
registered references from that baseline; supervise uses env_clear().envs directly.
Do not describe all global native baseline inheritance as the bug.

Unmerged Claude/Codex policy implementations currently show the intended ownership
pattern: caller keys must belong to own refs; known foreign reference names are removed
from the native baseline (Claude deliberately preserves specified OS identity names).
They are comparison inputs pending exact normally integrated reviewed contracts, not
a claim of universally safe or accepted providers. #5/#6 source owners stay separate.

## Required behavior

1. Resolve the exact persisted owning Project and the authoritative registered
   Project reference inventory. Reference names, not values, are state. Respect
   Project/Goal/Task/Session activity, ownership, raw/semantic version and native lock
   guards already required by #7. Malformed reference names fail closed before child
   spawn/dispatch; do not accept a caller-supplied Project/reference list as authority.
2. A caller-provided key must be explicitly declared in the owning Project's
   environment_refs, even when it is an ordinary RRX_/locale key. A name declared
   only by another Project is rejected. Explicit owning/shared refs can be used
   within existing native-control restrictions; no broad all-environment passthrough.
   Test fixture/runtime metadata names must be explicitly owned, not privileged
   through an undocumented RRX_ exception.
3. Before native whitelist application, remove constructor baseline names known to
   belong exclusively to other Projects. The inventory must consider persisted
   foreign references conservatively, including inactive records until a reviewed
   removal policy explicitly resolves them. Explicitly shared names owned by the
   current Project remain eligible under existing immutable native-control policy.
   Any OS identity exception must be narrow, justified and tested; Project reference
   metadata cannot make an identity/routing/control override eligible.
4. Preserve undeclared global native auth/config/hooks/safety/runtime baseline,
   constructor snapshot semantics and the existing whitelist. Do not read/extract
   credentials for evidence, inherit all ambient variables, redirect global native
   config/HOME/routing, disable controls or replace native baseline values through
   caller/Project settings. Existing native-control exact-value protection, Git_
   exclusion and malformed/NUL checks remain authoritative.
5. Freeze the selected child environment from a scoped snapshot; verify its current
   authority before any actual start/resume dispatch. Async Git/filesystem/native
   work happens outside SharedStore. Reference inventory replacement/registration
   and own Project changes during preparation cannot make an obsolete environment
   eligible. The design must define the atomic state admission/dispatch-intent fence
   and distinguish that admission from impossible atomic SQLite+OS spawn. Snapshot
   CAS of the owning Project alone does not detect another Project adding a reference
   to a formerly global name. No blanket lock around filesystem/native operations.
6. Resume/checkpoint must preserve native identity/known ownership and explicit fresh
   input requirements; they may not replay a previous mutating prompt or refresh
   runtime controls implicitly. Check current environment authority at whichever
   boundary actually selects/sends the continuation. Stale selected authority is
   rejected/retained conservatively; no inferred process death or uncertain outcome
   is made retryable by this environment fix.
7. Errors/audits preserve exact owning scope and stable bounded categories. Never
   persist/log baseline/request values, full environment, private config or secrets.
   Reference-name diagnostics must remain scoped, and no unscoped foreign metadata
   is copied into Task input/context. Unknown native cleanup remains Lost/reserved;
   #46/shared cleanup is not weakened for an environment pass.

## Acceptance and failure attribution

- Real isolated fake-child tests reproduce/reject both observed foreign ordinary and
  native-whitelisted paths; own refs, explicit shared refs and undeclared global
  native auth canary remain correct positive controls. Also cover undeclared caller
  key rejection, immutable native control replacement denial, GIT_/invalid refs.
- A causal second-Store mutation after snapshot/before final admission changes owning
  reference authority and separately registers/replaces a foreign reference formerly
  treated as global. Assert no native spawn/dispatch or stale consumed intent from
  rejected admission, before matching error wording. Control proves the same prepared
  path actually starts when authoritative refs remain unchanged.
- Exercise actual Grok start and continuation consumers, not a copied environment
  helper. Compiled caller-ownership/foreign-baseline-filter mutants must reach that
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
