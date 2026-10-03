# Issue #1 — Rust foundation decision

## Architecture and distribution

Use Rust 2024, minimum 1.91, with CI/development pinned to 1.91.1. One workspace
member `crates/rrx` contains the reusable library and CLI binary. Keep boundaries
as library modules until independent release/version needs justify more crates.
MVP installation builds a single release binary with Cargo on macOS/Linux;
prebuilt signed release packaging is separate future work. License remains TBD.

Intentional initial dependencies: clap for parsing/help, serde/serde_json for
typed configuration and future persisted event payloads, TOML for config and
anyhow for contextual CLI errors. Do not initialize an async runtime for help.
Subsequent process supervision uses Tokio only for needed process/io/signal/time
features, SQLite via rusqlite, and a platform PTY backend behind the adapter.
Native CLI providers must not leak into scheduling/domain modules.

## Configuration

`--config` loads runtime-global TOML. `--project-config` loads one explicit project
overlay recursively by key (arrays replace). Unknown fields, malformed TOML,
missing explicit paths and zero concurrency/context budgets are errors. Agent
models/efforts are optional values, without provider defaults at this layer.
Configuration is not a permission grant; workflow minimum combination belongs
to #8 and native safety remains authoritative. No secrets are printed by
`config-check`. Registered-project discovery belongs to #26.

## API, data and impact

`Config::load`/`validate` and `WorkflowClass` become shared runtime contracts;
their consumers will be registry, scheduler, workflow and context layers.
No database schema or external API change. `target/`, `worktree/` and `.rrx/`
are ignored. CI uses read-only repository permissions.

## Verification strategy

Unit checks protect default parallel capacity, overlay semantics and fail-closed
config validation. CLI integration checks help/version, config defaults and
missing-file errors. Both build modes plus fmt/clippy run on macOS/Linux CI.
Warm-start measurements report platform and sample count without claiming agent
execution/idle overhead. Idle CPU/memory measurement requires the live scheduler
in #14/#27 and is not inferred from a help invocation.
