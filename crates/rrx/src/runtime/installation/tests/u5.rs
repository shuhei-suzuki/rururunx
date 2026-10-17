//! Installed fresh P controls. Peers are account-free, never qualification of an official CLI.
use super::*;
use crate::execution::native::PreparationObservation;
use crate::runtime::startup::StartupState;

const PRELUDE: &str = r#"
P=bool(os.environ.get('RRX_UNIT_ID'))
step='version' if sys.argv[1:]==['--version'] else ('schema' if 'generate-json-schema' in sys.argv else 'handshake')
if P:
    with open(os.path.join(TRACE,os.environ['RRX_UNIT_ID']), 'a') as f: f.write(step+'\n')
    if CASE.startswith('writable-') and step=='version':
        target=os.environ['RRX_TMPDIR']
        for _ in range(int(CASE.split('-')[1])): target=os.path.dirname(target)
        os.chmod(target,0o777)
    if step=='schema' and CASE.startswith('ipc'):
        import socket, struct, uuid
        if CASE in ('ipc-ambiguous','ipc-wait'):
            import time
            with open(os.path.join(TRACE,'ready'),'w') as f: f.write('ready')
            while not os.path.exists(os.path.join(TRACE,'release')): time.sleep(0.01)
        peer=socket.socket(socket.AF_UNIX);peer.connect(os.environ['RRX_RUNTIME_SOCKET'])
        request={'unit':os.environ['RRX_UNIT_ID'],'cookie':os.environ['RRX_PROCESS_COOKIE'],'tool':'git','args':['init','must-not-exist'],'cwd':os.getcwd(),'git_parent':None}
        if CASE=='ipc-bad': request['cookie']='bad-cookie'
        if CASE=='ipc-missing': del request['cookie']
        if CASE=='ipc-foreign': request['unit']=str(uuid.uuid4())
        def exact(n):
            out=b''
            while len(out)<n:
                more=peer.recv(n-len(out))
                if not more: raise RuntimeError('EOF')
                out+=more
            return out
        for i in range(33 if CASE=='ipc-overflow' else 1):
            if i:
                peer=socket.socket(socket.AF_UNIX);peer.connect(os.environ['RRX_RUNTIME_SOCKET'])
            data=json.dumps(request).encode();peer.sendall(struct.pack('!I',len(data))+data)
            try:
                frame=json.loads(exact(struct.unpack('!I',exact(4))[0]))
                assert frame=={'type':'refused','value':'managed tool refused or observation incomplete'},frame
            except (ConnectionResetError,RuntimeError):
                assert CASE=='ipc-missing'
            peer.close()
    if CASE=='schema-fail' and step=='schema': sys.exit(0)
    if CASE in ('stop-wait','stop-overflow','stop-stdout-overflow','stop-refusal') and step==('handshake' if CASE=='stop-refusal' else 'schema'):
        import time
        with open(os.path.join(TRACE,'ready'),'w') as f: f.write(str(os.getpid()))
        while not os.path.exists(os.path.join(TRACE,'release')): time.sleep(0.01)
        if CASE=='stop-overflow': sys.stderr.write('x'*(64*1024+1));sys.stderr.flush()
        if CASE=='stop-stdout-overflow': sys.stdout.write('x'*(64*1024+1));sys.stdout.flush()
        if CASE=='stop-refusal': print('invalid-protocol',flush=True)
        time.sleep(120)
    if CASE=='schema-stderr' and step=='schema': sys.stderr.write('x'*(64*1024+1));sys.stderr.flush()
    if CASE=='version-stderr' and step=='version': sys.stderr.write('x'*(64*1024+1));sys.stderr.flush()
    if CASE=='handshake-stderr' and step=='handshake': sys.stderr.write('x'*(1024*1024+1));sys.stderr.flush()
    if CASE=='handshake-silent' and step=='handshake':
        import time
        time.sleep(120)
    if CASE=='handshake-tail': SCENARIO='tail'
    if CASE=='handshake-flood' and step=='handshake':
        for i in range(1025): print(json.dumps({'method':'u5/added','params':{}}),flush=True)
    if CASE=='handshake-fail': SCENARIO='schema-only'
    if CASE=='identity-schema' and step=='schema':
        with open(sys.argv[0],'a') as f: f.write('\n# identity changed\n')
    if CASE=='version-fail' and step=='version': print('unqualified');sys.exit(0)
"#;
pub(super) fn fixture_p(case: &str) -> ControlFixture {
    fixture_with("codex", true, |config| {
        for (alias, agent) in &mut config.agents {
            let old = std::path::PathBuf::from(&agent.command[0]);
            let path = old.with_file_name(format!("u5-{alias}"));
            let trace = old.with_file_name("u5-trace");
            std::fs::create_dir_all(&trace).unwrap();
            let pair = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/codex-contract/basis-pair.json");
            let peer = u4::PEER.replace(
                "print('arbitrary updated CLI 88.1')",
                "print('codex-cli 0.160.0' if P else 'arbitrary updated CLI 88.1')",
            );
            let peer = peer.replace("with open(PAIR) as f: files=json.load(f)['files']", "with open(PAIR) as f: files=json.load(f)['files']\n    if P and CASE in ('additions','handshake-flood'):\n        files['ServerNotification']['oneOf'].append({'type':'object','properties':{'method':{'type':'string','enum':['u5/added']},'params':{'type':'object','properties':{}}},'required':['method','params']})");
            let peer = peer.replace(
                "    send({'id':999 if SCENARIO=='id' else v['id'],'result':result})",
                "    if P and CASE=='handshake-signal' and index==5:\n        import time\n        with open(os.path.join(TRACE,'ready'),'w') as f: f.write('ready')\n        while not os.path.exists(os.path.join(TRACE,'release')): time.sleep(0.01)\n    send({'id':999 if SCENARIO=='id' else v['id'],'result':result})",
            );
            let peer = peer
                + "\nif P and step=='handshake' and CASE in ('handshake-grace','handshake-signal'):\n    import time\n    time.sleep(1 if CASE=='handshake-grace' else 120)\n";
            let script = format!(
                "#!/usr/bin/python3\nimport json, os, sys\nCASE={case:?}\nPROVIDER={:?}\nSCENARIO='normal'\nTRACE={:?}\nPAIR={:?}\nFILES={}\n{PRELUDE}\n{peer}",
                agent.provider.as_deref().unwrap(),
                trace.to_str().unwrap(),
                pair.to_str().unwrap(),
                serde_json::to_string(crate::execution::native::schema::FILES).unwrap()
            );
            std::fs::write(&path, script).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
            agent.command = vec![path.to_str().unwrap().into()];
        }
    })
}

