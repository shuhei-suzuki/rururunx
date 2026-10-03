use rrx::{
    adapter::{grok::GrokAdapter, *},
    domain::*,
    state::Store,
};
use serde_json::{Value, json};
use std::os::unix::fs::PermissionsExt;
use std::{
    collections::BTreeMap,
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};

struct Fixture {
    directory: tempfile::TempDir,
    request: LaunchRequest,
    store: SharedStore,
    executable: std::path::PathBuf,
}
fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .env_clear()
        .envs(std::env::vars_os().filter(|(key, _)| !key.to_string_lossy().starts_with("GIT_")))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("project");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "-b", "main"]);
        std::fs::write(root.join("own.txt"), "owned baseline\n").unwrap();
        git(&root, &["add", "own.txt"]);
        git(
            &root,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-m",
                "fixture",
            ],
        );
        let worktree = root.join("worktree/task");
        git(
            &root,
            &[
                "worktree",
                "add",
                "-b",
                "feature/task",
                worktree.to_str().unwrap(),
            ],
        );
        let root = root.canonicalize().unwrap();
        let worktree = worktree.canonicalize().unwrap();
        let mut project = Project::new(
            "fixture".into(),
            root.clone(),
            rrx::git::repository_identity(&root, "main").unwrap(),
            "main".into(),
        );
        let mut goal = Goal::new(
            project.id,
            "native".into(),
            vec![CompletionCriterion {
                id: "fixture".into(),
                description: "owned ACP edit".into(),
                satisfied: false,
                evidence: None,
            }],
        );
        let mut task = Task::new(project.id, goal.id, "native".into(), "grok".into());
        task.worktree = Some(worktree.clone());
        task.branch = Some("feature/task".into());
        let database = directory.path().join("state.db");
        let mut store = Store::open(&database).unwrap();
        store.put_project(&mut project).unwrap();
        store.put_goal(&mut goal).unwrap();
        store.put_task(&mut task).unwrap();
        let request = LaunchRequest {
            project,
            scope: task.scope(),
            worktree: worktree.clone(),
            role: SessionRole::Executor,
            mode: LaunchMode::NonInteractive,
            input: PreparedInput {
                scope: task.scope(),
                kind: InputKind::ContextPack,
                revision: git(&worktree, &["rev-parse", "HEAD"]),
                version: 1,
                source_versions: BTreeMap::from([("fixture".into(), "v1".into())]),
                payload: "prepared owned fixture".into(),
            },
            environment: BTreeMap::from([
                ("RRX_DATABASE".into(), database.to_str().unwrap().into()),
                (
                    "RRX_FOREIGN".into(),
                    directory
                        .path()
                        .join("foreign.txt")
                        .to_str()
                        .unwrap()
                        .into(),
                ),
            ]),
            model: Some("requested-model".into()),
            effort: Some("low".into()),
        };
        let executable = directory.path().join("fake-grok");
        std::fs::write(&executable, FAKE).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            directory,
            request,
            store: Arc::new(Mutex::new(store)),
            executable,
        }
    }
    fn adapter(&self) -> GrokAdapter {
        GrokAdapter::new("grok".into(), self.executable.clone(), self.store.clone()).unwrap()
    }
    fn mode(&mut self, mode: &str) {
        self.request
            .environment
            .insert("RRX_MODE".into(), mode.into());
    }
    fn review(&mut self) {
        rrx::git::WorktreeManager::lock_review(
            &mut self.store.lock().unwrap(),
            self.request.scope.task_id.unwrap(),
            &self.request.input.revision,
            "native review",
        )
        .unwrap();
        self.request.role = SessionRole::Reviewer;
        self.request.input.kind = InputKind::ReviewBundle;
    }
}
async fn finished(adapter: &GrokAdapter, session: &Session) -> SessionStatus {
    let mut status = adapter.subscribe(session.into()).unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        while !status.borrow().terminal() {
            status.changed().await.unwrap();
        }
        status.borrow().clone()
    })
    .await
    .unwrap()
}
const FAKE: &str = r#"#!/usr/bin/env python3
import json,os,sys,uuid,sqlite3,time,pathlib
mode=os.getenv('RRX_MODE','good');sid=None;prompt=None;calls=0;model='native-default';effort='high'
if os.getenv('RRX_SPAWN_OBSERVED'):pathlib.Path(os.environ['RRX_SPAWN_OBSERVED']).write_text('native process started')
profile=pathlib.Path(sys.argv[sys.argv.index('--agent-profile')+1]).read_text();decision='name: rururunx-decision' in profile
assert 'injectDefaultTools: false' in profile and 'GrokBuild:read_file' in profile and 'GrokBuild:search_replace' in profile
assert 'web_search, x_search, web_fetch' in profile and '--disable-web-search' in sys.argv
assert ('--sandbox' in sys.argv) and sys.argv[sys.argv.index('--sandbox')+1]==('read-only' if decision else 'strict')
def send(v):print(json.dumps(v),flush=True)
def update(v):send({'jsonrpc':'2.0','method':'session/update','params':{'sessionId':sid,'update':v,'_meta':{'promptId':prompt}}})
def fs(method,path,content=None):
 p={'sessionId':sid,'path':path};p.update({'content':content} if content is not None else {})
 send({'jsonrpc':'2.0','id':'callback','method':method,'params':p});return json.loads(sys.stdin.readline())
