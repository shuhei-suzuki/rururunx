# Issue 18 requirements: repository map and budgeted retrieval

Implement a local, provider-independent context index for a bound Task. A caller
supplies exact Project/Goal/Task scope; the index reads that Task's independent
Git worktree and the Project's registered primary rule/config references.

- Index source-relative paths, lexical definitions/signatures, imports, identifier
  references, and approximate file dependency edges without a supervisor model.
- Rank using task text, changed files/symbols, current dirty/untracked paths, and
  graph neighbors. Return deterministic slices with selection reasons and omitted
  references under positive, configurable byte and estimated-token budgets.
- Include Goal objective/criteria/constraints, Task acceptance criteria, all
  registered Project rules, and caller-declared evidence in full. Insufficient
  budget returns `NeedsBudget` without a launchable payload. Explicit expansions
  also require their complete requested file content to fit.
- Support file, symbol, caller/reference and callee/dependency expansions. These
  are lexical approximations, not compiler semantic claims.
- Bind freshness to scope, state versions, exact worktree, HEAD, inventory and
  SHA256 content hashes. Dirty files, missing/restored sources, added/deleted
  files, ignored admitted evidence, primary rules and config changes invalidate
  reuse. HEAD equality alone is insufficient.
- Reject foreign/ambiguous scope, unbound tasks, blocked/removed Projects, altered
  worktree ownership/branch, symlinks, nested repositories, Git metadata,
  namespace paths, and special files. Source reading must not follow symlinks
  between validation and open.
- Git/process/filesystem work happens outside `SharedStore`; take a state snapshot
  and recheck current versions at the observable mutation commit.
- Bound inputs, scan size, graph/index output, and Git time/output. Reject excess
  rather than silently lose mandatory context or fabricate completeness.
- Audit map generation, selection and expansion with scope, revision/hash,
  reasons, omitted refs, budget, byte estimates and nullable provider measurement.
  Do not store source payloads in audit events.

This Issue exposes a Rust library and a local inspection example. Durable Context
Pack assembly, progressive conditional rule selection, checkpoint condensation,
workflow integration and provider usage accounting remain Issues 19/20/8/22.
No native inference or provider token/cost measurement is implemented here.