/// Writable P ancestors refuse the next helper spawn and leave unrelated output unchanged.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5r1b_writable_p_ancestors_refuse_before_schema_spawn() {
    for (level, before_version) in (0..=3).flat_map(|level| [(level, true), (level, false)]) {
        let case = if before_version {
            "normal".into()
        } else {
            format!("writable-{level}")
        };
        let mut f = fixture_p(&case);
        f.register_real_git_project();
        let sentinel = f._dir.path().join("outside-sentinel");
        std::fs::write(&sentinel, "unchanged").unwrap();
        accept(&f, 1).await;
        let parked = before_version.then(|| park(&f, PreparationObservation::VersionReady));
        f.runtime.start().await.unwrap();
        assert_eq!(u4::settled_s(&f).await.state, StartupState::Passed);
        if let Some((arrived, release)) = parked {
            arrive(&f, &arrived).await;
            let job = f.runtime.phase_jobs.observed_jobs().remove(0);
            let mut target = f
                .owner
                .root
                .join("units")
                .join(job.unit.to_string())
                .join("tmp");
            for _ in 0..level {
                target.pop();
            }
            std::fs::set_permissions(target, std::fs::Permissions::from_mode(0o777)).unwrap();
            release.add_permits(1);
        }
        wait(
            || {
                f.runtime
                    .phase_jobs
                    .observed_jobs()
                    .iter()
                    .any(|j| j.finished)
                    || f.runtime.phase_jobs.observed_jobs().iter().any(|j| {
                        effects(&f, j.unit)
                            .iter()
                            .any(|e| e.kind == "native_phase_schema")
                    })
            },
            "unsafe P did not refuse",
        )
        .await;
        let jobs = f.runtime.phase_jobs.observed_jobs();
        assert_eq!(jobs.len(), 1);
        assert!(
            effects(&f, jobs[0].unit)
                .iter()
                .all(|e| e.kind != "native_phase_schema"),
            "Schema intent recorded under a writable ancestor"
        );
        assert_eq!(
            jobs[0].refusal.as_deref(),
            Some("P directory ownership or mode refused")
        );
        let trace = f
            ._dir
            .path()
            .join("u5-trace")
            .join(jobs[0].unit.to_string());
        if before_version {
            assert!(
                !trace.try_exists().unwrap(),
                "Version spawned under a writable ancestor"
            );
        } else {
            assert_eq!(std::fs::read_to_string(trace).unwrap(), "version\n");
        }
        no_session(&f);
        assert_eq!(std::fs::read_to_string(sentinel).unwrap(), "unchanged");
        // Restore only fixture-owned modes so its ordinary shutdown can inspect custody.
        let mut target = f
            .owner
            .root
            .join("units")
            .join(jobs[0].unit.to_string())
            .join("tmp");
        for _ in 0..level {
            target.pop();
        }
        std::fs::set_permissions(target, std::fs::Permissions::from_mode(0o700)).unwrap();
        let _ = f.runtime.shutdown().await;
        finish(f).await;
    }
}
pub(super) async fn wait(mut predicate: impl FnMut() -> bool, label: &str) {
    let end = tokio::time::Instant::now() + Duration::from_secs(60);
    while !predicate() {
        assert!(tokio::time::Instant::now() < end, "{label}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
fn sessions(f: &ControlFixture) -> Arc<crate::execution::native::NativeSessions> {
    f.runtime
        .installed
        .as_ref()
        .ok()
        .unwrap()
        .registry
        .native_phase_port("worker")
        .unwrap()
        .selected_adapter()
        .unwrap()
        .sessions
        .clone()
}
fn park(
    f: &ControlFixture,
    stage: PreparationObservation,
) -> (Arc<tokio::sync::Semaphore>, Arc<tokio::sync::Semaphore>) {
    let arrived = Arc::new(tokio::sync::Semaphore::new(0));
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let (a, r) = (arrived.clone(), release.clone());
    sessions(f).set_preparation_observer(Some(Arc::new(move |seen| {
        let (a, r) = (a.clone(), r.clone());
        Box::pin(async move {
            if seen == stage {
                a.add_permits(1);
                r.acquire().await.unwrap().forget();
            }
        })
    })));
    (arrived, release)
}
async fn arrive(f: &ControlFixture, a: &tokio::sync::Semaphore) {
    let end = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        if let Ok(permit) = a.try_acquire() {
            permit.forget();
            return;
        }
        let jobs = f.runtime.phase_jobs.observed_jobs();
        assert!(
            !jobs.iter().any(|j| j.finished),
            "genuine P ended before required stage: {jobs:?}"
        );
        assert!(
            tokio::time::Instant::now() < end,
            "genuine P did not reach stage"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

pub(super) fn no_session(f: &ControlFixture) {
    let db = raw(f);
    for kind in ["native_input", "native_phase_transport", "native_setup"] {
        let n: usize = db
            .query_row(
                "SELECT count(*) FROM managed_effects WHERE json_extract(body,'$.kind')=?1",
                [kind],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 0, "U5 pre-registration boundary: {kind}");
    }
    let n: usize = db
        .query_row(
            "SELECT count(*) FROM records WHERE kind='session'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 0, "U5 Session registered");
    assert_eq!(
        count(f, "native_invocations"),
        0,
        "U5 invocation registered"
    );
}
fn effects(
    f: &ControlFixture,
    unit: crate::execution::UnitId,
) -> Vec<crate::execution::ManagedEffect> {
    f.owner.store.lock().unwrap().managed_effects(unit).unwrap()
}

/// S-5/I-P: same-actor fresh metadata and Git13 reach the original receipt reader, isolated across live Units.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5_fresh_p_and_ipc_original_survive_complete_inventory_read() {
    let mut f = fixture_p("ipc");
    // The installed scheduler permits only one active Task per Project.
    // Keep two simultaneous P actors without relaxing that production limit.
    let mut tasks = Vec::new();
    for project in ["u5-ipc-project-a", "u5-ipc-project-b"] {
        f.register_real_git_project_named(project);
        tasks.extend(accept(&f, 1).await.1);
    }
    assert_ne!(tasks[0].project_id, tasks[1].project_id);
    let (a, r) = park(&f, PreparationObservation::BeforeCommand);
    f.runtime.start().await.unwrap();
    assert_eq!(u4::settled_s(&f).await.state, StartupState::Passed);
    arrive(&f, &a).await;
    arrive(&f, &a).await;
    let jobs = f.runtime.phase_jobs.observed_jobs();
    assert_eq!(jobs.len(), 2);
    for job in &jobs {
        let rows = effects(&f, job.unit);
        let p: Vec<_> = rows
            .iter()
            .filter(|e| e.kind.starts_with("native_phase_"))
            .collect();
        assert_eq!(p.len(), 16, "fresh Version/Schema/Handshake/Git13 missing");
        assert!(
            p.iter()
                .all(|e| e.state == crate::execution::EffectState::Confirmed)
        );
        assert_eq!(
            p.iter()
                .filter(|e| e.kind == "native_phase_schema"
                    && e.receipt.get("ipc_marker_refused").map(String::as_str) == Some("1"))
                .count(),
            1,
            "I-P: CookieAcceptedMarkerRefused=1 lost between helper original and read"
        );
        assert!(
            p.iter()
                .filter(|e| e.kind != "native_phase_schema")
                .all(|e| e.receipt.get("ipc_marker_refused").map(String::as_str) == Some("0")),
            "I-P: cross-helper or cross-Unit attribution"
        );
        assert!(
            p.iter()
                .all(|e| e.receipt.get("ipc_invalid").map(String::as_str) == Some("false"))
        );
        let trace =
            std::fs::read_to_string(f._dir.path().join("u5-trace").join(job.unit.to_string()))
                .unwrap();
        assert_eq!(
            trace, "version\nschema\nhandshake\n",
            "S was reused as P or metadata order changed"
        );
        let custody = f
            .runtime
            .phase_jobs
            .observed_preparation_custody(job.unit)
            .unwrap();
        let history = custody.observed_helper_history().unwrap();
        let launch = history[0].observed_plan().actor().launch().clone();
        let admission = launch.admission().enter(launch.clone()).await.unwrap();
        f.owner
            .store
            .lock()
            .unwrap()
            .confirm_phase_helper_history(history.clone(), &admission)
            .unwrap();
        for missing in 0..history.len() {
            let mut changed = history.clone();
            changed.remove(missing);
            assert!(
                f.owner
                    .store
                    .lock()
                    .unwrap()
                    .confirm_phase_helper_history(changed, &admission)
                    .is_err(),
                "missing P/Git step {missing}"
            );
        }
        let other = jobs.iter().find(|j| j.unit != job.unit).unwrap();
        let foreign = f
            .runtime
            .phase_jobs
            .observed_preparation_custody(other.unit)
            .unwrap()
            .observed_helper_history()
            .unwrap();
        let seal = history[3].observed_plan().git_seal().unwrap();
        assert!(seal.matches_actor(history[3].observed_plan().actor()));
        assert!(
            !seal.matches_actor(foreign[3].observed_plan().actor()),
            "foreign genuine Git seal accepted another Preparing actor"
        );
        let foreign_seal = foreign[3].observed_plan().git_seal().unwrap();
        let error = crate::state::Store::plan_phase_git_intent(
            &f.owner,
            history[2].clone(),
            crate::execution::native::readonly::PhaseGitAction::SourceTop,
            foreign_seal.clone(),
            None,
        )
        .err()
        .expect("foreign genuine Git seal accepted by intent planner");
        assert_eq!(error.to_string(), "Git seal original actor differs");
        for index in [0, 1, 2, 3, 15] {
            let mut changed = history.clone();
            changed[index] = foreign[index].clone();
            assert!(
                f.owner
                    .store
                    .lock()
                    .unwrap()
                    .confirm_phase_helper_history(changed, &admission)
                    .is_err(),
                "foreign actor step {index}"
            );
        }
        drop(admission);
    }
    no_session(&f);
    r.add_permits(tasks.len());
    wait_closed(&f, &tasks).await;
    no_session(&f);
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}

/// Bad context cannot become marker evidence, and overflow invalidates the same original helper.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5_ipc_bad_context_and_overflow_do_not_qualify_marker_evidence() {
    for case in ["ipc-bad", "ipc-missing", "ipc-foreign", "ipc-overflow"] {
        let mut f = fixture_p(case);
        f.register_real_git_project();
        let _ = accept(&f, 1).await;
        f.runtime.start().await.unwrap();
        wait(
            || {
                f.runtime
                    .phase_jobs
                    .observed_jobs()
                    .iter()
                    .any(|j| j.finished)
            },
            "P negative did not end",
        )
        .await;
        let jobs = f.runtime.phase_jobs.observed_jobs();
        assert_eq!(jobs.len(), 1);
        let rows = effects(&f, jobs[0].unit);
        let schema = rows
            .iter()
            .find(|e| e.kind == "native_phase_schema")
            .expect("fresh Schema absent");
        assert_eq!(
            schema.receipt["ipc_marker_refused"],
            if case == "ipc-overflow" { "32" } else { "0" }
        );
        assert_eq!(
            schema.receipt["ipc_invalid"],
            if case == "ipc-overflow" {
                "true"
            } else {
                "false"
            }
        );
        assert_eq!(
            jobs[0].preparation.helpers_completed,
            case != "ipc-overflow"
        );
        no_session(&f);
        // Known refusal closes through the existing nongrant path. An invalid
        // prefix remains Held; it cannot invent completion/no-dispatch.
        let _ = f.runtime.shutdown().await;
        finish(f).await;
    }
}

/// S Passed never fills a missing P step, and an executable swap prevents the next Native helper.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5_missing_fresh_steps_and_program_swap_never_reach_prepared() {
    for case in [
        "version-fail",
        "schema-fail",
        "handshake-fail",
        "identity-schema",
    ] {
        let mut f = fixture_p(case);
        f.register_real_git_project();
        let _ = accept(&f, 1).await;
        f.runtime.start().await.unwrap();
        assert_eq!(u4::settled_s(&f).await.state, StartupState::Passed);
        wait(
            || {
                f.runtime.phase_jobs.observed_jobs().iter().any(|j| {
                    j.finished
                        || effects(&f, j.unit)
                            .iter()
                            .any(|e| e.kind == "native_phase_git")
                })
            },
            "failed P did not settle",
        )
        .await;
        let jobs = f.runtime.phase_jobs.observed_jobs();
        assert!(
            !jobs.is_empty(),
            "missing fresh P step was incorrectly closed as a completed prefix"
        );
        let job = jobs.into_iter().next().unwrap();
        if case == "identity-schema" {
            assert_eq!(
                job.refusal.as_deref(),
                Some("P helper executable changed"),
                "executable replacement must be refused by helper hash verification"
            );
        }
        assert!(
            !job.preparation.helpers_completed
                && !job.preparation.no_dispatch
                && !job.preparation.transport
        );
        let rows = effects(&f, job.unit);
        assert!(
            !rows.iter().any(|e| e.kind == "native_phase_git"),
            "Git dispatched after missing fresh step"
        );
        no_session(&f);
        let _ = f.runtime.shutdown().await;
        finish(f).await;
    }
}

/// A known U6 refusal retains the nongrant closure and explicit retry wait rather than registering a Session.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5_pre_registration_refusal_closes_and_requires_explicit_retry() {
    let mut f = fixture_p("normal");
    f.register_real_git_project();
    let (_, tasks) = accept(&f, 1).await;
    let (a, r) = park(&f, PreparationObservation::BeforeTransport);
    f.runtime.start().await.unwrap();
    arrive(&f, &a).await;
    let jobs = f.runtime.phase_jobs.observed_jobs();
    assert_eq!(jobs.len(), 1);
    assert!(jobs[0].preparation.helpers_completed && jobs[0].preparation.no_dispatch);
    let n: usize = raw(&f)
        .query_row(
            "SELECT count(*) FROM quota_leases WHERE active=1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 1, "Admit lost its lease");
    let original_unit = f
        .owner
        .store
        .lock()
        .unwrap()
        .execution_unit(jobs[0].unit)
        .unwrap();
    let service = Arc::new(nonsuccess::ServicePark::default());
    let open = nonsuccess::OpenPark(service.clone());
    let hook = service.clone();
    service.close();
    *f.runtime.before_service_reconcile.lock().unwrap() = Some(Arc::new(move || hook.visit()));
    f.runtime.wake.notify_one();
    wait(
        || service.parked(),
        "service did not park before typed refusal",
    )
    .await;
    no_session(&f);
    r.add_permits(1);
    wait(
        || {
            f.runtime.phase_jobs.observed_jobs().iter().any(|j| {
                j.refusal.as_deref().is_some_and(|s| {
                    s.contains("Codex Executor unavailable before registration (U6)")
                })
            })
        },
        "typed U6 refusal absent",
    )
    .await;
    service.open();
    wait(
        || {
            raw(&f)
                .query_row(
                    "SELECT count(*) FROM audit WHERE kind='rrx.private.workflow.phase_closed'",
                    [],
                    |r| r.get::<_, usize>(0),
                )
                .unwrap()
                == 1
        },
        "Known nongrant closure missing",
    )
    .await;
    let records = f
        .owner
        .store
        .lock()
        .unwrap()
        .records(&tasks[0].scope(), RecordKind::Workflow)
        .unwrap();
    let snapshot: crate::workflow::WorkflowSnapshot =
        serde_json::from_value(records[0].data.clone()).unwrap();
    let attempt = &snapshot.history[snapshot.active.unwrap()];
    assert_eq!(attempt.state, crate::workflow::AttemptState::Failed);
    assert_eq!(
        attempt.detail.as_deref(),
        Some("native start ended before dispatch; explicit retry required")
    );
    let before = serde_json::to_string(&effects(&f, original_unit.id)).unwrap();
    let response = retired_request(&f, &original_unit).await;
    assert_eq!(
        response,
        serde_json::json!({"type":"refused","value":"managed tool refused or observation incomplete"})
    );
    assert_eq!(
        serde_json::to_string(&effects(&f, original_unit.id)).unwrap(),
        before,
        "retired request rewrote an old P receipt"
    );
    drop(open);
    wait_closed(&f, &tasks).await;
    no_session(&f);
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}

/// Known nongrant closure retires each SAME job and Source envelope before stop.
pub(super) async fn wait_closed(f: &ControlFixture, tasks: &[Task]) {
    wait(
        || {
            f.runtime.phase_jobs.observed_jobs().is_empty()
                && tasks
                    .iter()
                    .all(|task| f.runtime.observe_source_handoff(task.id).unwrap().is_none())
        },
        "Known nongrant closure did not retire original jobs and handoffs",
    )
    .await;
    assert_eq!(
        raw(f)
            .query_row(
                "SELECT count(*) FROM audit WHERE kind='rrx.private.workflow.phase_closed'",
                [],
                |r| r.get::<_, usize>(0),
            )
            .unwrap(),
        tasks.len(),
        "missing or duplicate nongrant closure"
    );
}

fn commit_files(f: &ControlFixture, files: usize) {
    for index in 0..files {
        std::fs::write(
            f.project.root.join(format!("budget-{index}.txt")),
            format!("ordinary {index}\n"),
        )
        .unwrap();
    }
    for argv in [
        vec!["add", "--", "."],
        vec![
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--allow-empty",
            "-m",
            "budget corpus",
        ],
    ] {
        let mut command = std::process::Command::new("/usr/bin/git");
        command.current_dir(&f.project.root).args(argv);
        assert!(rrx_spawn_hygiene::status_std(command).unwrap().success());
    }
}
/// B-1: real committed blobs produce the baseline, P adds 16 rows, and B+1 refuses before Version.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5_installed_history_budget_boundary_and_plus_one() {
    let mut base = None;
    for target in [0, 230, 231] {
        let mut f = fixture_p("normal");
        f.register_real_git_project();
        let files = if target == 0 {
            0
        } else {
            target - base.unwrap()
        };
        commit_files(&f, files);
        let _ = accept(&f, 1).await;
        let (a, r) = park(&f, PreparationObservation::VersionReady);
        f.runtime.start().await.unwrap();
        arrive(&f, &a).await;
        let job = f.runtime.phase_jobs.observed_jobs().remove(0);
        let baseline = effects(&f, job.unit);
        assert!(baseline.iter().all(|e| e.kind == "git_helper"
            && matches!(
                e.state,
                crate::execution::EffectState::Confirmed | crate::execution::EffectState::Resolved
            )));
        if target == 0 {
            base = Some(baseline.len());
            assert!(baseline.len() < 230);
        } else {
            assert_eq!(
                baseline.len(),
                target,
                "real Source producer baseline differs"
            );
        }
        if target == 230 {
            let (prepared, go) = park(&f, PreparationObservation::BeforeTransport);
            r.add_permits(1);
            arrive(&f, &prepared).await;
            assert_eq!(
                effects(&f, job.unit).len(),
                246,
                "Prepared must consume the complete 16-helper prefix"
            );
            let installed_rows = effects(&f, job.unit).len();
            let budget = crate::execution::native::native_effect_budget(
                "codex",
                crate::domain::SessionRole::Executor,
            )
            .unwrap();
            budget.check_prepared(installed_rows).unwrap();
            assert!(
                budget.check_prepared(installed_rows + 1).is_err(),
                "Prepared +1 exceeded actual installed tail reserve"
            );
            let custody = f
                .runtime
                .phase_jobs
                .observed_preparation_custody(job.unit)
                .unwrap();
            assert!(
                custody.observed_facts().helpers_completed && custody.observed_facts().no_dispatch
            );
            go.add_permits(1);
        } else {
            r.add_permits(1);
        }
        wait(
            || {
                f.runtime
                    .phase_jobs
                    .observed_jobs()
                    .iter()
                    .all(|j| j.finished)
            },
            "budget P did not end",
        )
        .await;
        if target == 231 {
            let rows = effects(&f, job.unit);
            assert_eq!(rows.len(), 231, "baseline+1 ran a Native helper");
            assert!(rows.iter().all(|e| e.kind == "git_helper"));
        }
        no_session(&f);
        let _ = f.runtime.shutdown().await;
        finish(f).await;
    }
}

/// A current epoch change invalidates an otherwise complete original history without minting a replacement.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5_complete_history_rejects_epoch_drift() {
    let mut f = fixture_p("normal");
    f.register_real_git_project();
    let _ = accept(&f, 1).await;
    let (a, r) = park(&f, PreparationObservation::BeforeCommand);
    f.runtime.start().await.unwrap();
    arrive(&f, &a).await;
    let job = f.runtime.phase_jobs.observed_jobs().remove(0);
    let custody = f
        .runtime
        .phase_jobs
        .observed_preparation_custody(job.unit)
        .unwrap();
    let history = custody.observed_helper_history().unwrap();
    let launch = history[0].observed_plan().actor().launch().clone();
    let admission = launch.admission().enter(launch.clone()).await.unwrap();
    f.owner
        .store
        .lock()
        .unwrap()
        .confirm_phase_helper_history(history.clone(), &admission)
        .unwrap();
    // Negative corruption only: no successful observation, actor or receipt is seeded.
    raw(&f)
        .execute(
            "UPDATE runtime_epoch SET epoch=epoch+1 WHERE singleton=1",
            [],
        )
        .unwrap();
    assert!(
        f.owner
            .store
            .lock()
            .unwrap()
            .confirm_phase_helper_history(history, &admission)
            .is_err()
    );
    drop(admission);
    r.add_permits(1);
    no_session(&f);
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

/// Q-1: Wait parks with no input; a due recovery Admit retains both the real lease and the pool's probe.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5_quota_wait_and_recovery_admit_preserve_original_tail_and_probe() {
    for recovery in [false, true] {
        let mut f = fixture_p("normal");
        f.register_real_git_project();
        let _ = accept(&f, 1).await;
        let now = crate::domain::now_ms();
        f.owner
            .store
            .lock()
            .unwrap()
            .observe_quota(&crate::execution::QuotaObservation {
                provider: "codex".into(),
                account_key: "unknown".into(),
                bucket: "primary".into(),
                window_id: "u5-window".into(),
                status: crate::execution::QuotaStatus::Exhausted,
                used_percent: Some(100.0),
                resets_at: Some(if recovery { now - 1 } else { now + 3_600_000 }),
                observed_at: now,
                source_version: "fixture".into(),
                confirmed_subscription: true,
            })
            .unwrap();
        let (a, r) = park(&f, PreparationObservation::BeforeTransport);
        f.runtime.start().await.unwrap();
        if recovery {
            arrive(&f, &a).await;
            let jobs = f.runtime.phase_jobs.observed_jobs();
            assert_eq!(jobs.len(), 1);
            let (lease,probe):(usize,String)=raw(&f).query_row("SELECT (SELECT count(*) FROM quota_leases WHERE active=1),probe_unit FROM quota_pools WHERE provider='codex' AND account_key='unknown'",[],|row|Ok((row.get(0)?,row.get(1)?))).unwrap();
            assert_eq!(lease, 1);
            assert_eq!(
                probe,
                jobs[0].unit.to_string(),
                "recovery Admit lost original probe"
            );
            assert!(jobs[0].preparation.no_dispatch && jobs[0].preparation.helpers_completed);
            no_session(&f);
            r.add_permits(1);
        } else {
            wait(
                || {
                    raw(&f)
                        .query_row(
                            "SELECT count(*) FROM managed_phase_readiness WHERE state='parked'",
                            [],
                            |row| row.get::<_, usize>(0),
                        )
                        .unwrap()
                        == 1
                },
                "quota Wait was not Parked",
            )
            .await;
            assert_eq!(
                raw(&f)
                    .query_row(
                        "SELECT count(*) FROM quota_leases WHERE active=1",
                        [],
                        |row| row.get::<_, usize>(0)
                    )
                    .unwrap(),
                0
            );
            assert_eq!(a.available_permits(), 0, "Wait reached Prepared");
            let job = f.runtime.phase_jobs.observed_jobs().remove(0);
            assert!(
                job.preparation.helpers_completed
                    && job.preparation.no_dispatch
                    && job.preparation.parked
            );
            no_session(&f);
        }
        let _ = f.runtime.shutdown().await;
        finish(f).await;
    }
}

/// Only the same Unit's fresh Schema additions reach the production Session decoder contract.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5_fresh_schema_additions_are_memory_only_session_decoder_input() {
    let mut f = fixture_p("additions");
    f.register_real_git_project();
    let _ = accept(&f, 1).await;
    let (a, r) = park(&f, PreparationObservation::BeforeTransport);
    f.runtime.start().await.unwrap();
    arrive(&f, &a).await;
    let job = f.runtime.phase_jobs.observed_jobs().remove(0);
    let custody = f
        .runtime
        .phase_jobs
        .observed_preparation_custody(job.unit)
        .unwrap();
    let contract = custody.observed_session_contract().unwrap();
    let notification = serde_json::json!({"method":"u5/added","params":{"ignored":"memory only"}});
    assert!(crate::codex::managed::frame(&notification, &contract).is_ok());
    assert!(
        crate::codex::managed::frame(
            &notification,
            &crate::execution::codex_wire::CodexWireContract::default()
        )
        .is_err()
    );
    let rows = effects(&f, job.unit);
    let schema = rows
        .iter()
        .find(|e| e.kind == "native_phase_schema")
        .unwrap();
    assert!(schema.receipt["schema_counts"].ends_with(":1"));
    assert!(
        !serde_json::to_string(&rows).unwrap().contains("u5/added"),
        "accepted names persisted"
    );
    no_session(&f);
    r.add_permits(1);
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

/// Ambiguous active candidates cannot receive marker attribution even with a genuine helper's current cookie.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5_ipc_ambiguous_original_candidates_invalidate_observation() {
    let mut f = fixture_p("ipc-ambiguous");
    f.register_real_git_project();
    let _ = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    let trace = f._dir.path().join("u5-trace");
    wait(
        || trace.join("ready").exists(),
        "actual Schema helper absent",
    )
    .await;
    let job = f.runtime.phase_jobs.observed_jobs().remove(0);
    let unit = f
        .owner
        .store
        .lock()
        .unwrap()
        .execution_unit(job.unit)
        .unwrap();
    f.owner.duplicate_active_preparation_sink(&unit).unwrap();
    std::fs::write(trace.join("release"), "release").unwrap();
    wait(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.finished)
        },
        "ambiguous P did not end",
    )
    .await;
    let rows = effects(&f, job.unit);
    let schema = rows
        .iter()
        .find(|e| e.kind == "native_phase_schema")
        .unwrap();
    assert_eq!(schema.receipt["ipc_marker_refused"], "0");
    assert_eq!(schema.receipt["ipc_invalid"], "true");
    assert!(
        !f.runtime.phase_jobs.observed_jobs()[0]
            .preparation
            .helpers_completed
    );
    no_session(&f);
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

