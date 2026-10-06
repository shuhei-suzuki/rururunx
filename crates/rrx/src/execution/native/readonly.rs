//! Finite readonly Git vocabulary. Facts and captured bytes grant no Native
//! input, registration or Session permission.
use super::*;
use std::path::{Path, PathBuf};

pub(crate) const HELPER_LIMIT: usize = 32;
pub(super) const AGGREGATE_BYTES: usize = 8 * 1024 * 1024;
pub(super) const GIT_BYTES: usize = 1024 * 1024;
pub(super) fn next_output_budget(
    helper_count: usize,
    captured: usize,
    action: &NativePhaseHelperAction,
) -> Result<usize> {
    ensure!(
        helper_count < HELPER_LIMIT,
        "finite preparation helper budget exhausted"
    );
    let remaining = AGGREGATE_BYTES
        .checked_sub(captured)
        .context("preparation aggregate capture exceeds inclusive 8-MiB bound")?;
    ensure!(remaining > 0, "preparation aggregate capture exhausted");
    Ok(action.output_limit().min(remaining))
}

#[derive(Clone)]
pub(crate) enum NativePhaseHelperAction {
    Version,
    Head,
    Status,
    Tree {
        revision: String,
        inventory: BTreeMap<String, crate::execution::workflow_source::CommittedTreeEntry>,
    },
    Blob {
        oid: String,
        sha256: String,
    },
    SourceTop {
        expected: PathBuf,
    },
    SourceGitDir {
        expected: PathBuf,
    },
    SourceCommon {
        expected: PathBuf,
    },
    SourceRoots {
        revision: String,
        expected: Vec<String>,
    },
    TaskTop {
        expected: PathBuf,
    },
    TaskCommon {
        expected: PathBuf,
    },
    Branch {
        expected: String,
    },
}
impl NativePhaseHelperAction {
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::Version => "native_phase_version",
            _ => "git_helper",
        }
    }
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Version => "version",
            Self::Head => "exact_head",
            Self::Status => "readonly_status",
            Self::Tree { .. } => "committed_inventory",
            Self::Blob { .. } => "committed_blob",
            Self::SourceTop { .. } => "source_top",
            Self::SourceGitDir { .. } => "source_git_dir",
            Self::SourceCommon { .. } => "source_common",
            Self::SourceRoots { .. } => "source_roots",
            Self::TaskTop { .. } => "task_top",
            Self::TaskCommon { .. } => "task_common",
            Self::Branch { .. } => "task_branch",
        }
    }
    pub(super) fn output_limit(&self) -> usize {
        match self {
            Self::Version => super::version::OUTPUT_BYTES,
            _ => GIT_BYTES,
        }
    }
    pub(super) fn git_argv(&self) -> Result<Vec<String>> {
        Ok(match self {
            Self::Head => vec![
                "rev-parse".into(),
                "--verify".into(),
                "HEAD^{commit}".into(),
            ],
            Self::Status => vec![
                "status".into(),
                "--porcelain=v1".into(),
                "--untracked-files=all".into(),
            ],
            Self::Tree { revision, .. } => {
                ensure!(
                    super::super::valid_oid(revision),
                    "committed inventory OID differs"
                );
                vec![
                    "ls-tree".into(),
                    "-r".into(),
                    "-z".into(),
                    "-l".into(),
                    "--full-tree".into(),
                    revision.clone(),
                ]
            }
            Self::Blob { oid, .. } => {
                ensure!(super::super::valid_oid(oid), "committed blob OID differs");
                vec!["cat-file".into(), "blob".into(), oid.clone()]
            }
            Self::SourceTop { .. } | Self::TaskTop { .. } => {
                vec!["rev-parse".into(), "--show-toplevel".into()]
            }
            Self::SourceGitDir { .. } => vec![
                "rev-parse".into(),
                "--path-format=absolute".into(),
                "--git-dir".into(),
            ],
            Self::SourceCommon { .. } | Self::TaskCommon { .. } => vec![
                "rev-parse".into(),
                "--path-format=absolute".into(),
                "--git-common-dir".into(),
            ],
            Self::SourceRoots { revision, .. } => {
                ensure!(
                    super::super::valid_oid(revision),
                    "source root commit OID differs"
                );
                vec![
                    "rev-list".into(),
                    "--max-parents=0".into(),
                    revision.clone(),
                ]
            }
            Self::Branch { .. } => vec![
                "symbolic-ref".into(),
                "--quiet".into(),
                "--short".into(),
                "HEAD".into(),
            ],
            Self::Version => anyhow::bail!("version is not a Git action"),
        })
    }
    pub(super) fn qualify(&self, stdout: &[u8], unit: &ExecutionUnit) -> Result<()> {
        match self {
            Self::Version => anyhow::bail!("version requires the selected provider parser"),
            Self::Head => ensure!(
                stdout == format!("{}\n", unit.base_sha).as_bytes(),
                "readonly exact HEAD differs from original prepared commit"
            ),
            Self::Status => ensure!(stdout.is_empty(), "readonly worktree status differs"),
            Self::Tree {
                revision,
                inventory,
            } => {
                qualify_tree(revision, inventory, stdout, &unit.base_sha)?;
            }
            Self::Blob { sha256, .. } => ensure!(
                native_result::digest(stdout) == *sha256,
                "committed source blob differs"
            ),
            Self::SourceTop { expected }
            | Self::SourceGitDir { expected }
            | Self::SourceCommon { expected }
            | Self::TaskTop { expected }
            | Self::TaskCommon { expected } => {
                let text = std::str::from_utf8(stdout)?
                    .strip_suffix('\n')
                    .context("Git path frame differs")?;
                let path = PathBuf::from(text);
                ensure!(
                    path == *expected && path.canonicalize()? == *expected,
                    "Git namespace physical identity differs"
                );
            }
            Self::SourceRoots { expected, .. } => {
                let mut actual: Vec<_> = std::str::from_utf8(stdout)?
                    .lines()
                    .map(str::to_owned)
                    .collect();
                actual.sort();
                ensure!(
                    !actual.is_empty()
                        && actual.iter().all(|oid| super::super::valid_oid(oid))
                        && &actual == expected,
                    "original Project repository root identity differs"
                );
            }
            Self::Branch { expected } => ensure!(
                stdout == format!("{expected}\n").as_bytes(),
                "original Task branch differs"
            ),
        }
        Ok(())
    }
}

