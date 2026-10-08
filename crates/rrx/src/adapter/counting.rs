//! FM §8.6.3: a read-only, test-only adapter callback counter. Each registered
//! adapter is wrapped in place; every `AgentAdapter` callback is counted and
//! then delegated unchanged. Registry lookups, Native phase ports and the
//! managed owner are not touched.
use super::*;
use std::sync::atomic::AtomicUsize;

struct Counting {
    inner: Arc<dyn AgentAdapter>,
    calls: Arc<AtomicUsize>,
}
impl Counting {
    fn count(&self) -> &dyn AgentAdapter {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.as_ref()
    }
}
impl AgentAdapter for Counting {
    fn capabilities(&self) -> BTreeSet<Capability> {
        self.count().capabilities()
    }
    fn probe(&self) -> AdapterResult<AgentInfo> {
        self.count().probe()
    }
    fn start(&self, request: LaunchRequest) -> AdapterFuture<'_, Session> {
        self.count().start(request)
    }
    fn managed_provider(&self) -> Option<&str> {
        self.count().managed_provider()
    }
    fn start_managed(
        &self,
        request: LaunchRequest,
        input: crate::execution::native::ManagedInput,
    ) -> AdapterFuture<'_, crate::execution::native::NativeStart> {
        self.count().start_managed(request, input)
    }
    fn transport_succeeded(&self, status: &SessionStatus) -> bool {
        self.count().transport_succeeded(status)
    }
    fn start_structured(
        &self,
        request: LaunchRequest,
        schema: Value,
    ) -> AdapterFuture<'_, Session> {
        self.count().start_structured(request, schema)
    }
    fn status(&self, session: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        self.count().status(session)
    }
    fn stop(&self, session: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        self.count().stop(session)
    }
    fn attach(&self, session: SessionRef) -> AdapterFuture<'_, ()> {
        self.count().attach(session)
    }
    fn resume(&self, session: SessionRef) -> AdapterFuture<'_, Session> {
        self.count().resume(session)
    }
    fn release(&self, session: SessionRef) -> AdapterResult<()> {
        self.count().release(session)
    }
    fn subscribe(&self, session: SessionRef) -> AdapterResult<watch::Receiver<SessionStatus>> {
        self.count().subscribe(session)
    }
    fn usage(
        &self,
        session: SessionRef,
        phase: String,
        review_round: Option<u32>,
    ) -> AdapterFuture<'_, Usage> {
        self.count().usage(session, phase, review_round)
    }
    fn submit_approval(&self, session: SessionRef, decision: Value) -> AdapterFuture<'_, ()> {
        self.count().submit_approval(session, decision)
    }
    fn pending_approvals(&self, session: SessionRef) -> AdapterFuture<'_, Value> {
        self.count().pending_approvals(session)
    }
    fn checkpoint(&self, session: SessionRef, input: PreparedInput) -> AdapterFuture<'_, ()> {
        self.count().checkpoint(session, input)
    }
    fn start_native_goal(&self, input: PreparedInput) -> AdapterFuture<'_, NativeGoalRef> {
        self.count().start_native_goal(input)
    }
    fn native_goal_status(&self, native_ref: NativeGoalRef) -> AdapterFuture<'_, Value> {
        self.count().native_goal_status(native_ref)
    }
    fn resume_native_goal(
        &self,
        native_ref: NativeGoalRef,
        input: PreparedInput,
    ) -> AdapterFuture<'_, NativeGoalRef> {
        self.count().resume_native_goal(native_ref, input)
    }
}
impl AgentRegistry {
    /// Wraps every registered adapter in place and returns the shared count of
    /// their callbacks. Call it before the registry is shared.
    pub(crate) fn count_callbacks(&mut self) -> Arc<AtomicUsize> {
        let calls = Arc::new(AtomicUsize::new(0));
        for adapter in self.adapters.values_mut() {
            *adapter = Arc::new(Counting {
                inner: adapter.clone(),
                calls: calls.clone(),
            });
        }
        calls
    }
}
