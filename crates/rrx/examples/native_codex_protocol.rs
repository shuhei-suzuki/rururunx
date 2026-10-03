//! Explicit opt-in native protocol proof on two temporary Git repositories.
//! `cargo run --example native_codex_protocol -- /absolute/codex [review]`
//! Without `review`, no model turn is sent. Existing native authentication remains
//! authoritative; the example never changes account/configuration/trust state.
use anyhow::{Context, Result, ensure};
use rrx::{
    codex::{
        policy::{DecisionPolicy, native_environment},
        protocol::{Event, UsageTracker},
        transport::NativeServer,
    },
    domain::Project,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut arguments = std::env::args_os().skip(1);
    let executable = PathBuf::from(
        arguments
            .next()
            .context("absolute native Codex executable required")?,
    )
    .canonicalize()?;
    let (review, execute) = match arguments.next() {
        None => (false, false),
        Some(mode) if mode == "review" => (true, false),
        Some(mode) if mode == "execute-policy" => (false, true),
        Some(_) => anyhow::bail!("only explicit review or execute-policy mode is supported"),
    };
    ensure!(arguments.next().is_none(), "unexpected arguments");
    let fixture = tempfile::Builder::new()
        .prefix("rrx-codex-proof-")
        .tempdir_in("/tmp")?;
    let root = fixture.path().canonicalize()?;
    let own = root.join("own");
    let foreign = root.join("foreign");
    for repository in [&own, &foreign] {
        std::fs::create_dir(repository)?;
        ensure!(
            std::process::Command::new("/usr/bin/git")
                .args(["init", "-b", "main"])
                .arg(repository)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()?
                .success(),
            "fixture Git init failed"
        );
        std::fs::write(
            repository.join("scope-sentinel"),
            if repository == &own { "OWN" } else { "FOREIGN" },
        )?;
    }
    let project = Project::new(
        "native-protocol-fixture".into(),
        own.clone(),
        "fixture".into(),
        "main".into(),
    );
    let environment = || {
        native_environment(
            std::env::vars_os(),
            std::slice::from_ref(&project),
            &project,
            &BTreeMap::new(),
        )
    };
    let uncertain = Arc::new(AtomicBool::new(false));
    let mut discovery =
        NativeServer::launch(&executable, &own, None, environment()?, uncertain.clone()).await?;
    let discovered = async {
        let config = discovery
            .rpc
            .call("config/read", json!({"cwd":own,"includeLayers":false}))
            .await?;
        let policy = if execute {
            DecisionPolicy::for_executor(&config["config"])?
        } else {
            DecisionPolicy::from_native(&config["config"])?
        };
        let account = discovery
            .rpc
            .call("account/read", json!({"refreshToken":false}))
            .await?;
        Ok::<_, rrx::adapter::AdapterError>((policy, account["account"].is_object()))
    }
    .await;
    discovery.shutdown().await?;
    let (policy, native_account_observed) = discovered?;
    ensure!(
        !uncertain.load(Ordering::SeqCst),
        "discovery process group remains uncertain"
    );
    let mut native = NativeServer::launch(
        &executable,
        &own,
        Some(&policy),
        environment()?,
        uncertain.clone(),
    )
    .await?;
    let verified = async {
        let config = native.rpc.call("config/read",json!({"cwd":own,"includeLayers":false})).await?;
        policy.verify_configuration(&config["config"])?;
        if execute { let environment=native.rpc.call("environment/status",json!({"environmentId":"local"})).await?; policy.verify_local_environment(&environment)?; }
        let started = native.rpc.call("thread/start",policy.thread_parameters(&own)).await?;
        let thread = policy.verify_thread(&started,&own)?;
        let mut cursor: Option<String> = None;
        let mut inventory_done = false;
        for _ in 0..32 {
            let page = native.rpc.call("mcpServerStatus/list",json!({"threadId":thread,"cursor":cursor,"limit":100})).await?;
            cursor = policy.verify_inventory_page(&page)?;
            if cursor.is_none() { inventory_done=true; break; }
        }
        if !inventory_done { return Err(rrx::adapter::AdapterError { kind:rrx::adapter::ErrorKind::ParseFailure,message:"inventory pagination exceeded bound".into() }); }
        let mut filesystem_canaries=None;
        if execute {
            let mut exits=Vec::new();
            for command in [vec!["/bin/cat".to_owned(),own.join("scope-sentinel").display().to_string()],vec!["/bin/cat".to_owned(),foreign.join("scope-sentinel").display().to_string()],vec!["/usr/bin/touch".to_owned(),own.join("owned-write").display().to_string()],vec!["/usr/bin/touch".to_owned(),foreign.join("foreign-write").display().to_string()]] {
                let response=native.rpc.call("command/exec",json!({"command":command,"cwd":own,"permissionProfile":policy.profile_name(),"timeoutMs":5000,"outputBytesCap":4096})).await?;
                exits.push(response["exitCode"].as_i64().ok_or_else(||rrx::adapter::AdapterError {kind:rrx::adapter::ErrorKind::ParseFailure,message:"native filesystem canary exit unavailable".into()})?);
            }
            if exits[0]!=0 || exits[1]==0 || exits[2]!=0 || exits[3]==0 || !own.join("owned-write").is_file() || foreign.join("foreign-write").exists() { return Err(rrx::adapter::AdapterError {kind:rrx::adapter::ErrorKind::ProcessFailure,message:"native executor filesystem isolation failed".into()}); }
            filesystem_canaries=Some(json!({"own_read":true,"foreign_read_denied":true,"own_write":true,"foreign_write_denied":true}));
        }
        let mut answer = None;
        let mut usage = None;
        if review {
            let schema=json!({"type":"object","properties":{"verdict":{"type":"string","enum":["DEFECT"]},"reason":{"type":"string"}},"required":["verdict","reason"],"additionalProperties":false});
            let turn=native.rpc.call("turn/start",json!({"threadId":thread,"input":[{"type":"text","text":"Review only this supplied synthetic function and contract, without tools or target operations: clamp(x)=x-1, required output >=0 for all integer x. Return DEFECT and a short mathematical witness in the requested JSON schema."}],"outputSchema":schema})).await?;
            let turn = turn["turn"]["id"].as_str().ok_or_else(|| rrx::adapter::AdapterError { kind:rrx::adapter::ErrorKind::ParseFailure,message:"native turn ID unavailable".into() })?.to_owned();
            let mut tracker = UsageTracker::new(thread.clone(),turn.clone());
            let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(120);
            loop {
                match tokio::time::timeout_at(deadline,native.rpc.receive()).await.map_err(|_| rrx::adapter::AdapterError { kind:rrx::adapter::ErrorKind::Timeout,message:"native fixture review timed out".into() })?? {
                    Event::Notification { method,params } => {
                        if params.get("threadId").is_some_and(|id| id != &json!(thread)) { return Err(rrx::adapter::AdapterError { kind:rrx::adapter::ErrorKind::OwnershipMismatch,message:"foreign native event in fixture".into() }); }
                        match method.as_str() {
                            "thread/tokenUsage/updated" => { tracker.update(&params)?; },
                            "item/completed" => {
                                let item=&params["item"];
                                match item["type"].as_str() {
                                    Some("agentMessage") => { answer=Some(serde_json::from_str::<Value>(item["text"].as_str().ok_or_else(|| rrx::adapter::AdapterError { kind:rrx::adapter::ErrorKind::ParseFailure,message:"native answer missing".into() })?).map_err(|_| rrx::adapter::AdapterError { kind:rrx::adapter::ErrorKind::ParseFailure,message:"native answer is not JSON".into() })?); },
                                    Some("userMessage" | "reasoning" | "plan") => {},
                                    _ => return Err(rrx::adapter::AdapterError { kind:rrx::adapter::ErrorKind::ProcessFailure,message:"unexpected target operation in decision fixture".into() }),
                                }
                            },
                            "turn/completed" => {
                                if params["turn"]["id"] != turn || params["turn"]["status"] != "completed" { return Err(rrx::adapter::AdapterError { kind:rrx::adapter::ErrorKind::ProcessFailure,message:"native turn failed or identity changed".into() }); }
                                usage=tracker.total; break;
                            },
                            _ => {},
                        }
                    },
                    Event::Request { id,.. } => {
                        native.rpc.send(json!({"id":id,"error":{"code":-32601,"message":"Decision fixture cannot approve target operations"}})).await?;
                        return Err(rrx::adapter::AdapterError { kind:rrx::adapter::ErrorKind::ProcessFailure,message:"unexpected decision fixture callback".into() });
                    },
                    _ => return Err(rrx::adapter::AdapterError { kind:rrx::adapter::ErrorKind::ParseFailure,message:"unsolicited native response".into() }),
                }
            }
        }
        Ok::<_,rrx::adapter::AdapterError>(json!({"native_account_observed":native_account_observed,"native_uuid":thread,"private_socket_verified":native.socket().is_ok(),"native_model":started["model"],"native_effort":started["reasoningEffort"],"instruction_source_count":started["instructionSources"].as_array().map(Vec::len),"scoped_profile_verified":true,"filesystem_canaries":filesystem_canaries,"native_execution_features":{"shell_tool":config["config"]["features"]["shell_tool"],"unified_exec":config["config"]["features"]["unified_exec"],"code_mode":config["config"]["features"]["code_mode"],"code_mode_host":config["config"]["features"]["code_mode_host"],"code_mode_only":config["config"]["features"]["code_mode_only"]},"native_approval_policy":started["approvalPolicy"],"native_approval_reviewer":started["approvalsReviewer"],"external_tool_count":0,"model_turn_sent":review,"structured_answer":answer,"cumulative_tokens":usage,"estimated_cost":null}))
    }.await;
    native.shutdown().await?;
    ensure!(
        !uncertain.load(Ordering::SeqCst),
        "native process group remains uncertain"
    );
    let mut result = verified?;
    result["owned_group_cleanup_verified"] = json!(true);
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
