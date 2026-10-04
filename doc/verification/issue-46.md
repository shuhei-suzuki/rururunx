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
