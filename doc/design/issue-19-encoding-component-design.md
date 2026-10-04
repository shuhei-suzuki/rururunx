# Issue 19: bounded artifact encoding component

Status: proposed independent component design gate. Actual merged main is schema3;
the unmerged Issue19 component baseline is schema5. This change neither implements
nor qualifies schema6, managed native ownership, legacy migration or whole Issue19.
Its implementation commit will contain only this encoding prerequisite and its
actual existing consumers. Other branch changes retain their separate open gates.

## Requirement and scope

The existing TaskPack, GoalPack and Checkpoint artifact contract is at most1 MiB
of complete compact UTF-8 JSON. `context_pack::bounded` currently allocates the
entire `serde_json::to_vec` result before comparing its length. A rejected oversized
artifact therefore allocates beyond its declared encoding capacity. Validate the
actual emitted encoding without retaining a rejected encoded buffer. No semantic
truncation or altered hashes/bytes for an accepted artifact is permitted.

This artifact cap is distinct from the final native model-frame1-MiB cap and the
proposed schema6 full ContextVersion8-MiB row cap. No whole ContextVersion, native
Session or Workflow body inherits this component's1-MiB artifact limit.

## Design

Keep the existing private `bounded(&impl Serialize)` consumer. Use a private
counting `Write` sink with checked arithmetic and a serde_json Formatter using
the same trait defaults as CompactFormatter, with ONLY five overrides: begin/end
array, begin/end object and write_raw_fragment. Delimiter overrides write the exact
compact delimiter to the checked sink; there is NO inner CompactFormatter forwarding.
In particular write_byte_array keeps its trait DEFAULT, whose self.begin_array/
end_array dispatch passes through these depth overrides (serde_json ser.rs1804).
Every future composite method must likewise route container openings through the
guard; source review inventories every override and delegation target.
An empty write returns Ok(0) without counting or clearing a latched refusal. Each
nonempty chunk returns Ok(len) or an error, never partial/zero success. Before
accepting, bound actual complete encoded bytes to1 MiB. Formatter
begin_object/begin_array computes checked depth+1; above120 latches Depth BEFORE
writing a delimiter or visiting children; otherwise increments then delegates the
write. Depth counts all nested JSON containers INCLUDING the artifact root as1;
120 opening containers accepts and121 refuses. end_object/end_array delegates the closing write then checked-decrements
after success. All other Formatter methods retain CompactFormatter's exact encoding. Quoted
strings, escaped quotes/backslashes and UTF-8 are handled by serde_json rather than
reparsed by an independent byte scanner. No output buffer or depth-sized stack is
retained. There is no redundant token-count acceptance claim. Actual artifacts use
no RawValue, and current resolved serde_json features omit raw_value/arbitrary_precision (actual cargo tree --locked -e features -i serde_json
--offline reports only default/std; Cargo.lock alone does not prove features).
Override write_raw_fragment with a sticky fixed unsupported-raw-JSON refusal rather
than permit a future raw container to bypass depth. Current accepted artifact
bytes remain unchanged; any future raw/arbitrary-precision artifact requires a
separate reviewed encoding contract, not implicit feature-unification acceptance.

First Bytes/Depth/Raw refusal is latched in shared private validation state. Every later
nonempty write refuses; empty writes do not clear the latch. After Serialize returns, inspect that state even if custom Serialize
swallowed errors: Bytes produces the existing “mandatory pack/checkpoint exceeds
1 MiB; narrow explicitly” error, Depth a distinct fixed artifact-nesting error,
Raw a distinct fixed unsupported-raw-JSON error. Sink/Formatter share one private
Cell-based state. All refusals use io::ErrorKind::Other, NEVER Interrupted (which
write_all retries); first refusal is the only source of truth for later writes.
With no latched refusal, propagate the original Serialize error unchanged. Valid
encoded-length measurement is reused for existing small RetainedEvent accounting
with the SAME1-MiB/depth/raw guard and fixed root-cause errors. MAX_TEXT8192 plus
fixed metadata makes that byte limit unreachable for current valid events; a future
event contract expansion must review these limits. History aggregate add/subtract
uses checked arithmetic; fitting RetainedEvent lengths remain exactly to_vec lengths.

