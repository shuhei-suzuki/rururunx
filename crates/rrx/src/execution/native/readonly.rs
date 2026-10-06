//! Finite private Git observations. None grants prepared input or a Session.
use super::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(crate) const HELPER_LIMIT: usize = 32;
pub(super) const AGGREGATE_BYTES: usize = 8 * 1024 * 1024;
pub(super) const GIT_BYTES: usize = 1024 * 1024;
pub(crate) const GIT_ACTIONS: usize = 13;
pub(super) const BATCH_SECONDS: u64 = 180;
const CONFIG_KEYS: &str = r"^(core\.(bare|worktree|sparsecheckout|autocrlf|filemode)|index\.sparse|extensions\.objectformat)$";
const ATTR_UNSPECIFIED: &str = ":(attr:!text !eol !crlf !ident !filter !working-tree-encoding)";
const ATTR_UNSET: &str = ":(attr:-text !eol !crlf !ident !filter !working-tree-encoding)";

pub(super) fn next_output_budget(
    count: usize,
    captured: usize,
    action: &NativePhaseHelperAction,
) -> Result<usize> {
    ensure!(
        count < HELPER_LIMIT,
        "finite preparation helper budget exhausted"
    );
    let remaining = AGGREGATE_BYTES
        .checked_sub(captured)
        .context("preparation aggregate capture exceeds inclusive 8-MiB bound")?;
    ensure!(remaining > 0, "preparation aggregate capture exhausted");
    Ok(action.output_limit().min(remaining))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PhaseGitAction {
    SourceTop,
    SourceGitDir,
    SourceCommon,
    SourceRoots,
    TaskTop,
    TaskCommon,
    Branch,
    Head,
    Config,
    IndexEntries,
    IndexTree,
    ConversionAttrs,
    Status,
}
impl PhaseGitAction {
    pub(crate) const ORDERED: [Self; GIT_ACTIONS] = [
        Self::SourceTop,
        Self::SourceGitDir,
        Self::SourceCommon,
        Self::SourceRoots,
        Self::TaskTop,
        Self::TaskCommon,
        Self::Branch,
        Self::Head,
        Self::Config,
        Self::IndexEntries,
        Self::IndexTree,
        Self::ConversionAttrs,
        Self::Status,
    ];
    pub(crate) fn ordinal(self) -> usize {
        Self::ORDERED
            .iter()
            .position(|a| *a == self)
            .expect("finite Git action")
    }
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::SourceTop => "source_top",
            Self::SourceGitDir => "source_git_dir",
            Self::SourceCommon => "source_common",
            Self::SourceRoots => "source_roots",
            Self::TaskTop => "task_top",
            Self::TaskCommon => "task_common",
            Self::Branch => "branch",
            Self::Head => "head",
            Self::Config => "config",
            Self::IndexEntries => "index_entries",
            Self::IndexTree => "index_tree",
            Self::ConversionAttrs => "conversion_attrs",
            Self::Status => "status",
        }
    }
    fn output_limit(self) -> usize {
        if matches!(
            self,
            Self::IndexEntries | Self::ConversionAttrs | Self::Status
        ) {
            GIT_BYTES
        } else {
            super::version::OUTPUT_BYTES
        }
    }
    pub(super) fn accepts_exit(self, code: Option<i32>) -> bool {
        code == Some(0) || (self == Self::Config && code == Some(1))
    }
    pub(super) fn argv(self, revision: &str) -> Result<Vec<String>> {
        ensure!(
            super::super::valid_oid(revision),
            "Git operand requires original exact OID"
        );
        let args: Vec<&str> = match self {
            Self::SourceTop | Self::TaskTop => vec!["rev-parse", "--show-toplevel"],
            Self::SourceGitDir => vec!["rev-parse", "--path-format=absolute", "--git-dir"],
            Self::SourceCommon | Self::TaskCommon => {
                vec!["rev-parse", "--path-format=absolute", "--git-common-dir"]
            }
            Self::SourceRoots => vec!["rev-list", "--max-parents=0", revision],
            Self::Branch => vec!["symbolic-ref", "--quiet", "--short", "HEAD"],
            Self::Head => vec!["rev-parse", "--verify", "HEAD^{commit}"],
            Self::Config => vec!["config", "--null", "--get-regexp", CONFIG_KEYS],
            Self::IndexEntries => vec!["ls-files", "-z", "-s", "-t", "-v"],
            Self::IndexTree => vec![
                "diff-index",
                "--cached",
                "--quiet",
                "--no-ext-diff",
                "--no-textconv",
                "--ignore-submodules=none",
                revision,
                "--",
            ],
            Self::ConversionAttrs => vec![
                "ls-files",
                "-z",
                "--cached",
                "--",
                ATTR_UNSPECIFIED,
                ATTR_UNSET,
            ],
            Self::Status => vec![
                "-c",
                "core.untrackedCache=false",
                "status",
                "--porcelain=v1",
                "-z",
                "--untracked-files=all",
                "--ignored=matching",
                "--ignore-submodules=none",
            ],
        };
        Ok(args.into_iter().map(str::to_owned).collect())
    }
    pub(super) fn cwd(self, actor: &Arc<NativePreparationActor>) -> &Path {
        if self.ordinal() < Self::TaskTop.ordinal() {
            &actor.launch().marker().original_plan().project().0.root
        } else {
            &actor.launch().allocation().unit_snapshot().worktree
        }
    }
}
#[derive(Clone)]
pub(crate) enum NativePhaseHelperAction {
    Version,
    Git(PhaseGitAction),
}
impl NativePhaseHelperAction {
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::Version => "native_phase_version",
            Self::Git(_) => "native_phase_git",
        }
    }
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Version => "version",
            Self::Git(a) => a.label(),
        }
    }
    pub(super) fn output_limit(&self) -> usize {
        match self {
            Self::Version => super::version::OUTPUT_BYTES,
            Self::Git(a) => a.output_limit(),
        }
    }
    pub(super) fn cwd<'a>(&self, actor: &'a Arc<NativePreparationActor>) -> &'a Path {
        match self {
            Self::Version => &actor.launch().allocation().unit_snapshot().worktree,
            Self::Git(a) => a.cwd(actor),
        }
    }
}