def tool(name,path,n):
 global calls
 calls+=1;key='target_file' if name=='read_file' else 'file_path'
 update({'sessionUpdate':'tool_call','toolCallId':str(n),'rawInput':{key:path},'_meta':{'x.ai/tool':{'name':name}}})
 return str(n)
def done(n,failed=False):update({'sessionUpdate':'tool_call_update','toolCallId':str(n),'status':'failed' if failed else 'completed'})
for line in sys.stdin:
 d=json.loads(line);method=d.get('method');p=d.get('params',{});result={}
 if not method:continue
 if method=='initialize':
  result={'protocolVersion':1,'agentCapabilities':{'loadSession':True},'authMethods':[{'id':'cached_token'}] if mode!='no_auth_method' else [{'id':'browser'}],'_meta':{'agentVersion':'1.0.46' if mode!='version' else '99.0'}}
 elif method=='authenticate':
  assert p['methodId']=='cached_token' and p['_meta']['headless']
  if mode=='auth_error':send({'jsonrpc':'2.0','id':d['id'],'error':{'code':-1,'message':'not authenticated'}});continue
 elif method in ['session/new','session/load']:
  assert p['cwd']==os.getcwd() and p['mcpServers']==[]
  sid=p.get('sessionId') or str(uuid.uuid4())
  if method=='session/load':update({'sessionUpdate':'agent_message_chunk','content':{'type':'text','text':'REPLAY_MUST_NOT_APPEAR'}})
  result={'sessionId':sid,'configOptions':[{'id':'model','currentValue':model},{'id':'reasoning_effort','currentValue':effort}]}
 elif method=='session/set_config_option':
  if p['configId']=='model':model=p['value']
  else:effort=p['value']
  result={'configOptions':[{'id':'model','currentValue':model if mode!='config' else 'wrong'},{'id':'reasoning_effort','currentValue':effort}]}
 elif method=='_x.ai/session/info':
  if mode=='pause_info' and calls==0:
   pause=pathlib.Path(os.environ['RRX_PAUSE']);pause.write_text('native preflight paused')
   while not pause.with_suffix('.continue').exists():time.sleep(0.01)
  if calls and mode=='late_write':assert fs('fs/write_text_file','late.txt','forbidden late effect').get('error')
  if calls and mode=='late_tool':tool('search_replace','late.txt',99)
  result={'result':{'sessionId':sid,'cwd':os.getcwd(),'agentName':'rururunx-decision' if decision else 'rururunx-executor','context':{'toolDefinitionsCount':(0 if decision else 2)+(1 if mode=='inventory' else 0),'toolCallCount':calls+(1 if calls and mode=='unnotified' else 0)}}}
 elif method=='session/prompt':
  if os.getenv('RRX_PROMPT_OBSERVED'):pathlib.Path(os.environ['RRX_PROMPT_OBSERVED']).write_text('actual prompt received')
  prompt=p['_meta']['promptId'];assert 'bash_command' not in p['prompt'][0].get('_meta',{})
  assert p['prompt'][0]['text'].startswith('Prepared Task input follows:\n\n'), 'native slash command authority escaped envelope'
  if os.getenv('RRX_EXPECT_INPUT'):assert p['prompt'][0]['text'].endswith(os.environ['RRX_EXPECT_INPUT'])
  send({'jsonrpc':'2.0','method':'session/update','params':{'sessionId':sid,'update':{'sessionUpdate':'session_info_update','title':'native title'}}})
  connection=sqlite3.connect(os.environ['RRX_DATABASE']);rows=connection.execute("select body from records where kind='session'").fetchall();connection.close()
  assert any(json.loads(row[0])['data']['recovery'].get('prompt_id')==prompt and json.loads(row[0])['data']['recovery'].get('input_version') in [1,2] for row in rows),'dispatch not durable'
  if mode=='hang':time.sleep(60)
  if mode=='malformed':print('invalid-json',flush=True);continue
  if mode=='oversize':print('x'*1100000,flush=True);continue
  if mode=='invalid_callback_id':
   send({'jsonrpc':'2.0','id':None,'method':'fs/write_text_file','params':{'sessionId':sid,'path':'invalid-id.txt','content':'forbidden'}});json.loads(sys.stdin.readline())
  if mode=='config_update':update({'sessionUpdate':'config_option_update','configOptions':[]})
  if mode=='callback_budget':
   for _ in range(257):
    send({'jsonrpc':'2.0','id':'permission','method':'session/request_permission','params':{'sessionId':sid,'options':[{'kind':'reject_once','optionId':'deny'}]}});assert json.loads(sys.stdin.readline())['result']['outcome']['optionId']=='deny'
  if mode=='path_budget':fs('fs/read_text_file','a'*4097)
  if mode=='permission':
   send({'jsonrpc':'2.0','id':'permission','method':'session/request_permission','params':{'sessionId':sid,'options':[{'kind':'allow_once','optionId':'allow'},{'kind':'reject_once','optionId':'deny'}]}})
   answer=json.loads(sys.stdin.readline());assert answer['result']['outcome']['optionId']=='deny'
  if not decision:
   n=tool('search_replace' if mode=='wrong_method' else 'read_file','own.txt',1)
   if mode=='ambiguous':tool('read_file','own.txt',99)
   assert fs('fs/read_text_file','own.txt').get('result')
   if mode!='unfinished':done(n)
   if mode=='ambiguous':done('99')
   if mode=='bypass':
    n=tool('search_replace','result.txt',2);done(n)
   else:
    n=tool('search_replace','result.txt',2);fs('fs/read_text_file','result.txt');assert fs('fs/write_text_file','result.txt','owned edit\n').get('result')=={};done(n)
   n=tool('search_replace',os.environ['RRX_FOREIGN'],3);assert fs('fs/write_text_file',os.environ['RRX_FOREIGN'],'forbidden').get('error');done(n,mode!='denied_completed')
  elif mode=='decision_tool':tool('read_file','own.txt',1)
  if mode in ['hook','hook_failure']:pathlib.Path('unexplained.txt').write_text('native hook effect')
  output={'verdict':'DENY','reason':'native fixture'}
  if mode=='schema_enum':output={'verdict':'ALLOW','reason':'native fixture'}
  elif mode=='schema_required':output={'verdict':'DENY'}
  elif mode=='schema_extra':output={'verdict':'DENY','reason':'native fixture','extra':True}
  result={'stopReason':'max_tokens' if mode=='hook_failure' else 'end_turn','_meta':{'sessionId':sid,'promptId':prompt,'usage':{'inputTokens':202 if 'explicit fresh continuation' in p['prompt'][0]['text'] else 101,'outputTokens':22 if 'explicit fresh continuation' in p['prompt'][0]['text'] else 11,'cachedReadTokens':0,'cacheCreationTokens':0},'structuredOutput':output}}
 send({'jsonrpc':'2.0','id':d['id'],'result':result})
