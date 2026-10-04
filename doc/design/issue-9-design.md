# Issue 9 design: configurable independent Review Sets

Status: Design4 candidate, provider-neutral and not implemented. The controlling
[requirements](../requirements/issue-9-requirements.md) are the Req38-gated revision
plus its Low wording clarifications at caa4390780f483f2d29f8182a36f65f924caf010,
SHA256 0af45849f067036815d36b3c43adb74515243c7a220b7a5789f379e6f722fcfb.
That document, including every stable I9-AC key and its closing/post-closure staging,
is normative; this design organizes consumers and authority without replacing its
limits or creating new policy. Names below are proposed private types, not compiled
kernel APIs. Actual main has one Reviewer Session per Workflow phase; it has no
ReviewSet implementation. Source qualification must use the real reviewed
19/43/native ports and a separately committed integration design before code.
23 composition is a later convergence consumer, not an additional9 source-merge prerequisite.

## Data and authority

Use immutable typed PolicySnapshot, ReviewerSlot, ReviewSet, RoundProposal,
ReviewRound, MemberInput, MemberOutcome, Finding, FindingDisposition, Verification,
LineageReference and ReviewCertificate envelopes. Every durable object carries
exact Project/Goal/Task, owning Set/round/slot where applicable, format version and
bounded referenced artifact identity. Facts and serialized IDs do not construct
native ownership, controller authority or recognized Human authority.

9 first owns an actual private provider-neutral library-composition Human-ingress
port under9.d. Its nonserializable capability follows the approved23 derivation:
actual OS caller UID plus the actual invocation origin, with NO composer-supplied
principal parameter. It derives the exact scoped event/reason/evidence/digest
through trusted composition; it never accepts
actor=Human JSON. Record origin=library-composition, always nongating, including
fixture authorship at creation. Activated accepted-ingress checks reject an
unaccepted origin; a controller cannot relabel this capability as CLI/UID Human
provenance. Native/Workflow/Broker API/IPC and output cannot acquire it. Arbitrary composer principal is unrepresentable/rejected at
the actual port. Component
Human-boundary positives use this real port with negative relabel controls, not
product certificates or an absent23 implementation.

On ACTUAL23 composition, REPLACE this interim derivation by importing the ONE
canonical23 type; do not wrap it or retain a second constructor. Convergence
controls compare principal/origin for the same accepted invocation, reject the
removed interim path and preserve nongating library origin. Actual15/24 production
handlers and10 Human-only refusal compose later under21.i. The separate controller
can instantiate an EXACT already Human-approved digest and execute frozen branches,
not new content, post-opinion roster/confirmers, clearance or budget reset.
Application-channel trust is explicit; direct same-UID OS/CLI/DB/linked-code control
is its disclosed limit, not biological identity or OS secrecy. Product-ingress and
readiness remain pending; this does not add a23 prerequisite to provider-neutral9.

PolicySnapshot freezes activated Runtime/registered Project governing rule/skill
source, exact approved digest, roster/model/effort, mode/quorum, independent floor,
risk/security behavior, confirmer assignments, clocks, schedule/channel profile and
byte/round/resource bounds, queue-attention threshold and optional per-slot queue
expiry BEFORE target opinions. Task-authored edits are reviewed
proposals, never governing rubric for their author. Resolving and hashing a source
is not trusted activation; base-sync cannot auto-approve new content. Scope-changing
updates create explicit history through the required authority, never in-place
reinterpretation of a started round (8/9/12).

