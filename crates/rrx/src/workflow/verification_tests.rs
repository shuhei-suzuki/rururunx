//! Managed Tests commands on the legacy Workflow Engine.
//!
//! FM §7.4 D4 (a) / §8.3 F2 (R): every scenario here published through the
//! legacy Engine's Native Implement, then Commit, then ran or refused the
//! managed Tests commands, which the D2 lane does not reach (S2). Each now
//! keeps its setup (provider, admitted Tests profile, verifier, engine) up to
//! the step into Implement and asserts the typed `ManagedBindingUnavailable`
//! with the §8.3 preimage unchanged and no Native effect. The former Commit
//! and Tests subjects are owned by the SC-N continuation.
use super::*;
use crate::execution::verification::{
    Applicability, Category, ManagedVerifier, TestCommand, TestsProfile,
};

fn profile(code: &str) -> TestsProfile {
    let python = std::fs::canonicalize("/usr/bin/python3").unwrap();
    let categories = [
        Category::Tests,
        Category::Typecheck,
        Category::Lint,
        Category::Build,
    ];
    TestsProfile {
        commands: categories
            .iter()
            .enumerate()
            .map(|(i, category)| TestCommand {
                id: format!("check_{i}"),
                category: *category,
                program: python.clone(),
                args: vec!["-c".into(), code.replace("COMMAND_INDEX", &i.to_string())],
                cwd: ".".into(),
                timeout_seconds: 10,
                drain_seconds: 1,
                stdout_bytes: 4096,
                stderr_bytes: 4096,
            })
            .collect(),
        applicability: categories
            .into_iter()
            .map(|c| (c, Applicability::Required))
            .collect(),
    }
}
/// The former `ready`: the Fixture, the admitted Tests profile (if any) and
/// the production engine with the verifier; then the F2 boundary at the
/// first Native phase (`publish` formerly stepped through it).
async fn refused(provider: &str, proposal: Option<TestsProfile>) {
    let f = Fixture::new(provider, WorkflowClass::Quick).await;
    let verifier = Arc::new(ManagedVerifier::new(f.owner.clone(), f.sources.clone()).unwrap());
    if let Some(proposal) = proposal {
        let version = f
            .owner
            .store
            .lock()
            .unwrap()
            .project(f.task.project_id)
            .unwrap()
            .unwrap()
            .version;
        verifier
            .admit_tests(f.task.project_id, version, proposal)
            .unwrap();
    }
    let engine = f.production_engine("pass").with_verifier(verifier).unwrap();
    let prepared = f.sources.prepare(f.task.id, provider).await.unwrap();
    engine.initialize(f.task.id, None).await.unwrap();
    early_phases(&engine, f.task.id, WorkflowClass::Quick).await;
    f.f2_refused(&engine).await;
    f.assert_no_native(prepared.id);
}

/// FM D4 R. Former subject: the admitted Tests commands complete on the
/// retained commit without Agent Sessions, with retained streams, and a
/// survivor workspace edit cannot replace the accepted source.
#[tokio::test]
async fn actual_tests_commands_complete_on_retained_commit_without_agent_sessions() {
    for provider in ["claude", "codex"] {
        refused(
            provider,
            Some(profile(
                "import os,pathlib,sys; assert 'MANDATORY_COMMITTED_RULE_A' in pathlib.Path('rules.md').read_text(); assert pathlib.Path(os.environ['CARGO_TARGET_DIR']).is_absolute(); print('actual-check-COMMAND_INDEX'); print('diagnostic-COMMAND_INDEX',file=sys.stderr)",
            )),
        )
        .await;
    }
}

/// FM D4 R. Former subject: an activated Tests contract without an admitted
/// profile waits and refuses a generic caller Passed, even with no Unit.
#[tokio::test]
async fn activated_tests_contract_refuses_generic_passed_even_with_no_unit() {
    refused("codex", None).await;
}

