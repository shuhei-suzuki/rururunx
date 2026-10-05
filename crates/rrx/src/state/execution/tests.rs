use super::*;
use crate::config::WorkflowClass;
use std::path::PathBuf;

fn fixture() -> (Store,Task,u64) {
    let mut store=Store::memory().unwrap();
    let mut p=Project::new("project".into(),PathBuf::from("/tmp/rrx-source"),"git-local".into(),"main".into());
    store.put_project(&mut p).unwrap();
    let mut g=Goal::new(p.id,"goal".into(),vec![CompletionCriterion {id:"result".into(),description:"accepted commit".into(),evidence:None,satisfied:false}]);
    store.put_goal(&mut g).unwrap();
    let mut t=Task::new(p.id,g.id,"task".into(),"codex".into());t.workflow=WorkflowClass::Quick;
    store.put_task(&mut t).unwrap();
    let (_,epoch)=store.begin_execution_epoch().unwrap();
    (store,t,epoch)
}
fn draft(task:&Task,epoch:u64) -> ExecutionUnit {
    let id=UnitId::new();let at=now_ms();
    ExecutionUnit {id,scope:task.scope(),kind:UnitKind::Executor,generation:0,owner_epoch:epoch,version:0,
        phase:"Implement".into(),provider:"codex".into(),state:UnitState::Reserved,native_effects_open:true,
        result_finalization_open:true,work:None,cleanup:CleanupOutcome::Unknown,disposition:Disposition::Active,
        worktree:PathBuf::from(format!("/tmp/rrx-source/worktree/{}-{id}",task.id)),branch:Some(format!("rrx/{}/{id}",task.id)),
        base_sha:"a".repeat(40),profile_digest:"b".repeat(64),cookie:Uuid::new_v4().to_string(),session_id:None,
        artifact_id:None,wait_reason:None,created_at:at,updated_at:at}
}

#[test]
fn normal_terminal_cleanup_keeps_finalization_and_result_publication_races_cancel() {
    let (mut store,t,epoch)=fixture();
    let unit=store.reserve_execution(draft(&t,epoch),t.version).unwrap();let a=unit.authority();
    let known=store.finish_execution(&a,WorkOutcome::Success,Disposition::Completed).unwrap();
    assert!(!known.native_effects_open && known.result_finalization_open);
    store.record_execution_cleanup(&CleanupObservation{unit_id:unit.id,at:now_ms(),outcome:CleanupOutcome::Unknown,
        coverage:BTreeMap::from([("process".into(),"inspection unavailable".into())]),remaining:vec![],errors:vec![]}).unwrap();
    assert_eq!(store.execution_unit(unit.id).unwrap().work,Some(WorkOutcome::Success));
    let current_authority=store.execution_unit(unit.id).unwrap().authority();
    assert!(store.validate_execution(&current_authority,false,true).is_ok());
    store.retire_execution(&current_authority,false).unwrap();
    assert!(store.validate_execution(&a,false,true).is_err());
    let current=store.task(t.id).unwrap().unwrap();
    let retry=store.reserve_execution(draft(&current,epoch),current.version).unwrap();
    assert_ne!(retry.worktree,unit.worktree);assert!(retry.generation>unit.generation);
    assert_eq!(store.execution_unit(unit.id).unwrap().work,Some(WorkOutcome::Success));
}