"#;

#[tokio::test]
async fn native_execute_edits_only_owned_files_and_preserves_actual_exit() {
    let mut fixture = Fixture::new();
    fixture.request.input.payload = "  /always-approve".into();
    fixture.request.environment.insert(
        "RRX_EXPECT_INPUT".into(),
        fixture.request.input.payload.clone(),
    );
    let adapter = fixture.adapter();
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    let status = finished(&adapter, &session).await;
    assert_eq!(
        status.session.state,
        SessionState::Exited,
        "{:?}",
        status.failure
    );
    assert!(adapter.transport_succeeded(&status));
    assert!(status.session.pid.is_none());
    assert_ne!(status.exit_code, Some(0));
    assert_eq!(
        std::fs::read_to_string(fixture.request.worktree.join("result.txt")).unwrap(),
        "owned edit\n"
    );
    assert!(!fixture.directory.path().join("foreign.txt").exists());
    let usage = adapter
        .usage((&session).into(), "execute".into(), None)
        .await
        .unwrap();
    assert_eq!(usage.input_tokens, Some(101));
    assert_eq!(usage.cached_input_tokens, Some(0));
    assert_eq!(usage.estimated_cost, None);
    let mut forged = status.clone();
    forged.session.recovery["prompt_id"] = json!("forged");
    assert!(!adapter.transport_succeeded(&forged));
    adapter.release((&session).into()).unwrap();
}
#[tokio::test]
async fn concurrent_native_reviewers_share_exact_lock_and_validate_structured_verdict() {
    let mut fixture = Fixture::new();
    fixture.review();
    let adapter = fixture.adapter();
    let schema = json!({"type":"object","properties":{"verdict":{"type":"string","enum":["DENY"]},"reason":{"type":"string"}},"required":["verdict","reason"],"additionalProperties":false});
    let (one, two) = tokio::join!(
        adapter.start_structured(fixture.request.clone(), schema.clone()),
        adapter.start_structured(fixture.request.clone(), schema)
    );
    let one = one.unwrap();
    let two = two.unwrap();
    assert_ne!(one.id, two.id);
    let (a, b) = tokio::join!(finished(&adapter, &one), finished(&adapter, &two));
    assert!(adapter.transport_succeeded(&a), "{:?}", a.failure);
    assert!(adapter.transport_succeeded(&b), "{:?}", b.failure);
    assert_eq!(
        serde_json::from_slice::<Value>(&a.stdout).unwrap()["verdict"],
        "DENY"
    );
}
#[tokio::test]
async fn native_auth_inventory_config_parser_and_tool_evidence_fail_closed() {
    for mode in [
        "version",
        "no_auth_method",
        "auth_error",
        "inventory",
        "config",
        "malformed",
        "oversize",
        "bypass",
        "hook",
        "unfinished",
        "denied_completed",
        "late_write",
        "late_tool",
        "invalid_callback_id",
        "config_update",
        "callback_budget",
        "path_budget",
        "wrong_method",
        "ambiguous",
        "unnotified",
        "hook_failure",
    ] {
        let mut fixture = Fixture::new();
        fixture.mode(mode);
        let adapter = fixture.adapter();
        let session = adapter.start(fixture.request.clone()).await.unwrap();
        let status = finished(&adapter, &session).await;
        assert_eq!(
            status.session.state,
            if [
                "malformed",
                "oversize",
                "bypass",
                "denied_completed",
                "invalid_callback_id",
                "config_update",
                "callback_budget",
                "path_budget",
                "wrong_method",
                "ambiguous"
            ]
            .contains(&mode)
            {
                SessionState::Lost
            } else {
                SessionState::Failed
            },
            "{mode}: {:?}",
            status.failure
        );
        assert!(!adapter.transport_succeeded(&status));
        assert!(status.failure.is_some(), "{mode}");
        if mode == "hook_failure" {
            let events = fixture
                .store
                .lock()
                .unwrap()
                .events(&fixture.request.scope, 0, 100)
                .unwrap();
            let observation = events
                .iter()
                .find(|e| e.kind == "grok.turn_observed")
                .unwrap();
            assert_eq!(observation.data["reconciliation_attempted"], true);
            assert!(
                observation.data["reconciliation_error"]
                    .as_str()
                    .unwrap()
                    .contains("unexplained native/concurrent worktree effect")
            );
        }
        assert!(status.session.pid.is_none(), "{mode}");
        assert!(
            !fixture.request.worktree.join("late.txt").exists(),
            "{mode}: late callback mutated Task"
        );
        assert!(
            !fixture.request.worktree.join("invalid-id.txt").exists(),
            "{mode}: malformed request mutated Task"
        );
    }
}
#[tokio::test]
async fn native_resume_requires_fresh_checkpoint_preserves_uuid_and_discards_replay() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter();
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    let first = finished(&adapter, &session).await;
    assert!(adapter.transport_succeeded(&first), "{:?}", first.failure);
    assert_eq!(
        adapter.resume((&session).into()).await.unwrap_err().kind,
        ErrorKind::InvalidInput
    );
    let mut input = fixture.request.input.clone();
    input.version = 2;
    input.payload = "explicit fresh continuation".into();
    adapter.checkpoint((&session).into(), input).await.unwrap();
    let resumed = adapter.resume((&session).into()).await.unwrap();
    assert_eq!(resumed.id, session.id);
    let second = finished(&adapter, &resumed).await;
    assert_eq!(second.session.native_ref, first.session.native_ref);
    assert!(adapter.transport_succeeded(&second), "{:?}", second.failure);
    assert!(!String::from_utf8_lossy(&second.stdout).contains("REPLAY"));
    assert_eq!(second.session.recovery["input_version"], 2);
    let usage = adapter
        .usage((&resumed).into(), "resume".into(), None)
        .await
        .unwrap();
    assert_eq!(usage.input_tokens, Some(202));
    assert_eq!(usage.output_tokens, Some(22));
}
#[tokio::test]
async fn native_stop_permissions_foreign_refs_and_environment_guards_are_explicit() {
    let mut fixture = Fixture::new();
    fixture.mode("permission");
    let adapter = fixture.adapter();
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    assert!(adapter.transport_succeeded(&finished(&adapter, &session).await));
    let mut foreign = SessionRef::from(&session);
    foreign.scope.task_id = Some(TaskId::new());
    assert_eq!(
        adapter.status(foreign).await.unwrap_err().kind,
        ErrorKind::OwnershipMismatch
    );
    fixture.mode("hang");
    let spawn_observed = fixture.directory.path().join("spawn-observed");
    fixture.request.environment.insert(
        "RRX_SPAWN_OBSERVED".into(),
        spawn_observed.to_str().unwrap().into(),
    );
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    let stopped = adapter.stop((&session).into()).await.unwrap();
    assert_eq!(stopped.session.state, SessionState::Stopped);
    assert!(
        !spawn_observed.exists(),
        "stop during preflight still spawned native process"
    );
    let events = fixture
        .store
        .lock()
        .unwrap()
        .events(&fixture.request.scope, 0, 100)
        .unwrap();
    assert!(
        !events
            .iter()
            .any(|e| e.kind == "grok.process_spawned" && e.data["session"] == json!(session.id)),
        "stopped native process was spawned but killed before fixture startup"
    );
    assert!(stopped.session.pid.is_none());
    assert!(!adapter.transport_succeeded(&stopped));
    for key in [
        "HOME",
        "GROK_HOME",
        "XAI_API_KEY",
        "LD_PRELOAD",
        "DYLD_INSERT_LIBRARIES",
        "NODE_OPTIONS",
        "NODE_TLS_REJECT_UNAUTHORIZED",
        "NODE_EXTRA_CA_CERTS",
        "NODE_PATH",
        "BUN_OPTIONS",
        "OPENSSL_CONF",
        "SSLKEYLOGFILE",
        "BASH_ENV",
        "ENV",
        "SHELL",
        "ZDOTDIR",
        "UNKNOWN_NATIVE_OVERRIDE",
        "HTTPS_PROXY",
    ] {
        let mut request = fixture.request.clone();
        request
            .environment
            .insert(key.into(), "foreign override".into());
        assert_eq!(
            adapter.start(request).await.unwrap_err().kind,
            ErrorKind::InvalidConfiguration
        );
    }
    assert_eq!(
        adapter.attach((&session).into()).await.unwrap_err().kind,
        ErrorKind::UnsupportedCapability
    );
    assert!(
        !adapter
            .capabilities()
            .contains(&Capability::PermissionInterception)
    );
}

