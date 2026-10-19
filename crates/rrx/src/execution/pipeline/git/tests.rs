use super::*;

/// Repository inspection reads exactly the scopes Git enables and preserves their union.
#[tokio::test]
async fn b5_config_active_scopes() {
    let config = tempfile::NamedTempFile::new().unwrap();
    for enabled in [false, true] {
        let mut calls = 0;
        let bytes = read_local_config(|args| {
            calls += 1;
            std::future::ready(Ok(if args.contains(&"--get") {
                if enabled {
                    b"true\n".to_vec()
                } else {
                    b"false\n".to_vec()
                }
            } else if args.contains(&"--git-path") {
                format!("{}\n", config.path().display()).into_bytes()
            } else if args.contains(&"--worktree") {
                assert!(enabled, "disabled worktree scope must not be read");
                b"file:config.worktree\0filter.fixture.clean\nfalse\0".to_vec()
            } else {
                assert!(args.contains(&"--local"));
                b"file:config\0diff.external\nfalse\0".to_vec()
            }))
        })
        .await
        .unwrap();
        let mut expected = b"file:config\0diff.external\nfalse\0".to_vec();
        if enabled {
            expected.extend(b"file:config.worktree\0filter.fixture.clean\nfalse\0");
        }
        assert_eq!(bytes, expected);
        assert_eq!(calls, if enabled { 4 } else { 2 });
    }
}

/// Failure to inspect any active configuration source refuses the whole inspection.
#[tokio::test]
async fn b5_config_read_failure() {
    let config = tempfile::NamedTempFile::new().unwrap();
    for failing_call in 0..4 {
        let mut call = 0;
        let result = read_local_config(|args| {
            let result = if call == failing_call {
                Err(anyhow::anyhow!("fixture config read failure"))
            } else if args.contains(&"--get") {
                Ok(b"true\n".to_vec())
            } else if args.contains(&"--git-path") {
                Ok(format!("{}\n", config.path().display()).into_bytes())
            } else {
                Ok(Vec::new())
            };
            call += 1;
            std::future::ready(result)
        })
        .await;
        assert_eq!(
            result.unwrap_err().to_string(),
            "fixture config read failure"
        );
        assert_eq!(call, failing_call + 1);
    }
}

/// An absent enabled worktree config contributes no bytes and needs no scope read.
#[tokio::test]
async fn b5_config_absent_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.worktree");
    let bytes = read_local_config(|args| {
        std::future::ready(Ok(if args.contains(&"--get") {
            b"true\n".to_vec()
        } else if args.contains(&"--git-path") {
            format!("{}\n", path.display()).into_bytes()
        } else {
            assert!(args.contains(&"--local"), "absent scope must not be read");
            b"file:config\0core.bare\nfalse\0".to_vec()
        }))
    })
    .await
    .unwrap();
    assert_eq!(bytes, b"file:config\0core.bare\nfalse\0");
}

/// Nonregular worktree config paths refuse inspection without following symlinks.
#[cfg(unix)]
#[tokio::test]
async fn b5_config_nonregular_file() {
    let dir = tempfile::tempdir().unwrap();
    let regular = dir.path().join("regular");
    std::fs::write(&regular, b"").unwrap();
    let link = dir.path().join("link");
    std::os::unix::fs::symlink(&regular, &link).unwrap();
    let dangling = dir.path().join("dangling");
    std::os::unix::fs::symlink(dir.path().join("absent"), &dangling).unwrap();
    for path in [dir.path(), link.as_path(), dangling.as_path()] {
        let result = read_local_config(|args| {
            std::future::ready(Ok(if args.contains(&"--get") {
                b"true\n".to_vec()
            } else if args.contains(&"--git-path") {
                format!("{}\n", path.display()).into_bytes()
            } else {
                assert!(
                    args.contains(&"--local"),
                    "nonregular scope must not be read"
                );
                Vec::new()
            }))
        })
        .await;
        assert_eq!(
            result.unwrap_err().to_string(),
            "worktree config is not a regular file"
        );
    }
}

/// An unrecognized extension result cannot disable inspection of worktree configuration.
#[tokio::test]
async fn b5_config_invalid_extension() {
    let result = read_local_config(|args| {
        std::future::ready(Ok(if args.contains(&"--get") {
            b"invalid\n".to_vec()
        } else {
            Vec::new()
        }))
    })
    .await;
    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid worktreeConfig value"
    );
}

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

#[cfg(target_os = "macos")]
mod host;