12.j's pre-claim/input consumer requires the actual registered native-view producer
for every discovery-reachable governing instruction/rule/skill in the authorized
review workspace. Compare that complete path/revision/digest set with the activated
governing snapshot. Task-edited or base-sync activation divergence refuses BEFORE
input with exact conflicting paths/digests; missing/unknown view is unavailable,
not equality-by-empty-list. Unapproved base-sync content refuses until exact trusted
Human digest approval, never controller activation. Native defaults stay unchanged;
no invented view producer or arbitrary-natural-language equivalence proof is claimed.

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
finding identity/severity/text and label prior exposure. Original verdicts stay ONLY
in retained private outcomes/certificates. Separate prior-round verdicts, approval
tallies and non-finding peer opinions are excluded from EVERY input/expansion,
prior-claim manifest, resolved-history expansion and MEMBER-DELIVERED baseline
unchanged-region basis (8.h/10.d). The PRIVATE certificate coverage record retains
each baseline slot's delivered+prior-approving or delivered+prior-nonapproving basis;
frozen require-approving-baseline reads retained private outcomes before admission.
Controls show this certificate field exists while delivered member bytes carry
ZERO verdict/tally hints. Finding applicability/adjudication metadata is not a vote
hint. Exact referenced resolved finding history remains bounded; unverified actor
fix claims are never facts or clearance. Same-tree transient retry after peers
APPROVEd must deliver zero verdict/tally fields; omitting that exclusion must change
actual delivered bytes and invalidate its basis, not merely an already-unready gate.
This is runtime-controlled input independence with
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
Its ingress requires a private controller-created conformance capability before
claim/reservation/setup/input: genuine newly created dedicated state and primary
fixture root/common-Git identity with actual complete nonoverlap checks against
registered user roots/scopes. No caller flag/path, copied marker or user registry
mints this birth capability. Direct real-state/user-Project calls, unknown/pending
physical scope and relabel attempts refuse with zero claim/process/model bytes
through the actual ingress (21.m); actual native/profile ownership still applies.

Current candidate feasibility under8.l is UNKNOWN, not qualified:

| Candidate | Concrete known basis | Missing proof / owner |
| --- | --- | --- |
| Claude | Req38's retrieved vendor basis describes repository memory shared across worktrees; observed CLI2.1.283 only | Actual installed read/write timing and key predicates, same-round peer-free schedule, owned read-only/cleanup conformance:5/9/19;16 measures representative behavior |
| Codex | Req38 records CLI0.160.0 and distinct memories feature versus field defaults;Issue6 F1 (escaped native command/tool cohort) is unresolved | Actual effective channel activation/key/timing plus genuine managed cohort settlement:6/F1/9/19;16 measures representative behavior |
| Grok |7 supplies an actual native adapter; no qualified default-preserving channel/settlement declaration is supplied here | Actual installed channel inventory/key/timing and normal/forced owned settlement:7/9/19;16 measures representative behavior |

This table claims channel capability context, not observed peer-finding injection;
no private user memory/config was read. Before any positive, templates must have
actual owned native fixture evidence and instantiate proved installed-version/
configuration/key/load-write/source/roster/schedule predicates on the real round.
The8.l sufficient candidates are: prove no reachable current-peer writer; prove
genuinely disjoint keys under exact authorized workspace/input scope; OR prove a
peer-free snapshot where ALL relevant loads precede ANY peer write under an enforced
finite schedule. None is presumed. Freeze every reachable queue/expiry schedule
within that proof; an unbounded queue cannot qualify a finite load/start window.
Unknown reachable current-
peer channels refuse; no universal family-equality floor or memory-disable workaround.
At most three candidate investigations per required configuration then21.l requires
named public owner/proof/scope-decision attention. Required repeated-family/4+,
2-reviewer and Triple positives remain open; a candidate refusal cannot close them.

| Required configuration | Current disposition / concrete missing property |
| --- | --- |
| M=1 non-author Quick/Standard | UNKNOWN: one actual registered family's installed channel, governing-view and full settlement proof |
| M=2 non-author first-class | UNKNOWN: both actual profiles plus complete qualified scheduling/cohort and same-core member closure |
| Triple preset | UNKNOWN: actual Claude/Codex/Grok profile declarations, floor eligibility and whole roster channel/settlement proof |
|4+/repeated-family serialized | UNKNOWN: actual repeated-family default key/load-write properties and enforced finite serialization/queue schedule |
| Two-author Strict | UNKNOWN: at least two eligible non-author Sessions under actual qualified registered profiles; authors cannot satisfyF2 |
| Conflicting rule-edit native view | BLOCKED/UNKNOWN:5/6/7 actual complete native-discovery view and19 workspace/input producer; no current certificate-capable exit |

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
19/43/native ports before production code; actual23 convergence remains separately
required by product readiness, without an additional9 source merge cycle.

