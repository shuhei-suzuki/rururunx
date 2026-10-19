use super::*;
fn root() -> tempfile::TempDir {
    tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap()
}
fn status(path: &str) -> Vec<u8> {
    format!("? {path}\0").into_bytes()
}
fn scope() -> Vec<String> {
    vec!["src/**".into()]
}
fn file(root: &Path, path: &str, bytes: &[u8]) {
    std::fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
    std::fs::write(root.join(path), bytes).unwrap();
}
/// Scope matches the complete path rather than a prefix.
#[test]
fn b5_scope_whole_path() {
    let root = root();
    file(root.path(), "src/file.rs.extra", b"code");
    assert!(
        changes(
            root.path(),
            &status("src/file.rs.extra"),
            &["src/*.rs".into()]
        )
        .is_err()
    );
    assert!(glob_match("src/**", "src/nested/file.rs"));
    assert!(!glob_match("src/*", "src/nested/file.rs"));
}
/// Protected instruction and Git paths require an explicit literal scope.
#[test]
fn b5_protected_paths() {
    for path in [
        ".gitconfig",
        ".gitattributes",
        ".gitmodules",
        ".claude/settings.json",
        ".codex/config.toml",
        "AGENTS.md",
        "CLAUDE.md",
        "src/CLAUDE.md",
        "src/AgEnTs.md",
        ".CoDeX/settings.toml",
    ] {
        let root = root();
        file(root.path(), path, b"data");
        assert!(
            changes(root.path(), &status(path), &["**".into()]).is_err(),
            "{path}"
        );
        assert!(
            changes(root.path(), &status(path), &[path.into()]).is_ok(),
            "{path}"
        );
    }
}
/// Only a regular file enters the captured change set.
#[test]
fn b5_symlink_refused() {
    let root = root();
    file(root.path(), "target", b"code");
    std::fs::create_dir(root.path().join("src")).unwrap();
    std::os::unix::fs::symlink("../target", root.path().join("src/link")).unwrap();
    assert!(changes(root.path(), &status("src/link"), &scope()).is_err());
}
/// A regular file over the per-file limit is not captured.
#[test]
fn b5_file_limit() {
    let root = root();
    file(root.path(), "src/large", &vec![b'a'; 1024 * 1024 + 1]);
    assert!(changes(root.path(), &status("src/large"), &scope()).is_err());
}
/// Binary bytes cannot become an Executor result.
#[test]
fn b5_binary_refused() {
    let root = root();
    file(root.path(), "src/binary", b"a\0b");
    assert!(changes(root.path(), &status("src/binary"), &scope()).is_err());
}
/// Changed path count cannot exceed the policy bound.
#[test]
fn b5_file_count() {
    let root = root();
    let mut rows = Vec::new();
    for i in 0..201 {
        let path = format!("src/{i}");
        file(root.path(), &path, b"x");
        rows.extend(status(&path));
    }
    assert!(changes(root.path(), &rows, &scope()).is_err());
}
/// Captured bytes cannot exceed the total addition bound.
#[test]
fn b5_total_bytes() {
    let root = root();
    let mut rows = Vec::new();
    for i in 0..9 {
        let path = format!("src/{i}");
        file(root.path(), &path, &vec![b'x'; 1024 * 1024]);
        rows.extend(status(&path));
    }
    assert!(changes(root.path(), &rows, &scope()).is_err());
}
/// A tracked gitlink or symlink mode cannot be represented as a regular blob.
#[test]
fn b5_index_modes() {
    let root = root();
    file(root.path(), "src/file", b"x");
    for mode in ["120000", "160000"] {
        let row = format!(
            "1 .M N... {mode} {mode} 100644 {} {} src/file\0",
            "a".repeat(40),
            "b".repeat(40)
        );
        assert!(changes(root.path(), row.as_bytes(), &scope()).is_err());
    }
}

/// Deleted files remain representable when their former parent directory is absent.
#[test]
fn b5_deleted_file() {
    let root = root();
    let row = format!(
        "1 .D N... 100644 100644 000000 {} {} src/removed/file.rs\0",
        "a".repeat(40),
        "b".repeat(40)
    );
    let changes = changes(root.path(), row.as_bytes(), &scope()).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].mode, "0");
    assert!(changes[0].bytes.is_none());
}
/// A parent symlink cannot redirect captured bytes outside the worktree.
#[test]
fn b5_parent_alias() {
    let root = root();
    file(root.path(), "real/file", b"code");
    std::os::unix::fs::symlink("real", root.path().join("src")).unwrap();
    assert!(changes(root.path(), &status("src/file"), &scope()).is_err());
}
/// Retained evidence is bound to both its exact length and its digest.
#[test]
fn b5_retained_digest() {
    let root = root();
    std::fs::write(root.path().join("last.md"), b"changed").unwrap();
    let receipt = json!({"bytes":7,"sha256":hash(b"initial"),"complete":true});
    assert!(retained(root.path(), "last.md", &receipt, 256).is_err());
    let receipt = json!({"bytes":8,"sha256":hash(b"changed"),"complete":true});
    assert!(retained(root.path(), "last.md", &receipt, 256).is_err());
}
/// Secret patterns in retained output cannot pass adoption.
#[test]
fn b5_output_secret() {
    let root = root();
    let bytes = b"EXAMPLE_TOKEN=fixture";
    std::fs::write(root.path().join("last.md"), bytes).unwrap();
    let receipt = json!({"bytes":bytes.len(),"sha256":hash(bytes),"complete":true});
    assert!(retained(root.path(), "last.md", &receipt, 256).is_err());
}
/// Added diff lines are scanned before evidence publication.
#[test]
fn b5_diff_secret() {
    assert!(SecretScanner::additions(b"+++ b/file\n+EXAMPLE_SECRET=fixture\n").is_err());
    assert!(SecretScanner::additions(b"-EXAMPLE_SECRET=fixture\n+safe\n").is_ok());
}

/// Adoption requires every retained output file rather than a subset of receipts.
#[test]
fn b5_evidence_files() {
    let root = root();
    let canonical = root.path().canonicalize().unwrap();
    let mut step: PipelineStep = serde_json::from_value(json!({"run_id":"fixture","seq":2,"attempt":1,"kind":"implement","provider":"codex","owner_epoch":1,"round":0,"state":"exited","version":1,"cookie":null,"pid":null,"birth":null,"step_base":"a".repeat(40),"input_sha256":"b".repeat(64),"result_commit":null,"result_tree":null,"body":{"exit":{}}})).unwrap();
    for (key, name) in [
        ("stdout", "stdout.jsonl"),
        ("stderr", "stderr.log"),
        ("last", "last.md"),
    ] {
        std::fs::write(canonical.join(name), b"data").unwrap();
        step.body["exit"][key] = json!({"bytes":4,"sha256":hash(b"data"),"complete":true});
    }
    evidence(&step, &canonical).unwrap();
    for name in ["stdout.jsonl", "stderr.log", "last.md"] {
        std::fs::remove_file(canonical.join(name)).unwrap();
        assert!(evidence(&step, &canonical).is_err(), "{name}");
        std::fs::write(canonical.join(name), b"data").unwrap();
    }
}

#[cfg(target_os = "macos")]
mod host;
