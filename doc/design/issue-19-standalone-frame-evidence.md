# Issue19 standalone complete-frame producer correction

This is an existing-contract source correction in the unmerged component5 tree.
It qualifies only `ContextPacks::prepare`, its actual `prepare_task` and nonlaunch
`prepare_draft` consumers, and the four newly added consumer tests. Actual merged
main remains schema3. Whole PR39, proposed6, native ownership/admission/settlement,
legacy migration and19/23/43/60 production integration remain OPEN.

## Contract and actual defect

[Issue19 requirements](../requirements/issue-19-requirements.md) already require
one1MiB absolute COMPLETE rendered native-input cap for standalone and Workflow,
including mandatory metadata/rules. This differs from the existing1MiB compact
artifact cap, proposed8MiB whole ContextVersion bound and16MiB repository source
budget. UTF-8 bytes are estimates, never provider token measurement.

Before the correction, shared `prepare` accepted caller budgets up to16MiB and
checked the final frame only against that budget. Actual Workflow capture already
had an independent1MiB cap; its tests did not cover standalone production.
Detached baseline afb9d8c (original53da61b source, committed synthetic tests)
reproduced both actual consumers accepting oversized complete frames:

| Actual baseline consumer | Observed Ready payload |
| --- | --- |
| Published Task `prepare_task` |1105693 UTF-8 bytes|
| Nonlaunch Draft `prepare_draft` |1105696 UTF-8 bytes|

Both baseline tests exited101 at the intended acceptance assertion, after native
fixture source observation, not at parser/compile failure. The initial sandboxed
attempt failed at native process-inspection permission before reaching the
consumer; it is preserved separately and is NOT a causal baseline result.
The actual successful Task preparation path invokes the existing private
frame-publication transaction before returning Ready. No model was launched.

## Correction and actual controls

Production commit513c937 clamps complete-frame availability to the smaller of
the explicit byte estimate, explicit token estimate and1MiB. The mandatory Task
wrapper consumes this availability before repository selection. Required-byte
addition is checked; mandatory overflow returns an explicit absolute-cap error
before final pack preparation audit or private prepared-frame publication.
Optional source selection uses the remaining budget by implementation; these new
controls qualify zero-remainder omission, not positive optional admission into a
nonzero remainder. Source capture's
16MiB limit and existing small-budget NeedsBudget behavior remain unchanged.

Four actual integration tests exercise both public producers, not an unused
helper. Five owned primary rule files each remain below the existing256KiB file
bound. Exact-boundary fixtures contain escaped tabs/quotes and multibyte UTF-8.
They calibrate from the actual complete rendered payload, not a duplicate wrapper
estimator, then verify:

- Exactly1048576 bytes remains Ready and preserves all six rules, Goal/Task
  constraints and the explicit decision. Task prepared input passes the existing
  exact private-frame validation; Draft remains a nonlaunch source DTO.
- Optional matching source sections cannot displace mandatory facts or raise
  the exact complete-frame cap.
- A1-byte mandatory overflow and the larger reproduced overflow refuse before
  final preparation audit/private frame insert. Oversize controls compare exact
  Task/Context snapshots and COMPLETE prepared-authority/final-audit rows without
  writes, including raw source-version/audit JSON bytes and every persisted column.
  Source observation audits are separate and may still be recorded.

Normal main63 integration was conflict-free; its usage/body diagnostics and
existing Session/environment policies are preserved. Commit205a80d adds the
escaped-UTF-8 controls. Its unchanged default-parallel full checks passed:
fmt, clippy `-D warnings`,335 tests (including2doc tests),24 explicit ignored,
debug and release builds. This does not turn ignored native conformance into
passed acceptance. The runner transcript is an excerpt: its first lib chunk was
tool-output truncated, while every suite completion summary was retained.

## Compiled causal proof

Each mutant was committed before running all four actual consumer tests. All
compiled and exited101 at their intended runtime assertion; no compiler/parser
failure receives credit. The second mutant tests explicit immutable-cap error
classification, not a Ready/native-boundary bypass.

| Mutant | Exact committed head | Actual cause |
| --- | --- | --- |
| Omit whole-frame clamp |56545e8f454401127e88713ead7d97822c8f75bf|oversized complete frame accepted|
| Omit absolute mandatory error |c733dd1d026ed3d612fb2d61215967e66c362213|incorrect larger-budget request instead of immutable-cap refusal|
| Exclusive1MiB boundary |49d5c61b258ec96be1773f17b9823cea062d12b7|fitting mandatory exact-cap frame refused|
| Permit1MiB+1 |a8d88a1402d58c0d60f493138eeb740d43ffcbae|cap+1 mandatory frame accepted|