21.k requires an actual upgrade-consumer inventory of merged8 single-reviewer
writers and downstream gates under genuine12 impact evidence. Historical legacy
Passed stays factual read-only history, refused by new formal PR/MergeGate/
finalization; no grandfathered certificate/context clone/repeat. Pre-epoch live or
Evaluating ownership refuses the upgrade unchanged or drains under19/14, preserving
external-effect history. The implementing change must withdraw legacy gating,
including Quick/retry/resume_gate/request_finalization, while readiness is absent.
Tests reach these actual consumers; current merged behavior is not retroactively
claimed to enforce the proposed withdrawal. Availability may hold existing work.

## Scheduling and settlement

A bounded RoundProposal freezes target/policy/source/hash refs and charges its exact
metadata; wholly queued work is not an admitted round.14.h fixes8192 bytes/proposal,
512 proposal records/lineage and a separate4MiB never-admitted allowance, all actual
retained metadata also charged to128MiB. No complete prepared member frame/copy is
persisted for an unadmitted proposal; reference exact existing immutable artifacts,
discard bounded volatile preparation when stale. Capacity exhaustion reports
NeedsProposalCapacity, not exhausted review rounds or silent pruning. At first
actual member admission, one transaction revalidates and materializes bounded frames,
reserves whole-roster quota/lock and charges64-round count AND the exact applicable
retry/confirmation allowance. Never-started stale/expired proposals charge metadata
only.512 target-change controls retain remaining admitted-round capacity.
Every initial/queued member
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

Queue attention is frozen1–3600s/default600; it notifies without timing out a
native model or requiring Human rescheduling. Queue expiry is Disabled/default or
1–3600s per slot from enqueue; invalid0/3601 refuses. Attention/expiry clocks remain
separate from startup/review/settlement, and qualification checks EVERY reachable
schedule at freeze AND pre-input. Expired wholly queued proposal closes without
round charge; partially admitted round closes noncertifying only after genuine
owner settlement. Lost/unknown capacity restoration is not expiry/retry authority.

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
Before initial/each later claim/input, actual roster eligibility derives cumulative
runtime-attributed author identities/families (including drafters), treats unknown
applicability conservatively inherited before actual12 proof, resolves unknown
attribution through required authority and refuses named excluded families when
eligible slots cannot meetF. Default self slots AND duplicate registered agents
refuse before input. Explicit allowSelf may count an author opinion toward mode,
NEVERF or confirmation. Separately, explicit allow-duplicate-agent/registered alias
permission admits distinct fresh NON-AUTHOR qualified Sessions toward mode ANDF,
subject to8.c and any frozen diversity rule; it does not lowerF. Duplicate/alias
permission NEVER creates dismissal/downgrade confirmer-pair independence. Default Triple and separately
labelled author-visible Triple are distinct configurations, not post-hoc aliases.

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

The typed rerun-authority consumer compares target TREE: same tree/new commit is
identity-only delta, never changed-byte retry. Cause(a) allows ONE lineage/tree
confirmation round with new concern-linked exact inspection target/location/check
or observed-source digest not previously used. Cause(b) requires safely settled
native failure/timeout/cancel whose failed slot produced ONLY verified diagnostic
progress, NO partial/ambiguous structured verdict/finding content. Cause(b) identity
is provider-family/error-kind; every OTHER settled SLOT must APPROVE, with no
blocker/nonapproval/unknown result. Thus one APPROVE plus two failed slots refuses
cause(b); no missing opinion is an approval or new multi-failure cause identity. Cause(c) requires its
frozen qualified schedule/resource/failure identity, genuine changed scheduling/
capacity fact, fully settled lost-free processing failure and every actual opinion
APPROVE with no partial/unknown suffix or dissent. Shared(b)+(c) ceiling2 per
lineage AND at most1 per exact tree/cause identity; changing trees never resets it.
Repeated causes, third retry, mixed dissent, unresolved outputs and Lost refuse
or use existing explicit Human/recovery authority. First-admission atomic charges
include these allowances and64 rounds; unchanged foreign-Lost wakes burn neither.
17.c/18.e controls compare genuine restored versus omitted cause/charge predicates.

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

