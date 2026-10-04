# Historical public Issue9 body before Req26

Historical provenance ONLY. Superseded refinement entries below are not current
normative instructions. Current requirements: [Issue9 requirements](../issue-9-requirements.md).
Original checkboxes/outcomes remain unchanged; no implementation acceptance is asserted.

Requirements/design preparation depends on merged #2, #4, and #8. Core source acceptance additionally requires the actual reviewed native provider contracts #5/#6/#7, typed context/member authority #19, and record-only Workflow binding #43. These are integration gates, not claims that the providers are already ready.

## Goal
Review count must not be hard-coded to Triple Review.

## Scope
- Review Set / Review Round execution
- 1, 2, 3, or more reviewers
- Parallel reviewer launch
- Completion modes: `all`, `quorum`, `any`
- N-of-M quorum
- Configurable model/effort per reviewer
- Immutable target revision
- Multi-round verify → fix → commit → re-review
- Triple Claude/Codex/Grok preset
- Deterministic Review Bundle input and delta-based re-review integration

## Acceptance criteria
- [ ] Two-reviewer configuration is first-class
- [ ] 2-of-3 quorum works
- [ ] all-of-2 works
- [ ] all-of-3 Triple Review preset works
- [ ] Reviewer failure/timeout does not silently pass
- [ ] Round history and individual findings are preserved
- [ ] Review can require a clean/locked worktree
- [ ] Independent reviewers receive equivalent factual Review Bundles and later rounds can consume revision deltas

## Acceptance staging and mandatory follow-ups
Issue #9 closes the configurable consumer using actual registered production read-only native declarations and supported explicit configurations, including first-class two-reviewer and Triple presets. Git-backed typed bundle/delta producer fixtures prove the consumer contract. Fixture-only registrations or unsupported presets do not satisfy core acceptance.

The production deterministic bundle/delta/expansion producer remains #20 (which depends on #9); #9 does not depend on #20, avoiding a cycle. Representative real native two-reviewer/Triple outcomes, supported isolation checks and the efficiency comparison remain mandatory MVP acceptance in #16. Neither producer readiness nor real-model results are inferred from synthetic orchestration.

Formal Core gates keep Critical/High/Medium blocking. QUICK/STANDARD require at least one independent non-author Session approval; STRICT requires two. Explicit allow-self opinions may participate in all/quorum/any but cannot satisfy that floor. Current-round peer findings and raw executor chat are excluded from runtime-controlled initial/expanded input, rules/source slices and manifests until all roster slots settle. Native permissions and residual same-user filesystem visibility remain explicit. Lost/uncertain members retain ownership/locks/permits until trusted #14 recovery.