async fn retired_request(
    f: &ControlFixture,
    unit: &crate::execution::ExecutionUnit,
) -> serde_json::Value {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::net::UnixStream::connect(f.owner.ipc_path())
        .await
        .unwrap();
    let request=serde_json::to_vec(&serde_json::json!({"unit":unit.id,"cookie":unit.cookie,"tool":"git","args":["init","must-not-exist"],"cwd":unit.worktree,"git_parent":null})).unwrap();
    stream.write_u32(request.len() as u32).await.unwrap();
    stream.write_all(&request).await.unwrap();
    let n = stream.read_u32().await.unwrap() as usize;
    assert!(n <= 128 * 1024);
    let mut bytes = vec![0; n];
    stream.read_exact(&mut bytes).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// An unmatchable post-Admit inventory stays Held with the lease; no receipt image may manufacture a close.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5_uncertain_original_inventory_keeps_admitted_lease() {
    let mut f = fixture_p("normal");
    f.register_real_git_project();
    let _ = accept(&f, 1).await;
    let (a, r) = park(&f, PreparationObservation::BeforeTransport);
    f.runtime.start().await.unwrap();
    arrive(&f, &a).await;
    let job = f.runtime.phase_jobs.observed_jobs().remove(0);
    let rows = effects(&f, job.unit);
    let original = rows
        .iter()
        .find(|e| e.kind == "native_phase_handshake")
        .unwrap();
    // A negative corruption, never a success seed or a new authority object.
    let mut unknown = original.clone();
    unknown.state = crate::execution::EffectState::Unknown;
    raw(&f)
        .execute(
            "UPDATE managed_effects SET state='unknown',body=?1 WHERE id=?2",
            rusqlite::params![
                serde_json::to_string(&unknown).unwrap(),
                original.id.to_string()
            ],
        )
        .unwrap();
    r.add_permits(1);
    wait(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.attention == Some("original preparation Held"))
        },
        "uncertain original was not Held",
    )
    .await;
    assert_eq!(
        raw(&f)
            .query_row(
                "SELECT count(*) FROM quota_leases WHERE active=1",
                [],
                |row| row.get::<_, usize>(0)
            )
            .unwrap(),
        1,
        "Held released original lease"
    );
    assert_eq!(
        raw(&f)
            .query_row(
                "SELECT count(*) FROM audit WHERE kind='rrx.private.workflow.phase_closed'",
                [],
                |row| row.get::<_, usize>(0)
            )
            .unwrap(),
        0,
        "unmatched inventory issued closure"
    );
    no_session(&f);
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

