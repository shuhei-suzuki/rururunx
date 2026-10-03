use std::process::Command;

#[test]
fn help_and_version_work_without_runtime_side_effects() {
    for option in ["--help", "--version"] {
        let output = Command::new(env!("CARGO_BIN_EXE_rrx"))
            .arg(option)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("rrx"));
    }
}

#[test]
fn missing_explicit_config_is_an_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_rrx"))
        .args(["--config", "/nonexistent-rrx-config.toml", "config-check"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot read config"));
}

#[test]
fn default_configuration_validates() {
    let output = Command::new(env!("CARGO_BIN_EXE_rrx"))
        .arg("config-check")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Configuration valid"));
}

#[test]
fn project_overlay_loads_from_files_without_losing_runtime_limits() {
    let dir = std::env::temp_dir().join(format!(
        "rrx-config-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&dir).unwrap();
    let global = dir.join("global.toml");
    let project = dir.join("project.toml");
    std::fs::write(&global, "[scheduler]\nglobal_max_sessions = 8\n").unwrap();
    std::fs::write(
        &project,
        "[scheduler]\nmax_tasks_per_project = 2\n[context]\nenabled = false\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rrx"))
        .arg("--config")
        .arg(&global)
        .arg("--project-config")
        .arg(&project)
        .arg("config-check")
        .output()
        .unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("global sessions 8, tasks/project 2, context baseline")
    );
}
