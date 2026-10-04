# Issue 60 inspection/reader component verification

Requirements2 candidate; no design/source approval or implementation. Isolated main054
baseline, no production edits or new process tests. Frozen Issue51 final18c CI remains
red; first bounded native inspection timeout and derived latch failures retained, cause
unknown. No rerun, deadline/latch/permission/Unknown relaxation.

## Requirements1 findings verified

Two independent immutable public-source native reviews at d1dfb5c completed request_changes
with actual cleanup verified. [Disposition](issue-60-requirements1-findings.json) retains
all14 findings and raw result digests. Observation:4Medium+3Low; lifetime:2High+3Medium+2Low.
Verified exact static timeout/check sites, cleanup Ok after relinquished signal authority
is not verified reap, missing owner/admission/Drop-driver contract, undecided reader-kind/
latch/budget precedence and observed Cancelled/Panic taskfuture closure qualification.

Requirements2 fixes those definitions before design:64global job permits cover supervisor/
2readers/cleanup (4perop,max16), no unstarted uncertainty on capacity refusal; actual
anchors outlive caller/runtime, no async blocking Group Drop or unbounded fallback;
Git-specific failed group stays retained, common native Drop remains separately gated.
One existing250ms output/abortjoin budget, frozen primary result ordering and intentional
not_observed→stickyflag/latch are explicit. Late actual resource settlement can release
permits but cannot reset a previously latched Unknown. Observed Panic/Cancelled joins
prove endpoint termination, not reads/death. Fully settled and in-budget abort Context
positives must detect an over-conservative latch mutant.

Every inspector check/guard maps to a finite site/refusal and safe facts; status-error
relinquishment is distinct from cleanup success. First facts survive existing Drop retry;
safe retained Context failure rendering is required. Missing endpoint/setup/read/syscall
errors are injected/unit observations, not claims real ps naturally reached them.
Requirements2 independent delta review remains pending; no finding is assumed resolved.

Requirements-only CI37219380561 passes bothOS on unchanged production; source/reader/
availability acceptance is not inferred. Public actual-checkout provenance will be
recorded independently of trigger head. Apple libproc/sysctl source-only recon establishes
no bounded complete replacement or exact installed-XNU equivalence. Full Issue60 workload/
effect/delegation/settlement and native16 remain pending.
