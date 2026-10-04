//! Private deterministic diagnostic guards and actual owned-child observations.
//! No test clock/category is a production budget, permission or death claim.
use super::{diagnostics::*, *};

fn run(body: &str, facts: Collector) -> io::Result<bool> {
    let mut command = std::process::Command::new("/bin/sh");
    command.args(["-c", body]);
    inspect_command_recorded(command, 42, |_| {}, facts)
}
fn open() -> Collector {
    let mut facts = Collector::new();
    facts.hooks.clock = Clock::Open;
    facts
}
fn observed(error: &io::Error) -> Snapshot {
    snapshot(error).expect("shared inspector return omitted diagnostic attachment")
}

#[test]
fn actual_frame_guard_facts_are_safe_and_success_preserves_live_dead() {
    for (body, site, refusal, validation, kind) in [
        (
            "printf '42 42 Z\\n'; printf 'DO_NOT_RENDER_NATIVE_BYTES' >&2",
            Site::Stderr,
            Framing::Stderr,
            "not_reached",
            io::ErrorKind::InvalidData,
        ),
        (
            "printf '42 42 Z\\n'; exit 1",
            Site::Exit,
            Framing::Exit,
            "not_reached",
            io::ErrorKind::InvalidData,
        ),
        (
            "printf '43 42 Z\\n'",
            Site::Validation,
            Framing::MissingLeader,
            "missing_leader",
            io::ErrorKind::InvalidData,
        ),
        (
            "printf '42 42 Z'",
            Site::Validation,
            Framing::Incomplete,
            "incomplete_frame",
            io::ErrorKind::InvalidData,
        ),
    ] {
        // Open logical clock isolates guard evidence from native scheduling. Real
        // child/pipe collection and reaping still execute; unit/seam credit only.
        let error = run(body, open()).unwrap_err();
        let sample = observed(&error);
        assert_eq!(error.kind(), kind);
        assert_eq!(sample.site, Some(site));
        assert_eq!(sample.refusal, Some(Refusal::Frame(refusal)));
        assert_eq!(sample.cleanup, Cleanup::ReapedByStatus);
        assert!(matches!(sample.kill, Kill::NotRequested));
        for endpoint in [sample.stdout.unwrap(), sample.stderr.unwrap()] {
            assert_eq!(endpoint.eof, Eof::Observed);
            assert!(endpoint.calls > endpoint.would_block + endpoint.interrupted);
        }
        let status = sample.status.unwrap();
        assert!(status.calls > 0);
        assert!(!matches!(status.exit, Exit::Unavailable));
        let text = error.to_string();
        assert!(text.contains(&format!("validation={validation}")), "{text}");
        assert!(!text.contains("DO_NOT_RENDER_NATIVE_BYTES"));
        assert_eq!(error.raw_os_error(), None);
    }
    assert!(run("printf '42 42 Z\\n'", open()).unwrap());
    assert!(!run("printf '42 42 S\\n'", open()).unwrap());
}

