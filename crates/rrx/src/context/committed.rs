//! Pure lexical selection over an immutable, producer-supplied Git corpus.
//! OIDs and byte provenance belong to the registered reader; this index grants
//! neither execution nor result-finalization authority and performs no I/O.
use super::{
    Budget, Expansion, FileEntry, MAX_FILE_BYTES, MAX_FILES, MAX_REFS, MAX_TOTAL_BYTES,
    SelectionEvidence, SelectionRequest, graph, hash, lexical, relative, validate_request, words,
};
use crate::domain::Scope;
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
    time::{Duration, Instant},
};

/// One entry of the complete committed inventory, including unread content.
pub struct CommittedFile {
    pub path: String,
    pub oid: String,
    pub bytes: Option<Vec<u8>>,
    pub skipped: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InventoryEntry {
    pub oid: String,
    pub sha256: Option<String>,
    pub bytes: Option<usize>,
    pub skipped: Option<String>,
}

/// Metadata is factual input to selection, not a deserializable authority token.
pub struct CommittedIndex {
    scope: Scope,
    revision: String,
    source_versions: BTreeMap<String, String>,
    inventory: BTreeMap<String, InventoryEntry>,
    inventory_hash: String,
    files: BTreeMap<String, FileEntry>,
    contents: BTreeMap<String, String>,
    skipped: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CommittedOutcome {
    Ready { slice: Box<CommittedSlice> },
    NeedsBudget { evidence: SelectionEvidence },
}

#[derive(Debug, Serialize)]
pub struct CommittedSlice {
    scope: Scope,
    revision: String,
    source_versions: BTreeMap<String, String>,
    payload: String,
    evidence: SelectionEvidence,
}
impl CommittedSlice {
    pub fn scope(&self) -> &Scope {
        &self.scope
    }
    pub fn revision(&self) -> &str {
        &self.revision
    }
    pub fn source_versions(&self) -> &BTreeMap<String, String> {
        &self.source_versions
    }
    pub fn payload(&self) -> &str {
        &self.payload
    }
    pub fn evidence(&self) -> &SelectionEvidence {
        &self.evidence
    }
}

fn oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn version_bytes(versions: &BTreeMap<String, String>) -> Result<()> {
    ensure!(
        versions.len() <= MAX_FILES + MAX_REFS,
        "too many committed dependencies"
    );
    let mut total = 0usize;
    for (key, value) in versions {
        ensure!(
            !key.is_empty()
                && key.len() <= 8192
                && !value.is_empty()
                && value.len() <= 4096
                && !key.contains('\0')
                && !value.contains('\0'),
            "invalid committed dependency"
        );
        total = total
            .checked_add(key.len() + value.len())
            .context("dependency size overflow")?;
    }
    ensure!(
        total <= MAX_TOTAL_BYTES,
        "committed dependency metadata exceeds 16 MiB"
    );
    Ok(())
}

impl CommittedIndex {
    // The caller is the registered object producer, not public serialized input.
    // Parent Sources integration supplies the production consumer separately.
    pub(crate) fn build(
        scope: Scope,
        revision: String,
        mut source_versions: BTreeMap<String, String>,
        files: Vec<CommittedFile>,
    ) -> Result<Self> {
        ensure!(
            scope.goal_id.is_some() && scope.task_id.is_some(),
            "committed context requires Task scope"
        );
        ensure!(
            oid(&revision),
            "committed revision must be an exact lowercase OID"
        );
        ensure!(
            files.len() <= MAX_FILES,
            "committed inventory exceeds 4096 files"
        );
        version_bytes(&source_versions)?;
        let mut inventory = BTreeMap::new();
        let mut contents = BTreeMap::new();
        let mut skipped = BTreeMap::new();
        let mut maps = BTreeMap::new();
        let mut total = 0usize;
        let mut path_bytes = 0usize;
        for file in files {
            relative(&file.path)?;
            ensure!(
                oid(&file.oid),
                "committed blob must have an exact lowercase OID"
            );
            ensure!(
                !inventory.contains_key(&file.path),
                "duplicate committed inventory path"
            );
            path_bytes += file.path.len();
            ensure!(
                path_bytes <= MAX_TOTAL_BYTES,
                "committed paths exceed 16 MiB"
            );
            let (digest, size, reason) = match (file.bytes, file.skipped) {
                (Some(bytes), None) => {
                    ensure!(
                        bytes.len() <= MAX_FILE_BYTES,
                        "committed file exceeds 256 KiB; declare it skipped"
                    );
                    total = total
                        .checked_add(bytes.len())
                        .context("committed corpus size overflow")?;
                    ensure!(total <= MAX_TOTAL_BYTES, "committed corpus exceeds 16 MiB");
                    let digest = hash(&bytes);
                    let size = bytes.len();
                    match String::from_utf8(bytes) {
                        Ok(text) if !text.contains('\0') => {
                            maps.insert(file.path.clone(), lexical(&file.path, &text, false));
                            contents.insert(file.path.clone(), text);
                            (Some(digest), Some(size), None)
                        }
                        _ => (Some(digest), Some(size), Some("binary".to_owned())),
                    }
                }
                (None, Some(reason)) => {
                    ensure!(
                        !reason.is_empty()
                            && reason.len() <= 1024
                            && !reason.chars().any(char::is_control),
                        "invalid committed skip reason"
                    );
                    (None, None, Some(reason))
                }
                _ => anyhow::bail!("committed file needs bytes or one explicit skip reason"),
            };
            if let Some(reason) = &reason {
                skipped.insert(file.path.clone(), reason.clone());
            }
            let entry = InventoryEntry {
                oid: file.oid,
                sha256: digest,
                bytes: size,
                skipped: reason,
            };
            let key = format!("committed:{}", file.path);
            let value = hash(&serde_json::to_vec(&entry)?);
            if let Some(old) = source_versions.get(&key) {
                ensure!(old == &value, "committed dependency digest conflict");
            }
            source_versions.insert(key, value);
            inventory.insert(file.path, entry);
        }
        graph(&mut maps, Instant::now() + Duration::from_secs(5))?;
        let inventory_hash = digest_json(&inventory)?;
        let key = "context:committed_inventory".to_owned();
        if let Some(old) = source_versions.get(&key) {
            ensure!(
                old == &inventory_hash,
                "committed inventory digest conflict"
            );
        }
        source_versions.insert(key, inventory_hash.clone());
        version_bytes(&source_versions)?;
        Ok(Self {
            scope,
            revision,
            source_versions,
            inventory,
            inventory_hash,
            files: maps,
            contents,
            skipped,
        })
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }
    pub fn revision(&self) -> &str {
        &self.revision
    }
    pub fn source_versions(&self) -> &BTreeMap<String, String> {
        &self.source_versions
    }
    pub fn inventory(&self) -> &BTreeMap<String, InventoryEntry> {
        &self.inventory
    }
    pub fn files(&self) -> &BTreeMap<String, FileEntry> {
        &self.files
    }
    pub fn skipped(&self) -> &BTreeMap<String, String> {
        &self.skipped
    }

