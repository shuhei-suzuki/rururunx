//! Installed-binary tool shims. IPC failure never grants a native effect.
use super::*;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    process::Command,
    task::{JoinHandle, JoinSet},
};

const FRAME: usize = 128 * 1024;
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolRequest {
    unit: UnitId,
    cookie: String,
    tool: String,
    args: Vec<String>,
    cwd: PathBuf,
    git_parent: Option<OperationId>,
    #[serde(default)]
    git_index: Option<PathBuf>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum Frame {
    Granted(Grant),
    Started {
        operation: OperationId,
    },
    Completed {
        operation: OperationId,
        code: Option<i32>,
        group_cleanup_unknown: bool,
    },
    Acknowledged,
    Cancel,
    Refused(String),
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Grant {
    operation: OperationId,
    program: PathBuf,
    args: Vec<String>,
    environment: BTreeMap<String, String>,
    tool_bin: PathBuf,
}
async fn write_frame<T: Serialize>(
    stream: &mut (impl AsyncWrite + Unpin),
    value: &T,
) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    ensure!(bytes.len() <= FRAME, "IPC frame too large");
    tokio::time::timeout(Duration::from_secs(10), async {
        stream.write_u32(bytes.len() as u32).await?;
        stream.write_all(&bytes).await?;
        stream.flush().await
    })
    .await??;
    Ok(())
}
async fn read_frame<T: serde::de::DeserializeOwned>(
    stream: &mut (impl AsyncRead + Unpin),
) -> Result<T> {
    let count = stream.read_u32().await? as usize;
    ensure!(count > 0 && count <= FRAME, "IPC frame too large");
    let mut bytes = vec![0; count];
    stream.read_exact(&mut bytes).await?;
    Ok(serde_json::from_slice(&bytes)?)
}
// Offsets survive select cancellation; a partial frame is never read as a new header.
#[derive(Default)]
struct Decoder {
    header: [u8; 4],
    header_used: usize,
    bytes: Vec<u8>,
    body_used: usize,
}
impl Decoder {
    async fn receive(&mut self, stream: &mut (impl AsyncRead + Unpin)) -> Result<Frame> {
        while self.header_used < 4 {
            let count = stream.read(&mut self.header[self.header_used..]).await?;
            ensure!(count > 0, "IPC peer closed");
            self.header_used += count;
        }
        if self.bytes.is_empty() {
            let count = u32::from_be_bytes(self.header) as usize;
            ensure!(count > 0 && count <= FRAME, "IPC frame too large");
            self.bytes.resize(count, 0);
        }
        while self.body_used < self.bytes.len() {
            let count = stream.read(&mut self.bytes[self.body_used..]).await?;
            ensure!(count > 0, "IPC peer closed");
            self.body_used += count;
        }
        let result = serde_json::from_slice(&self.bytes)?;
        *self = Self::default();
        Ok(result)
    }
}
pub struct ToolServer {
    accept: JoinHandle<()>,
}
impl ToolServer {
    pub fn start(owner: Arc<RuntimeOwner>) -> Result<Self> {
        let listener = UnixListener::bind(owner.ipc_path())?;
        std::fs::set_permissions(owner.ipc_path(), std::fs::Permissions::from_mode(0o600))?;
        let accept = tokio::spawn(async move {
            let permits = Arc::new(tokio::sync::Semaphore::new(32));
            let mut jobs = JoinSet::new();
            loop {
                tokio::select! {
                    client=listener.accept()=>{
                        let Ok((stream,_))=client else {break};
                        let Ok(permit)=permits.clone().try_acquire_owned() else {drop(stream);continue};
                        let owner=owner.clone();jobs.spawn(async move {let _permit=permit;let _=serve(owner,stream).await;});
                    },
                    _=jobs.join_next(),if !jobs.is_empty()=>{}
                }
            }
        });
        Ok(Self { accept })
    }
}
impl Drop for ToolServer {
    fn drop(&mut self) {
        self.accept.abort();
    }
}

fn current(
    owner: &Arc<RuntimeOwner>,
    request: &ToolRequest,
) -> Result<(ExecutionUnit, resources::ResourceProfile)> {
    let unit = owner
        .store
        .lock()
        .map_err(|_| anyhow::anyhow!("state poisoned"))?
        .execution_unit(request.unit)?;
    ensure!(
        unit.owner_epoch == owner.epoch && unit.cookie == request.cookie,
        "tool belongs to retired/foreign runtime"
    );
    owner
        .store
        .lock()
        .map_err(|_| anyhow::anyhow!("state poisoned"))?
        .validate_execution(&unit.authority(), true, false)?;
    let profile = resources::ResourceManager::new(owner.clone()).profile(&unit)?;
    Ok((unit, profile))
}
async fn serve(owner: Arc<RuntimeOwner>, mut stream: UnixStream) -> Result<()> {
    ensure!(
        stream.peer_cred()?.uid() == rustix::process::geteuid().as_raw(),
        "IPC peer user differs"
    );
    let request: ToolRequest =
        tokio::time::timeout(Duration::from_secs(10), read_frame(&mut stream)).await??;
    let admitted=async {
        let (unit,profile)=current(&owner,&request)?;
        ensure!(request.cwd.canonicalize()?==request.cwd && [profile.worktree.as_path(),profile.temp.as_path(),profile.output.as_path(),profile.cache.as_path()]
            .iter().any(|root|request.cwd.starts_with(root)),"tool cwd outside declared namespace");
        let operation=OperationId::new();let mut plan=tools::plan(&profile,&unit,&request.tool,&request.args,operation)?;
        if request.tool=="git" {
            ensure!(request.cwd.starts_with(&unit.worktree),"managed Git must use this unit's source namespace");
            if unit.kind==UnitKind::Executor {
                let (project,task)={let store=owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?;
                    (store.project(unit.scope.project_id)?.context("Project missing")?,store.task(unit.scope.task_id.context("Task missing")?)?.context("Task missing")?)};
                super::git_io::UnitGit::new(owner.clone(),&unit,true)?.ownership(&project,&task).await?;
            }else{
                let top=super::git_io::UnitGit::new(owner.clone(),&unit,true)?.text(&request.cwd,["rev-parse","--show-toplevel"]).await?;
                ensure!(Path::new(&top).canonicalize()?==unit.worktree,"nested/foreign review Git namespace");
            }
        }
        ensure!(request.git_index.is_none() || (request.tool=="git" && request.git_parent.is_some()),"candidate index requires a live Git hook chain");
        let _git=if plan.serialized_git || request.git_index.is_some() {Some(owner.git_lease(unit.id,request.git_parent).await?)}else{None};
        if let Some(lease)=&_git {plan.environment.insert("RRX_GIT_GATE_TOKEN".into(),lease.id.to_string());}
        if let Some(index)=&request.git_index {
            let index=if index.is_absolute(){index.clone()}else{request.cwd.join(index)};
            let admin=super::git_io::UnitGit::new(owner.clone(),&unit,true)?.text(&unit.worktree,["rev-parse","--absolute-git-dir"]).await?;
            let admin=Path::new(&admin).canonicalize()?;
            ensure!(index.canonicalize()?==index && index.parent()==Some(admin.as_path()) && index.is_file(),"candidate index is outside the owned Git administration directory");
            let name=index.file_name().and_then(|s|s.to_str()).context("candidate index name missing")?;
            let generated=name.strip_prefix("next-index-").map(|s|s.strip_suffix(".lock").unwrap_or(s));
            ensure!(matches!(name,"index"|"index.lock") || generated.is_some_and(|s|!s.is_empty() && s.bytes().all(|b|b.is_ascii_digit())),"unsupported native candidate index");
            plan.environment.insert("GIT_INDEX_FILE".into(),index.to_str().context("candidate index UTF-8")?.into());
        }
        // Runtime admits and journals; the shim executes in its inherited native sandbox.
        // No credentials, stdin, stdout or stderr are transported through Runtime.
        {
            let mut store=owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?;
            let now=store.execution_unit(unit.id)?;
            ensure!(now.generation==unit.generation && now.session_id==unit.session_id && now.owner_epoch==unit.owner_epoch,"tool admission retired while queued");
            store.reserve_managed_effect(&now.authority(),&ManagedEffect {id:operation,unit_id:unit.id,scope:unit.scope.clone(),kind:plan.kind.into(),
                idempotency_key:format!("tool-{operation}"),expected_target:plan.docker_name.clone().unwrap_or_else(||request.cwd.to_string_lossy().into()),
                state:EffectState::Pending,receipt:BTreeMap::new(),version:1})?;
        }
        let exchanged=async {
            let mut environment=profile.namespace_environment(&unit.cookie,owner.ipc_path());
            environment.extend(plan.environment);
            write_frame(&mut stream,&Frame::Granted(Grant {operation,program:plan.program,args:plan.args,environment,tool_bin:profile.tool_bin.clone()})).await?;
            let (mut reader,mut writer)=stream.split();
            let mut decoder=Decoder::default();let mut started=false;let mut cancelled=false;
            let mut deadline=tokio::time::Instant::now()+Duration::from_secs(1800);
            loop {
                tokio::select! {
                    frame=decoder.receive(&mut reader)=>match frame? {
                        Frame::Started {operation:id} if id==operation && !started=>started=true,
                        Frame::Completed {operation:id,code,group_cleanup_unknown} if id==operation && started=>{
                            let mut receipt=BTreeMap::new();
                            receipt.insert("exit".into(),code.map_or_else(||"signal".into(),|n|n.to_string()));
                            receipt.insert("group_cleanup".into(),if group_cleanup_unknown {"unknown"}else{"requested"}.into());
                            receipt.insert("after_retirement".into(),cancelled.to_string());
                            let state=if plan.docker_name.is_none(){EffectState::Confirmed}else{EffectState::Unknown};
                            owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.reconcile_managed_effect(operation,1,state,receipt)?;
                            write_frame(&mut writer,&Frame::Acknowledged).await?;return Ok::<_,anyhow::Error>(());
                        },_=>anyhow::bail!("invalid tool lifecycle frame")
                    },
                    _=tokio::time::sleep_until(deadline)=>anyhow::bail!("tool receipt deadline exceeded"),
                    _=tokio::time::sleep(Duration::from_millis(100)),if !cancelled=>{
                        if current(&owner,&request).is_err(){
                            cancelled=true;deadline=tokio::time::Instant::now()+Duration::from_secs(15);
                            write_frame(&mut writer,&Frame::Cancel).await?;
                        }
                    }
                }
            }
        }.await;
        if exchanged.is_err(){
            let _=owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.reconcile_managed_effect(operation,1,EffectState::Unknown,BTreeMap::new());
        }
        exchanged?;
        Ok::<_,anyhow::Error>(())
    }.await;
    if admitted.is_err() {
        let _ = write_frame(
            &mut stream,
            &Frame::Refused("managed tool refused or observation incomplete".into()),
        )
        .await;
    }
    admitted
}

/// Called before argument parsing when this same installed binary is invoked by a tool symlink.
pub fn tool_entry() -> Result<Option<i32>> {
    let argv = std::env::args_os().collect::<Vec<_>>();
    let name = Path::new(argv.first().context("argv0 missing")?)
        .file_name()
        .and_then(|n| n.to_str())
        .context("argv0 UTF-8")?;
    if !matches!(
        name,
        "git" | "docker" | "gradle" | "bazel" | "bazelisk" | "sccache" | "tmux" | "ssh"
    ) {
        return Ok(None);
    }
    let git_index = if name == "git" {
        std::env::var_os("GIT_INDEX_FILE").map(PathBuf::from)
    } else {
        None
    };
    let request = ToolRequest {
        unit: std::env::var("RRX_UNIT_ID")?.parse()?,
        cookie: std::env::var("RRX_PROCESS_COOKIE")?,
        tool: name.into(),
        args: argv
            .into_iter()
            .skip(1)
            .map(|s| {
                s.into_string()
                    .map_err(|_| anyhow::anyhow!("tool args UTF-8"))
            })
            .collect::<Result<_>>()?,
        cwd: std::env::current_dir()?.canonicalize()?,
        git_parent: std::env::var("RRX_GIT_GATE_TOKEN")
            .ok()
            .map(|s| s.parse())
            .transpose()?,
        git_index,
    };
    let socket =
        std::env::var_os("RRX_RUNTIME_SOCKET").context("Runtime IPC missing; refusing tool")?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let result = runtime.block_on(async {
        let mut stream = UnixStream::connect(socket).await?;
        ensure!(
            stream.peer_cred()?.uid() == rustix::process::geteuid().as_raw(),
            "IPC server user differs"
        );
        write_frame(&mut stream, &request).await?;
        let grant = match read_frame::<Frame>(&mut stream).await? {
            Frame::Granted(grant) => grant,
            Frame::Refused(reason) => anyhow::bail!(reason),
            _ => anyhow::bail!("invalid tool grant"),
        };
        let code = execute_grant(&mut stream, grant, &request.cwd).await?;
        Ok(Some(code.unwrap_or(125)))
    });
    runtime.shutdown_background();
    result
}

/// The real command stays in the Agent's inherited execution context and stdio.
async fn execute_grant(stream: &mut UnixStream, grant: Grant, cwd: &Path) -> Result<Option<i32>> {
    execute(stream, grant, cwd, Stdio::inherit())
        .await
        .map(|(code, _)| code)
}
async fn execute(
    stream: &mut UnixStream,
    grant: Grant,
    cwd: &Path,
    stdout: Stdio,
) -> Result<(Option<i32>, Vec<u8>)> {
    ensure!(
        grant.program.is_absolute(),
        "managed program must be absolute"
    );
    // Tokio sockets are CLOEXEC; the real tool must not inherit this authority channel.
    ensure!(
        rustix::io::fcntl_getfd(&*stream)?.contains(rustix::io::FdFlags::CLOEXEC),
        "tool IPC fd is inheritable"
    );
    let mut paths = vec![grant.tool_bin];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").context("native PATH missing")?,
    ));
    let path = std::env::join_paths(paths)?;
    let mut command = Command::new(&grant.program);
    command.env("PATH", path);
    command
        .args(&grant.args)
        .current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(stdout)
        .stderr(Stdio::inherit());
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_NAMESPACE",
    ] {
        command.env_remove(key);
    }
    // Only the admitted live same-unit hook chain may restore a native candidate
    // index. Arbitrary inherited Git namespace overrides remain removed.
    command.envs(&grant.environment);
    let mut child = process::OwnedProcess::spawn(&mut command)?;
    let stdout = child.child.stdout.take();
    let drain = tokio::spawn(async move {
        match stdout {
            Some(stdout) => process::bounded_read(stdout, 4096).await,
            None => Ok(Vec::new()),
        }
    });
    write_frame(
        stream,
        &Frame::Started {
            operation: grant.operation,
        },
    )
    .await?;
    let mut decoder = Decoder::default();
    let observed = tokio::select! {
        result=child.exited()=>result,
        frame=decoder.receive(stream)=>match frame {Ok(Frame::Cancel)=>Ok(()),Ok(_)=>Err(anyhow::anyhow!("invalid tool control")),Err(e)=>Err(e)}
    };
    let receipt = child.stop_and_reap().await;
    let bytes = tokio::time::timeout(Duration::from_secs(2), drain).await???;
    let receipt = receipt?;
    let completed = Frame::Completed {
        operation: grant.operation,
        code: receipt.status.code(),
        group_cleanup_unknown: receipt.group_error.is_some(),
    };
    write_frame(stream, &completed).await?;
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match decoder.receive(stream).await? {
                Frame::Acknowledged => break Ok::<_, anyhow::Error>(()),
                Frame::Cancel => {}
                _ => anyhow::bail!("invalid tool receipt acknowledgement"),
            }
        }
    })
    .await??;
    observed?;
    Ok((receipt.status.code(), bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    async fn invoke(
        owner: &RuntimeOwner,
        unit: &ExecutionUnit,
        cookie: &str,
        args: &[&str],
    ) -> Result<(Option<i32>, Vec<u8>)> {
        let mut stream = UnixStream::connect(owner.ipc_path()).await?;
        write_frame(
            &mut stream,
            &ToolRequest {
                unit: unit.id,
                cookie: cookie.into(),
                tool: "git".into(),
                args: args.iter().map(|a| (*a).into()).collect(),
                cwd: unit.worktree.clone(),
                git_parent: None,
                git_index: None,
            },
        )
        .await?;
        match read_frame::<Frame>(&mut stream).await? {
            Frame::Granted(grant) => {
                execute(&mut stream, grant, &unit.worktree, Stdio::piped()).await
            }
            Frame::Refused(reason) => anyhow::bail!(reason),
            _ => anyhow::bail!("invalid fixture grant"),
        }
    }
    #[tokio::test]
    async fn actual_tool_ipc_executes_current_scope_and_refuses_foreign_or_retired_cookie() {
        let (_dir, owner, task) = results::tests::fixture().await;
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, "codex", "Implement", None)
            .await
            .unwrap();
        let _server = ToolServer::start(owner.clone()).unwrap();
        let (code, bytes) = invoke(
            &owner,
            &unit,
            &unit.cookie,
            &["rev-parse", "--verify", "HEAD"],
        )
        .await
        .unwrap();
        assert_eq!(code, Some(0));
        assert_eq!(results::text(&bytes).unwrap(), unit.base_sha);
        let count = owner
            .store
            .lock()
            .unwrap()
            .managed_effects(unit.id)
            .unwrap()
            .len();
        assert!(
            invoke(&owner, &unit, "foreign-cookie", &["rev-parse", "HEAD"])
                .await
                .is_err()
        );
        assert!(
            invoke(
                &owner,
                &unit,
                &unit.cookie,
                &["diff", "--output=/tmp/foreign"]
            )
            .await
            .is_err()
        );
        owner
            .store
            .lock()
            .unwrap()
            .retire_execution(&unit.authority(), false)
            .unwrap();
        assert!(
            invoke(&owner, &unit, &unit.cookie, &["rev-parse", "HEAD"])
                .await
                .is_err()
        );
        assert_eq!(
            owner
                .store
                .lock()
                .unwrap()
                .managed_effects(unit.id)
                .unwrap()
                .len(),
            count
        );
    }
    #[tokio::test]
    async fn oid_shaped_branch_names_cannot_authorize_foreign_ref_mutation() {
        let (_dir, owner, task) = results::tests::fixture().await;
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, "codex", "Implement", None)
            .await
            .unwrap();
        let foreign = "a".repeat(40);
        results::git(&unit.worktree, ["branch", &foreign, &unit.base_sha])
            .await
            .unwrap();
        let reference = format!("refs/heads/{foreign}");
        let before = results::git(&unit.worktree, ["show-ref", "--verify", &reference])
            .await
            .unwrap();
        let count = owner
            .store
            .lock()
            .unwrap()
            .managed_effects(unit.id)
            .unwrap()
            .len();
        let _server = ToolServer::start(owner.clone()).unwrap();
        for args in [
            vec!["branch", "-D", &foreign],
            vec!["branch", "-M", unit.branch.as_deref().unwrap(), &foreign],
            vec!["branch", &foreign, &unit.base_sha],
            vec!["switch", "-C", &foreign, &unit.base_sha],
            vec!["checkout", "-B", &foreign, &unit.base_sha],
        ] {
            assert!(
                invoke(&owner, &unit, &unit.cookie, &args).await.is_err(),
                "{args:?}"
            );
        }
        assert_eq!(
            results::git(&unit.worktree, ["show-ref", "--verify", &reference])
                .await
                .unwrap(),
            before
        );
        assert_eq!(
            owner
                .store
                .lock()
                .unwrap()
                .managed_effects(unit.id)
                .unwrap()
                .len(),
            count
        );
    }

    #[tokio::test]
    async fn managed_post_commit_hook_can_reenter_without_blocking_sibling_git() {
        let (_dir, owner, task) = results::tests::fixture().await;
        let attempts = attempts::AttemptManager::new(owner.clone());
        let (unit, _) = attempts
            .prepare(task.id, "codex", "Implement", None)
            .await
            .unwrap();
        let project = owner
            .store
            .lock()
            .unwrap()
            .project(task.project_id)
            .unwrap()
            .unwrap();
        results::git(&project.root, ["config", "user.name", "Fixture"])
            .await
            .unwrap();
        results::git(
            &project.root,
            ["config", "user.email", "fixture@example.invalid"],
        )
        .await
        .unwrap();
        results::git(&project.root, ["config", "commit.gpgsign", "false"])
            .await
            .unwrap();
        let hook = project.root.join(".git/hooks/post-commit");
        let request = serde_json::to_string(&ToolRequest {
            unit: unit.id,
            cookie: unit.cookie.clone(),
            tool: "git".into(),
            args: vec![
                "update-ref".into(),
                format!("refs/heads/{}", unit.branch.as_ref().unwrap()),
                unit.base_sha.clone(),
                unit.base_sha.clone(),
            ],
            cwd: unit.worktree.clone(),
            git_parent: None,
            git_index: None,
        })
        .unwrap();
        let socket = serde_json::to_string(&owner.ipc_path()).unwrap();
        let script = format!(
            r#"#!/usr/bin/python3
# Account-free IPC peer fixture, not an installed-shim/native acceptance claim.
import json, os, socket, struct, subprocess
peer=socket.socket(socket.AF_UNIX);peer.connect({socket})
def send(value):
    data=json.dumps(value).encode();peer.sendall(struct.pack('!I',len(data))+data)
def exact(count):
    data=b''
    while len(data)<count:
        chunk=peer.recv(count-len(data))
        if not chunk: raise RuntimeError('EOF')
        data+=chunk
    return data
def receive():
    return json.loads(exact(struct.unpack('!I',exact(4))[0]))
request=json.loads({request:?});request['git_parent']=os.environ['RRX_GIT_GATE_TOKEN']
head=subprocess.check_output(['/usr/bin/git','rev-parse','HEAD'],cwd=request['cwd']).decode().strip()
request['args'][2]=head;request['args'][3]=head
send(request);frame=receive();assert frame['type']=='granted',frame
grant=frame['value'];operation=grant['operation'];send({{'type':'started','value':{{'operation':operation}}}})
env=dict(os.environ);env.update(grant['environment'])
code=subprocess.run([grant['program']]+grant['args'],env=env,cwd=request['cwd']).returncode
send({{'type':'completed','value':{{'operation':operation,'code':code,'group_cleanup_unknown':False}}}})
assert receive()['type']=='acknowledged'
assert code==0
with open(os.path.join(request['cwd'],'hook-completed'),'w') as marker: marker.write('nested Git completed')
"#
        );
        std::fs::write(&hook, script).unwrap();
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o700)).unwrap();
        let _server = ToolServer::start(owner.clone()).unwrap();
        let (code, _) = tokio::time::timeout(
            Duration::from_secs(10),
            invoke(
                &owner,
                &unit,
                &unit.cookie,
                &["commit", "--allow-empty", "-m", "hook reentry"],
            ),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(code, Some(0));
        assert!(unit.worktree.join("hook-completed").is_file());
        assert_eq!(
            owner
                .store
                .lock()
                .unwrap()
                .managed_effects(unit.id)
                .unwrap()
                .iter()
                .filter(|e| e.kind == "common_git"
                    && e.state == EffectState::Confirmed
                    && e.receipt.get("exit").is_some_and(|code| code == "0"))
                .count(),
            2
        );
        let mut sibling = crate::domain::Task::new(
            task.project_id,
            task.goal_id,
            "sibling".into(),
            "codex".into(),
        );
        owner.store.lock().unwrap().put_task(&mut sibling).unwrap();
        let (sibling, _) = attempts
            .prepare(sibling.id, "codex", "Implement", None)
            .await
            .unwrap();
        let (code, _) = tokio::time::timeout(
            Duration::from_secs(10),
            invoke(&owner, &sibling, &sibling.cookie, &["add", "answer.txt"]),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(code, Some(0));
    }

    #[tokio::test]
    async fn decoder_retains_partial_header_and_body_across_cancellation() {
        let (mut writer, mut reader) = tokio::io::duplex(1024);
        let mut decoder = Decoder::default();
        let bytes = serde_json::to_vec(&Frame::Cancel).unwrap();
        let header = (bytes.len() as u32).to_be_bytes();
        writer.write_all(&header[..2]).await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(5), decoder.receive(&mut reader))
                .await
                .is_err()
        );
        writer.write_all(&header[2..]).await.unwrap();
        writer.write_all(&bytes[..2]).await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(5), decoder.receive(&mut reader))
                .await
                .is_err()
        );
        writer.write_all(&bytes[2..]).await.unwrap();
        assert!(matches!(
            decoder.receive(&mut reader).await.unwrap(),
            Frame::Cancel
        ));
    }
}
