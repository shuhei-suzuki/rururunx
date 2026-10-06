//! Primitive parser and real Git controls only; no Native actor is fabricated.
use super::*;
use crate::context::committed::InventoryEntry;
use crate::execution::workflow_source::{CommittedTreeEntry, parse_committed_tree};
use std::os::unix::fs::PermissionsExt;

fn inventory() -> (
    BTreeMap<String, InventoryEntry>,
    BTreeMap<String, CommittedTreeEntry>,
) {
    let tree = parse_committed_tree(
        format!("100644 blob {} 3\ta b\tline\n.txt\0", "b".repeat(40)).as_bytes(),
    )
    .unwrap();
    let index = BTreeMap::from([(
        "a b\tline\n.txt".into(),
        InventoryEntry {
            oid: "b".repeat(40),
            sha256: Some(native_result::digest(b"abc")),
            bytes: Some(3),
            skipped: None,
        },
    )]);
    (index, tree)
}
#[test]
fn nongrant_git_fixed_argv_order_exit_and_inclusive_capture_bounds() {
    use PhaseGitAction as A;
    let r = "a".repeat(40);
    let expected: Vec<Vec<&str>> = vec![
        vec!["rev-parse", "--show-toplevel"],
        vec!["rev-parse", "--path-format=absolute", "--git-dir"],
        vec!["rev-parse", "--path-format=absolute", "--git-common-dir"],
        vec!["rev-list", "--max-parents=0", &r],
        vec!["rev-parse", "--show-toplevel"],
        vec!["rev-parse", "--path-format=absolute", "--git-common-dir"],
        vec!["symbolic-ref", "--quiet", "--short", "HEAD"],
        vec!["rev-parse", "--verify", "HEAD^{commit}"],
        vec!["config", "--null", "--get-regexp", CONFIG_KEYS],
        vec!["ls-files", "-z", "-s", "-t", "-v"],
        vec![
            "diff-index",
            "--cached",
            "--quiet",
            "--no-ext-diff",
            "--no-textconv",
            "--ignore-submodules=none",
            &r,
            "--",
        ],
        vec![
            "ls-files",
            "-z",
            "--cached",
            "--",
            ATTR_UNSPECIFIED,
            ATTR_UNSET,
        ],
        vec![
            "-c",
            "core.untrackedCache=false",
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignored=matching",
            "--ignore-submodules=none",
        ],
    ];
    let mut capture = 64 * 1024;
    let mut labels = BTreeSet::new();
    for (i, a) in A::ORDERED.into_iter().enumerate() {
        assert_eq!(a.ordinal(), i);
        assert_eq!(a.argv(&r).unwrap(), expected[i]);
        assert!(labels.insert(a.label()));
        assert!(a.accepts_exit(Some(0)));
        assert_eq!(a.accepts_exit(Some(1)), a == A::Config);
        assert!(!a.accepts_exit(Some(2)));
        assert!(!a.accepts_exit(None));
        let action = NativePhaseHelperAction::Git(a);
        assert_eq!(action.kind(), "native_phase_git");
        let limit = if matches!(a, A::IndexEntries | A::ConversionAttrs | A::Status) {
            GIT_BYTES
        } else {
            64 * 1024
        };
        assert_eq!(next_output_budget(i + 1, capture, &action).unwrap(), limit);
        capture += limit;
        for bad in [
            "HEAD",
            "--help",
            "a;touch x",
            "A000000000000000000000000000000000000000",
            "a",
        ] {
            assert!(a.argv(bad).is_err());
        }
    }
    assert_eq!(capture, 3_866_624);
    assert!(capture < AGGREGATE_BYTES);
    assert_eq!(A::ORDERED.len() + 1, 14);
    assert!(next_output_budget(32, 0, &NativePhaseHelperAction::Version).is_err());
    assert_eq!(
        next_output_budget(31, AGGREGATE_BYTES - 1, &NativePhaseHelperAction::Version).unwrap(),
        1
    );
    assert!(next_output_budget(1, AGGREGATE_BYTES, &NativePhaseHelperAction::Version).is_err());
    assert!(next_output_budget(1, AGGREGATE_BYTES + 1, &NativePhaseHelperAction::Version).is_err());
}
#[test]
fn nongrant_git_config_nullable_nul_multivalue_and_closed_booleans() {
    for value in ["false", "FALSE", "no", "off", "0", ""] {
        let frame = format!(
            "core.bare\n{value}\0core.sparsecheckout\n{value}\0index.sparse\n{value}\0core.autocrlf\n{value}\0"
        );
        qualify_config(frame.as_bytes(), Some(0), 40).unwrap();
    }
    for frame in [
        b"core.filemode\0".as_slice(),
        b"core.filemode\nyes\0core.filemode\nTRUE\0",
        b"extensions.objectformat\nsha1\0",
    ] {
        qualify_config(frame, Some(0), 40).unwrap();
    }
    qualify_config(b"", Some(1), 40).unwrap();
    qualify_config(b"extensions.objectformat\nsha256\0", Some(0), 64).unwrap();
    for bad in [
        b"core.autocrlf\0".as_slice(),
        b"core.autocrlf\ntrue\0",
        b"core.autocrlf\ninput\0",
        b"core.autocrlf\nfalse\0core.autocrlf\ntrue\0",
        b"core.filemode\nfalse\0",
        b"core.filemode\n\0",
        b"core.bare\nmaybe\0",
        b"core.worktree\0",
        b"core.worktree\n\0",
        b"filter.canary.clean\ncat\0",
        b"core.bare\nfalse",
        b"core.bare\nfalse\0\0",
        b"core.bare\nfalse\ntrue\0",
        b"core.bare\nfalse\0credential.helper\nsecret\0",
        b"core.bare\nfal\xffse\0",
        b"extensions.objectformat\nsha256\0",
    ] {
        assert!(qualify_config(bad, Some(0), 40).is_err(), "{bad:?}");
    }
    assert!(qualify_config(b"", Some(1), 64).is_err());
    assert!(qualify_config(b"core.filemode\0", Some(1), 40).is_err());
    assert!(qualify_config(b"", Some(2), 40).is_err());
    let maximum = format!("core.bare\nfalse{}", "\0".repeat(64 * 1024));
    assert!(qualify_config(maximum.as_bytes(), Some(0), 40).is_err());
}
#[test]
fn nongrant_git_index_exact_bytes_oid_mode_stage_and_pathset() {
    let (index, tree) = inventory();
    let frame = format!("H 100644 {} 0\ta b\tline\n.txt\0", "b".repeat(40));
    qualify_index(frame.as_bytes(), &index, &tree).unwrap();
    qualify_pathset(b"a b\tline\n.txt\0", &index).unwrap();
    for changed in [
        frame.replacen("H ", "h ", 1),
        frame.replacen("H ", "S ", 1),
        frame.replacen("H ", "M ", 1),
        frame.replace("100644", "100755"),
        frame.replace("100644", "120000"),
        frame.replace("100644", "160000"),
        frame.replace(" 0\t", " 1\t"),
        frame.replace(&"b".repeat(40), &"c".repeat(40)),
        frame.replace("a b", "a c"),
        frame.repeat(2),
        frame.trim_end_matches('\0').into(),
        frame.replace("H 100644", "H  100644"),
        format!("{frame}\0"),
        String::new(),
    ] {
        assert!(
            qualify_index(changed.as_bytes(), &index, &tree).is_err(),
            "{changed:?}"
        );
    }
    for bad in [
        b"a b\tline\n.txt".as_slice(),
        b"a b\tline\n.txt\0a b\tline\n.txt\0",
        b"other\0",
        b"\0",
        b"",
        b"a b\tline\n.tx\xff\0",
    ] {
        assert!(qualify_pathset(bad, &index).is_err());
    }
    assert!(qualify_pathset(&vec![b'x'; GIT_BYTES + 1], &index).is_err());
    assert!(qualify_index(&vec![b'x'; GIT_BYTES + 1], &index, &tree).is_err());
}
#[test]
fn nongrant_git_one_line_and_oid_frames_are_exact() {
    assert_eq!(line(b"branch\n").unwrap(), "branch");
    for bad in [
        b"branch".as_slice(),
        b"branch\r\n",
        b"branch\nextra\n",
        b"\n",
        b"branch\0\n",
    ] {
        assert!(line(bad).is_err());
    }
    let r = "a".repeat(40);
    assert_eq!(
        oid_lines(format!("{r}\n").as_bytes(), 40).unwrap(),
        vec![r.clone()]
    );
    for bad in [
        r.clone(),
        format!("{r}\n\n"),
        format!("{r}\r\n"),
        format!("{}\n", "A".repeat(40)),
    ] {
        assert!(oid_lines(bad.as_bytes(), 40).is_err());
    }
    assert!(oid_lines(format!("{r}\n").as_bytes(), 64).is_err());
}

