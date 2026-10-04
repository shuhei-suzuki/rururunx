# Issue 46 verification

STRICT shared owned-process death evidence. Requirements/design proposed; no code,
formal review, mutation, default-suite or exact-head CI success yet.

Issue41 exact `888d86d` macOS CI37169413513 reported `native process inspection
 timed out` from bounded Context Git, followed by uncertainty cascades. Linux was
cancelled by failfast after successful tests/debug. Native Claude separately
observed default-parallel Git5s timeouts. Neither establishes causality.

Read-only installed macOS26.6.2 build25G83 own-group fixture used exactly
`/opt/homebrew/bin/python3.14` with waitid/WNOWAIT. Two fresh session leaders each
owned two children. Explicit `ps -g PGID -o pid=,pgid=,stat=` returned exactly the
three expected members, equalled global-table filtering and excluded the second
owned group. An unreaped zombie leader with two live S children remained visible.
Only those owned groups were signaled; cleanup was verified before leader reap.

Five sequential quiet samples: global1431rows median23.121ms; selected3rows
median3.761ms. These are local observations, not CI-load evidence or a production
performance guarantee. Source/result scripts remain temporary fixture evidence;
public structured evidence will be published with validated formal/implementation
rounds. Published Apple blobs are pinned in the design; installed binary/source
identity is unverified. XNU global traversal/temporary allocation remains explicit.

Existing-source safety gap: Apple ps can emit sysctl stderr yet return0, including
initial/exhausted failures. Retry can sleep1s on non-ENOMEM. Current discarded
stderr/empty-as-dead is not sufficient death proof. No shared helper changed yet,
no250ms deadline/latch/serialization relaxation and no historical root-cause claim.

## Requirements review1 and verified refinements

Immutable public `f844d4c` Requirements1 native session
`f5c3cbdd-df13-437f-a863-92ef89f38e2c` completed request_changes; owned cleanup was
verified. R46-1 High is a verified requirement omission: exact-group completeness
must include cross-UID/non-TTY members, which same-UID fixtures cannot establish.
Pinned Apple ps selector under UNIX2003 u03 increments nselectors/xkeep_implied before default-owner
insertion; one-group optimization occurs afterward, then keepit bypasses UID
filtering. XNU PGRP callback has UID/TTY check flags zero. This source-scoped
verification is distinct from installed binary identity or privileged acceptance.

R46-2 Medium is verified: XNU skips SIDL/forking while collecting PIDs under lock,
then fills records after unlock with zombie fallback. The non-atomic fork/exit
window is now an explicit residual limitation. R46-3 Low partial-success KILL
non-claim and R46-4 Low observable error-category precision are also included;
inspection Unknown retains its category and maps to Lost, valid live returns
EPERM, no raw stderr or additional durable permission data. No code or runtime
evidence changed. Requirements2 delta re-review and design gate remain required.

## Requirements2 mode refinement

Public `7c9be4c` Requirements2 completed request_changes with verified cleanup.
R46-5 Medium identifies a missing explicit u03 precondition/source explanation.
Installed compat(5) and verified Apple Libc get_compat.c/h blobs establish
UNIX2003 true when COMMAND_MODE is absent; leading-minus -g does not clear u03.
The earlier fixture script actually used subprocess env={} for both query forms;
the review lacked that source/provenance, so an inherited mode is not attributed
to that observation. Exact argv/environment are now in the public historical
observation artifact, with a script digest. No new privileged or production-helper
acceptance is claimed. Legacy-mode negative acceptance remains required before
implementation acceptance. R46-6 Low removes undefined visible-member wording;
R46-7 Low aligns master/README observational and residual-limit language.
Requirements3 delta re-review precedes design gate/code.

## Requirements3 gate

Independent native Requirements3 at public `d3aa523` completed approve with no
blockers, verified owned cleanup. It verified R46-1..7 refinements from pinned
Libc/ps/XNU source; approval is requirements-only, not design/runtime evidence.
Optional R46-8 exact-env/legacy fixtures and R46-9 actual exit/stderr accounting
are now explicit acceptance conditions. Historical env={} metadata is an executor
attestation, not a committed reproducible fixture; unrecorded exit/stderr stay
null. R46-10 verified pinned sysctl_prochandle needed>copied oldlen returns ENOMEM,
then ps resize/retry/exhaustion surfaces stderr even if exit0. These refinements
are included for scoped recheck in the independent design gate before code.
Raw resumed native token/cost/API counters have unverified per-round attribution.


## Design1 findings verified before code

At immutable public d56f2bf, independent native ownership reviewer24c2b45d-8de7-
43c0-986e-a8bad432b1ea approved with optional refinements; bounded/consumer reviewer
 a92952cc-9d95-4f4f-922d-f106a0ea3a02 requested changes with three Medium blockers.
Both operations completed and owned cleanup was verified before these edits.

BOUND-D46-1 is verified: existing helper joins the reader after timeout and does not
bound EOF after inspector exit. Design2 chooses a single-thread nonblocking drain,
retained-writer causal fixture and explicit mandatory blocking reap outside the
unchanged250ms observation budget. No detached reader/new reader-spawn gate remains.
Pinned Rust1.91.1 pipe.rs confirms macOS pipe+separate CLOEXEC race; this is a residual
inheritance possibility, not a measured cause of existing CI failures.

BOUND-D46-2 is verified mutant misattribution: a removed/global selector commonly
causes rejection and does not prove false death. Design2 separates those liveness
controls from safety-critical -g→-p leader-only zombie/live-child false-death operator.
BOUND-D46-3 is verified: current fail_cleanup skips signal/resolver/inspection and
cannot prove Unknown consumer propagation. Explicit per-invocation macOS test plans
now target real resolver, bounded Git/Context isolated latch and native Grok terminal
consumer; injected PERM scope remains distinct from real native signal proof.

Optional OWN/BOUND refinements specify legacy argument ordering, direct Grok helper
consumption, constructor process_group(0) obligation, two-window Drop retry/blocking
residual, production-shaped fixtures, actual exit/stderr/state spelling and macOS-only
conditional compilation. Apple print.c/keyword.c formatter blobs and Rust pipe blob
were fetched from primary public repositories and pinned; installed/source identity
remains unverified. Requirements4 wording aligns the approved fallible-reader intent with fallible
nonblocking pipe setup and explicitly scopes the observation/reap deadline; no
timeout increase is introduced. The combined requirements/design alignment delta
is included in Design2 before code. No runtime code/tests changed.

Issue41 final63bbd1d exact CI37177404219 preserved Linux success/macOS failure: both
Grok failures reported native process inspection timed out after library, adapter,
CLI, Context and Git tests passed. Earlier ba70c3f/bab6f54 both-platform successes do
not supersede this exact failure. No rerun/dummy evidence commit or root-cause claim.