/// Executable replacement after S and after Prepared is rejected at the original identity boundary before any new spawn.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5_program_identity_is_checked_before_p_and_before_transport() {
    for stage in [
        PreparationObservation::VersionReady,
        PreparationObservation::BeforeTransport,
    ] {
        let mut f = fixture_p("normal");
        f.register_real_git_project();
        let _ = accept(&f, 1).await;
        let (a, r) = park(&f, stage);
        f.runtime.start().await.unwrap();
        arrive(&f, &a).await;
        let job = f.runtime.phase_jobs.observed_jobs().remove(0);
        let path = std::path::PathBuf::from(&f.runtime.config.agents["worker"].command[0]);
        let mut changed = std::fs::read(&path).unwrap();
        changed.extend_from_slice(b"\n# replacement after S\n");
        std::fs::write(path, changed).unwrap();
        // Hold Root so the precise original refusal can be inspected before its nongrant close.
        let service = Arc::new(nonsuccess::ServicePark::default());
        let open = nonsuccess::OpenPark(service.clone());
        let hook = service.clone();
        service.close();
        *f.runtime.before_service_reconcile.lock().unwrap() = Some(Arc::new(move || hook.visit()));
        f.runtime.wake.notify_one();
        wait(|| service.parked(), "service did not park").await;
        r.add_permits(1);
        wait(
            || {
                f.runtime
                    .phase_jobs
                    .observed_jobs()
                    .iter()
                    .any(|j| j.finished)
            },
            "identity refusal did not end",
        )
        .await;
        let jobs = f.runtime.phase_jobs.observed_jobs();
        let text = jobs[0].refusal.as_deref().unwrap();
        assert!(
            text.contains(if stage == PreparationObservation::VersionReady {
                "fresh P program differs from all-alias S identity"
            } else {
                "transport executable changed"
            }),
            "wrong refusal: {text}"
        );
        if stage == PreparationObservation::VersionReady {
            assert!(effects(&f, job.unit).iter().all(|e| e.kind == "git_helper"));
        }
        no_session(&f);
        service.open();
        drop(open);
        let _ = f.runtime.shutdown().await;
        finish(f).await;
    }
}