9.h freezes each later slot's required-disposition manifest before input. Assign
eligible original finder OR pre-observation designated independent confirmer pair;
missing eligible designation means the existing9.f Human-only hold, not favourable
selection. Every member still receives unresolved claims, but unassigned opinions
cannot clear blockers. Actual result DTO bounds provide metadata≤64KiB. Check
required_count≤256 AND checked(required_count×8192+metadata_bound+65536)≤1MiB,
reserving eight full-size new findings. Unknown DTO bound refuses before impossible
output obligation;112 required entries with64KiB metadata fit,113 refuse. Findings
and dispositions retain separate256-count caps with combined complete1MiB result.

Expansion uses frozen14.a request1–64, per-slot1–8MiB, per-artifact1MiB and aggregate
round32MiB caps. Actual delivered bytes charge EVERY response/reference delivery;
14.b request/slot allowances reset ONLY at a newly admitted(round,slot), never reset
per-round aggregate on slot renewal, and preserve inherited64-round/128MiB caps.
Actual producer validates exclusions, exact source/hash and assigned unresolved
text delivery before publication; omission mutants reach those byte consumers.

Typed atomic Store publication validates exact immutable scope/authority/slot/frame/
source/lock CAS and reserved quotas. History is append-only; generic Record/JSON
writers cannot construct or edit allocations/certificates. Derive hashes/plans
outside held SharedStore and recheck actual compact/full-row authority within its
explicit reviewed bounds; no unbounded SQL scan or claimed512KiB total-lock budget.
Coordinated incompatible reader/writer epoch follows the actual reviewed19/43/native
authority release and genuine legacy drain/recovery, not a guessed version or fixture
migration. Later23/58 composition preserves this exact epoch contract; it introduces
no additional merge prerequisite for provider-neutral9 library closure.
Current rows matching themselves are not original-frame proof.

A certificate binds exact Workflow phase/generation/active claim plus Project/Goal/
Task/worktree, current target/coverage/baseline, context/checkpoint provenance,
policy/roster and required governing instruction/skill digests, core/member frame
hashes, independence/channel/read-only/native basis, original findings/effective
dispositions and lineage/budgets. Include per-member prior-claim author/text-hash
exposure, tolerated failures/timeouts and early-stopped-before-output slots, actual
consumed Human principal/origin/event/reason/evidence/digest/self-adjudication flag,
accepted UID/ingress/trust-limit disclosure and visible alias/duplicate permission.
Record actual native_execution and read_only_basis=contract+construction,
native-enforcement-pending16 when that is the genuine basis, never synthetic native
proof. COMPLETE settlement is necessary, not alone approval. Certificates are
evidence, not command/merge permission.

Downstream binding assessment precedes permission/readiness: fitting nongating
basis yields binding-valid+permission-refused; drift/replay/phase-generation-claim
mismatch yields its exact binding-refusal+permission-refused. Readiness-unavailable
is separately typed. A mutant must change THIS actual assessment, not merely the
already-refused permission.21.i interim controls cannot claim21.n's separately
mandatory genuine formal-basis/readiness-positive matrix or final16 completion.

Each stable key's acceptance manifest carries BOTH evidence COMPONENTS required
by its body: actual consumer positive and/or refusal fixtures with meaningful
compiled omission controls, AND exact PUBLIC acceptance quote/permalink for EVERY
external owner named there, checked against the complete21 enumeration. Components
are not exclusive key classes or a fixed handoff shortlist.21.a may be quote-only;
21.f/21.l required actual profile positives cannot close on quotes, and explicitly
refusal-only12.k cannot fabricate RuntimeAuthor positive but still needs its60/12
quotes. Pending producer is annotation, NEVER substitute closing PRE evidence;
only21.n is the requirements' separately POST closing key.
Exercise real temporary Git identity/delta/source races, unchanged-current CPP across
parallel2/triple members, controller/Human separation, cohort contention/member
release, uncertainty/timeout clocks, accumulated findings/dispositions, exact encoded
byte quotas and full-floor policy matrices through actual ports. Synthetic fixtures
prove only labelled mechanics and noncertifying refusal. Required production5/6/7
profile declarations and repeated-family/2/triple positives close9 only with their
actual evidence.20 deterministic producer,14/15/24/27 product composers and final16
MVP dogfood remain the requirements' explicit separate gates; no pending producer
is silently counted as implemented and no dependency cycle or whole-MVP waiver is added.