    /// Complete rendered-budget packing; required sections are never truncated.
    pub fn select(
        &self,
        scope: &Scope,
        request: &SelectionRequest,
        mandatory: &BTreeMap<String, String>,
        budget: Budget,
    ) -> Result<CommittedOutcome> {
        self.check(scope, request, budget)?;
        let terms = words(&request.task_text);
        ensure!(
            terms.len() <= 512,
            "task text exceeds 512 distinct ranking terms"
        );
        let mut ranked = BTreeMap::<String, (usize, Vec<String>)>::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        for (path, file) in &self.files {
            ensure!(
                Instant::now() < deadline,
                "committed ranking exceeded time budget"
            );
            let mut score = 0;
            let mut reasons = vec![];
            if request.changed_files.contains(path) {
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
            let folded = path.to_lowercase();
            let n = terms
                .iter()
                .filter(|t| folded.contains(t.as_str()) || file.references.contains(*t))
                .count();
            if n > 0 {
                score += n.min(32) * 10;
                reasons.push("task_text".into());
            }
            if score > 0 {
                ranked.insert(path.clone(), (score, reasons));
            }
        }
        let roots: BTreeSet<_> = ranked.keys().cloned().collect();
        for (path, file) in &self.files {
            if roots.contains(path) {
                for dep in &file.dependencies {
                    ranked
                        .entry(dep.clone())
                        .or_insert((5, vec!["graph_neighbor".into()]));
                }
            } else if file.dependencies.iter().any(|p| roots.contains(p)) {
                ranked
                    .entry(path.clone())
                    .or_insert((5, vec!["graph_neighbor".into()]));
            }
        }
        for path in &request.changed_files {
            if self.skipped.contains_key(path) {
                ranked.insert(
                    path.clone(),
                    (100, vec!["changed_unavailable_source".into()]),
                );
            }
        }
        let mut candidates: Vec<_> = ranked.into_iter().map(|(p, (s, r))| (s, p, r)).collect();
        candidates.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        self.pack(request, mandatory, BTreeSet::new(), candidates, budget)
    }

    /// Expansion is required content and may not be silently cut to fit.
    pub fn expand(
        &self,
        scope: &Scope,
        request: &SelectionRequest,
        expansion: &Expansion,
        mandatory: &BTreeMap<String, String>,
        budget: Budget,
    ) -> Result<CommittedOutcome> {
        self.check(scope, request, budget)?;
        let paths = match expansion {
            Expansion::File { path } => {
                relative(path)?;
                ensure!(
                    self.contents.contains_key(path),
                    "requested committed content unavailable: {path} ({})",
                    self.skipped
                        .get(path)
                        .map(String::as_str)
                        .unwrap_or("unindexed")
                );
                BTreeSet::from([path.clone()])
            }
            Expansion::Symbol { name }
            | Expansion::Callers { name }
            | Expansion::Callees { name } => {
                ensure!(
                    !name.is_empty()
                        && name.len() <= 256
                        && name.chars().all(|c| c.is_alphanumeric() || c == '_'),
                    "invalid expansion symbol"
                );
                let definitions: BTreeSet<_> = self
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
                    Expansion::Symbol { .. } => definitions,
                    Expansion::Callers { .. } => self
                        .files
                        .iter()
                        .filter(|(_, f)| f.references.contains(&name.to_lowercase()))
                        .map(|(p, _)| p.clone())
                        .collect(),
                    Expansion::Callees { .. } => definitions
                        .iter()
                        .flat_map(|p| self.files[p].dependencies.iter().cloned())
                        .collect(),
                    _ => unreachable!(),
                }
            }
        };
        ensure!(paths.len() <= MAX_REFS, "expansion exceeds 128 files");
        self.pack(request, mandatory, paths, vec![], budget)
    }

