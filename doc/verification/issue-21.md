# Issue 21 verification

Requirements draft only, baseline main80452f4/schema3. Public Issue21 depends on
merged2/4 and is runnable requirements work within MVP tracking parent17.

Actual AgentAdapter::usage contract is adapter.rs223–228; GenericAdapter838–864
returns scoped nullable metrics and prepared-input byte fields with explicit
missing native telemetry. Store::put_usage828–876 validates finite cost and own
Session scope/agent, then appends usage/audit; Session lookup precedes the write
transaction. Store::usage879–892 returns raw scoped rows without qualified
aggregation or pagination. Usage domain stores nullable fields and arbitrary
cache_metadata. These are existing foundations, not21 acceptance.

Context master sections9/12/15 and Product40.7–40.10 require native cache awareness,
attributable telemetry and representative ON/OFF comparison preserving safety and
quality. Existing external native resumed review counters have unverified per-round
attribution; this draft specifically requires semantics/boundaries before summing.
No native default, credential, permission or hook is changed.

Pending: requirements review, bounded durable/query/normalization design with19
epoch/producers, actual implementation and causal mutation/regression controls,
native quality comparison, exact-head CI and independent source review. No token
savings, billed cost, cache correctness, TUI or final16 dogfood claim.

Req1 two independent native reviews requested changes with V2 actual owned cleanup
verified. Verified High classes: inventory-based missing-operation coverage and all
reduction-provider overhead; operator pricing/currency/category provenance; and
immutable dispatch-derived phase/round attribution (current usage parameters are
collection-time caller labels). Medium clarifications: model/tokenizer and nested
measurement units, scope-bound operator query ingress, exact event replay identity,
retired bounded legacy raw surfaces, genuine artifact sizes, predeclared equivalent
paired workloads/dispersion/quality oracle and explicit acceptance/KPI closure.
Req2 incorporates these contracts with real consumer mutations. The review's
history-free native-state suggestion is narrowed to honest known-channel exposure/
confounds and owned fixture cleanup, preserving native defaults and user history;
no OS sandbox or secret-state copying. New per-run approval/spend requirement is
not adopted: the active user Goal already authorizes native final dogfood. Respect
requested enforceable budgets and finite time bounds, without inventing a dollar
cap from unqualified native counters. Req2 review remains pending; no implementation.

Exact Req1 30a994b CI37193793804 succeeded; documentation-only main804 runtime
coverage, not21 telemetry or native savings acceptance.

Req2 11d5a4f CI37195027084 passed fmt/clippy/tests/debug and release builds on BOTH
Linux and macOS. Two independent native Req2 reviews requested changes, each with
actual V2 selected owned-group cleanup and child reap verified. Their raw resumed
usage/cost counters remain unattributed, never an incremental cost sum.

Verified Medium classes refined in Req3: actual consuming model/effort/tokenizer
observed separately from immutable requested settings; beneficiary links and
descriptive scoped metrics prevent hiding Goal/runtime shared provider overhead;
each credited reduction feature needs an omission-sensitive independent oracle;
mandatory rule/evidence delivery must be proven for every dispatch; controllable
path/session identity reuse and known-channel exposure invalidate causal claims;
and all-unavailable native tokens cannot close native token comparison acceptance.
Actual Issue21 requires estimated cost where available/configurable, so optional
unavailable monetary/cache fields do not become invented mandatory measurements.
Cost/cache savings still require their own complete qualified comparable fields.
These changes match Product40.10 and actual adapter/native provenance gaps; no
source, model/default, permission, hook or credential change. Req3 review pending.

Req3 38c3786 exact CI37196041124 passed all Linux/macOS fmt/clippy/test/debug/release
steps. Two independent native Req3 reviews requested changes with verified actual
owned cleanup. Verified remaining Medium classes: survivor-cohort bias, an untested
feature hidden inside a whole-lane result, exclusive uncached input mislabeled as
logical-context reduction, and a mandatory-set oracle computed by the reduction
loader itself or satisfied by stale delivery. Req4 makes all required repetitions
part of the claim denominator, requires every exercised feature's sensitive oracle,
names inclusive/exclusive token definitions, and fixes reduction-independent policy
and actual per-dispatch sent/retained delivery provenance. Native self-loading is
separate and unverified, never a substitute or an OS internal-attention claim.
Unknown effort/model produces honest non-comparability rather than requiring a
positive causal savings outcome; required token/quality/safety evidence remains.
Req4 review pending; no runtime/source or benchmark acceptance is claimed.
