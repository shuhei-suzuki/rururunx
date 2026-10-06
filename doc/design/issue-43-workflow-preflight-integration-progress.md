# Issue43: early Workflow Native refusal integration

This implementation checkpoint follows managed binding requirements MB3/MB6 and
managed binding design §2.1. It integrates the PR52 `b191b46` refusal contract into
composed `8a1341b`; it does not deliver a record-only binder or Native availability.

Fresh native phases select the registered adapter before `Sources.inputs`, because
that current consumer may admit registered Git helpers. Missing actor/adapter
refuses using internal Task/registry metadata only. Any registered descriptor then
receives ManagedBindingUnavailable before invoking a public adapter callback: no
actual installed private composition issuer exists. Execute/Review and
PreparedInputAdmission capability, probe identity and probe-error diagnostics stay
after that check and remain unreachable. Public `capabilities` and `probe` methods
have no enforced effect-free seal. Refusal precedes source capture, fresh Context,
hold clearance, reservation, Unit preparation, marker, Session and audit writes.
Ordinary non-native evidence phases retain their separate consumers.

The selected immutable adapter/alias/provider is carried to preparation rather than
looked up again. Preparation and managed-start reentry independently keep the hard
private-composition refusal. An existing pre-Session due waiter is refused before
claiming/clearing its record or preparing helpers. Already-dispatched nongrant
status, stop and terminal observations retain their existing separate contracts.

The existing FakeAgent positive fixtures do not advertise PreparedInputAdmission.
New direct preflight controls use descriptor variants with observable callback
counters and assert zero capability/probe calls for both roles and every native
phase. They create no Driver, allocated phase owner, Session, completed evidence or
positive Native admission. They do not qualify the still-missing actual
accepted-Goal Driver/marker/binder integration.

The original `ec77e6e` candidate called public `probe` before static composition
refusal. Closed independent findings ROOT-M1 / WF-PF-B-M1 identified that ordering
violation; its original tests and RED quality/regression evidence are retained.
The correction moves the composition check before all public adapter callbacks.
Actual fresh-step/due-wait zero-effect controls require genuine Workflow/Driver
preparation, which this source still cannot produce. No SQL owner seed, fabricated
Workflow waiter or positive Fake capability is used to reach those consumers;
their wiring is inspected statically and their genuine controls remain open.

The hard guard must only be replaced together with the actual implementation-owned
selected private composition and genuine owner/input/sole binder consumers. A
capability flag, configured provider, SQL row or returned Session cannot replace
it, and this checkpoint cannot open the legacy Task/version-writing binder.

Fixed source, independent source review, quality/regression outcomes and genuine
producer/both-OS/authenticated qualification are reported separately. Earlier RED
and setup failures remain unchanged; this change makes no execution/MVP claim and
changes no README, license, process tracking or containment mechanism.
