# Issue 9 design: configurable independent Review Sets

Status: Design1 candidate, provider-neutral and not implemented. The controlling
[requirements](../requirements/issue-9-requirements.md) are the Req38-gated revision
plus its Low wording clarifications at caa4390780f483f2d29f8182a36f65f924caf010,
SHA256 0af45849f067036815d36b3c43adb74515243c7a220b7a5789f379e6f722fcfb.
That document, including every stable I9-AC key and its closing/post-closure staging,
is normative; this design organizes consumers and authority without replacing its
limits or creating new policy. Names below are proposed private types, not compiled
kernel APIs. Actual main has one Reviewer Session per Workflow phase; it has no
ReviewSet implementation. Source qualification must use the real reviewed
19/23/43/native ports and a separately committed integration design before code.

## Data and authority

Use immutable typed PolicySnapshot, ReviewerSlot, ReviewSet, RoundProposal,
ReviewRound, MemberInput, MemberOutcome, Finding, FindingDisposition, Verification,
LineageReference and ReviewCertificate envelopes. Every durable object carries
exact Project/Goal/Task, owning Set/round/slot where applicable, format version and
bounded referenced artifact identity. Facts and serialized IDs do not construct
native ownership, controller authority or recognized Human authority.

Import the ONE canonical trusted-ingress derivation/type owned by23. Actual15/24
handlers construct production Human authority; actual10 rejects recognized
Human-only effects from Broker/native channels.9 owns those convergence consumer
controls under9.i/9.j/21.i. A separate controller may instantiate an EXACT already
Human-approved initial digest and execute its frozen scheduling/retry/cancellation
branches; it cannot invent Human approval, new policy content, post-opinion roster
or confirmer changes, blocker clearance or budget reset. Application-channel trust
is explicit; this claims neither biological identity nor same-UID OS/DB secrecy.
These producers remain pending, not test-library product ingress.

PolicySnapshot freezes activated Runtime/registered Project governing rule/skill
source, exact approved digest, roster/model/effort, mode/quorum, independent floor,
risk/security behavior, confirmer assignments, clocks, schedule/channel profile and
byte/round/resource bounds BEFORE target opinions. Task-authored edits are reviewed
proposals, never governing rubric for their author. Resolving and hashing a source
is not trusted activation; base-sync cannot auto-approve new content. Scope-changing
updates create explicit history through the required authority, never in-place
reinterpretation of a started round (8/9/12).

## Bundle and input consumer

Define one typed bundle contract owned by9 and consumed unchanged by20: exact
scope, canonical primary/common-Git and bound Task worktree identity, committed
Task-base/baseline/target HEAD AND trees, relevant dirty/ignored source versions,
CPP/checkpoint/promoted-reference provenance, governing source, factual coverage
manifest and bounded bytes. The same frozen core bytes/digest reach every slot.
Per-slot specialization is structurally additive with separate identity/source/hash;
record both core and COMPLETE frame hashes. Focus-only activation cannot exclude
mandatory rubric, rules, severities or facts. Total rendered frame includes mandatory
claims/rules/specialization/wrappers; source-budget size is not its byte limit.

Full Task-base→target coverage is the default. Reduced delta coverage requires the
qualifying exact frozen roster/policy/source baseline's COMPLETE settled delivery
and result, ALL cumulative baseline→target changes, ALL unresolved required claims
and affected unchanged consumers with supported current impact completeness. A
prior delta APPROVE is not a new full baseline. Missing/stale/incomplete12 impact
basis forces full delivery; full diff is not semantic impact proof. Controlled
Git-backed consumer fixtures label impact_basis=controlled-fixture and cannot
certify absent production12 or20.20 depends on9; no reverse merge prerequisite.