#[test]
fn pre_open_legacy_writer_and_cached_statement_cannot_write_after_upgrade() {
    let dir=tempfile::tempdir().unwrap();let path=dir.path().join("state.db");
    // Build the genuine old schema with a connection that has no new writer function.
    let old=Connection::open(&path).unwrap();old.execute_batch(include_str!("../schema.sql")).unwrap();
    old.pragma_update(None,"application_id",APPLICATION_ID).unwrap();old.pragma_update(None,"user_version",3).unwrap();
    let mut project=Project::new("old".into(),PathBuf::from("/p"),"git-local".into(),"main".into());project.version=1;
    old.execute("INSERT INTO projects(id,root,version,body) VALUES(?1,'/p',1,?2)",params![project.id.to_string(),serde_json::to_string(&project).unwrap()]).unwrap();
    let mut cached=old.prepare("UPDATE projects SET version=version+1,body=json_set(body,'$.version',version+1) WHERE root='/p'").unwrap();
    cached.execute([]).unwrap();
    let upgraded=Store::open(&path).unwrap();assert_eq!(upgraded.schema_version().unwrap(),4);
    assert!(cached.execute([]).is_err());
    for sql in ["UPDATE projects SET version=version+1 WHERE root='/p'","DELETE FROM projects WHERE root='/p'",
        "INSERT INTO projects(id,root,version,body) VALUES('q','/q',1,'{}')",
        "INSERT OR REPLACE INTO projects(id,root,version,body) VALUES('p','/p',1,'{}')"] {
        assert!(old.execute(sql,[]).is_err(),"legacy write succeeded: {sql}");
    }
    assert_eq!(old.query_row("SELECT version FROM projects WHERE root='/p'",[],|r|r.get::<_,i64>(0)).unwrap(),2);
}

#[test]
fn two_tasks_keep_independent_authority_and_orphan_port_leases() {
    let (mut store,t,epoch)=fixture();
    let first=store.reserve_execution(draft(&t,epoch),t.version).unwrap();
    let mut other=Task::new(t.project_id,t.goal_id,"sibling".into(),"codex".into());store.put_task(&mut other).unwrap();
    let second=store.reserve_execution(draft(&other,epoch),other.version).unwrap();
    let lease=ResourceLease{id:LeaseId::new(),unit_id:first.id,scope:first.scope.clone(),kind:ResourceKind::Ports,
        namespace:"host".into(),value:"30000-30031".into(),port_start:Some(30000),port_end:Some(30031),state:LeaseState::Reserved,version:1};
    store.reserve_execution_leases(&first.authority(),std::slice::from_ref(&lease)).unwrap();
    store.retire_execution(&first.authority(),false).unwrap();
    assert!(store.validate_execution(&second.authority(),true,false).is_ok());
    let collision=ResourceLease{unit_id:second.id,scope:second.scope.clone(),id:LeaseId::new(),..lease.clone()};
    assert!(store.reserve_execution_leases(&second.authority(),&[collision]).is_err());
    assert!(store.release_execution_lease(lease.id,second.id,1,true).is_err());
    assert_eq!(store.execution_leases(first.id).unwrap()[0].state,LeaseState::Reserved);
}

#[test]
fn stale_callbacks_and_generic_task_cancellation_cannot_reopen_authority() {
    let (mut store,t,epoch)=fixture();let unit=store.reserve_execution(draft(&t,epoch),t.version).unwrap();
    let preparing=store.transition_execution(&unit.authority(),UnitState::Preparing).unwrap();
    assert!(store.transition_execution(&unit.authority(),UnitState::DispatchPending).is_err());
    let pending=store.transition_execution(&preparing.authority(),UnitState::DispatchPending).unwrap();
    let running=store.transition_execution(&pending.authority(),UnitState::Running).unwrap();
    assert!(store.transition_execution(&running.authority(),UnitState::Preparing).is_err());
    let mut task=store.task(t.id).unwrap().unwrap();task.state=TaskState::Cancelled;store.put_task(&mut task).unwrap();
    assert!(store.validate_execution(&running.authority(),true,false).is_err());
    let old=store.execution_unit(unit.id).unwrap();assert!(!old.native_effects_open && !old.result_finalization_open);
}