#[test]
fn nongrant_git_pathset_inclusive_mib_and_4096_entry_limits() {
    let mut index = BTreeMap::new();
    let mut output = Vec::new();
    for i in 0..256 {
        let path = format!("{i:03}{}", "x".repeat(4092));
        output.extend_from_slice(path.as_bytes());
        output.push(0);
        index.insert(
            path,
            InventoryEntry {
                oid: "b".repeat(40),
                sha256: None,
                bytes: None,
                skipped: None,
            },
        );
    }
    assert_eq!(output.len(), GIT_BYTES);
    qualify_pathset(&output, &index).unwrap();
    // Keep the same complete, unique and properly framed path set. Only the
    // inclusive byte bound changes, so removing its guard admits this image.
    let (path, entry) = index.pop_first().unwrap();
    index.insert(format!("z{path}"), entry);
    output.insert(0, b'z');
    assert_eq!(output.len(), GIT_BYTES + 1);
    assert!(qualify_pathset(&output, &index).is_err());
    index.clear();
    output.clear();
    for i in 0..4096 {
        let path = format!("p{i}");
        output.extend_from_slice(path.as_bytes());
        output.push(0);
        index.insert(
            path,
            InventoryEntry {
                oid: "b".repeat(40),
                sha256: None,
                bytes: None,
                skipped: None,
            },
        );
    }
    qualify_pathset(&output, &index).unwrap();
    output.extend_from_slice(b"extra\0");
    index.insert(
        "extra".into(),
        InventoryEntry {
            oid: "b".repeat(40),
            sha256: None,
            bytes: None,
            skipped: None,
        },
    );
    assert!(qualify_pathset(&output, &index).is_err());
}