#[test]
fn adjacent_deadline_sites_and_deferred_validation_are_exact_unit_guards() {
    for (site, stream, body) in [
        (Site::LoopDeadline, StreamKind::None, "printf '42 42 Z\\n'"),
        (
            Site::DrainDeadline,
            StreamKind::Stdout,
            "printf '42 42 Z\\n'",
        ),
        (
            Site::DrainDeadline,
            StreamKind::Stderr,
            "printf '42 42 Z\\n'",
        ),
        (
            Site::AfterReadDeadline,
            StreamKind::Stdout,
            "printf '42 42 Z\\n'",
        ),
        (
            Site::AfterReadDeadline,
            StreamKind::Stderr,
            "printf '42 42 Z\\n'; printf x >&2",
        ),
        (
            Site::AfterStatusDeadline,
            StreamKind::None,
            "printf '42 42 Z\\n'",
        ),
        (
            Site::AfterValidationDeadline,
            StreamKind::None,
            "printf '42 42 S\\n'",
        ),
        (
            Site::AfterValidationDeadline,
            StreamKind::None,
            "printf '43 42 Z\\n'",
        ),
    ] {
        let mut facts = Collector::new();
        facts.hooks.clock = Clock::Expire(site, stream);
        let error = run(body, facts).unwrap_err();
        let sample = observed(&error);
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert_eq!(sample.site, Some(site));
        assert_eq!(sample.stream, stream);
        assert_eq!(sample.refusal, Some(Refusal::Deadline));
        assert!(sample.elapsed_us.is_some());
        if site == Site::LoopDeadline
            || (site == Site::DrainDeadline && stream == StreamKind::Stdout)
        {
            assert!(sample.status.is_none());
            assert_eq!(sample.stdout.unwrap().calls, 0);
        }
        if site == Site::AfterReadDeadline {
            let endpoint = if stream == StreamKind::Stdout {
                sample.stdout
            } else {
                sample.stderr
            }
            .unwrap();
            assert!(endpoint.bytes > 0 && endpoint.calls > 0);
        }
        if site == Site::AfterValidationDeadline {
            assert!(matches!(
                sample.validation,
                Validation::Live | Validation::Refused(Refusal::Frame(Framing::MissingLeader))
            ));
            assert_eq!(sample.cleanup, Cleanup::ReapedByStatus);
        } else {
            assert!(matches!(sample.validation, Validation::NotReached));
        }
    }
}

#[test]
fn setup_read_and_allocation_injections_preserve_reached_measurements() {
    for (site, stream) in [
        (Site::Endpoint, StreamKind::Stdout),
        (Site::Endpoint, StreamKind::Stderr),
        (Site::Setup, StreamKind::Stdout),
        (Site::Setup, StreamKind::Stderr),
        (Site::Read, StreamKind::Stdout),
        (Site::Read, StreamKind::Stderr),
        (Site::Allocation, StreamKind::Stdout),
        (Site::Allocation, StreamKind::Stderr),
    ] {
        let mut facts = open();
        facts.hooks.failure = Some((site, stream, io::ErrorKind::PermissionDenied));
        let error = run("printf '42 42 Z\\n'; printf x >&2", facts).unwrap_err();
        let sample = observed(&error);
        assert_eq!(sample.site, Some(site));
        assert_eq!(sample.stream, stream);
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert!(matches!(sample.cleanup, Cleanup::KillThenReaped));
        assert!(!matches!(sample.kill, Kill::NotRequested));
        if site == Site::Endpoint && stream == StreamKind::Stdout {
            assert!(sample.stdout.is_none() && sample.stderr.is_none());
        } else if site == Site::Endpoint {
            assert!(sample.stdout.is_some() && sample.stderr.is_none());
        } else {
            let endpoint = if stream == StreamKind::Stdout {
                sample.stdout
            } else {
                sample.stderr
            }
            .unwrap();
            if site == Site::Read || site == Site::Setup {
                assert_eq!(endpoint.calls, 0);
            }
            if site == Site::Allocation {
                assert!(endpoint.bytes > 0 && endpoint.calls > 0);
            }
        }
    }
}

