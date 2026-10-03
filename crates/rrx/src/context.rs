//! Local lexical repository maps. I/O runs outside the shared Store lock.
//! Estimates are explicit byte-based packing estimates, never provider usage.
use crate::{
    adapter::{InputKind, PreparedInput, SharedStore},
    domain::*,
    git,
};
use anyhow::{Context, Result, bail, ensure};
use rustix::fs::{AtFlags, FileType, Mode, OFlags, open, openat, statat};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

const MAX_FILES: usize = 4096;
const MAX_FILE_BYTES: usize = 256 * 1024;
const MAX_TOTAL_BYTES: usize = 16 * 1024 * 1024;
const MAX_QUERY_BYTES: usize = 8192;
const MAX_REFS: usize = 128;

#[derive(Debug, Clone, Serialize)]
pub struct Freshness {
    pub scope: Scope,
    pub project_version: u64,
    pub goal_version: u64,
    pub task_version: u64,
    pub worktree: PathBuf,
    pub source_root_file_id: String,
    pub worktree_file_id: String,
    pub revision: String,
    pub inventory_hash: String,
    pub source_hashes: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct Symbol {
    pub name: String,
    pub line: usize,
    pub signature: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<String>,
    pub references: BTreeSet<String>,
    pub dependencies: BTreeSet<String>,
    pub changed: bool,
    pub bytes: usize,
    pub lexical_limits_reached: bool,
}
/// Only the engine constructs maps: serialized metadata is not a trusted input.
#[derive(Debug, Clone, Serialize)]
pub struct RepositoryMap {
    freshness: Freshness,
    files: BTreeMap<String, FileEntry>,
    skipped: BTreeMap<String, String>,
    additional_paths: Vec<String>,
    #[serde(skip)]
    contents: BTreeMap<String, String>,
    #[serde(skip)]
    mandatory: BTreeMap<String, String>,
    #[serde(skip)]
    snapshot: Snapshot,
}
impl RepositoryMap {
    pub fn freshness(&self) -> &Freshness {
        &self.freshness
    }
    pub fn files(&self) -> &BTreeMap<String, FileEntry> {
        &self.files
    }
    pub fn skipped(&self) -> &BTreeMap<String, String> {
        &self.skipped
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub estimated_tokens: usize,
    pub bytes: usize,
}
impl Budget {
    fn validate(self) -> Result<()> {
        ensure!(
            self.estimated_tokens > 0
                && self.estimated_tokens <= MAX_TOTAL_BYTES
                && self.bytes > 0
                && self.bytes <= MAX_TOTAL_BYTES,
            "context budget must be positive and bounded to 16 MiB"
        );
        Ok(())
    }
    fn fits(self, payload: &str) -> bool {
        payload.len() <= self.bytes && payload.len() <= self.estimated_tokens
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionRequest {
    pub task_text: String,
    pub changed_files: Vec<String>,
    pub changed_symbols: Vec<String>,
    /// Full content from these indexed paths is mandatory, including ignored evidence.
    pub mandatory_evidence: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expansion {
    File { path: String },
    Symbol { name: String },
    Callers { name: String },
    Callees { name: String },
}
#[derive(Debug, Clone, Serialize)]
pub struct SelectionEvidence {
    pub required: BTreeMap<String, Vec<String>>,
    pub selected: BTreeMap<String, Vec<String>>,
    pub omitted: Vec<String>,
    pub required_bytes: usize,
    pub estimated_tokens: usize,
    pub measured_tokens: Option<u64>,
    pub estimate_method: &'static str,
    pub budget: Budget,
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SelectionOutcome {
    Ready { slice: Box<ContextSlice> },
    NeedsBudget { evidence: SelectionEvidence },
}
#[derive(Debug, Clone, Serialize)]
pub struct ContextSlice {
    freshness: Freshness,
    evidence: SelectionEvidence,
    payload: String,
    #[serde(skip)]
    map: RepositoryMap,
}
impl ContextSlice {
    pub fn freshness(&self) -> &Freshness {
        &self.freshness
    }
    pub fn evidence(&self) -> &SelectionEvidence {
        &self.evidence
    }
    pub fn payload(&self) -> &str {
        &self.payload
    }
    /// Validate current sources and state before constructing native adapter input.
    pub async fn prepared_input(
        &self,
        engine: &RepositoryContext,
        kind: InputKind,
        version: u64,
    ) -> Result<PreparedInput> {
        ensure!(version > 0, "prepared context version must be positive");
        engine.validate(&self.map).await?;
        Ok(PreparedInput {
            scope: self.freshness.scope.clone(),
            kind,
            revision: self.freshness.revision.clone(),
            version,
            source_versions: self.freshness.source_hashes.clone(),
            payload: self.payload.clone(),
        })
    }
}
#[derive(Debug, Clone)]
struct Snapshot {
    project: Project,
    goal: Goal,
    task: Task,
}
impl Snapshot {
    fn versions(&self) -> [u64; 3] {
        [self.project.version, self.goal.version, self.task.version]
    }
    fn read(store: &crate::state::Store, scope: &Scope) -> Result<Self> {
        let project = crate::project::registered_project(store, scope.project_id)?;
        let goal = store
            .goal(
                scope
                    .goal_id
                    .context("repository context requires a Goal")?,
            )?
            .context("unknown Goal")?;
        let task = store
            .task(
                scope
                    .task_id
                    .context("repository context requires a Task")?,
            )?
            .context("unknown Task")?;
        ensure!(
            goal.project_id == project.id && task.scope() == *scope && task.goal_id == goal.id,
            "foreign Project/Goal/Task context"
        );
        ensure!(
            task.worktree.is_some() && task.branch.is_some(),
            "context requires a bound Task worktree"
        );
        Ok(Self {
            project,
            goal,
            task,
        })
    }
    fn recheck(&self, store: &crate::state::Store) -> Result<()> {
        let current = Self::read(store, &self.task.scope())?;
        ensure!(
            current.project.version == self.project.version
                && current.goal.version == self.goal.version
                && current.task.version == self.task.version,
            "Project/Goal/Task changed during context operation"
        );
        Ok(())
    }
}
pub struct RepositoryContext {
    store: SharedStore,
}
impl RepositoryContext {
    pub fn new(store: SharedStore) -> Self {
        Self { store }
    }
    fn snapshot(&self, scope: &Scope) -> Result<Snapshot> {
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
        Snapshot::read(&store, scope)
    }
    /// Additional paths explicitly admit ignored sources/evidence; they remain scoped.
    pub async fn index(
        &self,
        scope: &Scope,
        additional_paths: Vec<String>,
    ) -> Result<RepositoryMap> {
        ensure!(
            additional_paths.len() <= MAX_REFS,
            "too many additional source paths"
        );
        for path in &additional_paths {
            relative(path)?;
        }
        let snapshot = self.snapshot(scope)?;
        let map = build(snapshot, additional_paths).await?;
        // A second scan detects changes during collection, including dirty/ignored contents.
        let confirm = build(map.snapshot.clone(), map.additional_paths.clone()).await?;
        ensure!(
            same_sources(&map, &confirm),
            "repository changed during indexing; retry"
        );
        let mut store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
        map.snapshot.recheck(&store)?;
        store.audit_if_current(scope,map.snapshot.versions(), "context.index.generated", json!({"revision":map.freshness.revision,"inventory_hash":map.freshness.inventory_hash,"source_manifest_hash":manifest_hash(&map),"additional_paths":map.additional_paths,"files":map.files.len(),"skipped":map.skipped.len(),"text_source_bytes":map.files.values().map(|f|f.bytes).sum::<usize>(),"algorithm":"lexical-v1"}))?;
        Ok(map)
    }
    pub async fn validate(&self, map: &RepositoryMap) -> Result<()> {
        let snapshot = self.snapshot(&map.freshness.scope)?;
        ensure!(
            snapshot.project.version == map.snapshot.project.version
                && snapshot.goal.version == map.snapshot.goal.version
                && snapshot.task.version == map.snapshot.task.version,
            "stale context state versions"
        );
        let fresh = build(snapshot, map.additional_paths.clone()).await?;
        ensure!(
            same_sources(map, &fresh),
            "stale repository map: HEAD, inventory, rules or source contents changed"
        );
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
        map.snapshot.recheck(&store)?;
        Ok(())
    }
    pub async fn select(
        &self,
        map: &RepositoryMap,
        request: &SelectionRequest,
        budget: Budget,
    ) -> Result<SelectionOutcome> {
        budget.validate()?;
        validate_request(request)?;
        for path in &request.changed_files {
            ensure!(
                map.freshness
                    .source_hashes
                    .contains_key(&format!("worktree:{path}")),
                "changed file unindexed; rebuild with additional_paths"
            );
        }
        self.validate(map).await?;
        let (mandatory, required) = mandatory_payload(map, &request.mandatory_evidence)?;
        let terms = words(&request.task_text);
        ensure!(
            terms.len() <= 512,
            "task text exceeds 512 distinct ranking terms"
        );
        let ranking_deadline = std::time::Instant::now() + Duration::from_secs(5);
        let mut ranked = vec![];
        for (path, file) in &map.files {
            ensure!(
                std::time::Instant::now() < ranking_deadline,
                "context ranking exceeded time budget"
            );
            let mut score = 0usize;
            let mut reasons = vec![];
            if file.changed || request.changed_files.contains(path) {
                score += 100;
                reasons.push("changed_file".into());
            }
            if file
                .symbols
                .iter()
                .any(|s| request.changed_symbols.contains(&s.name))
            {
                score += 100;
                reasons.push("changed_symbol".into());
            }
            let folded_path = path.to_lowercase();
            let matches = terms
                .iter()
                .filter(|t| folded_path.contains(t.as_str()) || file.references.contains(*t))
                .count();
            if matches > 0 {
                score += matches.min(32) * 10;
                reasons.push("task_text".into());
            }
            if score > 0 {
                ranked.push((score, path.clone(), reasons));
            }
        }
        let roots: BTreeSet<_> = ranked.iter().map(|(_, p, _)| p.clone()).collect();
        let mut neighbors = BTreeSet::new();
        for path in &roots {
            for dep in &map.files[path].dependencies {
                if !roots.contains(dep) {
                    neighbors.insert(dep.clone());
                }
            }
        }
        for (caller, file) in &map.files {
            if !roots.contains(caller) && file.dependencies.iter().any(|p| roots.contains(p)) {
                neighbors.insert(caller.clone());
            }
        }
        ranked.extend(
            neighbors
                .into_iter()
                .map(|p| (5, p, vec!["graph_neighbor".into()])),
        );
        ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        for (path, reason) in &map.skipped {
            if request.changed_files.contains(path) || reason == "tracked/additional source missing"
            {
                ranked.push((100, path.clone(), vec!["changed_or_missing_source".into()]));
            }
        }
        ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut unique = BTreeSet::new();
        ranked.retain(|(_, p, _)| unique.insert(p.clone()) && !required.contains(p));
        let candidates = ranked
            .into_iter()
            .map(|(_, path, reasons)| {
                let key = format!("worktree:{path}");
                let text = if let Some(entry) = map.files.get(&path) {
                    section(
                        "file_map",
                        &key,
                        &map.freshness.source_hashes[&key],
                        serde_json::to_value(entry).expect("serializable map entry"),
                    )
                } else {
                    section(
                        "unavailable_source",
                        &key,
                        &map.freshness.source_hashes[&key],
                        json!(map.skipped[&path]),
                    )
                };
                (path, reasons, text)
            })
            .collect();
        self.pack(
            map,
            mandatory,
            required,
            candidates,
            budget,
            "context.selection",
        )
        .await
    }
    pub async fn expand(
        &self,
        map: &RepositoryMap,
        request: &SelectionRequest,
        expansion: &Expansion,
        budget: Budget,
    ) -> Result<SelectionOutcome> {
        budget.validate()?;
        validate_request(request)?;
        self.validate(map).await?;
        let (mut mandatory, mut required) = mandatory_payload(map, &request.mandatory_evidence)?;
        let paths = expansion_paths(map, expansion)?;
        ensure!(
            paths.len() <= MAX_REFS,
            "expansion exceeds 128 files; request a narrower symbol/file"
        );
        for path in paths {
            if required.insert(path.clone()) {
                let key = format!("worktree:{path}");
                mandatory.push_str(&section(
                    "source",
                    &key,
                    &map.freshness.source_hashes[&key],
                    json!(map.contents[&path]),
                ));
            }
        }
        ensure!(
            mandatory.len() <= MAX_TOTAL_BYTES,
            "expanded mandatory context exceeds 16 MiB"
        );
        self.pack(
            map,
            mandatory,
            required,
            vec![],
            budget,
            "context.expansion",
        )
        .await
    }
    async fn pack(
        &self,
        map: &RepositoryMap,
        mut payload: String,
        required: BTreeSet<String>,
        candidates: Vec<(String, Vec<String>, String)>,
        budget: Budget,
        event: &str,
    ) -> Result<SelectionOutcome> {
        let required_bytes = payload.len();
        let mut selected: BTreeMap<_, _> = required
            .iter()
            .map(|p| {
                (
                    format!("worktree:{p}"),
                    vec![if event == "context.expansion" {
                        "required_expansion_or_evidence".into()
                    } else {
                        "mandatory_evidence".into()
                    }],
                )
            })
            .collect();
        for rule in map.mandatory.keys() {
            selected.insert(rule.clone(), vec!["mandatory_project_rule".into()]);
        }
        selected.insert(
            "context:header".into(),
            vec!["mandatory_goal_task_scope".into()],
        );
        let mut evidence = SelectionEvidence {
            required: selected.clone(),
            selected,
            omitted: vec![],
            required_bytes,
            estimated_tokens: required_bytes,
            measured_tokens: None,
            estimate_method: "utf8_bytes_v1 (estimate, not provider tokens)",
            budget,
        };
        let outcome = if !budget.fits(&payload) {
            evidence.selected.clear();
            evidence.omitted = candidates
                .into_iter()
                .map(|(p, _, _)| format!("worktree:{p}"))
                .collect();
            SelectionOutcome::NeedsBudget {
                evidence: evidence.clone(),
            }
        } else {
            for (path, reasons, text) in candidates {
                if payload.len().saturating_add(text.len())
                    <= budget.bytes.min(budget.estimated_tokens)
                {
                    payload.push_str(&text);
                    evidence
                        .selected
                        .insert(format!("worktree:{path}"), reasons);
                } else {
                    evidence.omitted.push(format!("worktree:{path}"));
                }
            }
            evidence.estimated_tokens = payload.len();
            SelectionOutcome::Ready {
                slice: Box::new(ContextSlice {
                    freshness: map.freshness.clone(),
                    evidence: evidence.clone(),
                    payload,
                    map: map.clone(),
                }),
            }
        };
        // Validate again after ranking/packing, before committing observable evidence.
        self.validate(map).await?;
        let mut store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
        map.snapshot.recheck(&store)?;
        store.audit_if_current(&map.freshness.scope,map.snapshot.versions(),event,json!({"revision":map.freshness.revision,"inventory_hash":map.freshness.inventory_hash,"source_manifest_hash":manifest_hash(map),"ready":matches!(outcome,SelectionOutcome::Ready{..}),"evidence":evidence}))?;
        Ok(outcome)
    }
}

fn same_sources(a: &RepositoryMap, b: &RepositoryMap) -> bool {
    a.freshness.scope == b.freshness.scope
        && a.freshness.worktree == b.freshness.worktree
        && a.freshness.source_root_file_id == b.freshness.source_root_file_id
        && a.freshness.worktree_file_id == b.freshness.worktree_file_id
        && a.freshness.revision == b.freshness.revision
        && a.freshness.inventory_hash == b.freshness.inventory_hash
        && a.freshness.source_hashes == b.freshness.source_hashes
}
async fn git_value(root: &Path, args: &[&str], deadline: tokio::time::Instant) -> Result<String> {
    static PROCESS_UNCERTAIN: OnceLock<Arc<AtomicBool>> = OnceLock::new();
    let executable = bounded_fs(|| {
        crate::adapter::resolve_executable("git")
            .map_err(|e| anyhow::anyhow!("native Git unavailable: {e:?}"))
    })
    .await?;
    let latch = PROCESS_UNCERTAIN
        .get_or_init(|| Arc::new(AtomicBool::new(false)))
        .clone();
    ensure!(
        !latch.load(Ordering::SeqCst),
        "earlier context Git cleanup uncertain; further context Git launches blocked"
    );
    // ProcessGroup marks its own flag true while alive. Share only a separate
    // uncertainty latch; live calls must not block each other's normal work.
    let uncertain = Arc::new(AtomicBool::new(false));
    let _observation = GitObservation {
        pending: uncertain.clone(),
        latch: latch.clone(),
    };
    let observed = crate::adapter::bounded_git_raw(
        &executable,
        root,
        &std::iter::once("--no-optional-locks".to_string())
            .chain(args.iter().map(|s| s.to_string()))
            .collect::<Vec<_>>(),
        git::native_environment(),
        deadline,
        uncertain.clone(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("bounded context Git: {e:?}"));
    let observed = confirm_git_cleanup(
        observed,
        uncertain.load(Ordering::SeqCst),
        latch.load(Ordering::SeqCst),
    )?;
    let value = String::from_utf8(observed).context("Git source metadata is not UTF-8")?;
    // NUL-separated lists must preserve leading whitespace filename bytes.
    Ok(if args.contains(&"-z") {
        value
    } else {
        value.trim_end_matches('\n').to_string()
    })
}
fn confirm_git_cleanup(
    observed: Result<Vec<u8>>,
    uncertain: bool,
    concurrent_uncertainty: bool,
) -> Result<Vec<u8>> {
    if uncertain {
        // Preserve the actual native failure before the process-wide latch blocks
        // subsequent operations. Successful output cannot clear uncertain ownership.
        observed
            .context("native Git process-group cleanup uncertain; context operation blocked")?;
        bail!("native Git process-group cleanup uncertain; context operation blocked");
    }
    if concurrent_uncertainty {
        observed
            .context("concurrent context Git cleanup uncertain; this call's result discarded")?;
        bail!("concurrent context Git cleanup uncertain; this call's result discarded");
    }
    observed
}
struct GitObservation {
    pending: Arc<AtomicBool>,
    latch: Arc<AtomicBool>,
}
impl Drop for GitObservation {
    fn drop(&mut self) {
        if self.pending.load(Ordering::SeqCst) {
            self.latch.store(true, Ordering::SeqCst);
        }
    }
}
async fn ownership(snapshot: &Snapshot, deadline: tokio::time::Instant) -> Result<String> {
    let p = &snapshot.project;
    let worktree = snapshot
        .task
        .worktree
        .as_ref()
        .context("missing worktree")?;
    let source_top = git_value(&p.root, &["rev-parse", "--show-toplevel"], deadline)
        .await?
        .into();
    let source_git_dir = git_value(
        &p.root,
        &["rev-parse", "--path-format=absolute", "--git-dir"],
        deadline,
    )
    .await?
    .into();
    let source_common = git_value(
        &p.root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        deadline,
    )
    .await?
    .into();
    let source_roots = git_value(
        &p.root,
        &[
            "rev-list",
            "--max-parents=0",
            &format!("refs/heads/{}", p.base_branch),
            "--",
        ],
        deadline,
    )
    .await?
    .lines()
    .map(str::to_string)
    .collect();
    let task_top = git_value(worktree, &["rev-parse", "--show-toplevel"], deadline)
        .await?
        .into();
    let task_common = git_value(
        worktree,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        deadline,
    )
    .await?
    .into();
    let branch = git_value(
        worktree,
        &["symbolic-ref", "--quiet", "--short", "HEAD"],
        deadline,
    )
    .await?;
    let revision = git_value(
        worktree,
        &["rev-parse", "--verify", "HEAD^{commit}"],
        deadline,
    )
    .await?;
    let project = p.clone();
    let task = snapshot.task.clone();
    let facts = git::WorktreeOwnershipFacts {
        source_top,
        source_git_dir,
        source_common,
        source_roots,
        task_top,
        task_common,
        branch,
        revision,
    };
    Ok(
        bounded_fs(move || git::validate_worktree_ownership(&project, &task, facts))
            .await?
            .revision,
    )
}
async fn build(snapshot: Snapshot, mut additional_paths: Vec<String>) -> Result<RepositoryMap> {
    additional_paths.sort();
    additional_paths.dedup();
    let open_snapshot = snapshot.clone();
    let readers = bounded_fs(move || {
        Ok((
            ScopedReader::new(
                open_snapshot
                    .task
                    .worktree
                    .as_ref()
                    .context("missing worktree")?,
                &open_snapshot.project,
            )?,
            ScopedReader::new(&open_snapshot.project.root, &open_snapshot.project)?,
        ))
    })
    .await?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let revision = ownership(&snapshot, deadline).await?;
    let worktree = snapshot.task.worktree.clone().context("missing worktree")?;
    let inventory = git_value(
        &worktree,
        &[
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ],
        deadline,
    )
    .await?;
    let changed = git_value(
        &worktree,
        &[
            "diff",
            "--name-only",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "-z",
            "HEAD",
            "--",
        ],
        deadline,
    )
    .await?;
    let untracked = git_value(
        &worktree,
        &["ls-files", "--others", "--exclude-standard", "-z"],
        deadline,
    )
    .await?;
    let mut paths: BTreeSet<String> = inventory
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect();
    paths.extend(additional_paths.iter().cloned());
    ensure!(
        paths.len() <= MAX_FILES,
        "repository index exceeds 4096 files; narrow repository before indexing"
    );
    let changed: BTreeSet<_> = changed
        .split('\0')
        .chain(untracked.split('\0'))
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect();
    paths.extend(changed.iter().cloned());
    ensure!(
        paths.len() <= MAX_FILES,
        "changed source inventory exceeds 4096 files"
    );
    let inventory_hash = hash(serde_json::to_string(&(&paths, &changed))?.as_bytes());
    let scan_snapshot = snapshot.clone();
    let scan_revision = revision.clone();
    let scan = bounded_fs(move || {
        scan(
            scan_snapshot,
            scan_revision,
            paths,
            changed,
            inventory_hash,
            additional_paths,
            readers,
        )
    })
    .await?;
    ensure!(
        ownership(
            &snapshot,
            tokio::time::Instant::now() + Duration::from_secs(5)
        )
        .await?
            == revision,
        "HEAD changed during source scan"
    );
    Ok(scan)
}
fn scan(
    snapshot: Snapshot,
    revision: String,
    paths: BTreeSet<String>,
    changed: BTreeSet<String>,
    inventory_hash: String,
    additional_paths: Vec<String>,
    readers: (ScopedReader, ScopedReader),
) -> Result<RepositoryMap> {
    let worktree = snapshot.task.worktree.clone().context("missing worktree")?;
    let (reader, rules_reader) = readers;
    reader.unchanged()?;
    rules_reader.unchanged()?;
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut files = BTreeMap::new();
    let mut skipped = BTreeMap::new();
    let mut contents = BTreeMap::new();
    let mut mandatory = BTreeMap::new();
    let mut source_hashes = BTreeMap::new();
    let mut total = 0usize;
    let mut file_ids = BTreeMap::new();
    for path in paths {
        ensure!(
            std::time::Instant::now() < deadline,
            "context scan exceeded time budget"
        );
        match reader.read(&path)? {
            Some((bytes, file_id)) => {
                ensure!(
                    file_ids.insert(file_id, path.clone()).is_none(),
                    "duplicate source alias/case-folded path"
                );
                total = total
                    .checked_add(bytes.len())
                    .context("source byte overflow")?;
                ensure!(
                    total <= MAX_TOTAL_BYTES,
                    "repository source scan exceeds 16 MiB"
                );
                source_hashes.insert(format!("worktree:{path}"), hash(&bytes));
                match String::from_utf8(bytes) {
                    Ok(text) if !text.contains('\0') => {
                        let entry = lexical(&path, &text, changed.contains(&path));
                        files.insert(path.clone(), entry);
                        contents.insert(path, text);
                    }
                    _ => {
                        skipped.insert(path, "binary source (hashed; no text slice)".into());
                    }
                }
            }
            None => {
                source_hashes.insert(format!("worktree:{path}"), "missing".into());
                skipped.insert(path, "tracked/additional source missing".into());
            }
        }
    }
    for rule in snapshot
        .project
        .rule_refs
        .iter()
        .chain(snapshot.project.config_ref.iter())
    {
        let path = rule
            .strip_prefix(&snapshot.project.root)
            .context("foreign Project rule reference")?
            .to_str()
            .context("rule path is not UTF-8")?;
        ensure!(
            std::time::Instant::now() < deadline,
            "rule scan exceeded time budget"
        );
        let (bytes, _) = rules_reader
            .read(path)?
            .context("mandatory Project rule missing")?;
        total = total
            .checked_add(bytes.len())
            .context("rule byte overflow")?;
        ensure!(total <= MAX_TOTAL_BYTES, "source/rule scan exceeds 16 MiB");
        source_hashes.insert(
            format!(
                "{}:{path}",
                if snapshot.project.rule_refs.contains(rule) {
                    "rule"
                } else {
                    "config"
                }
            ),
            hash(&bytes),
        );
        let text = String::from_utf8(bytes).context("mandatory rule must be UTF-8")?;
        ensure!(!text.contains('\0'), "mandatory rule cannot be binary");
        if snapshot.project.rule_refs.contains(rule) {
            mandatory.insert(format!("rule:{path}"), text);
        }
    }
    graph(&mut files, deadline)?;
    reader.unchanged()?;
    rules_reader.unchanged()?;
    let map = RepositoryMap {
        freshness: Freshness {
            scope: snapshot.task.scope(),
            project_version: snapshot.project.version,
            goal_version: snapshot.goal.version,
            task_version: snapshot.task.version,
            worktree,
            source_root_file_id: rules_reader.file_id.clone(),
            worktree_file_id: reader.file_id.clone(),
            revision,
            inventory_hash,
            source_hashes,
        },
        files,
        skipped,
        additional_paths,
        contents,
        mandatory,
        snapshot,
    };
    serde_json::to_writer(BoundedSize(0), &map)
        .context("repository map metadata exceeds 32 MiB")?;
    Ok(map)
}
struct BoundedSize(usize);
/// A stalled filesystem job retains its slot even after the caller times out;
/// at most two background filesystem jobs can exist process-wide.
async fn bounded_fs<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T> + Send + 'static,
) -> Result<T> {
    static SLOTS: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
    let slots = SLOTS
        .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(2)))
        .clone();
    let permit = tokio::time::timeout(Duration::from_secs(5), slots.acquire_owned())
        .await
        .context("context filesystem workers busy/timed out")??;
    let job = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        operation()
    });
    tokio::time::timeout(Duration::from_secs(5), job)
        .await
        .context("context filesystem operation timed out (worker reservation retained)")?
        .context("context filesystem worker failed")?
}
impl std::io::Write for BoundedSize {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self.0.saturating_add(bytes.len());
        if self.0 > 32 * 1024 * 1024 {
            return Err(std::io::Error::other("metadata budget exceeded"));
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn manifest_hash(map: &RepositoryMap) -> String {
    hash(
        serde_json::to_string(&map.freshness.source_hashes)
            .expect("serializable hashes")
            .as_bytes(),
    )
}
fn section(kind: &str, path: &str, source_hash: &str, body: serde_json::Value) -> String {
    format!(
        "{}\n",
        serde_json::to_string(
            &json!({"kind":kind,"path":path,"source_hash":source_hash,"body":body})
        )
        .expect("serializable context section")
    )
}
fn relative(value: &str) -> Result<PathBuf> {
    ensure!(
        !value.is_empty()
            && value.len() <= 4096
            && !value.contains('\0')
            && !value.contains('\n')
            && !value.contains('\r'),
        "invalid source path"
    );
    let path = PathBuf::from(value);
    ensure!(
        path.components()
            .all(|c| matches!(c,Component::Normal(n) if n != ".git")),
        "source must be a relative path without parent traversal or Git metadata"
    );
    let normalized: PathBuf = path.components().collect();
    ensure!(
        normalized.as_os_str() == std::ffi::OsStr::new(value),
        "source path must use exact normalized separators/components"
    );
    Ok(path)
}
struct ScopedReader {
    root: PathBuf,
    fd: std::os::fd::OwnedFd,
    forbidden: Vec<PathBuf>,
    file_id: String,
}
impl ScopedReader {
    fn new(root: &Path, project: &Project) -> Result<Self> {
        use std::os::unix::fs::MetadataExt;
        let expected = std::fs::symlink_metadata(root)?;
        ensure!(expected.is_dir(), "context root must be a real directory");
        ensure!(
            root.canonicalize()? == root,
            "source root must be canonical"
        );
        let fd = open(
            root,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )?;
        let file_id = file_id(&fd)?;
        ensure!(
            file_id == format!("{}:{}", expected.dev(), expected.ino()),
            "context root changed while opening"
        );
        let (common, _): (PathBuf, Vec<String>) =
            serde_json::from_str(&project.repository_identity)?;
        let mut forbidden = vec![common];
        // The authorized Task root itself lives below the namespace. Its own
        // descendants remain readable; primary-source refs cannot enter any Task.
        if root == project.root {
            forbidden.push(project.worktree_root.clone());
        }
        Ok(Self {
            root: root.to_path_buf(),
            fd,
            forbidden,
            file_id,
        })
    }
    fn unchanged(&self) -> Result<()> {
        ensure!(
            self.root.canonicalize()? == self.root,
            "source root ancestor changed"
        );
        let fd = open(
            &self.root,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )?;
        ensure!(
            file_id(&fd)? == self.file_id,
            "source root directory identity changed during observation"
        );
        Ok(())
    }
    fn read(&self, name: &str) -> Result<Option<(Vec<u8>, String)>> {
        let path = relative(name)?;
        let absolute = self.root.join(&path);
        ensure!(
            !self.forbidden.iter().any(|p| absolute.starts_with(p)),
            "source intersects worktree namespace or Git common directory"
        );
        let mut dir = openat(
            &self.fd,
            ".",
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )?;
        let components: Vec<_> = path.components().collect();
        for component in &components[..components.len() - 1] {
            dir = match openat(
                &dir,
                component.as_os_str(),
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            ) {
                Ok(fd) => fd,
                Err(rustix::io::Errno::NOENT) => return Ok(None),
                Err(e) => return Err(e).context("unsafe/symlink source ancestor"),
            };
            ensure!(
                statat(&dir, ".git", AtFlags::SYMLINK_NOFOLLOW)
                    .is_err_and(|e| e == rustix::io::Errno::NOENT),
                "nested/foreign Git repository is outside context scope"
            );
        }
        let fd = match openat(
            &dir,
            components.last().context("empty path")?.as_os_str(),
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        ) {
            Ok(fd) => fd,
            Err(rustix::io::Errno::NOENT) => return Ok(None),
            Err(e) => return Err(e).context("unsafe/symlink source"),
        };
        let stat = rustix::fs::fstat(&fd)?;
        ensure!(
            FileType::from_raw_mode(stat.st_mode) == FileType::RegularFile,
            "context source must be a regular file"
        );
        ensure!(
            stat.st_nlink == 1,
            "hard-linked context sources are outside isolated file ownership"
        );
        ensure!(
            stat.st_size >= 0 && stat.st_size as u64 <= MAX_FILE_BYTES as u64,
            "context source exceeds 256 KiB"
        );
        let mut bytes = Vec::new();
        File::from(fd)
            .take((MAX_FILE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() <= MAX_FILE_BYTES,
            "context source grew beyond 256 KiB"
        );
        Ok(Some((bytes, format!("{}:{}", stat.st_dev, stat.st_ino))))
    }
}
fn file_id(fd: &std::os::fd::OwnedFd) -> Result<String> {
    let stat = rustix::fs::fstat(fd)?;
    Ok(format!("{}:{}", stat.st_dev, stat.st_ino))
}
fn words(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| s.len() > 1)
        .map(|s| s.to_lowercase())
        .collect()
}
fn lexical(path: &str, text: &str, changed: bool) -> FileEntry {
    let mut symbols = vec![];
    let mut imports: Vec<String> = vec![];
    let mut limited = false;
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        let tokens: Vec<_> = line
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty())
            .collect();
        if let Some(pos) = tokens.iter().position(|s| {
            matches!(
                *s,
                "fn" | "struct"
                    | "enum"
                    | "trait"
                    | "type"
                    | "class"
                    | "def"
                    | "function"
                    | "interface"
            )
        }) && let Some(name) = tokens.get(pos + 1)
        {
            if name.len() <= 256 && symbols.len() < 64 {
                symbols.push(Symbol {
                    name: (*name).into(),
                    line: index + 1,
                    signature: line.chars().take(240).collect(),
                });
            } else {
                limited = true;
            }
        }
        if tokens
            .first()
            .is_some_and(|s| matches!(*s, "use" | "import" | "from" | "mod" | "require"))
        {
            if imports.len() < 64 {
                imports.push(line.chars().take(240).collect());
            } else {
                limited = true;
            }
        }
    }
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for token in text
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty() && s.len() <= 256)
    {
        *counts.entry(token.to_lowercase()).or_default() += 1;
    }
    let mut references: BTreeSet<String> = imports
        .iter()
        .flat_map(|s| words(s))
        .chain(symbols.iter().map(|s| s.name.to_lowercase()))
        .take(512)
        .collect();
    limited |= counts.len() > 512;
    let mut frequent: Vec<_> = counts.into_iter().collect();
    frequent.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for (name, _) in frequent {
        if references.len() >= 512 {
            break;
        }
        references.insert(name);
    }
    FileEntry {
        path: path.into(),
        symbols,
        imports,
        references,
        dependencies: BTreeSet::new(),
        changed,
        bytes: text.len(),
        lexical_limits_reached: limited,
    }
}
fn graph(files: &mut BTreeMap<String, FileEntry>, deadline: std::time::Instant) -> Result<()> {
    let mut targets: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (path, file) in files.iter() {
        let stem = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        targets.entry(stem).or_default().insert(path.clone());
        for symbol in &file.symbols {
            targets
                .entry(symbol.name.to_lowercase())
                .or_default()
                .insert(path.clone());
        }
    }
    let mut edge_bytes = 0usize;
    let mut visits = 0usize;
    for (path, file) in files.iter_mut() {
        ensure!(
            std::time::Instant::now() < deadline,
            "context graph exceeded time budget"
        );
        if visits >= 100_000 {
            file.lexical_limits_reached = true;
            continue;
        }
        for name in &file.references {
            visits += 1;
            if visits >= 100_000 || file.dependencies.len() >= MAX_REFS {
                file.lexical_limits_reached = true;
                break;
            }
            if let Some(targets) = targets.get(name) {
                for target in targets.iter().take(MAX_REFS + 1) {
                    visits += 1;
                    if visits >= 100_000 || file.dependencies.len() >= MAX_REFS {
                        file.lexical_limits_reached = true;
                        break;
                    }
                    if target != path && !file.dependencies.contains(target) {
                        if file.dependencies.len() < MAX_REFS
                            && edge_bytes.saturating_add(target.len()) <= 2 * 1024 * 1024
                        {
                            file.dependencies.insert(target.clone());
                            edge_bytes += target.len();
                        } else {
                            file.lexical_limits_reached = true;
                        }
                    }
                }
                file.lexical_limits_reached |= targets.len() > MAX_REFS;
            }
        }
    }
    Ok(())
}
fn validate_request(request: &SelectionRequest) -> Result<()> {
    ensure!(
        request.task_text.len() <= MAX_QUERY_BYTES
            && request.changed_files.len() <= MAX_REFS
            && request.changed_symbols.len() <= MAX_REFS
            && request.mandatory_evidence.len() <= MAX_REFS,
        "context request exceeds bounded input limits"
    );
    for path in request
        .changed_files
        .iter()
        .chain(request.mandatory_evidence.iter())
    {
        relative(path)?;
    }
    for symbol in &request.changed_symbols {
        ensure!(
            !symbol.is_empty()
                && symbol.len() <= 256
                && symbol.chars().all(|c| c.is_alphanumeric() || c == '_'),
            "invalid changed symbol"
        );
    }
    Ok(())
}
fn mandatory_payload(
    map: &RepositoryMap,
    evidence: &[String],
) -> Result<(String, BTreeSet<String>)> {
    let p = &map.snapshot;
    let mut payload = format!(
        "{}\n",
        serde_json::to_string(
            &json!({"kind":"context_header","scope":map.freshness.scope,"repository_identity":p.project.repository_identity,"worktree":map.freshness.worktree,"revision":map.freshness.revision,"inventory_hash":map.freshness.inventory_hash,"source_manifest_hash":manifest_hash(map),"project_version":map.freshness.project_version,"goal_version":map.freshness.goal_version,"task_version":map.freshness.task_version,"goal":p.goal.objective,"completion_criteria":p.goal.completion_criteria,"constraints":p.goal.constraints,"non_goals":p.goal.non_goals,"task":p.task.title,"acceptance_criteria":p.task.acceptance_criteria,"workflow":p.task.workflow,"risk":p.task.risk,"estimate_method":"utf8_bytes_v1; not measured provider tokens"})
        )?
    );
    for (path, text) in &map.mandatory {
        payload.push_str(&section(
            "project_rule",
            path,
            &map.freshness.source_hashes[path],
            json!(text),
        ));
    }
    let required: BTreeSet<_> = evidence.iter().cloned().collect();
    for path in &required {
        let text = map.contents.get(path).context(
            "mandatory evidence missing/binary/unindexed; rebuild index with additional_paths",
        )?;
        let key = format!("worktree:{path}");
        payload.push_str(&section(
            "evidence",
            &key,
            &map.freshness.source_hashes[&key],
            json!(text),
        ));
    }
    ensure!(
        payload.len() <= MAX_TOTAL_BYTES,
        "mandatory context exceeds absolute 16 MiB limit"
    );
    Ok((payload, required))
}
fn expansion_paths(map: &RepositoryMap, expansion: &Expansion) -> Result<BTreeSet<String>> {
    if let Expansion::File { path } = expansion {
        relative(path)?;
        ensure!(
            map.contents.contains_key(path),
            "file absent/binary/unindexed; rebuild with additional_paths"
        );
        return Ok(BTreeSet::from([path.clone()]));
    }
    let name = match expansion {
        Expansion::Symbol { name } | Expansion::Callers { name } | Expansion::Callees { name } => {
            name
        }
        _ => unreachable!(),
    };
    ensure!(
        !name.is_empty()
            && name.len() <= 256
            && name.chars().all(|c| c.is_alphanumeric() || c == '_'),
        "invalid expansion symbol"
    );
    let definitions: BTreeSet<_> = map
        .files
        .iter()
        .filter(|(_, f)| f.symbols.iter().any(|s| &s.name == name))
        .map(|(p, _)| p.clone())
        .collect();
    ensure!(
        !definitions.is_empty(),
        "symbol has no indexed lexical definition"
    );
    match expansion {
        Expansion::Symbol { .. } => Ok(definitions),
        Expansion::Callers { .. } => Ok(map
            .files
            .iter()
            .filter(|(_, f)| f.references.contains(&name.to_lowercase()))
            .map(|(p, _)| p.clone())
            .collect()),
        Expansion::Callees { .. } => Ok(definitions
            .iter()
            .flat_map(|p| map.files[p].dependencies.iter().cloned())
            .collect()),
        _ => bail!("unsupported expansion"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn uncertain_git_cleanup_retains_native_failure_and_rejects_successful_output() {
        let failure = anyhow::anyhow!("native process inspection timed out");
        let error = confirm_git_cleanup(Err(failure), true, false).unwrap_err();
        let diagnostic = format!("{error:#}");
        assert!(diagnostic.contains("cleanup uncertain; context operation blocked"));
        assert!(diagnostic.contains("native process inspection timed out"));
        assert!(confirm_git_cleanup(Ok(b"metadata".to_vec()), true, false).is_err());
        let error =
            confirm_git_cleanup(Err(anyhow::anyhow!("Git exit failed")), false, true).unwrap_err();
        let diagnostic = format!("{error:#}");
        assert!(diagnostic.contains("concurrent context Git cleanup uncertain"));
        assert!(diagnostic.contains("this call's result discarded"));
        assert!(diagnostic.contains("Git exit failed"));
        assert!(!diagnostic.contains("native Git process-group cleanup uncertain"));
        assert!(confirm_git_cleanup(Ok(b"metadata".to_vec()), false, true).is_err());
        assert_eq!(
            confirm_git_cleanup(Ok(b"metadata".to_vec()), false, false).unwrap(),
            b"metadata"
        );
        let error =
            confirm_git_cleanup(Err(anyhow::anyhow!("Git exit failed")), false, false).unwrap_err();
        assert_eq!(error.to_string(), "Git exit failed");
    }
    #[test]
    fn dense_graph_has_global_visit_bound_and_imports_survive_reference_limits() {
        let mut files: BTreeMap<_, _> = (0..4096)
            .map(|i| {
                let path = format!("f{i:04}.rs");
                let entry = lexical(&path, "pub fn shared() {}", false);
                (path, entry)
            })
            .collect();
        graph(
            &mut files,
            std::time::Instant::now() + Duration::from_secs(5),
        )
        .unwrap();
        assert!(files.values().map(|f| f.dependencies.len()).sum::<usize>() <= 100_000);
        assert!(files.values().any(|f| f.lexical_limits_reached));
        let noisy = (0..600).map(|i| format!("a{i:04} ")).collect::<String>()
            + "\nuse crate::zzcritical;\npub fn needed() {}";
        let entry = lexical("noisy.rs", &noisy, false);
        assert!(entry.references.contains("zzcritical"));
        assert!(entry.references.contains("needed"));
        assert!(entry.lexical_limits_reached);
        assert!(
            graph(
                &mut files,
                std::time::Instant::now() - Duration::from_secs(1)
            )
            .is_err()
        );
    }
    #[test]
    fn anchored_root_reader_cannot_read_a_replacement_project_directory() {
        let temp = tempfile::tempdir().unwrap();
        let parent = temp.path().canonicalize().unwrap();
        let root = parent.join("own");
        let foreign = parent.join("foreign");
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(&foreign).unwrap();
        std::fs::write(root.join("source"), "OWN").unwrap();
        std::fs::write(foreign.join("source"), "FOREIGN").unwrap();
        let project = Project::new(
            "own".into(),
            root.clone(),
            serde_json::to_string(&(root.join(".git"), vec!["root".to_string()])).unwrap(),
            "main".into(),
        );
        let reader = ScopedReader::new(&root, &project).unwrap();
        std::fs::rename(&root, parent.join("held")).unwrap();
        std::fs::rename(&foreign, &root).unwrap();
        assert!(reader.unchanged().is_err());
        assert_eq!(reader.read("source").unwrap().unwrap().0, b"OWN");
        assert_ne!(
            ScopedReader::new(&root, &project).unwrap().file_id,
            reader.file_id
        );
    }
}

/// Bounded read-only primary-source authority for a Goal, including before Tasks
/// exist or after their worktrees have been safely disposed. Never a launch map.
#[derive(Debug, Clone, Serialize)]
pub struct GoalSourceSnapshot {
    pub scope: Scope,
    pub project_version: u64,
    pub goal_version: u64,
    pub root: PathBuf,
    pub root_file_id: String,
    pub revision: String,
    pub repository_identity: String,
    pub source_hashes: BTreeMap<String, String>,
}
impl RepositoryContext {
    pub async fn goal_sources(&self, scope: &Scope) -> Result<GoalSourceSnapshot> {
        self.goal_sources_with_files(scope, vec![]).await
    }
    pub async fn goal_sources_with_files(
        &self,
        scope: &Scope,
        mut paths: Vec<String>,
    ) -> Result<GoalSourceSnapshot> {
        ensure!(
            paths.len() <= MAX_REFS && paths.iter().all(|p| p.len() <= 4096),
            "too many/oversized Goal artifacts"
        );
        paths.sort();
        paths.dedup();
        for path in &paths {
            relative(path)?;
        }
        ensure!(
            scope.goal_id.is_some() && scope.task_id.is_none(),
            "primary Goal observation requires exact Goal scope"
        );
        let (project, goal) = {
            let store = self
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
            let project = crate::project::registered_project(&store, scope.project_id)?;
            let goal = store
                .goal(scope.goal_id.unwrap())?
                .context("unknown Goal")?;
            ensure!(
                goal.scope() == *scope,
                "foreign Goal primary-source observation"
            );
            (project, goal)
        };
        ensure!(
            project.rule_refs.len() <= MAX_REFS,
            "too many Goal rule references"
        );
        let first = observe_primary(&project, paths.clone()).await?;
        let second = observe_primary(&project, paths.clone()).await?;
        ensure!(
            first.root_file_id == second.root_file_id
                && first.revision == second.revision
                && first.source_hashes == second.source_hashes,
            "primary Project changed during Goal observation"
        );
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("Store poisoned"))?;
        let current = crate::project::registered_project(&store, project.id)?;
        let current_goal = store.goal(goal.id)?.context("Goal disappeared")?;
        ensure!(
            current.version == project.version
                && current_goal.version == goal.version
                && current_goal.scope() == *scope,
            "Project/Goal changed during primary observation"
        );
        Ok(GoalSourceSnapshot {
            scope: scope.clone(),
            project_version: project.version,
            goal_version: goal.version,
            ..first
        })
    }
}
async fn primary_revision(project: &Project, deadline: tokio::time::Instant) -> Result<String> {
    let facts = git::ProjectOwnershipFacts {
        top: git_value(&project.root, &["rev-parse", "--show-toplevel"], deadline)
            .await?
            .into(),
        git_dir: git_value(
            &project.root,
            &["rev-parse", "--path-format=absolute", "--git-dir"],
            deadline,
        )
        .await?
        .into(),
        common: git_value(
            &project.root,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
            deadline,
        )
        .await?
        .into(),
        roots: git_value(
            &project.root,
            &[
                "rev-list",
                "--max-parents=0",
                &format!("refs/heads/{}", project.base_branch),
                "--",
            ],
            deadline,
        )
        .await?
        .lines()
        .map(str::to_owned)
        .collect(),
    };
    let owned = project.clone();
    bounded_fs(move || git::validate_project_ownership(&owned, facts)).await?;
    git_value(
        &project.root,
        &["rev-parse", "--verify", "HEAD^{commit}"],
        deadline,
    )
    .await
}
async fn observe_primary(project: &Project, paths: Vec<String>) -> Result<GoalSourceSnapshot> {
    let opening = project.clone();
    let reader = Arc::new(bounded_fs(move || ScopedReader::new(&opening.root, &opening)).await?);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let revision = primary_revision(project, deadline).await?;
    let scanning = project.clone();
    let source = reader.clone();
    let source_hashes = bounded_fs(move || {
        let mut result = BTreeMap::new();
        let mut total = 0usize;
        for reference in scanning.rule_refs.iter().chain(scanning.config_ref.iter()) {
            let relative = reference
                .strip_prefix(&scanning.root)
                .context("foreign Goal rule/config")?
                .to_str()
                .context("non-UTF8 Goal rule/config")?;
            let (bytes, _) = source.read(relative)?.context("Goal rule/config missing")?;
            total = total
                .checked_add(bytes.len())
                .context("Goal source size overflow")?;
            ensure!(
                total <= MAX_TOTAL_BYTES,
                "Goal rule/config sources exceed limit"
            );
            let text = std::str::from_utf8(&bytes).context("Goal rule/config must be UTF8")?;
            ensure!(!text.contains('\0'), "Goal rule/config cannot be binary");
            result.insert(
                format!(
                    "{}:{relative}",
                    if scanning.rule_refs.contains(reference) {
                        "rule"
                    } else {
                        "config"
                    }
                ),
                hash(&bytes),
            );
        }
        for path in paths {
            let (bytes, _) = source
                .read(&path)?
                .context("Goal authoritative artifact missing")?;
            total = total
                .checked_add(bytes.len())
                .context("Goal source size overflow")?;
            ensure!(
                total <= MAX_TOTAL_BYTES,
                "Goal artifacts exceed source limit"
            );
            let text = std::str::from_utf8(&bytes).context("Goal artifact must be UTF8")?;
            ensure!(!text.contains('\0'), "Goal artifact cannot be binary");
            result.insert(format!("project:{path}"), hash(&bytes));
        }
        source.unchanged()?;
        Ok(result)
    })
    .await?;
    ensure!(
        revision == primary_revision(project, deadline).await?,
        "primary HEAD changed during Goal observation"
    );
    let final_reader = reader.clone();
    bounded_fs(move || final_reader.unchanged()).await?;
    Ok(GoalSourceSnapshot {
        scope: Scope::project(project.id),
        project_version: project.version,
        goal_version: 0,
        root: project.root.clone(),
        root_file_id: reader.file_id.clone(),
        revision,
        repository_identity: project.repository_identity.clone(),
        source_hashes,
    })
}
