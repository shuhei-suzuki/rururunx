# M0 U5 verification — fresh P, Session decoder additions, IPC refusal original and budgets

Input HEAD: `f0ea4fc0`, on top of `0966a0a5` (review round 1 part B), `d0a03bae` (round 1 part A), `8aa652e3` (host fix 1) and `5cc86047` (U5 implementation), from the approved U4 at `fc6038c8`. Local macOS; Cargo `/Users/shuheisuzuki/.cargo/bin/cargo`, locked/offline. Every suite and mutation ran outside the command sandbox under the standing local-verification approval, at a committed HEAD with a clean tree before and after. No network, CI, deployment or live Agent qualification.

## Result and scope

- **Fresh P in the Preparing actor.** After all-alias S Passed, admission opens into Preparing. The retained actor performs Codex Version → Schema → Handshake → the 13 Git actions, each fresh; S never substitutes for any P step. P rehashes the CLI before Version and requires equality with the S identity; every Native helper rehashes, and the spawn boundary rechecks stat. Git keeps its own Source seal. Deadlines: Version and Git 30 s, Schema 60 s, Handshake 90 s, correlated response 30 s. Cancellation, deadlines, fence loss, overflow and I/O failure stop the group immediately; the 5 s self-exit grace applies only after Handshake completion or a protocol refusal; drain 2 s and reap 10 s stay inside the action deadline. EOF of both pipes and the leader reap are separate physical facts; missing facts stay Unknown. The receipt records the stop method and signal.
- **Private P directory chain.** P keeps descriptors for the owner root, `units`, the Unit and `tmp`, requires runtime-UID ownership and no group/other write on each, and rechecks the `(dev, ino)` chain before Schema spawn, read and cleanup. `ResourceManager::materialize` creates the Unit directories 0755 (reduced by umask). The Schema child is fresh 0700; it is recorded `not_cleaned` with the Unit id before creation and `cleaned` only after successful cleanup.
- **Schema additions → Session decoder.** Only this P's qualified Schema original supplies the Handshake and Session contract; the default contract stays closed; additions are memory-only.
- **Pre-registration refusal.** The Codex Executor is refused before registration (`ExecutorPreparationUnavailable`) with Session, transport and NativeInput at zero. Known goes through the existing nongrant closure and explicit retry; an uncertain original stays Held with its lease. The nongrant release now retires the Source handoff between the supervisor slot and the job.
- **IPC refusal original.** After the real peer / request / Unit / epoch / cookie checks, the marker refusal is classified `CookieAcceptedMarkerRefused`; attribution needs exactly one active helper and, where the peer pid is available, a matching process group; Granted-tool re-validation is not recorded; at most 32 requests. The original travels helper → `safe_receipt` → settlement → `ManagedEffect.receipt` → `Inventory::read` / `EffectImage::decode` → history match. Receipts never create authority.
- **Budgets.** Codex Executor B230 → prefix 246 → Prepared 246, checked on real installed history at the boundary and +1; Reviewer 232/248 and Claude 237/252/255 as policy only. The 16-item / 128-byte receipt bound applies to `native_phase_` rows only; other kinds keep their own bounds and stay readable.

No auth/CAS/owner/marker/fencing/#19/process-custody or quota check was relaxed; no Task.version authority, test authority constructor, SQL success seed, `allow(dead_code)`, suppression, ignore/skip/serialization or timeout-only change was introduced; no persisted value was removed or renamed.

## Host-verification corrections

| ID | Defect | Fix |
|---|---|---|
| H1 | Nongrant release retired the supervisor slot and the job but not the Source handoff, so shutdown stayed pending with retained originals. | Production fix: the release calls the existing `PhaseHandoffs::retire_closed` between the two; the dispatcher holds a `Weak` reference (`8aa652e3`). |
| H2 | The U4 I-S control still expected composition to be refused after S Passed. | It follows the U5 boundary (composition opens, Executor refused before registration, Session/transport/NativeInput zero); the four S shim checks are unchanged (`8aa652e3`). |
| H3 | The two-actor P control accepted two Tasks in one Project, which the one-Active-Task rule forbids. | One Task in each of two Projects under one Runtime (`8aa652e3`). |
| H4 | A round-1 test queried a nonexistent `managed_effects.kind` column. | It counts effects through the production body decoder; 44 SQL sites in the touched tests were checked (`f0ea4fc0`). |

