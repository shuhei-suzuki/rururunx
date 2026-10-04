# Issue19 encoding component source evidence

Scope: only the independently approved encoding component Design6 (`a3ff963`,
native review `611dd31e-9968-45f1-bb43-90f93e661a07`, no Critical/High/Medium).
Actual merged main remains schema3. Unmerged component source is schema5; schema6,
managed native producers, old-writer drain and full19/23/43/60/native acceptance
remain OPEN. These synthetic codec/provenance facts are not native completion,
settlement, review independence or deployment authority.

The existing `bounded()` now validates complete compact JSON through a scalar
counter/latched-refusal sink, retaining no encoded Vec. Exactly five Formatter
overrides retain composite trait defaults. All11 existing artifact callers inherit
it; three RetainedEvent length sites use the same measure and checked aggregate
arithmetic. Whole-row digest serialization is unchanged. Eight context loader calls
have explicit private read intent; raw typed guards precede named digest/clone/decode
stages, and opaque Task provenance remains opaque.

Observed checks: `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`,
debug/release `cargo build --locked` passed. Full default-parallel `cargo test --locked`
at production/test snapshot `d443258e2e68346f1a23928b569596a51755a9f2` passed235 tests
and2 doctests, with2 installed-native Grok opt-in tests ignored. Later commits change
only fixture priority assertions and bounded failure diagnostics; current16 focused
encoding/reader tests and clippy pass at `3fa9f04`. All three production files are
byte-identical across these test-only revisions. Exact published-head CI and independent
component Source review remain required separately.

Fixtures use owned temporary primary repositories and the actual Store/Worktree/
ContextPacks producer/readers. Deliberate persisted negative corruption temporarily
removes ONLY context_no_update in that owned DB transaction and restores the identical
trigger before commit; production immutability is unchanged. Initial terminal Consultant
history is factual mechanical input, never a native death/managed-owner certificate.
The maximum-depth general ContextVersion/Record round trips do not fabricate a Workflow
phase publication. Sandbox execution recorded an existing native cleanup EPERM failure;
the SAME default-parallel command with process-group permission passed. No serial rerun,
cleanup weakening or failure suppression was used.

The16 controls cover exact1-MiB acceptance/+encoded UTF-8/escape refusal; root120/121;
10,000 empty/nonempty siblings; serialize_bytes composite119/120 boundary; borrowed
8-MiB first child stopping before its sentinel second child; replacement Serialize error
versus first fixed refusal; empty write/sticky non-Interrupted/finish(Ok); raw fragment;
actual retained-event accounting; Task/Goal/phase/checkpoint raw reader order; opaque
GOAL_FORMAT-in-Task history; exact checkpoint identity/digest and Engine-only phase-probe
priority; maximum-depth actual Store codec; actual checkpoint generation and a fitting
TaskPack depth120 becoming PhasePackArtifact121, refused before append. Fitting whole-row
hashes remain unchanged. This proves serialization order, not a physical heap/RSS bound.

Installed serde_json1.0.151 primary source was inspected: CompactFormatter's impl is
empty; write_byte_array uses self.begin_array/end_array; from_str initializes128 and
refuses decrement-to-zero (effective127). Actual cargo feature tree resolves default/std,
not raw_value/arbitrary_precision. A compiled parser probe accepts120/121/127 and refuses128.
Artifacts use well-formed derive/Value; swallowed non-refusal custom serialization errors
have no additional depth-state promise. Dependency/feature drift invalidates the evidence
until the designated differential, composite boundary and Store codec controls rerun.

## Compiled mutation evidence

Each mutant was committed coherently BEFORE compilation/tests in the owned detached
checkout based on `18b0ba0`, then restored with a normal commit. Every mutant compiled
and failed a named runtime assertion (exit101), not a compiler diagnostic. Restoration
passed all16 controls, with clean detached head `4a3fcb38011a963c191d1c30e2dab2152ec2ec9f`.
The final test-only Result assertion helper preserves the exact Err-or-discard-Ok
semantics used by these mutants and avoids dumping oversized fixture bodies.

| Omitted or changed guarantee | Committed mutant SHA | Observed |
| --- | --- | --- |
| byte-limit | `54b10d902b41fe76c6cf489898c1dbbeaf70617d` | compiled; killed |
| depth-limit | `d49d97b416579097ac4bd76e274c3bc37d85be4a` | compiled; killed |
| array-opening | `bf50b34dd09b66eed53d6ec65fbc6c8de096af0b` | compiled; killed |
| object-opening | `123d6e2225f084231af758043397b6cbee62ceb6` | compiled; killed |
| array-closing | `e1d7918d9f1639b0ecc09970eb743866219eb67f` | compiled; killed |
| object-closing | `81653f2acac6249665759682c874567bc027c894` | compiled; killed |
| first-refusal | `1dfe0747b435053a40a12c6d1e094f8c34979310` | compiled; killed |
| eager-buffer | `41a09f6af34333926928e9407dafb5d478c7815f` | compiled; killed |
| byte-array-forward | `0cd2d57764d5c69786b5d3570ebd979afa0ca131` | compiled; killed |
| raw-fragment | `49b6136ada05918e5b6c208081e990800d89cab9` | compiled; killed |
| context-before-digest | `d072885b74c0cbe4692757545edb603202721137` | compiled; killed |
| goal-before-digest | `18598efdb6e591d8e65d20b466e9a690d2a637a3` | compiled; killed |
| checkpoint-before-digest | `828144c5c8e5772fbabd1b208092520285ca6571` | compiled; killed |
| capture-before-clone | `916c8499d8a40f2b3795ce684dc46d60cbfba1c3` | compiled; killed |
| phase-before-clone | `172b5eeb8dee5316f0605ae7eec0bf95bdf36c19` | compiled; killed |
| opaque-goal-format | `951a7a103deee3d50a450e5db9b90c558da9a910` | compiled; killed |
| reader-byte-limit | `1eaa5c2f426ca1851c54805ff5a8636e49cda9fc` | compiled; killed |
| reader-depth-limit | `a82a2bd19fda7efa4e0cb20bbf91d0913aefb263` | compiled; killed |

Global byte/depth mutants reached Ok in the actual Task/Goal/checkpoint/phase readers;
the expected refusal assertion failed. The eager-buffer mutant retained the fixed byte
refusal but visited the forbidden sentinel, so its early-stop assertion failed causally.
Loader-local omissions reached the marked digest/decode stage despite a later post-typed
bound error, proving ordering rather than merely an is_err result. Inner byte-array
forwarding failed the composite120/121 boundary. Four independent begin/end omissions
failed root-depth or sibling balance. Restored success is not production native readiness.

Private evidence manifest is `/private/tmp/rururunx-issue19-encoding-final-mutations.json`;
per-mutant logs use `/private/tmp/rururunx-issue19-encoding-final-mutant-*.txt` and restored
control output `rururunx-issue19-encoding-final-mutant-restored.txt`. These contain synthetic
codec facts only. Public reviewer input is restricted to exact Git source/design/evidence;
no runtime DB, authentication, environment or executor conversation is included.
