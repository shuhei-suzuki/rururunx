//! Result protection for cooperative host-native execution. Not a security sandbox.
//! Work and reclamation are independent; a process observation never proves custody.
pub mod model;
pub use model::*;
pub mod owner;
pub use owner::RuntimeOwner;
pub mod process;
pub mod results;
pub mod resources;
pub mod attempts;
pub mod tools;