/// Fresh P deadlines, aggregate stream capture, notification counts and malformed tails cannot be substituted by S success.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5_metadata_limits_and_tails_refuse_before_git() {
    for case in [
        "version-stderr",
        "schema-stderr",
        "handshake-stderr",
        "handshake-tail",
        "handshake-flood",
        "handshake-silent",
    ] {
        let mut f = fixture_p(case);
        f.register_real_git_project();
        let _ = accept(&f, 1).await;
        f.runtime.start().await.unwrap();
        assert_eq!(u4::settled_s(&f).await.state, StartupState::Passed);
        wait(
            || {
                f.runtime.phase_jobs.observed_jobs().iter().any(|j| {
                    j.finished
                        || effects(&f, j.unit)
                            .iter()
                            .any(|e| e.kind == "native_phase_git")
                })
            },
            "bounded P did not end",
        )
        .await;
        let job = f.runtime.phase_jobs.observed_jobs().remove(0);
        let rows = effects(&f, job.unit);
        assert!(
            !rows.iter().any(|e| e.kind == "native_phase_git"),
            "failed P metadata reached Git: {case}"
        );
        if case == "handshake-tail" {
            let handshake = rows
                .iter()
                .find(|e| e.kind == "native_phase_handshake")
                .unwrap();
            assert_eq!(
                handshake.state,
                crate::execution::EffectState::Confirmed,
                "EOF and reap known, malformed tail is a Known negative"
            );
            assert_eq!(handshake.receipt["expectation"], "mismatch");
        }
        assert!(!job.preparation.helpers_completed && !job.preparation.no_dispatch);
        no_session(&f);
        let _ = f.runtime.shutdown().await;
        finish(f).await;
    }
}

