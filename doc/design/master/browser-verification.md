# Browser verification backend

## Interface and boundaries

Product v0.8 treats deterministic headed verification as MVP Core. Adaptive
Stagehand/Jev support is optional/stretch; its remaining review/acceptance must
not gate the core supervision runtime.

`rrx::browser::BrowserVerifier` exposes capability discovery and `verify` over a
typed batch of `Step`s. `BrowserBinding::capture` copies durable Project/Task rows;
release the caller's Store lock before `validate`, capability probes or verification.
`WorktreeManager::validate_binding` checks cloned ownership against native Git.
No filesystem, Git, browser, SDK, model call or process wait occurs inside shared
Store access. The caller rechecks current task/project versions before persisting
`verification_record(result)` or attaching its evidence to a workflow gate.

Each verification creates a fresh `SessionId`. Its private Node bridge starts Chrome
in the same owned process group and connects Playwright and, when adaptive,
Stagehand to that private CDP endpoint. SDK `localBrowser.launch` detaches Chrome;
the bridge therefore owns the launch itself and uses official `localBrowser.connect`.
`owned_connect=true` describes this lifecycle. `external_connect=false` and
`connect_external` returns typed `Unsupported`: arbitrary endpoints provide no
reliable proof of Scope/profile ownership. Batch sessions close after completion;
there is no detached leader, session continuation, or remote browser profile reuse.

Rust core knows JSON protocol version 1, typed requests/results and capabilities,
not SDK internals. The helper pins `@browserbasehq/stagehand` **4.1.0**,
`playwright-core` **1.63.0**, and Zod **4.4.3** in a lockfile. Node must be
**22.18+**. Chrome must be installed; default paths support macOS Google Chrome
and Linux `/usr/bin/google-chrome`, with an explicit runtime path override.
Unix process-group ownership is currently required. No sandbox, TLS, web-security
or native permission guard is disabled.

## Routing, policy and fallback

`Auto` selects Playwright for known steps and Stagehand for observe/act/extract.
Even a Stagehand preference keeps a deterministic flow on Playwright and opens no
model client. Explicit Playwright rejects adaptive requests. Jev is not installed
or required; its capability is false. An unavailable/incompatible Stagehand SDK
reports `adaptive=false` and typed `Unsupported` before effects; Playwright
discovery and the configured safe deterministic fallback remain usable.

The request declares a short exact-origin allowlist. Native URL parsing rejects
credentials, non-HTTP(S) protocols and noncanonical origins. Navigation must remain in that list. Every redirect is held before following it:
Playwright `route.continue` does not re-run handlers for redirect hops, so the helper
uses `route.fetch(maxRedirects: 0)` and rejects 3xx responses before fulfillment.
HTTP interception denies other origins and write verbs in read-only sessions;
continuing a write marks `effect_possible` even if no click has started. Downloads
and all page WebSockets are denied. Document responses add `worker-src 'none'`
as an additional CSP policy, preserving the target's existing CSP. Worker script
destinations are blocked before dispatch; a private CDP connection also closes page
workers. These restrictions include blob workers. Apps requiring redirect, socket
or worker behavior return a hold and require a future explicit policy capability.
The exact packaged Stagehand extension resource/worker remains trusted. Installed
4.1.0 defaults OTLP exports to `https://example.com/v1/traces`; exports attributed
to that exact extension source plus `/v1/traces` POST/OPTIONS are aborted before
network dispatch, without making optional tracing a correctness dependency. Page
requests receive no trace exception. This is a browser
verification policy, not an OS network firewall for untrusted browser content.
Browser callbacks/native authentication intentionally use their provider connection.

All click/fill/adaptive-act operations hold by default. Runtime
`allow_loopback_actions=true` only permits explicitly authorized local ephemeral
fixtures (HTTP localhost/127.0.0.1/::1). It is not approval for production services
behind a local proxy or a tunnel. General staging/production mutations await
consumer #12's scoped approval/Human integration and remain held. Project policy
cannot enable this opt-in. The caller must classify its target honestly.

