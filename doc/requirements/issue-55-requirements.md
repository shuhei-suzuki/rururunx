# Issue 55 requirements: bounded Grok terminal cleanup provenance

Risk: STRICT for native process/state evidence. Requirements2 approved at2ebbbfd;
implementation and independently reviewed source outcomes are recorded in
[verification](../verification/issue-55.md). Dependencies #7 and reviewed #46;
#41 default acceptance currently exposes the gap. #16 native integration remains separate.

## Verified gap and limits

Exact Issue41 head3d844d153b1fb9f0d384d1a3be3afdc7ca791cb6 CI37192948085 passed Ubuntu.
macOS passed library106, adapter19, CLI5, Context16 and Git18; Grok target8passed/1failed/
2ignored. Mode unowned_read failed only pid.is_none at grok.rs479. Its Lost state,
transport refusal and failure-present assertions passed; macOS debug/release builds were
skipped. The assertion stopped the mode loop: later unknown_fs_method was not observed
on macOS in this receipt; a test-level failed count is not per-mode coverage. A durable sanitized assertion excerpt with run/job/head/OS and the full log digest is
[committed here](../verification/issue-55-failed-ci-observation.json). The local original
`/private/tmp/rururunx-issue41-ci-37192948085-failed.log` is supplementary, not durable authority.
No original error category, actual retained-child state or PS causal explanation can be
recovered from that receipt after runner teardown. Earlier source-approved c659eeb CI
passed both OS; that does not substitute for final red evidence.

Current supervise computes clean = cleanup.is_ok && !ownership.uncertain && output_verified.
Only clean clears PID; terminal Lost otherwise preserves reservations. Existing diagnostic
uses result.err before cleanup.err. grok.turn_observed records cleanup_verified but no
independent cleanup error, ownership uncertainty or output-drain completion facts. Thus a
real native cleanup failure can be hidden behind the expected protocol/tool denial. The
existing event and test receipt do not distinguish its cause. This is an observability
gap, not proof of unsafe cleanup or a production ownership fix.

## Required behavior

1. Extend only the existing exact owning-scoped grok.turn_observed receipt with independent
   measured cleanup outcome, bounded cleanup error category, process-ownership uncertainty
   and actual stderr-drain budget result. Preserve existing tool/protocol diagnostic and
   completed/reconciliation/exit fields. Values reflect the actual supervised attempt,
   at the exact existing clean evaluation point AFTER post-cleanup reconciliation and
   before terminal state publication; do not infer them from Session PID or state later.
   Distinguish not-attempted/no native process from attempted group cleanup success/failure,
   child-reap success/failure/timeout. Label every Grok-local ownership flag at creation:
   native child, pre-spawn preflight, in-session binding verification, post-cleanup
   reconciliation. Record uncertainty by those stages and the exact total operand used
   by clean. Native-child uncertainty accompanying failed cleanup is correlated, not a
   second independent cause. Grok-local label metadata never changes flag clearing,
   ownership or group lifecycle. Tests compare cleanup_verified == cleanup_ok &&
   !ownership_uncertain && output_verified from this exact receipt. If a detail was not measured, use null/unavailable rather than guessing.
2. Define typed/static bounded cleanup categories derived from actual trusted cleanup/reap
   errors at their measurement boundary; no arbitrary error/body/output text. Shared
   cleanup_group currently erases finer IO/worker cause into AdapterError kind/text.
   No substring/prefix classification of AdapterError.message. This Grok-local issue
   records not_attempted, group_cleanup_failed_unclassified (kill or worker unknown),
   reap_timeout, reap_error with measured io::ErrorKind, or succeeded. Finer group/PS
   categories require separately coordinated reviewed shared typed API and are out of
   scope; unclassified means the exact cause is unavailable, not an inspection timeout. Retain a bounded generic category when an
   actual error cannot be safely classified. A category is diagnostic provenance only,
   never authority to claim native model completion, release reservations or clear PID.
   Explicitly identify what the existing output_verified boolean measures: stderr task
   completion within its existing wait budget, not complete stdout/structured output,
   absence of reader panic or perfect capture. Record drain not_started (whether a
   child spawned or not), joined_ok, joined_io_error if actually observed,
   joined_panic_or_cancelled, or budget_elapsed_abort_requested with termination
   unobserved. Any return/IO observation is Grok-local and preserves existing drain
   output/watch side effects. Do not infer joined_io_error from arbitrary failure text.
   Record join outcome separately if observed;
   do not strengthen the existing clean calculation in this observability-only change.
3. Keep primary model/protocol diagnostic independent of cleanup provenance, including
   simultaneous result failure and group-cleanup failure. Do not overwrite it with a
   cleanup category or lose cleanup facts due to diagnostic priority. Existing cleanup,
   clean/state/PID/transport equations, native completion and persistence/watch ordering
   remain byte-for-byte equivalent apart from producing diagnostic facts.