/// One batch identity retains the SAME original launch, Source seal and Frame.
/// Private construction runs outside locks before the first Git intent.
pub(crate) struct NativeGitSourceSeal {
    launch: Arc<crate::state::managed_binding::PhaseLaunchParts>,
    source: Arc<crate::execution::workflow_source::SourceNativePreparationSeal>,
    input_digest: String,
    started: std::sync::OnceLock<tokio::time::Instant>,
}
impl NativeGitSourceSeal {
    pub(super) fn new(actor: &Arc<NativePreparationActor>) -> Result<Arc<Self>> {
        actor.validate_open()?;
        let launch = actor.launch().clone();
        let source = launch.preparation_seal()?;
        source.qualify_git(
            launch.allocation(),
            launch.marker().original_plan().project().0,
        )?;
        Ok(Arc::new(Self {
            input_digest: native_result::digest(launch.allocation().facts().input_bytes),
            launch,
            source,
            started: std::sync::OnceLock::new(),
        }))
    }
    pub(crate) fn matches_actor(&self, actor: &Arc<NativePreparationActor>) -> bool {
        Arc::ptr_eq(&self.launch, actor.launch())
    }
    pub(crate) fn revision(&self) -> &str {
        self.source.revision()
    }
    pub(crate) fn target(&self, action: PhaseGitAction) -> String {
        let f = self.launch.allocation().facts();
        let hash = native_result::digest(
            format!(
                "{}:{}:{}:{}:{}:{}:{}:{}",
                f.operation_id,
                f.pair_id,
                f.epoch,
                f.profile_digest,
                self.revision(),
                self.source.inventory_digest(),
                self.input_digest,
                action.label()
            )
            .as_bytes(),
        );
        format!("git:{}:{hash}", action.label())
    }
    pub(crate) fn deadline(&self) -> Option<tokio::time::Instant> {
        self.started
            .get()
            .map(|s| *s + Duration::from_secs(BATCH_SECONDS))
    }
    pub(super) fn start_batch(&self) {
        self.started.get_or_init(tokio::time::Instant::now);
    }
    pub(crate) fn validate_deadline(&self) -> Result<()> {
        ensure!(
            self.deadline()
                .is_none_or(|d| tokio::time::Instant::now() < d),
            "Git batch deadline expired"
        );
        Ok(())
    }
    pub(super) fn qualify(
        &self,
        action: PhaseGitAction,
        stdout: &[u8],
        exit: Option<i32>,
    ) -> Result<()> {
        ensure!(action.accepts_exit(exit), "Git action exit differs");
        ensure!(
            stdout.len() <= action.output_limit(),
            "Git action output bound exceeded"
        );
        use PhaseGitAction as A;
        let marker = self.launch.marker().original_plan();
        let project = marker.project().0;
        let unit = self.launch.allocation().unit_snapshot();
        let (common, mut roots): (PathBuf, Vec<String>) =
            serde_json::from_str(&project.repository_identity)?;
        roots.sort();
        match action {
            A::SourceTop => {
                exact_path(stdout, &project.root)?;
            }
            A::SourceGitDir | A::SourceCommon | A::TaskCommon => {
                exact_path(stdout, &common)?;
            }
            A::TaskTop => {
                exact_path(stdout, &unit.worktree)?;
            }
            A::SourceRoots => {
                let mut actual = oid_lines(stdout, self.revision().len())?;
                actual.sort();
                ensure!(actual == roots, "original repository roots differ");
            }
            A::Branch => ensure!(
                line(stdout)?
                    == unit
                        .branch
                        .as_deref()
                        .context("original Task branch absent")?,
                "original Task branch differs"
            ),
            A::Head => ensure!(
                line(stdout)? == self.revision(),
                "exact original HEAD differs"
            ),
            A::Config => qualify_config(stdout, exit, self.revision().len())?,
            A::IndexEntries => qualify_index(stdout, self.source.inventory(), self.source.tree())?,
            A::ConversionAttrs => qualify_pathset(stdout, self.source.inventory())?,
            A::IndexTree | A::Status => {
                ensure!(stdout.is_empty(), "Git tree/status expectation differs")
            }
        }
        Ok(())
    }
}
fn line(stdout: &[u8]) -> Result<&str> {
    let value = std::str::from_utf8(stdout)?
        .strip_suffix('\n')
        .context("Git one-line frame absent")?;
    ensure!(
        !value.is_empty() && !value.contains(['\n', '\r', '\0']),
        "Git one-line frame differs"
    );
    Ok(value)
}
fn exact_path(stdout: &[u8], expected: &Path) -> Result<PathBuf> {
    let path = PathBuf::from(line(stdout)?);
    ensure!(
        path == expected && path.is_absolute() && path.canonicalize()? == expected,
        "Git physical namespace differs"
    );
    Ok(path)
}
fn oid_lines(stdout: &[u8], length: usize) -> Result<Vec<String>> {
    let text = std::str::from_utf8(stdout)?;
    ensure!(text.ends_with('\n'), "Git OID frame absent");
    let values: Vec<_> = text
        .strip_suffix('\n')
        .expect("checked LF")
        .split('\n')
        .collect();
    ensure!(
        !values.is_empty()
            && values
                .iter()
                .all(|v| v.len() == length && super::super::valid_oid(v)),
        "Git OID set differs"
    );
    Ok(values.into_iter().map(str::to_owned).collect())
}
fn records(stdout: &[u8]) -> Result<impl Iterator<Item = &[u8]>> {
    ensure!(
        stdout.is_empty() || stdout.last() == Some(&0),
        "Git NUL framing absent"
    );
    Ok(stdout
        .strip_suffix(&[0])
        .unwrap_or(stdout)
        .split(|b| *b == 0)
        .filter(|r| !stdout.is_empty() || !r.is_empty()))
}
fn boolean(value: Option<&str>) -> Result<bool> {
    match value.map(str::to_ascii_lowercase).as_deref() {
        None | Some("true" | "yes" | "on" | "1") => Ok(true),
        Some("false" | "no" | "off" | "0" | "") => Ok(false),
        _ => anyhow::bail!("Git boolean spelling unparsed"),
    }
}
pub(super) fn qualify_config(stdout: &[u8], exit: Option<i32>, oid_length: usize) -> Result<()> {
    ensure!(
        stdout.len() <= super::version::OUTPUT_BYTES && matches!(exit, Some(0 | 1)),
        "Git config bound/exit differs"
    );
    if exit == Some(1) {
        ensure!(stdout.is_empty(), "Git config no-key exit has output");
    }
    let mut format_seen = false;
    for record in records(stdout)? {
        let record = std::str::from_utf8(record)?;
        let (key, value) = record
            .split_once('\n')
            .map_or((record, None), |(k, v)| (k, Some(v)));
        match key.to_ascii_lowercase().as_str() {
            "core.bare" | "core.sparsecheckout" | "index.sparse" | "core.autocrlf" => {
                ensure!(!boolean(value)?, "Git config requires false")
            }
            "core.filemode" => ensure!(boolean(value)?, "Git filemode requires true"),
            "core.worktree" => anyhow::bail!("Git core.worktree must be absent"),
            "extensions.objectformat" => {
                format_seen = true;
                ensure!(
                    matches!(
                        (value, oid_length),
                        (Some("sha1"), 40) | (Some("sha256"), 64)
                    ),
                    "Git object format differs"
                );
            }
            _ => anyhow::bail!("Git config key outside finite query"),
        }
    }
    if !format_seen {
        ensure!(oid_length == 40, "absent Git format requires SHA1");
    }
    Ok(())
}
fn qualify_pathset(
    stdout: &[u8],
    inventory: &BTreeMap<String, crate::context::committed::InventoryEntry>,
) -> Result<()> {
    ensure!(
        stdout.len() <= GIT_BYTES,
        "Git attribute capture exceeds bound"
    );
    let mut paths = BTreeSet::new();
    for path in records(stdout)? {
        ensure!(
            paths.len() < 4096 && !path.is_empty() && paths.insert(path),
            "Git attribute path duplicate/bound differs"
        );
    }
    ensure!(
        paths == inventory.keys().map(|p| p.as_bytes()).collect(),
        "Git effective conversion path set differs"
    );
    Ok(())
}
fn qualify_index(
    stdout: &[u8],
    inventory: &BTreeMap<String, crate::context::committed::InventoryEntry>,
    tree: &BTreeMap<String, crate::execution::workflow_source::CommittedTreeEntry>,
) -> Result<()> {
    ensure!(stdout.len() <= GIT_BYTES, "Git index capture exceeds bound");
    let mut paths = BTreeSet::new();
    for record in records(stdout)? {
        let tab = record
            .iter()
            .position(|b| *b == b'\t')
            .context("Git index path separator absent")?;
        let header = std::str::from_utf8(&record[..tab])?;
        let mut tokens = header.split(' ');
        let fields = [
            tokens.next().unwrap_or(""),
            tokens.next().unwrap_or(""),
            tokens.next().unwrap_or(""),
            tokens.next().unwrap_or(""),
        ];
        ensure!(
            tokens.next().is_none()
                && fields[0] == "H"
                && matches!(fields[1], "100644" | "100755")
                && super::super::valid_oid(fields[2])
                && fields[3] == "0",
            "Git index entry mode/flags/stage/OID differs"
        );
        let path = std::str::from_utf8(&record[tab + 1..])?;
        let entry = inventory.get(path).context("Git index extra path")?;
        let original = tree.get(path).context("Source tree path absent")?;
        ensure!(
            entry.oid == fields[2]
                && original.oid == fields[2]
                && original.mode == fields[1]
                && original.kind == "blob"
                && paths.len() < 4096
                && paths.insert(path),
            "Git index exact Source entry differs"
        );
    }
    ensure!(
        paths == inventory.keys().map(String::as_str).collect(),
        "Git index complete path set differs"
    );
    Ok(())
}

