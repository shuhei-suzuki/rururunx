//! Owner-only real pass and the same Runtime wiring exercised with native fake CLIs.
use super::*;
use crate::{
    config::{PipelineCli, PipelineClis},
    execution::{
        pipeline::{
            LaunchResult, PipelineGit, PipelineRole, review, review_prompt, review_snapshot,
            step_arguments, test_support::FakeCli,
        },
        verification::hash,
    },
};
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::PathBuf,
    time::Instant,
};

const IMPLEMENT: &[u8] = b"Implement add(a, b) in src/calc.py so add(2, 3) == 5. Scope: src/**. Do not change tests or README, commit, or use the network. Leave the implementation for the runtime to commit.\n";
const CALC_TEST: &[u8] = b"import importlib.util\nimport unittest\n\nspec = importlib.util.spec_from_file_location('calc', 'src/calc.py')\ncalc = importlib.util.module_from_spec(spec)\nspec.loader.exec_module(calc)\n\nclass CalcTest(unittest.TestCase):\n    def test_add(self):\n        self.assertEqual(calc.add(2, 3), 5)\n";

// Values come only from the launcher's fixed reviewer environment. Never log argv beyond
// the subcommand, nor any unrelated environment. Unexpected values are redacted on read.
const GIT_PROBE: &str = r##"#!/bin/bash
set -eu
subcommand=''
skip=false
for arg in "$@"; do
    if "$skip"; then skip=false; continue; fi
    case "$arg" in
        -C|-c|--git-dir|--work-tree|--namespace|--config-env|--exec-path) skip=true ;;
        --version|--help) subcommand="$arg"; break ;;
        -*) ;;
        *) subcommand="$arg"; break ;;
    esac
done
printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$subcommand" "${GIT_CONFIG_COUNT+x}" "${GIT_CONFIG_COUNT-}" "${GIT_CONFIG_KEY_0+x}" "${GIT_CONFIG_KEY_0-}" "${GIT_CONFIG_VALUE_0+x}" "${GIT_CONFIG_VALUE_0-}" "${GIT_CONFIG_NOSYSTEM+x}" "${GIT_CONFIG_NOSYSTEM-}" >> "${TMPDIR:?}/git-probe.log"
exec /usr/bin/git "$@"
"##;

fn token_names() -> Vec<String> {
    let mut names: Vec<_> = std::env::vars_os()
        .filter_map(|(key, _)| {
            let name = key.to_string_lossy();
            let upper = name.to_ascii_uppercase();
            ["TOKEN", "SECRET", "PASSWORD", "API_KEY"]
                .iter()
                .any(|part| upper.contains(part))
                .then(|| name.into_owned())
        })
        .collect();
    names.sort();
    names
}

fn env_text(key: &str) -> Result<String> {
    let text = std::env::var(key).with_context(|| format!("missing {key}"))?;
    ensure!(
        !text.trim().is_empty() && !text.chars().any(char::is_control),
        "invalid {key}"
    );
    Ok(text)
}

fn env_path(key: &str) -> Result<PathBuf> {
    let path = PathBuf::from(env_text(key)?);
    ensure!(path.is_absolute(), "{key} must be absolute");
    Ok(path)
}

fn real_clis() -> Result<PipelineClis> {
    Ok(PipelineClis {
        codex: PipelineCli {
            executable: env_path("RRX_REAL_CODEX")?,
            version: env_text("RRX_REAL_CODEX_VERSION")?,
            model: std::env::var("RRX_REAL_CODEX_MODEL").unwrap_or_else(|_| "gpt-6.1-sol".into()),
            effort: Some(serde_json::from_value(json!(
                std::env::var("RRX_REAL_CODEX_EFFORT").unwrap_or_else(|_| "medium".into())
            ))?),
            codex_home: Some(match std::env::var_os("RRX_REAL_CODEX_HOME") {
                Some(_) => env_path("RRX_REAL_CODEX_HOME")?,
                None => env_path("HOME")?.join(".codex"),
            }),
        },
        claude: PipelineCli {
            executable: env_path("RRX_REAL_CLAUDE")?,
            version: env_text("RRX_REAL_CLAUDE_VERSION")?,
            model: std::env::var("RRX_REAL_CLAUDE_MODEL")
                .unwrap_or_else(|_| "claude-opus-5-5".into()),
            effort: None,
            codex_home: None,
        },
    })
}

