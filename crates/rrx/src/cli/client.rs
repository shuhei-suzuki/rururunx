//! Read-only discovery and single-operation control client. No Store or owner.
use super::{endpoint, transport};
use crate::runtime::control::{ControlAction, ControlRequest, ControlResponse};
use anyhow::{Result, ensure};
use std::path::Path;
use uuid::Uuid;

pub async fn request(state: &Path, action: ControlAction) -> Result<ControlResponse> {
    let (identity, mut connection) = endpoint::connect(state).await?;
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: identity.instance,
        epoch: identity.epoch,
        action,
    };
    transport::send(connection.get_mut(), &request, transport::REQUEST_BYTES).await?;
    let response: ControlResponse =
        transport::receive(&mut connection, transport::RESPONSE_BYTES).await?;
    ensure!(
        !matches!(response, ControlResponse::Rejected { .. }),
        "Runtime control refused; state or scope may have changed"
    );
    Ok(response)
}
