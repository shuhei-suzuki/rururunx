# Issue #19 curated preparation custody integration

Status: implementation assembled; composition verification and independent
review pending. Whole Issue #19 and native readiness remain open.

## Immutable inputs and scope

Base main: `768f84319cd2a73e14cd39336eb12d99e9be81a7`.
Selected approved source: `857950094ae9732d31d6c11d773e7fe96adbae02`.
Historical source evidence: `a8aa95108bfe2fc098f5333f7bb5cf445c4c74a9`.

Only five code paths are imported: Codex attempt.rs, custody.rs,
custody_tests.rs, mod.rs and session.rs. Every imported code blob is identical
to the selected source. Main attempt/session/mod blobs match the approved
Design5 baseline `3a12ef787e6bcf1cdb5f6732fdc18b2c482b783a` exactly.
Thus the main-to-source code diff is the selected mechanical component.
The broad parent PR #39's other 29 paths, including Context Pack, Context,
Store and schema prototypes, are excluded. Three selected design/evidence
documents are retained; README and master adapter design describe this limited
composition. No source approval of the broad parent is inferred.

Two independent native reviewers approved selected Source2 with no
Critical/High/Medium findings. Their bounded pending hardening limits and
rejected prior rounds remain in the [historical evidence](../design/issue-19-lifetime-component-evidence.md).
The current Create factory is test-only. Its unreproduced actual worker-spawn
error plus pending Note combination and hypothetical future pre-created handler
panic require qualification before future factory/handler activation.

## Regression history and required gates

Exact selected-source CI 37256266809 failed on macOS: seven library failures,
including three independent inspection deadline observations and four subsequent
Context uncertainty refusals; the eighteen selected mechanics passed. Linux
debug passed and release build was cancelled by matrix fail-fast. The retained
last-poll facts do not establish child status or output at the deadline, because
the loop checks its deadline before the next poll/read. Cause and component
contribution remain unknown. Earlier Grok/watchdog/inspection failures remain
recorded, and no same-head retry or timeout/parallelism/latch change is used.

This smaller composition must compile and pass its own focused Debug/Release
controls, workspace regression, fmt, warnings-denied all-target clippy,
Debug/Release builds, relevant compiled mutation controls, two independent
composition reviews and exact final-head macOS/Linux CI before a limited merge.
Any later green run records its own observation and does not explain old failures.

No ordinary native capability, ready backend, native cleanup certificate, private
producer, schema upgrade or full parent acceptance is added. Production custody
selection remains None; the closed fixture creates no native/Git subprocess.
