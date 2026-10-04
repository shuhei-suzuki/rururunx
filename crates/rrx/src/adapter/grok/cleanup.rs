//! Bounded measured facts only. These values never authorize native ownership.
use super::ownership::StageUncertainty;
use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CleanupState {
    NotAttempted,
    GroupCleanupFailedUnclassified,
    ReapTimeout,
    ReapError,
    Succeeded,
}
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum DrainState {
    NotStarted,
    JoinedReturned,
    JoinedPanic,
    JoinedCancelled,
    BudgetElapsedAbortRequested,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ReapKind {
    NotFound,
    PermissionDenied,
    Interrupted,
    InvalidInput,
    InvalidData,
    TimedOut,
    WouldBlock,
    UnexpectedEof,
    BrokenPipe,
    OutOfMemory,
    WriteZero,
    Other,
}
impl From<std::io::ErrorKind> for ReapKind {
    fn from(kind: std::io::ErrorKind) -> Self {
        use std::io::ErrorKind;
        match kind {
            ErrorKind::NotFound => Self::NotFound,
            ErrorKind::PermissionDenied => Self::PermissionDenied,
            ErrorKind::Interrupted => Self::Interrupted,
            ErrorKind::InvalidInput => Self::InvalidInput,
            ErrorKind::InvalidData => Self::InvalidData,
            ErrorKind::TimedOut => Self::TimedOut,
            ErrorKind::WouldBlock => Self::WouldBlock,
            ErrorKind::UnexpectedEof => Self::UnexpectedEof,
            ErrorKind::BrokenPipe => Self::BrokenPipe,
            ErrorKind::OutOfMemory => Self::OutOfMemory,
            ErrorKind::WriteZero => Self::WriteZero,
            _ => Self::Other,
        }
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) struct CleanupReceipt {
    pub owned_process_group_created: bool,
    pub cleanup_ok: bool,
    pub cleanup_state: CleanupState,
    pub reap_io_kind: Option<ReapKind>,
    pub output_verified: bool,
    pub stderr_drain_state: DrainState,
    pub stderr_read_error: &'static str,
    pub ownership_uncertain: bool,
    pub uncertainty_by_stage: StageUncertainty,
    pub dispatched: bool,
    pub native_outcome: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reap_error_kind_is_a_closed_crate_owned_vocabulary() {
        use std::io::ErrorKind;
        for (kind, word) in [
            (ErrorKind::NotFound, "not_found"),
            (ErrorKind::PermissionDenied, "permission_denied"),
            (ErrorKind::Interrupted, "interrupted"),
            (ErrorKind::InvalidInput, "invalid_input"),
            (ErrorKind::InvalidData, "invalid_data"),
            (ErrorKind::TimedOut, "timed_out"),
            (ErrorKind::WouldBlock, "would_block"),
            (ErrorKind::UnexpectedEof, "unexpected_eof"),
            (ErrorKind::BrokenPipe, "broken_pipe"),
            (ErrorKind::OutOfMemory, "out_of_memory"),
            (ErrorKind::WriteZero, "write_zero"),
            (ErrorKind::ConnectionReset, "other"),
        ] {
            assert_eq!(serde_json::to_value(ReapKind::from(kind)).unwrap(), word);
        }
        // Error body is never an input to this bounded mapping.
        let synthetic = std::io::Error::other("synthetic private error body");
        assert_eq!(
            serde_json::to_value(ReapKind::from(synthetic.kind())).unwrap(),
            "other"
        );
    }
}
