//! Explicit opt-in real native adapter review/resume on two owned temporary repos.
//! cargo run --example native_codex_session -- /absolute/codex review|resume
use anyhow::{Context, Result, ensure};
use rrx::{adapter::*, codex::CodexAdapter, domain::*, git::WorktreeManager, state::Store};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("/usr/bin/git")
        .current_dir(root)
        .args(args)
        .env_clear()
        .envs(std::env::vars_os().filter(|(key, _)| !key.to_string_lossy().starts_with("GIT_")))
        .output()?;
    ensure!(
        output.status.success(),
        "owned fixture Git operation failed"
    );
    Ok(String::from_utf8(output.stdout)?.trim().into())
}
async fn finished(adapter: &CodexAdapter, session: &Session) -> Result<SessionStatus> {
    let mut status = adapter.subscribe(session.into())?;
    let result = tokio::time::timeout(Duration::from_secs(180), async {
        while !status.borrow().terminal() {
            status.changed().await.context("native supervisor closed")?;
        }
        Ok::<_, anyhow::Error>(status.borrow().clone())
    })
    .await;
    match result {
        Ok(result) => result,
        Err(_) => {
            let _ = adapter.stop(session.into()).await;
            anyhow::bail!("explicit native fixture deadline exceeded")
        }
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let mut arguments = std::env::args_os().skip(1);
    let executable = PathBuf::from(arguments.next().context("absolute executable required")?);
    let (resume, execute) = match arguments.next().as_deref() {
        Some(mode) if mode == "review" => (false, false),
        Some(mode) if mode == "resume" => (true, false),
        Some(mode) if mode == "execute" => (false, true),
        _ => anyhow::bail!("explicit review, resume or execute opt-in required"),
    };
    ensure!(arguments.next().is_none(), "unexpected arguments");
    let fixture = tempfile::Builder::new()
        .prefix("rrx-native-session-")
        .tempdir_in("/tmp")?;
    let root = fixture.path().canonicalize()?;
    let mut store = Store::open(&root.join("state.sqlite3"))?;
    let mut projects = vec![];
    for name in ["own", "foreign"] {
        let repository = root.join(name);
        std::fs::create_dir(&repository)?;
        git(&repository, &["init", "-b", "main"])?;
        std::fs::write(repository.join(".gitignore"), "worktree/\n")?;
        std::fs::write(repository.join("scope-sentinel"), name)?;
        git(&repository, &["add", ".gitignore", "scope-sentinel"])?;
        git(
            &repository,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-m",
                "fixture",
            ],
        )?;
        let mut project = Project::new(
            name.into(),
            repository.clone(),
            rrx::git::repository_identity(&repository, "main")?,
            "main".into(),
        );
        store.put_project(&mut project)?;
        projects.push(project);
    }
    let project = projects[0].clone();
    let mut goal = Goal::new(
        project.id,
        "native review proof".into(),
        vec![CompletionCriterion {
            id: "review".into(),
            description: "native structured verdict".into(),
            satisfied: false,
            evidence: None,
        }],
    );
    store.put_goal(&mut goal)?;
    let mut task = Task::new(project.id, goal.id, "native review".into(), "codex".into());
    task.issue = Some(42);
    store.put_task(&mut task)?;
    let worktree = WorktreeManager::create(&mut store, task.id)?;
    if !execute {
        WorktreeManager::lock_review(
            &mut store,
            task.id,
            &worktree.revision,
            "native immutable review fixture",
        )?;
    }
    let scope = task.scope();
    let store = Arc::new(Mutex::new(store));
    let adapter = CodexAdapter::new("codex".into(), executable, store.clone())?;
    let payload = if execute {
        "Create the file result.txt in your current owning worktree with exactly NATIVE_EXECUTED followed by a newline. Use native tools and preserve existing files. Then return EXECUTED and a short reason in the JSON schema."
    } else {
        "Review only the supplied synthetic function clamp(x)=x-1 and contract that output >=0 for every integer x. Return DEFECT with x=0 as a mathematical witness. Do not execute target operations."
    };
    let mut input = PreparedInput {
        scope: scope.clone(),
        kind: if execute {
            InputKind::ContextPack
        } else {
            InputKind::ReviewBundle
        },
        revision: worktree.revision,
        version: 1,
        source_versions: BTreeMap::from([("synthetic-contract".into(), "v1".into())]),
        payload: payload.into(),
    };
    let request = LaunchRequest {
        project,
        scope,
        worktree: worktree.worktree,
        role: if execute {
            SessionRole::Executor
        } else {
            SessionRole::Reviewer
        },
        mode: LaunchMode::NonInteractive,
        input: input.clone(),
        environment: BTreeMap::new(),
        model: None,
        effort: None,
    };
    let verdict = if execute { "EXECUTED" } else { "DEFECT" };
    let schema = json!({"type":"object","properties":{"verdict":{"type":"string","enum":[verdict,"FAILED"]},"reason":{"type":"string"}},"required":["verdict","reason"],"additionalProperties":false});
    let session = adapter.start_structured(request, schema).await?;
    let first = finished(&adapter, &session).await?;
    ensure!(
        first.session.state == SessionState::Exited,
        "native review failed: {:?}",
        first.failure
    );
    ensure!(
        adapter.transport_succeeded(&first),
        "native completion journal did not validate Workflow transport"
    );
    let answer: Value = serde_json::from_slice(&first.stdout)?;
    ensure!(
        answer["verdict"] == verdict,
        "native fixture verdict failed: {answer}"
    );
    if execute {
        ensure!(
            std::fs::read_to_string(session.worktree.join("result.txt"))
                .with_context(|| format!("native requested artifact missing; answer={answer}"))?
                == "NATIVE_EXECUTED\n",
            "native executor did not create exact owning artifact"
        );
        ensure!(
            !projects[1].root.join("result.txt").exists(),
            "foreign Project artifact created"
        );
    }
    let foreign = SessionRef {
        id: session.id,
        scope: Scope::project(projects[1].id),
    };
    ensure!(
        adapter.status(foreign).await.unwrap_err().kind == ErrorKind::OwnershipMismatch,
        "foreign scope accepted"
    );
    let mut second = None;
    if resume {
        input.version = 2;
        input.payload="Continue the same supplied review with witness x=-3. Return DEFECT and its exact numeric output; do not perform operations.".into();
        adapter.checkpoint((&session).into(), input).await?;
        let resumed = adapter.resume((&session).into()).await?;
        ensure!(
            resumed.id == session.id && resumed.native_ref == session.native_ref,
            "native resume changed identity"
        );
        let status = finished(&adapter, &resumed).await?;
        ensure!(
            status.session.state == SessionState::Exited,
            "native resume failed: {:?}",
            status.failure
        );
        ensure!(
            adapter.transport_succeeded(&status),
            "resumed native completion journal unavailable"
        );
        second = Some(serde_json::from_slice::<Value>(&status.stdout)?);
    }
    let usage = adapter
        .usage(
            (&session).into(),
            if execute { "implementation" } else { "review" }.into(),
            if execute {
                None
            } else {
                Some(if resume { 2 } else { 1 })
            },
        )
        .await?;
    let persisted = store
        .lock()
        .unwrap()
        .session(session.id)?
        .context("Session record missing")?
        .0;
    ensure!(
        persisted.state == SessionState::Exited && persisted.pid.is_none(),
        "native reservation still active"
    );
    println!(
        "{}",
        serde_json::to_string(
            &json!({"session_id":session.id,"native_uuid":session.native_ref,"model":session.model,"effort":session.effort,"structured_answer":answer,"resumed_answer":second,"foreign_scope_rejected":true,"durable_exit_verified":true,"usage":usage})
        )?
    );
    adapter.release((&session).into())?;
    Ok(())
}
