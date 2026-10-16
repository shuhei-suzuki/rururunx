//! U3 integrated controls: S remains nongrant and all-alias admission stays closed.
use super::*;
use crate::runtime::startup::{ProbeState, StartupRefusal, StartupState};
const PEER: &str = r#"
import json, os, sys, time
with open(TRACE, 'a') as trace:
    trace.write(('version' if sys.argv[1:] == ['--version'] else 'initialize') + '\n')
if sys.argv[1:] == ['--version']:
    if SCENARIO == 'dirty':
        with open('unexpected', 'w') as created: created.write('fixture')
    if SCENARIO == 'program-replace':
        with open(sys.argv[0], 'a') as program: program.write('\n# changed\n')
    if SCENARIO == 'replace':
        cwd = os.getcwd()
        os.rename(cwd, cwd + '-old')
        os.mkdir(cwd, 0o700)
    print('arbitrary CLI 99.1')
    sys.exit(0)
assert sys.argv[-2:] == ['--permission-mode','plan']
for line in sys.stdin:
    value = json.loads(line)
    assert value['type'] == 'control_request'
    assert value['request'] == {'subtype':'initialize','hooks':None}
    response = {'subtype':'success','request_id':value['request_id'],'response':{}}
    if SCENARIO == 'id': response['request_id'] = 'foreign'
    if SCENARIO == 'missing': del response['request_id']
    if SCENARIO == 'error': response = {'subtype':'error','request_id':value['request_id'],'error':'fixture'}
    if SCENARIO == 'notifications':
        while True:
            print(json.dumps({'type':'auth_status','isAuthenticating':False,'output':[]}),flush=True)
            time.sleep(0.02)
    if SCENARIO == 'aggregate':
        terminal = json.dumps({'type':'control_response','response':response}) + '\n'
        auth = {'type':'auth_status','isAuthenticating':False,'output':['']}
        overhead = len(json.dumps(auth)) + 1
        auth['output'][0] = 'x' * (1024*1024 - len('arbitrary CLI 99.1\n') + EXTRA - len(terminal) - overhead)
        print(json.dumps(auth), flush=True)
    if SCENARIO == 'stderr': sys.stderr.write('x' * (1024*1024+1)); sys.stderr.flush()
    if SCENARIO == 'absent': sys.exit(0)
    if SCENARIO != 'absent': print(json.dumps({'type':'control_response','response':response}),flush=True)
    if SCENARIO == 'tail': print('{',flush=True)
    if SCENARIO == 'hold':
        while True: time.sleep(0.02)