fn packet(name: &str, bytes: &[u8]) -> Value {
    json!({"path":name,"sha256":hash(bytes),"bytes":bytes.len()})
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    file.sync_all()?;
    Ok(())
}

async fn sample_commit(f: &Fixture) -> Result<String> {
    let project = f
        .store()
        .project(f.project)?
        .context("sample project missing")?;
    let git = PipelineGit::new(&project.root)?;
    git.run(&["init", "-b", "main"]).await?;
    let mut index = Vec::new();
    for (name, bytes) in [
        ("README.md", b"Synthetic add(a, b) sample.\n".as_slice()),
        ("src/calc.py", b"".as_slice()),
        ("tests/test_calc.py", CALC_TEST),
    ] {
        let oid = git
            .invoke(
                &["hash-object", "-w", "--no-filters", "--stdin"],
                bytes,
                None,
                false,
                4096,
            )
            .await?;
        index.extend_from_slice(
            format!("100644 {}\t{name}\n", std::str::from_utf8(&oid)?.trim()).as_bytes(),
        );
    }
    git.invoke(&["update-index", "--index-info"], &index, None, false, 4096)
        .await?;
    let tree = git.text(&["write-tree"]).await?;
    let commit = git
        .invoke(
            &["commit-tree", &tree],
            b"synthetic sample\n",
            None,
            true,
            4096,
        )
        .await?;
    let commit = std::str::from_utf8(&commit)?.trim().to_owned();
    git.run(&["update-ref", "refs/heads/main", &commit]).await?;
    Ok(commit)
}

fn last_step(f: &Fixture) -> Result<PipelineStep> {
    f.store()
        .pipeline_goal_view(f.goal)?
        .steps
        .last()
        .cloned()
        .context("step missing")
}

fn record(f: &Fixture, kind: &str, start: Instant, result: &Result<bool>) -> Value {
    match last_step(f) {
        Ok(step) if step.kind == kind => {
            json!({"kind":kind,"state":step.state,"reason":step.body.get("reason")
            .cloned().unwrap_or_else(|| if result.is_err() { json!("harness_operation_failed") } else { Value::Null }),
            "duration_ms":start.elapsed().as_millis(),"adopted":matches!(result, Ok(true)) && step.state == "adopted"})
        }
        Ok(_) => json!({"kind":kind,"state":"not_planned","reason":"harness_planning_failed",
            "duration_ms":start.elapsed().as_millis(),"adopted":false}),
        Err(_) => json!({"kind":kind,"state":"unavailable","reason":"step_read_failed",
            "duration_ms":start.elapsed().as_millis(),"adopted":false}),
    }
}

fn plan(f: &Fixture, id: &str, kind: &str, prompt: &[u8], nonce: Option<&str>) -> Result<()> {
    let view = f.store().pipeline_goal_view(f.goal)?;
    let mut next = planned(
        id,
        view.steps.last().context("previous step missing")?.seq + 1,
        1,
        kind,
    );
    next.step_base = view.runs[0].head_commit.clone();
    next.input_sha256 = hash(prompt);
    if let Some(nonce) = nonce {
        next.body["review_nonce"] = json!(nonce);
    }
    let cursor = f.cursor(id);
    f.store().pipeline_plan_next(&f.owner, &cursor, &next, true)
}

