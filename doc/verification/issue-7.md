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
