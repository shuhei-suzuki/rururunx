# Issue 7 verification

Risk: STRICT for native authentication, filesystem/process scope and durable dispatch authority.
All runtime fixtures use isolated temporary Git repositories and normal existing native auth.
No credential extraction, configuration replacement, hook/rule bypass or approval grants.

## Committed verification

- Clean `cab8a08`: full locked workspace tests pass; locked all-target clippy with warnings denied passes.
- Clean `fe380ad`: eight fake native ACP subprocess integration tests pass; installed-native test remains explicitly ignored in ordinary CI.
- Three private cross-connection Store dispatch CAS regressions pass: parent lifecycle versions, exact lock/version ABA and Session/Project-only ownership/activity guards.
- Scoped descriptor FS and bounded schema unit regressions cover aliases, protected authority, FIFOs and local schema validation.
- Actual fake subprocesses cover native version/auth/config gating, concurrent locked reviewers, fresh checkpoint resume/replay isolation, permissions/stop, foreign refs/environment, parser/frame limits, unexplained hook effects, missing FS evidence, independent schema enum/required/extra-property violations, live decision tools and Lost reservations.
- Native slash-command dispatch was verified from first-party source: prepared input now receives a fixed non-command first line, with wire assertion preserving a leading `/always-approve` payload as content.

## Installed-native acceptance

Installed Grok 1.0.46 used its normal cached authentication and native rules/hooks/config.
At clean `f261aa5`, `installed_native_edit_fresh_continuation_and_structured_decision`
passed in 30.64 seconds. It verifies exact owned-file edit, absent foreign sentinel,
fresh PreparedInput v2 continuation preserving the exact native UUID, then a committed
immutable review lock and native structured DENY with zero tool calls. Aggregate actual
input/output token counters are positive; unknown cost remains null. Native model/effort
are explicitly configured through ACP setters and verified. Actual process exit status
is retained separately from privately owned native turn completion, without synthetic zero.

Earlier trials exposed owned session-title notifications without prompt IDs (fixed with a
narrow session-only housekeeping exception and foreign-session regression). One subsequent
trial failed conservatively with an uncorrelated ACP response; its root cause remains
unexplained. It retained Lost reservation after verified process cleanup. Safe response-ID
shape diagnostics were added; the next acceptance passed. That success does not prove the
transient impossible or reinterpret its uncertain dispatch as completed.

## Independent review

Two immutable published native formal design reviews identified and refined scope,
auth/profile, dispatch CAS, live-tool evidence, schema, filesystem and resume requirements.
Implementation review, compiled boundary mutants and exact Linux/macOS CI are recorded
below. Independent review preserves the default native Claude model and hooks,
with tools disabled and an empty strict MCP inventory; it is a manual read-only source
review under the native read-only exception, not Triple Review dogfood.

## Limits

Only Task-scoped file execution and decision consultation/review are advertised.
Protected/dot paths, non-ASCII names, ranged reads, absent write parents, files over
1 MiB and worktrees exceeding the bounded inventory are explicit unsupported/failure cases.
Permissions are always denied; ordinary native default-policy edits may not issue a callback.
ApprovalReviewer binding, shell/PTY/attach, universal interception, native Goals, configured
provider factories and restart recovery remain pending their separate Issues.
Native lifecycle hooks/config discovery are preserved: before/after owned Task observations
reject unexplained effects, but deliberately escaped process groups and native-global hook
side effects are outside the model file-tool scope claim.

## Implementation Review 1 dispositions

Independent native review of public immutable `511f3e3` completed in 461.186 seconds,
with no Critical/High, five Medium and nine Low findings. No tools/MCP or permission
denials; the 349,649-byte source bundle included all shared ownership authorities.

