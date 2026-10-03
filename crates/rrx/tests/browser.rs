use rrx::browser::Usage;
use rrx::{browser::*, domain::*, git::WorktreeManager, state::Store};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use tempfile::TempDir;

struct Fixture {
    temp: TempDir,
    store: Store,
    binding: BrowserBinding,
}
fn git(path: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .current_dir(path)
            .args(args)
            .output()
            .unwrap()
            .status
            .success()
    );
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap().join("repo");
        fs::create_dir(&root).unwrap();
        git(&root, &["init", "-b", "main"]);
        for (key, value) in [
            ("user.email", "fixture@example.invalid"),
            ("user.name", "Fixture"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", ".git/hooks"),
        ] {
            git(&root, &["config", key, value]);
        }
        fs::write(root.join("tracked"), "fixture").unwrap();
        git(&root, &["add", "tracked"]);
        git(&root, &["commit", "-m", "fixture"]);
        let mut store = Store::memory().unwrap();
        let mut project = Project::new(
            "fixture".into(),
            root.clone(),
            rrx::git::repository_identity(&root, "main").unwrap(),
            "main".into(),
        );
        store.put_project(&mut project).unwrap();
        let mut goal = Goal::new(
            project.id,
            "fixture".into(),
            vec![CompletionCriterion {
                id: "done".into(),
                description: "fixture".into(),
                evidence: None,
                satisfied: false,
            }],
        );
        store.put_goal(&mut goal).unwrap();
        let mut task = Task::new(project.id, goal.id, "browser fixture".into(), "fake".into());
        task.issue = Some(31);
        store.put_task(&mut task).unwrap();
        WorktreeManager::create(&mut store, task.id).unwrap();
        let binding = BrowserBinding::capture(&store, &task.scope()).unwrap();
        Self {
            temp,
            store,
            binding,
        }
    }
    fn request(&self) -> VerificationRequest {
        VerificationRequest {
            scope: self.binding.task().scope(),
            allowed_origins: vec!["http://127.0.0.1:12345".into()],
            steps: vec![
                Step::Navigate {
                    url: "http://127.0.0.1:12345/".into(),
                },
                Step::ReadText {
                    selector: "#status".into(),
                },
            ],
            deterministic_fallback: None,
        }
    }
    fn config(&self, script: &str) -> BrowserConfig {
        let path = self.temp.path().join("bridge.py");
        fs::write(&path, script).unwrap();
        BrowserConfig {
            bridge_command: vec!["python3".into(), path.to_string_lossy().into_owned()],
            artifact_root: self.temp.path().join("artifacts"),
            timeout_ms: 500,
            step_timeout_ms: 100,
            max_output_bytes: 2048,
            ..Default::default()
        }
    }
}
fn adaptive() -> Step {
    Step::Observe {
        instruction: "Find the status".into(),
        scope_selector: "#status".into(),
    }
}
fn result(request: &VerificationRequest, failure: Failure, effect: bool) -> VerificationResult {
    VerificationResult {
        scope: request.scope.clone(),
        session_id: SessionId::new(),
        backend: Backend::Stagehand,
        success: false,
        failure: Some(failure),
        effect_possible: effect,
        evidence: vec![],
        artifacts: vec![],
        usage: Usage::default(),
        fallback_used: false,
        artifact_directory: None,
        verification_file: None,
        worktree: None,
        revision: None,
    }
}
#[test]
fn routing_never_spends_model_tokens_for_known_flows() {
    let mut config = BrowserConfig::default();
    let deterministic = vec![Step::ReadText {
        selector: "#status".into(),
    }];
    for backend in [Backend::Auto, Backend::Playwright, Backend::Stagehand] {
        config.backend = backend;
        assert_eq!(route(&config, &deterministic).unwrap(), Backend::Playwright);
    }
    config.backend = Backend::Auto;
    assert_eq!(route(&config, &[adaptive()]).unwrap(), Backend::Stagehand);
    config.backend = Backend::Playwright;
    assert!(route(&config, &[adaptive()]).is_err());
}
#[test]
fn fallback_does_not_replay_failed_assertions_or_uncertain_actions() {
    let fixture = Fixture::new();
    let mut request = fixture.request();
    request.steps.push(adaptive());
    request.deterministic_fallback = Some(fixture.request().steps);
    let mut config = BrowserConfig::default();
    assert!(may_fallback(
        &config,
        &request,
        &result(&request, Failure::Unavailable, false)
    ));
    for failure in [
        Failure::Assertion,
        Failure::Operation,
        Failure::PolicyHold,
        Failure::Timeout,
        Failure::Protocol,
    ] {
        assert!(!may_fallback(
            &config,
            &request,
            &result(&request, failure, false)
        ));
    }
    assert!(!may_fallback(
        &config,
        &request,
        &result(&request, Failure::Unavailable, true)
    ));
    config.safe_fallback = false;
    assert!(!may_fallback(
        &config,
        &request,
        &result(&request, Failure::Unavailable, false)
    ));
}
#[test]
fn scope_and_policy_hold_prevent_process_launch() {
    let fixture = Fixture::new();
    let other = Fixture::new();
    let mut request = fixture.request();
    let verifier = BridgeVerifier {
        config: fixture.config("raise RuntimeError('must not start')"),
    };
    assert!(verifier.verify(&other.binding, &request).is_err());
    assert!(BrowserBinding::capture(&fixture.store, &other.binding.task().scope()).is_err());
    request.steps.push(Step::Click {
        selector: "button".into(),
    });
    assert_eq!(
        verifier.verify(&fixture.binding, &request).unwrap().failure,
        Some(Failure::PolicyHold)
    );
    let mut verifier = verifier;
    verifier.config.allow_loopback_actions = true;
    request.allowed_origins = vec!["http://127.0.0.1.evil.example".into()];
    assert_eq!(
        verifier.verify(&fixture.binding, &request).unwrap().failure,
        Some(Failure::PolicyHold)
    );
    assert!(!verifier.config.artifact_root.exists());
}
#[test]
fn bounded_process_failure_is_normalized_and_descendants_are_owned() {
    let fixture = Fixture::new();
    let request = fixture.request();
    let child_pid_file = fixture.temp.path().join("owned-child.pid");
    let timeout_script = format!(
        "import subprocess,sys,time\nchild=subprocess.Popen([sys.executable,'-c','import signal,time;signal.signal(signal.SIGTERM,signal.SIG_IGN);time.sleep(20)'])\nopen({},'w').write(str(child.pid))\ntime.sleep(20)\n",
        serde_json::to_string(&child_pid_file).unwrap()
    );
    let timeout = BridgeVerifier {
        config: fixture.config(&timeout_script),
    }
    .verify(&fixture.binding, &request)
    .unwrap();
    assert_eq!(timeout.failure, Some(Failure::Timeout));
    assert!(
        timeout.evidence.last().unwrap()["browser_phase_ms"]
            .as_u64()
            .unwrap()
            < 3000
    );
    let child_pid = fs::read_to_string(child_pid_file).unwrap();
    let state = Command::new("/bin/ps")
        .args(["-p", child_pid.trim(), "-o", "stat="])
        .output()
        .unwrap();
    let state = String::from_utf8(state.stdout).unwrap();
    assert!(state.trim().is_empty() || state.trim().starts_with('Z'));
    let overflow = BridgeVerifier {
        config: fixture.config("print('x' * 10000)\n"),
    }
    .verify(&fixture.binding, &request)
    .unwrap();
    assert_eq!(overflow.failure, Some(Failure::OutputLimit));
    let invalid = BridgeVerifier {
        config: fixture.config("print('{bad}')\n"),
    }
    .verify(&fixture.binding, &request)
    .unwrap();
    assert_eq!(invalid.failure, Some(Failure::Protocol));
    let foreign = BridgeVerifier { config: fixture.config("import json,sys\ni=json.load(sys.stdin)\nprint(json.dumps({'scope':{}, 'backend':'playwright'}))\n") }.verify(&fixture.binding, &request).unwrap();
    assert_eq!(foreign.failure, Some(Failure::Protocol));
}
const SUCCESS_BRIDGE: &str = r#"import json,sys,os
i=json.load(sys.stdin)
r={'scope':i['request']['scope'],'session_id':i['session_id'],'backend':i['backend'],'success':True,'failure':None,'effect_possible':False,'evidence':[{'cwd':os.getcwd(),'owned_group':os.getpid()==os.getpgrp()}],'artifacts':[],'usage':{},'fallback_used':False}
print(json.dumps(r))
"#;
#[test]
fn private_group_cwd_and_scoped_evidence_are_checked() {
    let fixture = Fixture::new();
    let request = fixture.request();
    let verifier = BridgeVerifier {
        config: fixture.config(SUCCESS_BRIDGE),
    };
    let first = verifier.verify(&fixture.binding, &request).unwrap();
    let second = verifier.verify(&fixture.binding, &request).unwrap();
    assert!(first.success, "{first:?}");
    assert_ne!(first.session_id, second.session_id);
    assert_eq!(first.evidence[0]["owned_group"], json!(true));
    assert_eq!(
        first.evidence[0]["cwd"],
        json!(
            fixture
                .binding
                .task()
                .worktree
                .as_ref()
                .unwrap()
                .to_string_lossy()
        )
    );
    let record = verification_record(&first).unwrap();
    assert_eq!(record.scope, request.scope);
    assert_eq!(record.kind, RecordKind::Verification);
    assert!(first.usage.input_tokens.is_none());
    let path = verifier
        .config
        .artifact_root
        .join(request.scope.project_id.to_string())
        .join(request.scope.task_id.unwrap().to_string())
        .join(first.session_id.to_string());
    assert!(path.join("verification.json").is_file());
    assert!(!path.join("profile").exists());
}
#[cfg(unix)]
#[test]
fn artifact_ancestor_symlinks_cannot_cross_projects() {
    let fixture = Fixture::new();
    let request = fixture.request();
    let config = fixture.config(SUCCESS_BRIDGE);
    fs::create_dir(&config.artifact_root).unwrap();
    let foreign = fixture.temp.path().join("foreign");
    fs::create_dir(&foreign).unwrap();
    std::os::unix::fs::symlink(
        &foreign,
        config
            .artifact_root
            .join(request.scope.project_id.to_string()),
    )
    .unwrap();
    assert!(
        BridgeVerifier { config }
            .verify(&fixture.binding, &request)
            .is_err()
    );
    assert_eq!(fs::read_dir(foreign).unwrap().count(), 0);
}