/// A correlated Handshake gets its fixed spontaneous-exit grace before any group signal is attempted.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5_handshake_spontaneous_exit_grace_precedes_signal() {
    let mut f = fixture_p("handshake-grace");
    f.register_real_git_project();
    let _ = accept(&f, 1).await;
    let (a, r) = park(&f, PreparationObservation::BeforeCommand);
    f.runtime.start().await.unwrap();
    arrive(&f, &a).await;
    let job = f.runtime.phase_jobs.observed_jobs().remove(0);
    let rows = effects(&f, job.unit);
    let handshake = rows
        .iter()
        .find(|e| e.kind == "native_phase_handshake")
        .unwrap();
    assert_eq!(handshake.state, crate::execution::EffectState::Confirmed);
    assert_eq!(
        handshake.receipt["exit"], "0",
        "Handshake was signalled before its self-exit grace"
    );
    assert_eq!(
        handshake.receipt["capture"],
        "stdout_eof=true;stderr_eof=true;leader_reaped=true;stop=self"
    );
    no_session(&f);
    r.add_permits(1);
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

/// Existing producer receipts stay readable through preparation and nongrant closure.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5r1a_existing_receipts_survive_preparation_and_closure() {
    let mut f = fixture_p("normal");
    f.register_real_git_project();
    let (_, tasks) = accept(&f, 1).await;
    let (a, r) = park(&f, PreparationObservation::VersionReady);
    f.runtime.start().await.unwrap();
    arrive(&f, &a).await;
    let job = f.runtime.phase_jobs.observed_jobs().remove(0);
    let row = effects(&f, job.unit)
        .into_iter()
        .find(|e| e.kind == "git_helper" && e.state == crate::execution::EffectState::Confirmed)
        .unwrap();
    // Use the existing compatible writer, never a successful SQL seed.
    f.owner
        .store
        .lock()
        .unwrap()
        .reconcile_managed_effect(
            row.id,
            row.version,
            row.state,
            std::collections::BTreeMap::from([
                ("dispatch_detail".into(), "x".repeat(129)),
                ("config".into(), "a".repeat(256)),
            ]),
        )
        .unwrap();
    r.add_permits(1);
    wait_closed(&f, &tasks).await;
    let rows = effects(&f, job.unit);
    assert_eq!(
        rows.iter().find(|e| e.id == row.id).unwrap().receipt["config"].len(),
        256
    );
    assert_eq!(
        rows.iter()
            .filter(|e| e.kind.starts_with("native_phase_"))
            .count(),
        16
    );
    assert_eq!(
        raw(&f)
            .query_row(
                "SELECT count(*) FROM quota_leases WHERE active=1",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
        0
    );
    no_session(&f);
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}

