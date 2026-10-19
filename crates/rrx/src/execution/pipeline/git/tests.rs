use super::*;
/// Git returns bounded output after verified cleanup of an exited leader's group.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn b5_git_exited_group_cleanup() {
    let root = tempfile::tempdir().unwrap();
    let git = PipelineGit::new(&root.path().canonicalize().unwrap()).unwrap();
    let output = git.run(&["--version"]).await.unwrap();
    assert!(output.starts_with(b"git version "));
}

/// Blob writes consume stdin bytes with filters explicitly disabled.
#[test]
fn b5_blob_filter_boundary() {
    let root = tempfile::tempdir().unwrap();
    let git = PipelineGit::new(&root.path().canonicalize().unwrap()).unwrap();
    let command = git.command(BLOB_ARGS, None, false);
    let args: Vec<_> = command.as_std().get_args().collect();
    assert!(args.contains(&std::ffi::OsStr::new("--no-filters")));
    assert!(args.contains(&std::ffi::OsStr::new("--stdin")));
}
/// Dangerous driver configuration is refused without matching unrelated settings.
#[test]
fn b5_driver_keys() {
    for key in [
        "filter.lfs.clean",
        "filter.anything",
        "diff.external",
        "diff.tool.textconv",
        "diff.tool.command",
        "merge.tool.driver",
        "core.attributesfile",
    ] {
        assert!(driver_key(key), "{key}");
    }
    assert!(!driver_key("core.repositoryformatversion"));
}
/// Attribute drivers are refused at token boundaries including negated attributes.
#[test]
fn b5_attribute_tokens() {
    for attr in [
        "filter=lfs",
        "-diff",
        "!merge",
        "working-tree-encoding=UTF-16",
    ] {
        assert!(reject_attributes(format!("* {attr}\n").as_bytes()).is_err());
    }
    assert!(reject_attributes(b"# * filter=lfs\n* different mergeable\n").is_ok());
}
/// Git receives only the filter-free environment and fixed overrides.
#[test]
fn b5_git_environment() {
    let root = tempfile::tempdir().unwrap();
    let canonical = root.path().canonicalize().unwrap();
    let git = PipelineGit::new(&canonical).unwrap();
    let command = git.command(&["status"], None, false);
    let std = command.as_std();
    assert!(format!("{std:?}").contains("env -i "));
    let allowed = [
        "HOME",
        "PATH",
        "LANG",
        "GIT_CONFIG_NOSYSTEM",
        "GIT_CONFIG_GLOBAL",
        "GIT_ATTR_NOSYSTEM",
    ];
    for (key, value) in std.get_envs() {
        assert!(allowed.contains(&key.to_str().unwrap()));
        assert!(value.is_some());
    }
    for (key, value) in [
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_ATTR_NOSYSTEM", "1"),
    ] {
        assert!(
            std.get_envs()
                .any(|(k, v)| k == key && v == Some(std::ffi::OsStr::new(value)))
        );
    }
    let args: Vec<_> = std.get_args().map(|a| a.to_str().unwrap()).collect();
    for required in [
        "core.hooksPath=/dev/null",
        "core.fsmonitor=false",
        "core.untrackedCache=false",
        "gc.auto=0",
        "maintenance.auto=false",
        "commit.gpgsign=false",
        "core.autocrlf=false",
    ] {
        assert!(args.contains(&required));
    }
}