#[tokio::test]
async fn unknown_native_dispatch_keeps_clean_dead_executor_reserved() {
    let mut fixture = Fixture::new();
    fixture.mode("oversize");
    let adapter = fixture.adapter();
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    let status = finished(&adapter, &session).await;
    assert_eq!(status.session.state, SessionState::Lost);
    assert!(status.session.pid.is_none());
    assert!(!adapter.transport_succeeded(&status));
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .session(session.id)
            .unwrap()
            .unwrap()
            .0
            .state,
        SessionState::Lost
    );
    assert_eq!(
        git(&fixture.request.worktree, &["status", "--porcelain"]),
        ""
    );
    assert!(
        rrx::git::WorktreeManager::lock_review(
            &mut fixture.store.lock().unwrap(),
            fixture.request.scope.task_id.unwrap(),
            &fixture.request.input.revision,
            "cannot lock Lost executor"
        )
        .is_err()
    );
    assert_eq!(
        adapter
            .start(fixture.request.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::StateConflict
    );
    assert_eq!(
        adapter.release((&session).into()).unwrap_err().kind,
        ErrorKind::SessionLost
    );
    let mut input = fixture.request.input.clone();
    input.version = 2;
    assert_eq!(
        adapter
            .checkpoint((&session).into(), input)
            .await
            .unwrap_err()
            .kind,
        ErrorKind::InvalidInput
    );
}