#[test]
fn project_overlay_tightens_policy_and_cannot_supply_executables() {
    let mut config = BrowserConfig {
        headed: true,
        safe_fallback: false,
        ..Default::default()
    };
    config.apply_project(BrowserOverlay {
        backend: Some(Backend::Playwright),
        headed: Some(false),
        safe_fallback: Some(true),
        timeout_ms: Some(900_000),
        step_timeout_ms: Some(1000),
    });
    assert_eq!(config.backend, Backend::Playwright);
    assert!(config.headed && !config.safe_fallback);
    assert_eq!(config.timeout_ms, 120_000);
    assert_eq!(config.step_timeout_ms, 1000);
    assert!(
        serde_json::from_value::<BrowserOverlay>(json!({"bridge_command":["other-provider"]}))
            .is_err()
    );
    assert!(
        serde_json::from_value::<BrowserOverlay>(json!({"allow_loopback_actions":true})).is_err()
    );
}

#[test]
fn provider_baseline_does_not_leak_another_projects_scoped_api_key() {
    if std::env::var_os("RRX_BROWSER_ENV_FIXTURE_CHILD").is_none() {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "provider_baseline_does_not_leak_another_projects_scoped_api_key",
            ])
            .env("RRX_BROWSER_ENV_FIXTURE_CHILD", "true")
            .env("ANTHROPIC_API_KEY", "synthetic-other-project-key")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        return;
    }
    let mut fixture = Fixture::new();
    let other = Fixture::new();
    let mut project = Project::new(
        "other".into(),
        other.binding.project().root.clone(),
        other.binding.project().repository_identity.clone(),
        other.binding.project().base_branch.clone(),
    );
    project.environment_refs = vec!["ANTHROPIC_API_KEY".into()];
    fixture.store.put_project(&mut project).unwrap();
    fixture.binding =
        BrowserBinding::capture(&fixture.store, &fixture.binding.task().scope()).unwrap();
    let request = fixture.request();
    let script = SUCCESS_BRIDGE.replace(
        "'owned_group':os.getpid()==os.getpgrp()",
        "'key_present':'ANTHROPIC_API_KEY' in os.environ",
    );
    let verifier = BridgeVerifier {
        config: fixture.config(&script),
    };
    let result = verifier.verify(&fixture.binding, &request).unwrap();
    assert_eq!(result.evidence[0]["key_present"], json!(false));
    let mut current = fixture.binding.project().clone();
    current.environment_refs = vec!["ANTHROPIC_API_KEY".into()];
    fixture.store.put_project(&mut current).unwrap();
    fixture.binding = BrowserBinding::capture(&fixture.store, &request.scope).unwrap();
    let result = verifier.verify(&fixture.binding, &request).unwrap();
    assert_eq!(result.evidence[0]["key_present"], json!(true));
}