Fallback requires `safe_fallback`, an unavailable/unsupported adaptive backend,
`effect_possible=false`, and an explicit nonempty equivalent deterministic
read-only batch. Assertions, timeouts, protocol errors and operation failures do
not trigger fallback. `effect_possible` is set **before** starting any mutation;
uncertain side effects are never replayed. Rust cleans the primary profile before
fallback, then creates an independent private `fallback` directory/profile under
the same scoped evidence session. Primary failure, usage, evidence and artifact
references remain in the final record; aggregate counters remain null whenever
an attempt lacks the corresponding metric. `fallback_used=true` is set by Rust.

## Isolation, failures and evidence

Artifacts are in canonical runtime root / ProjectId / TaskId / SessionId, with
private 0700 directories and no symlink ancestors. Screenshot names are simple
owned `.png` basenames; returned files must be regular files. `artifact_directory`
plus these relative names forms the durable reference. `verification_file` identifies
the session root's durable record, including when final screenshots use the separate
fallback directory. `verification.json` includes
Scope, backend, session, task worktree, Git revision, assertions and actual usage.
If HEAD changes during verification, the result fails with binding-change evidence.
Git revision does not fingerprint arbitrary concurrent uncommitted files: consumer
workflow locking/provenance remains necessary before treating evidence as a gate.

Each session has a fresh Chrome profile. Browser cookies are cleared and the entire
profile is removed on completion/failure/timeout; Rust also guards cleanup when a
helper cannot complete. Existing browser/auth profiles are never copied. A runtime
API-key environment name must be present in `Project.environment_refs`; values never
enter requests, config serialization, records or error evidence. The process receives
the native user baseline (HOME/PATH/config paths/locale, native Claude/Anthropic
policy/auth/model variables and proxy/TLS/Node settings) and authorized references.
Provider-account baseline variables are reserved for native model access; explicit
Stagehand API mode still requires the Project's authorized key-name reference.
The Store snapshot collects other registered/blocked Projects' environment names:
even a provider-prefixed baseline variable is excluded when another Project declares
it, unless the current Project also explicitly declares the same name. Only names
are snapshotted; values never enter rows or logs. Consumers recapture this snapshot
before dispatch when registry/environment ownership changes.
API account credentials remain in their native user auth system.

The bridge receives at most 64 KiB stdin. Rust bounds output (default 256 KiB,
maximum 1 MiB), total browser/helper lifetime (maximum 10 minutes shared across
fallback attempts), step lifetime and step count. Native Git ownership validation
precedes this browser-phase deadline. The unreaped leader reserves its PGID;
`waitid(NOWAIT)` observes exit before cleanup signals so an EOF-before-exit race
cannot interrupt a completed bridge or recycle its group identity. Group signals
use Rust syscalls rather than an external `kill` executable. On macOS, the shared
bounded native `/bin/ps` inspection accepts EPERM only for a verified dead group.
A live bridge gets up to 1 second between TERM and KILL to emit partial scoped
outcomes; an already exited bridge uses 100 ms. Reaping and pipe drain each have a
500 ms cleanup grace. Valid scoped terminal output remains available when the core
normalizes timeout/cleanup failure. `browser_phase_ms` measures attempts/profile
cleanup without attributing Git pre/postflight time to the browser deadline. An escaped pipe holder
produces typed `Cleanup` instead of an indefinite join; trusted callbacks must
inherit the owned group. An unreapable direct child stays with a private reaper. Thread creation is fallible:
if a reaper cannot start, its unreaped Child is retained and polled on a later browser
call. A self-detaching custom child with private stdio cannot be detected from group
or pipe observations; custom callbacks are trusted programs, not an untrusted OS
sandbox.
Consumers must hold an uncertain Cleanup result rather than replay the request.
Failures normalize to unavailable, unsupported, policy_hold, timeout, assertion,
operation, protocol, output_limit or cleanup. Raw native errors, SDK stack traces,
full HTML and model conversations are not returned. Selected text and semantic
data are individually limited (8192 serialized UTF-8 bytes per selected fact), with
a cumulative evidence budget reserving 16 KiB for scope/failure/usage metadata.
The packaged helper rejects output budgets below 16 KiB before launching Chrome.
Selected text truncates at a code-point boundary; adaptive operations require an explicit DOM subtree.
The caller should still avoid selecting secrets and treat screenshots as private.