fn qualify_tree(
    revision: &str,
    inventory: &BTreeMap<String, crate::execution::workflow_source::CommittedTreeEntry>,
    stdout: &[u8],
    original_revision: &str,
) -> Result<()> {
    ensure!(
        revision == original_revision,
        "committed inventory revision differs"
    );
    let actual = crate::execution::workflow_source::parse_committed_tree(stdout)?;
    ensure!(&actual == inventory, "committed source inventory differs");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nongrant_helper_history_budget_keeps_count_aggregate_and_git_caps() {
        let action = NativePhaseHelperAction::Head;
        assert_eq!(next_output_budget(31, 0, &action).unwrap(), GIT_BYTES);
        assert!(next_output_budget(32, 0, &action).is_err());
        assert_eq!(
            next_output_budget(1, AGGREGATE_BYTES - 1, &action).unwrap(),
            1
        );
        assert!(next_output_budget(1, AGGREGATE_BYTES, &action).is_err());
        assert!(next_output_budget(1, AGGREGATE_BYTES + 1, &action).is_err());
        assert_eq!(
            next_output_budget(0, 0, &NativePhaseHelperAction::Version).unwrap(),
            64 * 1024
        );
    }
    #[test]
    fn nongrant_fixed_git_argv_rejects_mutable_or_option_object_names() {
        for bad in [
            "HEAD",
            "--help",
            "a;touch file",
            "A000000000000000000000000000000000000000",
            "a",
        ] {
            assert!(
                NativePhaseHelperAction::Tree {
                    revision: bad.into(),
                    inventory: BTreeMap::new()
                }
                .git_argv()
                .is_err()
            );
            assert!(
                NativePhaseHelperAction::Blob {
                    oid: bad.into(),
                    sha256: "b".repeat(64)
                }
                .git_argv()
                .is_err()
            );
        }
        let oid = "a".repeat(40);
        assert_eq!(
            NativePhaseHelperAction::Tree {
                revision: oid.clone(),
                inventory: BTreeMap::new()
            }
            .git_argv()
            .unwrap(),
            vec!["ls-tree", "-r", "-z", "-l", "--full-tree", oid.as_str()]
        );
        assert_eq!(
            NativePhaseHelperAction::Status.git_argv().unwrap(),
            vec!["status", "--porcelain=v1", "--untracked-files=all"]
        );
    }
    #[test]
    fn nongrant_original_committed_tree_checks_complete_physical_inventory() {
        let revision = "a".repeat(40);
        let tree = format!("100644 blob {} 3\tsource.txt\0", "b".repeat(40));
        let inventory =
            crate::execution::workflow_source::parse_committed_tree(tree.as_bytes()).unwrap();
        qualify_tree(&revision, &inventory, tree.as_bytes(), &revision).unwrap();
        for changed in [
            tree.replace("100644", "100755"),
            tree.replace(&"b".repeat(40), &"c".repeat(40)),
            tree.replace(" 3\t", " 4\t"),
            tree.replace("source.txt", "other.txt"),
            String::new(),
        ] {
            assert!(
                qualify_tree(&revision, &inventory, changed.as_bytes(), &revision).is_err(),
                "changed physical inventory must refuse"
            );
        }
        assert!(qualify_tree(&revision, &inventory, tree.as_bytes(), &"c".repeat(40)).is_err());
        for bad in [
            tree.trim_end_matches('\0').to_owned(),
            tree.repeat(2),
            tree.replace("100644", "040000"),
            tree.replace("source.txt", "../source.txt"),
            tree.replace("source.txt", "/source.txt"),
        ] {
            assert!(
                crate::execution::workflow_source::parse_committed_tree(bad.as_bytes()).is_err()
            );
        }
        assert!(
            crate::execution::workflow_source::parse_committed_tree(b"100644 blob a 3\tbad\xff\0")
                .is_err()
        );
    }
}