/// Abandon, lost fences and either stream overflow signal an actual helper without self-exit grace.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5r1a_cancel_fence_and_overflow_signal_without_grace() {
    for cause in ["abandon", "fence", "stop-overflow", "stop-stdout-overflow"] {
        let mut f = fixture_p(if cause.starts_with("stop-") {
            cause
        } else {
            "stop-wait"
        });
        f.register_real_git_project();
        let _ = accept(&f, 1).await;
        f.runtime.start().await.unwrap();
        let ready = f._dir.path().join("u5-trace/ready");
        wait(
            || std::fs::read_to_string(&ready).is_ok_and(|v| v.parse::<i32>().is_ok()),
            "helper did not reach stop barrier",
        )
        .await;
        let job = f.runtime.phase_jobs.observed_jobs().remove(0);
        let pid = std::fs::read_to_string(&ready)
            .unwrap()
            .parse::<i32>()
            .unwrap();
        let start = tokio::time::Instant::now();
        match cause {
            "abandon" => f
                .runtime
                .phase_jobs
                .observed_preparation_custody(job.unit)
                .unwrap()
                .abandon(),
            "fence" => {
                raw(&f)
                    .execute(
                        "UPDATE runtime_epoch SET epoch=epoch+1 WHERE singleton=1",
                        [],
                    )
                    .unwrap();
            }
            _ => std::fs::write(f._dir.path().join("u5-trace/release"), "release").unwrap(),
        }
        if cause == "fence" {
            // Retiring the runtime owner also forbids SQL settlement. Observe
            // the actual sleeping child disappear, while its row stays Held.
            let pid = rustix::process::Pid::from_raw(pid).unwrap();
            wait(
                || rustix::process::getpgid(Some(pid)) == Err(rustix::io::Errno::SRCH),
                "fenced helper was not reaped",
            )
            .await;
            assert!(
                start.elapsed() < Duration::from_secs(3),
                "fence received self-exit grace"
            );
            assert!(
                effects(&f, job.unit)
                    .iter()
                    .any(|e| e.kind == "native_phase_schema"
                        && e.state == crate::execution::EffectState::Pending
                        && e.receipt.is_empty())
            );
            no_session(&f);
            let _ = f.runtime.shutdown().await;
            finish(f).await;
            continue;
        }
        wait(
            || {
                effects(&f, job.unit)
                    .iter()
                    .any(|e| e.kind == "native_phase_schema" && e.receipt.contains_key("exit"))
            },
            "stopped helper did not retain its physical receipt",
        )
        .await;
        let rows = effects(&f, job.unit);
        let schema = rows
            .iter()
            .find(|e| e.kind == "native_phase_schema")
            .unwrap();
        assert_eq!(schema.receipt["exit"], "signal:9", "{cause}");
        assert_eq!(
            schema.receipt["capture"],
            "stdout_eof=true;stderr_eof=true;leader_reaped=true;stop=forced_signal",
            "{cause}"
        );
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "{cause} received self-exit grace"
        );
        assert_eq!(schema.receipt["expectation"], "mismatch");
        no_session(&f);
        let _ = f.runtime.shutdown().await;
        finish(f).await;
    }
}