fn fake_step(f: &Fixture, id: &str, kind: &str) -> Result<()> {
    let config = f.config.pipeline.as_ref().context("config missing")?;
    let step = last_step(f)?;
    let view = f.store().pipeline_goal_view(f.goal)?;
    let review_step = kind == "review";
    let cli = if review_step {
        &config.clis.claude
    } else {
        &config.clis.codex
    };
    let fake = FakeCli {
        directory: cli
            .executable
            .parent()
            .context("CLI parent missing")?
            .into(),
        cli: cli.clone(),
    };
    let work = if review_step {
        review_snapshot::snapshot_path(&config.work_root, id, step.seq, 1)?
    } else {
        PathBuf::from(view.runs[0].worktree.as_ref().context("worktree missing")?)
    };
    let attempt = config
        .work_root
        .join("runs")
        .join(id)
        .join(format!("{}-1", step.seq));
    let args = step_arguments(
        if review_step {
            PipelineRole::Review
        } else {
            PipelineRole::Implement
        },
        cli,
        &work,
        &attempt,
        if review_step {
            review::REVIEW_SCHEMA
        } else {
            ""
        },
    )?;
    fake.expect_arguments(&args.iter().map(String::as_str).collect::<Vec<_>>());
    fake.control("exec-git-probe", "");
    if review_step {
        fake.control(
            "exec-output",
            &serde_json::to_string(&json!({"type":"result","subtype":"success",
            "is_error":false,"modelUsage":{cli.model.clone():{}},"structured_output":{
            "schema":"rrx.review.v1","verdict":"approve","summary":"safe","findings":[]}}))?,
        );
    } else {
        fake.control("exec-calc", "def add(a, b):\n    return a + b\n");
        fake.control("exec-last", "Implementation complete.\n");
        fake.control("exec-output", "{\"type\":\"turn.completed\"}\n");
    }
    Ok(())
}

// Read after death, before test adoption can schedule TMPDIR cleanup. The raw log
// is never copied blindly into evidence; accept only the four known non-secret values.
fn git_probe(f: &Fixture) -> Value {
    let Ok(step) = last_step(f) else {
        return json!({"state":"step_unavailable","called":false});
    };
    let Some(tmp) = step.body["intent"]["tmpdir"].as_str() else {
        return json!({"state":"not_started","called":false});
    };
    let log = Path::new(tmp).join("git-probe.log");
    if matches!(fs::symlink_metadata(&log), Err(error) if error.kind() == std::io::ErrorKind::NotFound)
    {
        return json!({"state":"no_calls","called":false});
    }
    let bytes = match crate::execution::pipeline::adoption::read_regular(&log, 64 * 1024) {
        Ok(bytes) => bytes,
        Err(_) => return json!({"state":"log_unreadable","called":null}),
    };
    let mut calls = Vec::new();
    for line in String::from_utf8_lossy(&bytes).lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 9 {
            return json!({"state":"invalid_log","called":true});
        }
        let subcommand = fields[0];
        let mut env = serde_json::Map::new();
        for (i, (key, expected)) in [
            ("GIT_CONFIG_COUNT", "1"),
            ("GIT_CONFIG_KEY_0", "safe.bareRepository"),
            ("GIT_CONFIG_VALUE_0", "explicit"),
            ("GIT_CONFIG_NOSYSTEM", "1"),
        ]
        .iter()
        .enumerate()
        {
            let set = fields[1 + i * 2] == "x";
            let value = fields[2 + i * 2];
            env.insert((*key).into(), json!({"set":set,"value":if value.is_empty() || value == *expected { value } else { "[unexpected]" }}));
        }
        // Subcommands are names, not arbitrary argument strings or paths.
        calls.push(json!({"subcommand":if !subcommand.is_empty() && subcommand.len() <= 64 && subcommand.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b)) { subcommand } else { "[unexpected]" },"environment":env}));
    }
    json!({"state":"recorded","called":!calls.is_empty(),"calls":calls})
}

