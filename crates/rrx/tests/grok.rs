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
 elif method=='_x.ai/session/info':result={'result':{'sessionId':sid,'cwd':os.getcwd(),'agentName':'rururunx-decision' if decision else 'rururunx-executor','context':{'toolDefinitionsCount':(0 if decision else 2)+(1 if mode=='inventory' else 0),'toolCallCount':calls}}}
 elif method=='session/prompt':
  prompt=p['_meta']['promptId'];assert 'bash_command' not in p['prompt'][0].get('_meta',{})
  connection=sqlite3.connect(os.environ['RRX_DATABASE']);rows=connection.execute("select body from records where kind='session'").fetchall();connection.close()
  assert any(json.loads(row[0])['data']['recovery'].get('prompt_id')==prompt and json.loads(row[0])['data']['recovery'].get('input_version') in [1,2] for row in rows),'dispatch not durable'
  if mode=='hang':time.sleep(60)
  if mode=='malformed':print('invalid-json',flush=True);continue
  if mode=='oversize':print('x'*1100000,flush=True);continue
  if mode=='permission':
   send({'jsonrpc':'2.0','id':'permission','method':'session/request_permission','params':{'sessionId':sid,'options':[{'kind':'allow_once','optionId':'allow'},{'kind':'reject_once','optionId':'deny'}]}})
   answer=json.loads(sys.stdin.readline());assert answer['result']['outcome']['optionId']=='deny'
  if not decision:
   n=tool('read_file','own.txt',1);assert fs('fs/read_text_file','own.txt').get('result');done(n)
   if mode=='bypass':
    n=tool('search_replace','result.txt',2);done(n)
   else:
    n=tool('search_replace','result.txt',2);fs('fs/read_text_file','result.txt');assert fs('fs/write_text_file','result.txt','owned edit\n').get('result')=={};done(n)
   n=tool('search_replace',os.environ['RRX_FOREIGN'],3);assert fs('fs/write_text_file',os.environ['RRX_FOREIGN'],'forbidden').get('error');done(n,True)
  elif mode=='decision_tool':tool('read_file','own.txt',1)
  if mode=='hook':pathlib.Path('unexplained.txt').write_text('native hook effect')
  output={'verdict':'DENY','reason':'native fixture'} if mode!='schema' else {'verdict':'ALLOW','extra':'invalid'}
  result={'stopReason':'end_turn','_meta':{'sessionId':sid,'promptId':prompt,'usage':{'inputTokens':101,'outputTokens':11,'cachedReadTokens':0,'cacheCreationTokens':0},'structuredOutput':output}}
 send({'jsonrpc':'2.0','id':d['id'],'result':result})
"#;

#[tokio::test]
async fn native_execute_edits_only_owned_files_and_preserves_actual_exit() {
    let fixture = Fixture::new();
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
    ] {
        let mut fixture = Fixture::new();
        fixture.mode(mode);
        let adapter = fixture.adapter();
        let session = adapter.start(fixture.request.clone()).await.unwrap();
        let status = finished(&adapter, &session).await;
        assert_eq!(
            status.session.state,
            if ["malformed", "oversize", "bypass"].contains(&mode) {
                SessionState::Lost
            } else {
                SessionState::Failed
            },
            "{mode}: {:?}",
            status.failure
        );
        assert!(!adapter.transport_succeeded(&status));
        assert!(status.failure.is_some(), "{mode}");
        assert!(status.session.pid.is_none(), "{mode}");
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
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    let stopped = adapter.stop((&session).into()).await.unwrap();
    assert_eq!(stopped.session.state, SessionState::Stopped);
    assert!(stopped.session.pid.is_none());
    assert!(!adapter.transport_succeeded(&stopped));
    for key in [
        "HOME",
        "GROK_HOME",
        "XAI_API_KEY",
        "LD_PRELOAD",
        "DYLD_INSERT_LIBRARIES",
        "NODE_OPTIONS",
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
