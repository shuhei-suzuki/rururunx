# Issue 24: Recorded Goal facts view — WHAT

Status: proposed; implementation requires independent design approval.

## Purpose and acceptance

An existing scoped Goal read displays recorded completion criteria and the accepted DAG with exact Goal/Task identities and versions, honest incomplete/unknown facts and finite encoded budgets, without writes or new authority.

This increment lets an operator inspect the criteria, dependencies and recorded states of explicitly accepted work using `rrx goal status <goal-id>` and `rrx goal tasks <goal-id>`. It advances the visibility part of Issue 24. It does not satisfy the full objective's automatic continuation, parallel execution, recovery, native attach, event-driven updates or full CLI/TUI acceptance.

## Required behavior

- Status identifies the selected Project and current Goal/version, distinguishes accepted work from an inert proposal, and exposes the recorded criterion ID, description, evaluator declaration, satisfied flag and optional evidence reference. A stored flag/reference is not verified evaluator completion. The verified result remains unavailable until its genuine producer exists.
- Task pages expose each returned Task's exact identity/version, stored state and existing effective display state, plus its incoming hard/soft dependencies and the referenced Tasks' exact versions/stored states from that same validated read. A node without hard edges is structurally unconstrained; a node with hard edges requires prerequisite evidence. Neither classification grants runnable capacity or proves prerequisite verification.
- All lists are narrow allowlisted projections. Over-budget facts are explicitly unavailable with their recorded counts; they are never silently truncated, replaced with an empty complete list or reported as false evaluation. The HOW defines exact encoded budgets and overflow behavior. The default recorded status must preserve readability of every valid proposal accepted by the existing writer, including its full objective; its new budget must not introduce a refusal regression.
- Existing Task ordering, maximum and scope-checked `after`/`next` semantics remain. Each response is an independent observation. Neither a final page nor equal Goal versions across pages establishes coherent combined completeness.
- Plain output labels recorded, unknown and unavailable facts clearly; JSON exposes the same distinctions. Evidence is opaque recorded text, never opened or interpreted. Every Unicode control character, including DEL and C1, is escaped in every user-controlled plain-output string, including the proposal objective. JSON preserves exact stored string values after decoding through the existing serde_json encoding.
- The existing strict wire protocol receives an explicit opt-in extension. Omitted view selection preserves legacy request/response shape. Unknown fields, duplicate keys and unknown view values remain refused; there is no protocol-pin relaxation or silent downgrade to fabricated facts.

## Preserved boundaries

The service retains actual same-UID Human ingress, observed Runtime instance/epoch and current-owner checks. Accepted reads retain the current exact selected Project ID/Goal body-index/version/definition checks, complete scoped Task inventory checks and DAG validation. Foreign Goal/Project combinations and foreign or disappeared Task cursors refuse.

Project routing is a separate observation at the current source baseline: the accepted Goal reader verifies Project membership but does not reread the Project body/version. This view must not invent a same-transaction Project version or claim a combined routing/Goal snapshot.

Successful and refused reads leave all durable rows, versions, owner/epoch, acknowledgements, audit and attention unchanged. The client neither opens a Store/owner nor starts a service or reconciles Projects. Rendering never drives evaluation, scheduling, native permission or lifecycle transitions.

Goal body, transport frame, Task page and scoped inventory limits protect different stages. This increment does not widen them, migrate schema or turn facts, hashes or cursors into authority.

## Future implementation acceptance controls

1. Extend the actual accepted compiled-service IPC fixture with hard, soft and disconnected nodes. Assert exact criterion fields, DAG endpoint identities, Goal/Task versions and stored/effective state distinctions through both CLI formats.
2. Preserve inert proposal status and refusal of a Task page for nonaccepted work. Assert unavailable evaluator completion and runnable admission, including a recorded `satisfied=true` observation without claiming it is verified.
3. Preserve foreign Project/Goal, foreign/disappeared cursor, stale Runtime identity/epoch and malformed wire refusals. Inspect exact output allowlists and complete unchanged durable row images for successful and refused reads under a stable idle baseline.
4. Exercise encoded boundaries, escaped text, large criteria/relationships, page-size reduction and independent changing pages. Include a genuine compiled 16 KiB U+0001 proposal readable through both legacy and recorded consumers in plain and JSON with the exact objective after decoding, and a genuine accepted DEL/C1 criterion fixture with escaped plain text and exact decoded JSON. Assert explicit overflow/unknown facts, bounded complete frames and no combined-complete claim. Strict-profile overflow is a defensive unit control; it is unreachable through these bounded reader projections and must not be credited as end-to-end coverage.
5. Run relevant existing compiled Goal CLI, Runtime reader/ingress and strict transport regressions. Demonstrate causal compiled omissions at the intended assertions; setup or compilation failure is not a mutant kill.

These are acceptance obligations for future source work, not executed qualification in this document-only increment. No native provider is needed for this reader's controls. Genuine evaluator completion, prerequisite evidence and native/full-MVP controls remain separate requirements.

## Sources and exclusions

Source baseline: `c9791175d153b4801c2fd42cc0e557b84a8e477e`. Inputs: the full objective attachment (sections 7 and 32), Root-observed Issue 24 requirements at `2026-10-06T16:49:00Z`, [product requirements](product-requirements.md#6-goal-mode), [Goal Runtime master](../design/master/goal-runtime.md), [Runtime CLI HOW](../design/runtime-cli-integration-design.md), and Root-reviewed `current-cli-mvp-consumer-recon.md` evidence. The [HOW](../design/issue-24-goal-facts-view-design.md) fixes this increment's projection and compatibility details.

No public hierarchy, TUI, subscription, new control writer, schema, native activation, review, quota, cleanup, SourceStop/Cancel or R2 change is included. Native execution remains the separate primary lane; unavailable dispatch and all full-MVP gates remain honest.
