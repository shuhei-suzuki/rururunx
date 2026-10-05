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
const CHUNK: usize = 16 * 1024;
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolRequest {
    unit: UnitId,
    cookie: String,
    tool: String,
    args: Vec<String>,
    cwd: PathBuf,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum Frame {
    Input(Vec<u8>),
    InputClosed,
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    Exit(Option<i32>),
    Refused(String),
}
async fn write_frame<T: Serialize>(
    stream: &mut (impl AsyncWrite + Unpin),
    value: &T,
) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    ensure!(bytes.len() <= FRAME, "IPC frame too large");
    stream.write_u32(bytes.len() as u32).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;
    Ok(())
}
async fn read_frame<T: serde::de::DeserializeOwned>(
    stream: &mut (impl AsyncRead + Unpin),
) -> Result<T> {
    let count = stream.read_u32().await? as usize;
    ensure!(count <= FRAME, "IPC frame too large");
    let mut bytes = vec![0; count];
    stream.read_exact(&mut bytes).await?;
    Ok(serde_json::from_slice(&bytes)?)
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
        let operation=OperationId::new();let plan=tools::plan(&profile,&unit,&request.tool,&request.args,operation)?;
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
        let _git=if plan.serialized_git {Some(owner.git_gate.lock().await)}else{None};
        let mut command=Command::new(&plan.program);
        command.args(&plan.args).current_dir(&request.cwd).envs(profile.environment(&unit.cookie,owner.ipc_path())?)
            .envs(&plan.environment).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        // Keep native settings/auth inherited; remove only Git routing overrides.
        for key in ["GIT_DIR","GIT_WORK_TREE","GIT_COMMON_DIR","GIT_INDEX_FILE","GIT_OBJECT_DIRECTORY","GIT_ALTERNATE_OBJECT_DIRECTORIES","GIT_NAMESPACE"] {command.env_remove(key);}
        let mut child={
            let mut store=owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?;
            let now=store.execution_unit(unit.id)?;
            ensure!(now.generation==unit.generation && now.session_id==unit.session_id && now.owner_epoch==unit.owner_epoch,"tool admission retired while queued");
            store.reserve_managed_effect(&now.authority(),&ManagedEffect {id:operation,unit_id:unit.id,scope:unit.scope.clone(),kind:plan.kind.into(),
                idempotency_key:format!("tool-{operation}"),expected_target:plan.docker_name.clone().unwrap_or_else(||request.cwd.to_string_lossy().into()),
                state:EffectState::Pending,receipt:BTreeMap::new(),version:1})?;
            match process::OwnedProcess::spawn(&mut command) {
                Ok(child)=>child,
                Err(e)=>{store.reconcile_managed_effect(operation,1,EffectState::Unknown,BTreeMap::new())?;return Err(e);}
            }
        };
        let mut stdin=child.child.stdin.take().context("tool stdin missing")?;
        let mut stdout=child.child.stdout.take().context("tool stdout missing")?;
        let mut stderr=child.child.stderr.take().context("tool stderr missing")?;
        let (mut input,mut output)=stream.split();
        let exchanged={let pumping=async {
            let mut out=vec![0;CHUNK];let mut err=vec![0;CHUNK];
            let mut in_open=true;let mut out_open=true;let mut err_open=true;
            while out_open || err_open {tokio::select! {
                frame=read_frame::<Frame>(&mut input),if in_open=>{
                    match frame? {Frame::Input(bytes)=>{ensure!(bytes.len()<=CHUNK,"tool input chunk too large");stdin.write_all(&bytes).await?;},
                        Frame::InputClosed=>{stdin.shutdown().await?;in_open=false;},_=>anyhow::bail!("invalid tool input frame")}
                },
                n=stdout.read(&mut out),if out_open=>{let n=n?;if n==0{out_open=false}else{write_frame(&mut output,&Frame::Stdout(out[..n].to_vec())).await?;}},
                n=stderr.read(&mut err),if err_open=>{let n=n?;if n==0{err_open=false}else{write_frame(&mut output,&Frame::Stderr(err[..n].to_vec())).await?;}}
            }}Ok::<_,anyhow::Error>(())
        };
        tokio::pin!(pumping);
        let fenced=async {loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if current(&owner,&request).is_err(){break;}
        }};
        tokio::time::timeout(Duration::from_secs(1800),async {tokio::select! {
            r=&mut pumping=>{r?;child.exited().await?;Ok::<_,anyhow::Error>(())},
            r=child.exited()=>{r?;let _=child.signal_group();tokio::time::timeout(Duration::from_secs(2),&mut pumping).await.context("tool output drain incomplete")?},
            _=fenced=>Err(anyhow::anyhow!("tool authority closed"))
        }}).await};
        let stopped=child.stop_and_reap().await;
        let observed=exchanged.ok().and_then(Result::ok);
        let mut receipt=BTreeMap::new();
        if let Ok(stopped)=&stopped {
            receipt.insert("exit".into(),stopped.status.code().map_or_else(||"signal".into(),|n|n.to_string()));
            receipt.insert("group_cleanup".into(),if stopped.group_error.is_some(){"unknown"}else{"requested"}.into());
        }
        // Docker needs label verification/reconciliation; an exit code alone is not a remote receipt.
        let state=if plan.docker_name.is_none() && observed.is_some() && stopped.is_ok(){EffectState::Confirmed}else{EffectState::Unknown};
        owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.reconcile_managed_effect(operation,1,state,receipt)?;
        let stopped=stopped?;
        write_frame(&mut output,&Frame::Exit(if observed.is_some(){stopped.status.code()}else{None})).await?;
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
        let (mut input, mut output) = stream.into_split();
        let sender = tokio::spawn(async move {
            let mut stdin = tokio::io::stdin();
            let mut bytes = vec![0; CHUNK];
            loop {
                let n = stdin.read(&mut bytes).await?;
                if n == 0 {
                    write_frame(&mut output, &Frame::InputClosed).await?;
                    break;
                }
                write_frame(&mut output, &Frame::Input(bytes[..n].to_vec())).await?;
            }
            Ok::<_, anyhow::Error>(())
        });
        let result = async {
            loop {
                match read_frame::<Frame>(&mut input).await? {
                    Frame::Stdout(bytes) => tokio::io::stdout().write_all(&bytes).await?,
                    Frame::Stderr(bytes) => tokio::io::stderr().write_all(&bytes).await?,
                    Frame::Exit(code) => break Ok::<_, anyhow::Error>(Some(code.unwrap_or(125))),
                    Frame::Refused(reason) => anyhow::bail!(reason),
                    _ => anyhow::bail!("invalid Runtime output frame"),
                }
            }
        }
        .await;
        sender.abort();
        result
    });
    runtime.shutdown_background();
    result
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
            },
        )
        .await?;
        write_frame(&mut stream, &Frame::InputClosed).await?;
        let mut output = Vec::new();
        loop {
            match read_frame::<Frame>(&mut stream).await? {
                Frame::Stdout(bytes) => {
                    ensure!(output.len() + bytes.len() <= 4096, "fixture stdout bound");
                    output.extend(bytes);
                }
                Frame::Stderr(_) => {}
                Frame::Exit(code) => return Ok((code, output)),
                Frame::Refused(reason) => anyhow::bail!(reason),
                _ => anyhow::bail!("invalid fixture output"),
            }
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
}
