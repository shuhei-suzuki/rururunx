# Issue 14: fence explicit retry of an unbound native dispatch

Requirements1 candidate, NOT APPROVED. Risk STRICT: shared Workflow reservation/replay boundary.
Base main2c6ae9d; related open Issue14's "Verified explicit retry gap" acceptance.
This is a limited prerequisite, not Scheduler/restart/native recovery completion.

## Problem and scope

`WorkflowEngine::retry` admits Waiting/Failed nonirreversible attempts and checks
persisted Sessions, but a committed native dispatch marker with no bound Session
can pass the Session loop. `preparation_existing_explicit_retry_gap_is_characterized_not_recovery_proof`
records the existing Failed outcome. Missing durable Session does not prove that
an adapter was never called, that no input was sent, or that an owner is absent.
Explicit replay must obey the same unknown-outcome reservation boundary as ordinary
observation. Issue41 deliberately left this behavior for Issue14.

## Required behavior

1. For an active Executor or Reviewer attempt with `dispatch_started=true` and
   no bound `session_id`, explicit `retry()` refuses. Failed/Waiting state, a user
   retry reason, no Session rows, or unrelated terminal Session rows do not prove
   safe release. Preserve the exact Task, Workflow Record/body/version, active
   pointer, history/retries, context rows/pointer, Sessions, owners and audit.
   Return a fixed explanation that trusted owner/dispatch recovery is required;
   do not include raw native or persisted payload data.
2. Running/Interrupted/Evaluating attempts keep their existing retry refusal;
   irreversible EvidencePort attempts keep their external-outcome reconciliation
   fence. EvidencePort phase markers are not native launch evidence and are not
   subject to the new native-only predicate. Existing live/Lost/executor/Session
   scope checks and transition CAS must remain intact.
3. Existing supported retries remain available: resolved nonirreversible
   EvidencePort Waiting/Failed outcomes, and agent Waiting/Failed outcomes with an
   exact terminal bound Session that pass all existing Store/Engine fences. This
   component adds no terminal/death attestation and does not certify those legacy
   bound-Session paths beyond their existing policy.
4. Do not introduce a bypass option, force flag, inferred no-dispatch certificate,
   public recovery JSON, elapsed-time release, PID adoption, timeout relaxation,
   schema change, or fabricated Session/termination. The retained unbound marker
   remains reserved until a separately reviewed genuine Issue14 recovery producer
   resolves the original attempt. That producer is unimplemented here.
5. Preserve proven pre-marker owner-local release and committed terminal Task
   recovery under Issue41's exact ownership/dispatch/Session fences. A marker may
   exist even when launch was never called; this component does not classify or
   clear that case on absence alone.

## Acceptance and impact

- Use an actual `WorkflowEngine::step` native adapter start held after the marker
  commits. Observe the real durable marker and no Session, release the held start
  with a synthetic unknown-outcome error, then verify the actual Failed unbound
  attempt is retained and public retry refuses without any durable changes or
  second adapter call. Both Executor and Reviewer paths require actual controls.
  A Fake fixture supplies no native/F1 certificate and proves no host process death.
- Retain the earlier synchronous start-error characterization as historical
  evidence; update its behavior assertion only after the approved fix. Cover the
  shared refusal without asserting an unreachable production Waiting path was run.
- Positive actual consumers must preserve resolved agent terminal-Session retry
  and nonirreversible EvidencePort retry. Assert their real active-pointer and
  RetryEvent transitions and meaningful no-extra-launch observations.
- Compiled mutations removing the new guard, narrowing it to one actor, or wrongly
  applying it to EvidencePort must fail at the relevant public consumer. Conditional
  mutants without a reachable independent control earn no kill credit. Restore
  exact committed source/tree and rerun controls.
- Inspect step/poll, retry, resume_gate, terminal reservation release, Store's
  workflow transition guard and Project removal consumers. Preserve shared schema,
  native/environment/Context and ownership policies. Run default full workspace
  regression, fmt, all-target Clippy, debug/release builds and affected Workflow
  release tests; independent immutable source review and current Linux/macOS
  exact-source CI are required before a limited merge.
- Keep README/master Workflow/Issue41 historical disclosure aligned with the
  component's implemented state and pending trusted recovery. Do not close whole
  Issue14 or claim scheduling, restart, F1/native or full MVP acceptance.