Verified fixes queued for committed verification/re-review: GROK7-01 closes callbacks
immediately on the correlated terminal response and rejects late notifications; 02 counts
only successful FS operations as completed-tool evidence; 03 protects corresponding Task
copies of absolute source-root authority refs; 04 restricts ordinary environment keys and
expands native baseline selectors. Additional fixes address cancellation-safe partial frames
(06), request IDs before effects (07), unsupported live mode/config updates and decision
permission attempts (08), read-only Git optional locks (10), callback/path/time budgets
while joining host workers (11), and auth transport/spawn error distinction (13).

GROK7-05: retain conservative Lost reservation for an aborted decision without an observed
native terminal outcome. This can block Task progression until explicit Issue 13 recovery;
verified process death alone does not establish inference/side-effect outcome.
GROK7-09: retain reviewed shared immediate owned-group kill-before-reap semantics. Installed
session/load acceptance proves the tested conversation survived; it does not guarantee
native-global persistence under every abrupt shutdown. `exit_code=None` for signal death is
honest shared API behavior; signal number is not represented by that field. Reconnaissance
status 143 was a separate TERM probe, not the production adapter's synthetic exit code.
GROK7-12: first-party resume reconnaissance already measured fresh per-prompt aggregates
(71,903 resumed input versus 91,985 previous input); no previous-turn subtraction or fallback
zero is invented. Fake resume telemetry now distinguishes the two turns. Missing reason
continues to disclose uncontracted cost, even with known token counters.
GROK7-14: consultant/executor overlap is conservatively invalidated by authority rechecks;
mutual scheduling exclusion belongs to future runtime/broker integration. Shared helper
semantics are unchanged while concurrent provider consumers integrate the same contract.

Ten initial compiled mutants produced nine assertion kills and one survivor: removing the
final unfinished-tool gate lacked an unfinished-terminal fixture. A causal subprocess
regression now supplies that case; rerun is pending. Restored original baseline fake tests
passed with no crate diff. Source inspection additionally verified a release/begin stale-Arc
race; successful retirement now leaves its begin fence permanently closed, with a private
causal regression. A real paused-native-info second-Store Task replacement fixture checks
no prompt reaches the wire and no durable dispatch consumption is written.

## Implementation Review 2 and verified residual fixes

Exact `4fad23d`: 170 workspace tests, fmt/clippy/debug/release pass; installed Grok
acceptance passes in 33.81 seconds. Linux and macOS CI run `37120879240` both succeed.
Ten scoped delta mutants all fail by assertions, including the original unfinished-tool
survivor. Across both rounds: 20 executions, 19 distinct boundaries, 19 assertion kills
plus the original survivor subsequently killed by its missing fixture. No build errors.
Restored baseline has no crate diff, normal fake nine-test suite passes; owned detached
mutation worktree removed normally.

Independent Round 2 completed in 266.556 seconds on public immutable `4fad23d`, with no
Critical/High/Medium and no unresolved blockers. Five Low residuals were verified and
fixed/hardened for Round 3: R2-01 terminal actor guard disarms before publishing idle,
preventing late Drop from clearing retirement/new claims (causal old-guard/release/Drop
private test); 02 an observed preflight stop prevents native spawn (actual fake startup
sentinel); 03 absent rule-copy protection folds ASCII case and directory refs are invalid;
04 successful read/write method binds one unambiguous live tool, post-count delta equals
observed native tools; 05 failed dispatched turns also reconcile after verified cleanup,
with separate factual attempt/error audit. Unknown outcome remains Lost regardless of
reconciliation or process death; known failures are not converted into successful results.
These changes passed committed verification below and await immutable scoped rereview.

## Round 3 committed gates and native observations

Clean `5bf217c`: 170 locked workspace checks pass with serial fixture execution
(including two doctests); two installed-native tests are explicitly ignored in ordinary CI.
Fmt, locked all-target clippy with warnings denied, debug and release builds pass.
The three cross-connection dispatch CAS tests also verify atomic scoped intent journaling,
rollback and exclusion of private recovery payload. Shared `Session.saved` projects only
`dispatch_intent` alongside existing ownership evidence; Grok records input version and
prompt ID before the same atomic consumption/write fence. No schema/helper contract change.
Copied scope snapshots filter wildcard record queries to exact nullable scope.

