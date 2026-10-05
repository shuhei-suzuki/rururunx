use super::*;
use crate::{config::Config,domain::{CompletionCriterion,Goal,Task},project::{AddProject,ProjectRegistry}};

async fn fixture()->(tempfile::TempDir,Arc<RuntimeOwner>,Task) {
    let dir=tempfile::tempdir().unwrap();let source=dir.path().join("repo");std::fs::create_dir(&source).unwrap();
    git(&source,["init","-b","main"]).await.unwrap();
    std::fs::write(source.join("answer.txt"),"base\n").unwrap();commit(&source).await;
    let owner=RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
    let mut store=owner.store.lock().unwrap();
    let project=ProjectRegistry::new(&mut store).add(&source,AddProject::default(),&Config::default()).unwrap();
    let mut goal=Goal::new(project.id,"fixture".into(),vec![CompletionCriterion{id:"answer".into(),description:"pinned result".into(),evidence:None,satisfied:false}]);store.put_goal(&mut goal).unwrap();
    let mut task=Task::new(project.id,goal.id,"answer".into(),"codex".into());store.put_task(&mut task).unwrap();
    drop(store);(dir,owner,task)
}
async fn commit(path:&Path)->String {
    git(path,["add","answer.txt"]).await.unwrap();
    git(path,["-c","user.name=Fixture","-c","user.email=fixture@example.invalid","-c","commit.gpgsign=false","commit","-m","fixture"]).await.unwrap();
    text(&git(path,["rev-parse","HEAD"]).await.unwrap()).unwrap()
}
fn make_writable(path:&Path) {
    let m=std::fs::symlink_metadata(path).unwrap();assert!(!m.file_type().is_symlink());
    std::fs::set_permissions(path,std::fs::Permissions::from_mode(m.permissions().mode()|0o700)).unwrap();
    if m.is_dir(){for entry in std::fs::read_dir(path).unwrap(){make_writable(&entry.unwrap().path());}}
}

#[tokio::test]
async fn captured_graph_survives_executor_changes_and_review_source_is_readonly() {
    let (_dir,owner,task)=fixture().await;let attempts=super::super::attempts::AttemptManager::new(owner.clone());
    let (unit,_)=attempts.prepare(task.id,"codex","Implement",None).await.unwrap();
    std::fs::write(unit.worktree.join("answer.txt"),"accepted\n").unwrap();let sha=commit(&unit.worktree).await;
    let done=owner.store.lock().unwrap().finish_execution(&unit.authority(),WorkOutcome::Success,Disposition::Completed).unwrap();
    let results=ResultStore::new(owner.clone());let artifact=results.capture(&done.authority(),&sha,BTreeMap::new()).await.unwrap();
    let version=owner.store.lock().unwrap().task(task.id).unwrap().unwrap().version;
    let accepted=results.publish(&done.authority(),&artifact,version).await.unwrap();
    assert!(!accepted.repository.join("objects/info/alternates").exists());
    let (reviewer,_)=attempts.prepare_snapshot(task.id,accepted.id,UnitKind::Reviewer,"codex","Review").await.unwrap();
    let snapshot=results.snapshot(&reviewer).await.unwrap();
    std::fs::write(unit.worktree.join("answer.txt"),"survivor changed old path\n").unwrap();commit(&unit.worktree).await;
    assert_eq!(std::fs::read_to_string(snapshot.source().join("answer.txt")).unwrap(),"accepted\n");
    assert!(std::fs::write(snapshot.source().join("answer.txt"),"mutate-consume-restore").is_err());
    assert!(git(snapshot.source(),["checkout","--detach",&unit.base_sha]).await.is_err());
    snapshot.verify().await.unwrap();
    // Deliberately dispose only this test's private old executor namespace.
    let project=owner.store.lock().unwrap().project(task.project_id).unwrap().unwrap();
    git(&project.root,["worktree","remove","--force",unit.worktree.to_str().unwrap()]).await.unwrap();
    results.verify(&accepted).await.unwrap();snapshot.verify().await.unwrap();
    make_writable(snapshot.source());
}

