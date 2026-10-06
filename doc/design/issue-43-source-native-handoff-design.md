# Same-preparation Sources to Runtime handoff

This is a finite integration of the approved [Driver producer design](issue-43-runtime-driver-producer-design.md) and [Binding10 §5.1](issue-43-managed-binding-design.md). It adds no result, cleanup, input, or native-readiness authority. Source baseline is `e99fdb006d67cbfdb74614f2798d284ada1fbd43`; the separately owned Runtime/Native consumer inventory is pinned to `ca46e4cd7e0ffb3ab390b1e71c4b917fff6662e4`. Normal source composition and independent source qualification are still required.

## 1. Actual consumer gap

The real `step_driven_initial` first-Executor edge has already committed the prescribed Task/Workflow/Context/Driver images and adopted the SAME Sources-owned bootstrap Unit. Known-post publication updates the SAME armed `PreparedExecutor`; it does not start Native or create Session/input evidence. The active Executor branch currently waits for Root's marker/jobs consumer.

`NativePhasePort::allocate` can provisionally capture that actual selected port, current Unit, complete encoded input and fresh allocation identities without effects. `PhaseSupervisor::reserve` already checks the allocation and actual guard before retaining both. However, `Runtime::reserve_pending_phase` takes those objects by value before awaiting `control_admission`: taking the sole guard out of Sources and placing it in that cancellable future is not a safe handoff. Root's marker planning also consumes its ticket; planning failure must not permit a fresh current-row ticket to replace the original admission frame.

The integration therefore separates Sources custody from Root's independent retained handoff envelope. Engine invokes the concrete synchronous Root retention port in the same poll as the Sources producer returns, before constructing or awaiting admission/publication. Engine subsequently holds an observation handle; Runtime owns the invocation and outcome.

## 2. Original Sources producer

`ManagedWorkflowSources::offer_first_executor` is crate-private and accepts the actual selected `Arc<NativePhasePort>`, the SAME owned `DriverReadTicket`, and the captured Task/Workflow/Context view. It returns a non-Clone/non-Deserialize `SourceNativeHandoff`; no public DTO, callback, ID, or flag constructs it. The producer retains a `SourceNativeCustody` in the original Sources slot before returning. A second offer or legacy `take_initial_executor` cannot take an already offered preparation.

Before allocation or guard transfer, the producer checks all of the following:

- The original Sources map membership/slot pointer and immutable Frame pointer are unchanged, recovery is absent, and the SAME `PreparedExecutor` still owns its exact adopted Unit through its armed guard. No pending preparatory advance is accepted as a completed cache publication.
- The ticket's complete captured Task/Workflow/Context equals the Engine view. This is the first configured Executor after the genuinely closed initial Evidence phases; generation, active index, Unit reference, executor alias and phase match. Session/execution binding, dispatch marker and prior native history are absent. Root's unchanged composition/preflight refusal precedes this producer.
- The immutable committed revision, dependency hashes, governing rules/configuration and mandatory selection match the actual frame and Context. PreparedInput is built from the complete existing Context envelope: scope, ContextPack kind, revision, Context version, source hashes and `serde_json::to_string(context.data)`. Model/effort come from that frame's actual selected agent configuration. There is no live-source fallback or caller-supplied payload grant.
- The selected port belongs to the same Runtime owner, alias/provider and actual registry origin. Its real allocation agrees with the complete adopted Unit, generation/epoch/version, namespace/base/path/branch/profile, role, artifact absence and exact rendered/encoded input. Actual allocation IDs/pair are kept as one object; no second allocation is made after a refusal or uncertain result.

Allocation remains effect-free. Its DTO facts are inspected only as negative identity checks, not consumed-input or qualified-profile evidence. Registered namespace/helper evidence and the original preparation remain the producers of their own facts; Source success or an empty frame never substitutes for them. Complete payload/encoding, corpus, namespace and captured-row bounds remain those of the existing consumers.

## 3. Ownership and APIs

Source-owned shapes are private and contain these real objects:

```text
SourceNativeCustody:
    immutable Frame Arc; original/adopted Unit identity;
    Weak<ManagedWorkflowSources>; Weak<original Sources slot>;
    Weak of the SAME original DriverReadTicket (identity only, never reminted);
    mutex-protected SAME PreparedExecutor + SAME NativeAllocation before admission;
    bounded transfer/held observation after admission, never a native grant.

SourceNativeHandoff (nonClone):
    Arc<SourceNativeCustody>;
    Arc of the SAME nonClone DriverReadTicket;
    exact original Workflow identity.
```

The Source-held custody contains **no strong ticket, Driver plan, capacity, OriginalMarker or Runtime reference**. Its frame has no producer backlink. The original ticket owns a Driver association, whose retained worker owns Sources; storing that ticket in the Source slot would create `Sources → slot → ticket → DriverRegistry/job → Sources`. A capacity/marker/Driver-plan backlink would create the same problem. The Source slot therefore never owns the handoff envelope, directly or indirectly.

Root owns a bounded `PhaseHandoffs` sibling registry, independent of Sources, PhaseSupervisor and DriverRegistry. Its concrete API is:

```text
Runtime::retain_source_handoff(
    handoff: SourceNativeHandoff
) -> Result<SourceHandoffObservation, SourceNativeHandoff> // synchronous

Runtime::admit_source_handoff(
    observation: &SourceHandoffObservation
) -> Result<...> // Root-owned async, actual retained identity only

SourceNativeCustody::lock_transfer(
    owner: &Arc<RuntimeOwner>, ticket: &DriverReadTicket
) -> Result<SourceNativeTransfer<'_>>
```