/// FM D4 R. Former subject: a failing, overflowing, timed-out or
/// source-writing command never certifies Tests.
#[tokio::test]
async fn actual_command_failure_overflow_timeout_and_source_drift_never_certify_tests() {
    for scenario in ["exit", "overflow", "timeout", "source"] {
        let code = match scenario {
            "exit" => "import sys; print('known failure'); sys.exit(3)",
            "overflow" => "print('x'*100000)",
            "timeout" => "import time; time.sleep(30)",
            "source" => {
                "import pathlib; p=pathlib.Path('rules.md'); p.chmod(0o600); p.write_text('changed input'); print('source writer exited zero')"
            }
            _ => unreachable!(),
        };
        let mut proposal = profile(code);
        if scenario == "overflow" {
            for c in &mut proposal.commands {
                c.stdout_bytes = 128
            }
        }
        if scenario == "timeout" {
            for c in &mut proposal.commands {
                c.timeout_seconds = 1
            }
        }
        refused("codex", Some(proposal)).await;
    }
}

/// FM D4 R. Former subject: accepted raw stream retrieval rejects tampering
/// and a short budget after a reopen.
#[tokio::test]
async fn accepted_raw_stream_retrieval_rejects_tampering_and_short_budget_after_reopen() {
    refused(
        "claude",
        Some(profile("print('independent retained evidence')")),
    )
    .await;
}

/// FM D4 R. Former subject: the command grant rejects Native Session, helper
/// delegation and terminal writers.
#[tokio::test]
async fn actual_command_grant_rejects_native_session_helper_delegation_and_terminal_writers() {
    refused(
        "codex",
        Some(profile("print('command-only actual control')")),
    )
    .await;
}

/// FM D4 R. Former subject: an owner drift after command preparation refuses
/// before the command intent.
#[tokio::test]
async fn actual_command_preparation_then_owner_drift_refuses_before_command_intent() {
    refused("codex", Some(profile("print('must not start')"))).await;
}

/// FM D4 R. Former subject: a cancelled command retains its observed receipt
/// without accepting Tests.
#[tokio::test]
async fn actual_command_cancel_retains_observed_receipt_without_accepting_tests() {
    refused(
        "codex",
        Some(profile(
            "import os,pathlib,time; pathlib.Path(os.environ['TMPDIR']).joinpath('command-running').write_text('ready'); print('before cancellation',flush=True); time.sleep(30)",
        )),
    )
    .await;
}

/// FM D4 R. Former subject: four command Tasks run in parallel and
/// cancelling one preserves its siblings' artifacts and receipts. The three
/// legacy `put_task` siblings only mattered after Commit and are dropped; the
/// first Task's parallel profile is kept.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn four_actual_command_tasks_cancel_one_preserving_sibling_artifacts_and_receipts() {
    let mut parallel_profile = profile(
        "import os,pathlib,time\np=pathlib.Path(os.environ['TMPDIR'])\nif COMMAND_INDEX == 0:\n p.joinpath('verification-ready').write_text('ready')\n while not p.joinpath('verification-release').exists(): time.sleep(0.01)\nprint(os.environ['RRX_UNIT_ID']+'-COMMAND_INDEX')",
    );
    for command in &mut parallel_profile.commands {
        command.timeout_seconds = 60;
    }
    refused("codex", Some(parallel_profile)).await;
}

/// FM D4 R. Former subject: a dropped resource-admission waiter or a Task
/// cancel creates no Unit or command.
#[tokio::test]
async fn actual_resource_admission_waiter_drop_and_task_cancel_create_no_unit_or_command() {
    refused("codex", Some(profile("print('must not start')"))).await;
}

/// FM D4 R. Former subject: a collected known exit (0, 7) survives a
/// retention I/O failure without acceptance.
#[tokio::test]
async fn actual_collected_known_exit_survives_retention_io_failure_without_acceptance() {
    for exit in [0, 7] {
        refused(
            "codex",
            Some(profile(&format!(
                "import sys; print('known-before-retention',flush=True); sys.exit({exit})"
            ))),
        )
        .await;
    }
}

/// FM D4 R. Former subject: an aborted Workflow future fences the command
/// Unit and quarantines unknown work.
#[tokio::test]
async fn actual_workflow_future_abort_fences_command_unit_and_quarantines_unknown_work() {
    refused(
        "codex",
        Some(profile(
            "import os,pathlib,time; pathlib.Path(os.environ['TMPDIR']).joinpath('abort-ready').write_text('ready'); print('not completion',flush=True); time.sleep(30)",
        )),
    )
    .await;
}

/// FM D4 R. Former subject: an aborted completion handoff preserves a known
/// success without accepting Tests.
#[tokio::test]
async fn actual_completion_handoff_abort_preserves_known_success_without_tests_acceptance() {
    refused("codex", Some(profile("print('known command success')"))).await;
}
