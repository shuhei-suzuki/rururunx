# Browser verification backend

## Interface and boundaries

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
or required; its capability is false.

The request declares a short exact-origin allowlist. Native URL parsing rejects
credentials, non-HTTP(S) protocols and noncanonical origins. Navigation and redirects
must remain in that list. Playwright HTTP interception denies other origins and
write verbs in read-only sessions; downloads are denied. This is a browser
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
uncertain side effects are never replayed. A fallback uses a new ephemeral profile
under the same scoped evidence session, and records `fallback_used=true`.

## Isolation, failures and evidence

Artifacts are in canonical runtime root / ProjectId / TaskId / SessionId, with
private 0700 directories and no symlink ancestors. Screenshot names are simple
owned `.png` basenames; returned files must be regular files. `artifact_directory`
plus these relative names forms the durable reference. `verification.json` includes
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
maximum 1 MiB), total lifetime (maximum 10 minutes), step lifetime and step count.
Failures normalize to unavailable, unsupported, policy_hold, timeout, assertion,
operation, protocol, output_limit or cleanup. Raw native errors, SDK stack traces,
full HTML and model conversations are not returned. Selected text and semantic
data are individually limited; adaptive operations require an explicit DOM subtree.
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
usage metadata; there is no resumed cumulative-cost ambiguity.
`llm_calls` here counts observed SDK/native callback invocations, not unreported
internal provider API retries. Native terminal tokens, cache read/write, reported
USD cost and API duration are aggregated only when present on every callback;
unknown fields remain null. For ordinary API models the helper reads actual fresh
Stagehand session metrics; unreported price/callback count remains null.
Deterministic runs report measured `llm_calls=0` without inventing token/cost zeros.

## Setup and verification

```sh
cd scripts/browser
npm ci --ignore-scripts --no-audit --no-fund
node --test native-claude.test.mjs
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
Its four callback invocations reported input 8, output 192, cached input 8640,
cache write 49661, USD 0.402888 and API duration 8408 ms. These are one measured
fixture run, not estimates or a performance guarantee. Synthetic fallback/contract
tests do not stand in for these real SDK operations.

## Primary references

- [Stagehand v4 quickstart](https://docs.stagehand.dev/v4/first-steps/quickstart)
- [v3→v4 migration](https://docs.stagehand.dev/v4/migrations/v3)
- [v4 browser lifecycle](https://docs.stagehand.dev/v4/configuration/browser)
- [v4 custom models](https://docs.stagehand.dev/v4/configuration/models#custom-models)
- [v4 observability](https://docs.stagehand.dev/v4/configuration/observability)
- [official Stagehand source and releases](https://github.com/browserbase/stagehand)
- [Playwright BrowserType/CDP](https://playwright.dev/docs/api/class-browsertype#browser-type-connect-over-cdp)
- [Chromium unpacked extension ID algorithm](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/components/crx_file/id_util.cc)
- [native Claude CLI](https://code.claude.com/docs/en/cli-reference)
- [native Claude permissions](https://code.claude.com/docs/en/agent-sdk/permissions)
