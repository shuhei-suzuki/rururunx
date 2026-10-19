use super::*;
use serde_json::json;

fn empty() -> PipelineModeInventory {
    Store::memory().unwrap().pipeline_mode_inventory().unwrap()
}

/// Both directions require complete coverage even when the discovered match set is empty.
#[test]
fn b3_discovery_limited() {
    for mode in [ExecutionMode::ManagedNative, ExecutionMode::CliPipeline] {
        let mut inventory = empty();
        inventory.to_cli_pipeline = PipelineQuiescenceVerdict::CookieDiscoveryRequired;
        inventory.to_managed_native = PipelineQuiescenceVerdict::CookieDiscoveryRequired;
        inventory.native_cookies = vec!["native-cookie-012345".into()];
        inventory.pipeline_cookies = vec!["pipeline-cookie-0123".into()];
        assert!(check_mode_inventory(&inventory, mode, |_| Ok((true, 0, 1))).is_err());
    }
}
/// Both directions refuse a matching process despite complete scan coverage.
#[test]
fn b3_discovery_match() {
    for mode in [ExecutionMode::ManagedNative, ExecutionMode::CliPipeline] {
        let mut inventory = empty();
        inventory.to_cli_pipeline = PipelineQuiescenceVerdict::CookieDiscoveryRequired;
        inventory.to_managed_native = PipelineQuiescenceVerdict::CookieDiscoveryRequired;
        inventory.native_cookies = vec!["native-cookie-012345".into()];
        inventory.pipeline_cookies = vec!["pipeline-cookie-0123".into()];
        assert!(check_mode_inventory(&inventory, mode, |_| Ok((false, 1, 1))).is_err());
    }
}
/// Every opposite-mode cookie must be scanned before quiescence is accepted.
#[test]
fn b3_discovery_all_cookies() {
    for mode in [ExecutionMode::ManagedNative, ExecutionMode::CliPipeline] {
        let mut inventory = empty();
        inventory.to_cli_pipeline = PipelineQuiescenceVerdict::CookieDiscoveryRequired;
        inventory.to_managed_native = PipelineQuiescenceVerdict::CookieDiscoveryRequired;
        inventory.native_cookies =
            vec!["native-cookie-012345".into(), "native-cookie-678901".into()];
        inventory.pipeline_cookies =
            vec!["pipeline-cookie-0123".into(), "pipeline-cookie-4567".into()];
        let expected = if mode == ExecutionMode::CliPipeline {
            &inventory.native_cookies
        } else {
            &inventory.pipeline_cookies
        };
        let mut seen = Vec::new();
        let result = check_mode_inventory(&inventory, mode, |cookie| {
            seen.push(cookie.to_owned());
            Ok((false, 0, 5))
        })
        .unwrap();
        assert_eq!(&seen, expected);
        assert_eq!(
            result.into_body()["discovery"],
            json!({"cookies":2,"inspected":10})
        );
    }
}
/// A failed discovery cannot be treated as an empty successful scan.
#[test]
fn b3_discovery_error() {
    let mut inventory = empty();
    inventory.to_cli_pipeline = PipelineQuiescenceVerdict::CookieDiscoveryRequired;
    inventory.native_cookies = vec!["native-cookie-012345".into()];
    assert!(
        check_mode_inventory(&inventory, ExecutionMode::CliPipeline, |_| anyhow::bail!(
            "scan unavailable"
        ))
        .is_err()
    );
}
/// Excessive cookie inventories fail closed without scanning any process.
#[test]
fn b3_discovery_inventory_bound() {
    let mut inventory = empty();
    inventory.to_cli_pipeline = PipelineQuiescenceVerdict::CookieDiscoveryRequired;
    inventory.native_cookies = vec!["native-cookie-012345".into(); 4097];
    assert!(
        check_mode_inventory(&inventory, ExecutionMode::CliPipeline, |_| panic!(
            "over bound"
        ))
        .is_err()
    );
}
/// An empty native installation retains the default mode without Q records.
#[test]
fn b3_default_native_owner() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let owner = RuntimeOwner::open(&path).unwrap();
    assert_eq!(owner.execution_mode(), ExecutionMode::ManagedNative);
    assert!(
        owner
            .store
            .lock()
            .unwrap()
            .pipeline_qualifications(&owner)
            .unwrap()
            .is_empty()
    );
    let epoch = owner.epoch();
    drop(owner);
    assert_eq!(RuntimeOwner::open(&path).unwrap().epoch(), epoch + 1);
}
/// Quiescence is recorded only after the successful pipeline owner epoch begins.
#[test]
#[cfg(target_os = "macos")]
fn b3_pipeline_owner_records_quiescence() {
    let dir = tempfile::tempdir().unwrap();
    let owner =
        RuntimeOwner::open_mode(&dir.path().join("state.db"), ExecutionMode::CliPipeline).unwrap();
    let rows = owner
        .store
        .lock()
        .unwrap()
        .pipeline_qualifications(&owner)
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].subject, "mode_quiescence");
    assert_eq!(rows[0].state, "qualified");
    assert_eq!(
        rows[0].body["discovery"],
        json!({"cookies":0,"inspected":0})
    );
    assert_eq!(rows[0].body["inventory"]["native_units"], 0);
}
