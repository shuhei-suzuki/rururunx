# Issue19 selected lifetime source evidence

Status: selected mechanical source candidate; Source1 returned a verified Medium;
its actual corrections await independent Source2 gates. Whole Issue19 remains OPEN.
Two native selected Design5 reviewers approved public
`3a12ef787e6bcf1cdb5f6732fdc18b2c482b783a` with no Critical/High/Medium findings
(sessions `3eefede9-bb34-43f9-8041-df0b616c72ec` and
`9f6946c2-1d60-4255-a777-fd10c3e9acf0`). This gate covers the private
Control/file/thread mechanics described in the adjacent component design.
It does not qualify the unmerged CPP/schema5 prototype, candidate schema6,
whole PR39, native admission, managed operation or settlement receipts.
Production native availability remains EMPTY. Parent Design37's rejected broad
findings remain OPEN in `issue-19-open-design-findings.md`.

## Actual source and consumer

The existing Codex `Control::spawn`, `CallerGuard`, `TaskGuard`, adapter
registration and context-error paths now use a private retained custody pool
when the closed fixture factory is selected. Ordinary unavailable native callers
keep their existing refusal before installation. The same pool/queue/cancellation
code retains actual file workers after caller Drop or adapter Err, observes
actual actor/worker/custodian joins, and reconciles only exact original registry
dispositions. It introduces no public factory/capability, schema, Session lease,
managed marker or cleanup certificate. File/thread fixtures execute no native
subprocess through that consumer; their initial isolated repository setup is
separate from the resource mechanics.

Each closed fixture reserves three declared frames before creation, against the
same 64-job pool algorithm. Tests inject separate pools to retain default parallel
execution. Twenty-one live three-frame owners charge63; the next invocation
refuses before its worker factory. Genuine joins replenish the pool and the actual
registration sweep handles40 sequential owners on one adapter. Opaque construction
failure or unknown join retains its unproved slots. Requests use one non-Clone
generation-bound endpoint, a64-entry inbox and a4096-byte streaming JSON cap.
These are encoded metadata/job-count limits, not a physical heap/RSS bound or
proof of native process descendants.

## Preserved failures and corrections

At `c9837504627e3e30b943cbd54f1bcce3ad89d666`, the first mutation restore ran
10 passing controls and1 failure: actor panic let the custodian return on global
Unknown while an installed worker was still pending. The worker handle remained
retained, but healthy late join observation ended and two slots stayed held.
`543b7cd07a59050f0fc5e55d8f3697c239530319` keeps that same observer while the
worker handle is pending. An actual held-worker control observes three slots
before release, then the genuinely joined worker/custodian release two while the
unknown actor slot remains held. It does not clear Unknown or mint completion.

The committed reverse-order control at
`6f49f1d7661d164478e9d68140f7587ca42ecbbf` failed after actual worker cleanup:
FreshUnpublished was factual, but bookkeeping-only actor/custodian joins caused
the registry to retain an entry forever. `939354572e3be8b0a11ebe5973a4d6553317ae97`
distinguishes outstanding resources/created Unknown from zero-effect bookkeeping,
as the approved design requires. Actual worker join before the context error now
preserves the original no-work removal. Created resources and Unknown remain held.

Selected controls at9393545 passed15/15 under default test parallelism,
including consumed/Failing preparation-latch revocation with no native stop,
closed/missing runtime accounting, pre-/post-creation caller Drop, full revoked
queue independent join cleanup, endpoint EOF, panic-before-Begin, actual previous
Control restoration, opaque construction Err, capacity and both ordering cases.
The consumed latch fixture manufactures no Session/native admission or wire proof.

The earlier current-head full workspace run atc983 remains RED:
301 library tests passed,7 existing Grok tests failed and23 were ignored.
Four reported native Git ownership-preflight timeouts before intended dispatch/
policy assertions; another asserted a non-dispatched cleanup receipt where it
expected Lost, and two isolated child failures did not provide a terminal cause.
The whole cause and whether the new component contributes remain UNKNOWN.
That result is not erased by focused passes, a deadline change, serial tests or
a same-head retry. Lint, debug and release build passed atc983; lint and14 focused
controls passed at543b. Later exact-head regression and independent source gates
must be recorded separately.