#[test]
fn arbitrary_external_connection_is_typed_unsupported_and_profiles_are_ephemeral() {
    let fixture = Fixture::new();
    let request = fixture.request();
    let verifier = BridgeVerifier {
        config: fixture.config("raise RuntimeError('external browser must never be contacted')"),
    };
    assert_eq!(
        verifier
            .connect_external(&fixture.binding, &request, "http://127.0.0.1:9222")
            .unwrap()
            .failure,
        Some(Failure::Unsupported)
    );
    let script = "import json,sys,os,time\ni=json.load(sys.stdin)\np=os.path.join(i['artifact_dir'],'profile')\nos.mkdir(p)\nopen(os.path.join(p,'Cookies'),'w').write('synthetic-cookie')\ntime.sleep(20)\n";
    let verifier = BridgeVerifier {
        config: fixture.config(script),
    };
    let result = verifier.verify(&fixture.binding, &request).unwrap();
    assert_eq!(result.failure, Some(Failure::Timeout));
    assert!(!result.artifact_directory.unwrap().join("profile").exists());
}
#[test]
fn explicit_fallback_is_exercised_only_before_effects() {
    let fixture = Fixture::new();
    let mut request = fixture.request();
    request.steps.push(adaptive());
    request.deterministic_fallback = Some(fixture.request().steps);
    let script = SUCCESS_BRIDGE.replace("'success':True,'failure':None", "'success':i['backend']=='playwright','failure':'unavailable' if i['backend']=='stagehand' else None");
    let verifier = BridgeVerifier {
        config: fixture.config(&script),
    };
    let result = verifier.verify(&fixture.binding, &request).unwrap();
    assert!(result.success && result.fallback_used, "{result:?}");
    assert_eq!(result.backend, Backend::Playwright);
    request.deterministic_fallback = Some(vec![Step::Click {
        selector: "button".into(),
    }]);
    assert!(verifier.verify(&fixture.binding, &request).is_err());
}