Detailed proposed requirements and acceptance: [issue-9-requirements.md](https://github.com/shuhei-suzuki/rururunx/blob/4dd0e1f53a4232f5e39a8c1fa77712e4d459dd9d/doc/requirements/issue-9-requirements.md). All acceptance checkboxes above remain unchecked until actual evidence exists.

### Core pre-recovery availability boundary

The proposed Core ReviewSet contract deliberately retains persisted Lost/uncertain member ownership, locks and resource shares until the actual trusted Issue #14 recovery handoff. A settlement deadline may therefore permanently hold a Task and capacity before #14, even if the original supervisor later observes cleanup; that observation is retained as evidence and does not manufacture a release or APPROVE. There is no in-runtime Human release shortcut. Issue #16 must measure settlement-expiry Lost frequency, retained capacity/Task-time and Human-interruption impact before defaults are treated as validated. This is an explicit availability limitation, not completed recovery.

An exhausted obligation lineage cannot regain review budget by Task decomposition. An all-author roster cannot regain eligibility through Human opinion. Supported completion requires an actually eligible activated-policy custom roster/provider; otherwise terminate without a ReviewSet certificate. Details and acceptance remain in the linked Issue 9 requirements gate.

This includes normal-completion cleanup expiry after a valid APPROVE, as well as cancellation/timeout cleanup. #16 must report these settlement-expiry Lost frequencies separately, with retained capacity and Task-time; late owned cleanup alone does not release the hold.

Issue #9 owns its ReviewSet-specific trusted library ingress/controller authority constructors and actual boundary tests, using the approved Issue #23 application trust rule without a #23 merge dependency or a CLI.

Criterion21 is the canonical #16 measurement handoff for every attributed stable acceptance ID, including native auto-injection versus visibility, blind/exposed re-review, large or accumulated required coverage, zero-eligible/two-author STRICT availability, partial-output Human attention, all settlement-expiry causes and round/artifact-quota exhaustion frequency and Task/decomposition impact. Lineage exhaustion can terminate review permanently; decomposition never resets the budget. Closure must trace each obligation to its public owner or immutable commit permalink; unimplemented measurement remains an MVP gate.

Issue9 owns the frozen typed consumer bundle/delta/identity/expansion contract (8.g), which production20 must conform to. Criterion21 also covers every deferred evidence obligation: native read-only enforcement/trace-observed isolation16.b/16.c/8.a, efficiency attribution16.a, and14 lineage/descendant/target-mutation holds11.b. Recorded held/exhausted/uncertified contributions merged out of band retain unaccepted provenance and applicable obligations. The linked requirements revision corrects the prior unconditional Project-wide ancestry veto: proven disjoint new obligations may progress with fresh budget, using trusted impact/applicability evidence over changed AND relied-on source/dependency/context plus retained findings/contributions. Affected/unknown obligations inherit exact vetoes/authors/shared budgets or refuse. Actual Lost/native/resource holds keep their full physical effect scope until14 recovery;16 measures scoped applicability/unknown holds, disjoint progress and resource-scope blocking11.d. Only an actual accepted review certificate plus scoped merge artifact for the same reviewed contribution establishes normal upstream handoff.

Frozen settlement deadlines, including provisional default30s, require reviewed production caller complete normal (including fallback/cancel during cleanup) AND forced cleanup envelopes plus profile margin; unavailable or too-short configuration rejects before input. Before actual14 recovery and required real native producer/16 proof, library/synthetic certificates do not establish production merge-gate readiness. Canonical21 includes Human-only blocker-clearance availability9.f and normal-cleanup/profile evidence18.g. Mixed transient failure and dissent requires concern-linked verification/Human authority, not blind retry; post-opinion roster changes require trusted ingress/controller authority.

The linked requirements revision adds typed per-prior-finding dispositions9.g, fresh native Sessions per(round,slot), and single current default30 deadline oracle (ceil-ms maximum of COMPLETE normal/forced envelopes plus profile margin minimum100ms <=30000ms;29900ms accepts,29901ms or unknown refuses). Public handoff21.e requires15/24 trusted product Human ingress;21.f assigns actual5/6/7 producer contracts or explicitly tracked follow-ups;21.g assigns actual14/27 retained-share recovery/fair admission with12 verification and16 metrics. These public tracking obligations do not claim completed handlers, providers or fairness and do not introduce a cyclic merge gate. Status distinguishes actionable Human decisions from recovery-pending/nonactionable holds.

The linked requirements revision explicitly separates the 128 MiB ACTUAL RETENTION quota from repeated delivery: every owned blob/copy/manifest charges retained bytes, immutable Git references charge retained metadata plus any copied content, and every delivery still charges frozen per-slot/round actual-byte budgets. Across 64 rounds, 32 slots and 32 MiB aggregate expansion per round, rrx-controlled prepared-frame plus expansion delivery is bounded by 4 GiB per obligation lineage; native prefixes/telemetry remain separately bounded/reported. This corrects the ambiguous previous expansion delivery charge, without hiding storage or tokens/cost. Current impact coverage includes unchanged consumers of changed mechanisms (10.d); missing/stale/unknown closure forces full coverage. Conflicting eligible finding dispositions hold, and confirmer designation freezes before input (9.g). Normal settlement has a one-time complete-result/native-terminal/cleanup start and reviewed upper bound, not percentile (18.g). Prior separate verdict/tally fields are excluded from member input, original finding claims remain labelled (8.h). Actual narrow/widened effect scope is producer/evidence-bound, never granted by Reviewer label (11.e). All acceptance remains pending.


Requirements production-enablement clarification (linked immutable revision): #9 owns the typed applicability consumer/checker; #12 owns the actual reviewed impact/applicability producer using #18/#20 inputs. Until its real conformance exists, unknown applicability inherits/refuses; synthetic consumer fixtures do not certify disjoint fresh-budget production admissions. Multi-Project production ReviewSet integration stays disabled until actual #14 retained-share recovery and #27 fair admission compose. Core actual supported native declarations/presets remain acceptance prerequisites, not waived by tracking.


The linked requirements revision proposes an explicit minimum/default100ms cleanup safety margin (profiles may require stronger), never universal OS timing or native death proof; default30s accepts known29.9s, rejects29.901s/unknown. Native hooks/defaults remain; known unbounded profile refuses before input, unexpected postdispatch uncertainty remains owned/held for14. Governing native rubric conflicts (Task edit/base sync) name exact paths and require a genuinely supported reviewed profile or termination, with16availability evidence. Actual product review gating stays disabled for single and multiProject until real native/profile conformance,14recovery, trusted activation and required15/24Human ingress; multiProject additionally27. Queue attention automatically continues on returned permits with fresh currency. Unknown authorship policy cannot launder exclusions. Raw Executor transcripts stay excluded unconditionally; native auto-memory channels and restartLost are explicitly recorded/measured without changing native defaults or inventing provider-family restrictions. These are proposed requirements, no implementation/production acceptance claim.


The linked requirements revision explicitly records pre12 conservative availability: unknown applicability of recorded out-of-band contributions can accumulate inherited authors/budgets/vetoes and make STRICT/allClasses zero-eligible.12actualdisjoint evidence is the scoped exit, not bytes-only intersection or Human assertion;16measures author-growth and Task-time. Both normal AND cancel/startup/review-timeout/early-stop forced cleanup envelopes must fit the100ms-or-stronger margin (ceil milliseconds); known2snormal+35sforced refuses default30 before input. A safely settled round releases real delegation/locks for Executor remediation, while Set retains obligations/history; next round binds one Workflow milestone context with no per-member bump. Only verified nonauthor/trustedHuman/deterministic evidence applies upwardrisk; immediate holds remain. Descendant Sets share immutable origin-phase budget roots with atomic concurrent reservations. Unknown nativeMemory eligibility is an explicit scoped F-counting risk measured16, never a peer-free native claim; actualknown currentpeer injection rejects. All component/source/native gates remain pending.


Req24 refinement at the linked requirements revision: independent-gate readiness needs actual registered native memory/history channel scope and timing conformance, preserving defaults; documentation of a channel does not prove actual peer injection. Unknown reachable current-peer channels cannot earn independence. Actual runtime-effect attribution, bounded retained native row overhead, queue lock/expiry controls and fixed shared reservation arithmetic have stable acceptance keys. Public #12 complete delta impact closure and #13 post-certificate irreversible reconciliation remain tracked producer obligations. Requirements/native production gates remain pending; no ReviewEngine implementation or acceptance is claimed.


Req25 linked revision supersedes earlier refinement entries where they differ. Current closure quotes use I9-AC-<key> to distinguish acceptance IDs from GitHub Issue numbers; historical short keys remain aliases. I9-AC-9.h bounds and assigns required disposition outputs before input; I9-AC-18.g complete normal cleanup includes escalation/cancel during normal under one timer. I9-AC-14.h charges round/retry counts at first actual member admission: stale entirely queued proposals retain charged byte provenance without silent target rebind. I9-AC-16.d is actual #5/#6/#7 supported-profile DENY/zero-ALLOW producer conformance, tracked through21.f/#16; I9-AC-2.b explicitly holds quorum/any with a Lost member. Gates remain pending, no implementation or production acceptance is claimed.