Seven new compiled R2 mutants all fail by assertions: terminal guard disarm, requested
and configured rule-case aliases, observed-stop/no-native-creation, callback method,
post-tool count and failed-turn observation. Totals: 27 executions, 26 distinct boundaries,
26 assertion kills and the original survivor subsequently killed. Restored baseline has no
crate diff; nine fake subprocess tests pass. Owned detached mutation worktree removed normally.

Installed acceptance at clean `5bf217c` passes in 40.08 seconds: exact owned edit, foreign
write denial, explicit higher-version continuation with the same native UUID and immutable
locked zero-tool schema-constrained DENY. A separate isolated decision-only correlation
test at `80d3e42` passes in 8.79 seconds with no worktree changes.

Two intervening actual-native trials failed conservatively. At `fbb0c0f`, execution and
fresh continuation succeeded, but the decision received an unexpected string RPC response
ID while a numeric ID was owned. At `1d3f15d`, continuation cleanup could not verify group
death because the trusted macOS `/bin/ps` inspector exceeded its unchanged 250 ms deadline.
Neither cause is retroactively attributed to the other or declared impossible by later
success. Both retain Lost reservation. Safe response-shape/ownership boolean diagnostics
contain no raw IDs, result content or credentials. No numeric/string coercion, unknown-frame
exception, inspection-deadline increase, death-guard relaxation or uncertainty-latch reset
was introduced. The bounded inspector may conservatively require explicit recovery under
host load; actual native dispatch completion and process death remain separate authorities.

## Implementation Review 3 dispositions

Independent native Round 3 of immutable public `83bc484` completed in 190.001 seconds,
with no Critical/High/Medium or unresolved blockers, and verified all R2 dispositions.
Five Low refinements were verified: reject noncanonical/symlinked configured authority
refs instead of protecting only their target name (R3-01); reduce unmatched-response
diagnostics to shape/equality booleans including local numeric-counter checks (02); assert
Grok's own atomic dispatch intent audit from the fake process before prompt handling
(03); require successful writes to correspond to an unfinished owned search_replace tool,
and fence reads by current-turn correlated paths (04); document stop/reconciliation
latency and helper uncertainty raising known failure to Lost (05, availability only).
R3-04's initial strict read gate at `470a57e` failed installed-native acceptance in
16.77 seconds. Boolean-only diagnostics at `f0a7269` reproduced it in 20.12 seconds and
proved the extra successful read revisited a path already correlated to a finished tool
in that same turn. Its native purpose is not inferred. The narrow compatibility rule
permits that descriptor-scoped supplemental read, or a single pending search_replace
target-read dependency, without any extra tool or write completion credit. Unseen
existing-file reads and unmatched writes remain fatal Lost outcomes; decision roles
still expose no filesystem callbacks.

Clean `d2a9f8f`: nine fake ACP subprocess tests pass (two installed-native tests ignored),
including positive existing-file read-before-write and finished-path supplemental reads,
and negative unseen-file reads/unmatched writes. All-target clippy, fmt and debug/release
builds pass. Installed native combined acceptance passes in 29.97 seconds after resume:
owned edit, absent foreign write, explicit higher-version same-UUID continuation and
locked schema-constrained zero-tool DENY. This isolated native acceptance establishes
the named scope boundary; it is not a default-concurrency or four-Task throughput proof.

Three R3 compiled mutants assertion-kill configured symlink alias protection, Grok's own
pre-wire dispatch-intent assignment and reverse callback evidence. The reverse mutant
causally returns Exited for an unmatched write where Lost is required. One additional
supplemental-path mutant returns Exited for an unseen existing-file read and is killed.
Totals: 31 executions, 30 distinct boundaries, 30 assertion kills and the original
survivor subsequently killed. Latest restored baseline passes all nine fake tests with
no crate diff; owned detached mutation worktree removed normally. A temporary mutant
whitespace staging failure occurred before any test and is excluded from execution counts.