#[test]
fn settled_child_status_error_is_relinquishment_not_cleanup_authority() {
    let child = std::process::Command::new("/bin/sh")
        .args(["-c", ":"])
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut inspector = Inspector {
        child,
        unreaped: true,
        exit: None,
        settled_fixture: None,
    };
    let mut facts = open();
    facts.hooks.status_error_after_reap = true;
    // Actual Child remains owned through synthetic branch and cleanup. No numeric
    // callback PID adoption/signal/rescue. Successful try_wait settles it first.
    let watchdog = Instant::now() + Duration::from_secs(2);
    let result = loop {
        if Instant::now() >= watchdog {
            break io::Error::other("unit status-injection watchdog");
        }
        match inspector.observe(&mut facts) {
            Err(e) => break e,
            Ok(()) => std::thread::sleep(Duration::from_millis(1)),
        }
    };
    cleanup_failure(&mut inspector, &mut facts);
    let error = facts.attach(result);
    // Cleanup-before-assertion, including mutations: cfg(test) recorded operations
    // stub any attempted second kill/wait on the already reaped actual Child.
    assert!(facts.hooks.actual_reaped.is_some());
    assert!(inspector.child.try_wait().unwrap().is_some());
    assert_eq!(facts.hooks.synthetic_kills, 0);
    assert_eq!(facts.hooks.synthetic_waits, 0);
    let sample = observed(&error);
    assert_eq!(sample.site, Some(Site::Status));
    assert_eq!(sample.cleanup, Cleanup::Relinquished);
    assert!(matches!(sample.kill, Kill::NotRequested));
    assert_eq!(error.kind(), io::ErrorKind::Other);
    let status = sample.status.unwrap();
    assert_eq!(status.source_kind, Some(io::ErrorKind::PermissionDenied));
    assert_eq!(status.errors, 1);
    assert_eq!(status.exit, Exit::Unavailable);
}

#[test]
fn actual_inner_read_errno_is_preserved_and_outer_rendering_is_value_free() {
    let directory = tempfile::tempdir().unwrap();
    let child = std::process::Command::new("/bin/sh")
        .args(["-c", ":"])
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut inspector = Inspector {
        child,
        unreaped: true,
        exit: None,
        settled_fixture: None,
    };
    let stderr = inspector.child.stderr.take().unwrap();
    let mut facts = open();
    let original = complete_recorded(
        &mut inspector,
        File::open(directory.path()).unwrap(),
        File::from(OwnedFd::from(stderr)),
        Instant::now() + BUDGET,
        &mut facts,
    )
    .err()
    .unwrap();
    cleanup_failure(&mut inspector, &mut facts);
    assert_eq!(
        original.raw_os_error(),
        Some(rustix::io::Errno::ISDIR.raw_os_error())
    );
    let error = facts.attach(original);
    let sample = observed(&error);
    assert_eq!(sample.site, Some(Site::Read));
    assert_eq!(sample.stream, StreamKind::Stdout);
    assert_eq!(sample.stdout.unwrap().calls, 1);
    assert_eq!(sample.stdout.unwrap().bytes, 0);
    assert_eq!(sample.stdout.unwrap().eof, Eof::NotObserved);
    assert!(sample.status.is_none());
    assert_eq!(sample.cleanup, Cleanup::KillThenReaped);
    assert!(
        error
            .to_string()
            .starts_with("native inspection stream read failed")
    );
    assert!(
        !error
            .to_string()
            .contains(&directory.path().to_string_lossy().to_string())
    );
}

#[test]
fn entry_hop_spawn_facts_pass_through_unchanged_resolver_without_effects() {
    let directory = tempfile::tempdir().unwrap();
    let error = super::super::resolve_macos_signal_result(Err(rustix::io::Errno::PERM), || {
        super::super::process_group_inspection(&directory.path().join("does-not-exist"), 42)
    })
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    let sample = observed(&error);
    assert_eq!(sample.site, Some(Site::Spawn));
    assert_eq!(sample.cleanup, Cleanup::NotReached);
    assert!(sample.stdout.is_none() && sample.stderr.is_none() && sample.status.is_none());
    assert!(sample.spawn_us.is_some() && sample.elapsed_us.is_none());
    assert!(!error.to_string().contains("does-not-exist"));
}

