# Issue 43: Claude CLI pin 2.1.283 → 2.1.293 — draft for review

Base: `b2eb694` (PR #80 merged). This is a production compatibility change (STRICT), so the design review comes before the code.

## 1. Facts

- **Owner decision.** The owner decided to raise the supported Claude CLI version to 2.1.293. The trigger was the official Native run in #43 (6055469975): the installed `2.1.293` was refused with profile `unsupported_or_unknown`, and no NativeInput or Session was created. The run then used an isolated official `2.1.283`.
- **The pin is an exact single version**, by design. `claude_wire::verify_version` (`execution/claude_wire.rs:26-37`) says: "A pinned local version/schema baseline … Future versions require a capability probe, rather than inheriting permissions."
- **Production sites, all exact-match:**

  | Site | Role |
  | --- | --- |
  | `execution/claude_wire.rs:30` | Version gate on `--version` stdout (first whitespace token) |
  | `execution/native/version.rs:150` | `qualified_profile()` → `"claude-2.1.283"` |
  | `execution/native/compat.rs:83` | Declared `cli_version` must equal the pin |
  | `execution/native/compat.rs:139` | Declared version → profile label, which must equal the observed profile |

- **Test and fixture sites.** These only follow the pin and grant nothing:
  - `execution/native/compat.rs:253`;
  - `config.rs:426, 455`;
  - `runtime/installation/tests.rs:69` and `tests/fm_d2.rs:65`;
  - `execution/native/native_fixture.py:9` (`--version` output);
  - `execution/quota.rs:227` (fixture `source_version` text).
- **Nothing persists the profile label.** It is recomputed from the SAME closed version observation's stdout on every use (`version.rs:138-153`). No schema change or migration is needed.
- **Existing configurations.** A config or a committed original Frame that declares `cli_version='2.1.283'` stops qualifying after the change. The refusal is typed, "native compatibility version unsupported" (`compat.rs:87`), with no effect. The fix is an explicit config change. Nothing is silently rewritten.

## 2. HOW (proposed)

1. **Replace the single pin.** Every production site above changes `2.1.283` to `2.1.293`, and the label becomes `claude-2.1.293`. It stays exact and single: 2.1.283 is no longer accepted. Accepting both is option A2 below.
2. **Nothing else changes:**
   - the wire handling (`control_request` initialize, `system/init`, result and `rate_limit_event` parsing);
   - auth, settings, rules, hooks and permissions;
   - the profile `rrx-native-inherited-v1` and `settings='inherited'`;
   - the Codex pin.
3. **Tests and fixtures** follow the pin: the fixture prints `2.1.293 (Claude Code)`. A negative is added: a `2.1.283` declaration or observation is refused with a typed error and no NativeInput.

## 3. Qualification (needed before the pin claims compatibility)

The comment at `claude_wire.rs:27-29` makes the pin a schema baseline, so raising it needs evidence that 2.1.293's wire matches what `claude_wire`/Native parse.

- **Q-a.** An official Native run with the real CLI `2.1.293` and existing auth, on the same lane as 6055469975: macOS, 1 Project, first implement.
  - Needed: NativeInput and transport confirmed, Session RUNNING → EXITED, `session_bound` with `normal_return`, `gate_claim`, `gate_observed` and `phase_closed`, Unit `work=success`, and no Pending/Unknown effects.
  - Any parse failure (UnsupportedCapability/ParseFailure) means the wire changed. The pin is then not raised, and the wire delta becomes its own design.
- **Q-b.** The `--version` output format of 2.1.293. Its first whitespace token must be exactly `2.1.293`.
- **Q-c.** `rate_limit_event` / quota frames, if 2.1.293 emits them in the run. They must still be parsed, not ignored.

## 4. Controls

| Control | Expected |
| --- | --- |
| C1: fixture `2.1.293` | The installed-lane SC suites pass unchanged in count |
| C2: declared `2.1.283` after the bump | Typed "native compatibility version unsupported", no NativeInput |
| C3: observed `2.1.283` stdout with declared `2.1.293` | `qualified_profile()` None → "observed version differs", no NativeInput |
| Mutant M1: the gate accepts any `2.1.*` | C3 fails |
| Mutant M2: the label is not updated in `compat.rs:139` | C1 fails (profile mismatch) |

## 5. Open question for review

- **A1 (recommended): replace.** Only 2.1.293 is accepted, which keeps a single exact baseline.
- **A2: accept both 2.1.283 and 2.1.293.** This keeps existing configs working, but doubles the qualified baseline. Each version then needs its own Q-a evidence, and the profile label must be selected per observed version.
