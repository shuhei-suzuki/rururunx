# Issue 43: marker publication retention

This implements the retention boundary required by the approved managed binding
design §§2.2 and 5, on `8a1341b`. It is not the dispatch-marker writer,
OriginalMarker, PhaseLaunch, Native admission, or the record-only Session binder.
No composition availability or positive Fake capability is added.

The actual Runtime control admission consumes its same real pending-capacity
handle into publication retention before a marker transaction may begin. Queue
membership and supervisor identity are checked atomically. Caller Drop, shutdown,
and an uncertain transaction do not remove a publishing slot through unmarked
abandonment. Shutdown drains only explicitly unmarked slots. The original armed
PreparationGuard stays in the same slot; publication changes its Drop policy so
even final Runtime/slot destruction cannot falsely retire a potentially marked
Unit as never handed off. Such destruction leaves durable open ownership for
recovery. It neither reports cleanup nor certifies successful work.

Rollback is a separate actual Runtime operation under the same control admission.
It encodes the genuine allocation's original complete Unit outside SharedStore.
The actual Store Immediate checks exact Unit bytes/version and runtime epoch plus
absence of the allocated operation/pair/Session/NativeInvocation. Only a
successfully ended read-only transaction issues the private rollback observation.
Any changed row, present marker/identity, poison, or transaction error retains the
original slot. Restoring an unmarked slot after shutdown removes and retires only
that slot, after releasing SharedStore and queue locks. A plain ID, status DTO,
capability bit or copied observation cannot restore it.

Publication retention is nongrant. The actual Driver ticket, sealed marker plan,
one private publication transaction, known-commit OriginalMarker, same PhaseLaunch
handoff and Native producer still must compose. Until then, publishing slots have
no Native children and no legitimate marked-completion removal API. The genuine
producer test uses accepted Goal ingress, actual preparation and the installed
Native allocation port; it cannot pass while that preparation requires the absent
Driver producer. A setup failure is recorded as unverified, never substituted by
SQL ownership or a fabricated Session. Independent source review and fixed-head
checks are separate from whole Runtime/Native acceptance.