#[test]
fn maximum_value_formatter_is_finite_and_counters_saturate_without_raw_bodies() {
    let all_sites = [
        Site::Input,
        Site::Spawn,
        Site::Endpoint,
        Site::Setup,
        Site::LoopDeadline,
        Site::DrainDeadline,
        Site::Read,
        Site::Allocation,
        Site::AfterReadDeadline,
        Site::Budget,
        Site::Status,
        Site::AfterStatusDeadline,
        Site::Exit,
        Site::Stderr,
        Site::Validation,
        Site::AfterValidationDeadline,
    ];
    let all_frames = [
        Framing::InvalidLeader,
        Framing::Incomplete,
        Framing::Utf8,
        Framing::Identifier,
        Framing::MissingState,
        Framing::UnknownState,
        Framing::ZombieSuffix,
        Framing::StateSuffix,
        Framing::MissingPid,
        Framing::MissingGroup,
        Framing::MissingRowState,
        Framing::UnexpectedRow,
        Framing::MissingLeader,
        Framing::StdoutBudget,
        Framing::StderrBudget,
        Framing::Exit,
        Framing::Stderr,
    ];
    for site in all_sites {
        for code in all_frames {
            let mut facts = Collector::new();
            facts.fail(
                site,
                StreamKind::Stderr,
                Refusal::Frame(code),
                io::ErrorKind::InvalidData,
            );
            let endpoint = StreamFacts {
                calls: u64::MAX,
                bytes: u64::MAX,
                would_block: u64::MAX,
                interrupted: u64::MAX,
                eof: Eof::Observed,
            };
            facts.facts.stdout = Some(endpoint);
            facts.facts.stderr = Some(endpoint);
            facts.facts.status = Some(StatusFacts {
                calls: u64::MAX,
                pending: u64::MAX,
                interrupted: u64::MAX,
                errors: u64::MAX,
                exit: Exit::Signaled,
                source_kind: Some(io::ErrorKind::Other),
            });
            facts.facts.validation = Validation::Refused(Refusal::Frame(code));
            facts.facts.cleanup = Cleanup::WaitFailed;
            facts.facts.kill = Kill::Error(io::ErrorKind::PermissionDenied);
            facts.facts.elapsed_us = Some(u64::MAX);
            facts.facts.spawn_us = Some(u64::MAX);
            facts.facts.cleanup_us = Some(u64::MAX);
            let text = facts.attach(framing(code)).to_string();
            assert!(
                text.len() < 2048,
                "maximum static formatter exceeded finite bound"
            );
            assert!(text.contains("cleanup=wait_failed_uncertain"));
        }
    }
    // Exercise every non-framing formatter enum arm with maximum numeric fields.
    let mut facts = Collector::new();
    for cause in [
        Refusal::Deadline,
        Refusal::Allocation,
        Refusal::MissingEndpoint,
        Refusal::Io(io::ErrorKind::NotFound),
        Refusal::Io(io::ErrorKind::PermissionDenied),
        Refusal::Io(io::ErrorKind::Interrupted),
        Refusal::Io(io::ErrorKind::InvalidInput),
        Refusal::Io(io::ErrorKind::InvalidData),
        Refusal::Io(io::ErrorKind::TimedOut),
        Refusal::Io(io::ErrorKind::WouldBlock),
        Refusal::Io(io::ErrorKind::UnexpectedEof),
        Refusal::Io(io::ErrorKind::BrokenPipe),
        Refusal::Io(io::ErrorKind::OutOfMemory),
        Refusal::Io(io::ErrorKind::WriteZero),
        Refusal::Io(io::ErrorKind::Other),
    ] {
        for stream in [StreamKind::None, StreamKind::Stdout, StreamKind::Stderr] {
            facts.fail(Site::Endpoint, stream, cause, io::ErrorKind::Other);
            assert!(
                facts
                    .attach(io::Error::other("DO_NOT_RENDER_ERROR_BODY"))
                    .to_string()
                    .len()
                    < 2048
            );
        }
    }
    for (cleanup, kill, validation, exit, eof) in [
        (
            Cleanup::NotReached,
            Kill::NotRequested,
            Validation::NotReached,
            Exit::Unavailable,
            Eof::NotObserved,
        ),
        (
            Cleanup::ReapedByStatus,
            Kill::Ok,
            Validation::Dead,
            Exit::Success,
            Eof::Observed,
        ),
        (
            Cleanup::KillThenReaped,
            Kill::Error(io::ErrorKind::Other),
            Validation::Live,
            Exit::Nonzero,
            Eof::Pending,
        ),
        (
            Cleanup::Relinquished,
            Kill::NotRequested,
            Validation::NotReached,
            Exit::Signaled,
            Eof::Pending,
        ),
    ] {
        facts.facts.cleanup = cleanup;
        facts.facts.kill = kill;
        facts.facts.validation = validation;
        facts.facts.status = Some(StatusFacts {
            exit,
            ..Default::default()
        });
        facts.facts.stdout = Some(StreamFacts {
            eof,
            ..Default::default()
        });
        let text = facts
            .attach(io::Error::other("DO_NOT_RENDER_ERROR_BODY"))
            .to_string();
        assert!(text.len() < 2048 && !text.contains("DO_NOT_RENDER_ERROR_BODY"));
    }
    assert!(
        Collector::new()
            .attach(io::Error::other("private text"))
            .to_string()
            .contains("site=unavailable")
    );
    // Actual prepared File reads exercise saturating increments, not just rendering.
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bytes");
    std::fs::write(&path, b"x").unwrap();
    let mut stream = Stream::new(File::open(&path).unwrap(), Framing::StdoutBudget).unwrap();
    let mut facts = open();
    let endpoint = facts.stream(StreamKind::Stdout).unwrap();
    endpoint.calls = u64::MAX;
    endpoint.bytes = u64::MAX;
    assert!(
        stream
            .drain_recorded(Instant::now() + BUDGET, &mut facts, StreamKind::Stdout)
            .unwrap()
    );
    assert_eq!(facts.facts.stdout.unwrap().calls, u64::MAX);
    assert_eq!(facts.facts.stdout.unwrap().bytes, u64::MAX);
}