Initial input, expansion, rules/source slices and manifests always exclude raw
executor conversation, runtime DB/sidecars and current-round peer outputs. Store
outputs outside reviewed/expandable source eligibility; exact/symlink references
cannot bypass exclusions. A conflicting mandatory reference holds before input.
After every preceding member safely settles, later rounds deliver bounded mandatory
prior-claim manifests and all required unresolved claim text, retain original
finding identity/severity/text/verdict and label prior exposure. Exact referenced
resolved history is expandable within the same budgets; unverified actor fix claims
are never facts or clearance. This is runtime-controlled input independence with
explicit residual native filesystem visibility (8/14.e).

Source observation/hash/Git work runs outside SharedStore. Snapshot, observe through
actual60 owned source/job effects where native execution is involved, then atomic
publication and pre-input CAS validate the exact original source/lifecycle/locks.
A read-only label, Session JSON or selected group death creates no source-job lease.
Mandatory output/acquisition/serialization bounds stop overflow before semantic
publication; byte overflow preserves bounded evidence and never truncates to approval.

## Native profiles and current feasibility

A registered profile consumes actual production5/6/7 permission construction,
read-only denial, owned startup/normal/forced cleanup and channel qualification.
Preserve default auth/hooks/rules/memory/history behavior. Capability labels,
provider family, fresh SID, linked worktree or docs alone do not prove independence
or all-descendant settlement.21.j's sealed fixed conformance ingress uses these SAME
actual producer ports without formal gating authority; there is no public bypass.

Current candidate feasibility under8.l is UNKNOWN, not qualified:

| Candidate | Concrete known basis | Missing proof / owner |
| --- | --- | --- |
| Claude | Req38's retrieved vendor basis describes repository memory shared across worktrees; observed CLI2.1.283 only | Actual installed read/write timing and key predicates, same-round peer-free schedule, owned read-only/cleanup conformance:5/9/19/16 |
| Codex | Req38 records CLI0.160.0 and distinct memories feature versus field defaults;6/F1 actual detached-tool cohort is unresolved | Actual effective channel activation/key/timing plus genuine managed cohort settlement:6/F1/9/19/16 |
| Grok |7 supplies an actual native adapter; no qualified default-preserving channel/settlement declaration is supplied here | Actual installed channel inventory/key/timing and normal/forced owned settlement:7/9/19/16 |

This table claims channel capability context, not observed peer-finding injection;
no private user memory/config was read. Before any positive, templates must have
actual owned native fixture evidence and instantiate proved installed-version/
configuration/key/load-write/source/roster/schedule predicates on the real round.
A peer-free starting snapshot is sufficient only when actual ALL relevant loads
precede ANY peer write under an enforced finite schedule. Unknown reachable current-
peer channels refuse; no universal family-equality floor or memory-disable workaround.
At most three candidate investigations per required configuration then21.l requires
named public owner/proof/scope-decision attention. Required repeated-family/4+,
2-reviewer and Triple positives remain open; a candidate refusal cannot close them.

## Workflow and member ownership

Workflow delegates one exact Reviewer claim and immutable Task CPP/locked revision
to a private ReviewSet roster. Allocate one distinct owner/frame per member without
incrementing Task/context authority per Reviewer or cloning CPP. Bind Set/round/slot,
agent/model/effort, actual full-frame SHA/bytes, common core, governing/input/source
pins and original marker P/G/T/W/full lock frame.43's single native binder is not an
N-way delegation. Never relax its default ownership checks by role or public JSON.
Executor/Consultant/ApprovalReviewer cannot impersonate formal roster slots.

A genuine supervisor retains the sole managed operation across caller Drop and
lends actor/lifetime-bound preparation capability. Actual setup, consumption,
callbacks, cleanup and receipt producers must compose with19; lost futures, returned
Session IDs or selected process groups cannot reconstruct it. New rounds use fresh
native Sessions and exact fresh member admission. A genuine original consumed frame
is distinct from a later observation or fresh prepared input. No previous terminal
owner or receipt authorizes another dispatch.

