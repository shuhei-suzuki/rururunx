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

## Review and remaining gates

Two immutable published native formal design reviews identified and refined scope,
auth/profile, dispatch CAS, live-tool evidence, schema, filesystem and resume requirements.
Implementation review, compiled boundary mutants, final build gates and exact Linux/macOS
CI are pending. Independent review preserves the default native Claude model and hooks,
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
These changes await committed verification and immutable scoped rereview.
