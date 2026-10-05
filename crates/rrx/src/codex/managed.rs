//! Protocol reuse for the separately admitted result-protection profile.
//! This does not enable the legacy custody adapter or its fixture availability.
use super::protocol::*;
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::path::Path;

pub(crate) fn version(value: &str) -> Result<()> {
    verify_native_version(value)?;
    Ok(())
}
pub(crate) fn initialization(value: &Value) -> Result<()> {
    verify_initialize(value)?;
    Ok(())
}
pub(crate) fn frame(value: &Value) -> Result<()> {
    Event::parse(&serde_json::to_vec(value)?)?;
    Ok(())
}
pub(crate) struct Approvals {
    ledger: ApprovalLedger,
}
impl Approvals {
    pub(crate) fn new(thread: &str, turn: &str, source: &Path) -> Self {
        Self {
            ledger: ApprovalLedger::new(thread.into(), turn.into()).for_workspace(source.into()),
        }
    }
    pub(crate) fn item(&mut self, value: &Value) -> Result<()> {
        self.ledger.observe_item(value)?;
        Ok(())
    }
    pub(crate) fn insert(&mut self, value: &Value) -> Result<Value> {
        let id: RpcId = serde_json::from_value(value["id"].clone())?;
        let method = value["method"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("method missing"))?;
        let pending = self
            .ledger
            .insert(id, method.into(), value["params"].clone())?;
        // Pending native arguments stay only in the private ledger, never the public view.
        Ok(json!({"id":pending.id,"method":pending.method,"operation_hash":pending.operation_hash}))
    }
    pub(crate) fn reply(&mut self, id: &Value, hash: &str, allow: bool) -> Result<Value> {
        let id: RpcId = serde_json::from_value(id.clone())?;
        ensure!(
            self.ledger.request(&id)?.operation_hash == hash,
            "approval operation changed"
        );
        Ok(self.ledger.reply(
            &id,
            if allow {
                OperationDecision::Approve
            } else {
                OperationDecision::Deny
            },
        )?)
    }
}