Absent profile, capability or private producer returns typed ReviewGatingUnavailable
BEFORE phase/context reservation, marker, process or model input; no weaker legacy
formal fallback. Workflow's single-Reviewer path remains historical behavior, not
successful configurable gating. ApprovalReviewer means it does not execute the
requested action; its future9/10 decision slot still needs actual native lifetime/
input/settlement authority. No current DecisionTask port or native-free exemption
is invented. Review Runner integration is independently reviewed against compiled
19/23/43 ports before production code.

## Scheduling and settlement

A bounded RoundProposal freezes target/policy/source/hash refs and charges its exact
metadata; wholly queued work is not an admitted round. Every initial/queued member
admission revalidates exact qualification and elapsed schedule. Concurrent-only
profiles acquire COMPLETE global/Project/provider cohort permits all-or-nothing,
retain ZERO partial permits while queued and hold no unadmitted round worktree lock.
Serialized-capable profiles still refuse a queued member outside their proved finite
window and make the whole partially admitted round non-certifying (8.k/14/18).

Use actual retained Lost shares to compute non-Lost-reachable capacity; healthy busy
shares remain reachable. Impossible wholly queued proposals park with one unchanged
identity/cause, no admission/round/retry charge or repeated metadata spam. Genuine14
restoration may revalidate that SAME current proposal; stale target closes rather
than rebinds. After partial admission, unrelated Lost capacity restoration creates
no autonomous retry cause. Keep prior charge/hold and the18.h required authorized
exit.27 fairness and product14 recovery remain pending consumers.

Reserve whole-roster worst-case retained evidence BEFORE the first native effect.
Each member releases ONLY its own native permit after genuine COMPLETE owned
settlement AND exact member closure; the round target lock, unresolved evidence and
unused safety headroom stay until safe WHOLE-round closure. Retained server/tools/
Unknown/Lost hold ownership. Timers and native turn result are not death proof.
Unknown suffix/target mutation/native uncertainty prevents certificates regardless
of mode. Lost remains absorbing pending genuine14 recovery.

Freeze three separate startup/review/settlement clocks and their exact start causes
under14/16. Settlement starts ONCE at the earliest owned COMPLETE result/native
terminal/cleanup request (or specified cancellation/expiry); never restarts between
cleanup stages. Validate actual supported normal end-to-end AND forced envelopes,
ceiled integer-ms arithmetic and required margin, including escalation/cancellation
within normal cleanup. Preserve default100ms/minimum and stronger profile margins;
a finite configured bound is no universal OS jitter/death guarantee. Unknown stages
refuse before effects; unexpected escaped/late ownership stays factual held14.

## Opinions, findings and lineage

Mode all/quorum/any counts well-formed exact-current APPROVE opinions; independently
apply Quick/Standard floor1 and Strict floor2 eligible NON-AUTHOR Sessions. Frozen
allowSelf may affect opinion tally, never that floor. Every recorded applicable delta
contributor identity/provider family is excluded from self-confirmation. Ordinary
approvers need no mutual provider-family diversity unless frozen policy requires it;
the dismissal/downgrade confirmer PAIR retains its required distinct diversity.

Persist immutable individual outcomes before evaluation. Findings and assigned
per-prior-finding dispositions reference exact original hash/author/severity/target/
inspected evidence, and remain distinct from native transport status. Potential
blocking findings veto until genuine eligible finder/frozen independent confirmer
or actual trusted Human adjudication meets9's exact rule. Controller summaries,
APPROVE, changed text, dismissal claim or target movement cannot clear them.
REQUEST_CHANGES stays REQUEST_CHANGES; disputed dispositions/ESCALATE remain held.
Any/quorum does not publish early: all admitted native members must safely settle,
all blockers need effective resolution and both mode/floor must hold. Policy-safe
early cancellation remains forbidden for Strict/security as required.

Concern-linked reruns retain recorded verification BEFORE fix commit, truthful
attributed fix claim and actual nonempty committed concern-linked delta, with genuine
bounded retry authority. No within-round retry, cosmetic resampling, new-target
budget laundering, inherited verdict reuse or controller roster/confirmer shopping.
18.i's disclosed residual nonblocking resampling remains an availability/metric
limitation; it is no blocker clearance. Every admitted round and actual delivered/
retained evidence charges the original immutable obligation-budget lineage.