#[tokio::test]
async fn cancellation_wins_publication_without_losing_captured_draft() {
    let (_dir,owner,task)=fixture().await;let attempts=super::super::attempts::AttemptManager::new(owner.clone());
    let (unit,_)=attempts.prepare(task.id,"codex","Implement",None).await.unwrap();
    let done=owner.store.lock().unwrap().finish_execution(&unit.authority(),WorkOutcome::Success,Disposition::Completed).unwrap();
    let results=ResultStore::new(owner.clone());let artifact=results.capture(&done.authority(),&done.base_sha,BTreeMap::new()).await.unwrap();
    let version=owner.store.lock().unwrap().task(task.id).unwrap().unwrap().version;
    attempts.retire(&done.authority(),false).unwrap();
    assert!(results.publish(&done.authority(),&artifact,version).await.is_err());
    results.verify(&artifact).await.unwrap();
    assert_eq!(owner.store.lock().unwrap().result_artifact(artifact.id).unwrap().state,ArtifactState::Ready);
    assert_eq!(owner.store.lock().unwrap().execution_unit(unit.id).unwrap().work,Some(WorkOutcome::Success));
    let (retry,_)=attempts.prepare(task.id,"codex","Implement",None).await.unwrap();assert_ne!(retry.worktree,unit.worktree);
}

#[tokio::test]
async fn cancellation_while_waiting_for_common_git_prevents_preparation_spawn() {
    let (_dir,owner,task)=fixture().await;
    let gate=owner.git_gate.lock().await;
    let worker_owner=owner.clone();
    let worker=tokio::spawn(async move {
        super::super::attempts::AttemptManager::new(worker_owner).prepare(task.id,"codex","Implement",None).await
    });
    let unit=tokio::time::timeout(std::time::Duration::from_secs(10),async {
        loop {
            let units=owner.store.lock().unwrap().execution_units(None).unwrap();
            if let Some(unit)=units.into_iter().find(|u|u.state==UnitState::Preparing){break unit;}
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }).await.unwrap();
    owner.store.lock().unwrap().retire_execution(&unit.authority(),false).unwrap();
    drop(gate);
    assert!(worker.await.unwrap().is_err());
    assert!(!unit.worktree.exists());
    assert!(owner.store.lock().unwrap().managed_effects(unit.id).unwrap().is_empty());
}

#[tokio::test]
async fn preparation_failure_closes_authority_and_keeps_fresh_retry_admissible() {
    let (_dir,owner,task)=fixture().await;
    let project=owner.store.lock().unwrap().project(task.project_id).unwrap().unwrap();
    // Known before spawn, after durable reservation: an ordinary filesystem failure.
    std::fs::write(&project.worktree_root,"not a directory").unwrap();
    let attempts=super::super::attempts::AttemptManager::new(owner.clone());
    assert!(attempts.prepare(task.id,"codex","Implement",None).await.is_err());
    let old=owner.store.lock().unwrap().execution_units(Some(&task.scope())).unwrap().pop().unwrap();
    assert!(!old.native_effects_open && !old.result_finalization_open);
    assert_eq!(old.work,Some(WorkOutcome::Unknown));
    assert_eq!(old.cleanup,CleanupOutcome::Unknown);
    assert!(!owner.store.lock().unwrap().execution_leases(old.id).unwrap().is_empty());
    std::fs::remove_file(&project.worktree_root).unwrap();
    let (retry,profile)=attempts.prepare(task.id,"codex","Implement",None).await.unwrap();
    assert_ne!(retry.worktree,old.worktree);
    for args in [vec!["diff","--output=/tmp/unrelated"],vec!["show","--ext-diff"],vec!["log","--textconv"]] {
        let args=args.into_iter().map(str::to_owned).collect::<Vec<_>>();
        assert!(super::super::tools::plan(&profile,&retry,"git",&args,OperationId::new()).is_err());
    }
}
