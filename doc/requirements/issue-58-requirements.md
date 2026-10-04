# Issue 58: Retained ownership of Task-free native Consultant work

Workflow: STRICT (native ownership, durable authority and Project exclusion).
Status: Requirements2; verified independent Req1 gaps corrected below;
requirements/design/source gates pending.
Baseline: main80452f4/schema3. MVP follow-up from #6 F1 and #19; see
[Issue58](https://github.com/shuhei-suzuki/rururunx/issues/58).

## Purpose and current gap

Required native consultation/attach must not let actual owned startup, native work
or cleanup disappear behind an ordinary terminal Session label. Project removal,
root/common-Git-directory mutations and competing admission must observe retained
ownership even before a first Session exists, after a turn ends while its server
remains owned, or when cleanup/outcome is unknown.

Actual main Store::ensure_project_idle1300–1331 uses terminal Session labels;
generic put_record_tx freezes Session actor/worktree identity but does not attest
cleanup. git::executor_reserved359–368 only counts Executor roles. These are
existing foundations, not a private Consultant hold/settlement producer.
Public #19 Design19
[02e9ea2](https://github.com/shuhei-suzuki/rururunx/blob/02e9ea2d7276973eeae7ac7efb9b3025eed9aad8/doc/design/issue-19-design.md)
proposes managed Task phase operations and excludes Task-free Consultant from
those operations; it is pending review/source, not this feature's implementation.
Public #6 design's proposed native TUI gateway is Project-only Consultant,
sole-upstream actor, text/decision-only and per-submission CAS. Interactive human
results do not prove Workflow transport success. Preserve that scope.

## Ownership and supported scope

- Scope is exact Project and optional Goal, with Task absent. Bind to canonical
  primary Project.root, repository/common-dir identity, registered implementation,
  native profile, actor/role, mode and native session/turn identities. Overlapping
  Issue names, paths or native IDs in another Project grant no authority.
  Conflict identity is canonical physical scope, not Project ID alone: overlapping
  ancestor/nested roots or a shared common-dir are checked across all registered
  Projects. Independent means non-overlapping physical effects and distinct shared
  Git metadata. Registration/root/identity changes cannot introduce an unobserved
  overlap around a held effect; include those consumers in the coherent CAS frame.
- Acquisition requires actual registered Project/current versions and applicable
  Goal authority/lifecycle under #23. A Goal-less consultation does not fabricate
  a Goal or Task. No Task-worktree path/namespace, Executor/Reviewer/ApprovalReviewer
  conversion or Task dispatch/completion permit is introduced.
- Actual local runtime composition selects supported profiles; native/model JSON,
  public role names, configuration self-labels or first-use success cannot enroll
  a producer. Every fresh Task-free native entry requires this actual retained-owner
  port before effects. The shared Store denies EVERY generic nonterminal Task-free
  Session insertion/update, including terminal-to-live and identical-body version
  bumps, regardless of provider/agent/model/profile/role/path labels; controlled or
  unknown implementations have no generic exemption. A direct adapter or wrapper cannot
  evade this fence. Initial terminal factual history grants no launch/release proof.
  Each implemented profile must have reviewed native conformance for
  its complete required startup/turn/service/cleanup workload under #6 F1.
  Unsupported profiles reject before reserve/process/native-visible effect;
  required Consult/attach acceptance stays open rather than waived.
- Before ANY native startup/probe/discovery side effect, the actual owned supervisor
  must receive an opaque, nonserializable retained-owner capability captured from
  successful exact scoped acquisition. A durable row alone, supplied PID, copied
  UUID or claimed runtime identity cannot stand in for the live owner.
- Durable identity/lifetime and input/source pins are immutable. Transfer ownership
  once to the actual supervisor; do not clone a terminal-release capability or let
  an untrusted collector/observer manufacture it. Generic Session/Record writers
  cannot hide or mutate its owner-bound factual Session, including identical-body
  version bumps; actual supervisor observations have a dedicated private path.
  Metrics and bounded audit remain separate and cannot grant release/admission.
  ALL current input submissions, consumed-intent publications, operation grants/
  ALLOW replies and input-capable attach bindings use one private owner-bound port.
  It requires the actual live opaque capability, exact retained hold and current
  source/Session/Project/Goal/full-lock versions in the same transaction. Correct
  public IDs/versions or a scope self-match never grant admission; generic/public
  scoped CAS refuses. Only that port may distinguish its exact owner from a
  competing hold. Pending #6 TUI admission must compose this ownership check with
  its existing planned per-submission CAS, not manufacture an alternate ledger.
- Retain Session-less setup, pending input, live service, native background work,
  dropped futures, publication conflicts and Lost/unknown outcomes. A completed
  native turn does not release a still-owned server or the consultation lifetime.
  Missing Session/PID, terminal public labels, process-group hints, interrupt/close
  acknowledgements and configuration claims are not cleanup proof.

## Exclusion, settlement and recovery

- Define the actual root/common-dir conflict scope in design from complete effect
  inventory, including preserved mandatory native hooks/filesystem and delegation.
  A Consultant label is not proof of read-only behavior. Reject conflicting native/
  Git/write admission and Project removal while retained work may affect that
  scope. Independent Projects remain runnable; no runtime-global blanket hold.
  The exclusion query also includes actual Task-managed ownership from #19 with
  reachable shared root/common-Git effects, including Lost/uncertain Task work;
  Task identity or separate worktree never proves disjoint shared effects.
  Conflict exclusion is symmetric and durable: each out-of-lock root/common-dir
  Git/worktree mutation, removal/root change and effectful pack publication acquires
  an actual retained conflict reservation atomically against all applicable holds,
  complete locks and in-flight mutations, before its effects. Consultant acquisition,
  fresh admission and continuation perform the reverse checks in the same Immediate
  transaction that acquires/adopts their exact authority. Retain the effect owner
  through cleanup/settlement/conflict; a prior idle read or version pin alone is
  insufficient. Every side uses the shared exclusion protocol; unsupported missing
  consumer producers block that scope/profile's production readiness.
  Same-Project concurrency follows actual reviewed compatibility: live Consult vs
  Task native phases in other worktrees or another Consult may coexist only with
  continuously enforced disjoint/compatible effects; separate paths/role labels alone
  are insufficient. Task worktree create/adopt/remove and Project-root/shared-Git
  mutation require their conflict reservation and refuse intersecting Consult work.
  Unknown compatibility is a scoped conflict. Status names the blocking owner and
  supported stop/close drives real settlement; idle TUI retains its actual lifetime.
  This conservative fallback does not satisfy mandatory native4+parallel acceptance:
  #6/#19 and actual Git consumers must establish supported compatible profiles.
- Every actual consumer reads retained ownership under its current authority/CAS:
  Project removal/mutation, root/common-dir Git/worktree operations, native and
  input admission, applicable Goal completion/pack publication, capacity and restart
  reconciliation. Include every Goal terminal/cancel/fail surface. Enumerate exact
  call sites before source. Do not rely solely on
  Session state, executor_reserved or a cached ready/status result.
- An exact owner may perform its separately allowed factual observations, denial
  and cleanup without treating its own hold as an unrelated competing owner.
  Native/Source/Git calls remain outside SQLite/SharedStore locks. Inactive-owner
  closure never grants new native work or relaxes native current-input/Broker guards.
  EVERY fresh submission, ALLOW/operation grant, input-capable attach, continuation
  and owner publication granting fresh input/operation authority revalidates current
  Registered Project and
  applicable accepted nonterminal Goal authority/lifecycle under #23. No broad
  Blocked/lifecycle exemption. Authorized Goal cancel/fail may conservatively close
  its logical lifecycle while retaining all native holds and denying new admission;
  completion still requires actual settled work. Exact factual observations, historic
  DENY and cleanup (including actual verified settlement/release) keep their separate
  inactive-owner rules and grant no new input/operation authority or progression.
- Release is private actual supervisor evidence of no dispatch/settled setup, or
  known exact native outcome AND all profile-required resource cleanup. It pins the
  same immutable operation/scope, input/Session versions and actual owner; publication
  and permitted release commit atomically with bounded audit. Unknown/Lost/native
  escape or failed publication keeps the hold and exposes attention.
- Keep input admission, native completion, resource settlement and accepted Goal/
  Task success distinct. A Consultant receipt cannot close a Workflow phase,
  certify a review, resolve a criterion or release another owner.
- Runtime restart never reconstructs a live owner from persisted hints. Retain
  unknown ownership until separately reviewed #14 recovery proves actual cleanup
  or safely fenced no-effect and excludes the former owner. Human labels/boot hints
  cannot fabricate death or unlock replay. Lost remains absorbing in ordinary APIs.
- Migration coordinates one schema/writer-epoch history with #19, fences ALL generic
  and private writers including old open connections, and refuses unresolved legacy
  workload. Preserve actual old bytes/history; no legacy terminal-label ratification.
  Final production profile readiness includes actual #14 recovery, not fake receipts.
  #19's generic Task-scope live-Session fence, managed operations and complete
  locks/Workflow claims are mandatory in the same or an earlier composed epoch;
  #58 cannot ship first with Executor-only or unmanaged Task exclusion.
  Whole-schema upgrade deliberately drains every actual older owner and requires
  reviewed #14 recovery for uncertainty, under #19's atomic migration preflight.
  Failure leaves original schema/bytes/history unchanged; retagging legacy labels
  as scoped holds cannot prove old native owners fenced. This documented upgrade
  availability limit is distinct from normal current-epoch scoped exclusion, where
  genuinely disjoint Projects remain runnable. No silent legacy ratification.

## Native TUI and continuation

- Reuse #6's Project-only native TUI gateway: one actor/upstream connection, exact
  stored native session, bounded text/decision inputs and one unresolved submission.
  Every human/native-originated inference submission and operation grant must use
  the pending composed private-owner and per-submission source/Session CAS before
  wire; no second ledger. Goal-scoped interactive TUI/attach remains Unsupported
  until separately reviewed #6 conformance extends its Project-only gateway.
- Native-originated steer/compact/review/inference either consumes that same genuine
  current admission or is explicitly unsupported before effect. Historical DENY
  keeps its separately applicable exact request/turn authority; it cannot release
  lifetime ownership or acquire fresh input/operation authority.
- Attach binds only the live exact owning gateway, with its retained hold. A dead/
  terminal seed does not reopen by ID. Continuation uses a separately validated
  exact prior owned outcome/settlement and fresh admission, with existing native
  UUID/input semantics; unknown cleanup blocks it. Preserve native config/auth/
  model/effort/hook/UI defaults and existing unsupported Task interactive conversions.
  Product34 native `rrx attach <task>` remains mandatory/open under #6/#11/#15/
  #19/#14 composition. Project-only Consult attach does not satisfy or waive it.
- General detached-native-command containment stays #6 F1. This issue consumes a
  real supported cleanup contract; it does not introduce privileged infrastructure,
  change platform scope or assert arbitrary descendant death from group cleanup.

## Bounds, verification and delivery

Design finite ownership/history/turn/reference/serialized/nesting/query bounds and
checked versions, including coherent complete Project-level exclusion scans and
capacity exhaustion. No truncated scan proves idle or permits a release. Persist
only allowlisted identities/digests/versions and bounded reason codes, never secrets,
environment values, native transcripts or prompt text. Status is bounded, scoped,
read-only and honest about uncertain owned work. Trust is the application API;
malicious linked code/direct same-UID machine/DB actions are outside it, not secured
by a biological Human or OS sandbox claim.

Real controlled/private and required native controls cover startup before Session,
retained live server after turn, TUI input/decision ordering, same-scope continuation,
generic terminal/role/owner forgery, conflicting writer/publication failure, dropped
owner, Lost, restart, inactive closure, Project remove and common-dir operations.
Use two physically disjoint Projects plus nested/shared-scope negative controls;
never modify unrelated user repositories. Add same-Project compatible/conflicting
profiles, already-running peer→Lost enforcement, stop/close, Blocked Project and
terminal Goal controls; no blanket conflict fallback receives parallelism credit.

Compiled causal mutants must reach the actual removal/admission/settlement/TUI
consumers: omit hold lookup, release on terminal label/PID absence, forge owner from
row identity, treat a turn as lifetime settlement, omit root/common-dir exclusion,
or bypass a submission CAS. Controls prove exact settled positives and independent
Project progress. Also mutate owner-less version-correct input/ALLOW, unknown-label
generic live writes, reverse effect reservation and physical overlap lookup. Setup
refusal, timeout or helper-only checks are not consumer
kills. Restore exact source and passing controls after each mutation.

Run shared-state/native/Git/registry/Workflow regressions, fmt/clippy/build, exact
Linux/macOS CI and independent immutable source reviews. Update README/master
designs to actual behavior. Dependencies: #19 coordinated private authority/epoch;
#6 profile/TUI conformance; #11 consult driver, #14 recovery, #23 Goal consumers and
#26 Project management. Requirements research is runnable independently; no missing
producer is replaced by a fixture. Keep #58/#6 acceptance open until actual
composition passes; no full #16 dogfood/MVP completion is claimed here.