#[tokio::test]
async fn stop_after_dispatch_preserves_unknown_outcome_until_explicit_recovery() {
    let mut fixture = Fixture::new();
    fixture.mode("hang");
    let adapter = fixture.adapter();
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    let mut watch = adapter.subscribe((&session).into()).unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        while watch.borrow().session.state != SessionState::Running {
            watch.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    let stopped = adapter.stop((&session).into()).await.unwrap();
    assert_eq!(stopped.session.state, SessionState::Lost);
    assert!(stopped.session.pid.is_none());
    assert!(!adapter.transport_succeeded(&stopped));
    assert_eq!(
        adapter
            .start(fixture.request.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::StateConflict
    );
}

#[tokio::test]
#[ignore = "requires installed Grok and existing native auth; explicit isolated acceptance"]
async fn installed_native_edit_fresh_continuation_and_structured_decision() {
    let mut fixture = Fixture::new();
    let native = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|path| path.join("grok"))
        .find(|path| path.is_file())
        .expect("installed native Grok");
    let adapter = GrokAdapter::new(
        "grok".into(),
        native.canonicalize().unwrap(),
        fixture.store.clone(),
    )
    .unwrap();
    fixture.request.model = Some("grok-4.7".into());
    fixture.request.effort = Some("low".into());
    let foreign = fixture.directory.path().join("foreign.txt");
    fixture.request.input.payload = format!(
        "Authorized isolated native adapter acceptance. Use read_file to read own.txt. Use search_replace to create result.txt containing exactly NATIVE_EDIT_PROVED plus a newline. Then explicitly attempt search_replace on {} with content FOREIGN_MUST_BE_DENIED, and report the supervisor rejection. Use only those named tools and fixture paths. Finish when the own file is correct; do not use alternate methods.",
        foreign.display()
    );
    async fn native_finished(adapter: &GrokAdapter, session: &Session) -> SessionStatus {
        let mut status = adapter.subscribe(session.into()).unwrap();
        tokio::time::timeout(Duration::from_secs(330), async {
            while !status.borrow().terminal() {
                status.changed().await.unwrap();
            }
            status.borrow().clone()
        })
        .await
        .unwrap()
    }
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    let first = native_finished(&adapter, &session).await;
    assert!(adapter.transport_succeeded(&first), "{:?}", first.failure);
    assert_eq!(
        std::fs::read_to_string(fixture.request.worktree.join("result.txt")).unwrap(),
        "NATIVE_EDIT_PROVED\n"
    );
    assert!(!foreign.exists());
    let mut input = fixture.request.input.clone();
    input.version = 2;
    input.payload="New explicit continuation input. Read result.txt and preserve it. Create continued.txt with exactly NATIVE_CONTINUATION_PROVED plus a newline using search_replace. Do not repeat or modify the prior file, and use no other paths/tools.".into();
    adapter.checkpoint((&session).into(), input).await.unwrap();
    let resumed = adapter.resume((&session).into()).await.unwrap();
    let second = native_finished(&adapter, &resumed).await;
    assert!(adapter.transport_succeeded(&second), "{:?}", second.failure);
    assert_eq!(second.session.native_ref, first.session.native_ref);
    assert_eq!(
        std::fs::read_to_string(fixture.request.worktree.join("continued.txt")).unwrap(),
        "NATIVE_CONTINUATION_PROVED\n"
    );
    git(
        &fixture.request.worktree,
        &["add", "result.txt", "continued.txt"],
    );
    git(
        &fixture.request.worktree,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-m",
            "verified native fixture",
        ],
    );
    fixture.request.input.version = 3;
    fixture.request.input.revision = git(&fixture.request.worktree, &["rev-parse", "HEAD"]);
    fixture.request.input.payload="Decision-only supplied public fixture bundle. Requirement: writes must remain in the owned Task. Proposed operation: write to a different Project. Return verdict DENY and a concise reason; no files, searches, tools or operations are authorized.".into();
    fixture.review();
    let schema = json!({"type":"object","properties":{"verdict":{"type":"string","enum":["DENY"]},"reason":{"type":"string"}},"required":["verdict","reason"],"additionalProperties":false});
    let review = adapter
        .start_structured(fixture.request.clone(), schema)
        .await
        .unwrap();
    let decision = native_finished(&adapter, &review).await;
    assert!(
        adapter.transport_succeeded(&decision),
        "{:?}",
        decision.failure
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&decision.stdout).unwrap()["verdict"],
        "DENY"
    );
    assert_eq!(
        git(&fixture.request.worktree, &["status", "--porcelain"]),
        ""
    );
    let usage = adapter
        .usage((&resumed).into(), "continuation".into(), None)
        .await
        .unwrap();
    assert!(usage.input_tokens.is_some_and(|v| v > 0));
    assert!(usage.output_tokens.is_some_and(|v| v > 0));
    assert_eq!(usage.estimated_cost, None);
}

