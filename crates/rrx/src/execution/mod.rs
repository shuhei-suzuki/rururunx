//! Result protection for cooperative host-native execution. Not a security sandbox.
//! Work and reclamation are independent; a process observation never proves custody.
pub mod model;
pub use model::*;
pub mod owner;
pub use owner::RuntimeOwner;
pub mod attempts;
mod claude_wire;
pub mod cleanup;
pub(crate) mod docker;
pub(crate) mod git_io;
pub mod ipc;
pub mod native;
pub mod native_result;
pub mod process;
pub mod quota;
pub mod resources;
pub mod results;
pub(crate) mod retained_io;
pub(crate) use results::{ReadonlyCompletion, WorkflowPublication};
pub mod strict_json;
pub mod tools;
pub mod workflow_gates;
pub mod workflow_source;