#[test]
fn prepared_byte_budget_refusal_records_read_without_claiming_pipe_throughput() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("next-byte");
    std::fs::write(&path, b"\n").unwrap();
    for (stream_kind, code) in [
        (StreamKind::Stdout, Framing::StdoutBudget),
        (StreamKind::Stderr, Framing::StderrBudget),
    ] {
        let mut stream = Stream::new(File::open(&path).unwrap(), code).unwrap();
        stream.bytes.resize(LIMIT, b' ');
        let mut facts = open();
        let error = stream
            .drain_recorded(Instant::now() + BUDGET, &mut facts, stream_kind)
            .unwrap_err();
        assert_eq!(error.to_string(), code.message());
        assert_eq!(facts.facts.site, Some(Site::Budget));
        assert_eq!(facts.facts.refusal, Some(Refusal::Frame(code)));
        let endpoint = facts.stream(stream_kind).unwrap();
        assert_eq!(endpoint.calls, 1);
        assert_eq!(endpoint.bytes, 1);
        assert_eq!(endpoint.eof, Eof::Pending);
    }
}

#[test]
fn invalid_owned_leader_fails_before_any_spawn_and_has_unavailable_observations() {
    let called = std::cell::Cell::new(false);
    let error = inspect_command(std::process::Command::new("/bin/sh"), 1, |_| {
        called.set(true)
    })
    .unwrap_err();
    assert!(!called.get());
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    let sample = observed(&error);
    assert_eq!(sample.site, Some(Site::Input));
    assert_eq!(sample.refusal, Some(Refusal::Frame(Framing::InvalidLeader)));
    assert!(sample.stdout.is_none() && sample.stderr.is_none() && sample.status.is_none());
    assert!(sample.spawn_us.is_none() && sample.elapsed_us.is_none());
    assert_eq!(sample.cleanup, Cleanup::NotReached);
}