Historical initial parents205a80d645e14f5a5699aefcc77f00b161b15f56,
7948061d60d35552ea42c6b96a7db3d94bc03fff,
e64722b3458281edfa8f866060c680a170cfe714 and
88f640ce0be6de3be3e76889965fae2a95de35f6 all have crate subtree
701fef967dc21068fea6f23c38d68e1d8b4e2559, equal production205a80d.
Clean restored f9f870695e95bf2718aa4c0be779080e90d96d02 has that same subtree
and all four controls PASS. Private saved manifest/logs:
`/private/tmp/rururunx-issue19-frame-cap-mutations.json` and
`rururunx-issue19-frame-cap-mutant-*.txt`; these are not native authority.

Test-only ae1c14f strengthens refusal evidence from counts to complete persisted
row snapshots. Its fmt/clippy/four focused controls PASS. Production cap bytes are
unchanged from205a80d. All four mutants were then compiled/run AGAIN on the exact
strengthened consumer tree; previous results above remain historical:

| Current mutant | Exact committed head | Actual parent |
| --- | --- | --- |
| Omit frame clamp |fe92a8092b227725d2951b8f8ce91f6771f526e0|8c34da6bc190014d4487c9a444a957fecf638848|
| Omit mandatory error (classification) |0081a88ea9f20213b47547a10bc6f557e62d7290|1420ef7ee04082685b2f800dee54324532f4628d|
| Exclusive boundary |f60cfedf92a3a7b64e27a4380a13c705285d2566|69f2b213b1b30ecbc2c75edcaa4d0ea3e1ec54ce|
| Permit cap+1 |83b1c76ef3d759469973b2006591b173c158ab6a|38b7c545594fbb85fa06df400db69eb1aafa9b84|

Both exclusive/permissive boundary mutants change the `available` CLAMP at
context_pack.rs929 (`min(MAX_BYTES-1)`/`min(MAX_BYTES+1)`), not the mandatory
`required_bytes <= MAX_BYTES` comparison. Equality of that latter comparison
under a smaller caller budget is source-reviewed, not additional compiled
boundary credit from these four controls.

All current parents and clean restored6bc1893dd9697db39f90869e0be4eb6e17cd679a
have crate tree1d0c0b0618780b9dc0ff0d1a076a70605d632926, equal ae1c14f.
Each current mutant compiled and failed at its same intended consumer cause;
restored current tree passes all four controls. Current saved proof:
`/private/tmp/rururunx-issue19-frame-cap-current-mutations.json` and
`rururunx-issue19-frame-cap-current-restored.json`. Full335 at205a80d is distinguished
from these newer focused checks; no unrun full current-head pass is assumed.

## Remaining qualification

Scoped Source1 at cee2d83447319813820f480d3885464e9785cb79 completed with no
Critical/High/Medium/blockers,2 Low evidence clarifications and1 Info. Native
session28e592b4-fa41-49f7-82d2-c0e168a83ac6 exited before inspection. Exact PUBLIC
Git-only zero-operation packet116002bytes/SHA256
2a1d7729d073dbba784ceee7d046f65e0e6311013ff21c2ad02747aa7deb029f.
The reviewer statically established full wrapper/context/rules accounting and cap
before Ready/private publication; execution/Git claims remain author assertions.
The Low clarifications above identify the mutated expression and narrow optional
test credit. Info was verified from actual context.rs184–199:
ContextSlice::prepared_input copies `self.payload` exactly; it adds no payload
bytes. No additional cap or native authorization is inferred from that fact.

Exact cee2d83 CI37239202952: Linux all checks PASS; mac fmt/clippy/lib185/adapter19/
CLI5/context16 PASS, then CPP10PASS/34FAIL. First two actual native process-inspection
deadline facts measured288241/288728us, followed by conservative cleanup-uncertain
latch failures; all four new cap tests fail at earlier source setup, not their cap
assertions. Whole cause/regression attribution remains UNKNOWN. Earlier53da61b
CI37236662337 separately failed context7 after327–331ms observations. Both actual
default-parallel failures are retained; no timeout/latch/parallelism relaxation or
historical green substitution. Current code CI readiness is NOT achieved.

Private preserved parsed review/failed CI evidence:
`/private/tmp/rururunx-issue19-frame-cap-source-native-review1-parsed.txt` and
`/private/tmp/rururunx-issue19-ci-37239202952-failed.txt`.

This correction creates no managed native owner, receipt, operation allocation,
phase closure or proof of all-descendant cleanup. Existing component5 frame
authority is a mechanical preparation contract, not proposed6 NativeCAS.
Native Git observation still needs the actual60 supervised source/job producer;
6/F1 cohort containment,14 recovery and accepted23/43 ports remain required.