## Final verification

At `f0ea4fc0` (`.rrx/u3/host/full12/`): fmt, clippy and the bin build pass; lib 841 / 110 / 27; every integration target and both helper crates pass except `daemon_native` 1 / 2 / 0. All 20 U5 tests pass. The five failures outside the alone baseline (grok environment ×2, `adapter::tests` preflight ×2, the u3 aggregate) pass alone (parallel-only). The remaining failures are a subset of the U4 alone baseline; 11 baseline lib controls now pass because RN-H1/RN-P1, DC1–DC4 and seven S2 candidate / occupancy / final-claim / legacy-limit controls moved to the genuine S/P-capable fixture. The others stay blocked because their shared peers do not answer S Schema/Handshake; they are carried to U6, where the peers become S-capable and the Executor path opens.

## Mutations

All 237 operators ran once at `f0ea4fc0`, after the round-2 fixes (`.rrx/u3/host/mut-u5r2/index.tsv`, logs under `.rrx/u3/host/mutations-u5r2/`, failing assertions in `mut-u5r2/kill-reasons.tsv`). Each run must execute exactly one test; originals are restored by hash with a clean-tree check. Earlier partial runs at `8aa652e3` and `0966a0a5` were stopped on purpose because the following fixes changed the tests and operators.

| Outcome | Count | Detail |
|---|---:|---|
| Killed at the intended check, restored run passes | 227 | No kill at a SETUP step. 137 (depth guard removed) is killed by the unbounded recursion it prevents (stack overflow). |
| Killed after a rerun without incremental compilation | 7 | 166, 168, 176, 179, 190, 200, 226 first hit a rustc 1.91.1 incremental-compilation ICE (`evaluate_obligation` on an async block in `version.rs`) and did not reach the test. With `CARGO_INCREMENTAL=0` each reached its test and failed at the intended assertion; restored runs pass (`mut-u5r2/notrun-rerun.tsv`). |
| Equivalent (layered bound) | 3 | 36, 105 and 110 as in U4; the combined mutants 36s, 105s and 110s are killed at this HEAD. |

The reserve-side Git seal check is kept as defence in depth and is not an operator: over production-constructible plans it is equivalent to the planner check, which is killed (review round 3 confirmed).

## Review

| Round | Reviewer | Result |
|---|---|---|
| 1 | Sol (implementation), Opus (security/contract), HEAD `8aa652e3` | Sol: I01 directory permissions, I02 stale RN expectation, I03 mutation detection (Medium, required), I04 duplicate (Low). Opus: M-1 receipt bound over all rows, M-2 grace on cancel/fence/overflow (Medium, required), L-1..L-4 (Low). All verified. |
| 1 fixes | Sol (part A for Opus findings), Sol (part B for Sol findings) | The Opus author path refused at its precondition (global Git config findings) and launched nothing, so Sol fixed part B; independence kept by an Opus re-review. |
| 2 | Opus, `0966a0a5` | All nine round-1 findings resolved; **コードセキュリティ承認**; five optional Low (N-1..N-5), fixed with H4 in `f0ea4fc0`. |
| 3 | Opus, `f0ea4fc0` | N-1..N-5 and H4 resolved; the reserve-side seal check is equivalent over production-constructible plans; **コードセキュリティ承認**. R3-1..R3-4 (Low/Info, test hardening) → #105. |

L-4 (Claude Reviewer has no dedicated pre-registration refusal once the Codex refusal is removed) is a precondition of U6.

## Limits

- 105 lib controls from the alone baseline (including the known bootstrap file-reader) and the two `daemon_native` controls still fail alone because their shared peers are not S-capable; U6 must move them to S-capable peers or record why each is retired.
- Ancestors above the owner root, the same UID and ACLs remain outside the P directory check (design §2).
- Physical-unknown subprocess outcomes are covered by inventory-uncertainty controls, not by every process-custody fault.
