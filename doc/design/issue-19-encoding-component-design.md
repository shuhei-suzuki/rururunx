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
counting `Write` sink with checked arithmetic and a delegating serde_json Formatter.
The sink accepts an entire write chunk or returns an error, never partial/zero
success. Before accepting, bound actual complete encoded bytes to1 MiB. Formatter
begin_object/begin_array bounds nested containers to120 BEFORE their children are
serialized; remaining methods retain CompactFormatter's exact encoding. Quoted
strings, escaped quotes/backslashes and UTF-8 are handled by serde_json rather than
reparsed by an independent byte scanner. No output buffer or depth-sized stack is
retained. There is no redundant token-count acceptance claim.

First Bytes/Depth refusal is latched in shared private validation state. Every later
write refuses. After Serialize returns, inspect that state even if custom Serialize
swallowed errors: Bytes produces the existing “mandatory pack/checkpoint exceeds
1 MiB; narrow explicitly” error, Depth a distinct fixed artifact-nesting error.
With no latched refusal, propagate the original Serialize error unchanged. Valid
encoded-length measurement is reusable for existing small HistoryEvent accounting.

Actual Store decode is `serde_json::from_str` (state/mod.rs:1418); installed
serde_json1.0.151 initializes remaining_depth128 and refuses when decrement reaches
zero (de.rs:63,1375–1377): effective127 containers. The artifact cap120 deliberately
leaves at least7 levels for the current persisted Record/ContextVersion envelope.
No recursion limit is disabled or parser behavior changed. Pin the current envelope
in real Store round-trip tests; a future envelope expansion must retain the margin.
120 accepts and121 refuses at artifact validation; both can reach the existing
Store parser in the consistent-corruption negative fixture.

Guard the persisted checkpoint Value BEFORE digest serialization/typed decode.
Guard plain recognized TaskPack/GoalPack data in load_context BEFORE digest and clone/
decode; reject unknown pack formats there without treating them as valid artifacts.
For a recognized Workflow phase ContextVersion, guard ONLY its nested task_pack
artifact before its clone/decode (also validate_capture/context_artifact direct
consumers), not the whole ContextVersion. Keep the post-typed artifact check too:
persisted Value bytes and typed re-encoding can differ, for example omitted null
optional fields; both applicable complete-artifact encodings must fit. Existing
scope/kind/version/digest, shape and semantic checks remain mandatory. The whole
ContextVersion digest necessarily retains its separate existing serialization;
this component makes no allocation-cap claim for that full row/native payload.
No publication, pointer, DB schema, transaction, native operation, migration or
public API authority changes. Input Values/custom Serialize code may already own
memory; this guard bounds emitted validation, not arbitrary user code allocations.

## Actual impact inventory

Source search covers serialized-size measurements and every existing artifact
bounded call, including the child workflow module. Inspect5 direct allocate-to-
measure sites; convert4 (one shared bounded check and3 HistoryEvent measurements).
The11 existing bounded artifact calls inherit the shared check; add the pre-decode
Value guards above. No native or whole-row budget is silently changed.

| Actual site at component baseline | Disposition |
| --- | --- |
| context_pack.rs1568 bounded; callers481/527/631/1107/1352/1402; workflow.rs221/321/361/454/509 | IN: existing1-MiB artifact check; streaming validation |
| context_pack.rs710/1039/1048 recent-history encoded accounting | IN: streaming exact encoded lengths, checked aggregate; no Vec just to count |
| context_pack.rs515/530/614/1393 and context_pack/workflow.rs315/359 loaders | IN: matching artifact Value guard before digest/clone/typed decode |
| adapter/grok/schema.rs24 schema16-KiB check | OUT: separately owned provider schema contract, not context artifacts |
| context_pack/workflow.rs141/157/285/330/462/472 rendered payload and native-input cap | OUT: actual payload bytes are retained for delivery;1-MiB native frame unchanged |
| context_pack.rs1562 digest,1575 projection; context.rs inventory/source encoding; workflow.rs1036 Context input | OUT: required hashes/materialization/general native contexts; broader allocation/source ownership pending their gates |

## Verification and acceptance

Require exact complete encoded-byte boundary and+1 refusal, including a boundary-
crossing multi-byte UTF-8/escape chunk, and120/121-container boundary. Differentially
compare measured bytes/refusal with serde_json::to_vec on fitting actual checkpoint,
TaskPack/GoalPack fixtures and escaped/control/UTF-8 corpus; preserve exact digests.
Exercise a custom serializer that ignores a size/depth error and continues: refusal
stays latched, and a separate early-stop serializer proves later children are not
visited after the first normal propagated error. Exercise a real existing
checkpoint loader with a genuinely published fixture checkpoint, then an explicitly
corrupted oversize/deep stored artifact negative. Recompute its reference digest,
retain all other valid identity/semantic/history pins, and assert the EXACT byte/depth
error; the size/depth-omitted mutant must reach later acceptance rather than fail
on digest/parser/shape. Corruption is a negative fixture, never positive ownership.
Publish/reload a maximum-depth bounded synthetic artifact through the actual Store
codec and loader, with a valid checkpoint producer/control separately retained. All original digest/semantic checks must still run for a fitting artifact.
Existing context artifact integration tests must pass. Compile independent mutants
omitting the byte limit and depth limit, with passing controls reaching the intended
actual-consumer assertions. Controlled fixture Git/provenance is component evidence,
not production #60 containment or native #16 acceptance.

Commit before checks, run fmt/clippy/test/debug/release as appropriate, review the
immutable exact component delta independently, and retain exact evidence. Component
approval does not authorize deploying the other unreviewed branch changes or report
whole Issue19 acceptance. Full schema6/native/legacy/co-integration gates remain open.
