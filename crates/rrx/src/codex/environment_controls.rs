// Synthetic-only controls at the actual constructor/launch/CAS consumers.
use super::*;
use super::super::super::environment::{ExecSite, HookPoint};
use crate::{domain::Project, state::Store};
use std::os::unix::ffi::OsStrExt;

fn synthetic_adapter(owned: &Fixture, executable: PathBuf) -> Arc<CodexAdapter> {
    Arc::new(CodexAdapter::with_environment("codex".into(),executable,owned.store.clone(),[("OPENAI_API_KEY".into(),"synthetic-codex-marker".into())]).unwrap().component_fixture())
}
fn writer(owned: &Fixture) -> Store {
    Store::open(&owned.request.project.root.parent().unwrap().join("state.sqlite3")).unwrap()
}
fn foreign(writer: &mut Store, reference: &str) {
    let mut project=Project::new("foreign synthetic".into(),"/tmp/foreign-synthetic".into(),"foreign-synthetic".into(),"main".into());
    project.environment_refs=vec![reference.into()]; writer.put_project(&mut project).unwrap();
}
#[tokio::test(flavor="multi_thread", worker_threads=2)]
async fn initial_and_each_selected_exec_refuse_new_foreign_conflict_before_that_spawn() {
    for site in [ExecSite::Initial,ExecSite::Version,ExecSite::Discovery,ExecSite::Main] {
        let owned=Fixture::new(false);
        let (executable,directory)=wire_fixture(&owned,"complete");
        let adapter=synthetic_adapter(&owned,executable);
        let mut writer=writer(&owned);
        adapter.environment_hooks.install(site,if site==ExecSite::Initial { HookPoint::Initial } else { HookPoint::PreCas },move |_| foreign(&mut writer,"OPENAI_API_KEY"));
        let before=no_effect_snapshot(&adapter,&owned.request.scope);
        let result=bounded(adapter.start(owned.request.clone())).await;
        // Complete any mutant-created actual supervisor before assertions.
        if let Ok(session)=&result { terminal_status(&adapter,&SessionRef::from(session)).await; }
        if directory.join("leader").exists() { assert_leader_reaped(&directory.join("leader")); }
        assert!(adapter.environment_hooks.total(site)==(0,0));
        assert!(!journal_values(&directory).iter().any(|v| v["method"]=="turn/start"));
        if site==ExecSite::Initial {
            assert!(adapter.availability.sites()[Site::GitExecution as usize]==0);
            let after=no_effect_snapshot(&adapter,&owned.request.scope);
            assert!(after["sessions"]==before["sessions"] && after["events"]==before["events"]);
        }
        let error=match result { Err(error)=>error, Ok(_)=>panic!("new foreign conflict reached native inference") };
        assert!(error.kind==ErrorKind::InvalidConfiguration && error.message=="native environment authority unavailable");
    }
}
#[tokio::test(flavor="multi_thread", worker_threads=2)]
async fn irrelevant_foreign_change_does_not_stale_own_start_or_restore_native_values() {
    let owned=Fixture::new(false);
    let (executable,directory)=wire_fixture(&owned,"complete");
    let adapter=synthetic_adapter(&owned,executable);
    let mut writer=writer(&owned);
    adapter.environment_hooks.install(ExecSite::Initial,HookPoint::Initial,move |_| foreign(&mut writer,"IRRELEVANT_SYNTHETIC"));
    let session=bounded(adapter.start(owned.request.clone())).await.unwrap();
    let status=terminal_status(&adapter,&session.into()).await;
    assert!(status.session.state==SessionState::Exited && status.session.pid.is_none());
    for site in [ExecSite::Version,ExecSite::Discovery,ExecSite::Main] { assert!(adapter.environment_hooks.total(site)==(1,1)); }
    assert!(std::fs::read_to_string(directory.join("version-canary")).unwrap()=="present");
    assert!(std::fs::read_to_string(directory.join("selected-canary")).unwrap()=="present");
    assert_leader_reaped(&directory.join("leader"));
}
#[test]
fn preparing_only_pre_exec_preserves_first_cause_and_never_calls_closed_admission() {
    for state in [0,1,2,3] {
        let preparation=super::super::super::preparation::Preparation::new();
        match state {
            0=>{ preparation.cancel(); },
            1=>{ preparation.consume(||Ok(())).unwrap(); },
            2=>{ preparation.checkpoint(2,||Ok(())).unwrap(); },
            _=>{ preparation.failed(failure(ErrorKind::Timeout,"synthetic first cause")); },
        }
        let mut called=false;
        let result=preparation.before_exec(|| { called=true; Ok(()) });
        assert!(!called && result.is_err());
        let error=result.err().unwrap();
        assert!(error.kind==if state==3 { ErrorKind::Timeout } else { ErrorKind::StateConflict });
    }
    let preparation=super::super::super::preparation::Preparation::new();
    preparation.before_exec(||Ok(())).unwrap();
    assert!(matches!(preparation.state().unwrap(),Admission::Preparing));
    let result=preparation.before_exec(||Err::<(),_>(failure(ErrorKind::StateConflict,"synthetic CAS")));
    assert!(result.is_err() && matches!(preparation.state().unwrap(),Admission::Failing(_)));
}
#[tokio::test]
#[ignore="env-cleared real public constructor entry; invoked by retained own process"]
async fn public_constructor_raw_child() {
    let directory=PathBuf::from(std::env::var_os("RRX_RAW_CTOR_FIXTURE").unwrap());
    let store=Arc::new(Mutex::new(Store::open(&directory.join("state.sqlite3")).unwrap()));
    let mut project=Project::new("raw synthetic".into(),"/tmp/raw-synthetic".into(),"raw-synthetic".into(),"main".into());
    project.environment_refs=vec!["OPENAI_API_KEY".into()];store.lock().unwrap().put_project(&mut project).unwrap();
    let adapter=CodexAdapter::new("codex".into(),"/definitely-not-native".into(),store).unwrap();
    assert!(adapter.baseline.value_matches("OPENAI_API_KEY",std::ffi::OsStr::from_bytes(b"synthetic-\xff")));
    assert!(adapter.environment_candidates(project.id).unwrap()==BTreeSet::from(["OPENAI_API_KEY".into()]));
    assert!(adapter.capabilities().is_empty());
    let long_name=format!("LC_{}","x".repeat(254));
    // A separately constructed common iterator proves the bound, without ambient override.
    assert!(CodexAdapter::with_environment("codex".into(),"/definitely-not-native".into(),adapter.store.clone(),[(long_name.into(),"synthetic".into())]).is_err());
    std::fs::write(directory.join("complete"),"raw constructor checked").unwrap();
}
#[tokio::test]
async fn real_public_constructor_captures_raw_os_once_in_scoped_child() {
    use crate::adapter::{ProcessGroup,cleanup_group};
    use std::process::Stdio;
    let directory=tempfile::tempdir().unwrap();
    let mut command=tokio::process::Command::new(std::env::current_exe().unwrap());
    command.args(["--exact","codex::session::tests::environment_controls::public_constructor_raw_child","--ignored","--nocapture"])
        .env_clear().env("HOME",directory.path()).env("PATH","/usr/bin:/bin")
        .env("RRX_RAW_CTOR_FIXTURE",directory.path())
        .env("OPENAI_API_KEY",std::ffi::OsStr::from_bytes(b"synthetic-\xff"))
        .env(std::ffi::OsStr::from_bytes(b"LC_\xff"),"synthetic excluded name")
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).kill_on_drop(true).process_group(0);
    let mut child=ProcessGroup::new(command.spawn().unwrap(),Arc::new(AtomicBool::new(false))).unwrap();
    let observed=tokio::time::timeout(std::time::Duration::from_secs(30),child.observe_exit()).await;
    child=cleanup_group(child).await.unwrap();
    let status=child.reap().await.unwrap();
    assert!(observed.is_ok() && status.success() && directory.path().join("complete").exists());
}
