# Issue #1 — Rust executable foundation

Source: Product Requirements v0.7, Architecture, GitHub #1. Workflow: STRICT (configuration isolation boundary).

## Purpose and scope

Bootstrap the fixed Rust core, local CLI, explicit TOML configuration, tests,
format/lint/build CI and a reproducible startup measurement. Keep the execution
path free of supervisor/planner LLM calls.

## Non-scope

Task execution, persistence, PTY implementation and TUI arrive in their dependent
Issues. The no-subcommand CLI prints bootstrap help until #15 supplies the TUI.

## Acceptance criteria

- Rust stack/MSRV/distribution and process abstraction boundaries are documented.
- Debug and release binaries build; `rrx --help` executes locally.
- Unit and executable integration tests exist.
- macOS/Linux CI runs fmt, clippy, test and both builds with a locked dependency graph.
- README documents setup and measurements.
- Startup benchmark emits machine-readable measured timings and its scope.

## Constraints

Rust is mandatory. macOS/Linux are the MVP hosts. Runtime inputs are local; no
cloud control plane. Project overlays are explicit so this foundation cannot
discover and inject unrelated repository configuration.