    fn check(&self, scope: &Scope, request: &SelectionRequest, budget: Budget) -> Result<()> {
        ensure!(scope == &self.scope, "foreign committed context scope");
        budget.validate()?;
        validate_request(request)?;
        for path in &request.changed_files {
            ensure!(
                self.inventory.contains_key(path),
                "changed file absent from committed inventory"
            );
        }
        Ok(())
    }

    fn pack(
        &self,
        request: &SelectionRequest,
        mandatory: &BTreeMap<String, String>,
        expansion: BTreeSet<String>,
        candidates: Vec<(usize, String, Vec<String>)>,
        budget: Budget,
    ) -> Result<CommittedOutcome> {
        ensure!(mandatory.len() <= MAX_REFS, "too many mandatory sections");
        let mut total = 0usize;
        for (key, text) in mandatory {
            ensure!(
                !key.is_empty() && key.len() <= 4096 && !key.chars().any(char::is_control),
                "invalid mandatory section key"
            );
            total = total
                .checked_add(key.len() + text.len())
                .context("mandatory size overflow")?;
        }
        ensure!(total <= MAX_TOTAL_BYTES, "mandatory input exceeds 16 MiB");
        let mut frame = Frame::new(budget.bytes.min(budget.estimated_tokens));
        frame.line(&serde_json::json!({"kind":"committed_context_header","scope":self.scope,"revision":self.revision,"inventory_hash":self.inventory_hash,"inventory_files":self.inventory.len(),"readable_files":self.contents.len(),"skipped_files":self.skipped.len(),"skipped_hash":digest_json(&self.skipped)?,"dependencies_hash":digest_json(&self.source_versions)?,"estimate_method":"utf8_bytes_v1; not provider tokens"}))?;
        let mut required = BTreeMap::from([(
            "context:header".to_owned(),
            vec!["mandatory_scope_revision_manifest".into()],
        )]);
        for (key, text) in mandatory {
            frame.line(&serde_json::json!({"kind":"mandatory","path":key,"sha256":hash(text.as_bytes()),"body":text}))?;
            required.insert(
                format!("mandatory:{key}"),
                vec!["mandatory_frame_section".into()],
            );
        }
        let mut paths: BTreeSet<_> = request.mandatory_evidence.iter().cloned().collect();
        paths.extend(expansion);
        ensure!(paths.len() <= 2 * MAX_REFS, "too many required sources");
        for path in &paths {
            let text = self.contents.get(path).with_context(|| {
                format!(
                    "required committed content unavailable: {path} ({})",
                    self.skipped
                        .get(path)
                        .map(String::as_str)
                        .unwrap_or("unindexed")
                )
            })?;
            frame.line(&serde_json::json!({"kind":"committed_source","path":path,"oid":self.inventory[path].oid,"sha256":self.inventory[path].sha256,"body":text}))?;
            required.insert(
                format!("committed:{path}"),
                vec!["required_evidence_or_expansion".into()],
            );
        }
        let mut evidence = SelectionEvidence {
            required: required.clone(),
            selected: required,
            omitted: vec![],
            required_bytes: frame.length,
            estimated_tokens: frame.length,
            measured_tokens: None,
            estimate_method: "utf8_bytes_v1 (estimate, not provider tokens)",
            budget,
        };
        if frame.length > frame.limit {
            evidence.selected.clear();
            evidence.omitted = candidates
                .into_iter()
                .filter(|(_, p, _)| !paths.contains(p))
                .map(|(_, p, _)| format!("committed:{p}"))
                .collect();
            return Ok(CommittedOutcome::NeedsBudget { evidence });
        }
        for (_, path, reasons) in candidates {
            if paths.contains(&path) {
                continue;
            }
            let mut candidate = Frame::new(frame.limit - frame.length);
            if let Some(file) = self.files.get(&path) {
                candidate.line(&serde_json::json!({"kind":"committed_file_map","oid":self.inventory[&path].oid,"sha256":self.inventory[&path].sha256,"body":file}))?;
            } else {
                candidate.line(&serde_json::json!({"kind":"unavailable_source","path":path,"oid":self.inventory[&path].oid,"reason":self.skipped[&path]}))?;
            }
            if candidate.length <= candidate.limit {
                frame.bytes.extend(candidate.bytes);
                frame.length += candidate.length;
                evidence
                    .selected
                    .insert(format!("committed:{path}"), reasons);
            } else {
                evidence.omitted.push(format!("committed:{path}"));
            }
        }
        evidence.estimated_tokens = frame.length;
        Ok(CommittedOutcome::Ready {
            slice: Box::new(CommittedSlice {
                scope: self.scope.clone(),
                revision: self.revision.clone(),
                source_versions: self.source_versions.clone(),
                payload: String::from_utf8(frame.bytes)?,
                evidence,
            }),
        })
    }
}

/// Counts encoded bytes even after a budget miss, without retaining the excess.
struct Frame {
    bytes: Vec<u8>,
    length: usize,
    limit: usize,
}
impl Frame {
    fn new(limit: usize) -> Self {
        Self {
            bytes: vec![],
            length: 0,
            limit,
        }
    }
    fn line(&mut self, value: &impl Serialize) -> Result<()> {
        serde_json::to_writer(&mut *self, value)?;
        self.write_all(b"\n")?;
        Ok(())
    }
}
impl Write for Frame {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.length = self
            .length
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("encoded size overflow"))?;
        if self.length <= self.limit {
            self.bytes.extend_from_slice(bytes);
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn digest_json(value: &impl Serialize) -> Result<String> {
    use sha2::{Digest, Sha256};
    struct HashWriter(Sha256);
    impl Write for HashWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut writer = HashWriter(Sha256::new());
    value.serialize(&mut serde_json::Serializer::new(&mut writer))?;
    Ok(format!("{:x}", writer.0.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{GoalId, ProjectId, TaskId};

    fn scope() -> Scope {
        Scope::task(ProjectId::new(), GoalId::new(), TaskId::new())
    }
    fn file(path: &str, bytes: &[u8]) -> CommittedFile {
        CommittedFile {
            path: path.into(),
            oid: "b".repeat(40),
            bytes: Some(bytes.into()),
            skipped: None,
        }
    }
    fn build_index(scope: Scope, files: Vec<CommittedFile>) -> CommittedIndex {
        CommittedIndex::build(
            scope,
            "a".repeat(40),
            BTreeMap::from([("governing".into(), "pinned".into())]),
            files,
        )
        .unwrap()
    }
    fn budget() -> Budget {
        Budget {
            estimated_tokens: 1024 * 1024,
            bytes: 1024 * 1024,
        }
    }
    fn ready(outcome: CommittedOutcome) -> Box<CommittedSlice> {
        match outcome {
            CommittedOutcome::Ready { slice } => slice,
            _ => panic!("expected ready"),
        }
    }

    #[test]
    fn selection_ranks_actual_symbols_and_neighbors_and_binds_manifest() {
        let scope = scope();
        let index = build_index(
            scope.clone(),
            vec![
                file("src/lib.rs", b"mod service; fn run() { service(); }"),
                file("src/service.rs", b"pub fn service() {}"),
                file("unrelated.txt", b"nothing else"),
            ],
        );
        let request = SelectionRequest {
            task_text: "run".into(),
            ..Default::default()
        };
        let slice = ready(
            index
                .select(&scope, &request, &BTreeMap::new(), budget())
                .unwrap(),
        );
        assert!(
            slice
                .evidence()
                .selected
                .contains_key("committed:src/lib.rs")
        );
        assert_eq!(
            slice.evidence().selected["committed:src/service.rs"],
            vec!["graph_neighbor"]
        );
        assert!(!slice.payload().contains("unrelated.txt"));
        assert_eq!(slice.scope(), &scope);
        assert_eq!(slice.revision(), "a".repeat(40));
        assert_eq!(slice.source_versions(), index.source_versions());
        assert!(
            slice
                .source_versions()
                .contains_key("committed:unrelated.txt")
        );
        assert_eq!(slice.evidence().estimated_tokens, slice.payload().len());
        assert_eq!(slice.evidence().measured_tokens, None);
    }

    #[test]
    fn full_required_frame_fits_exact_encoded_boundary_and_never_truncates() {
        let scope = scope();
        let index = build_index(
            scope.clone(),
            vec![file("evidence.txt", "proof \"\\\n日本語".as_bytes())],
        );
        let required = BTreeMap::from([("rules".into(), "never drop \"\\\n日本語".into())]);
        let request = SelectionRequest {
            mandatory_evidence: vec!["evidence.txt".into()],
            ..Default::default()
        };
        let slice = ready(index.select(&scope, &request, &required, budget()).unwrap());
        let length = slice.payload().len();
        for line in slice.payload().lines() {
            serde_json::from_str::<serde_json::Value>(line).unwrap();
        }
        let sections: Vec<serde_json::Value> = slice
            .payload()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert!(sections.iter().any(|section| section["kind"] == "mandatory"
            && section["path"] == "rules"
            && section["body"] == required["rules"]));
        assert!(
            sections
                .iter()
                .any(|section| section["kind"] == "committed_source"
                    && section["path"] == "evidence.txt"
                    && section["body"] == "proof \"\\\n日本語")
        );
        let exact = Budget {
            bytes: length,
            estimated_tokens: length,
        };
        assert_eq!(
            ready(index.select(&scope, &request, &required, exact).unwrap()).payload(),
            slice.payload()
        );
        for short in [
            Budget {
                bytes: length - 1,
                estimated_tokens: length,
            },
            Budget {
                bytes: length,
                estimated_tokens: length - 1,
            },
        ] {
            let CommittedOutcome::NeedsBudget { evidence } =
                index.select(&scope, &request, &required, short).unwrap()
            else {
                panic!("mandatory frame was truncated or exceeded budget")
            };
            assert!(evidence.required.contains_key("mandatory:rules"));
            assert!(evidence.required.contains_key("committed:evidence.txt"));
            assert_eq!(evidence.required_bytes, length);
            assert!(evidence.selected.is_empty());
        }
    }

    #[test]
    fn scoped_selection_and_expansion_cannot_take_foreign_scope() {
        let scope = scope();
        let index = build_index(scope.clone(), vec![file("answer.rs", b"fn answer() {}")]);
        for foreign in [
            Scope::task(
                ProjectId::new(),
                scope.goal_id.unwrap(),
                scope.task_id.unwrap(),
            ),
            Scope::task(scope.project_id, GoalId::new(), scope.task_id.unwrap()),
            Scope::task(scope.project_id, scope.goal_id.unwrap(), TaskId::new()),
        ] {
            assert!(
                index
                    .select(
                        &foreign,
                        &SelectionRequest::default(),
                        &BTreeMap::new(),
                        budget()
                    )
                    .is_err()
            );
            assert!(
                index
                    .expand(
                        &foreign,
                        &SelectionRequest::default(),
                        &Expansion::File {
                            path: "answer.rs".into()
                        },
                        &BTreeMap::new(),
                        budget()
                    )
                    .is_err()
            );
        }
    }

    #[test]
    fn inventory_keeps_binary_large_and_unavailable_with_required_refusal() {
        let scope = scope();
        let index = build_index(
            scope.clone(),
            vec![
                file("binary.bin", b"\0\xff"),
                CommittedFile {
                    path: "large.txt".into(),
                    oid: "c".repeat(40),
                    bytes: None,
                    skipped: Some("large".into()),
                },
                CommittedFile {
                    path: "missing.txt".into(),
                    oid: "d".repeat(40),
                    bytes: None,
                    skipped: Some("content_unavailable".into()),
                },
            ],
        );
        assert_eq!(index.inventory().len(), 3);
        assert_eq!(index.skipped().len(), 3);
        assert_eq!(
            index.inventory()["binary.bin"].sha256,
            Some(hash(b"\0\xff"))
        );
        let request = SelectionRequest {
            changed_files: vec!["large.txt".into()],
            ..Default::default()
        };
        let slice = ready(
            index
                .select(&scope, &request, &BTreeMap::new(), budget())
                .unwrap(),
        );
        assert!(slice.payload().contains("unavailable_source"));
        assert!(slice.payload().contains("large"));
        assert!(!slice.payload().contains("missing.txt"));
        for path in index.skipped().keys() {
            let mandatory = SelectionRequest {
                mandatory_evidence: vec![path.clone()],
                ..Default::default()
            };
            assert!(
                index
                    .select(&scope, &mandatory, &BTreeMap::new(), budget())
                    .unwrap_err()
                    .to_string()
                    .contains(&index.skipped()[path])
            );
            assert!(
                index
                    .expand(
                        &scope,
                        &SelectionRequest::default(),
                        &Expansion::File { path: path.clone() },
                        &BTreeMap::new(),
                        budget()
                    )
                    .unwrap_err()
                    .to_string()
                    .contains(&index.skipped()[path])
            );
        }
    }

    #[test]
    fn expansion_is_full_required_source_and_bounded_to_128_files() {
        let scope = scope();
        let index = build_index(
            scope.clone(),
            vec![
                file("callee.rs", b"fn callee() {}"),
                file("caller.rs", b"fn caller() { callee(); }"),
            ],
        );
        for expansion in [
            Expansion::Symbol {
                name: "callee".into(),
            },
            Expansion::Callers {
                name: "callee".into(),
            },
            Expansion::Callees {
                name: "caller".into(),
            },
        ] {
            let slice = ready(
                index
                    .expand(
                        &scope,
                        &SelectionRequest::default(),
                        &expansion,
                        &BTreeMap::new(),
                        budget(),
                    )
                    .unwrap(),
            );
            assert!(slice.payload().contains("committed_source"));
            assert!(
                slice
                    .evidence()
                    .selected
                    .contains_key("committed:callee.rs")
            );
            assert!(matches!(
                index
                    .expand(
                        &scope,
                        &SelectionRequest::default(),
                        &expansion,
                        &BTreeMap::new(),
                        Budget {
                            bytes: 1,
                            estimated_tokens: 1
                        }
                    )
                    .unwrap(),
                CommittedOutcome::NeedsBudget { .. }
            ));
        }
        let many = build_index(
            scope.clone(),
            (0..129)
                .map(|n| file(&format!("{n}.rs"), b"fn shared() {}"))
                .collect(),
        );
        assert!(
            many.expand(
                &scope,
                &SelectionRequest::default(),
                &Expansion::Symbol {
                    name: "shared".into()
                },
                &BTreeMap::new(),
                budget()
            )
            .unwrap_err()
            .to_string()
            .contains("128 files")
        );
    }

    #[test]
    fn constructor_rejects_ambiguous_paths_oids_and_unbounded_corpus() {
        let scope = scope();
        for path in ["../a", "/a", "a//b", "./a", ".git/config", "a\nb"] {
            assert!(
                CommittedIndex::build(
                    scope.clone(),
                    "a".repeat(40),
                    BTreeMap::new(),
                    vec![file(path, b"ok")]
                )
                .is_err()
            );
        }
        assert!(
            CommittedIndex::build(
                scope.clone(),
                "a".repeat(40),
                BTreeMap::new(),
                vec![file("a", b"1"), file("a", b"2")]
            )
            .is_err()
        );
        assert!(
            CommittedIndex::build(
                scope.clone(),
                "a".repeat(40),
                BTreeMap::new(),
                vec![file("a", b"same"), file("a", b"same")]
            )
            .is_err()
        );
        let mut bad = file("a", b"1");
        bad.oid = "bad".into();
        assert!(
            CommittedIndex::build(scope.clone(), "a".repeat(40), BTreeMap::new(), vec![bad])
                .is_err()
        );
        assert!(
            CommittedIndex::build(scope.clone(), "HEAD".into(), BTreeMap::new(), vec![]).is_err()
        );
        assert!(
            CommittedIndex::build(
                scope.clone(),
                "a".repeat(40),
                BTreeMap::new(),
                vec![file("a", &vec![0; MAX_FILE_BYTES + 1])]
            )
            .is_err()
        );
        assert!(
            CommittedIndex::build(
                scope.clone(),
                "a".repeat(40),
                BTreeMap::new(),
                (0..65)
                    .map(|n| file(&format!("{n}.txt"), &vec![b'x'; MAX_FILE_BYTES]))
                    .collect()
            )
            .is_err()
        );
        assert!(
            CommittedIndex::build(
                scope.clone(),
                "a".repeat(40),
                BTreeMap::new(),
                (0..MAX_FILES + 1)
                    .map(|n| CommittedFile {
                        path: format!("{n}.txt"),
                        oid: "a".repeat(40),
                        bytes: None,
                        skipped: Some("large".into())
                    })
                    .collect()
            )
            .is_err()
        );
        assert!(
            CommittedIndex::build(
                Scope::project(scope.project_id),
                "a".repeat(40),
                BTreeMap::new(),
                vec![]
            )
            .is_err()
        );
    }

    #[test]
    fn manifest_changes_for_omitted_sources_and_digest_conflicts_refuse() {
        let scope = scope();
        let one = build_index(scope.clone(), vec![file("quiet.txt", b"one")]);
        let two = build_index(scope.clone(), vec![file("quiet.txt", b"two")]);
        let request = SelectionRequest::default();
        let a = ready(
            one.select(&scope, &request, &BTreeMap::new(), budget())
                .unwrap(),
        );
        let b = ready(
            two.select(&scope, &request, &BTreeMap::new(), budget())
                .unwrap(),
        );
        assert_ne!(a.payload(), b.payload());
        assert_ne!(a.source_versions(), b.source_versions());
        assert!(!a.payload().contains("quiet.txt"));
        assert!(
            CommittedIndex::build(
                scope,
                "a".repeat(40),
                BTreeMap::from([("committed:quiet.txt".into(), "forged".into())]),
                vec![file("quiet.txt", b"one")]
            )
            .is_err()
        );
    }
}