#[test]
fn retirement_is_allowed_after_goal_pause_but_resume_cannot_revive_unit() {
    let (mut store,t,epoch)=fixture();let unit=store.reserve_execution(draft(&t,epoch),t.version).unwrap();
    let mut goal=store.goal(t.goal_id).unwrap().unwrap();goal.state=GoalState::Paused;store.put_goal(&mut goal).unwrap();
    assert!(store.validate_execution(&unit.authority(),true,false).is_err());
    store.retire_execution(&unit.authority(),false).unwrap();
    goal.state=GoalState::Running;store.put_goal(&mut goal).unwrap();
    assert!(store.validate_execution(&unit.authority(),true,false).is_err());
}

#[test]
fn ambiguous_quota_updates_cannot_clear_exhaustion_and_terminal_probe_is_released() {
    let (mut store,t,epoch)=fixture();let unit=store.reserve_execution(draft(&t,epoch),t.version).unwrap();
    let at=now_ms();let reset=at+60000;
    let observation=QuotaObservation {provider:"codex".into(),account_key:"unknown".into(),bucket:"all".into(),window_id:"old".into(),
        status:QuotaStatus::Exhausted,used_percent:Some(100.0),resets_at:Some(reset),observed_at:at,source_version:"fixture".into(),confirmed_subscription:true};
    store.observe_quota(&observation).unwrap();
    for state in [QuotaStatus::Stale,QuotaStatus::Unknown] {
        store.observe_quota(&QuotaObservation {status:state,observed_at:at+1,..observation.clone()}).unwrap();
        assert_eq!(store.quota_observations("codex","unknown").unwrap()[0].status,QuotaStatus::Exhausted);
    }
    assert!(matches!(store.reserve_execution_quota(&unit.authority(),"codex","unknown",6,2,3,at+2).unwrap(),quotas::QuotaAdmission::Waiting {reason:WaitReason::Quota,..}));
    let current=store.execution_unit(unit.id).unwrap();
    assert_eq!(store.reserve_execution_quota(&current.authority(),"codex","unknown",6,2,3,reset+1).unwrap(),quotas::QuotaAdmission::Admitted);
    let current=store.execution_unit(unit.id).unwrap();store.retire_execution(&current.authority(),false).unwrap();
    let task=store.task(t.id).unwrap().unwrap();let next=store.reserve_execution(draft(&task,epoch),task.version).unwrap();
    assert_eq!(store.reserve_execution_quota(&next.authority(),"codex","unknown",6,2,3,reset+1800001).unwrap(),quotas::QuotaAdmission::Admitted);
}

#[test]
fn malformed_legacy_authority_rolls_back_schema_upgrade() {
    let dir=tempfile::tempdir().unwrap();let path=dir.path().join("state.db");let old=Connection::open(&path).unwrap();
    old.execute_batch(include_str!("../schema.sql")).unwrap();old.pragma_update(None,"application_id",APPLICATION_ID).unwrap();old.pragma_update(None,"user_version",3).unwrap();
    old.execute("INSERT INTO projects(id,root,version,body) VALUES('invalid','/p',1,'{}')",[]).unwrap();
    assert!(Store::open(&path).is_err());assert_eq!(old.pragma_query_value(None,"user_version",|r|r.get::<_,i64>(0)).unwrap(),3);
    assert_eq!(old.query_row("SELECT COUNT(*) FROM sqlite_schema WHERE name='execution_units'",[],|r|r.get::<_,i64>(0)).unwrap(),0);
}

#[test]
fn body_corruption_cannot_redirect_unit_or_lease_authority() {
    let (mut store,t,epoch)=fixture();let unit=store.reserve_execution(draft(&t,epoch),t.version).unwrap();
    let mut corrupt=unit.clone();corrupt.worktree=PathBuf::from("/foreign");
    store.connection.execute("UPDATE execution_units SET body=?1 WHERE id=?2",params![serde_json::to_string(&corrupt).unwrap(),unit.id.to_string()]).unwrap();
    assert!(store.execution_unit(unit.id).is_err());assert!(store.retire_execution(&unit.authority(),false).is_err());
}
