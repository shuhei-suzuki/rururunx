//! Test-only EOF observations and finite controls; no process authority.
use std::{
    fs::File,
    io::{self, Read},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvTimeoutError, TryRecvError},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReadPurpose {
    Ready,
    Eof,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Operation {
    Configure,
    Poll,
    Read,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ErrorRecord {
    kind: io::ErrorKind,
    errno: Option<i32>,
}
impl From<io::Error> for ErrorRecord {
    fn from(error: io::Error) -> Self {
        Self {
            kind: error.kind(),
            errno: error.raw_os_error(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReadOutcome {
    Ready,
    Eof,
    ReadyClosed,
    ReadyMismatch,
    UnexpectedData,
    Deadline {
        purpose: ReadPurpose,
        attempted_read: bool,
    },
    Cancelled,
    IoFailure {
        operation: Operation,
        error: ErrorRecord,
    },
}
impl ReadOutcome {
    fn error(operation: Operation, error: io::Error) -> Self {
        Self::IoFailure {
            operation,
            error: error.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeliveryAttempt {
    Sent,
    NotSent,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ReaderExit {
    pub read: ReadOutcome,
    pub delivery: DeliveryAttempt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JoinedReader {
    Completed(ReaderExit),
    Panicked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TransportOutcome {
    Received(ReadOutcome),
    Disconnected,
    TimedOut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LivePoint {
    Before,
    After,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LiveObservation {
    Alive,
    Exited,
    Error(ErrorRecord),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Fault {
    None,
    ReadEio,
    PollError,
    ReaderPanic,
    Disconnect,
    TransportTimeout,
    Cancel,
    ReadyReadEio,
    ReadyPollError,
    ReadyMissing,
    ReadyDeadline,
    ReadyClosed,
    ReadyCancel,
    BeforeError,
    AfterError,
    BeforeExited,
    AfterExited,
    InactiveDeadline,
}
impl Fault {
    pub fn live_observation(self, point: LivePoint) -> LiveObservation {
        match (self, point) {
            (Self::BeforeError, LivePoint::Before) | (Self::AfterError, LivePoint::After) => {
                LiveObservation::Error(io::Error::from_raw_os_error(libc::EIO).into())
            }
            (Self::BeforeExited, LivePoint::Before) | (Self::AfterExited, LivePoint::After) => {
                LiveObservation::Exited
            }
            _ => LiveObservation::Alive,
        }
    }

    fn read_error(self, purpose: ReadPurpose) -> bool {
        matches!(
            (self, purpose),
            (Self::ReadEio, ReadPurpose::Eof) | (Self::ReadyReadEio, ReadPurpose::Ready)
        )
    }

    fn poll_error(self, purpose: ReadPurpose) -> bool {
        matches!(
            (self, purpose),
            (Self::PollError, ReadPurpose::Eof) | (Self::ReadyPollError, ReadPurpose::Ready)
        )
    }
}

pub(super) fn poll_read(
    mut file: File,
    cancel: &AtomicBool,
    purpose: ReadPurpose,
    fault: Fault,
) -> ReadOutcome {
    if purpose == ReadPurpose::Ready && fault == Fault::ReadyCancel {
        cancel.store(true, Ordering::Release);
    }
    if purpose == ReadPurpose::Eof && fault == Fault::InactiveDeadline {
        return ReadOutcome::Deadline {
            purpose,
            attempted_read: false,
        };
    }
    // A successful nonblocking read attempt, including WouldBlock, is required
    // before an EOF reader deadline can become a causal delayed-EOF witness.
    // Poll timeout alone neither proves EOF nor permits a blocking read.
    let flags = match rustix::fs::fcntl_getfl(&file) {
        Ok(flags) => flags,
        Err(error) => return ReadOutcome::error(Operation::Configure, error.into()),
    };
    if let Err(error) = rustix::fs::fcntl_setfl(&file, flags | rustix::fs::OFlags::NONBLOCK) {
        return ReadOutcome::error(Operation::Configure, error.into());
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut attempted_read = false;
    let mut bytes = Vec::new();
    loop {
        if cancel.load(Ordering::Acquire) {
            return ReadOutcome::Cancelled;
        }
        if Instant::now() >= deadline {
            return ReadOutcome::Deadline {
                purpose,
                attempted_read,
            };
        }
        let mut fds = [rustix::event::PollFd::new(
            &file,
            rustix::event::PollFlags::IN,
        )];
        let timeout = rustix::event::Timespec {
            tv_sec: 0,
            tv_nsec: 50_000_000,
        };
        let polled = if fault.poll_error(purpose) {
            Err(rustix::io::Errno::IO)
        } else {
            rustix::event::poll(&mut fds, Some(&timeout))
        };
        match polled {
            Err(rustix::io::Errno::INTR) => continue,
            Err(error) => return ReadOutcome::error(Operation::Poll, error.into()),
            Ok(_) => {}
        }
        if cancel.load(Ordering::Acquire) {
            return ReadOutcome::Cancelled;
        }
        if Instant::now() >= deadline {
            return ReadOutcome::Deadline {
                purpose,
                attempted_read,
            };
        }
        let mut byte = [0u8; 1];
        let read = if fault.read_error(purpose) {
            Err(io::Error::from_raw_os_error(libc::EIO))
        } else {
            file.read(&mut byte)
        };
        match read {
            Ok(0) => {
                return match purpose {
                    ReadPurpose::Ready => ReadOutcome::ReadyClosed,
                    ReadPurpose::Eof => ReadOutcome::Eof,
                };
            }
            Ok(_) => {
                attempted_read = true;
                if purpose == ReadPurpose::Eof {
                    return ReadOutcome::UnexpectedData;
                }
                bytes.push(byte[0]);
                if bytes == b"ready\n" {
                    return ReadOutcome::Ready;
                }
                if !b"ready\n".starts_with(&bytes) {
                    return ReadOutcome::ReadyMismatch;
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                attempted_read = true;
            }
            Err(error) => return ReadOutcome::error(Operation::Read, error),
        }
    }
}

pub(super) fn receive_reader(
    receiver: &Receiver<ReadOutcome>,
    reader: &JoinHandle<ReaderExit>,
) -> TransportOutcome {
    loop {
        match receiver.recv_timeout(Duration::from_millis(50)) {
            Ok(read) => return TransportOutcome::Received(read),
            Err(RecvTimeoutError::Disconnected) => return TransportOutcome::Disconnected,
            Err(RecvTimeoutError::Timeout) if reader.is_finished() => {
                // Producer send precedes thread completion. Recheck after the
                // completion observation so a send at the sampling boundary is
                // never mistaken for a transport timeout. Join is still required.
                return match receiver.try_recv() {
                    Ok(read) => TransportOutcome::Received(read),
                    Err(TryRecvError::Disconnected) => TransportOutcome::Disconnected,
                    Err(TryRecvError::Empty) => TransportOutcome::TimedOut,
                };
            }
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
}

#[derive(Debug)]
pub(super) enum WorkerObservation {
    Pass,
    DelayedEof,
    Setup(String),
    NoResult(String),
}
impl WorkerObservation {
    pub fn emit(self) {
        match self {
            Self::Pass => println!("P4C RESULT pass"),
            Self::DelayedEof => println!("P4C RESULT fail:inherited writer delayed EOF"),
            Self::Setup(reason) => println!("P4C SETUP fail:{reason}"),
            Self::NoResult(reason) => println!("P4C NO_RESULT {reason}"),
        }
    }
}

pub(super) fn classify_inheritance(
    in_group: bool,
    ready: ReadOutcome,
    before: LiveObservation,
    after: LiveObservation,
    transport: TransportOutcome,
    joined: &[JoinedReader],
) -> WorkerObservation {
    let no_result = |reason: &str| WorkerObservation::NoResult(reason.to_owned());
    // Inspect the join independently of the transport result, including panic.
    let exit = match joined {
        [JoinedReader::Completed(exit)] => exit,
        [JoinedReader::Panicked] => return no_result("EOF reader panicked"),
        _ => return no_result("EOF reader join missing or ambiguous"),
    };
    let received = match transport {
        TransportOutcome::Received(read) => read,
        TransportOutcome::Disconnected => return no_result("EOF observation channel disconnected"),
        TransportOutcome::TimedOut => return no_result("EOF observation transport timed out"),
    };
    if exit.delivery != DeliveryAttempt::Sent || exit.read != received {
        return no_result("EOF transport and joined reader disagree");
    }
    if matches!(
        ready,
        ReadOutcome::IoFailure { .. } | ReadOutcome::Cancelled
    ) {
        return WorkerObservation::NoResult(format!("ready observation failed: {ready:?}"));
    }
    if matches!(before, LiveObservation::Error(_)) || matches!(after, LiveObservation::Error(_)) {
        return WorkerObservation::NoResult(format!(
            "liveness observation failed: before={before:?} after={after:?}"
        ));
    }
    // Even with failed ready/liveness prerequisites, an EOF observation failure
    // remains an observation failure rather than a fabricated causal result.
    let eof_valid = matches!(
        received,
        ReadOutcome::Eof
            | ReadOutcome::Deadline {
                purpose: ReadPurpose::Eof,
                attempted_read: true,
            }
    );
    if !eof_valid {
        return WorkerObservation::NoResult(format!("EOF observation failed: {received:?}"));
    }
    if !in_group {
        return WorkerObservation::Setup("inner group not established".into());
    }
    if ready != ReadOutcome::Ready {
        return WorkerObservation::Setup(format!("ready not established: {ready:?}"));
    }
    if before != LiveObservation::Alive || after != LiveObservation::Alive {
        return WorkerObservation::Setup("child not live at observations".into());
    }
    match received {
        ReadOutcome::Eof => WorkerObservation::Pass,
        ReadOutcome::Deadline {
            purpose: ReadPurpose::Eof,
            attempted_read: true,
        } => WorkerObservation::DelayedEof,
        _ => no_result("unrecognized EOF observation"),
    }
}

#[derive(Clone, Copy)]
pub(super) struct WorkerInput {
    pub cloexec: bool,
    pub high: bool,
    pub fault: Fault,
}

#[derive(Clone, Copy)]
enum Expected {
    Setup,
    NoResult,
}

struct Control {
    fault_case: &'static str,
    normal_case: &'static str,
    fault: Fault,
    expected: Expected,
    witness: &'static str,
}

const CONTROLS: &[Control] = &[
    Control {
        fault_case: "EOF_READ_EIO",
        normal_case: "EOF_READ_EIO_NORMAL",
        fault: Fault::ReadEio,
        expected: Expected::NoResult,
        witness: "read: IoFailure { operation: Read",
    },
    Control {
        fault_case: "EOF_POLL_ERROR",
        normal_case: "EOF_POLL_ERROR_NORMAL",
        fault: Fault::PollError,
        expected: Expected::NoResult,
        witness: "read: IoFailure { operation: Poll",
    },
    Control {
        fault_case: "EOF_READER_PANIC",
        normal_case: "EOF_READER_PANIC_NORMAL",
        fault: Fault::ReaderPanic,
        expected: Expected::NoResult,
        witness: "joined=[Panicked]",
    },
    Control {
        fault_case: "EOF_DISCONNECT",
        normal_case: "EOF_DISCONNECT_NORMAL",
        fault: Fault::Disconnect,
        expected: Expected::NoResult,
        witness: "transport=Disconnected",
    },
    Control {
        fault_case: "EOF_TRANSPORT_TIMEOUT",
        normal_case: "EOF_TRANSPORT_TIMEOUT_NORMAL",
        fault: Fault::TransportTimeout,
        expected: Expected::NoResult,
        witness: "transport=TimedOut",
    },
    Control {
        fault_case: "EOF_CANCEL",
        normal_case: "EOF_CANCEL_NORMAL",
        fault: Fault::Cancel,
        expected: Expected::NoResult,
        witness: "read: Cancelled",
    },
    Control {
        fault_case: "EOF_READY_EIO",
        normal_case: "EOF_READY_EIO_NORMAL",
        fault: Fault::ReadyReadEio,
        expected: Expected::NoResult,
        witness: "ready=IoFailure { operation: Read",
    },
    Control {
        fault_case: "EOF_READY_POLL_ERROR",
        normal_case: "EOF_READY_POLL_ERROR_NORMAL",
        fault: Fault::ReadyPollError,
        expected: Expected::NoResult,
        witness: "ready=IoFailure { operation: Poll",
    },
    Control {
        fault_case: "EOF_READY_MISSING",
        normal_case: "EOF_READY_MISSING_NORMAL",
        fault: Fault::ReadyMissing,
        expected: Expected::Setup,
        witness: "ready=ReadyMismatch",
    },
    Control {
        fault_case: "EOF_READY_DEADLINE",
        normal_case: "EOF_READY_DEADLINE_NORMAL",
        fault: Fault::ReadyDeadline,
        expected: Expected::Setup,
        witness: "ready=Deadline { purpose: Ready, attempted_read: true }",
    },
    Control {
        fault_case: "EOF_READY_CLOSED",
        normal_case: "EOF_READY_CLOSED_NORMAL",
        fault: Fault::ReadyClosed,
        expected: Expected::Setup,
        witness: "ready=ReadyClosed",
    },
    Control {
        fault_case: "EOF_READY_CANCEL",
        normal_case: "EOF_READY_CANCEL_NORMAL",
        fault: Fault::ReadyCancel,
        expected: Expected::NoResult,
        witness: "ready=Cancelled",
    },
    Control {
        fault_case: "EOF_BEFORE_ERROR",
        normal_case: "EOF_BEFORE_ERROR_NORMAL",
        fault: Fault::BeforeError,
        expected: Expected::NoResult,
        witness: "before=Error(",
    },
    Control {
        fault_case: "EOF_AFTER_ERROR",
        normal_case: "EOF_AFTER_ERROR_NORMAL",
        fault: Fault::AfterError,
        expected: Expected::NoResult,
        witness: "after=Error(",
    },
    Control {
        fault_case: "EOF_BEFORE_EXITED",
        normal_case: "EOF_BEFORE_EXITED_NORMAL",
        fault: Fault::BeforeExited,
        expected: Expected::Setup,
        witness: "before=Exited",
    },
    Control {
        fault_case: "EOF_AFTER_EXITED",
        normal_case: "EOF_AFTER_EXITED_NORMAL",
        fault: Fault::AfterExited,
        expected: Expected::Setup,
        witness: "after=Exited",
    },
    Control {
        fault_case: "EOF_INACTIVE_DEADLINE",
        normal_case: "EOF_INACTIVE_DEADLINE_NORMAL",
        fault: Fault::InactiveDeadline,
        expected: Expected::NoResult,
        witness: "read: Deadline { purpose: Eof, attempted_read: false }",
    },
];

pub(super) fn worker_input(case: &str) -> Option<WorkerInput> {
    let regular = |cloexec, high, fault| WorkerInput {
        cloexec,
        high,
        fault,
    };
    match case {
        "C1" => return Some(regular(false, false, Fault::None)),
        "C2a" => return Some(regular(true, false, Fault::None)),
        "C2f" => return Some(regular(false, true, Fault::None)),
        _ => {}
    }
    for control in CONTROLS {
        if case == control.fault_case {
            return Some(regular(false, false, control.fault));
        }
        if case == control.normal_case {
            return Some(regular(false, false, Fault::None));
        }
    }
    None
}

#[test]
fn observation_failures_have_normal_worker_pairs() {
    use super::owner::{Classification, WorkerGroupOwner};
    for control in CONTROLS {
        let expected = match control.expected {
            Expected::Setup => Classification::Setup,
            Expected::NoResult => Classification::NoResult,
        };
        for (case, classification, negative) in [
            (control.fault_case, expected, true),
            (control.normal_case, Classification::Pass, false),
        ] {
            let owner = WorkerGroupOwner::start(case)
                .unwrap_or_else(|error| panic!("worker setup case={case}: {error}"));
            let report = owner.finish();
            assert!(
                report.complete,
                "case={case}: incomplete recovery: {report:?}"
            );
            assert_eq!(
                report.classification, classification,
                "case={case}: {report:?}"
            );
            assert!(!report.killed(false), "case={case}: {report:?}");
            assert!(!report.killed(true), "case={case}: {report:?}");
            assert!(
                report.trace.starts_with(&["S1:group", "S2:exited"]),
                "case={case}: {report:?}"
            );
            assert_eq!(
                &report.trace[report.trace.len() - 3..],
                &["S4:accepted", "S6:reaped", "S7:ESRCH"],
                "case={case}: {report:?}"
            );
            assert!(
                report.stdout.contains("P4C EOF observations "),
                "case={case}: reader/cleanup evidence missing: {report:?}"
            );
            if negative {
                assert!(
                    report.stdout.contains(control.witness),
                    "case={case}: fault witness missing: {report:?}"
                );
            } else {
                assert!(
                    report.stdout.contains("ready=Ready before=Alive after=Alive transport=Received(Eof) joined=[Completed(ReaderExit { read: Eof, delivery: Sent })]"),
                    "case={case}: normal observation pair missing: {report:?}"
                );
            }
        }
    }
}

#[test]
fn incomplete_or_conflicting_reader_evidence_is_no_result() {
    let eof = ReaderExit {
        read: ReadOutcome::Eof,
        delivery: DeliveryAttempt::Sent,
    };
    let classify = |transport, joined: &[JoinedReader]| {
        classify_inheritance(
            true,
            ReadOutcome::Ready,
            LiveObservation::Alive,
            LiveObservation::Alive,
            transport,
            joined,
        )
    };
    assert!(matches!(
        classify(TransportOutcome::Received(ReadOutcome::Eof), &[]),
        WorkerObservation::NoResult(_)
    ));
    assert!(matches!(
        classify(
            TransportOutcome::Received(ReadOutcome::Eof),
            &[JoinedReader::Completed(eof), JoinedReader::Completed(eof)]
        ),
        WorkerObservation::NoResult(_)
    ));
    assert!(matches!(
        classify(
            TransportOutcome::Received(ReadOutcome::Deadline {
                purpose: ReadPurpose::Eof,
                attempted_read: true,
            }),
            &[JoinedReader::Completed(eof)]
        ),
        WorkerObservation::NoResult(_)
    ));
    for delivery in [DeliveryAttempt::NotSent, DeliveryAttempt::Failed] {
        assert!(matches!(
            classify(
                TransportOutcome::Received(ReadOutcome::Eof),
                &[JoinedReader::Completed(ReaderExit {
                    read: ReadOutcome::Eof,
                    delivery,
                })]
            ),
            WorkerObservation::NoResult(_)
        ));
    }
    let deadline = ReadOutcome::Deadline {
        purpose: ReadPurpose::Eof,
        attempted_read: true,
    };
    assert!(matches!(
        classify(
            TransportOutcome::TimedOut,
            &[JoinedReader::Completed(ReaderExit {
                read: deadline,
                delivery: DeliveryAttempt::Sent,
            })]
        ),
        WorkerObservation::NoResult(_)
    ));
}
