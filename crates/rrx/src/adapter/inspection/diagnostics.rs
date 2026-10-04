//! Private value-free inspector facts. No fact is process or permission authority.
use std::{
    fmt, io,
    process::ExitStatus,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Site {
    Input,
    Spawn,
    Endpoint,
    Setup,
    LoopDeadline,
    DrainDeadline,
    Read,
    Allocation,
    AfterReadDeadline,
    Budget,
    Status,
    AfterStatusDeadline,
    Exit,
    Stderr,
    Validation,
    AfterValidationDeadline,
}
impl Site {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Input => "input_guard",
            Self::Spawn => "spawn",
            Self::Endpoint => "endpoint_setup",
            Self::Setup => "nonblocking_setup",
            Self::LoopDeadline => "deadline_loop_entry",
            Self::DrainDeadline => "deadline_drain_entry",
            Self::Read => "stream_read",
            Self::Allocation => "stream_allocation",
            Self::AfterReadDeadline => "deadline_after_read",
            Self::Budget => "stream_budget",
            Self::Status => "direct_child_status",
            Self::AfterStatusDeadline => "deadline_after_status",
            Self::Exit => "exit_guard",
            Self::Stderr => "stderr_guard",
            Self::Validation => "frame_validation",
            Self::AfterValidationDeadline => "deadline_after_validation",
        }
    }
    fn message(self) -> &'static str {
        match self {
            Self::Spawn => "native process inspection spawn failed",
            Self::Endpoint => "missing inspection endpoint",
            Self::Setup => "native inspection nonblocking setup failed",
            Self::Read => "native inspection stream read failed",
            Self::Allocation => "native inspection allocation failed",
            Self::Status => "native inspector reap ownership uncertain",
            _ => "native process inspection failed",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum StreamKind {
    Stdout,
    Stderr,
    #[default]
    None,
}
impl StreamKind {
    fn name(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
            Self::None => "none",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Framing {
    InvalidLeader,
    Incomplete,
    Utf8,
    Identifier,
    MissingState,
    UnknownState,
    ZombieSuffix,
    StateSuffix,
    MissingPid,
    MissingGroup,
    MissingRowState,
    UnexpectedRow,
    MissingLeader,
    StdoutBudget,
    StderrBudget,
    Exit,
    Stderr,
}
impl Framing {
    pub(super) fn message(self) -> &'static str {
        match self {
            Self::InvalidLeader => "invalid owned inspection leader",
            Self::Incomplete => "incomplete native process inspection frame",
            Self::Utf8 => "invalid native inspection UTF-8",
            Self::Identifier => "invalid native inspection PID/group",
            Self::MissingState => "missing native process state",
            Self::UnknownState => "unknown native process state",
            Self::ZombieSuffix => "invalid zombie process state suffix",
            Self::StateSuffix => "invalid native process state suffix",
            Self::MissingPid => "missing native PID",
            Self::MissingGroup => "missing native process group",
            Self::MissingRowState => "missing native process state",
            Self::UnexpectedRow => "unexpected/duplicate native inspection row",
            Self::MissingLeader => "native inspection expected leader missing",
            Self::StdoutBudget => "native inspection stdout exceeds byte budget",
            Self::StderrBudget => "native inspection stderr exceeds byte budget",
            Self::Exit => "native process inspection exit failed",
            Self::Stderr => "native process inspection emitted diagnostics",
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::InvalidLeader => "invalid_leader",
            Self::Incomplete => "incomplete_frame",
            Self::Utf8 => "invalid_utf8",
            Self::Identifier => "invalid_identifier",
            Self::MissingState | Self::MissingRowState => "missing_state",
            Self::UnknownState => "unknown_state",
            Self::ZombieSuffix => "invalid_zombie_suffix",
            Self::StateSuffix => "invalid_state_suffix",
            Self::MissingPid => "missing_pid",
            Self::MissingGroup => "missing_group",
            Self::UnexpectedRow => "unexpected_row",
            Self::MissingLeader => "missing_leader",
            Self::StdoutBudget | Self::StderrBudget => "overflow",
            Self::Exit => "exit_failed",
            Self::Stderr => "stderr_nonempty",
        }
    }
}
#[derive(Debug)]
pub(super) struct FramingError(pub(super) Framing);
impl fmt::Display for FramingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.message())
    }
}
impl std::error::Error for FramingError {}
pub(super) fn framing(code: Framing) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, FramingError(code))
}