Exact `83bc484` and `7d2d2a5` Linux/macOS CI pass using default test concurrency. The
latest supplemental-read refinements await immutable delta rereview and exact-head CI.

## Implementation Review 4 dispositions

Public immutable `8074e6c`: independent native Round 4 completed in 144.190 seconds,
with no Critical/High/Medium or unresolved blockers, verifying all R3 dispositions.
Exact default-concurrency Linux/macOS CI run `37158507627` passes. Three Low refinements
were verified: possible write effects preceding a failed sync must remain separate from
success and invalidate completion (R4-01); supplemental reads require a finished tool
with an actually successful FS callback, including one that later reports failed, pinned
by a fake process case without any new credit (02, semantic clarification); foreign
notification diagnostics now use booleans instead of copying native-controlled kind text
(03, bounded diagnostic refinement).

Write authority is checked before any worker can create/write; attempted unowned writes
remain Lost without file effects. Host results independently carry possible-effects
evidence and are audited before post-operation checks. A sticky failed-effect marker
denies completion even if native reports end_turn and inventory matches intended bytes.
A test-only sync_data fault applies bytes, reports failure, proves reconciliation alone
would accept them, and requires the same callback-evidence consumer to reject completion.
Clean `758bdf2`: ten private Grok tests pass; all-target locked clippy with warnings
denied, fmt and debug/release builds pass. Unsupported filesystem methods are rejected
before any worker or `grok.fs_observed` event, pinned by a real fake ACP subprocess.
The restored full fake suite passes all nine tests in 36.17 seconds, with both installed
tests explicitly ignored. Installed combined acceptance passes in 47.74 seconds:
owned edit, absent foreign write, explicit fresh-input same-UUID continuation and locked
zero-tool locally validated structured DENY with actual aggregate usage. This is an
isolated scoped diagnostic acceptance, not a default-concurrency throughput claim.

Three R4 compiled mutants are assertion-killed: removing failed-write-effect taint
incorrectly accepts completion despite the sync failure; removing possible-effect
observation violates the actual applied-write fixture; removing Actor's pre-write
authority creates an unowned file, caught independently of its eventual Lost state.
Totals: 34 executions, 33 distinct boundaries, 33 assertion kills and the original
survivor subsequently killed. Restored baseline has no crate diff; its full fake suite
passes and the owned detached mutation worktree was removed normally. Immutable
scoped Round 5 rereview and exact default-concurrency Linux/macOS CI subsequently pass
at `6f10dff`, as recorded below.

## Implementation Review 5 dispositions

Independent native Round 5 of public immutable `6f10dff` completed in 134.315 seconds,
with no Critical/High/Medium or unresolved blockers. All three R4 fixes are verified;
exact default-concurrency Linux/macOS CI run `37160386577` succeeds. Three Low refinements
were checked: R5-01 identifies a caller-wiring test gap, with current code correct. A
private actual Actor callback fixture injects the same applied-write sync failure and
asserts both scoped audit possible-effects evidence and refusal of completion; its
owned piped echo subprocess is explicitly reaped before assertions. R5-02 synchronizes
the master design's pre-syscall ownership and failed-effect completion invariants.
R5-03 checks two intentional bounded diagnostic sites: `grok.fs_observed` retains the
callback path (at most 4096 bytes), and inventory reconciliation reports the unexplained
entry name (bounded by depth/filename limits). Both retain factual scope/effect evidence,
are data rather than authority and may include foreign paths. Boolean-only unmatched
identity/kind diagnostics remain separate. No general claim that all diagnostics exclude
native-controlled text is made.

