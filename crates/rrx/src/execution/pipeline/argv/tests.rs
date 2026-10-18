use super::*;
fn cli() -> PipelineCli {
    PipelineCli {
        executable: "/codex".into(),
        version: "fixture".into(),
        model: "model".into(),
        effort: Some(crate::config::ReasoningEffort::High),
        codex_home: Some("/home".into()),
    }
}

/// Implement and fix always request workspace-write with network access disabled.
#[test]
fn b3_executor_template() {
    for role in [PipelineRole::Implement, PipelineRole::Fix] {
        let args = step_arguments(
            role,
            &cli(),
            Path::new("/work"),
            Path::new("/attempt"),
            "{}",
        )
        .unwrap();
        assert_eq!(
            args,
            [
                "exec",
                "--sandbox",
                "workspace-write",
                "-c",
                "sandbox_workspace_write.network_access=false",
                "-c",
                "projects./work.trust_level=\"untrusted\"",
                "-m",
                "model",
                "-c",
                "model_reasoning_effort=high",
                "-C",
                "/work",
                "-o",
                "/attempt/last.md",
                "--json",
                "--color",
                "never",
                "-"
            ]
        );
    }
}
/// Confirmation and review templates retain their fixed read-only permissions and schema arguments.
#[test]
fn b3_readonly_templates() {
    let confirm = step_arguments(
        PipelineRole::Confirm,
        &cli(),
        Path::new("/work"),
        Path::new("/attempt"),
        "{}",
    )
    .unwrap();
    assert_eq!(
        confirm,
        [
            "exec",
            "--sandbox",
            "read-only",
            "-c",
            "projects./work.trust_level=\"untrusted\"",
            "-m",
            "model",
            "-c",
            "model_reasoning_effort=high",
            "-C",
            "/work",
            "--output-schema",
            "/attempt/confirm.schema.json",
            "-o",
            "/attempt/last.json",
            "--json",
            "--color",
            "never",
            "-"
        ]
    );
    let review = step_arguments(
        PipelineRole::Review,
        &cli(),
        Path::new("/work"),
        Path::new("/attempt"),
        "{\"schema\":1}",
    )
    .unwrap();
    assert_eq!(
        review,
        [
            "-p",
            "--permission-mode",
            "plan",
            "--tools",
            "Read,Grep,Glob",
            "--disallowedTools",
            "Bash,Edit,Write,NotebookEdit,WebFetch,WebSearch,Task",
            "--strict-mcp-config",
            "--mcp-config",
            "/attempt/mcp.json",
            "--setting-sources",
            "",
            "--disable-slash-commands",
            "--model",
            "model",
            "--output-format",
            "json",
            "--json-schema",
            "{\"schema\":1}"
        ]
    );
}
/// Template paths cannot introduce Codex dotted keys or unsafe path characters.
#[test]
fn b3_template_paths() {
    for path in ["relative", "/work.dot", "/work space", "/work/../other"] {
        assert!(
            step_arguments(
                PipelineRole::Implement,
                &cli(),
                Path::new(path),
                Path::new("/attempt"),
                "{}"
            )
            .is_err()
        );
    }
}