"#;
fn s_fixture(scenario: &str) -> ControlFixture {
    fixture_with("codex", true, |config| {
        if scenario == "notifications" {
            config.agents.retain(|alias, _| alias == "rev-a");
        }
        for (alias, agent) in config.agents.iter_mut() {
            let path = std::path::PathBuf::from(&agent.command[0]);
            let trace = path.with_file_name(format!("s-trace-{alias}"));
            let script = format!(
                "#!/usr/bin/python3\nSCENARIO={scenario:?}\nTRACE={:?}\n{PEER}",
                trace.to_str().unwrap()
            );
            // Every alias has its own peer so trace identity cannot overlap.
            let program = path.with_file_name(format!("s-program-{alias}"));
            std::fs::write(&program, script).unwrap();
            std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
            agent.command = vec![program.to_str().unwrap().into()];
        }
    })
}
async fn settled(f: &ControlFixture) -> crate::runtime::startup::StartupStatus {
    let end = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        let s = f.runtime.startup.snapshot().unwrap();
        if !matches!(s.state, StartupState::NotStarted | StartupState::Pending) {
            return s;
        }
        assert!(
            tokio::time::Instant::now() < end,
            "S task did not settle: {s:?}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
/// AC-02/03/08: real Claude plan initialize settles, Codex gaps refuse, Unit effects stay zero.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u3_real_startup_claude_plan_and_codex_refusal_have_zero_execution_effects() {
    let f = s_fixture("success");
    let (_, tasks) = accept(&f, 1).await;
    assert!(
        f.runtime.installed.is_ok(),
        "AC-06: checked installation absent"
    );
    f.runtime.start().await.unwrap();
    let status = settled(&f).await;
    assert_eq!(status.state, StartupState::Refused);
    assert_eq!(
        status.aliases.len(),
        f.runtime.config.agents.len(),
        "S-1: configured alias omitted"
    );
    for alias in &status.aliases {
        assert_eq!(
            alias.steps[0].state,
            ProbeState::Passed,
            "T-1: Version missing"
        );
        if alias.provider == "claude" {
            assert_eq!(
                alias.steps[1].state,
                ProbeState::Passed,
                "S-3: Reviewer plan initialize missing"
            );
        } else {
            assert_eq!(
                alias.steps[1].reason,
                Some(StartupRefusal::CodexSchemaUnavailable)
            );
            assert_eq!(
                alias.steps[2].reason,
                Some(StartupRefusal::CodexHandshakeUnavailable)
            );
        }
    }
    // This helper is intentionally first: removing the S gate must reach a real
    // claim and fail the no-effects boundary itself (N-4), not an earlier label.
    let _ = f.runtime.admit_ready_tasks();
    assert_no_execution_effects(&f);
    assert!(f.runtime.installed_driver_composition(&tasks[0]).is_err());
    let response = f
        .runtime
        .handle_control(&f.socket, f.request(ControlAction::RuntimeStatus))
        .await
        .unwrap();
    let wire = serde_json::to_value(response).unwrap();
    let decoded: ControlResponse = serde_json::from_value(wire.clone()).unwrap();
    assert!(matches!(
        decoded,
        ControlResponse::RuntimeMetadata {
            startup: crate::runtime::startup::StartupStatus {
                state: StartupState::Refused,
                ..
            },
            ..
        }
    ));
    let mut old = wire.clone();
    old.as_object_mut().unwrap().remove("startup");
    assert!(
        serde_json::from_value::<ControlResponse>(old).is_err(),
        "C-1: old status accepted"
    );
    let mut unknown = wire;
    unknown["startup"]["unexpected"] = serde_json::json!(true);
    assert!(
        serde_json::from_value::<ControlResponse>(unknown).is_err(),
        "C-1: startup unknown fields accepted"
    );
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}
/// S-3/T-1/MP-2: wrong correlation, missing response, stderr overflow and cwd replacement never pass.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u3_startup_negative_peers_fail_before_any_model_or_unit_effect() {
    for scenario in [
        "id",
        "missing",
        "error",
        "absent",
        "tail",
        "stderr",
        "replace",
        "dirty",
        "program-replace",
    ] {
        let f = s_fixture(scenario);
        f.runtime.start().await.unwrap();
        let status = settled(&f).await;
        for alias in status.aliases.iter().filter(|a| a.provider == "claude") {
            assert_ne!(
                alias.steps[1].state,
                ProbeState::Passed,
                "S-3/MP-2/T-1: {scenario} passed"
            );
        }
        assert_no_execution_effects(&f);
        f.runtime.shutdown().await.unwrap();
        finish(f).await;
    }
}
/// S-4: stop cancels real retained startup children and waits for the S task.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u3_stop_joins_startup_and_keeps_admission_closed() {
    let f = s_fixture("hold");
    f.runtime.start().await.unwrap();
    let end = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let trace = std::path::Path::new(&f.runtime.config.agents["rev-a"].command[0])
            .with_file_name("s-trace-rev-a");
        if std::fs::read_to_string(trace).is_ok_and(|s| s.contains("initialize")) {
            break;
        }
        assert!(tokio::time::Instant::now() < end, "S child not reached");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    f.runtime.shutdown().await.unwrap();
    assert_eq!(
        f.runtime.startup.snapshot().unwrap().state,
        StartupState::Stopped
    );
    assert_no_execution_effects(&f);
    finish(f).await;
}

/// MP-2 root rejection is caused by S itself after Runtime construction, without binding ControlEndpoint.
#[tokio::test]
async fn u3_endpoint_free_start_checks_the_actual_root_descriptor() {
    let f = s_fixture("success");
    std::fs::set_permissions(f.owner.state_root(), std::fs::Permissions::from_mode(0o777)).unwrap();
    f.runtime.start().await.unwrap();
    let status = settled(&f).await;
    assert_eq!(
        status.reason,
        Some(StartupRefusal::PrivatePath),
        "MP-2: S did not reject the root"
    );
    assert!(!f.owner.state_root().join("startup").exists());
    assert_no_execution_effects(&f);
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}

/// S-4 Runtime Drop revokes an actual running S child and retains custody until its task exits.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u3_drop_drains_the_actual_startup_custody() {
    let f = s_fixture("hold");
    f.runtime.start().await.unwrap();
    let end = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let trace = std::path::Path::new(&f.runtime.config.agents["rev-a"].command[0])
            .with_file_name("s-trace-rev-a");
        if std::fs::read_to_string(trace).is_ok_and(|s| s.contains("initialize")) {
            break;
        }
        assert!(tokio::time::Instant::now() < end, "S child not reached");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let retained = f.runtime.startup.clone();
    let custody = Arc::downgrade(&retained);
    finish(f).await;
    assert!(
        retained.is_revoked(),
        "S-4: Drop did not revoke actual S custody"
    );
    drop(retained);
    success::wait_for(
        || custody.upgrade().is_none(),
        "S-4: Drop lost retained task/child custody",
        10,
    )
    .await;
}

