# Issue 9 requirements: configurable independent review

Status: preparation on merged main; implementation has not started. Public Issue 9
depends on state persistence, AgentAdapter and Workflow Engine, all merged. Actual
context publication must integrate the reviewed context-pack contract before
acceptance. Deterministic delta construction is the separate review-bundle port;
this engine must accept and preserve its provenance without inventing deltas.

## Behavior

A Task review phase creates a durable Review Set with one or more uniquely
identified reviewer slots. Two reviewers are a first-class configuration. Counts
are bounded, never hard-coded to three. Each slot freezes agent identity and its
explicit model/effort requirements; unsupported options fail before model input.
The Claude/Codex/Grok triple-adversarial configuration is a preset, not a separate
algorithm. Default native configuration/auth/hooks remain authoritative.

Each independent round freezes exact Project/Goal/Task scope, actual bound
worktree identity, target revision, relevant source hashes, policy, reviewer roster
and equivalent factual Review Bundle. Required rules, requirements/design,
verification evidence and unresolved verified facts cannot be omitted by budget.
Provider-specific framing may differ, while factual bundle digest remains equal.
Peer findings and executor conversation are hidden during independent review.

Independent configured slots launch concurrently within explicit resource bounds.
Completion policies are all (every slot), quorum (N of M) and any (one). Only
successful, well-formed, target-bound approvals count. Failure, malformed output,
timeout, cancellation and Lost are preserved as distinct individual outcomes and
never count as approval. Quorum/any may tolerate nonapproving results according to
explicit policy, but verified blocking defects remain a global veto. Unverified
potential blockers require verification before acceptance. Eligibility is distinct
from safe completion: outstanding owned sessions must be terminal or explicitly
reconciled before releasing immutable review authority. Uncertain cleanup cannot
silently become Passed.

Findings retain reviewer, round, exact target, location, severity and rationale.
They are proposals until repository verification records a verified defect, false
positive or human judgment with actual evidence. Class-wide verification records
locations inspected/changed. No supervisor LLM normalizes or verifies findings.
Verification/fix/commit/required tests/re-review is an explicit durable sequence;
new rounds have new immutable targets and do not overwrite old results.

Optional clean/locked review checks are explicit per policy. A required lock
excludes executor mutation through actual Store/native launch authority. Relevant
external source/HEAD/dirty changes invalidate result publication. Fixing during
review uses a separate worktree. Parallel reviewer success never authorizes a
merge by itself, and consultation/ApprovalReviewer roles cannot impersonate a
formal reviewer slot.

## Persistence and workflow integration

Review Set/Round/slot transitions use scoped atomic CAS with the actual Workflow
claim and context pointer. Native input consumption validates exact private slot,
frame, current target and pinned reviewer configuration before wire delivery.
Caller JSON cannot create a slot, declare a review complete, or replace an active
launch frame. The existing single-session actor path is insufficient for N-way
review; acceptance requires the real Workflow review delegation port, not only a
standalone library or synthetic gate that reports Passed.

Round history, individual failures/findings, verifier evidence, policy resolution,
input digests and nullable provider measurements survive reopening. New authority
contracts require an ordered persistence marker and native old-writer refusal
proof. Restart must retain uncertain owned review sessions and locks; automatic
recovery cannot be claimed until its real reconciliation integration exists.

## Acceptance evidence

Use actual scoped temporary Git repositories and the real Store/Workflow ports to
prove one reviewer, all-of-two, two-of-three, all-of-three preset and any mode;
configured roster/model/effort validation; concurrent launches and hidden peer
results; duplicate-slot and competing coordinator exclusion; failure/timeout/Lost
accounting; immutable target/dirty drift; and active session/context fencing.

Causal tests must prove failed or malformed reviewers cannot count, verified
blockers veto quorum, unresolved findings cannot be treated as facts, unsafe
cleanup retains ownership, and stale rounds cannot publish after a new target.
Exercise verify/false-positive/fix/commit/test/re-review history and reopening.
Use meaningful mutation tests for policy, source/slot ownership, history and lock
boundaries. Independent native review and exact-head Linux/macOS CI are required.
Synthetic adapters prove orchestration only; actual two/triple native runtime
review with real output and measurement is separate mandatory dogfood evidence.