Actual Store decode is `serde_json::from_str` (state/mod.rs:1418); installed
serde_json1.0.151 initializes remaining_depth128 and refuses when decrement reaches
zero (de.rs:63,1375–1377): effective127 containers. The artifact cap120 deliberately
leaves at least7 levels for the current persisted Record/ContextVersion envelope.
No recursion limit is disabled or parser behavior changed. Pin the current envelope
in real Store round-trip tests; a future envelope expansion must retain the margin.
120 accepts and121 refuses at artifact validation; both can reach the existing
Store parser in the consistent-corruption negative fixture.

Guard the persisted checkpoint Value BEFORE digest serialization/typed decode.
The private context loader takes an explicit consumer read intent, not a public
optional authority selector. TypedTask always guards whole data, or ONLY nested
task_pack if phase, before digest/clone/decode even if format is unrecognized.
TypedGoal always guards whole data before digest/decode even if its format is
unrecognized. Those malformed typed loads already fail today; only their error
priority changes. TaskProvenance preserves EXACTLY today's opaque classification:
Cheap phase OR format==FORMAT classification happens BEFORE digest: typed uses
EXACTLY the TypedTask raw guard, opaque has no new guard. Every other Task context
is opaque (including
GOAL_FORMAT in Task scope) and receives no new guard. Actual call inventory:
task_pack515, validate_task_map's phase probe726 and source recheck756, and physical
manifest recheck1198 use TypedTask; goal_pack1393 and validate_goal1406 use TypedGoal;
validate_task_reference1182 and Goal Task-descriptor probe1456 use TaskProvenance.
No generic shape-only GOAL_FORMAT check may reject opaque Task provenance.
Typed loaders keep their own shape rejection. This is not a1-MiB limit on every
historical/general ContextVersion.
For a recognized Workflow phase ContextVersion, guard ONLY its nested task_pack
artifact before its clone/decode (also validate_capture/context_artifact direct
consumers), not the whole ContextVersion. Keep the post-typed artifact check too:
persisted Value bytes and typed re-encoding can differ, for example omitted null
optional fields; both applicable complete-artifact encodings must fit. Existing
scope/kind/version/digest, shape and semantic checks remain mandatory. The whole
ContextVersion digest necessarily retains its separate existing serialization;
this component makes no allocation-cap claim for that full row/native payload.
No publication, pointer, DB schema, transaction, native operation, migration or
public API authority changes. The new120-depth guard and persisted-Value byte
check intentionally refuse malformed/legacy shapes previously fitting only a
shorter typed re-encoding or depth121–125: explicit bounded-artifact errors, no
silently rewritten head or claim of lossless arbitrary legacy acceptance. Actual
Project150–171/Goal234–252/Task284–318 domain projection fields are scalar strings,
paths, IDs/enums/numbers or fixed nonrecursive collections/structs; none is Value or
a recursive domain type (domain.rs). Fixed projection nesting is well below120.
Historically persisted artifact Values remain deliberately subject to the new
raw/depth refusal even if prior typed re-encoding was shallow; opaque contexts are unaffected. Depth/byte correctness applies to well-formed derive/Value serialization. After
a custom serializer swallows a non-latched error inside an open child, counters
need not recover its malformed structure; latched refusals always remain final.
Input Values/custom Serialize code may already own
memory; this guard bounds emitted validation, not arbitrary user code allocations.
Store has already materialized its body String/Value BEFORE a loader guard: this
component prevents further encoding/clone/decode stages at the declared entry,
not prior Store row materialization or a total reload-memory ceiling.

## Actual impact inventory

Source search covers serialized-size measurements and every existing artifact
bounded call, including the child workflow module. Inspect5 direct allocate-to-
measure sites; convert4 (one shared bounded check and3 RetainedEvent measurements).
The11 existing bounded artifact calls inherit the shared check; add the pre-decode
Value guards above. No native or whole-row budget is silently changed.