Private process groups are created for the bridge; browser and bundled model
children inherit that group. Stop signals TERM then KILL only that owned group;
no global leader lookup, shared session shutdown or process-name kill is used.
Trusted custom callbacks must obey the same child-ownership contract.

## Adaptive model access and telemetry

Configure either a normal Stagehand model with a Project-authorized API-key
environment reference, or an explicit opt-in `custom_command`. The bundled
`native-claude.mjs` implements Stagehand's public `ClientLLM` callback using the
existing Claude CLI account. It does not extract/reuse OAuth tokens in an API client.
It receives only the SDK's pruned factual browser request/schema and exact Scope;
native cwd is the task worktree, with default native model/effort and all existing
rules/hooks/auth retained unless a model is explicitly configured.

The installed **4.1.0** `ClientLLMSchema` expects
`model: {generate: async(params) => result}`. The current v4 custom-model docs show
a function shorthand; this helper follows the installed public exported schema.
Claude is invoked without built-in tools (`--tools ''`), with all tool definitions
disallowed (`--disallowedTools '*'`) and `--strict-mcp-config` plus exact empty
`{"mcpServers":{}}`. No bare/system-prompt replacement, config-source suppression,
auth replacement or permission bypass occurs. Managed policy that rejects these
restrictions produces failure; the bridge never weakens them. Native hooks are
part of the trusted user baseline, not a sandboxed callback interception guarantee.

Native Claude 2.1.283's `--json-schema` cannot consume the SDK draft-2020-12 schema.
The original schema remains factual input; native terminal JSON must be successful,
and its answer is validated with `z.fromJSONSchema` before the SDK receives it.
Native session UUIDs are fresh, terminal-verified and retained in the exact Scope's
usage metadata; `native_session_attempts` separately records IDs supplied to
successfully launched native children when no confirming terminal arrives.
Attempt IDs do not claim successful native session establishment. Failed/aborted
callbacks retain these IDs with unknown numeric telemetry rather than stale
successful-call totals. The inner native deadline precedes SDK/outer callback
bounds by 1.5 seconds; in-flight callbacks are aborted/awaited during bounded
cleanup. Native pipe drain after direct-child exit is bounded to 100 ms: a held
pipe produces failure while retaining complete, verified terminal usage, and
Rust cleans the still-owned group. No callback PID is signalled after Node reaps
it. There is no resumed cumulative-cost ambiguity.
`llm_calls` here counts observed SDK/native callback invocations, not unreported
internal provider API retries. Native terminal tokens, cache read/write, reported
USD cost and API duration are aggregated only when present on every callback;
terminal-verified usage remains available even when native generation/schema
validation fails, without returning raw model errors or webpage content.
Unknown fields remain null. Admission counts concurrent invocations before their
first await, so custom generation stops before a 65th callback.
For ordinary API models the helper reads actual fresh
Stagehand session metrics; unreported price/callback count remains null.
Deterministic runs report measured `llm_calls=0` without inventing token/cost zeros.

## Setup and verification

Executable paths must be absolute or bare program names resolved using the
trusted runtime's absolute PATH entries before adopting the Task cwd. The same
rule applies to explicitly configured Chrome and adaptive callback programs.
Runtime-supplied arguments remain trusted; use absolute helper script paths.
Headed sessions forward native display routing (DISPLAY/WAYLAND_DISPLAY/
XAUTHORITY/XDG_RUNTIME_DIR), with other-Project environment references excluded
unless the current Project also authorizes that name. Valid scoped terminal
results retain usage even on nonzero exit; Rust sets actual fallback facts.

```sh
cd scripts/browser
npm ci --ignore-scripts --no-audit --no-fund
node --test *.test.mjs
# Explicit local Chrome attack/budget tests; no model calls:
RRX_BROWSER_REAL_TESTS=1 node --test network-real.test.mjs
node smoke.mjs
node smoke.mjs --headed
# Explicit real inference through existing native Claude auth:
node smoke.mjs --adaptive --headed
```