Trusted12 applicability evidence covers changed AND relied-on source/dependency/
context plus retained contributions/findings and method incompleteness. Missing/
unknown proof inherits/refuses; filenames/lexical map/model/Human assertion alone
cannot grant a disjoint new budget. Proven disjoint obligations may progress without
certifying old exhausted history. Affected rework/supersession/decomposition retains
unresolved vetoes/authors/exhausted shared budget. Only genuine certificate PLUS exact
merge artifact accepts old upstream; generic Merged/Completed/rebase cannot wash it.
Real Lost/root/common-Git physical holds survive all review-scope labels (11/12).

## Budgets, publication and acceptance

Implement the requirements' checked simultaneous limits directly:32 slots/local
parallel setting,64 admitted lineage rounds,256 findings/result,8192 COMPLETE encoded
bytes/finding and1MiB COMPLETE member frame/result. Distinguish source budget,
mandatory metadata, actual retained bytes, every actual delivery and nullable native
measurements.128MiB bounds retained lineage evidence by real ownership/copy identity;
external references still charge metadata/actual copies and have exact owner/bounds.
Dedup does not hide actual copies or repeated delivery. The separate64×32 prepared+
expansion delivery ceiling4GiB does not include invented native token/cost measures.

Whole-roster reservations include frozen actual core/frames, bounded result/diagnostic/
expansion representation, genuine native owner/operation/receipt/audit overhead and
FIXED8MiB verification plus FIXED4MiB control allowances with reserved64KiB margin.
Unknown producer bounds or insufficient quota refuse before input. Acquired model
prefix1MiB/diagnostics64KiB retain exact count/hash/overflow, never unseen suffix
approval. Between-round verification reserves its own bounded8MiB allowance against
the same128MiB BEFORE owned effects; it borrows no unadmitted round reservation.
Only safely closed unused reservations release; mandatory existing facts survive.

Typed atomic Store publication validates exact immutable scope/authority/slot/frame/
source/lock CAS and reserved quotas. History is append-only; generic Record/JSON
writers cannot construct or edit allocations/certificates. Derive hashes/plans
outside held SharedStore and recheck actual compact/full-row authority within its
explicit reviewed bounds; no unbounded SQL scan or claimed512KiB total-lock budget.
Coordinated incompatible reader/writer epoch follows the actual linear19/23/43/58
release and genuine legacy drain/recovery, not a guessed version or fixture migration.
Current rows matching themselves are not original-frame proof.

A certificate binds current target/coverage/baseline, policy/roster, exact core/member
frames, independence/read-only/native evidence basis, dispositions, lineage/budgets,
individual tolerated failures/exposures and COMPLETE settlement. It is evidence,
not command/merge permission. The downstream consumer independently assesses exact
binding/basis BEFORE production readiness, preserving reachable bad-basis refusal
while unready.21.i's interim controls cannot claim21.n's separately mandatory genuine
formal-basis/readiness-positive matrix or final16 completion.

Maintain a stable-key acceptance manifest for EVERY requirement: actual consumer,
positive/negative fixture, compiled omission mutant or named pending producer.
Exercise real temporary Git identity/delta/source races, unchanged-current CPP across
parallel2/triple members, controller/Human separation, cohort contention/member
release, uncertainty/timeout clocks, accumulated findings/dispositions, exact encoded
byte quotas and full-floor policy matrices through actual ports. Synthetic fixtures
prove only labelled mechanics and noncertifying refusal. Required production5/6/7
profile declarations and repeated-family/2/triple positives close9 only with their
actual evidence.20 deterministic producer,14/15/24/27 product composers and final16
MVP dogfood remain the requirements' explicit separate gates; no pending producer
is silently counted as implemented and no dependency cycle or whole-MVP waiver is added.
