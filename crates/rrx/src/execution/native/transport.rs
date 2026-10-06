//! Physical transport custody of the SAME prepared command and registered actor.
use super::*;
use std::sync::OnceLock;
use crate::state::{NativeTransportStartPlan,NativeTransportSettlementPlan,RegistrationProbe};

#[derive(Clone,Copy,PartialEq,Eq)]
enum Creation { NotAttempted, Attempted, ReturnedChild, SpawnErr(&'static str) }
#[derive(Clone,Copy,PartialEq,Eq)]
enum Handoff { None, Offered, Accepted, DroppedUnpolled }
struct ChildCustody {
    cell: process::NativeChildCell,
    creation: Creation,
    handoff: Handoff,
    stop_requested: bool,
    qualified: bool,
    pipes: bool,
    hygiene: &'static str,
    reap: Option<std::process::ExitStatus>,
}
pub(crate) struct NativeTransportCustody {
    plan: Arc<NativeTransportStartPlan>,
    candidate: Arc<phase_protocol::PhaseActor>,
    child: Mutex<ChildCustody>,
    registration_uncertain: Mutex<bool>,
    control: OnceLock<mpsc::Sender<Control>>,
    changed: tokio::sync::Notify,
    observation: Mutex<Option<Arc<NativeTransportObservation>>>,
    settlement: Mutex<Option<Arc<NativeTransportSettlementPlan>>>,
    terminal: Mutex<Option<Arc<NativeTerminal>>>,
}
pub(crate) struct NativeTransportObservation {
    custody: Weak<NativeTransportCustody>,
    plan: Arc<NativeTransportStartPlan>,
    actor: Arc<phase_protocol::PhaseActor>,
    state: EffectState,
    receipt: BTreeMap<String,String>,
}
impl NativeTransportObservation {
    pub(crate) fn plan(&self)->&Arc<NativeTransportStartPlan> { &self.plan }
    pub(crate) fn actor(&self)->&Arc<phase_protocol::PhaseActor> { &self.actor }
    pub(crate) fn state(&self)->EffectState { self.state }
    pub(crate) fn receipt(&self)->&BTreeMap<String,String> { &self.receipt }
    pub(crate) fn validate_original(&self)->Result<()> {
        let custody=self.custody.upgrade().context("transport observation custody ended")?;
        ensure!(Arc::ptr_eq(&custody.plan,&self.plan) && Arc::ptr_eq(&custody.candidate,&self.actor) && Arc::ptr_eq(self.actor.owner.origin(),&self.plan),"transport observation original differs");
        let original=custody.observation.lock().map_err(|_|anyhow::anyhow!("transport observation poisoned"))?;
        ensure!(original.as_ref().is_some_and(|v|std::ptr::eq(v.as_ref(),self)),"transport observation not retained original");
        drop(original);
        self.plan.prepared().validate_original()
    }
}
impl NativeTransportCustody {
    pub(super) fn matches_prepared(&self, prepared:&Arc<PreparedNativePhase>)->bool { Arc::ptr_eq(self.plan.prepared(),prepared) }
    pub(super) fn request_stop(&self) {
        let transferred={let mut child=self.child.lock().unwrap_or_else(std::sync::PoisonError::into_inner);child.stop_requested=true;matches!(child.handoff,Handoff::Offered|Handoff::Accepted)};
        self.candidate.owner.revoke();
        if transferred { if let Some(control)=self.control.get() { let _=control.try_send(Control::Cancel); } }
        self.changed.notify_waiters();
    }
    fn observe(self:&Arc<Self>)->Result<Arc<NativeTransportObservation>> {
        if let Some(saved)=self.observation.lock().map_err(|_|anyhow::anyhow!("transport observation poisoned"))?.clone() { return Ok(saved); }
        let (state,receipt)={
            let child=self.child.lock().map_err(|_|anyhow::anyhow!("transport child poisoned; retention only"))?;
            let (state,receipt)=match child.creation {
                Creation::NotAttempted => (EffectState::Resolved,BTreeMap::from([("creation".into(),"not_attempted".into()),("handoff".into(),"none".into())])),
                Creation::Attempted => anyhow::bail!("transport creation observation incomplete"),
                Creation::SpawnErr(class)=>(EffectState::Unknown,BTreeMap::from([("creation".into(),"attempt_without_returned_handle".into()),("spawn_error".into(),class.into())])),
                Creation::ReturnedChild=>{
                    let handoff=match child.handoff { Handoff::None=>"precore_retained",Handoff::Accepted=>"core_accepted",Handoff::DroppedUnpolled=>"core_dropped_unpolled",Handoff::Offered=>anyhow::bail!("transport handoff remains Offered") };
                    let reap=child.reap.map_or_else(||if child.handoff==Handoff::Accepted {"n/a".into()} else {"unknown".into()},|s|s.code().map_or_else(||"signal".into(),|code|format!("exit:{code}")));
                    (EffectState::Confirmed,BTreeMap::from([("creation".into(),"returned_child".into()),("identity".into(),if child.qualified {"qualified"}else{"unqualified"}.into()),("pipes".into(),if child.pipes {"complete"}else{"incomplete"}.into()),("handoff".into(),handoff.into()),("hygiene".into(),if child.handoff==Handoff::DroppedUnpolled {"group_signal_attempted"} else {child.hygiene}.into()),("reap".into(),reap)]))
                },
            }; (state,receipt)
        };
        let observation=Arc::new(NativeTransportObservation { custody:Arc::downgrade(self),plan:self.plan.clone(),actor:self.candidate.clone(),state,receipt });
        let mut slot=self.observation.lock().map_err(|_|anyhow::anyhow!("transport observation poisoned"))?;
        if let Some(saved)=&*slot { return Ok(saved.clone()); }
        *slot=Some(observation.clone());Ok(observation)
    }
    async fn precore_hygiene(&self)->Result<()> {
        {let mut child=self.child.lock().unwrap_or_else(std::sync::PoisonError::into_inner);if !matches!(child.cell,process::NativeChildCell::Raw(_)|process::NativeChildCell::Owned(_)) {return Ok(());}child.hygiene=child.cell.hygiene();}
        tokio::time::timeout(Duration::from_secs(10),async {
            loop {
                {let mut child=self.child.lock().unwrap_or_else(std::sync::PoisonError::into_inner);if let Some(status)=child.cell.try_reap()? {child.reap=Some(status);return Ok::<_,anyhow::Error>(());}}
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }).await.context("pre-Core leader reap pending")?
    }
    fn persist_no_core_terminal(&self,owner:&RuntimeOwner)->Result<()> {
        self.candidate.owner.revoke();
        let saved=self.terminal.lock().map_err(|_|anyhow::anyhow!("transport terminal poisoned"))?.clone();
        let terminal=if let Some(saved)=saved {saved} else {
            let binding=self.candidate.owner.binding_snapshot()?;
            let f=self.plan.launch().allocation().facts();
            let receipt=native_result::NativeResultReceipt {id:NativeResultId::new(),invocation_id:f.invocation_id,unit_id:f.unit_id,session_id:f.session_id,scope:f.scope.clone(),generation:f.generation,owner_epoch:f.epoch,provider:f.provider.into(),native_thread:None,native_turn:None,acquisition:native_result::AcquisitionStatus::Missing,authority:native_result::ReceiptAuthority::HistoricalDraft,text:None,structured_output:None,prefix:None,answer_sha256:None,terminal_sha256:None,wire:None,observed_work:WorkOutcome::Unknown,disposition:Disposition::Lost,diagnostics:vec!["native_supervisor_unavailable".into()],captured_at:now_ms(),version:1};
            let terminal=Arc::new(NativeTerminal {receipt,session:binding.session().clone(),failure:None,session_version:binding.record_version()});
            let mut slot=self.terminal.lock().map_err(|_|anyhow::anyhow!("transport terminal poisoned"))?;
            if let Some(saved)=&*slot {saved.clone()}else{*slot=Some(terminal.clone());terminal}
        };
        persist_saved_terminal(owner,Some(&self.candidate),&terminal)?;Ok(())
    }
    pub(super) async fn reconcile(self:&Arc<Self>,owner:&Arc<RuntimeOwner>)->Result<()> {
        if *self.registration_uncertain.lock().map_err(|_|anyhow::anyhow!("transport registration poisoned"))? {
            let launch=self.plan.launch().clone();
            let admission=launch.admission().enter(launch.clone()).await?;
            let probe=owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.confirm_prepared_transport(&self.plan,&admission)?;
            match probe {
                RegistrationProbe::Committed(known)=>{self.request_stop();ensure!(matches!(self.candidate.activate(known),phase_protocol::Activation::RevokedKnown),"transport uncertain activation mismatch");*self.registration_uncertain.lock().map_err(|_|anyhow::anyhow!("transport registration poisoned"))?=false;},
                _=>anyhow::bail!("SAME registration not known; transport Held"),
            }
        }
        self.candidate.owner.validate_known_registration()?;
        let no_core={let child=self.child.lock().unwrap_or_else(std::sync::PoisonError::into_inner);child.handoff==Handoff::None};
        if no_core { self.request_stop();self.precore_hygiene().await?; }
        let observation=self.observe()?;
        let saved=self.settlement.lock().map_err(|_|anyhow::anyhow!("transport settlement poisoned"))?.clone();
        let plan=if let Some(saved)=saved {saved}else{
            let normal=self.candidate.owner.is_live();
            let plan=crate::state::Store::plan_transport_settlement(owner,observation,!normal)?;
            *self.settlement.lock().map_err(|_|anyhow::anyhow!("transport settlement poisoned"))?=Some(plan.clone());plan
        };
        owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.record_transport_settlement(&plan)?;
        if no_core {
            self.persist_no_core_terminal(owner)?;
            let safe={let child=self.child.lock().unwrap_or_else(std::sync::PoisonError::into_inner);child.creation==Creation::NotAttempted || child.reap.is_some()};
            if safe {self.plan.prepared().actor.release_gate();}
        }
        Ok(())
    }
}

/// Armed only inside the successful cell-to-Core move; shell carries plain Weak.
pub(super) struct TransportHandoff { custody: Weak<NativeTransportCustody> }
impl TransportHandoff {
    pub(super) fn accept(&self)->Result<()> {
        let custody=self.custody.upgrade().context("transport custody ended before Core poll")?;
        {let mut child=custody.child.lock().map_err(|_|anyhow::anyhow!("transport child poisoned"))?;ensure!(child.handoff==Handoff::Offered,"Core was not offered SAME child");child.handoff=Handoff::Accepted;}
        custody.changed.notify_waiters();custody.plan.prepared().actor.release_gate();Ok(())
    }
}
impl Drop for TransportHandoff {
    fn drop(&mut self) {
        if let Some(custody)=self.custody.upgrade() {let mut child=custody.child.lock().unwrap_or_else(std::sync::PoisonError::into_inner);if child.handoff==Handoff::Offered {child.handoff=Handoff::DroppedUnpolled;drop(child);custody.changed.notify_waiters();}}
    }
}
struct CoreShell {
    owner:Arc<RuntimeOwner>,unit:ExecutionUnit,session:Session,invocation:NativeInvocationId,
    wire:Lines,update:watch::Sender<NativeStatus>,controls:mpsc::Receiver<Control>,native:String,
    frozen_terminal:Arc<Mutex<Option<Arc<NativeTerminal>>>>,phase:Arc<phase_protocol::PhaseActor>,
    custody:Weak<NativeTransportCustody>,drain:Option<tokio::task::JoinHandle<()>>,
}
impl CoreShell {
    fn into_core(self,child:process::OwnedProcess)->Core {
        Core {owner:self.owner,unit:self.unit,session:self.session,record_version:1,invocation:self.invocation,collector:native_result::Collector::default(),receipt_saved:false,captured_terminal:None,observed_input:None,observed_terminal:None,frozen_terminal:self.frozen_terminal,wire:self.wire,child,update:self.update,controls:self.controls,native:self.native,drain:self.drain.unwrap(),phase:Some(self.phase),handoff:Some(TransportHandoff {custody:self.custody})}
    }
}
fn physical_command(prepared:&PreparedNativePhase)->Command {
    let planned=&prepared.command;
    let mut command=Command::new(&planned.program);
    command.args(&planned.argv).current_dir(&planned.cwd).envs(&planned.environment).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).process_group(0).kill_on_drop(true);
    command
}
impl NativeSessions {
    /// G5 alone will connect the actual issuer to this private physical seam.
    pub(super) async fn start_prepared_transport(&self,preparation:&Arc<NativePreparationCustody>,prepared:Arc<PreparedNativePhase>)->Result<NativePhaseStart> {
        prepared.validate_open()?;
        let profile=version::qualified_physical_profile(&self.owner,&prepared.actor)?;
        let plan=crate::state::Store::plan_prepared_transport(&self.owner,prepared.clone())?;
        let candidate=phase_protocol::PhaseActor::prepared_candidate(&plan)?;
        let handle=ManagedSessionRef {scope:plan.unit().scope.clone(),unit:plan.unit().id,generation:plan.unit().generation,epoch:plan.unit().owner_epoch,session:plan.session().id};
        let initial=NativeStatus {handle:handle.clone(),authority:plan.unit().authority(),session:plan.session().clone(),work:None,disposition:Disposition::Active,cleanup:CleanupOutcome::Unknown,wait_reason:None,pending:vec![],result:None,receipt:None,observed_work:None,metrics:None,diagnostic:None,failure:None};
        let (update,status)=watch::channel(initial);let (control,controls)=mpsc::channel(16);let terminal=Arc::new(Mutex::new(None));
        ensure!(!self.entries.lock().map_err(|_|anyhow::anyhow!("sessions poisoned"))?.contains_key(&handle.session),"original Session registry key occupied");
        let custody=Arc::new(NativeTransportCustody {plan:plan.clone(),candidate:candidate.clone(),child:Mutex::new(ChildCustody {cell:process::NativeChildCell::Empty,creation:Creation::NotAttempted,handoff:Handoff::None,stop_requested:false,qualified:false,pipes:false,hygiene:"n/a",reap:None}),registration_uncertain:Mutex::new(false),control:OnceLock::new(),changed:tokio::sync::Notify::new(),observation:Mutex::new(None),settlement:Mutex::new(None),terminal:Mutex::new(None)});
        let mut command=physical_command(&prepared);
        preparation.retain_transport(custody.clone(),&prepared)?;
        let launch=plan.launch().clone();let admission=launch.admission().enter(launch.clone()).await?;
        admission.validate_for(&launch)?;prepared.actor.validate_open()?;ensure!(candidate.is_candidate(),"transport actor no longer Candidate");
        let known={self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.register_prepared_transport(&plan,&admission)};
        let known=match known {Ok(known)=>known,Err(error)=>{*custody.registration_uncertain.lock().map_err(|_|anyhow::anyhow!("registration custody poisoned"))?=true;return Err(error.context("SAME transport registration uncertain; no spawn"));}};
        let activation=candidate.activate(known);
        ensure!(!matches!(activation,phase_protocol::Activation::Mismatch),"transport activation mismatched");
        let open=admission.validate_for(&launch).is_ok() && prepared.actor.validate_open().is_ok() && candidate.owner.is_live();
        {let mut child=custody.child.lock().map_err(|_|anyhow::anyhow!("transport child poisoned"))?;
            if open && !child.stop_requested && matches!(child.cell,process::NativeChildCell::Empty) {
                child.creation=Creation::Attempted;
                match command.spawn() {Ok(process)=>{child.cell.adopt(process);child.creation=Creation::ReturnedChild;},Err(error)=>{child.creation=Creation::SpawnErr(match error.kind() {std::io::ErrorKind::NotFound=>"not_found",std::io::ErrorKind::PermissionDenied=>"permission_denied",_=>"other"});}}
            }
        }
        drop(admission);
        // Every subsequent local is child-free; no await until Core owns child.
        let pipes={let mut child=custody.child.lock().map_err(|_|anyhow::anyhow!("transport child poisoned"))?;child.cell.qualify()?;child.qualified=true;child.cell.upgrade_in_place()?;let pipes=child.cell.take_native_pipes()?;child.pipes=true;pipes};
        let limit=if plan.unit().provider=="claude" {claude_wire::LINE_LIMIT}else{4*1024*1024};
        let mut shell=CoreShell {owner:self.owner.clone(),unit:plan.unit().clone(),session:plan.session().clone(),invocation:launch.allocation().facts().invocation_id,wire:Lines::new(pipes.stdin,pipes.stdout,limit),update:update.clone(),controls,native:prepared.command.native_session.map(|id|id.to_string()).unwrap_or_default(),frozen_terminal:terminal.clone(),phase:candidate.clone(),custody:Arc::downgrade(&custody),drain:None};
        {let mut entries=self.entries.lock().map_err(|_|anyhow::anyhow!("sessions poisoned"))?;ensure!(!entries.contains_key(&handle.session),"original Session registry collision");entries.insert(handle.session,Entry {handle:handle.clone(),status,control:control.clone(),terminal,update,phase:Some(candidate.clone())});}
        shell.drain=Some(tokio::spawn(async move {use tokio::io::AsyncReadExt;let mut stderr=pipes.stderr;let mut buf=[0;8192];loop {match stderr.read(&mut buf).await {Ok(0)|Err(_)=>break,Ok(_)=>{}}}}));
        let transferred={let mut child=custody.child.lock().map_err(|_|anyhow::anyhow!("transport child poisoned"))?;let open=child.handoff==Handoff::None && !child.stop_requested && candidate.owner.is_live();let result=child.cell.transfer_with(open,shell,CoreShell::into_core);if result.is_ok() {child.handoff=Handoff::Offered;let _=custody.control.set(control);}result};
        let core=match transferred {Ok(core)=>core,Err(refused)=>{self.entries.lock().map_err(|_|anyhow::anyhow!("sessions poisoned"))?.remove(&handle.session);if let Some(drain)=&refused.shell.drain {drain.abort();}drop(refused.shell);anyhow::bail!(refused.reason);}};
        let f=launch.allocation().facts();
        tokio::spawn(core.run(f.input.clone(),f.model.map(str::to_owned),f.effort.map(str::to_owned),profile));
        let _=tokio::time::timeout(Duration::from_secs(5),async {loop {let changed=custody.changed.notified();let done={let child=custody.child.lock().unwrap_or_else(std::sync::PoisonError::into_inner);matches!(child.handoff,Handoff::Accepted|Handoff::DroppedUnpolled)};if done {break;}changed.await;}}).await;
        custody.reconcile(&self.owner).await?;
        ensure!(custody.child.lock().map_err(|_|anyhow::anyhow!("transport child poisoned"))?.handoff!=Handoff::DroppedUnpolled,"Core dropped before first poll; SAME custody Held");
        Ok(NativePhaseStart::Launched {handle,binding:Box::new(candidate.owner.binding_snapshot()?)})
    }
}
