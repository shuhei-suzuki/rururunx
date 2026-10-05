# Issue 9: staged formal-review refusal integration

Status: Design1 candidate, NOT implemented or source-qualified. This is a finite
integration of the already approved Req38/Design4 refusal contract. It creates no
ReviewSet, allocation, certificate, profile, Human authority, native preparation,
settlement producer or database upgrade. Whole Issue9 acceptance remains open.

## Controlling contract and exact inputs

Req38 plus its wording clarification at
`caa4390780f483f2d29f8182a36f65f924caf010` is normative (requirements SHA256
`0af45849f067036815d36b3c43adb74515243c7a220b7a5789f379e6f722fcfb`). Design4 is
`7972b85c04a903d5ce148e73d62e662dd8ab7611`; subsequent `4f084ee` only clarifies
the preclosure handoff wording. Its native review `6afc16ee-0e24-4eae-a339-1614472a6cd4`
approved the narrow Design4 delta with a Low wording clarification, not source.

Existing requirements quoted here, without changing WHAT:

- I9-AC-21.j: "production Workflow formal-review consumers return
  ReviewGatingUnavailable before claim/reservation/native input" and "no legacy
  weaker single-Reviewer fallback". The same criterion explicitly covers
  Requirements/Design/Implementation/Security Review, QUICK/STANDARD/STRICT and
  applicable initial/retry/resume_gate/request_finalization consumers.
- I9-AC-21.k: "Historical legacy Passed review records stay factual read-only
  history, but cannot satisfy new formal review gates or downstream
  PR/MergeGate/finalization." Live/evaluating old owners require deliberate
  drain/recovery, rather than retrospective new authority.
- Design4 requires an independently reviewed integration against actual compiled
  19/43/native ports before production code. Missing producers are unavailable,
  not a test-mode positive or a weaker formal fallback.

| Pinned tree | Actual consumer / limit |
| --- | --- |
| main `768f84319cd2a73e14cd39336eb12d99e9be81a7` | schema3; `workflow.rs` has one `PhaseAttempt.session_id`, selects `task.reviewers.first()`, and validates a public `Evidence.review_approved` bool. No ReviewSet/certificate consumer. |
| #43 `b191b466d5dea303585cfaf6968c6fb178be79cd` | `preflight_native_adapter` checks actual/probed role and `PreparedInputAdmission`, probe identity and an always-unavailable private binding-composition predicate before clear_hold/prepare_pack/reserve/marker. It protects Executor AND Reviewer launch readiness, not ReviewSet certification or downstream legacy Passed use. |
| #19 `a8aa95108bfe2fc098f5333f7bb5cf445c4c74a9` | Unmerged schema5 CPP prototype remains unapproved as a whole. Selected Control/custody mechanics are separately reviewed; the `Factory` producer is cfg(test) and gives no managed allocation/admission/settlement port. |
| #6 `c8ce5a86ebc2278ed6ca9e4ecdf7d127e5628f18` | Production Availability is EMPTY; no workload/backend producer. A cfg(test) component route is not readiness. |
| #60 `81e7c29ce2cc51746308874a57f8bc9aed888dc1` | Private GitPool retains selected process-local jobs; no exported setup capability or durable managed receipt. Source2 reviews are separate and open at this inventory. |

Read source through these exact commits. Before implementation normally compose
current reviewed main into the owned Issue9 branch, reconcile the #43 guard by its
normal ancestry when applicable, and refresh line references/diff inventory. Do not
copy unapproved #19 authority code or reset merged #41/#14 policies. No main edits.

## One typed refusal; no alternate readiness constructor

Introduce an error type `ReviewGatingUnavailable` in the Workflow module, carried
by its existing `anyhow::Result` APIs and recognized by downcast. It states that
formal ReviewSet authority/native-profile integration is not composed. A private
zero-effect `require_formal_review_ready()` always returns this error in this
slice. There is no public bool, policy/role/JSON override, native capability that
sets it ready, cfg(test) success constructor or conformance ingress. No automatic
retry classification: callers receive a configuration/readiness hold, not a
failed native opinion or spent review round.

This error comes from I9-AC-21.j, independently of #43's
`NativePreflightRefusal`. Do not duplicate its adapter selection, probe or
PreparedInputAdmission validation. Formal refusal runs before those new-review
effects; nonreview Executor selection retains #43 unchanged. A false capability
advertiser cannot turn this refusal into readiness. Neither predicate grants input.