Root retention consumes the SAME envelope and wraps/retains one Arc before any future construction. Capacity refusal, stopped service, duplicate identity or poison returns the SAME envelope, not IDs or a new ticket. The returned observation contains only finite identity/Weak access to the actual retained entry. It cannot start, bind, prove liveness or reconstruct an entry. The bounded Root registry counts pending and held entries and uses its actual fair observation/reconciliation policy; fullness never evicts original custody to make room.

The Root admission consumer first acquires the actual `control_admission` boundary and verifies service/current original envelope and ticket. Only then does it obtain `SourceNativeTransfer`, a non-Clone synchronous RAII borrow of the actual custody. The custody's Weak ticket must upgrade to that exact captured object; another ticket with equal row bytes cannot replace it. Map membership and original slot/frame remain protected against replacement for the transfer; no Source mutex is held across an await or SharedStore acquisition. Root's concrete synchronous PhaseSupervisor bridge uses this borrow; it is not a generic closure/callback admission API.

The transfer splits the SAME PreparedExecutor into its actual guard and a private non-Clone remainder retaining its original owner/Unit. No `PreparationGuard::new`, disarm or Unit refresh occurs. Root passes the SAME allocation and guard to the actual reserve consumer. An ordinary refusal returns those SAME objects; `restore_refused(PendingReservationError)` rejoins the private remainder and returned guard under the original custody borrow. Success first records the actual returned capacity in Root's retained envelope, then `finish_accepted()` makes Source custody a finite/Weak observation only. It cannot steal the queue guard, reoffer, or retain a strong capacity. These steps have no intervening await, SQL write, native effect, or fallible post-transfer cache construction.

Root's envelope retains the actual capacity and original Arc ticket across marker planning/refusal, and retains the SAME saved marker plan before SQL. Root adapts marker planning to borrow/retain that Arc rather than consume the only original ticket on an error. Sharing this one captured object creates no fresh read credential. The envelope stays outside PhaseSupervisor slots: a slot must not retain an envelope that owns a capacity pointing back to that slot. After Root's existing queue/job/plan actually owns the complete continuation, envelope retirement is a custody transfer, not operation release.

## 4. Refusal, Drop and reconciliation

Before Root retention, a dropped/refused envelope cannot launch anything; Sources keeps the original armed preparation/allocation. Losing the original ticket makes that offered custody Held and non-reusable, rather than allowing a fresh ticket/allocation to ratify it. The same returned intact envelope can be submitted again to Root retention without recapture. Actual Source removal/retirement revokes access; a retained envelope detects the Weak-origin/membership mismatch and cannot grant admission. It still preserves its owned evidence/custody according to the existing actual guard policy.

After Root retention, Engine timeout/Drop/abort does not own or cancel the only admission/start future. Waiting for `control_admission` leaves all assets in retained custody. After successful reserve, PhaseSupervisor owns the SAME armed guard/allocation independently of Engine. Service stop prevents further admission; it uses the existing actual unmarked removal policy and keeps publishing/uncertain slots charged. Membership alone never proves a live Driver: the original ticket/OriginalMarker current-live validator remains mandatory.

Unwind or an incomplete transfer/cache publication is Held when its exact ownership/post state cannot be established. A generic error, missing guard, Session None, worker exit or Lost label cannot prove rollback/no dispatch. No partially transferred object is reconstructed from Unit/SQL rows. Marker uncertainty uses Root's SAME saved plan and exact post/absence predicates; only its authentic unpublished-marker proof enables the existing rollback path. Native start, returned proof, binder planning and job outcomes remain Root/B consumers, independently retained before Engine receives observations. Source does not release them or append authority writes.

## 5. Qualification boundary

Implement A's producer/custody/split-rejoin seams and Root's retention/admission consumer in their owned worktrees, then normally compose fixed commits. No Root/B-owned edits are made by A. InstalledDriverComposition issuer and native preflight stay unavailable until the genuine whole composition is present and positively qualified. A cannot test a successful Driver/native handoff by private SQL seeding, fake selected ports, fabricated pair/input proof or a readiness flag.

Required causal controls for the actual composed route include the original source/Context/Unit/provider mismatch before effects, caller Drop before/while admission, same-original capacity refusal/restoration, concurrent duplicate offer/admission and Source retirement, exact ticket retention on marker-plan error, stop before first effect, and uncertainty preserving the original plan/guard. A nongrant split/projection control is labelled only as mechanical evidence. Compile/setup failures and tests stopped at the unchanged missing-issuer refusal do not qualify those positive paths. Existing full regression RED and unclassified failures remain recorded; component design/source approval does not qualify Runtime, Binding43, Phase2, native accounts or MVP.

### Concrete Source API refinement

The implementation factors the original transient borrow as `custody.original_origin() -> Result<SourceNativeOrigin>` followed by `origin.lock_transfer(&custody, &owner, &ticket) -> Result<SourceNativeTransfer<'_>>`. The non-Clone origin upgrades only the SAME Weak producer/slot and lives on the Root caller's stack; the returned guards borrow it, without a self-referential container or a stored Source/Driver backlink. Root holds actual admission before this synchronous map→tried-slot→custody borrow. `take_original()` moves only the original allocation/guard and keeps the private remainder in custody. `restore_refused` returns either factual Restored/Held or the SAME boxed ownership-returning error on a protocol mismatch; Root must retain that error outside Source locks. `finish_accepted` is infallible after Root stores actual capacity. An unfinished transfer's Drop marks Held without creating a new preparation or rollback proof.

The real successful `AttemptManager::prepare_driver_source` returns a new armed guard after its internal plan-bearing preparation guard has been disarmed. That returned guard has no Driver-plan backlink; `publish_driven_adoption` updates only its exact Unit. The existing `PreparationGuard` implementation is unchanged. The Source capsule's weak ticket identity and the independent Root envelope therefore implement the stated cycle boundary without changing pending Driver-plan ownership.
