use super::*;
fn cli() -> PipelineCli {
    PipelineCli {
        executable: "/codex".into(),
        version: "fixture".into(),
        model: "model".into(),
        effort: Some("high".into()),
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
        assert_eq!(&args[..3], ["exec", "--sandbox", "workspace-write"]);
        assert!(
            args.windows(2)
                .any(|pair| pair == ["-c", "sandbox_workspace_write.network_access=false"])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["-c", "projects./work.trust_level=\"untrusted\""])
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
    assert_eq!(&confirm[..3], ["exec", "--sandbox", "read-only"]);
    assert!(
        confirm
            .windows(2)
            .any(|pair| pair == ["--output-schema", "/attempt/confirm.schema.json"])
    );
    let review = step_arguments(
        PipelineRole::Review,
        &cli(),
        Path::new("/work"),
        Path::new("/attempt"),
        "{\"schema\":1}",
    )
    .unwrap();
    assert!(
        review
            .windows(2)
            .any(|pair| pair == ["--permission-mode", "plan"])
    );
    assert!(
        review
            .windows(2)
            .any(|pair| pair == ["--tools", "Read,Grep,Glob"])
    );
    assert!(
        review
            .windows(2)
            .any(|pair| pair == ["--setting-sources", ""])
    );
    assert!(
        review
            .windows(2)
            .any(|pair| pair == ["--json-schema", "{\"schema\":1}"])
    );
    assert!(review.contains(&"--strict-mcp-config".into()));
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
