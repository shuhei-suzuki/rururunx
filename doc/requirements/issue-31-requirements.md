# Issue #31: browser verification backend

Source: [user-authored Issue #31](https://github.com/shuhei-suzuki/rururunx/issues/31).

## Required behavior

- A provider-neutral Rust `BrowserVerifier` manages a fresh bounded verification
  session in an owned Project/Goal/Task worktree. SDK types remain in a thin helper.
- Deterministic navigation, locators, input, click, text/visibility assertions,
  selected text, and screenshot use Playwright without a model client. Adaptive
  observe/act and schema-validated extraction prefer actual Stagehand **v4**.
- Runtime and safe Project policy configure routing, headed display, time budgets,
  and fallback. Project overlays cannot change executables, credentials, artifact
  roots, profiles, or mutation permissions.
- Only exact permitted HTTP(S) origins are navigable. Browser mutation is held by
  default. The explicit loopback-action opt-in is for authorized ephemeral fixtures;
  production/destructive interactions require the future approval/Human gate and
  are held by this backend. A safe fallback must have caller-supplied deterministic
  read-only checks, and must precede any possibly executed mutation.
- Sessions, cookies, native callback cwd, and artifacts belong to exact Scope.
  No shared browser, remote profile, or cookie reuse across Projects occurs. Private
  browser/CLI children are stopped on failure/timeout; profiles are deleted.
- Return compact assertions, selected semantic facts and artifact references.
  Missing token/cache/cost values remain null. Jev is optional, currently reported
  unsupported, and has no correctness dependency.

## Deliverable boundary

This issue provides the library, runtime/Project configuration, pinned Node helper,
and scoped `RecordKind::Verification` envelope. The workflow runner (#12) and
context/evidence integration (#20) consume this API later; their gates, approval
tokens, scheduling and automatic Store persistence are not claimed here.

## Acceptance and evidence

- Rust fixture tests cover routing, no unsafe replay, ownership mismatch, mutation
  holds, scoped evidence, profile/artifact isolation, output/timeout/protocol failure.
- Explicit real Rust→Playwright integration uses a temporary Git Project/Task and
  a synthetic localhost target. It verifies assertions, screenshot and zero calls.
- Real Stagehand 4.1.0→native Claude ClientLLM dogfood verifies observe, act,
  structured extract, assertions and screenshot against a private synthetic target.
  Actual headed mode is checked using Chrome's CDP-reported launch arguments.
- Native callback contract tests check identical process group/cwd, zero built-in
  tools, exact empty strict MCP config, preserved defaults, missing nullable usage,
  and termination when a child ignores SIGTERM. These are synthetic contract tests,
  separate from the real Stagehand/browser evidence.
- Meaningful mutation checks must make the relevant routing, replay, isolation and
  bounded-lifecycle tests fail, then pass after restoring the implementation.

See [browser-verification.md](../design/master/browser-verification.md) for the
protocol, prerequisites, isolation assumptions and reproducible commands.