#[tokio::test]
#[ignore = "requires installed Grok and existing native auth; isolated decision correlation acceptance"]
async fn installed_native_structured_decision_has_exact_response_correlation() {
    let mut fixture = Fixture::new();
    let native = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|path| path.join("grok"))
        .find(|path| path.is_file())
        .expect("installed native Grok");
    let adapter = GrokAdapter::new(
        "grok".into(),
        native.canonicalize().unwrap(),
        fixture.store.clone(),
    )
    .unwrap();
    fixture.review();
    fixture.request.model = Some("grok-4.7".into());
    fixture.request.effort = Some("low".into());
    fixture.request.input.payload = "Decision-only supplied fixture bundle. Requirement: writes must remain in the owned Task. Proposed operation: write to a different Project. Return verdict DENY and a concise reason; no files, searches, tools or operations are authorized.".into();
    let schema = json!({"type":"object","properties":{"verdict":{"type":"string","enum":["DENY"]},"reason":{"type":"string"}},"required":["verdict","reason"],"additionalProperties":false});
    let session = adapter
        .start_structured(fixture.request.clone(), schema)
        .await
        .unwrap();
    let mut status = adapter.subscribe((&session).into()).unwrap();
    let decision = tokio::time::timeout(Duration::from_secs(330), async {
        while !status.borrow().terminal() {
            status.changed().await.unwrap();
        }
        status.borrow().clone()
    })
    .await
    .unwrap();
    assert!(
        adapter.transport_succeeded(&decision),
        "{:?}",
        decision.failure
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&decision.stdout).unwrap()["verdict"],
        "DENY"
    );
    assert_eq!(
        git(&fixture.request.worktree, &["status", "--porcelain"]),
        ""
    );
}

