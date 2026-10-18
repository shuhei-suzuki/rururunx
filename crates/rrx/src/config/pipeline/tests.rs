use super::*;
use crate::config::{Config, ProjectOverlay};

fn configured() -> Config {
    toml::from_str("[execution]\nmode='cli_pipeline'\n[pipeline]\nwork_root='/Users/operator/rrx'\npath_prefix=['/usr/local/bin']\n[pipeline.clis.codex]\nexecutable='/usr/local/bin/codex'\nversion='codex pinned'\nmodel='executor'\neffort='high'\ncodex_home='/Users/operator/rrxhome'\n[pipeline.clis.claude]\nexecutable='/usr/local/bin/claude'\nversion='claude pinned'\nmodel='reviewer'\n").unwrap()
}
/// Default execution keys do not change the existing native policy encoding.
#[test]
fn b3_config_default_digest() {
    let config = Config::default();
    assert_eq!(config.execution.mode, ExecutionMode::ManagedNative);
    let encoded = serde_json::to_value(config).unwrap();
    assert!(encoded.get("execution").is_none());
    assert!(encoded.get("pipeline").is_none());
}
/// Pipeline mode requires runtime pipeline settings before state is opened.
#[test]
fn b3_config_pipeline_required() {
    let mut config = configured();
    config.validate().unwrap();
    config.pipeline = None;
    assert!(config.validate().is_err());
}
/// Project inputs cannot override execution mode, pipeline pins or templates.
#[test]
fn b3_config_overlay_refused() {
    for input in [
        "[execution]\nmode='cli_pipeline'",
        "[pipeline]\nwork_root='/tmp/work'",
        "[pipeline.clis.codex]\nmodel='other'",
    ] {
        assert!(toml::from_str::<ProjectOverlay>(input).is_err());
    }
}
/// Pipeline runtime configuration admits values but never arbitrary argv or concurrency.
#[test]
fn b3_config_closed_surface() {
    let config = configured();
    let encoded = toml::to_string(&config).unwrap();
    for field in [
        "argv=['--unsafe']",
        "command=['x']",
        "max_concurrent=2",
        "env={OPENAI_API_KEY='fixture'}",
    ] {
        let input = encoded.replace(
            "[pipeline.clis.codex]",
            &format!("[pipeline.clis.codex]\n{field}"),
        );
        assert!(toml::from_str::<Config>(&input).is_err());
    }
}
/// Pipeline executable, home and work paths obey the fixed absolute path constraints.
#[test]
fn b3_config_paths() {
    for path in [
        "relative",
        "/tmp/has space",
        "/tmp/../work",
        "/tmp/work/..",
        "/abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz",
    ] {
        let mut config = configured();
        config.pipeline.as_mut().unwrap().work_root = path.into();
        assert!(config.validate().is_err(), "{path}");
    }
    let mut config = configured();
    config.pipeline.as_mut().unwrap().clis.codex.executable = "relative".into();
    assert!(config.validate().is_err());
}

/// Only the supported reasoning efforts can reach fixed Codex configuration argv.
#[test]
fn b3_config_effort_enum() {
    let encoded = toml::to_string(&configured()).unwrap();
    for effort in ["low", "medium", "high", "xhigh"] {
        assert!(
            toml::from_str::<Config>(
                &encoded.replace("effort = \"high\"", &format!("effort = \"{effort}\""))
            )
            .is_ok()
        );
    }
    for effort in ["arbitrary", "max", "high,other=true"] {
        assert!(
            toml::from_str::<Config>(
                &encoded.replace("effort = \"high\"", &format!("effort = \"{effort}\""))
            )
            .is_err()
        );
    }
}