The next exact code run at
`94f7f744fa266e0e9f41c29131bc5650ebfa5d13` completed normally:
468 workspace tests plus2 doctests passed,26 existing cases were ignored, and
fmt/clippy with warnings denied passed. All15 mechanical consumers and44
CPP regression tests passed. This head normally composed the narrow diagnostic
dependency `3d6ac5fb40f50239f0d0c69795f9c11247d263e1`; its helper adds only
test-side observation labels and does not fix or explain the earlier failure.
After owned checks closed, normal main768f843 composition produced
`181b4de6475cf86e23564f95eeacf39d871f3ba6` with the IDENTICAL whole tree
`9ef6d3a2cc6478477a0867f5835f5db9b0318f41`. Debug and release workspace
builds passed there. Code tree `f5c09c564cc33d12e71abc64f795928051dfb9b3`
is exact for both executions; selected Codex tree
`ae44cc25c71366e0edea5b2a0c2a4b9f3db06687` is unchanged from the mutation
baseline9393545. Historical7Grok failures and the diagnostic dependency's
earlier Codex inspection/watchdog failures retain UNKNOWN cause. Neither the
new green run nor the mechanics source grants broad CPP/native/MVP acceptance.

## Compiled boundary controls

Mutation evidence is scoped to the actual assertions, not a native grant claim.
The old registry-sweep mutations failed to compile because their explicit type
was wrong; they earn no mutation credit. The corrected selector keeps the actual
SessionId type inference. The source mutant replaces one relevant production
predicate/constant at a time in an owned detached temporary worktree, commits it
before the test, and restores the exact baseline for the positive controls.

At9393545, all13 final mutants compiled and were killed by their selected actual
consumer assertions. The restored exact source passed15/15 (no serial/deadline
change). The complete per-mutant head/test/exit/log manifest is retained at
`/private/tmp/rururunx-issue19-lifetime-source4-mutations.json`; these commits were
created only in the owned detached mutation worktree, then that worktree returned
clean to9393545. Earlier failure manifests were preserved, not overwritten.

| Omitted/changed boundary | Actual causal assertion |
| --- | --- |
| Armed caller job revocation | Postcreation Drop leaves endpoint revoked |
| Endpoint generation admission | Corrupted generation admission refuses |
| Handler revocation recheck | Queued revoked notes never change the owner's count |
|4096-byte encoded metadata cap | Exact4096/+1 and escaped-byte controls |
|64-entry inbox cap |65th admitted request refuses |
| Observed creator NotCreated credit | Never-created worker slot replenishes after actual joins |
| Original registry sweep |40 sequential owners remain available on one adapter |
| Unknown worker accounting | Unknown worker retains its unproved slot |
| Worker Begin wait | Callback panic before Begin creates no file |
| Held registration retention | Actual resource-owning Err keeps transition/entry |
| Late worker observer | Actor Unknown keeps pending worker observed until join |
| Opaque constructor Err reservation | Complete unproved declaration stays charged |
| Zero-effect bookkeeping distinction | Worker join before Err preserves original removal |

Caller revocation and handler revocation are separate controls;
foreign-generation admission refusal is an API boundary control because the
handler also independently rejects a foreign generation. Unknown-slot refunds
and NotCreated credit controls qualify accounting, not native release authority.
Opaque construction Err tests the conservative shared result branch through a
private injected error; it does not prove any OS spawn error has no effects.

Whole Issue19/#43/#23/#60/#14 co-integration and actual supported native backend
readiness remain required. This selected component cannot satisfy those gates.

## Source1 findings and actual corrections

Both independent Source1 reviewers examined public immutable
`f2162926b4d4a22a2c3e897a5e6483fb5f2f0a0e` and returned REQUEST_CHANGES:
one verified Medium each, no Critical/High. Their native sessions were
`b9036bb7-c5cb-4398-b118-767821c001d5` and
`b8d9f67f-9ca3-4b5c-b20d-b906196fefc1`. Both operations finished before
either result was inspected. The identical public-only packet was458148 bytes,
SHA256 `8fb53088a1f8ab510ae997e7df51b83338f1d539f22ebc8c21fc1913de69f2ee`.
Exact f216 CI37253422993 passed all steps on Linux/macOS; that historical green
did not discharge the reviewers' actual counter-ordering defect.

