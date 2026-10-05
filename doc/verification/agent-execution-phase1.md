# Native Agent execution Phase1 report

Status: requirements and design completed with independent static review and
re-review. Stop here for explicit human approval before Phase2 implementation.
No production implementation, native acceptance, or MVP completion is claimed.

## Scope and artifacts

The user's `進めてください` authorized Phase1 after the reviewed Phase0 report.
The selected request protects results while cleanup is best effort; it supersedes
the earlier strong custody proposal. Root AGENTS.md/CLAUDE.md are absent at the
pinned baseline; global user instructions, README development conventions,
master design and issue-4/46/60 rules were applied. Native/schema/public API work
remains STRICT. Requirements were reviewed before design was written.

Source baselines remain main `6146b1639bf03f244d1c5b716304dc1d20146194` and unmerged
Claude `f9b671fc110bcabce66d9d4d9c01e098d69d8076`. The Phase0 source manifests are
unchanged. Phase1 is on `docs/agent-execution-phase1`, isolated from main.

- [Requirements R1–R9](../requirements/agent-execution-requirements.md): reviewed
  at `1eccba7b4ea24566c1299d66b9ef5e8ee379dc74`, unchanged afterward.
- [Detailed design](../design/agent-execution-design.md) and
  [master policy](../design/master/agent-execution.md): final reviewed content at
  `548ff698a66c8552736c7028f1a37d75223e18c8`.
- Seven existing master documents have a short proposed-policy precedence note;
  their historical implementation bodies remain unchanged.
- [Review ledger](agent-execution-phase1-reviews.json): immutable targets, reviewer
  records, findings/dispositions and file blob/SHA256 identities. This report and
  ledger are verification metadata added after the reviewed normative content.

## Confirmed by source inspection and document review

| Decision | Design connection / evidence |
| --- | --- |
| Protect accepted results | Independent retained Git object storage; exact commit/base capture; distinct reviewer/verifier snapshots and hashed evidence; expected-old-target publication. SHA strings alone were insufficient. |
| Stop reuse and old callbacks | Immutable unit/attempt ID, Task generation, historical Session binding, scoped CAS and fresh worktrees/resources. Logical fencing replaces cleanup-only Task reservation, not scope or external-outcome guards. |
| Separate normal completion and cancellation | Native effect rights close at terminal; Runtime finalization remains until capture/draft disposition. Cancellation/replacement closes both; janitor cannot delete capture input first. |
| Resource policy | Durable port/path/Docker leases and managed command profiles for Git, Docker, Gradle, Bazel, sccache, tmux and SSH. Env names alone do not implement labeling or daemon independence. |
| Quota scheduling | Provider/account/bucket pool, independent usage/concurrency accounting, confirmed exhaustion waiting, unknown metadata bounded admission, review headroom and one bounded recovery probe per pool. |
| Best-effort cleanup | Safe owned children/groups, cookie discovery and identity limits, optional Linux user scope, exact Docker labels, quarantined orphan leases and independent cleanup backlog. Work/cleanup axes persist separately. |
| Durable recovery | Schema4 typed tables, intent/ack/reconciliation boundaries, exclusive owner epoch, generation fencing, no SQL transaction across native I/O; legacy unknown effects remain unresolved. |
| Public compatibility | Host official CLIs retain user auth/settings/hooks and own sandbox policy. No new outer sandbox, privileged service or credential copying. Required unsupported resource/input profiles refuse before effects. |

Two independent reviewers (`/root/issue26_registry` and
`/root/agent_execution_phase0_review_b`) each approved requirements with zero
C/H/M/L findings. Initial design review at
`245c64b898fab65abfac3ff6a5630688c0693a5b` identified three distinct issues:

| Verified finding | Correction and final disposition |
| --- | --- |
| High: natural cleanup could retire the authority needed for result publication | Separate native effects/finalization and add destructive-cleanup dependencies and concurrent-janitor control. Closed in both final reviews. |
| Medium: pre/post snapshot hashes miss mutate-consume-restore | Read-only disposable source/Git copies, qualified output profiles, native read-only review capability, source-write negative control and explicit permission/raw bypass limits. Closed on re-review. |
| Medium: pre-open schema3 connections bypass reopen version refusal | Connection-local version function and DB write triggers plus cached/REPLACE/read→write/new-writer controls. Source version checks and writes were inspected to verify the gap. Closed in both final reviews. |

Both independently approved final design at `548ff698...`, each with zero C/H/M/L
findings. Their reviews are static document/source reasoning, not execution of
native provider reviewers or production acceptance tests.

Root checks: relative Markdown links resolve; JSON parses; review targets match
committed blob/SHA256 values; approved requirements bytes remain unchanged; the
seven historical master bodies match after removing the added note; `git diff
--check` passes; diff is documentation only. Source/runtime tests were not run for
these documentation changes. No native Agent/account probe, Docker workload,
scope creation, kill/crash fixture or credential/config inspection was performed.

## Unverified and limited

No actual Claude/Codex launch, four-Task overlap, completion/cancellation, macOS or
Linux conformance, subscription exhaustion/reset/resume, installation, native
auth/settings/hooks, filesystem input protection, schema migration or Docker/
systemd/tool mediation was executed in Phase1. Main's production native readiness
is unchanged. Phase2 must implement and verify the mechanisms; Phase3 must retain
the actual OS/CLI versions and case table, distinguishing fixtures, native runs,
failures, skips, unsupported and unverified rows. README Status stays unchanged
until that evidence exists.

The guarantee is cooperative. Same-user absolute-path/raw-socket/global-resource
bypasses, shared Git destruction, storage exhaustion and undeclared plugins/hooks
are not contained. A scan cannot prove no remaining descendants. Native quota
leases do not guarantee a survivor stopped consuming its subscription.

## Premise qualifications and human decision

1. A discovered cookie and birth timestamp are not an atomic signal handle on
   every OS. Where safe identity cannot be retained, especially discovered macOS
   processes, skip numeric-PID kill and report leftovers/unknown; safe owned-group
   reclamation still runs. No entitlement/privilege/VM is introduced to strengthen it.
2. Commit SHA plus pre/post hashes alone do not prove actual transient test input.
   Supported read-only source/output profiles are required. Source-writing tools
   or required hooks need a compatible explicit derived-output integration or
   refusal; chmod/raw bypass remains outside the cooperative guarantee.
3. Remaining subscription allowance is not established uniformly across CLIs.
   Unknown balances use explicit bounded scheduling; only qualified signals mark
   confirmed exhaustion. Actual Claude/Codex quota compatibility remains Phase3.
4. Runtime SIGKILL tests durability and restart/reconciliation. They cannot promise
   that a dead Runtime continues scheduling or cleaning its Tasks.

These qualifications are part of the proposed supported scope, not completed
behavior. Human approval is needed to begin Phase2 with this scope and the ordered
delivery: native dispatch → captured commits/fresh attempts → resources → quotas
→ best-effort cleanup. Preserve exact permissions, evidence, independent reviews
and external-effect reconciliation; do not reopen ownership proof or extend ps.
No merge, release or README capability claim is included in this phase approval.
