//! Native workload ownership is unavailable; this example starts no fixture.
use rrx::adapter::{AdapterError, ErrorKind};

fn main() -> Result<(), AdapterError> {
    Err(AdapterError {
        kind: ErrorKind::UnsupportedCapability,
        message: "native workload ownership and dispatch producer unavailable".into(),
    })
}
