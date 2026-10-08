use std::{path::PathBuf, process::Command};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "rrx-config-test-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        Self(dir)
    }
    fn write(&self, name: &str, text: &str) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, text).unwrap();
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn rrx() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rrx"))
}

#[test]
fn help_version_and_config_check_have_no_runtime_side_effects() {
    let fixture = Fixture::new();
    for option in ["--help", "--version", "config-check"] {
        let output = rrx().current_dir(&fixture.0).arg(option).output().unwrap();
        assert!(output.status.success());
        assert!(!output.stdout.is_empty());
        assert_eq!(std::fs::read_dir(&fixture.0).unwrap().count(), 0);
    }
}

#[test]
fn missing_explicit_config_is_an_error_with_and_without_subcommand() {
    let fixture = Fixture::new();
    for args in [vec!["config-check"], vec![]] {
        let output = rrx()
            .arg("--config")
            .arg(fixture.0.join("missing.toml"))
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("cannot read config"));
    }
}

#[test]
fn project_file_overrides_project_fields_and_preserves_global_limits() {
    let fixture = Fixture::new();
    let global = fixture.write("global.toml", "minimum_workflow = 'STRICT'\n[scheduler]\nglobal_max_sessions = 8\nmax_tasks_per_project = 1\n[context]\nenabled = true\n[agents.codex]\nmodel = 'old'\ncommand = ['codex', 'exec']");
    let project = fixture.write("project.toml", "minimum_workflow = 'QUICK'\n[scheduler]\nmax_tasks_per_project = 1\n[context]\nenabled = false\n[agents.codex]\nmodel = 'new'");
    let config = rrx::config::Config::load(Some(&global), Some(&project)).unwrap();
    assert_eq!(config.minimum_workflow, rrx::config::WorkflowClass::Strict);
    assert_eq!(config.agents["codex"].model.as_deref(), Some("new"));
    assert_eq!(config.agents["codex"].command, ["codex", "exec"]);
    let output = rrx()
        .arg("--config")
        .arg(global)
        .arg("--project-config")
        .arg(project)
        .arg("config-check")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("global sessions 8, tasks/project 1, context baseline")
    );
}

#[test]
fn invalid_configs_fail_through_actual_loader_with_source_path() {
    let fixture = Fixture::new();
    for input in [
        "unexpected = true",
        "[context]\nunknown = true",
        "[invalid",
        "[scheduler]\nglobal_max_sessions = 0",
        "[context]\nrepo_map_tokens = 0",
        "[agents.codex]\ncommand = [' ']",
        "[agents.codex]\nmax_concurrent = 0",
        "[agents.codex]\nmodel = ' '",
    ] {
        let path = fixture.write("invalid.toml", input);
        let output = rrx()
            .arg("--config")
            .arg(&path)
            .arg("config-check")
            .output()
            .unwrap();
        assert!(!output.status.success(), "accepted {input}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("invalid.toml"));
    }
    for input in [
        "[scheduler]\nglobal_max_sessions = 99",
        "[scheduler]\nmax_tasks_per_project = 0",
        "[agents.codex]\ncommand = ['other']",
        "[agents.codex]\nmax_concurrent = 9",
        "[context]\nreview_context_tokens = 0",
        "[agents.unknown]\nmodel = 'x'",
    ] {
        let path = fixture.write("project.toml", input);
        let output = rrx()
            .arg("--project-config")
            .arg(path)
            .arg("config-check")
            .output()
            .unwrap();
        assert!(!output.status.success(), "accepted {input}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("project.toml"));
    }
}

#[test]
fn readme_configuration_example_is_valid() {
    let fixture = Fixture::new();
    let readme = include_str!("../../../README.md");
    let toml = readme
        .split("## Development")
        .nth(1)
        .unwrap()
        .split("```toml\n")
        .nth(1)
        .unwrap()
        .split("```")
        .next()
        .unwrap();
    let path = fixture.write("example.toml", toml);
    assert!(
        rrx()
            .arg("--config")
            .arg(path)
            .arg("config-check")
            .output()
            .unwrap()
            .status
            .success()
    );
}

/// C-S2d (CLI): a runtime config, an overlay or `--max-tasks` other than 1 is
/// refused with the typed message, and `project add` writes no state at all.
#[test]
fn c_s2d_limits_other_than_one_are_refused_without_state() {
    let fixture = Fixture::new();
    let global = fixture.write("global.toml", "[scheduler]\nmax_tasks_per_project = 2");
    let overlay = fixture.write("project.toml", "[scheduler]\nmax_tasks_per_project = 2");
    for args in [
        vec!["--config".into(), global.clone(), "config-check".into()],
        vec!["--project-config".into(), overlay, "config-check".into()],
    ] {
        let output = rrx().args(&args).output().unwrap();
        assert!(!output.status.success(), "{args:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("MVP supports exactly 1 active Task per Project"),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let state = fixture.0.join("state").join("rrx.db");
    let output = rrx()
        .arg("--state")
        .arg(&state)
        .args(["project", "add"])
        .arg(&fixture.0)
        .args(["--max-tasks", "2"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("CliFlag requested 2"));
    assert!(!fixture.0.join("state").exists(), "project add wrote state");
}
