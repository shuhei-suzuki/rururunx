# Issue 6: Native Codex verification in progress

This evidence is partial; Issue 6 is not ready to merge. Native TUI/attach,
authentication/provider compatibility hardening, immutable independent source
review, semantic mutations and final exact-head CI remain required.

## Installed native execution

Installed Codex CLI 0.160.0 used existing authentication/configuration/hooks/trust
and its unspecified native model/effort defaults (`gpt-6.1-sol`, `high`). No OAuth
extraction, direct inference API, global config/auth/trust write, bypass flag,
profile inheritance, native reviewer substitution or foreign user repository was
used. Fixtures are two independent temporary committed Git repositories with
Project/Goal/Task/WorktreeManager ownership and durable SQLite Sessions.

At production commit `bba475b1c0177be22a11372bfc87e77a094b9033`, actual Executor
native UUID `01a100b0-7956-70a1-aff3-e36c1e4f6f86` created and physically verified
`result.txt` containing exactly `NATIVE_EXECUTED\n` in its owning Task worktree.
Foreign Scope was rejected; the other Project had no result artifact. The Session
persisted Exited with PID cleared only after confirmed owned group cleanup.
Observed entire-turn native usage: input 22,603, cached input 10,880, output 269,
total 22,872; monetary cost unavailable. This includes both model/tool cycles,
instead of reporting only the final call. Context Pack payload was 216 bytes.

Actual immutable review and checkpoint/resume used the same rrx Session/native
UUID `01a100b0-763f-7082-83a0-b6239875aa5a`. Supplied synthetic `clamp(x)=x-1`
returned the real witness `x=0 => -1`, then `x=-3 => -4` after resume. The second
turn retained native identity/defaults, rejected foreign Scope and persisted
verified Exited. Observed second-turn usage after subtracting the owned prior
gauge: input 9,663, cached 0, output 37, total 9,700; cost null. Checkpoint version
2 and updated 123-byte input were attributed correctly. These model runs verify
native transport/structured review, not independent implementation-source review.

## Committed Rust controls

At `50c027d`, all 131 workspace tests passed, including 34 Codex tests. Formatting,
all-targets Clippy with warnings denied and locked offline release build passed.
Tests cover actual owning Git roots/worktrees, dirty immutable reviews, directory
replacement with identical HEAD, lock ABA, lifecycle/version/scope conflicts,
native PID/PG authentication and exact socket cleanup, bounded WebSocket frames,
early events, cumulative usage, resume baselines and counter resets.

Unix WebSocket callback regressions verify exact one-time replies and audit
intent, Human-route grant rejection, previous-turn/replayed/cancelled replies,
changed Session owner, paused Goal and injected audit failure before wire delivery.
These are protocol fixtures, not a claim of real installed native approval routing
or completed Cross-Agent Approval dogfood. Native default automatic review is
retained; explicit broker opt-in requires its existing client route and is rejected
before inference otherwise. Persistent or additional permissions are unsupported.

The previous native TUI reconnaissance connected an exact stored UUID to an owned
private server and retained its trust prompt. That establishes transport/native
gating only. Final interactive lifecycle and operation evidence remains pending.