4. Terminal event insertion remains existing scoped audit behavior; no schema marker,
   global record, complete Session/recovery dump, extra foreign scope, permission-grant
   capability or authorization surface. Diagnostics contain no environment/credentials,
   native response bodies, private config, foreign IDs/names, full PS inventory or arbitrary
   child output. Existing Session failure text is not copied into new evidence fields.
5. The existing strict PID-cleared test failure receipt must show only these safe facts
   and actual owned state when it fails. The assertion remains failing for uncertain
   cleanup. Missing event/facts must fail meaningfully, not default to synthetic success.
   Diagnostics alone never fix an underlying retained child or justify a green gate.
6. Do not relax250ms/1MiB bounds, sticky uncertainty, process ownership, clean/PID/Lost,
   executor reservation, stop/retry/recovery/dispatch/native transport rules, native
   settings/auth/hooks or default internal test concurrency. Do not signal an unknown PID,
   disable scopes/sandbox or adopt a caller-supplied recovery/evidence field as authority.
   No native environment/ACP/FS/tool behavior or global config change belongs here.

## Causal acceptance

- Actual private sanitized Grok supervision consumer, using the reviewed existing owned
  Unknown inspection plan, has an independently recorded original protocol/result error
  plus forced cleanup failure. Assert exact scoped event categories/facts, Lost and durable
  executor reservation/transport refusal before matching wording. The existing plan
  delivers real KILL only to the exact owned PGID and forces Unknown inspection. The
  adapter does not call or claim verified reap on that forced failure. The env-cleared
  parent owns/cleans/reaps the sanitized test process; native grandchild reap remains
  unobserved (Tokio orphan handling), not an independent verified native cleanup proof.
- The required forced failure also reaches an actual prepared session/prompt dispatch
  and unknown-native-outcome diagnostic rewrite using a crate-internal dispatch-capable
  sanitized fake. Assert facts survive the original-error rewrite, with dispatched and
  native_outcome recorded independently: Lost from unknown dispatch may have clean true,
  while forced-unclean has clean false. A pre-dispatch /bin/cat failure alone cannot
  satisfy this acceptance or earn its mutant credit. If this dispatch path cannot be
  safely reached, label it source-only and keep this acceptance blocker open.
- Paired verified-clean actual consumer records measured successful cleanup/reap, absent
  cleanup error and known ownership/drain facts, with ordinary unchanged Failed/Exited,
  PID/transport and diagnostic behavior. Fixed injected Unknown need not imply any model
  inference occurred; fixture provenance labels that distinction.
- Consumer coverage for not-attempted/preflight uncertainty and group-success/reap-failure/
  timeout uses private owned seams only if real lifecycle boundary is reachable. Any
  source-only or unit-only category coverage is labelled honestly; do not invent syscall
  observations, actual clean subprocess death or native acceptance from helper enums.
- Select each terminal receipt by exact owning scope, Session and attempt identity:
  prompt when present, otherwise a private operation ordinal. Assert exactly one match
  and complete paging; a truncated100-event lookup cannot fall back to another receipt.
  Resume attempts must not reuse an earlier Session event as current provenance.
- Existing real Grok negative PID assertion includes the bounded receipt. It never accepts
  arbitrary Lost/PID/error alternatives. A future real failure retains concrete cause,
  unavailable facts and original CI log, and remains an acceptance blocker.
- At least one compiled actual-consumer mutant omits/loses the independent cleanup facts
  in a result-error case and dies on the intended event assertion; another removes
  ownership uncertainty provenance if meaningful. Record exact source/base/patch/head,
  failure and restored control; no masked/helper-only false consumer credit. At least
  one intended event assertion kill reaches the dispatched/unknown-rewrite path, rather
  than earning that credit from an initialize failure.
- Immutable native requirements/design/source reviews, verified fixes/rereviews, commit
  before tests. Appropriate targeted/default debug and release workspace, fmt/all-target
  Clippy-Dwarnings, debug/release builds and exact Linux/macOS CI. Preserve unrelated
  cleanup/legacy overflow failures; no blind rerun/dummy commit or serialized acceptance.
  Forced-Unknown consumers/mutants use the existing macOS-only plan; Linux covers clean
  receipts and common compilation/default gates, not forced-failure evidence. Attribute
  each causal credit to its actual OS/job and preserve uncovered categories explicitly.

## Integration

Own Grok terminal provenance only. Coordinate root/native6 owners before shared API changes;
no shared change is presently proposed. Requirement approval precedes design, two independent
STRICT design/source gates precede implementation/acceptance. Root controls merge/close unless
explicitly delegates authorized normal PR workflow. #41 red final gate, #14 actual recovery,
#43 native binding, #51 environment isolation and #16 native dogfood remain independent.
