# Bounded strict JSON decoder design

Risk: STRICT. Status: pure component implemented at `bff8b27` after independent design review;
actual native/review consumer integration remains pending. Implements
[J1–J5](../requirements/strict-json-decoder-requirements.md) as a shared pure
content decoder. Source baseline `414bc2b`. No Store schema migration, third-party
dependency, public grant or native input authority is added.

## 1. Module and API

Add `execution::strict_json`, using existing serde/serde_json. Make its decoding
API public within the public module so downstream native/review components can
use it without dead-code exemptions or treating a Result as authority. It accepts
raw `&[u8]` plus an explicit `Limits` profile, and returns `Result<Value, Error>`.
Limits and Error are content DTOs; no deserialize-derived private proof exists.
Public exposure of a parsing utility cannot mutate execution/persistence.

Compiled ceilings: original bytes4MiB; container depth32; total nodes65,536;
one decoded string/key1MiB; aggregate decoded string/key bytes4MiB;
object entries4096; array entries4096. Each profile field must be positive and
at most its ceiling. Native frames may request4MiB; Review envelope profiles use
at most1MiB and still need semantic entry/answer bounds. No consumer is silently
assigned the maximum profile. The original frame cap precedes parser construction.

## 2. Raw duplicate detection and allocation

Use a DeserializeSeed sharing a private mutable Budget, with recursive seeds
carrying container depth. Start at depth0; entering an object/array consumes one
container level. A scalar root has depth0; an object root has depth1. Increment
nodes once for each value, including a container itself; object key names consume
decoded byte budgets but do not count as value nodes. Keys and values decode as
ordinary JSON strings; count UTF-8 bytes, not characters or source escape length.

The visitor constructs a normal serde_json::Value. Check a decoded object key
against the current object's map BEFORE inserting its value; duplicate keys are
rejected rather than replaced. Check per-object/per-array counts before accepting
the next element. Enforce string and cumulative budgets before retaining their
decoded copies. Frame bounds cap transient serde string/key allocations even
when the decoded limit is smaller. Never claim a streaming parser allocates zero
bytes for an over-limit token: the complete input slice already belongs to the
caller, and serde may decode one bounded token before the visitor rejects it.

Numeric parsing uses serde_json's normal finite number representation; NaN,
Infinity, invalid integer forms and unsupported numeric overflow reject. Parsing
ends with Deserializer::end(), preventing a successful prefix from hiding a
second value. No generic map reserialization proves original duplicate absence;
the raw seed must run first.

## 3. Errors and consumer integration

The budget records the first finite limit/duplicate category before returning a
serde custom error. The API discards serde's error string and returns only the
recorded category or InvalidJson. Categories: InvalidLimits, FrameBytes,
InvalidJson, Depth, Nodes, StringBytes, TotalStringBytes, ObjectEntries,
ArrayEntries, DuplicateKey. Display/Debug contain only those categories.
No source content enters Error; raw parser diagnostics are not logged.

B's native acquisition implementation will explicitly invoke the utility before
Value conversion where candidate/structured content can gain interpretation.
Later ReviewEngine validates its complete answer DTO and expected frozen bindings
after this stage. Both still require their actual owned raw producer controls;
adding this module does not change existing NativeStatus or Workflow gates.
Failure/acquisition/ownership/read-only/certification remain separate predicates.

## 4. Controls and impact

Only a new module, its module declaration, tests and these documents change.
Existing native protocols, SQL schema5, license files, unsafe forbid, credentials,
settings/hooks, ps observations and execution policy remain untouched.

| Control | Required observation |
| --- | --- |
| Nested valid values and all scalar forms | Exact Value returned, no content interpretation. |
| Root/nested/array-element and escaped duplicate keys | DuplicateKey, including otherwise valid APPROVE text. |
| Every inclusive budget boundary and one excess | Boundary returns Value; excess returns exact finite category. |
| Depth32 profile versus33 containers | Exact boundary accepted; excess rejected before descent. |
| UTF-8 and Unicode escapes | Decoded byte budgets and escaped-key equality are enforced. |
| Missing/invalid number/extra value/Markdown | InvalidJson, no substring recovery. |
| Secret-shaped duplicate key/malformed value | Error text contains only category, no payload. |
| Zero or above-ceiling Limits | InvalidLimits before JSON allocation. |
| Compiled duplicate/depth guard omissions | Actual consumer assertions fail; restored controls pass. |

Independent design/source approval is limited to this content component. Both-OS
CI, real native answer compatibility, full schema validation, Review votes and
operational Runtime are explicitly outside its acceptance evidence.

## Component evidence

The [source checkpoint](../verification/agent-execution-phase2-strict-json-checkpoint.json)
pins actual controls, full bounded regression, compiled guard omissions/restoration
and independent static reviews. Consumer integration and all owned-result/approval
semantics remain separately required; this is not native or full Phase2 acceptance.
