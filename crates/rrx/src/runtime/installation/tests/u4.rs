//! Real S child conformance; these peers never create model input or P authority.
use super::*;
use crate::runtime::startup::{FreshPreparationUnavailable, ProbeState, StartupState};
const PEER: &str = r#"
import json, os, sys
def send(v): print(json.dumps(v),flush=True)
def shim(role):
    # A real Runtime IPC tool request from this S child. S has no Unit, so only
    # the server's refusal frame can produce 'refused'; any failure is recorded.
    import socket, struct, subprocess, uuid
    try:
        with open(os.path.join(EVIDENCE,'socket')) as f: path=f.read()
        peer=socket.socket(socket.AF_UNIX);peer.connect(path)
        data=json.dumps({'unit':str(uuid.uuid4()),'cookie':'s-child','tool':'git','args':['init',os.path.join(EVIDENCE,'granted-effect')],'cwd':os.getcwd(),'git_parent':None}).encode()
        peer.sendall(struct.pack('!I',len(data))+data)
        def exact(n):
            out=b''
            while len(out)<n:
                chunk=peer.recv(n-len(out))
                if not chunk: raise RuntimeError('EOF')
                out+=chunk
            return out
        frame=json.loads(exact(struct.unpack('!I',exact(4))[0]))
        if frame['type']=='granted':
            subprocess.run([frame['value']['program']]+frame['value']['args'],cwd=os.getcwd())
        outcome=frame['type']
    except Exception as e:
        outcome='error:'+type(e).__name__
    with open(os.path.join(EVIDENCE,ALIAS+'-'+role),'w') as f: f.write(outcome)
args=sys.argv[1:]
if args==['--version']:
    print('arbitrary updated CLI 88.1')
    sys.exit(0)
if SCENARIO=='shim': shim('schema' if args[:1]==['app-server'] and 'generate-json-schema' in args else 'handshake')
if args[:3]==['app-server','generate-json-schema','--experimental']:
    assert PROVIDER=='codex' and args[3]=='--out'
    with open(PAIR) as f: files=json.load(f)['files']
    for source,path in FILES:
        target=os.path.join(args[4],path)
        os.makedirs(os.path.dirname(target),exist_ok=True)
        with open(target,'w') as f: json.dump(files[source],f)
    sys.exit(0)
if PROVIDER=='codex': assert args==['app-server','--listen','stdio://']
else: assert args[-2:]==['--permission-mode','plan']
if SCENARIO=='schema-only' and PROVIDER=='codex':
    sys.stdin.readline()
    sys.exit(0)
expected=['initialize','initialized','account/read','environment/status','account/rateLimits/read']
index=0
for line in sys.stdin:
    v=json.loads(line)
    if PROVIDER=='claude':
        assert v['request']=={'subtype':'initialize','hooks':None}
        send({'type':'control_response','response':{'subtype':'success','request_id':v['request_id'],'response':{}}})
        continue
    assert v['method']==expected[index]
    index+=1
    if v['method']=='initialized':
        assert v=={'method':'initialized'}
        continue
    if SCENARIO=='missing' and v['method']=='environment/status': sys.exit(0)
    if SCENARIO=='unsupported':
        send({'id':v['id'],'error':{'code':-32601,'message':'unsupported'}})
        continue
    if v['method']=='initialize':
        result={'codexHome':'/fixture','platformFamily':'unix','platformOs':'macos' if sys.platform=='darwin' else 'linux','userAgent':'fixture'}
    elif v['method']=='account/read':
        assert v['params']=={'refreshToken':False}
        result={'requiresOpenaiAuth':True,'account':{'type':'apiKey' if SCENARIO=='api' else 'chatgpt','email':None,'planType':'pro'}}
    elif v['method']=='environment/status':
        assert v['params']=={'environmentId':'local'}
        result={'status':'pending' if SCENARIO=='pending' else 'ready'}
    else:
        assert v['params']=={}
        result={'rateLimits':{'primary':{'usedPercent':20}} if SCENARIO=='usable' else {}}
        if SCENARIO=='rate-malformed': result['rateLimitsByLimitId']={'foreign':{'primary':{'usedPercent':'bad'}}}
    send({'id':999 if SCENARIO=='id' else v['id'],'result':result})
    if SCENARIO=='tail' and index==5: print('{',flush=True)