This stage changes no persistence enum/schema/epoch, ownership index or generic
Store writer. It fences the actual Workflow consumer; it does NOT claim incompatible
already-open writer fencing, whole legacy upgrade, certified-history migration or
proof against direct generic old writers. Those I9-AC-21.k/19/43 integration gates
remain required before whole-source/production enablement. Read-only status exposes
historical labels as history, never as a new certificate.

## Exact main callsite inventory and selected wiring

Line references below name the pinned main source, not the older Issue9 branch.
Formal phases are exactly `Phase::actor() == Actor::Reviewer`: RequirementsReview,
DesignReview, ImplementationReview, SecurityReview (`workflow.rs:74`). Protected
downstream publication phases are Pr, MergeGate, and Cleanup completion. They may
not consume legacy approval as new formal authority while readiness is absent.

| Actual callsite | Selected refusal placement / retained behavior |
| --- | --- |
| `step` at 796; `next_phase`, source capture 820+, clear_hold/context/reserve 901+ | With no active attempt, classify the selected next phase and reject formal review or NEW Pr/MergeGate/Cleanup before `inputs`, context preparation, hold clearing, history/claim/marker or external gate. Recheck a newly selected formal phase before any invalidation/escalation-created context; do not infer eligibility from stale history/config. Finished workflow/status reads are factual, not permission. |
| `prepare_pack` and its initialize/escalate/invalidate/apply_outcome callers | Central formal-phase guard at entry, before context capture/publication. This closes successor-context paths: a successful Tests/Commit can currently prepare the next review pack before the next `step`. Do not publish that pack via another entry. Initialize or nonreview work remains outside this new review refusal; no blanket rejection of all Task creation. |
| `prepare_agent` at 1107+ | Retain #43 role/admission/identity checks and original claim policy; no parallel second adapter selector. Defense before a new formal native dispatch if an already admitted legacy preparation reaches this internal route. Failure does not reset a marked claim, fabricate terminal status or authorize replacement. |
| `poll` at 1206 | Keep status lookup, exact persisted-Session comparison, nonterminal Running/Waiting observations, unbound-owner holds and transport-failure handling. Do not add an unconditional top-level formal guard that prevents observing an existing native owner. A successful terminal reviewer reaches the guarded evaluation/publication path below, rather than certifying from transport success. |
| `evaluate` at 1674 | Before new formal review evaluation or NEW Pr/MergeGate effects, reject before owner refresh/source recapture, Evaluating claim, external `PhaseGates::complete` or journal publication. No bool approval from a third-party gate bypasses this. Existing already-claimed unknown evaluations remain held/read-only through poll/recovery. |
| `apply_outcome` at 1865+ | Reject `Passed` for a formal review or Pr/MergeGate/Cleanup completion before fresh sources, completed-map/Task/context/active mutation. This includes a genuinely previously journaled legacy Passed outcome replayed by `poll`; retain the journal verbatim. Waiting/Failed factual outcomes do not become approvals. A previously admitted irreversible operation's observation must remain factual, not be erased or repeated to get acceptance. |
| `retry` at 2010 | Preserve existing structural, marked-unbound #14 and live/Lost Session guards/error precedence. After those read-only checks but before RetryEvent/active/Task/Record/audit mutation, refuse restarting a formal review or a protected downstream attempt. Never release an existing owner because readiness is missing. |
| `resume_gate` at 1600 | For formal reviews or Pr/MergeGate, refuse after actual attempt/state validation but before adapter status/gate recapture/evaluation. Ordinary `step` status observation remains available. Existing Cleanup reconciliation is separate as described below. |
| `request_finalization` at 1427 | Preserve reason/lifecycle/finished-QUICK shape checks, then typed refusal before MergeGate source capture, configured_phases/finalizations/finished/context/Task mutation. A legacy Quick Pr/Passed record supplies no current formal basis. |

QUICK already includes ImplementationReview before Pr (`phases`, line124); STANDARD
and STRICT add the appropriate earlier/security reviews. The new requirement is
not that QUICK executes every STANDARD review. It is that its own required formal
review and downstream Pr cannot evade the same unavailable readiness predicate.
Reject downstream Pr/MergeGate even if a malformed or older configured list omitted
a Reviewer phase; do not derive a permission from an empty prerequisite list.
Existing raw transition/body validation is retained independently.

