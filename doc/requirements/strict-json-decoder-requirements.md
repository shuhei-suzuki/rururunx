# Bounded strict JSON decoder requirements

Risk: STRICT (shared native-result/review decoding). Status: pre-implementation
component proposal. This supplies content validation only, not execution,
result ownership, reviewer identity or approval authority. Requirements/design
and independent review precede source. No additional human boundary within Phase2.

## 1. Purpose and sources

Implement the raw duplicate-key and bounded-decoding contract required by
[ReviewEngine R3](review-engine-integration-requirements.md) and the
[native result supplement](../design/native-result-receipts-design.md).
Source baseline `414bc2b` uses serde_json::Value in provider paths; conversion to
Value can lose original duplicate keys. The new decoder must inspect the original
bytes before consumers select an answer or validate a typed envelope.

## 2. Content contract

J1. Accept exactly one valid UTF-8 JSON value, with whitespace permitted around
it. Reject trailing values/garbage, invalid numbers and duplicate keys in EVERY
object. Key equality uses decoded strings, including equivalent escaped spellings.
Do not repair fenced Markdown, extract a JSON substring or choose a last answer.

J2. Bound original bytes before allocation, container depth, total value nodes,
individual decoded strings/keys, aggregate decoded string bytes and per-container
array/object entries. Reject a value at the first exceeded bound; do not drain an
unbounded suffix. Exact maxima are inclusive. A caller may select a smaller
profile; no profile may exceed finite compiled ceilings. Every field is positive.
All counters use checked arithmetic. Limits compose: individual maxima do not
promise all simultaneous maxima fit the complete frame or aggregate budget.

J3. Return a parsed Value or a finite error category without raw text, key names,
model content, transport payloads or serde's input-derived diagnostic. Callers may
record category and bounded byte count; authentication/error streams must never
be passed as model answer content. The decoder itself performs no IO, logging,
side effects, persistent writes or native process operations.

J4. A parsed Value is untrusted content. Consumers MUST subsequently validate
their deny-unknown-field schema, required identities/digests/locations/evidence,
their original owned invocation/receipt and semantic bounds. Parsing APPROVE
does not count an opinion, finish work, mint a grant or publish any result.
Native failure with valid content remains failure. Current consumers remain
unchanged until their separately qualified integration invokes this decoder on
the actual raw wire/answer bytes; tests of this helper alone do not close R3.

## 3. Required verification

J5. Exercise complete positive nested scalar/object/array inputs; repeated keys
at root, inside array elements and deeply nested objects; escaped-key equivalence;
malformed UTF-8/JSON/numbers; extra values/Markdown; exact and exceeded limits for
every budget; zero/out-of-ceiling profiles; decoded multibyte/escaped strings;
and noncontent error rendering. No generic successful parse is an approval test.

Compile causal omissions of duplicate detection and depth enforcement, run their
actual assertions, restore exact baseline bytes and rerun controls. Compilation
failure/setup refusal does not count as a killed mutant. Fixed clean source must
pass applicable tests, fmt/Clippy/build and independent source review. Linux,
authenticated native final-answer/profile behavior, resource accounting and full
ReviewEngine/Runtime remain separate qualifications.