#[tokio::test]
async fn native_structured_consumer_rejects_each_schema_violation_and_live_decision_tools() {
    for mode in [
        "schema_enum",
        "schema_required",
        "schema_extra",
        "decision_tool",
    ] {
        let mut fixture = Fixture::new();
        fixture.review();
        fixture.mode(mode);
        let adapter = fixture.adapter();
        let schema = json!({"type":"object","properties":{"verdict":{"type":"string","enum":["DENY"]},"reason":{"type":"string"}},"required":["verdict","reason"],"additionalProperties":false});
        let session = adapter
            .start_structured(fixture.request.clone(), schema)
            .await
            .unwrap();
        let status = finished(&adapter, &session).await;
        assert_eq!(
            status.session.state,
            if mode == "decision_tool" {
                SessionState::Lost
            } else {
                SessionState::Failed
            },
            "{mode}: {:?}",
            status.failure
        );
        assert!(!adapter.transport_succeeded(&status));
        assert!(status.failure.is_some());
    }
}

#[tokio::test]
async fn parent_replacement_after_native_preflight_never_reaches_prompt_wire() {
    let mut fixture = Fixture::new();
    fixture.mode("pause_info");
    let pause = fixture.directory.path().join("pause");
    let prompt_observed = fixture.directory.path().join("prompt-observed");
    fixture
        .request
        .environment
        .insert("RRX_PAUSE".into(), pause.to_str().unwrap().into());
    fixture.request.environment.insert(
        "RRX_PROMPT_OBSERVED".into(),
        prompt_observed.to_str().unwrap().into(),
    );
    let adapter = fixture.adapter();
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        while !pause.exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let mut other = Store::open(&fixture.directory.path().join("state.db")).unwrap();
    let mut task = other
        .task(fixture.request.scope.task_id.unwrap())
        .unwrap()
        .unwrap();
    task.title = "concurrent replacement after native admission".into();
    other.put_task(&mut task).unwrap();
    std::fs::write(pause.with_extension("continue"), "resume native response").unwrap();
    let status = finished(&adapter, &session).await;
    assert_eq!(
        status.session.state,
        SessionState::Failed,
        "{:?}",
        status.failure
    );
    assert!(!adapter.transport_succeeded(&status));
    assert!(status.session.pid.is_none());
    assert!(
        !prompt_observed.exists(),
        "native prompt was sent under replaced parent authority"
    );
    let saved = other.session(session.id).unwrap().unwrap().0;
    assert!(saved.recovery.get("prompt_id").is_none());
    assert_ne!(saved.recovery["dispatch_state"], "dispatching");
    assert_eq!(other.task(task.id).unwrap().unwrap().title, task.title);
}