`step`'s `finished => Finished` and `snapshot()` may report old final labels. They
perform no new grant and do not convert history into formal evidence. Any subsequent
new formal consumer (including request_finalization) must still refuse.

## Existing owned observations, cancellation and cleanup

Do not modify `cancel`, `fail_task`, `release_terminal_reservation`, #41 owner-local
preparation release, #14 marked-unbound retry fences, native adapter stop/status or
Session/lock ownership semantics. Their current guarantees and limitations remain.
Missing formal readiness never implies death, resource refund, terminal input,
replacement/native retry, receipt or cleanup success. Universal Lost/managed epoch
contracts are NOT newly implemented by this error.

For an already ACTIVE Cleanup attempt, permit the existing frozen-authority
evaluation/reconciliation and genuine factual gate journaling; do not create a new
Cleanup claim from legacy Pr/MergeGate. Existing unknown external claims remain held
for their exact #13/#14 reconciliation. A known factual cleanup outcome remains in
the observation journal, but its Passed application cannot newly mark the Task
Completed or formally finalize a legacy workflow without the missing authority.
Thus physical reconciliation is preserved while new formal completion is held.
No new cleanup producer or permission is created; existing scoped gate predicates,
prior-observation identity and source/lifecycle restrictions still apply. If this
boundary needs broader producer changes, stop that source portion and report it;
do not invent an inactive-owner exception.

Nonreview actual gate facts may be journaled before successor formal context refusal.
For example Tests Passed remains an immutable actual observation, while the current
`apply_outcome` cannot finish its context/Task transition if it prepares a blocked
ImplementationReview pack. The unchanged claim/journal is held; do not rewrite a
test result, manufacture a formal context, clear reservations or infer resource death.
This is an explicit availability consequence of the staged refusal, not a readiness
positive or rollback of already observed effects.

## Consumer controls, compatibility and source gate

Implementation uses actual Workflow entrypoints with dedicated Store/Project/Task
fixtures and counters for Sources.capture, gate invocation, adapter probe/start,
and complete DB snapshots (owners, Records, ALL Contexts, Sessions, locks, audits).
Before-input refusal requires zero new relevant capture/effect/claim/model delivery
and byte-identical durable state. Internal successor and already-observed outcome
controls compare before/after at that exact boundary: earlier actual observations
are retained, not falsely claimed to have never occurred.

Cover all four Review phases/classes, Quick implementation review and Pr,
retry/resume_gate/request_finalization, legacy Passed→Pr/MergeGate,
already-journaled Passed replay, formal successor-context creation, active native
Running/status-error/unbound/Lost observation, and active Cleanup reconciliation
versus refused new Cleanup/completion. Cancel and existing #14 terminal-release
negative/positive component controls remain meaningful and never mint managed death.
Historical fixtures are explicitly schema3 factual state, not new typed allocation;
no Session JSON/private SQL seeding/ready flag certifies a positive native owner.

Compiled omission controls must reach each public branch, context preparation,
downstream approval application and Quick/finalization bypass. A separate original
control proves actual running-owner observation and cleanup factual journaling
remain reachable; an overbroad refusal mutant must fail that consumer. Never claim
a mutant kill merely because an earlier unrelated producer guard already refused.

Retain existing tests and their actual semantics/evidence. Main's successful
single-reviewer FakeAgent fixtures are legacy mechanics, not current ReviewSet
positives. Their formal success expectations will conflict with the deliberate
withdrawal; do not add fake readiness, ignore/delete them, relabel failures PASS or
claim whole regression green. Refactor only a concretely reachable nonformal or
historical-observation control when it preserves the tested guarantee, label lost
positive coverage and keep remaining migration dependencies open.

The #43 staged tree already has 55 legacy Workflow failures pending genuine #19
migration, despite approved negative controls. Current #19/curated custody also has
separate failed full runs/CI with causes not established. Neither this design nor a
new negative test waives those failures or makes #43/#19 deployable. Source tests
must report the exact resulting head, selected controls and full failures separately.
Whole positive profile/migration/native/MVP gates stay open.

Commit this bounded design clean, obtain TWO independent explicit design reviews
against the pinned actual consumers, verify findings and fix/re-review before source.
After approval, use one coherent implementation commit and actual controls/mutations,
then independently review that immutable source. No broad #9 or PR39 acceptance can
be inferred from this selected refusal component.