/// T-1 charges all aliases into one 8 MiB capture budget, allowing the boundary and refusing +1.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u3_aggregate_capture_boundary_is_shared_by_all_aliases() {
    for extra in [0, 1] {
        let f = fixture_with("codex", true, |config| {
            let mut agent = config.agents["rev-a"].clone();
            let parent = std::path::Path::new(&agent.command[0])
                .parent()
                .unwrap()
                .to_path_buf();
            config.agents.clear();
            for i in 0..8 {
                let alias = format!("review-{i}");
                let program = parent.join(format!("aggregate-program-{i}"));
                let script = format!(
                    "#!/usr/bin/python3\nSCENARIO='aggregate'\nEXTRA={}\nTRACE={:?}\n{PEER}",
                    if i == 7 { extra } else { 0 },
                    parent.join(format!("trace-{i}")).to_str().unwrap()
                );
                std::fs::write(&program, script).unwrap();
                std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
                agent.command = vec![program.to_str().unwrap().into()];
                config.agents.insert(alias, agent.clone());
            }
        });
        f.runtime.start().await.unwrap();
        let status = settled(&f).await;
        assert_eq!(status.aliases.len(), 8);
        let last = &status.aliases[7].steps[1];
        if extra == 0 {
            assert!(
                status
                    .aliases
                    .iter()
                    .flat_map(|a| &a.steps)
                    .all(|s| s.state == ProbeState::Passed),
                "T-1: exact aggregate boundary refused: {status:?}"
            );
        } else {
            assert_ne!(last.state, ProbeState::Passed, "T-1: aggregate +1 accepted");
            assert_eq!(last.reason, Some(StartupRefusal::Budget));
        }
        let bytes: u64 = raw(&f)
            .query_row(
                "SELECT sum(json_extract(body,'$.observation.bytes')) FROM runtime_startup_probes",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            bytes,
            8 * 1024 * 1024 + extra as u64,
            "T-1: capture counter missed bytes"
        );
        assert_no_execution_effects(&f);
        f.runtime.shutdown().await.unwrap();
        finish(f).await;
    }
}

/// N-1 binds the saved-Task refusal to the installed native port even if config later diverges.
#[tokio::test]
async fn u3_saved_executor_provider_comes_from_selected_port() {
    let f = s_fixture("success");
    let (_, tasks) = accept(&f, 1).await;
    let mut f = f
        .restart_with_config(|config| {
            config.agents.get_mut("worker").unwrap().provider = Some("claude".into());
        })
        .await;
    // Only advisory config changes. The original installed Native port remains Claude.
    Arc::get_mut(&mut f.runtime)
        .expect("SETUP: unique unstarted Runtime")
        .config
        .agents
        .get_mut("worker")
        .unwrap()
        .provider = Some("codex".into());
    f.runtime.start().await.unwrap();
    let guard = f.runtime.control_admission.lock().await;
    let error = f
        .runtime
        .installed_driver_composition(&tasks[0])
        .err()
        .expect("N-1: Claude selected port accepted");
    assert_eq!(
        error.downcast_ref::<crate::execution::capability::CapabilityRefusal>(),
        Some(&crate::execution::capability::CapabilityRefusal::ClaudeExecutorUnavailable),
        "N-1: config replaced the selected provider"
    );
    assert_no_execution_effects(&f);
    drop(guard);
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}

/// T-1 legal pre-initialize notifications do not renew the fixed 30-second response deadline.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u3_notifications_cannot_refresh_initialize_deadline() {
    let f = s_fixture("notifications");
    f.runtime.start().await.unwrap();
    let end = tokio::time::Instant::now() + Duration::from_secs(35);
    loop {
        let status = f.runtime.startup.snapshot().unwrap();
        if matches!(status.state, StartupState::Refused | StartupState::Unknown) {
            assert_eq!(status.aliases.len(), 1);
            assert_eq!(
                status.aliases[0].steps[1].reason,
                Some(StartupRefusal::Deadline)
            );
            assert_ne!(status.aliases[0].steps[1].state, ProbeState::Passed);
            break;
        }
        assert!(
            tokio::time::Instant::now() < end,
            "T-1: initialize deadline renewed by notifications"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_no_execution_effects(&f);
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}