/// Completed and refused Handshakes retain the grace and record its signal separately from exit.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5r1a_handshake_completion_and_refusal_keep_signal_grace() {
    for case in ["handshake-signal", "stop-refusal"] {
        let mut f = fixture_p(case);
        f.register_real_git_project();
        let _ = accept(&f, 1).await;
        let (a, r) = park(&f, PreparationObservation::BeforeCommand);
        f.runtime.start().await.unwrap();
        let ready = f._dir.path().join("u5-trace/ready");
        wait(|| ready.exists(), "Handshake response barrier absent").await;
        // Both peers emit completion/refusal immediately after this release.
        let start = tokio::time::Instant::now();
        std::fs::write(f._dir.path().join("u5-trace/release"), "release").unwrap();
        wait(
            || {
                f.runtime.phase_jobs.observed_jobs().iter().any(|j| {
                    effects(&f, j.unit).iter().any(|e| {
                        e.kind == "native_phase_handshake" && e.receipt.contains_key("exit")
                    })
                })
            },
            "Handshake did not settle",
        )
        .await;
        assert!(start.elapsed() >= Duration::from_secs(5));
        if case == "handshake-signal" {
            arrive(&f, &a).await;
        }
        let job = f.runtime.phase_jobs.observed_jobs().remove(0);
        let rows = effects(&f, job.unit);
        let handshake = rows
            .iter()
            .find(|e| e.kind == "native_phase_handshake")
            .unwrap();
        assert_eq!(handshake.receipt["exit"], "signal:9");
        assert!(handshake.receipt["capture"].ends_with("stop=grace_signal"));
        assert_eq!(
            handshake.receipt["expectation"],
            if case == "handshake-signal" {
                "match"
            } else {
                "mismatch"
            }
        );
        assert!(
            handshake.receipt.len() <= 16
                && handshake
                    .receipt
                    .iter()
                    .all(|(k, v)| k.len() <= 128 && v.len() <= 128)
        );
        no_session(&f);
        r.add_permits(1);
        let _ = f.runtime.shutdown().await;
        finish(f).await;
    }
}

/// Monitoring failures and a foreign process group do not become an active helper's IPC evidence.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5r1a_ipc_monitor_and_foreign_group_are_not_helper_requests() {
    let mut f = fixture_p("ipc-wait");
    f.register_real_git_project();
    let _ = accept(&f, 1).await;
    let (a, r) = park(&f, PreparationObservation::BeforeCommand);
    f.runtime.start().await.unwrap();
    let ready = f._dir.path().join("u5-trace/ready");
    wait(|| ready.exists(), "Schema IPC barrier absent").await;
    let job = f.runtime.phase_jobs.observed_jobs().remove(0);
    let unit = f
        .owner
        .store
        .lock()
        .unwrap()
        .execution_unit(job.unit)
        .unwrap();
    for _ in 0..33 {
        assert!(crate::execution::ipc::revalidate_preparation_for_test(&f.owner, &unit).is_err());
    }
    // This authenticated request originates in the test process, outside the helper group.
    let response = retired_request(&f, &unit).await;
    assert_eq!(response["type"], "refused");
    std::fs::write(f._dir.path().join("u5-trace/release"), "release").unwrap();
    arrive(&f, &a).await;
    let rows = effects(&f, job.unit);
    let schema = rows
        .iter()
        .find(|e| e.kind == "native_phase_schema")
        .unwrap();
    assert_eq!(schema.receipt["ipc_marker_refused"], "1");
    assert_eq!(schema.receipt["ipc_other_refused"], "0");
    assert_eq!(schema.receipt["ipc_invalid"], "false");
    r.add_permits(1);
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

/// Schema creation is audited as not cleaned even when Schema qualification refuses.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5r1a_schema_refusal_retains_audited_directory() {
    let mut f = fixture_p("schema-fail");
    f.register_real_git_project();
    let (_, tasks) = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    wait(
        || {
            f.runtime.phase_jobs.observed_jobs().iter().any(|j| {
                effects(&f, j.unit)
                    .iter()
                    .any(|e| e.kind == "native_phase_schema" && e.receipt.contains_key("exit"))
            })
        },
        "Schema refusal did not settle",
    )
    .await;
    let job = f.runtime.phase_jobs.observed_jobs().remove(0);
    let unit = f
        .owner
        .store
        .lock()
        .unwrap()
        .execution_unit(job.unit)
        .unwrap();
    let profile = crate::execution::resources::ResourceManager::new(f.owner.clone())
        .profile(&unit)
        .unwrap();
    assert!(std::fs::read_dir(&profile.temp).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("p-schema-")
    }));
    let events = f
        .owner
        .store
        .lock()
        .unwrap()
        .events(&tasks[0].scope(), 0, 10000)
        .unwrap();
    let cleanups: Vec<_> = events
        .iter()
        .filter(|e| e.kind == "native.phase.schema_directory")
        .map(|e| {
            assert_eq!(e.data["unit"], serde_json::json!(job.unit));
            assert_eq!(e.data.as_object().unwrap().len(), 2);
            e.data["cleanup"].as_str().unwrap()
        })
        .collect();
    assert_eq!(cleanups, ["not_cleaned"]);
    no_session(&f);
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

/// A failed residual audit prevents Schema directory creation and helper intent.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u5r2_schema_audit_failure_leaves_no_unrecorded_directory() {
    let mut f = fixture_p("normal");
    f.register_real_git_project();
    accept(&f, 1).await;
    let (arrived, release) = park(&f, PreparationObservation::VersionReady);
    f.runtime.start().await.unwrap();
    arrive(&f, &arrived).await;
    let job = f.runtime.phase_jobs.observed_jobs().remove(0);
    let unit = f
        .owner
        .store
        .lock()
        .unwrap()
        .execution_unit(job.unit)
        .unwrap();
    let profile = crate::execution::resources::ResourceManager::new(f.owner.clone())
        .profile(&unit)
        .unwrap();
    // Failure injection only: no authority or successful helper row is seeded.
    raw(&f)
        .execute_batch("CREATE TRIGGER u5r2_refuse_schema_audit BEFORE INSERT ON audit WHEN NEW.kind='native.phase.schema_directory' BEGIN SELECT RAISE(ABORT,'u5r2 schema audit refused'); END;")
        .unwrap();
    release.add_permits(1);
    wait(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.finished)
        },
        "audit failure did not refuse P",
    )
    .await;
    let job = f.runtime.phase_jobs.observed_jobs().remove(0);
    assert!(job.refusal.unwrap().contains("u5r2 schema audit refused"));
    assert!(std::fs::read_dir(&profile.temp).unwrap().all(|e| {
        !e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("p-schema-")
    }));
    assert!(
        effects(&f, job.unit)
            .iter()
            .all(|e| e.kind != "native_phase_schema")
    );
    assert_eq!(
        std::fs::read_to_string(f._dir.path().join("u5-trace").join(job.unit.to_string())).unwrap(),
        "version\n"
    );
    raw(&f)
        .execute_batch("DROP TRIGGER u5r2_refuse_schema_audit")
        .unwrap();
    no_session(&f);
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