Clean `a9a468e`: the actual Actor fault regression and all-target locked clippy pass.
Two caller-wiring mutants are causally assertion-killed: passing false to TurnEvidence
incorrectly allows completion, and recording false in the scoped FS audit loses the
actual possible-effects fact. Both exercise the unchanged production Actor callback;
the owned echo process is reaped before either failing assertion. Totals: 36 executions,
35 distinct boundaries, 35 assertion kills and the original survivor subsequently killed.
Restored baseline passes all eleven private Grok tests, has no crate diff, and the
owned detached mutation worktree was removed normally. This final delta changes tests
and documentation only; installed acceptance at `758bdf2` verifies the identical
production implementation. Immutable scoped rereview and exact-head CI subsequently
pass at `29f0613`, as recorded below.

## Final scoped review and CI

Public immutable `29f0613`: independent native Round 6 completes in 61.897 seconds,
verifies every R5 disposition and finds no Critical/High/Medium/Low actual defects on
the changed causal paths. Exact default-concurrency Linux/macOS CI run `37160836363`
succeeds, including fmt, locked all-target clippy, full workspace tests and debug/release
builds. The final outcome-only documentation delta preserves this reviewed code; its
exact publication checks and conclusion are also recorded on PR 40.

Two informational limits were verified and retained. The Actor sync-fault fixture ends
at the shared TurnEvidence completion consumer; the supervise-to-transport composition
is separately pinned by the unfinished-tool fake subprocess and its compiled mutant.
A panicked FS worker or failed audit transaction may omit the factual possible-effects
record, but propagates an error and keeps the consumed dispatch Lost, never completed
or automatically replayable. Post-turn observation still checks unexplained effects.
Optional extra audit method/path/count assertions do not represent an unverified
completion or ownership claim. No process inspector deadline, death guard, uncertainty
latch, shared CAS, schema or native safety/auth configuration was changed by this delta.

## Issue acceptance coverage

The current GitHub Issue 7's five criteria are mapped explicitly: configured prepared
execution/review input and actual owned edits; ACP-verified explicit model/effort;
collected locally validated native structured verdicts; normalized auth/unavailable/
protocol/parse/state/timeout/Lost failures; and concurrent independent Reviewer Sessions
through the explicitly registered object-safe AgentRegistry. The last fixture also
retains a separate concurrent schema-constrained pair through the native inherent API.
It proves adapter participation, not Review Set scheduling/policy/aggregation (Issue 9).
ApprovalReviewer binding, native attach/PTY and configured provider factory are explicit
later integration boundaries, not unimplemented GitHub Issue 7 acceptance criteria.
The added Registry consumer test changes no production implementation; its committed
targeted gate at clean `26d1074` passes in 8.43 seconds. Locked all-target clippy and
fmt pass. The initial test comparison failed to compile because Session intentionally
does not implement PartialEq; it now compares the complete serialized snapshot. This
was a test-only compile correction, not a mutation result or runtime failure.
The earlier outcome-only `9d233b1` exact Linux/macOS CI run `37161109321` also succeeds.
Final immutable Registry-consumer rereview and exact-head publication checks are
recorded on PR 40; no native/runtime code changed since reviewed `29f0613`.

Public immutable `77b5f7b`: focused independent native Round 7 completes in 64.036
seconds with no Critical/High/Medium/Low actual defects or blockers. It verifies genuine
dynamic Registry dispatch for concurrent launch, subscribe, status, owned completion
and release, plus truthful coverage of all five Issue criteria. Exact default-concurrency
Linux/macOS CI run `37161473710` succeeds with all required test/lint/build gates.
Reviewer trait start already requests a minimal object schema; the Registry fixture
asserts transport/lifecycle rather than that output's contents. Explicit constrained
schema collection is verified by the retained native pair and installed acceptance.
Custom schema dispatch through the generic trait remains the upcoming Issue 6 additive
integration. The status comparison is backed by `entry`'s persisted Session ownership
check; no extra Registry-specific mutation result is claimed. Final outcome-only docs
preserve this independently reviewed implementation and consumer test.