#[test]
fn fallback_attempts_share_one_total_deadline() {
    let fixture = Fixture::new();
    let mut request = fixture.request();
    request.steps.push(adaptive());
    request.deterministic_fallback = Some(fixture.request().steps);
    let script = SUCCESS_BRIDGE.replace("import json,sys,os", "import json,sys,os,time")
        .replace("i=json.load(sys.stdin)", "i=json.load(sys.stdin)\ntime.sleep(.45 if i['backend']=='stagehand' else 1.75)")
        .replace("'success':True,'failure':None", "'success':i['backend']=='playwright','failure':'unavailable' if i['backend']=='stagehand' else None");
    let mut config = fixture.config(&script);
    config.timeout_ms = 2000;
    let result = BridgeVerifier { config }
        .verify(&fixture.binding, &request)
        .unwrap();
    assert!(result.fallback_used, "{result:?}");
    assert_eq!(result.failure, Some(Failure::Timeout));
}

#[test]
fn post_attempt_failures_preserve_effects_usage_and_scoped_results() {
    for case in ["artifact", "binding", "record", "sessions"] {
        let fixture = Fixture::new();
        let mut request = fixture.request();
        request.steps.push(Step::Click {
            selector: "button".into(),
        });
        let script = SUCCESS_BRIDGE.replace("'effect_possible':False", "'effect_possible':True");
        let extra = match case {
            "artifact" => "r['artifacts']=['missing.png']",
            "binding" => "os.rename(os.getcwd(),os.getcwd()+'.moved')",
            "record" => "os.mkdir(os.path.join(i['artifact_dir'],'verification.json'))",
            _ => {
                "r['usage']['native_sessions']=[str(__import__('uuid').uuid4()) for _ in range(65)]"
            }
        };
        let script = script.replace("print(json.dumps(r))", &format!(
            "r['usage']={{'llm_calls':1,'input_tokens':7,'native_sessions':['00000000-0000-4000-8000-000000000001']}}\n{extra}\nprint(json.dumps(r))"));
        let mut config = fixture.config(&script);
        config.allow_loopback_actions = true;
        config.max_output_bytes = 16384;
        let result = BridgeVerifier { config }
            .verify(&fixture.binding, &request)
            .unwrap();
        assert!(!result.success, "{case}: {result:?}");
        assert!(result.effect_possible);
        assert_eq!(result.usage.input_tokens, Some(7), "{case}: {result:?}");
        assert_eq!(result.scope, request.scope);
        if case == "record" {
            assert_eq!(result.failure, Some(Failure::Cleanup));
            assert!(result.verification_file.is_none());
        } else {
            assert_eq!(result.failure, Some(Failure::Protocol));
            let stored: VerificationResult =
                serde_json::from_slice(&fs::read(result.verification_file.unwrap()).unwrap())
                    .unwrap();
            assert!(stored.effect_possible && !stored.success);
            assert_eq!(
                stored.usage.native_sessions.len(),
                if case == "sessions" { 65 } else { 1 }
            );
        }
    }
}