| Actual site at component baseline | Disposition |
| --- | --- |
| context_pack.rs1568 bounded; callers481/527/631/1107/1352/1402; workflow.rs221/321/361/454/509 | IN: existing1-MiB artifact check; streaming validation |
| context_pack.rs710/1039/1048 recent RetainedEvent encoded accounting | IN: streaming exact encoded lengths, checked aggregate; no Vec just to count |
| context_pack.rs515/530/614/1393 and context_pack/workflow.rs315/359 loaders | IN: matching artifact Value guard before digest/clone/typed decode |
| adapter/grok/schema.rs24 schema16-KiB check | OUT: separately owned provider schema contract, not context artifacts |
| context_pack.rs890 prepare header; context_pack/workflow.rs141/157/285/330/462/472 rendered payload and native-input cap | OUT: actual payload bytes are retained for delivery;1-MiB native frame unchanged |
| context_pack.rs1113 checkpoint previous_pack/reference(c),1293 Goal descriptor reference(c), workflow.rs237 restored_inputs/reference(context) | OUT: caller-side full ContextVersion reference digest precedes loader entry; no artifact-allocation claim |
| context_pack.rs1562 digest,1575 projection; context.rs inventory/source encoding; workflow.rs1036 Context input | OUT: required hashes/materialization/general native contexts; broader allocation/source ownership pending their gates |

## Verification and acceptance

Require exact complete encoded-byte boundary and+1 refusal, including a boundary-
crossing multi-byte UTF-8/escape chunk, and120/121-container boundary. Differentially
compare measured bytes/refusal with serde_json::to_vec on fitting actual checkpoint,
TaskPack/GoalPack fixtures and escaped/control/UTF-8 corpus; preserve exact digests.
Exercise a custom serializer that ignores a size/depth error then replaces it with a DISTINCT
custom Serialize error: the EXACT fixed bound root cause must win, so omitting
post-return latch inspection fails (not merely is_err()). Separately exercise the
real sink + common finalization path with swallowed refusal and Ok(()); reject
Bytes/Depth/Raw despite Ok. No unsafe construction of generic Serializer::Ok. Also
pin refusal kind != Interrupted. Refusal stays latched, and a separate early-stop serializer proves later children are not
visited after the first normal propagated error. Exercise a real existing
checkpoint loader with a genuinely published fixture checkpoint, then an explicitly
corrupted oversize/deep stored artifact negative. Use the following branch-specific
carriers and acceptance points, preserving valid identity/reference digest.

| Actual branch | Byte-excess carrier | Depth-excess carrier | Limit-omitted acceptance point / consistency |
| --- | --- | --- | --- |
| load_checkpoint | mandatory_goal padding (not text constrained to8192) | mandatory_goal Value | load_checkpoint Ok; recompute CheckpointRef digest and recorded accounting unchanged |
| plain TaskPack | decisions | task Value | task_pack Ok, not live validate_task projection/currency |
| GoalPack | cross_task_decisions | goal Value | goal_pack Ok, not validate_goal projection/currency |
| phase context_artifact | source_versions, outside header/native payload | pack.task Value | context_artifact Ok; source_versions mirrored into ContextVersion.source_hashes and exact SourceSnapshot; depth fixture rewrites source payload header, payload_digest, estimated_bytes/tokens, mandatory/optional bytes and source_payload_offset consistently |
| direct validate_capture | source_versions mirrored into SourceSnapshot | pack.task Value | validate_capture Ok; same exact phase/HEAD/scope/budget, depth header/payload/accounting consistency as above |