The custodian sent a rejected Create reply BEFORE retiring that request's
accepted/inflight counts. The actual actor could observe outstanding bookkeeping,
freeze CustodyHeld with created=false, then retain the registry entry forever:
the unchanged original-disposition gate correctly refuses uncreated workers.
A deterministic real enqueue/revoke/handler-retirement barrier at
`2039b07084674c6fe286c7772e97c20b8616cf47` reproduced0PASS/1FAIL.
`3fa536720b677ea2913a52ad7383ecb4cef0dc44` retires the request before replying.

A second-handler control at `28edada3a25387f76de9f7faa1b14c7cd5e0427f`
then reproduced0PASS/1FAIL with a fixed Note queued AFTER the rejected Create.
Even correctly ordered Create retirement left that later Note outstanding when
the actor reported its genuine pre-work error. `6e300f3a6229193254e9900caf28e30af28f9bef` distinguishes pending
effect-capable requests/actual effects from fixed metadata. Only an exact normal
Finished FreshUnpublished/RestoredBeforeAdmission with no created/unknown/pending
effect can preserve ordinary remove/restore while metadata drains. Preparing,
Lost, real created resources and Unknown are not exempted. No new native authority
or permission exists; OriginalDisposition still requires CREATED, all three
actual joins, no Unknown/outstanding and no published Lost.

The actual second-handler and checkpoint consumer holds the custodian while the
registry removes a fresh no-work entry or restores its exact previous Control.
All THREE job slots remain charged and joined=false throughout that interval;
only actual subsequent actor/custodian joins plus observed worker NotCreated
refund them. No Session/input admission or availability effect occurs. This
separates registry bookkeeping from resource/job release.

The two Source1 Low observations remain explicit pending-activation limitations:
(a) successful custody installation followed by actor-install failure can retain
a polling custodian; the actual fresh Control is Vacant with an unpoisoned private
mutex and is spawned exactly once, while closed runtime installs a genuinely
observable cancelled JoinHandle. No current consumer reproduces the proposed
no-actor error. Opaque constructor Err already retains its complete reservation.
(b) globally Unknown plus all three Joined/NotCreated results could refund an
unproved frame if a future handler adds an unattributed panic. Current fixed
Note handling cannot panic; actor join Err sets actor_joined=false, worker/callback
panic keeps WorkerDisposition::Unknown, and Begin is sent after exact retained
installation. Neither hypothetical source is reachable in this selected handler
inventory. New handler/Ok native supervision activation must qualify its own
actor/panic/unknown-frame semantics; this component does not qualify those paths.
No native backend or parent policy expansion is claimed from these dispositions.

At immutable `0a21095a4a0b4931f1be6d47abc279414abf10a1`, all18 selected
consumers passed under default parallelism. The exact current compiled mutation
run killed16/16 by actual assertions; the restored18 controls passed and the
owned detached mutation worktree returned clean to that same head. The prior13
controls were retained, and these three causal omissions were added:

| Changed current boundary | Actual causal assertion |
| --- | --- |
| Reply before own effect-request retirement | Revoked queued Create freezes an invalid uncreated Held result |
| Fixed Note counts as outstanding effect | Actual second handler overlaps actor error; genuine Fresh no-work must survive |
| Registry holds pure metadata after exact Restored outcome | Actual checkpoint must restore its exact previous Control while all3 jobs stay charged |

The whole workspace default-parallel regression passed471 tests plus2 doctests;
26 existing cases were ignored. Fmt, warnings-denied clippy and Debug/Release builds
passed. CPP44 and all18 current mechanics passed; code tree is
`bc515cf97c25a12714763bf9e16f7158654efbeb`. This current-head observation does
not explain the historical7Grok failures or older native inspection/watchdog RED
results. Source2 must review this exact selected correction/consumer scope; even
approval cannot qualify broad CPP/schema5/schema6/native producer or make PR39 ready.
