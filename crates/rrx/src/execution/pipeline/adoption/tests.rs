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
        ".McP.JsOn",
        "src/AGENTS.override.md",
        "src/CLAUDE.local.md",
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
/// Unicode aliases and protected components require explicit scope without banning ordinary Unicode.
#[test]
fn b5_f2_protected_unicode() {
    for path in [
        "AGENTſ.md",
        "src/agentſ.override.md",
        ".gitattributeſ",
        "K/AGENTS.md",
    ] {
        let root = root();
        file(root.path(), path, b"data");
        assert_eq!(
            changes(root.path(), &status(path), &["**".into()])
                .err()
                .unwrap()
                .to_string(),
            "protected_path",
            "{path}"
        );
        assert!(changes(root.path(), &status(path), &[path.into()]).is_ok());
    }
    for path in ["src/日本語.rs", "src/Kelvin.rs", "src/agentſ.txt"] {
        let root = root();
        file(root.path(), path, b"data");
        assert!(changes(root.path(), &status(path), &["**".into()]).is_ok());
    }
}
/// Every ignored protected component is refused while ordinary ignored output is accepted.
#[test]
fn b5_f2_ignored_components() {
    reject_ignored_entries("?? AGENTS.md\0 M CLAUDE.md\0!! target/\0!! 日本語/\0".as_bytes())
        .unwrap();
    for path in [
        ".gitignore",
        "AGENTS.override.md",
        "src/CLAUDE.local.md",
        "src/.claude/",
        ".CoDeX/",
        "src/AGENTſ.md",
    ] {
        let rows = format!("!! target/\0!! {path}\0");
        assert_eq!(
            reject_ignored_entries(rows.as_bytes())
                .unwrap_err()
                .to_string(),
            "protected_path",
            "{path}"
        );
    }
}
/// Invalid UTF-8 components are ordinary without hiding adjacent protected components.
#[test]
fn b5_f3_ignored_non_utf8_components() {
    reject_ignored_entries(b"!! target/\xff\0!! .git\xff/config\0!! AGENTS.md\xff\0").unwrap();
    for path in [
        b"\xff/AGENTS.md".as_slice(),
        b"CLAUDE.md/\xff",
        b"ordinary/\xff/.codex/config.toml",
        "\u{fffd}/AGENTſ.md".as_bytes(),
    ] {
        let mut rows = b"!! target/\xff\0!! ".to_vec();
        rows.extend(path);
        rows.push(0);
        assert_eq!(
            reject_ignored_entries(&rows).unwrap_err().to_string(),
            "protected_path"
        );
    }
}
/// Ignore rules are refused at any depth and casing even when the scope names them.
#[test]
fn b5_f3_gitignore_explicit() {
    let tracked = |kind: &str, mode: &str, path: &str| {
        format!(
            "1 {kind} N... 100644 100644 {mode} {} {} {path}\0",
            "a".repeat(40),
            "b".repeat(40)
        )
        .into_bytes()
    };
    for path in [".gitignore", "src/.gitignore", ".GitIgnore"] {
        let scope = [path.to_owned(), "src/**".to_owned()];
        let (present, absent) = (root(), root());
        file(present.path(), path, b"/scratch/\n");
        for (dir, rows) in [
            (&present, status(path)),
            (&present, tracked(".M", "100644", path)),
            (&absent, tracked(".D", "000000", path)),
        ] {
            assert_eq!(
                changes(dir.path(), &rows, &scope)
                    .err()
                    .unwrap()
                    .to_string(),
                "protected_path",
                "{path}"
            );
        }
    }
    // Other protected names keep the explicit-scope exception.
    for path in ["AGENTS.md", ".gitignore.bak"] {
        let dir = root();
        file(dir.path(), path, b"data");
        assert!(
            changes(dir.path(), &status(path), &[path.into(), "src/**".into()]).is_ok(),
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
    assert!(
        SecretScanner::additions(
            b"diff --git a/file b/file\n+++ b/file\n@@ -0,0 +1 @@\n+EXAMPLE_SECRET=fixture\n"
        )
        .is_err()
    );
    assert!(SecretScanner::additions(b"-EXAMPLE_SECRET=fixture\n+safe\n").is_ok());
}

/// Hunk additions beginning with pluses cannot masquerade as file headers.
#[test]
fn b5_f1_plus_secret() {
    assert!(
        SecretScanner::additions(
            b"diff --git a/file b/file\n+++ b/file\n@@ -0,0 +1 @@\n+++ EXAMPLE_SECRET=fixture\n"
        )
        .is_err()
    );
    assert!(SecretScanner::additions(b"diff --git a/file b/EXAMPLE_SECRET=fixture\n+++ b/EXAMPLE_SECRET=fixture\n@@ -0,0 +1 @@\n+safe\ndiff --git a/next b/EXAMPLE_SECRET=fixture\n+++ b/EXAMPLE_SECRET=fixture\n@@ -0,0 +1 @@\n+safe\n").is_ok());
}

/// Git executable mode depends only on the owner execute bit.
#[test]
fn b5_f1_owner_executable() {
    let root = root();
    file(root.path(), "src/file", b"safe");
    for (mode, expected) in [(0o654, "100644"), (0o744, "100755")] {
        std::fs::set_permissions(
            root.path().join("src/file"),
            std::fs::Permissions::from_mode(mode),
        )
        .unwrap();
        assert_eq!(
            changes(root.path(), &status("src/file"), &scope()).unwrap()[0].mode,
            expected
        );
    }
}

/// Unpublished crash remnants never replace the final complete readonly artifact.
#[test]
fn b5_f1_artifact_crash_recovery() {
    for crash_bytes in [b"".as_slice(), b"com", b"complete patch"] {
        let root = root();
        let final_path = root.path().join("diff.patch");
        // On-disk states after creation, partial write, and full write before chmod.
        let mut artifact = PendingArtifact::new(&final_path).unwrap();
        artifact
            .pending
            .as_file_mut()
            .write_all(crash_bytes)
            .unwrap();
        let (_, pending_path) = artifact.pending.keep().unwrap();
        assert_eq!(std::fs::read(&pending_path).unwrap(), crash_bytes);
        assert!(!final_path.exists());
        write_artifact(&final_path, b"complete patch").unwrap();
        assert_eq!(std::fs::read(&final_path).unwrap(), b"complete patch");
        assert_eq!(
            std::fs::metadata(&final_path).unwrap().permissions().mode() & 0o777,
            0o400
        );
        // Exited recovery also rebuilds legacy incomplete/writable final artifacts.
        std::fs::set_permissions(&final_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::write(&final_path, crash_bytes).unwrap();
        write_artifact(&final_path, b"complete patch").unwrap();
        assert_eq!(std::fs::read(&final_path).unwrap(), b"complete patch");
        assert_eq!(
            std::fs::metadata(&final_path).unwrap().permissions().mode() & 0o777,
            0o400
        );
    }
}

/// Evidence publication refuses aliases without changing their targets.
#[test]
fn b5_f1_artifact_alias() {
    let root = root();
    file(root.path(), "target", b"original");
    let path = root.path().join("diff.patch");
    std::os::unix::fs::symlink("target", &path).unwrap();
    assert!(write_artifact(&path, b"patch").is_err());
    assert_eq!(
        std::fs::read(root.path().join("target")).unwrap(),
        b"original"
    );
}

/// Committed recovery requires the recorded complete readonly diff evidence.
#[test]
fn b5_f1_committed_diff_evidence() {
    let root = root();
    let mut step: PipelineStep = serde_json::from_value(json!({"run_id":"fixture","seq":2,"attempt":1,"kind":"implement","provider":"codex","owner_epoch":1,"round":0,"state":"committed","version":1,"cookie":null,"pid":null,"birth":null,"step_base":"a".repeat(40),"input_sha256":"b".repeat(64),"result_commit":"c".repeat(40),"result_tree":"d".repeat(40),"body":{}})).unwrap();
    let path = root.path().join("diff.patch");
    step.body["commit_evidence"] = json!({"diff_bytes":5,"diff_sha256":hash(b"patch")});
    assert!(diff_evidence(&step, root.path()).is_err());
    write_artifact(&path, b"patch").unwrap();
    diff_evidence(&step, root.path()).unwrap();
    step.body["commit_evidence"]["diff_bytes"] = json!(6);
    assert!(diff_evidence(&step, root.path()).is_err());
    step.body["commit_evidence"]["diff_bytes"] = json!(5);
    step.body["commit_evidence"]["diff_sha256"] = json!(hash(b"other"));
    assert!(diff_evidence(&step, root.path()).is_err());
    step.body["commit_evidence"]["diff_sha256"] = json!(hash(b"patch"));
    for bytes in [b"pa".as_slice(), b"other", b"patch"] {
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::write(&path, bytes).unwrap();
        assert!(diff_evidence(&step, root.path()).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o400)).unwrap();
        if bytes != b"patch" {
            assert!(diff_evidence(&step, root.path()).is_err());
        }
    }
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