fn envelope(f: &Fixture, id: &str) -> Value {
    let Ok(step) = last_step(f) else {
        return json!({"state":"step_unavailable"});
    };
    let Some(config) = &f.config.pipeline else {
        return json!({"state":"config_missing"});
    };
    let path = config
        .work_root
        .join("runs")
        .join(id)
        .join(format!("{}-1/result.json", step.seq));
    let decoded = crate::execution::pipeline::adoption::read_regular(&path, review::REVIEW_BYTES)
        .ok()
        .filter(|bytes| review::scan_raw(bytes, 0).is_ok())
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
    match decoded.and_then(|value| value.as_object().cloned()) {
        Some(object) => {
            json!({"state":"recorded","top_level_keys":object.keys().collect::<Vec<_>>(),
            "model_usage_keys":object.get("modelUsage").and_then(Value::as_object).map(|o| o.keys().collect::<Vec<_>>())})
        }
        None => json!({"state":"absent_or_invalid"}),
    }
}

async fn pass_flow(real: bool, summary: &mut Value) -> Result<()> {
    let probe = tempfile::Builder::new()
        .prefix("rrx-git-probe-")
        .tempdir()?;
    let probe_path = probe.path().canonicalize()?;
    fs::write(probe_path.join("git"), GIT_PROBE)?;
    fs::set_permissions(probe_path.join("git"), fs::Permissions::from_mode(0o700))?;
    let clis = if real { Some(real_clis()?) } else { None };
    if let Some(clis) = &clis {
        summary["clis"] = cli_summary(clis);
    }
    let command = TestCommand {
        id: "test".into(),
        category: Category::Tests,
        program: "/usr/bin/python3".into(),
        args: vec![
            "-m".into(),
            "unittest".into(),
            "discover".into(),
            "-s".into(),
            "tests".into(),
        ],
        cwd: ".".into(),
        timeout_seconds: 60,
        drain_seconds: 5,
        stdout_bytes: 64 * 1024,
        stderr_bytes: 64 * 1024,
    };
    let qualification = std::sync::Arc::new(std::sync::Mutex::new(Value::Null));
    let inputs = FixtureInputs {
        clis,
        path_prefix: vec![probe_path],
        qualification: qualification.clone(),
        task: TaskDefinition {
            key: "task".into(),
            title: "Implement add(a, b) in src/calc.py".into(),
            acceptance_criteria: vec!["add(2, 3) == 5; the admitted unittest passes".into()],
            executor: "codex".into(),
            reviewers: vec!["claude".into()],
            workflow: WorkflowClass::Standard,
            risk: RiskClass::R2,
        },
    };
    let fixture_result = fixture_build(
        true,
        true,
        None,
        None,
        false,
        Some(vec![command]),
        Some(inputs),
    )
    .await;
    summary["qualification"] = qualification.lock().unwrap().clone();
    let f = fixture_result?;
    let config = f.config.pipeline.as_ref().context("config missing")?;
    summary["clis"] = cli_summary(&config.clis);
    // Walk the same launcher PATH, omitting only the first (probe) prefix.
    let paths = crate::execution::pipeline::test_support::reviewer_path(config)?;
    let git_path = paths.iter().skip(1).map(|dir| dir.join("git")).find(|p| {
        p.metadata()
            .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    });
    summary["probes"]["reviewer_git_without_probe"] =
        json!({"state":if git_path.is_some() {"resolved"} else {"not_found"},"path":git_path});
    let base = sample_commit(&f).await?;
    let project = f.store().project(f.project)?.context("project missing")?;
    let git = PipelineGit::new(&project.root)?;
    summary["sample_tree"] = packet("sample-base.tar", &git.run(&["archive", &base]).await?);
    let (f, runtime, id) = super::b5::prepare_fixture_at(f, base).await?;
    let mut active = true;
    for kind in ["prepare", "implement", "test", "review"] {
        if !active {
            continue;
        }
        let start = Instant::now();
        let result: Result<bool> = async {
            runtime.wait_pipeline_test_boundary().await;
            if kind == "prepare" { return runtime.prepare_pipeline_task(f.cursor(&id)).await; }
            let prompt = if kind == "implement" { IMPLEMENT.to_vec() } else if kind == "review" {
                let view = f.store().pipeline_goal_view(f.goal)?;
                let task = f.store().task(f.task)?.context("task missing")?;
                let data = json!({"title":task.title,"acceptance_criteria":task.acceptance_criteria,"scope":["src/**"]});
                let diff = git.diff(&view.runs[0].base_commit, &view.runs[0].head_commit).await?;
                let nonce = Uuid::new_v4().simple().to_string();
                let prompt = review_prompt(&nonce, &data, &diff)?;
                plan(&f, &id, kind, &prompt, Some(&nonce))?;
                prompt
            } else { Vec::new() };
            if kind != "review" { plan(&f, &id, kind, &prompt, None)?; }
            if !real && kind != "test" { fake_step(&f, &id, kind)?; }
            if kind != "test" {
                summary["prompts"][kind] = packet(&format!("{kind}-prompt.md"), &prompt);
            }
            let launched = match kind {
                "implement" => runtime.launch_pipeline_executor(f.cursor(&id), prompt).await?,
                "test" => runtime.launch_pipeline_test(f.cursor(&id)).await?,
                "review" => runtime.launch_pipeline_review(f.cursor(&id)).await?,
                _ => unreachable!(),
            };
            let launched = if matches!(launched, LaunchResult::Waiting) {
                runtime.wait_pipeline_test_boundary().await;
                // Waiting does not spawn; retain the same planned attempt and prompt.
                match kind {
                    "implement" => runtime.launch_pipeline_executor(f.cursor(&id), IMPLEMENT.to_vec()).await?,
                    "test" => runtime.launch_pipeline_test(f.cursor(&id)).await?,
                    "review" => runtime.launch_pipeline_review(f.cursor(&id)).await?,
                    _ => unreachable!(),
                }
            } else { launched };
            let LaunchResult::Started(custody) = launched else { return Ok(false); };
            if kind != "test" {
                let packet = summary["prompts"][kind].clone();
                summary["sent_prompts"].as_array_mut().unwrap().push(packet);
            }
            custody.wait().await?;
            summary["probes"]["git_forwarding"][kind] = git_probe(&f);
            if kind == "review" { summary["probes"]["envelope"] = envelope(&f, &id); }
            runtime.wait_pipeline_test_boundary().await;
            match kind {
                "implement" => runtime.adopt_pipeline_executor(f.cursor(&id)).await,
                "test" => runtime.adopt_pipeline_test(f.cursor(&id)).await,
                "review" => runtime.adopt_pipeline_review(f.cursor(&id)).await,
                _ => unreachable!(),
            }
        }.await;
        let observation = record(&f, kind, start, &result);
        active = observation["adopted"] == true;
        summary["steps"].as_array_mut().unwrap().push(observation);
        if kind == "implement" && active {
            let archived: Result<Vec<u8>> = async {
                let view = f.store().pipeline_goal_view(f.goal)?;
                git.run(&["archive", &view.runs[0].head_commit]).await
            }
            .await;
            match archived {
                Ok(bytes) => summary["review_tree"] = packet("sample-review.tar", &bytes),
                Err(_) => {
                    summary["evidence_failure"] = json!("review_tree_archive_failed");
                    active = false;
                }
            }
        }
    }
    let shutdown = runtime.shutdown().await;
    // Snapshots are sealed. Unseal/remove only this fixture's review snapshots.
    let view = f.store().pipeline_goal_view(f.goal)?;
    for step in view.steps.iter().filter(|s| s.kind == "review") {
        review_snapshot::remove(&review_snapshot::snapshot_path(
            &f.config.pipeline.as_ref().unwrap().work_root,
            &id,
            step.seq,
            step.attempt,
        )?)?;
    }
    shutdown?;
    ensure!(active, "one or more steps did not adopt");
    Ok(())
}

