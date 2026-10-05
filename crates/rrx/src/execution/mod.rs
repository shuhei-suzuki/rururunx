//! Result protection for cooperative host-native execution. Not a security sandbox.
//! Work and reclamation are independent; a process observation never proves custody.
pub mod model;
pub use model::*;
pub mod owner;
pub use owner::RuntimeOwner;
pub mod attempts;
mod claude_wire;
pub(crate) mod git_io;
pub mod ipc;
pub mod native;
pub mod process;
pub mod quota;
pub mod resources;
pub mod results;
pub(crate) use results::{ReadonlyCompletion, WorkflowPublication};
pub mod tools;