#[test]
fn fallback_clears_primary_profile_and_retains_all_attempt_telemetry() {
    let fixture = Fixture::new();
    let mut request = fixture.request();
    request.steps.push(adaptive());
    request.deterministic_fallback = Some(fixture.request().steps);
    let script = SUCCESS_BRIDGE.replace("print(json.dumps(r))", r#"
if i['backend']=='stagehand':
 p=os.path.join(i['artifact_dir'],'profile');os.mkdir(p)
 open(os.path.join(p,'Cookies'),'w').write('fixture-cookie')
 open(os.path.join(i['artifact_dir'],'primary.png'),'w').write('fixture-artifact')
 r.update(success=False,failure='unsupported',artifacts=['primary.png'])
 r['usage']={'llm_calls':2,'input_tokens':7,'cost_usd':.5,'native_sessions':['00000000-0000-4000-8000-000000000001']}
else:
 assert not os.path.exists(os.path.join(i['artifact_dir'],'profile'))
 assert not os.path.exists(os.path.join(os.path.dirname(i['artifact_dir']),'profile'))
 p=os.path.join(i['artifact_dir'],'profile');os.mkdir(p)
 open(os.path.join(p,'Cookies'),'w').write('fallback-cookie')
 open(os.path.join(i['artifact_dir'],'fallback.png'),'w').write('fixture-artifact')
 r['artifacts']=['fallback.png'];r['usage']={'llm_calls':0}
print(json.dumps(r))
"#);
    let result = BridgeVerifier {
        config: fixture.config(&script),
    }
    .verify(&fixture.binding, &request)
    .unwrap();
    assert!(result.success && result.fallback_used, "{result:?}");
    assert_eq!(result.usage.llm_calls, Some(2));
    assert_eq!(result.usage.native_sessions.len(), 1);
    assert_eq!(result.usage.cost_usd, None);
    assert_eq!(result.evidence[0]["usage"]["cost_usd"], json!(0.5));
    assert!(
        result
            .artifact_directory
            .as_ref()
            .unwrap()
            .join("fallback.png")
            .is_file()
    );
    assert!(
        !result
            .artifact_directory
            .as_ref()
            .unwrap()
            .join("profile")
            .exists()
    );
    let record = result.verification_file.unwrap();
    assert!(record.parent().unwrap().join("primary.png").is_file());
    assert!(!record.parent().unwrap().join("profile").exists());
}

#[test]
fn fallback_authorities_and_primary_evidence_fail_closed() {
    let fixture = Fixture::new();
    let mut request = fixture.request();
    request.steps.push(adaptive());
    request.deterministic_fallback = Some(vec![Step::Click {
        selector: "#write".into(),
    }]);
    assert!(
        BridgeVerifier {
            config: fixture.config(SUCCESS_BRIDGE)
        }
        .verify(&fixture.binding, &request)
        .is_err()
    );
    request.deterministic_fallback = Some(fixture.request().steps);
    for artifact in ["../foreign.png", "missing.png"] {
        let script = SUCCESS_BRIDGE.replace("print(json.dumps(r))", &format!(
            "assert i['backend']=='stagehand'\nr.update(success=False,failure='unsupported',artifacts=[{artifact:?}])\nr['usage']={{'llm_calls':2}}\nprint(json.dumps(r))"));
        let result = BridgeVerifier {
            config: fixture.config(&script),
        }
        .verify(&fixture.binding, &request)
        .unwrap();
        assert_eq!(result.failure, Some(Failure::Protocol));
        assert!(!result.fallback_used);
        assert!(result.artifacts.is_empty());
        assert_eq!(result.usage.llm_calls, Some(2));
    }
    let script = SUCCESS_BRIDGE.replace("print(json.dumps(r))", "assert i['backend']=='stagehand'\nos.mkdir(os.path.join(i['artifact_dir'],'fallback'))\nr.update(success=False,failure='unsupported')\nr['usage']={'llm_calls':2}\nprint(json.dumps(r))");
    let result = BridgeVerifier {
        config: fixture.config(&script),
    }
    .verify(&fixture.binding, &request)
    .unwrap();
    assert_eq!(result.failure, Some(Failure::Cleanup));
    assert_eq!(result.usage.llm_calls, Some(2));
    assert!(!result.fallback_used);
}

#[test]
fn timeout_retains_a_verified_graceful_terminal_result() {
    let fixture = Fixture::new();
    let script = SUCCESS_BRIDGE.replace("print(json.dumps(r))", r#"
import signal,time
r['usage']={'llm_calls':1,'input_tokens':7,'native_session_attempts':['00000000-0000-4000-8000-000000000001']}
r.update(success=False,failure='timeout',effect_possible=True)
def stop(signum,frame):
 print(json.dumps(r),flush=True);sys.exit(0)
signal.signal(signal.SIGTERM,stop)
time.sleep(20)
"#);
    let result = BridgeVerifier {
        config: fixture.config(&script),
    }
    .verify(&fixture.binding, &fixture.request())
    .unwrap();
    assert_eq!(result.failure, Some(Failure::Timeout));
    assert!(result.effect_possible);
    assert_eq!(result.usage.input_tokens, Some(7));
    assert_eq!(result.usage.native_session_attempts.len(), 1);
    assert!(result.verification_file.unwrap().is_file());
}

#[test]
fn escaped_pipe_holder_cannot_wedge_the_bounded_supervisor() {
    let fixture = Fixture::new();
    let script = "import json,sys,os,subprocess,time\ni=json.load(sys.stdin)\nc=subprocess.Popen([sys.executable,'-c','import time;time.sleep(3)'],start_new_session=True)\nopen(os.path.join(i['artifact_dir'],'escaped.pid'),'w').write(str(c.pid))\ntime.sleep(20)\n";
    let result = BridgeVerifier {
        config: fixture.config(script),
    }
    .verify(&fixture.binding, &fixture.request())
    .unwrap();
    assert!(
        result.evidence.last().unwrap()["browser_phase_ms"]
            .as_u64()
            .unwrap()
            < 3000
    );
    assert_eq!(result.failure, Some(Failure::Cleanup));
    // The intentionally contract-breaking synthetic child self-expires; inspect
    // only its exact fixture PID, never stop a shared/global process.
    let pid = fs::read_to_string(result.artifact_directory.unwrap().join("escaped.pid")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let state = Command::new("/bin/ps")
            .args(["-p", pid.trim(), "-o", "stat="])
            .output()
            .unwrap();
        let state = String::from_utf8(state.stdout).unwrap();
        if state.trim().is_empty() || state.trim().starts_with('Z') {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "owned synthetic child did not expire"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// Real backend, explicit because Chrome and pinned Node SDK installation are host prerequisites.
#[test]
#[ignore = "requires configured Node >=22.18 and local Chrome; run explicitly for browser dogfood"]
fn actual_rust_to_playwright_browser_fixture() {
    actual_rust_fixture(false);
}
#[test]
#[ignore = "requires configured Node/Chrome and explicit native Claude inference authorization"]
fn actual_rust_to_stagehand_native_fixture() {
    actual_rust_fixture(true);
}
fn actual_rust_fixture(adaptive: bool) {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        thread,
    };
    let fixture = Fixture::new();
    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    server.set_nonblocking(true).unwrap();
    let origin = format!("http://{}", server.local_addr().unwrap());
    let stop = Arc::new(AtomicBool::new(false));
    let stopped = stop.clone();
    let handle = thread::spawn(move || {
        while !stopped.load(Ordering::Relaxed) {
            if let Ok((mut stream, _)) = server.accept() {
                stream
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                let mut bytes = [0; 4096];
                let _ = stream.read(&mut bytes);
                let body = "<main><h1 id='status'>Scoped browser fixture</h1></main>";
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }
    });
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let command = vec![
        std::env::var("RRX_BROWSER_NODE").unwrap_or_else(|_| "node".into()),
        manifest
            .join("../../scripts/browser/bridge.mjs")
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
    ];
    let verifier = BridgeVerifier {
        config: BrowserConfig {
            model: adaptive.then(|| ModelConfig {
                model_name: None,
                api_key_env: None,
                custom_command: Some(vec![
                    command[0].clone(),
                    manifest
                        .join("../../scripts/browser/native-claude.mjs")
                        .canonicalize()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                ]),
            }),
            bridge_command: command,
            artifact_root: fixture.temp.path().join("evidence"),
            headed: adaptive,
            ..Default::default()
        },
    };
    let request = VerificationRequest {
        scope: fixture.binding.task().scope(),
        allowed_origins: vec![origin.clone()],
        steps: vec![
            Step::Navigate { url: origin },
            Step::AssertText {
                selector: "#status".into(),
                expected: "Scoped browser fixture".into(),
            },
            Step::Screenshot {
                name: "verified.png".into(),
            },
        ],
        deterministic_fallback: None,
    };
    let mut request = request;
    if adaptive {
        request.steps.insert(2, Step::Extract {instruction: "Extract the displayed heading as title.".into(), scope_selector: "#status".into(),
        schema: json!({"type":"object", "properties":{"title":{"type":"string"}}, "required":["title"], "additionalProperties":false}) });
    }
    let caps = verifier.capabilities(&fixture.binding).unwrap();
    assert_eq!(caps.stagehand_version.as_deref(), Some("4.1.0"));
    let result = verifier.verify(&fixture.binding, &request);
    stop.store(true, Ordering::Relaxed);
    handle.join().unwrap();
    let result = result.unwrap();
    assert!(result.success, "{result:?}");
    if adaptive {
        assert!(result.usage.llm_calls.is_some_and(|n| n > 0));
        assert!(!result.usage.native_sessions.is_empty());
        assert!(result.evidence.iter().any(|e| {
            e.get("data")
                .and_then(|d| d.get("title"))
                .is_some_and(|t| t == "Scoped browser fixture")
        }));
    } else {
        assert_eq!(result.usage.llm_calls, Some(0));
    }
    assert_eq!(result.artifacts, ["verified.png"]);
}