pub(super) fn namespace_actions(
    actor: &Arc<NativePreparationActor>,
) -> Result<Vec<NativePhaseHelperAction>> {
    use NativePhaseHelperAction as A;
    let marker = actor.launch().marker().original_plan();
    let project = marker.project().0;
    let task = marker.task_after().0;
    let unit = actor.launch().allocation().unit_snapshot();
    let (common, mut roots): (PathBuf, Vec<String>) =
        serde_json::from_str(&project.repository_identity)?;
    roots.sort();
    ensure!(
        project.root.canonicalize()? == project.root
            && common.is_absolute()
            && common.canonicalize()? == common
            && !roots.is_empty()
            && roots.iter().all(|r| super::super::valid_oid(r))
            && roots.len() <= 128,
        "original Project repository namespace unavailable"
    );
    // Reuse the original ownership policy with expected immutable facts before
    // any effects; each fact is then checked against an authentic Git capture.
    crate::git::validate_worktree_ownership(
        project,
        task,
        crate::git::WorktreeOwnershipFacts {
            source_top: project.root.clone(),
            source_git_dir: common.clone(),
            source_common: common.clone(),
            source_roots: roots.clone(),
            task_top: unit.worktree.clone(),
            task_common: common.clone(),
            branch: unit.branch.clone().context("original Task branch absent")?,
            revision: unit.base_sha.clone(),
        },
    )?;
    Ok(vec![
        A::SourceTop {
            expected: project.root.clone(),
        },
        A::SourceGitDir {
            expected: common.clone(),
        },
        A::SourceCommon {
            expected: common.clone(),
        },
        A::SourceRoots {
            revision: unit.base_sha.clone(),
            expected: roots,
        },
        A::TaskTop {
            expected: unit.worktree.clone(),
        },
        A::TaskCommon { expected: common },
        A::Branch {
            expected: unit.branch.clone().context("original Task branch absent")?,
        },
    ])
}
impl NativePhaseHelperAction {
    pub(super) fn cwd<'a>(&self, actor: &'a Arc<NativePreparationActor>) -> &'a Path {
        match self {
            Self::SourceTop { .. }
            | Self::SourceGitDir { .. }
            | Self::SourceCommon { .. }
            | Self::SourceRoots { .. } => &actor.launch().marker().original_plan().project().0.root,
            _ => &actor.launch().allocation().unit_snapshot().worktree,
        }
    }
}