Byte corruption carriers are only just above1 MiB (one boundary-crossing chunk);
assert each complete stored row remains below8 MiB, including duplicated phase
source_versions/source_hashes. This prevents a future row cap masking the consumer.
No mutant may be credited merely for returning a different later error. Excess
phase bytes in pack itself would hit the separate1-MiB native-frame cap, so do not
use that masked fixture. Loader acceptance is distinct from live source/projection
validation. For pre-guard wiring mutants, forbidden-stage traces separately prove
order even where the retained post-typed guard must still refuse.
Recompute its reference digest,
retain all other valid identity/semantic/history pins, and assert the EXACT byte/depth
error; the size/depth-omitted mutant must reach later acceptance rather than fail
on digest/parser/shape. Corruption is a negative fixture, never positive ownership.
Publish/reload a maximum-depth bounded synthetic general artifact through the actual
Store codec (nongating, no checkpoint/native authority), with a valid checkpoint
producer/control separately retained. Existing embedding is Record.data or
ContextVersion.data, phase artifact at ContextVersion.data.task_pack; source Engine
retains only refs/contextVersion in Workflow history, not a full duplicated artifact.
Pin these actual envelope paths/depths in tests; no future uninspected embedding is
assumed safe. All original digest/semantic checks must still run for a fitting artifact.
The workspace forbids unsafe code (Cargo.toml25); no inline GlobalAlloc, new allocator
dependency or physical heap/RSS ceiling is part of this component. The early-stop
control is causal: custom Serialize emits an8-MiB BORROWED string first, then a
second child that records it was visited. Streaming refusal must leave that second
child unvisited; a compiled revert-to-to_vec-before-count consumes it before the
late length error and MUST fail. Source review verifies the real sink retains only
scalar counters/refusal state, never an encoded output buffer. This is observed
serialization progress plus source evidence, not a quantified allocation theorem.
For EACH loader pre-guard branch, traces start at the named load_checkpoint,
task_pack/goal_pack Typed loader or context_artifact/validate_capture entry, AFTER
any caller-side reference(c) whole-row digest. Include TaskProvenance1182/1456
typed guards and opaque GOAL_FORMAT positive in these ordering controls. Test-only bounded stage traces at the ACTUAL digest/
clone/typed-decode entry record ordering. Oversize/deep refusal must occur before
any such later stage; removing/moving that guard must reach the forbidden stage and
fail the assertion even if a post-typed guard also returns the same error. No authority
or consumer is replaced by the trace. Use fitting genuine artifact controls, exact
recomputed digest and otherwise-valid pins, and assert the named byte/depth error.
Include direct context_artifact/validate_capture preguard branches; whole-context
digest is deliberately allowed for opaque/general contexts, not an artifact guard.
Opaque Task contexts (unknown FORMAT and GOAL_FORMAT in Task scope) in
publish_goal/validate_goal must remain accepted with typed_context=false; a
shape-only GOAL_FORMAT rejection mutant must fail. TypedTask/TypedGoal attempted
unrecognized-format oversized data must hit the guard before digest/clone/decode,
then retain normal shape errors for fitting data.
Separate120/121 pure-array and pure-object nesting controls pin both opening
overrides.10,000 sibling empty/nonempty ARRAYS and OBJECTS pin both closing
overrides including len==0 fast paths. Independently omit each of four begin-limit/
end-decrement overrides: its corresponding actual consumer test must fail. Cheap
kind/scope/version checks precede the raw checkpoint guard, which precedes digest;
foreign-kind/scope/version/version!=1 oversized checkpoint data still yields the
original stale/foreign identity refusal. Correct identity with a stale/wrong digest
AND oversized/deep data intentionally yields the artifact-bound error BEFORE digest;
fitting wrong-digest data retains the original digest refusal. The TypedTask phase
probe intentionally yields artifact-bound refusal before the existing Engine-only
phase refusal. Pin all these priority cases. Wrapped checkpoint-capacity errors
assert the fixed root cause through error.chain(), not a top-level capacity label.
Actual checkpoint-admission negative uses a digest-consistent corrupted PREVIOUS
plain TaskPack with historical_consultation at total artifact depth120; task_pack
loader fits but derived PhasePackArtifact adds1 and refuses121. Drive checkpoint()
and pin typed-phase-capacity context + exact depth root cause, with no
append_pack_checkpoint entry. The Checkpoint domain projection itself is shallow;
do not claim it can naturally produce depth120. A depth121 previous TaskPack is
covered by the loader refusal, not a fabricated admission success.
Also test empty writes, root-inclusive depth and direct raw-fragment refusal/latch.
A custom serializer nests119 outer arrays then serialize_bytes(&[1]) as its120th
container: accepts;120 outer arrays plus byte array is121: refuses. A compiled
write_byte_array override forwarding to CompactFormatter MUST fail this boundary
assertion. Byte/depth refusals are not inferred merely from a later parser failure.

Existing context artifact integration tests must pass. Compile independent mutants
omitting the byte limit and depth limit, with passing controls reaching the intended
actual-consumer assertions. Controlled fixture Git/provenance is component evidence,
not production #60 containment or native #16 acceptance.

Commit before checks; run cargo fmt --check and clippy/test/debug/release with
--locked. Any serde_json version/feature change invalidates this component evidence
until the to_vec differential corpus,119/120+serialize_bytes boundary and maximum-
depth actual Store round-trip are rerun. Verify cited installed1.0.151 source at
the source gate, not Cargo.lock alone. Review the
immutable exact component delta independently, and retain exact evidence. Component
approval does not authorize deploying the other unreviewed branch changes or report
whole Issue19 acceptance. Full schema6/native/legacy/co-integration gates remain open.