struct GitFixture {
    temp: tempfile::TempDir,
    root: PathBuf,
    home: PathBuf,
    git: PathBuf,
}
impl GitFixture {
    fn new(attributes: Option<&str>) -> Self {
        let temp = tempfile::Builder::new()
            .prefix("rrx-native-git-")
            .tempdir()
            .unwrap();
        let root = temp.path().join("repo");
        let home = temp.path().join("home");
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(&home).unwrap();
        // The platform temp directory can be an alias (for example /var on
        // macOS); retain the physical paths returned by Git observations.
        let root = root.canonicalize().unwrap();
        let home = home.canonicalize().unwrap();
        let git = crate::execution::resources::resolve_program("git")
            .unwrap()
            .canonicalize()
            .unwrap();
        let f = Self {
            temp,
            root,
            home,
            git,
        };
        f.setup(&["init", "-q"]);
        f.setup(&["config", "user.name", "primitive"]);
        f.setup(&["config", "user.email", "primitive@example.invalid"]);
        f.setup(&["config", "core.filemode", "true"]);
        f.setup(&["config", "core.autocrlf", "false"]);
        std::fs::write(f.root.join("file.txt"), b"original\n").unwrap();
        std::fs::write(f.root.join("large.bin"), vec![b'x'; 256 * 1024 + 1]).unwrap();
        std::fs::write(f.root.join("run.sh"), b"exit 0\n").unwrap();
        std::fs::set_permissions(
            f.root.join("run.sh"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        if let Some(a) = attributes {
            std::fs::write(f.root.join(".gitattributes"), a).unwrap();
        }
        f.setup(&["add", "--all"]);
        f.setup(&["commit", "-qm", "primitive"]);
        f
    }
    fn setup(&self, args: &[&str]) -> std::process::Output {
        let out = std::process::Command::new(&self.git)
            .current_dir(&self.root)
            .args(args)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.home)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", self.home.join("config"))
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "primitive Git setup failed: {:?}",
            out.stderr
        );
        out
    }
    async fn collect(&self, action: PhaseGitAction, revision: &str) -> std::process::Output {
        let mut command =
            super::super::super::results::git_command_for(&self.root, &self.git).unwrap();
        // Hermetic attribute/config sources are test isolation only.
        command
            .args(action.argv(revision).unwrap())
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.home)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", self.home.join("config"))
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .env("GIT_NO_LAZY_FETCH", "1")
            .stdin(Stdio::null())
            .output()
            .await
            .unwrap()
    }
    fn revision(&self) -> String {
        line(&self.setup(&["rev-parse", "HEAD"]).stdout)
            .unwrap()
            .into()
    }
    fn inventory(
        &self,
        r: &str,
    ) -> (
        BTreeMap<String, InventoryEntry>,
        BTreeMap<String, CommittedTreeEntry>,
    ) {
        let tree = parse_committed_tree(
            &self
                .setup(&["ls-tree", "-r", "-z", "-l", "--full-tree", r])
                .stdout,
        )
        .unwrap();
        let index = tree
            .iter()
            .map(|(p, e)| {
                (
                    p.clone(),
                    InventoryEntry {
                        oid: e.oid.clone(),
                        sha256: None,
                        bytes: None,
                        skipped: Some("primitive OID only".into()),
                    },
                )
            })
            .collect();
        (index, tree)
    }
}
#[tokio::test]
async fn nongrant_real_git_conversion_positive_executable_large_binary_and_unset_text() {
    for attrs in [None, Some("*.bin binary\n"), Some("*.txt -text\n")] {
        let f = GitFixture::new(attrs);
        let r = f.revision();
        let top = f.collect(PhaseGitAction::SourceTop, &r).await;
        assert!(top.status.success());
        exact_path(&top.stdout, &f.root).unwrap();
        let (index, tree) = f.inventory(&r);
        let cfg = f.collect(PhaseGitAction::Config, &r).await;
        qualify_config(&cfg.stdout, cfg.status.code(), 40).unwrap();
        let entries = f.collect(PhaseGitAction::IndexEntries, &r).await;
        assert!(entries.status.success());
        qualify_index(&entries.stdout, &index, &tree).unwrap();
        let equal = f.collect(PhaseGitAction::IndexTree, &r).await;
        assert!(equal.status.success());
        let attrs = f.collect(PhaseGitAction::ConversionAttrs, &r).await;
        assert!(attrs.status.success());
        qualify_pathset(&attrs.stdout, &index).unwrap();
        let status = f.collect(PhaseGitAction::Status, &r).await;
        assert!(status.status.success() && status.stdout.is_empty());
        f.temp.close().unwrap();
    }
}
#[tokio::test]
async fn nongrant_real_git_effective_attributes_refuse_before_status_and_canary() {
    for (origin, attrs) in [
        ("committed", "* filter=canary\n"),
        ("untracked", "* filter=canary\n"),
        ("info", "* filter=canary\n"),
        ("global", "* filter=canary\n"),
        ("macro", "[attr]m filter=canary\n* m\n"),
        ("index", "* filter=canary\n"),
        ("text", "* text=auto\n"),
        ("eol", "* eol=crlf\n"),
        ("crlf", "* crlf\n"),
        ("unset-crlf", "* -crlf\n"),
        ("ident", "* ident\n"),
        ("encoding", "* working-tree-encoding=UTF-16\n"),
        ("process", "* filter=canary\n"),
    ] {
        let committed = matches!(origin, "committed" | "index");
        let f = GitFixture::new(if committed { Some(attrs) } else { None });
        let r = f.revision();
        let (index, _) = f.inventory(&r);
        match origin {
            "committed" => {}
            "index" => {
                std::fs::remove_file(f.root.join(".gitattributes")).unwrap();
            }
            "info" => {
                std::fs::write(f.root.join(".git/info/attributes"), attrs).unwrap();
            }
            "global" => {
                let path = f.home.join("attributes");
                std::fs::write(&path, attrs).unwrap();
                f.setup(&["config", "core.attributesFile", path.to_str().unwrap()]);
            }
            _ => {
                std::fs::write(f.root.join(".gitattributes"), attrs).unwrap();
            }
        }
        let canary = f.temp.path().join("canary");
        let filter = format!("touch {}; cat", canary.display());
        f.setup(&[
            "config",
            if origin == "process" {
                "filter.canary.process"
            } else {
                "filter.canary.clean"
            },
            &filter,
        ]);
        std::fs::write(f.root.join("file.txt"), b"modified\r\n").unwrap();
        let observed = f.collect(PhaseGitAction::ConversionAttrs, &r).await;
        assert!(observed.status.success(), "{origin}: {:?}", observed.stderr);
        assert!(
            qualify_pathset(&observed.stdout, &index).is_err(),
            "{origin} must refuse before Status"
        );
        assert!(
            !canary.exists(),
            "{origin}: filter unexpectedly invoked by readonly collector"
        );
        f.temp.close().unwrap();
    }
}
#[tokio::test]
async fn nongrant_real_git_index_flags_tree_drift_and_original_revision_roots() {
    let f = GitFixture::new(None);
    let r = f.revision();
    let (index, tree) = f.inventory(&r);
    for flag in ["--assume-unchanged", "--skip-worktree"] {
        f.setup(&["update-index", flag, "file.txt"]);
        let entries = f.collect(PhaseGitAction::IndexEntries, &r).await;
        assert!(entries.status.success() && qualify_index(&entries.stdout, &index, &tree).is_err());
        f.setup(&[
            "update-index",
            "--no-assume-unchanged",
            "--no-skip-worktree",
            "file.txt",
        ]);
    }
    std::fs::set_permissions(
        f.root.join("run.sh"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    f.setup(&["add", "run.sh"]);
    let equal = f.collect(PhaseGitAction::IndexTree, &r).await;
    assert_eq!(equal.status.code(), Some(1));
    let first = f.collect(PhaseGitAction::SourceRoots, &r).await;
    let branch = line(&f.setup(&["symbolic-ref", "--short", "HEAD"]).stdout)
        .unwrap()
        .to_owned();
    f.setup(&["update-ref", "-d", &format!("refs/heads/{branch}")]);
    let second = f.collect(PhaseGitAction::SourceRoots, &r).await;
    assert!(first.status.success() && second.status.success());
    assert_eq!(first.stdout, second.stdout);
    f.temp.close().unwrap();
}
#[tokio::test]
async fn nongrant_real_git_autocrlf_filemode_nullable_and_multivalue_rejections() {
    for (key, value) in [
        ("core.autocrlf", "true"),
        ("core.autocrlf", "input"),
        ("core.filemode", "false"),
    ] {
        let f = GitFixture::new(None);
        let r = f.revision();
        f.setup(&["config", key, value]);
        let observed = f.collect(PhaseGitAction::Config, &r).await;
        assert!(qualify_config(&observed.stdout, observed.status.code(), 40).is_err());
        f.temp.close().unwrap();
    }
    let f = GitFixture::new(None);
    let r = f.revision();
    f.setup(&["config", "--add", "core.autocrlf", "true"]);
    let observed = f.collect(PhaseGitAction::Config, &r).await;
    assert!(qualify_config(&observed.stdout, observed.status.code(), 40).is_err());
    // Genuine Git valueless key frame, using only this fixture's local config.
    std::fs::write(f.root.join(".git/config"), "[core]\nfilemode\n").unwrap();
    let observed = f.collect(PhaseGitAction::Config, &r).await;
    assert_eq!(observed.stdout, b"core.filemode\0");
    qualify_config(&observed.stdout, observed.status.code(), 40).unwrap();
    f.temp.close().unwrap();
}