/// Only retained actual observations from one seal can construct these facts.
pub(crate) struct ConversionIdentity {
    seal: Arc<NativeGitSourceSeal>,
    _config: Arc<super::version::NativeVersionObservation>,
    _attributes: Arc<super::version::NativeVersionObservation>,
}
impl ConversionIdentity {
    pub(super) fn qualify(
        seal: &Arc<NativeGitSourceSeal>,
        config: Arc<super::version::NativeVersionObservation>,
        attributes: Arc<super::version::NativeVersionObservation>,
    ) -> Result<Arc<Self>> {
        config.qualify_git_for(seal, PhaseGitAction::Config)?;
        attributes.qualify_git_for(seal, PhaseGitAction::ConversionAttrs)?;
        Ok(Arc::new(Self {
            seal: seal.clone(),
            _config: config,
            _attributes: attributes,
        }))
    }
    pub(crate) fn matches_seal(&self, seal: &Arc<NativeGitSourceSeal>) -> bool {
        Arc::ptr_eq(&self.seal, seal)
    }
}
pub(crate) fn require_conversion(
    action: PhaseGitAction,
    seal: &Arc<NativeGitSourceSeal>,
    conversion: Option<&Arc<ConversionIdentity>>,
) -> Result<()> {
    if action == PhaseGitAction::Status {
        ensure!(
            conversion.is_some_and(|c| c.matches_seal(seal)),
            "Status lacks SAME batch conversion identity"
        );
    }
    Ok(())
}
pub(super) struct NativeGitCorrespondence {
    _seal: Arc<NativeGitSourceSeal>,
    _conversion: Arc<ConversionIdentity>,
    _observations: Vec<Arc<super::version::NativeVersionObservation>>,
}
pub(super) fn qualify_correspondence(
    seal: Arc<NativeGitSourceSeal>,
    conversion: Arc<ConversionIdentity>,
    observations: Vec<Arc<super::version::NativeVersionObservation>>,
) -> Result<NativeGitCorrespondence> {
    ensure!(
        conversion.matches_seal(&seal) && observations.len() == GIT_ACTIONS,
        "physical correspondence lacks SAME complete batch"
    );
    for (action, observation) in PhaseGitAction::ORDERED.into_iter().zip(&observations) {
        observation.qualify_git_for(&seal, action)?;
    }
    qualify_ownership(&seal, &observations[..8])?;
    Ok(NativeGitCorrespondence {
        _seal: seal,
        _conversion: conversion,
        _observations: observations,
    })
}
pub(super) fn qualify_ownership(
    seal: &Arc<NativeGitSourceSeal>,
    observations: &[Arc<super::version::NativeVersionObservation>],
) -> Result<()> {
    ensure!(
        observations.len() == 8,
        "Git ownership observations incomplete"
    );
    for (action, o) in PhaseGitAction::ORDERED[..8].iter().zip(observations) {
        o.qualify_git_for(seal, *action)?;
    }
    let marker = seal.launch.marker().original_plan();
    crate::git::validate_worktree_ownership(
        marker.project().0,
        marker.task_after().0,
        crate::git::WorktreeOwnershipFacts {
            source_top: PathBuf::from(line(observations[0].stdout())?),
            source_git_dir: PathBuf::from(line(observations[1].stdout())?),
            source_common: PathBuf::from(line(observations[2].stdout())?),
            source_roots: oid_lines(observations[3].stdout(), seal.revision().len())?,
            task_top: PathBuf::from(line(observations[4].stdout())?),
            task_common: PathBuf::from(line(observations[5].stdout())?),
            branch: line(observations[6].stdout())?.into(),
            revision: line(observations[7].stdout())?.into(),
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
