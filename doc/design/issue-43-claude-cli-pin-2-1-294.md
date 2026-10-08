# Issue 43: Claude CLI pin 2.1.283 → 2.1.294 — A1 decided by the owner; Sol 6058424878 M01, L01 and L02 addressed; implemented `cad5113`

Base: `b2eb694` (PR #80 merged). This is a production compatibility change (STRICT), so the design review comes before the code.

## 1. Facts

- **Owner decision.** The owner decided to raise the supported Claude CLI version, to exactly **2.1.294** and with option A1 (replace). The trigger was the official Native run in #43 (6055469975): the then-installed `2.1.293` was refused with profile `unsupported_or_unknown`, and no NativeInput or Session was created. The run then used an isolated official `2.1.283`.
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
  - `execution/quota.rs:227`: the `rate_limit_event` `source_version` text. This is **production metadata**, stored in `quota_windows` (Codex 6058354808), so it follows the pin as `Claude Code 2.1.294/rate_limit_event`.
- **Qualification re-derives the profile; the audit keeps it (L01).** Qualification never reads a stored profile. It re-derives the profile from the SAME closed version observation's stdout every time (`version.rs:138-153`). The profile is written into the helper receipt for audit (`version.rs:201` → `state/.../version.rs:788` → `closure.rs:390`), so historical receipts keep `claude-2.1.283`. That history grants nothing, so no schema change or migration is needed.
- **Existing configurations (L02).**
  - An installed declaration of `2.1.283` is refused, typed, with "native compatibility version unsupported" (`compat.rs:87`).
  - An installed `2.1.294` with a retained original Frame declaring `2.1.283` is refused by the Frame digest check (`compat.rs:97`), a generic refusal. Editing the config does not update a retained Frame; that recovery is outside this change.
  - In both cases, "no effect" means no NativeInput, Native invocation or Session. It does not mean an unchanged database: the preparation readiness commit (`preparation.rs:1048`) happens before the static compatibility check.
  - Nothing is silently rewritten.

## 2. HOW (proposed)

1. **Replace the single pin.** Every production site above changes `2.1.283` to `2.1.294`, and the label becomes `claude-2.1.294`. It stays exact and single: 2.1.283 is no longer accepted. Accepting both is option A2 below.
2. **Nothing else changes:**
   - the wire handling (`control_request` initialize, `system/init`, result and `rate_limit_event` parsing);
   - auth, settings, rules, hooks and permissions;
   - the profile `rrx-native-inherited-v1` and `settings='inherited'`;
   - the Codex pin.
3. **Tests and fixtures** follow the pin: the fixture prints `2.1.294 (Claude Code)`. Negatives are added, each with no NativeInput: a `2.1.283` declaration is refused with the typed `native compatibility version unsupported` (C2), and a `2.1.283` observation is refused with the generic helper-qualification refusal (C3, §4).

## 3. Qualification (needed before the pin claims compatibility)

The comment at `claude_wire.rs:27-29` makes the pin a schema baseline, so raising it needs evidence that 2.1.294's wire matches what `claude_wire`/Native parse.

- **Q-a.** An official Native run with the real CLI `2.1.294` and existing auth, on the same lane as 6055469975: macOS, 1 Project, first implement.
  - Needed: NativeInput and transport confirmed, Session RUNNING → EXITED, `session_bound` with `normal_return`, `gate_claim`, `gate_observed` and `phase_closed`, Unit `work=success`, and no Pending/Unknown effects.
  - Any parse failure (UnsupportedCapability/ParseFailure) means the wire changed. The pin is then not raised, and the wire delta becomes its own design.
- **Q-b.** The `--version` output format of 2.1.294. Its first whitespace token must be exactly `2.1.294`.
- **Q-c.** `rate_limit_event` / quota frames, if 2.1.294 emits them in the run. They must still be parsed, not ignored.

## 4. Controls

| Control | Expected |
| --- | --- |
| C1: fixture `2.1.294` | The installed-lane SC suites pass unchanged in count |
| C2: declared `2.1.283` after the bump | Typed "native compatibility version unsupported", no NativeInput |
| C3: observed `2.1.283` stdout with declared `2.1.294` | The SAME version observation does not qualify (`qualified_profile()` is None), so the helper qualification refuses first, with "original helper qualification failed or remains unknown" (`version.rs:244-246`, via `prepare_phase_git`/`closed()` before `compat.observe()`; M01). No NativeInput or Native invocation |
| Mutant M1: the gate accepts any `2.1.*` | C3 fails |
| Mutant M2: the label is not updated in `compat.rs:139` | C1 fails (profile mismatch) |

## 5. Open question for review

- **A1 — decided by the owner: replace.** Only 2.1.294 is accepted, which keeps a single exact baseline.
- **A2: accept both 2.1.283 and 2.1.294.** This keeps existing configs working, but doubles the qualified baseline. Each version then needs its own Q-a evidence, and the profile label must be selected per observed version.

## 6. Review outcome and implementation

| Item | Verdict | Implemented |
| --- | --- | --- |
| HOW §2 (A1, four sites) | Sol 6058424878: REQUEST CHANGES, M01 only | `cad5113` |
| M01: C3 contract | Mandatory | C3 asserts the real helper qualification refusal (§4) |
| L01: profile persistence wording | Optional, adopted | §1 |
| L02: old Frame and "no effect" wording | Optional, adopted | §1 |
| Q-a, Q-b, Q-c: official 2.1.294 run | Codex 6058354808: pass on macOS, 1 Project, first implement | — |

Controls (non-root):

| Control | Result |
| --- | --- |
| Gate unit `version_gate_accepts_only_the_exact_pinned_release` | pass |
| C2 `pin_c2_old_declared_cli_version_is_refused_before_native` | pass |
| C3 `pin_c3_old_observed_cli_version_is_refused_before_native` | pass |
| M1: the gate accepts any `2.1.*` | gate unit FAIL; C3 FAIL (no refusal observed) |
| M2: the label stays `claude-2.1.283` | Claude success lane (`br3_claude_linked_plan_is_never_marked`) FAIL |