"#;
/// Directory beside the peers where each S child records its IPC outcome.
const EVIDENCE: &str = "u4-shim-evidence";
fn fixture_s(scenario: &str) -> ControlFixture {
    fixture_with("codex", true, |config| {
        for (alias, agent) in config.agents.iter_mut() {
            let old = std::path::PathBuf::from(&agent.command[0]);
            let path = old.with_file_name(format!("u4-{alias}"));
            let pair = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/codex-contract/basis-pair.json");
            let evidence = old.with_file_name(EVIDENCE);
            let script = format!(
                "#!/usr/bin/python3\nPROVIDER={:?}\nSCENARIO={scenario:?}\nALIAS={alias:?}\nEVIDENCE={:?}\nPAIR={:?}\nFILES={}\n{PEER}",
                agent.provider.as_deref().unwrap(),
                evidence.to_str().unwrap(),
                pair.to_str().unwrap(),
                serde_json::to_string(crate::execution::native::schema::FILES).unwrap()
            );
            std::fs::write(&path, script).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
            agent.command = vec![path.to_string_lossy().into_owned()];
        }
    })
}
async fn settled_s(f: &ControlFixture) -> crate::runtime::startup::StartupStatus {
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            let s = f.runtime.startup.snapshot().unwrap();
            if !matches!(s.state, StartupState::NotStarted | StartupState::Pending) {
                break s;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap()
}
/// S pass is public for every alias, cleans only its own epoch, and still cannot grant admission without fresh P.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u4_all_alias_s_pass_keeps_admission_zero() {
    for scenario in ["empty_valid", "usable"] {
        let f = fixture_s(scenario);
        let (_, tasks) = accept(&f, 1).await;
        f.runtime.start().await.unwrap();
        let status = settled_s(&f).await;
        assert_eq!(status.state, StartupState::Passed, "{status:?}");
        assert!(
            status
                .aliases
                .iter()
                .all(|a| a.steps.iter().all(|s| s.state == ProbeState::Passed))
        );
        assert_eq!(
            status.aliases.iter().map(|a| a.steps.len()).sum::<usize>(),
            7
        );
        assert!(
            f.runtime
                .installed_driver_composition(&tasks[0])
                .err()
                .unwrap()
                .is::<FreshPreparationUnavailable>()
        );
        let _ = f.runtime.admit_ready_tasks();
        assert_no_execution_effects(&f);
        let entries = std::fs::read_dir(f.owner.root.join("startup"))
            .unwrap()
            .count();
        assert_eq!(entries, 0, "successful S epoch retained");
        assert_eq!(
            count(&f, "quota_windows"),
            0,
            "S metadata updated quota windows"
        );
        assert_eq!(
            count(&f, "quota_pools"),
            0,
            "S metadata updated quota pools"
        );
        f.runtime.shutdown().await.unwrap();
        finish(f).await;
    }
}
/// I-S: tool requests from Schema and Handshake children reach the real server and are refused without managed context.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u4r1b_s_children_shim_requests_reach_server_and_are_refused() {
    let f = fixture_s("shim");
    let evidence = f._dir.path().join(EVIDENCE);
    std::fs::create_dir(&evidence).unwrap();
    std::fs::write(
        evidence.join("socket"),
        f.owner.ipc_path().to_str().unwrap(),
    )
    .unwrap();
    let (_, tasks) = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    let status = settled_s(&f).await;
    assert_eq!(status.state, StartupState::Passed, "{status:?}");
    for record in [
        "worker-schema",
        "worker-handshake",
        "rev-a-handshake",
        "rev-b-handshake",
    ] {
        assert_eq!(
            std::fs::read_to_string(evidence.join(record))
                .ok()
                .as_deref(),
            Some("refused"),
            "{record}: request did not reach a refusing server"
        );
    }
    assert!(
        !evidence.join("granted-effect").exists(),
        "helper-external effect from an S child"
    );
    assert!(f.runtime.installed_driver_composition(&tasks[0]).is_err());
    let _ = f.runtime.admit_ready_tasks();
    assert_no_execution_effects(&f);
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}
/// Schema conformance cannot replace correlated successful account/environment/rate RPCs or tolerate a malformed tail.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn u4_schema_alone_and_bad_handshake_refuse() {
    for scenario in [
        "schema-only",
        "id",
        "api",
        "pending",
        "unsupported",
        "missing",
        "tail",
        "rate-malformed",
    ] {
        let f = fixture_s(scenario);
        let (_, tasks) = accept(&f, 1).await;
        f.runtime.start().await.unwrap();
        let status = settled_s(&f).await;
        assert_ne!(status.state, StartupState::Passed, "{scenario}");
        let codex = status
            .aliases
            .iter()
            .find(|a| a.provider == "codex")
            .unwrap();
        assert_eq!(
            codex.steps[1].state,
            ProbeState::Passed,
            "{scenario}: {status:?}"
        );
        assert_ne!(codex.steps[2].state, ProbeState::Passed, "{scenario}");
        assert!(f.runtime.installed_driver_composition(&tasks[0]).is_err());
        let _ = f.runtime.admit_ready_tasks();
        assert_no_execution_effects(&f);
        assert!(
            std::fs::read_dir(f.owner.root.join("startup"))
                .unwrap()
                .next()
                .is_some(),
            "failed S paths removed"
        );
        f.runtime.shutdown().await.unwrap();
        finish(f).await;
    }
}
