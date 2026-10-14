//! Safe, consuming process launch boundaries.
//!
//! On macOS, descriptors above stderr are marked CLOEXEC in the forked child,
//! never closed. Linux adds no hygiene callback and delegates unchanged; other
//! targets refuse launch with ENOSYS.
//! The original std/Tokio operation retains its stdio, cancellation, and Child
//! ownership semantics. Existing Tokio failures after std spawn may still leave
//! a child: this boundary does not change or resolve that upstream limitation.

use std::future::Future;
use std::io;
use std::process::{Child, Command, ExitStatus, Output};

mod platform;

fn prepare_std(mut command: Command) -> io::Result<Command> {
    platform::install(&mut command)?;
    Ok(command)
}

fn prepare_tokio(mut command: tokio::process::Command) -> io::Result<tokio::process::Command> {
    platform::install(command.as_std_mut())?;
    Ok(command)
}

/// Launch synchronously, preserving the original Child owner and error path.
#[allow(
    clippy::disallowed_methods,
    reason = "the consuming hygiene boundary delegates to std"
)]
pub fn spawn_std(command: Command) -> io::Result<Child> {
    prepare_std(command)?.spawn()
}

/// Collect output with std's original default and explicitly configured stdio.
#[allow(
    clippy::disallowed_methods,
    reason = "delegate to std output without rebuilding its stdio defaults"
)]
pub fn output_std(command: Command) -> io::Result<Output> {
    prepare_std(command)?.output()
}

/// Wait for status with std's original stdio defaults.
#[allow(
    clippy::disallowed_methods,
    reason = "delegate to std status preserving configured stdio"
)]
pub fn status_std(command: Command) -> io::Result<ExitStatus> {
    prepare_std(command)?.status()
}

/// Launch synchronously with Tokio's original registration and kill-on-drop.
#[allow(
    clippy::disallowed_methods,
    reason = "the consuming hygiene boundary delegates to Tokio"
)]
pub fn spawn_tokio(command: tokio::process::Command) -> io::Result<tokio::process::Child> {
    prepare_tokio(command)?.spawn()
}

/// Spawn at the call site, then return Tokio's original output future.
#[allow(
    clippy::disallowed_methods,
    reason = "Tokio output must retain synchronous spawn and stdio overrides"
)]
pub fn output_tokio(command: tokio::process::Command) -> impl Future<Output = io::Result<Output>> {
    let future = prepare_tokio(command).map(|mut command| command.output());
    async move { future?.await }
}

/// Spawn at the call site, then return Tokio's original status future.
#[allow(
    clippy::disallowed_methods,
    reason = "Tokio status must retain synchronous spawn and cancellation"
)]
pub fn status_tokio(
    command: tokio::process::Command,
) -> impl Future<Output = io::Result<ExitStatus>> {
    let future = prepare_tokio(command).map(|mut command| command.status());
    async move { future?.await }
}

#[cfg(test)]
mod tests;