fn cli_summary(clis: &PipelineClis) -> Value {
    json!({"codex":{"version":clis.codex.version,"model":clis.codex.model,"effort":clis.codex.effort},
        "claude":{"version":clis.claude.version,"model":clis.claude.model}})
}

async fn run_pass(real: bool, out: &Path) {
    let mut summary = json!({"mode":if real {"real"} else {"fake"},"clis":null,"steps":[],
        "prompts":{},"sent_prompts":[],"sample_tree":null,"review_tree":null,
        "probes":{"envelope":{"state":"not_run"},"git_forwarding":{
            "implement":{"state":"not_run","called":false},"test":{"state":"not_run","called":false},
            "review":{"state":"not_run","called":false}},"reviewer_git_without_probe":{"state":"not_run","path":null}}});
    let start = Instant::now();
    let result = pass_flow(real, &mut summary).await;
    for kind in ["prepare", "implement", "test", "review"] {
        if !summary["steps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|step| step["kind"] == kind)
        {
            summary["steps"]
                .as_array_mut()
                .unwrap()
                .push(json!({"kind":kind,"state":"not_run",
                "reason":"previous_step_or_setup_not_adopted","duration_ms":0,"adopted":false}));
        }
    }
    summary["state"] = json!(if result.is_ok() { "adopted" } else { "failed" });
    summary["duration_ms"] = json!(start.elapsed().as_millis());
    // Never serialize anyhow errors: a CLI or filesystem failure may contain output.
    let mut packets = summary["sent_prompts"].as_array().unwrap().clone();
    for key in ["sample_tree", "review_tree"] {
        if !summary[key].is_null() {
            packets.push(summary[key].clone());
        }
    }
    let send_record = json!({"purpose":"Issue #106 synthetic Executor-test-Reviewer verification",
        "destination":if real {"Codex CLI (OpenAI) and Claude CLI (Anthropic)"} else {"local native fake CLIs"},
        "packets":packets,"content":"Synthetic sample trees and prompts only; no secrets, production data, PII, resource IDs or raw Agent output"});
    write_json(&out.join("summary.json"), &summary).unwrap();
    write_json(&out.join("send-record.json"), &send_record).unwrap();
    assert!(
        result.is_ok(),
        "pass failed; inspect summary.json (Agent output is not printed)"
    );
    if !real {
        assert_eq!(summary["steps"].as_array().unwrap().len(), 4);
        assert_eq!(
            summary["probes"]["git_forwarding"]["implement"]["called"],
            true
        );
        assert_eq!(
            summary["probes"]["git_forwarding"]["review"]["called"],
            true
        );
        assert_eq!(summary["probes"]["envelope"]["state"], "recorded");
        let calls = summary["probes"]["git_forwarding"]["review"]["calls"]
            .as_array()
            .unwrap();
        for call in calls {
            for (key, expected) in [
                ("GIT_CONFIG_COUNT", "1"),
                ("GIT_CONFIG_KEY_0", "safe.bareRepository"),
                ("GIT_CONFIG_VALUE_0", "explicit"),
                ("GIT_CONFIG_NOSYSTEM", "1"),
            ] {
                assert_eq!(
                    call["environment"][key],
                    json!({"set":true,"value":expected})
                );
            }
        }
        assert_eq!(
            summary["probes"]["envelope"]["model_usage_keys"],
            json!(["fixture-model"])
        );
    }
}

/// The normal host suite exercises Executor, admitted Seatbelt tests and Reviewer without real CLIs.
#[tokio::test]
async fn fake_cli_pass() {
    let out = tempfile::tempdir().unwrap();
    run_pass(false, out.path()).await;
}

/// Real CLIs require both --ignored --exact and the owner's explicit environment gate.
#[tokio::test]
#[ignore = "owner only: real provider calls on a synthetic sample"]
async fn real_cli_pass() {
    if std::env::var("RRX_REAL_CLI_PASS").as_deref() != Ok("1") {
        return;
    }
    let names = token_names();
    assert!(
        names.is_empty(),
        "token-like environment variable names: {}",
        names.join(", ")
    );
    let out = env_path("RRX_REAL_CLI_OUT").expect("absolute RRX_REAL_CLI_OUT required");
    fs::create_dir_all(&out).unwrap();
    assert!(
        !out.join("summary.json").exists() && !out.join("send-record.json").exists(),
        "output already used"
    );
    run_pass(true, &out).await;
}
