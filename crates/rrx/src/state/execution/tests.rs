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
    assert!(store.validate_execution(&a,false,true).is_ok());
    store.retire_execution(&a,false).unwrap();
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
    old.execute("INSERT INTO projects(id,root,version,body) VALUES('p','/p',1,'{}')",[]).unwrap();
    let mut cached=old.prepare("UPDATE projects SET version=version+1 WHERE id='p'").unwrap();
    cached.execute([]).unwrap();
    let upgraded=Store::open(&path).unwrap();assert_eq!(upgraded.schema_version().unwrap(),4);
    assert!(cached.execute([]).is_err());
    for sql in ["UPDATE projects SET version=version+1 WHERE id='p'","DELETE FROM projects WHERE id='p'",
        "INSERT INTO projects(id,root,version,body) VALUES('q','/q',1,'{}')",
        "INSERT OR REPLACE INTO projects(id,root,version,body) VALUES('p','/p',1,'{}')"] {
        assert!(old.execute(sql,[]).is_err(),"legacy write succeeded: {sql}");
    }
    assert_eq!(old.query_row("SELECT version FROM projects WHERE id='p'",[],|r|r.get::<_,i64>(0)).unwrap(),2);
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