Runtime configuration (absolute paths keep task cwd independent):

```toml
[browser_verification]
backend = "auto"
bridge_command = ["node", "/absolute/path/rururunx/scripts/browser/bridge.mjs"]
artifact_root = "/absolute/private/runtime/browser-evidence"
headed = true
safe_fallback = true
# Leave false for ordinary targets; true is only an explicit ephemeral fixture grant.
allow_loopback_actions = false
timeout_ms = 120000
step_timeout_ms = 30000

[browser_verification.model]
custom_command = ["node", "/absolute/path/rururunx/scripts/browser/native-claude.mjs"]
```

```sh
cargo test --offline --locked --test browser
python3 scripts/browser/mutations.py
RRX_BROWSER_NODE=/absolute/path/to/node cargo test --offline --locked \
  --test browser actual_rust_to_playwright_browser_fixture -- --ignored --exact
# Explicit inference through the scoped Rust API and native account:
RRX_BROWSER_NODE=/absolute/path/to/node cargo test --offline --locked \
  --test browser actual_rust_to_stagehand_native_fixture -- --ignored --exact
```

Real host evidence on 2026-10-03: Node 22.19, installed Chrome, Playwright 1.63,
Stagehand 4.1.0 and native Claude 2.1.283. Deterministic headless/headed fill,
click, text assertion and screenshot succeeded with zero model calls. An owned
temporary Git Project/Task also completed the Rust→helper→Chrome assertion and
screenshot. Actual headed Stagehand observe→act→assert→extract→screenshot succeeded,
returning reference **42**; CDP-reported launch arguments proved headed mode.
Its four callback invocations reported input 8, output 375, cached input 8656,
cache write 50141, USD 0.4103912 and API duration 8324 ms. These are one measured
fixture run from native-reported counters, including native cost accounting; they
are not a performance guarantee. Synthetic fallback/contract
tests do not stand in for these real SDK operations.
The reversible mutation harness checks routing, uncertain replay, exact Scope,
artifact symlinks, cross-Project provider credentials, output/time bounds,
native child group inheritance, strict empty MCP, exact CDP extension origin and
independent deterministic availability when the adaptive SDK is missing.
The harness also checks group-signal effectiveness, primary fallback profile/usage
retention, primary artifact validation, and escaped pipe cleanup bounds. Real
receiver-zero tests cover redirect, WebSocket, service/dedicated/blob workers;
a Japanese-text batch proves cumulative output failure still emits bounded usage. Restored-source Rust/Node suites run
after every mutation round.
Rust checks and the no-model Node contract checks run on both Linux and macOS CI;
the explicit real Chrome/native-model fixtures require host setup and authorization.

## Primary references

- [Stagehand v4 quickstart](https://docs.stagehand.dev/v4/first-steps/quickstart)
- [v3→v4 migration](https://docs.stagehand.dev/v4/migrations/v3)
- [v4 browser lifecycle](https://docs.stagehand.dev/v4/configuration/browser)
- [v4 custom models](https://docs.stagehand.dev/v4/configuration/models#custom-models)
- [v4 observability](https://docs.stagehand.dev/v4/configuration/observability)
- [official Stagehand source and releases](https://github.com/browserbase/stagehand)
- [Playwright BrowserType/CDP](https://playwright.dev/docs/api/class-browsertype#browser-type-connect-over-cdp)
- [Chromium unpacked extension ID algorithm](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/components/crx_file/id_util.cc)
- [Playwright route redirect behavior](https://playwright.dev/docs/api/class-page#page-route)
- [Playwright WebSocket routing](https://playwright.dev/docs/api/class-websocketroute)
- [CSP worker-src and intersecting policies](https://w3c.github.io/webappsec-csp/#directive-worker-src)
- [CDP Target auto-attach](https://chromedevtools.github.io/devtools-protocol/tot/Target/#method-setAutoAttach)
- [native Claude CLI](https://code.claude.com/docs/en/cli-reference)
- [native Claude permissions](https://code.claude.com/docs/en/agent-sdk/permissions)
