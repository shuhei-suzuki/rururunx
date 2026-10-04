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

Keep the existing private `bounded(&impl Serialize)` consumer. Serialize through
`serde_json::to_writer` into a private validating/counting `Write` sink, retaining
only scalar counters and string/escape state, with checked arithmetic. Before
accepting a chunk, bound total actual UTF-8 encoded bytes to1 MiB. Count nesting
from emitted object/array delimiters outside quoted strings; refuse a129th nested
container before its children can be recursively serialized. The128-container cap
is a serialization guard, not the stricter proposed native Session depth32 bound.
Quoted keys/string contents, escaped quotes/backslashes and UTF-8 do not increase
container depth. Count emitted JSON tokens, including object keys, with the finite
ceiling of1 MiB tokens (already implied by the encoded-byte ceiling). The sink
allocates no JSON buffer or depth-sized stack. Serialize errors still propagate.

Apply this guard at every existing `bounded` artifact call. For `load_checkpoint`,
also guard the complete persisted checkpoint Value before its digest serialization
or typed deserialization. Existing exact scope/kind/version/digest and semantic
checks remain mandatory. No publication, pointer, DB schema, transaction, native
operation, migration or public API authority changes. This does not bound memory
already owned by an input Value or arbitrary custom Serialize implementation.

## Verification and acceptance

Require exact complete encoded-byte boundary and+1 refusal, escaped quote/backslash
and multibyte UTF-8 cases,128/129-container boundary, and a custom serializer proving
validation stops before later children are visited. Exercise a real existing
checkpoint loader with a genuinely published fixture checkpoint, then an explicitly
corrupted oversize/deep stored artifact negative; corruption cannot create positive
authority. All original digest/semantic checks must still run for a fitting artifact.
Existing context artifact integration tests must pass. Compile independent mutants
omitting the byte limit and depth limit, with passing controls reaching the intended
actual-consumer assertions. Controlled fixture Git/provenance is component evidence,
not production #60 containment or native #16 acceptance.

Commit before checks, run fmt/clippy/test/debug/release as appropriate, review the
immutable exact component delta independently, and retain exact evidence. Component
approval does not authorize deploying the other unreviewed branch changes or report
whole Issue19 acceptance. Full schema6/native/legacy/co-integration gates remain open.
