# Issue 43 verification

Requirements gate: native independent review4 approved public
`d8c5266625e11fa565dd8b86a265ec24f8fdcc9c` with no findings or unresolved blockers.
Reviewer session `05c61b58-9864-4961-882a-d130d4940d7d`; owned wrapper cleanup
verified before editing the worktree. Defaults preserved; tools disabled and
empty strict MCP configuration. Only GitHub-public byte-verified requirements,
delta and primary source excerpts were supplied, without private configuration
or executor transcript. Resumed usage/cost/API gauges have unverified cumulative
attribution and are not reported as the incremental review's metrics.

Earlier verified blockers R43-01, R43-03, R43-04 and R43-05 were corrected in
requirements. R43-02's premise was withdrawn after actual StateOnly source was
supplied. The optional missing-Project-fence hypothesis was also withdrawn:
Codex/Grok capture compares the complete Project before subsequent scope checks.

The optional DENY precision is verified in actual Claude f9b671f: scope checks
and commit_current are ALLOW-only; DENY uses commit_session_only. Exact final
error mapping, provider-specific native controls and any future resumed-UUID
lineage remain explicit design/integration checks, not implementation evidence.

Design1 at public12f545f received two independent request_changes reviews:
18d34d9a-7662-4add-9caa-f7f0853f71cc (ownership) and
03638622-3385-4910-9dc0-f82e72369a70 (currency). Both completed and owned cleanup
was verified before edits; neither saw the other's current-round findings.
Root verified four Medium classes against the supplied code: undefined optional
private-owner selector, ordinary-transition binding bypass, latest durable UUID
omitted after a None return, and sibling/non-admission controls misclassified.
Design2 corrects those classes, makes #19 proof mandatory for every fresh native
binding, and specifies co-integration without a completed-merge dependency cycle.
Additional primary inspection found the new audit name needs the existing public
reserved-kind guard; its causal test/mutant is now included. Audit kind/payload,
event consumers, marker tuple, writer inventory and probe impact are explicit.

Design2 is proposed. No production code, combined native matrix, mutation,
workspace verification, typed ownership integration, CI or merge is claimed.