pub(super) fn io_name(kind: io::ErrorKind) -> &'static str {
    match kind {
        io::ErrorKind::NotFound => "not_found",
        io::ErrorKind::PermissionDenied => "permission_denied",
        io::ErrorKind::Interrupted => "interrupted",
        io::ErrorKind::InvalidInput => "invalid_input",
        io::ErrorKind::InvalidData => "invalid_data",
        io::ErrorKind::TimedOut => "timed_out",
        io::ErrorKind::WouldBlock => "would_block",
        io::ErrorKind::UnexpectedEof => "unexpected_eof",
        io::ErrorKind::BrokenPipe => "broken_pipe",
        io::ErrorKind::OutOfMemory => "out_of_memory",
        io::ErrorKind::WriteZero => "write_zero",
        _ => "other",
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Refusal {
    Frame(Framing),
    Io(io::ErrorKind),
    Deadline,
    Allocation,
    MissingEndpoint,
    #[cfg(test)]
    FixtureWatchdog,
}
impl Refusal {
    fn name(self) -> &'static str {
        match self {
            Self::Frame(c) => c.name(),
            Self::Io(k) => io_name(k),
            Self::Deadline => "deadline",
            Self::Allocation => "allocation",
            Self::MissingEndpoint => "missing_endpoint",
            #[cfg(test)]
            Self::FixtureWatchdog => "unit_fixture_watchdog",
        }
    }
}
pub(super) fn refusal(error: &io::Error) -> Refusal {
    error
        .get_ref()
        .and_then(|e| e.downcast_ref::<FramingError>())
        .map(|e| Refusal::Frame(e.0))
        .unwrap_or(Refusal::Io(error.kind()))
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Eof {
    #[default]
    NotObserved,
    Pending,
    Observed,
}
impl Eof {
    fn name(self) -> &'static str {
        match self {
            Self::NotObserved => "not_observed",
            Self::Pending => "pending",
            Self::Observed => "observed",
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct StreamFacts {
    pub(super) calls: u64,
    pub(super) bytes: u64,
    pub(super) would_block: u64,
    pub(super) interrupted: u64,
    pub(super) eof: Eof,
}
impl StreamFacts {
    pub(super) fn interrupted_read(&mut self) {
        self.interrupted = self.interrupted.saturating_add(1);
        self.eof = Eof::Pending;
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Exit {
    #[default]
    Unavailable,
    Success,
    Nonzero,
    Signaled,
}
impl Exit {
    pub(super) fn observed(exit: ExitStatus) -> Self {
        if exit.success() {
            Self::Success
        } else if exit.code().is_some() {
            Self::Nonzero
        } else {
            Self::Signaled
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::Success => "success",
            Self::Nonzero => "nonzero_exit",
            Self::Signaled => "signaled_or_other",
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct StatusFacts {
    pub(super) calls: u64,
    pub(super) pending: u64,
    pub(super) interrupted: u64,
    pub(super) errors: u64,
    pub(super) exit: Exit,
    pub(super) source_kind: Option<io::ErrorKind>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Cleanup {
    #[default]
    NotReached,
    ReapedByStatus,
    KillThenReaped,
    WaitFailed,
    Relinquished,
}
impl Cleanup {
    fn name(self) -> &'static str {
        match self {
            Self::NotReached => "not_reached",
            Self::ReapedByStatus => "reaped_by_status_observation",
            Self::KillThenReaped => "kill_requested_then_wait_reaped",
            Self::WaitFailed => "wait_failed_uncertain",
            Self::Relinquished => "relinquished_without_cleanup",
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub(super) enum Kill {
    #[default]
    NotRequested,
    Ok,
    Error(io::ErrorKind),
}
impl Kill {
    fn name(self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::Ok => "returned_ok",
            Self::Error(_) => "returned_error",
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub(super) enum Validation {
    #[default]
    NotReached,
    Live,
    Dead,
    Refused(Refusal),
}
impl Validation {
    fn name(self) -> &'static str {
        match self {
            Self::NotReached => "not_reached",
            Self::Live => "valid_live",
            Self::Dead => "valid_dead",
            Self::Refused(r) => r.name(),
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Snapshot {
    pub(super) site: Option<Site>,
    pub(super) stream: StreamKind,
    pub(super) refusal: Option<Refusal>,
    pub(super) returned_kind: Option<io::ErrorKind>,
    pub(super) stdout: Option<StreamFacts>,
    pub(super) stderr: Option<StreamFacts>,
    pub(super) status: Option<StatusFacts>,
    pub(super) validation: Validation,
    pub(super) cleanup: Cleanup,
    pub(super) kill: Kill,
    pub(super) elapsed_us: Option<u64>,
    pub(super) spawn_us: Option<u64>,
    pub(super) cleanup_us: Option<u64>,
}
pub(super) fn micros(elapsed: Duration) -> u64 {
    u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX)
}
pub(super) struct Collector {
    pub(super) facts: Snapshot,
    pub(super) started: Option<Instant>,
    #[cfg(test)]
    pub(super) hooks: Hooks,
}
impl Collector {
    pub(super) fn new() -> Self {
        Self {
            facts: Snapshot::default(),
            started: None,
            #[cfg(test)]
            hooks: Hooks::default(),
        }
    }
    pub(super) fn fail(
        &mut self,
        site: Site,
        stream: StreamKind,
        cause: Refusal,
        kind: io::ErrorKind,
    ) {
        self.facts.site = Some(site);
        self.facts.stream = stream;
        self.facts.refusal = Some(cause);
        self.facts.returned_kind = Some(kind);
        self.facts.elapsed_us = self.started.map(|t| micros(t.elapsed()));
    }
    pub(super) fn stream(&mut self, stream: StreamKind) -> Option<&mut StreamFacts> {
        match stream {
            StreamKind::Stdout => Some(self.facts.stdout.get_or_insert_with(StreamFacts::default)),
            StreamKind::Stderr => Some(self.facts.stderr.get_or_insert_with(StreamFacts::default)),
            StreamKind::None => None,
        }
    }
    pub(super) fn deadline(
        &mut self,
        deadline: Instant,
        site: Site,
        stream: StreamKind,
    ) -> io::Result<()> {
        #[cfg(test)]
        if !matches!(self.hooks.clock, Clock::Real)
            && Instant::now()
                >= *self
                    .hooks
                    .watchdog
                    .get_or_insert_with(|| Instant::now() + Duration::from_secs(10))
        {
            let error = io::Error::other("native inspection unit fixture watchdog elapsed");
            self.fail(site, stream, Refusal::FixtureWatchdog, error.kind());
            return Err(error);
        }
        #[cfg(test)]
        let result = match self.hooks.clock {
            Clock::Real => super::current(deadline),
            Clock::Open => Ok(()),
            Clock::Expire(target, endpoint) if target == site && endpoint == stream => {
                Err(super::timed_out())
            }
            Clock::Expire(_, _) => Ok(()),
        };
        #[cfg(not(test))]
        let result = super::current(deadline);
        if let Err(e) = &result {
            self.fail(site, stream, Refusal::Deadline, e.kind());
        }
        result
    }
    #[cfg(test)]
    pub(super) fn inject(&mut self, site: Site, stream: StreamKind) -> io::Result<()> {
        if let Some((target, endpoint, kind)) = self.hooks.failure
            && target == site
            && endpoint == stream
        {
            let refusal = match site {
                Site::Allocation => Refusal::Allocation,
                Site::Endpoint => Refusal::MissingEndpoint,
                _ => Refusal::Io(kind),
            };
            self.fail(site, stream, refusal, kind);
            return Err(io::Error::new(kind, site.message()));
        }
        Ok(())
    }
    pub(super) fn idle(&self, deadline: Instant) -> Duration {
        #[cfg(test)]
        if !matches!(self.hooks.clock, Clock::Real) {
            return super::IDLE;
        }
        super::IDLE.min(deadline.saturating_duration_since(Instant::now()))
    }
    pub(super) fn attach(&self, error: io::Error) -> io::Error {
        io::Error::new(error.kind(), Failure { facts: self.facts })
    }
}
#[derive(Debug)]
struct Failure {
    facts: Snapshot,
}
impl std::error::Error for Failure {}
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let x = &self.facts;
        let prefix = match x.refusal {
            Some(Refusal::Frame(c)) => c.message(),
            Some(Refusal::Deadline) => "native process inspection timed out",
            Some(Refusal::Allocation) => "native inspection allocation failed",
            #[cfg(test)]
            Some(Refusal::FixtureWatchdog) => "native inspection unit fixture watchdog elapsed",
            Some(Refusal::MissingEndpoint) => match x.stream {
                StreamKind::Stdout => "missing inspection stdout",
                StreamKind::Stderr => "missing inspection stderr",
                StreamKind::None => "missing inspection endpoint",
            },
            _ => x
                .site
                .map(Site::message)
                .unwrap_or("native process inspection failed"),
        };
        f.write_str(prefix)?;
        if x.cleanup == Cleanup::WaitFailed {
            f.write_str("; native inspector cleanup/reap uncertain")?;
        }
        write!(
            f,
            "; inspection_facts{{site={},stream={},refusal={},returned_io_kind={}",
            x.site.map(Site::name).unwrap_or("unavailable"),
            x.stream.name(),
            x.refusal.map(Refusal::name).unwrap_or("unavailable"),
            x.returned_kind.map(io_name).unwrap_or("unavailable")
        )?;
        for (name, stream) in [("stdout", x.stdout), ("stderr", x.stderr)] {
            if let Some(s) = stream {
                write!(
                    f,
                    ",{name}_reads={},{name}_bytes={},{name}_would_block={},{name}_interrupted={},{name}_eof={}",
                    s.calls,
                    s.bytes,
                    s.would_block,
                    s.interrupted,
                    s.eof.name()
                )?;
            } else {
                write!(f, ",{name}=unavailable")?;
            }
        }
        if let Some(s) = x.status {
            write!(
                f,
                ",status_polls={},status_pending={},status_interrupted={},status_errors={},exit={},status_source_io_kind={}",
                s.calls,
                s.pending,
                s.interrupted,
                s.errors,
                s.exit.name(),
                s.source_kind.map(io_name).unwrap_or("unavailable")
            )?;
        } else {
            f.write_str(",status=unavailable")?;
        }
        write!(
            f,
            ",validation={},cleanup={},kill={}",
            x.validation.name(),
            x.cleanup.name(),
            x.kill.name()
        )?;
        if let Kill::Error(kind) = x.kill {
            write!(f, ",kill_io_kind={}", io_name(kind))?;
        }
        for (name, value) in [
            ("observation_us", x.elapsed_us),
            ("spawn_us", x.spawn_us),
            ("cleanup_us", x.cleanup_us),
        ] {
            if let Some(v) = value {
                write!(f, ",{name}={v}")?;
            } else {
                write!(f, ",{name}=unavailable")?;
            }
        }
        f.write_str("}")
    }
}

#[cfg(test)]
#[derive(Default)]
pub(super) struct Hooks {
    pub(super) clock: Clock,
    pub(super) watchdog: Option<Instant>,
    pub(super) failure: Option<(Site, StreamKind, io::ErrorKind)>,
    pub(super) status_error_after_reap: bool,
    pub(super) actual_reaped: Option<ExitStatus>,
    pub(super) synthetic_kills: u64,
    pub(super) synthetic_waits: u64,
}
#[cfg(test)]
#[derive(Clone, Copy, Default)]
pub(super) enum Clock {
    #[default]
    Real,
    Open,
    Expire(Site, StreamKind),
}

#[cfg(test)]
pub(super) fn snapshot(error: &io::Error) -> Option<Snapshot> {
    error
        .get_ref()
        .and_then(|e| e.downcast_ref::<Failure>())
        .map(|f| f.facts)
}
