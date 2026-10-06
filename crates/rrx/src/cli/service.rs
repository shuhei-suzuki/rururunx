//! Foreground product control service. This composes controls, not native drivers.
use super::{endpoint::ControlEndpoint, transport};
use crate::{
    config::Config,
    execution::RuntimeOwner,
    runtime::{
        Runtime,
        control::{ControlAction, ControlRequest, ControlResponse},
    },
};
use anyhow::{Context, Result};
use std::{path::Path, sync::Arc, time::Duration};
use tokio::{
    io::BufReader,
    sync::{Notify, Semaphore},
    task::JoinSet,
};

pub const MAX_CONNECTIONS: usize = 64;
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn serve(state: &Path, config: Config) -> Result<()> {
    let owner = RuntimeOwner::open(state)?;
    let runtime = Arc::new(Runtime::new(owner.clone(), config)?);
    let endpoint = ControlEndpoint::bind(owner)?;
    runtime.start().await?;
    let stop = Arc::new(Notify::new());
    let permits = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    let mut connections = JoinSet::new();
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let result = loop {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => break signal.context("Runtime signal unavailable"),
            _ = terminate.recv() => break Ok(()),
            () = stop.notified() => break Ok(()),
            Some(_) = connections.join_next(), if !connections.is_empty() => {},
            accepted = endpoint.accept_peer() => {
                let Ok(accepted) = accepted else {
                    // Individual peer/Hello failures cannot retire the service.
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                };
                while connections.try_join_next().is_some() {}
                if connections.len() >= MAX_CONNECTIONS {
                    drop(accepted);
                    continue;
                }
                let Ok(permit) = permits.clone().try_acquire_owned() else {
                    // No control, queue or detached worker is admitted on overload.
                    drop(accepted);
                    continue;
                };
                let runtime = runtime.clone();
                let stop = stop.clone();
                connections.spawn(async move {
                    let _permit = permit;
                    let mut reader = BufReader::new(accepted);
                    if transport::send(reader.get_mut(), &transport::Hello {
                        protocol: transport::PROTOCOL_VERSION,
                        identity: runtime.control_identity(),
                    }, transport::RESPONSE_BYTES).await.is_err() { return; }
                    let request = transport::receive::<ControlRequest>(&mut reader, transport::REQUEST_BYTES).await;
                    let (response, stopping) = match request {
                        Ok(request) => {
                            let id = request.request_id;
                            let is_stop = matches!(request.action, ControlAction::RuntimeStop);
                            let response = runtime.handle_control(reader.get_ref().stream(), request).await;
                            (response.unwrap_or(ControlResponse::Rejected { request_id: Some(id) }), is_stop && runtime.is_stopping())
                        }
                        Err(_) => (ControlResponse::Rejected { request_id: None }, false),
                    };
                    // A stop acknowledgement is emitted only by successful Runtime shutdown.
                    let _ = transport::send(reader.get_mut(), &response, transport::RESPONSE_BYTES).await;
                    if stopping { stop.notify_one(); }
                });
            }
        }
    };
    // Stop accepting before waiting for other connections; do not self-join.
    // Already committed controls are not rolled back by a cancelled socket wait.
    let shutdown = runtime.shutdown().await;
    let drained = tokio::time::timeout(DRAIN_TIMEOUT, async {
        while connections.join_next().await.is_some() {}
    })
    .await;
    if drained.is_err() {
        connections.abort_all();
        tokio::time::timeout(DRAIN_TIMEOUT, async {
            while connections.join_next().await.is_some() {}
        })
        .await
        .context("Runtime cancelled connection shutdown remains pending")?;
    }
    drop(endpoint); // Removes only the descriptor belonging to this retained owner.
    result?;
    shutdown?;
    Ok(())
}
