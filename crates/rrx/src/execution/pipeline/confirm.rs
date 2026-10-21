//! Rule filtering and strict confirmation of an adopted review, never free-form advice.
use super::{
    ReviewFinding, ReviewResult, ReviewSeverity, adoption::read_regular, launcher::hash, review,
    rounds,
};
use crate::{
    execution::{
        strict_json::{self, Limits},
        verification::StreamEvidence,
    },
    state::pipeline::PipelineStep,
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    os::unix::fs::PermissionsExt,
    path::{Component, Path},
};

pub(crate) const CONFIRM_BYTES: usize = 256 * 1024;
const LIMITS: Limits = Limits {
    frame_bytes: CONFIRM_BYTES,
    depth: 6,
    nodes: 1024,
    string_bytes: 8192,
    total_string_bytes: CONFIRM_BYTES,
    object_entries: 16,
    array_entries: 32,
};
pub(crate) const CONFIRM_SCHEMA: &str = r#"{"type":"object","additionalProperties":false,"required":["schema","labels"],"properties":{"schema":{"const":"rrx.confirm.v1"},"labels":{"type":"array","maxItems":32,"items":{"type":"object","additionalProperties":false,"required":["id","label","evidence","reason"],"properties":{"id":{"type":"string","pattern":"^F[0-9]{1,2}$"},"label":{"enum":["confirmed","rejected","unclear"]},"evidence":{"type":"object","additionalProperties":false,"required":["file","line","quote"],"properties":{"file":{"type":"string","maxLength":512},"line":{"type":"integer","minimum":1},"quote":{"type":"string","minLength":1,"maxLength":512}}},"reason":{"type":"string","maxLength":2000}}}}}}"#;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InvalidReason {
    Path,
    MissingFile,
    LineRange,
    UnchangedFile,
    OutsideScope,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InvalidFinding {
    pub id: String,
    pub reason: InvalidReason,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FilteredReview {
    pub blocking: Vec<ReviewFinding>,
    pub invalid: Vec<InvalidFinding>,
    pub non_blocking: Vec<String>,
}

pub(crate) fn normalized(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', '\n', '\r', '\t', '\0'])
        && path
            .split('/')
            .all(|c| !c.is_empty() && c != "." && c != "..")
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

fn file_text(root: &Path, file: &str) -> Result<Option<String>> {
    ensure!(normalized(file), "confirm_path_invalid");
    let path = root.join(file);
    // A missing file is invalid; other I/O failures refuse the operation.
    match std::fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
        Ok(m) if !m.is_file() || m.file_type().is_symlink() => return Ok(None),
        Ok(_) => {}
    }
    ensure!(path.canonicalize()? == path, "review_snapshot_changed");
    let bytes = read_regular(&path, 4 * 1024 * 1024)?;
    Ok(String::from_utf8(bytes).ok())
}

pub(crate) fn filter(
    review: &ReviewResult,
    threshold: ReviewSeverity,
    root: &Path,
    reviewed: &BTreeSet<String>,
    changed: &BTreeSet<String>,
    scopes: &[String],
) -> Result<FilteredReview> {
    let mut filtered = FilteredReview {
        blocking: Vec::new(),
        invalid: Vec::new(),
        non_blocking: Vec::new(),
    };
    for f in &review.findings {
        if !rounds::blocking(f, threshold) {
            filtered.non_blocking.push(f.id.clone());
            continue;
        }
        let reason = if !normalized(&f.file) {
            Some(InvalidReason::Path)
        } else if !reviewed.contains(&f.file) {
            // Filesystem aliases never grant membership in the reviewed commit.
            Some(InvalidReason::MissingFile)
        } else if let Some(text) = file_text(root, &f.file)? {
            let count = text.lines().count();
            let end = f.end_line.unwrap_or(f.line);
            if f.line == 0 || f.line > end || end as usize > count {
                Some(InvalidReason::LineRange)
            } else if !changed.contains(&f.file) {
                Some(InvalidReason::UnchangedFile)
            } else if !scopes
                .iter()
                .any(|scope| super::adoption::glob_match(scope, &f.file))
            {
                Some(InvalidReason::OutsideScope)
            } else {
                None
            }
        } else {
            Some(InvalidReason::MissingFile)
        };
        if let Some(reason) = reason {
            filtered.invalid.push(InvalidFinding {
                id: f.id.clone(),
                reason,
            });
        } else {
            filtered.blocking.push(f.clone());
        }
    }
    review::scan_strings(&serde_json::to_value(&filtered)?)?;
    Ok(filtered)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Label {
    Confirmed,
    Rejected,
    Unclear,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Evidence {
    pub file: String,
    pub line: u32,
    pub quote: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FindingLabel {
    pub id: String,
    pub label: Label,
    pub evidence: Evidence,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfirmResult {
    pub schema: String,
    pub labels: Vec<FindingLabel>,
}
impl ConfirmResult {
    pub(crate) fn validate(&self, findings: &[ReviewFinding]) -> Result<()> {
        ensure!(
            self.schema == "rrx.confirm.v1" && self.labels.len() <= 32,
            "confirm_schema"
        );
        let expected: BTreeSet<_> = findings.iter().map(|f| f.id.as_str()).collect();
        let actual: BTreeSet<_> = self.labels.iter().map(|l| l.id.as_str()).collect();
        ensure!(
            expected == actual && actual.len() == self.labels.len(),
            "confirm_label_ids"
        );
        for l in &self.labels {
            ensure!(
                (1..=512).contains(&l.evidence.quote.chars().count())
                    && l.reason.chars().count() <= 2000
                    && l.evidence.file.chars().count() <= 512
                    && l.evidence.line >= 1,
                "confirm_schema"
            );
        }
        Ok(())
    }
}

fn evidence_matches(e: &Evidence, f: &ReviewFinding, text: &str) -> bool {
    e.file == f.file
        && e.line >= 1
        && e.line as usize <= text.lines().count()
        && e.line >= f.line.saturating_sub(3).max(1)
        && e.line <= f.end_line.unwrap_or(f.line).saturating_add(3)
        && e.quote.chars().filter(|c| !c.is_whitespace()).count() >= 8
        && quote_on_line(text, &e.quote, e.line)
}

fn quote_on_line(text: &str, quote: &str, line: u32) -> bool {
    let start: usize = text
        .split_inclusive('\n')
        .take(line.saturating_sub(1) as usize)
        .map(str::len)
        .sum();
    text[start..]
        .find(quote)
        .is_some_and(|offset| !text[start..start + offset].contains('\n'))
}

pub(crate) fn decode(bytes: &[u8], findings: &[ReviewFinding], snapshot: &Path) -> Result<Value> {
    review::scan_raw(&bytes[..bytes.len().min(CONFIRM_BYTES)], 0)?;
    ensure!(bytes.len() <= CONFIRM_BYTES, "confirm_schema");
    let value = strict_json::decode(bytes, LIMITS).context("confirm_schema")?;
    review::scan_strings(&value)?;
    let mut result: ConfirmResult = serde_json::from_value(value).context("confirm_schema")?;
    result.validate(findings)?;
    for label in &mut result.labels {
        if label.label == Label::Confirmed {
            let finding = findings
                .iter()
                .find(|f| f.id == label.id)
                .context("confirm_label_ids")?;
            let text = file_text(snapshot, &finding.file)?.context("review_snapshot_changed")?;
            if !evidence_matches(&label.evidence, finding, &text) {
                label.label = Label::Unclear;
            }
        }
    }
    let stored = json!({"confirmation": result, "result_sha256": hash(bytes)});
    review::scan_strings(&stored)?;
    ensure!(
        serde_json::to_vec(&stored)?.len() <= 240 * 1024,
        "confirm_storage_limit"
    );
    Ok(stored)
}

pub(crate) fn prompt(nonce: &str, findings: &[ReviewFinding], diff: &[u8]) -> Result<Vec<u8>> {
    rounds::framed_prompt(
        nonce,
        "Check each supplied blocking finding against the read-only target snapshot. Return exactly one label per ID using rrx.confirm.v1. Confirm only with matching source evidence.",
        &serde_json::to_value(findings)?,
        diff,
    )
}

pub(crate) fn output(step: &PipelineStep, attempt: &Path) -> Result<Vec<u8>> {
    ensure!(
        step.body["death_confirmed"] == true && attempt.canonicalize()? == attempt,
        "confirm_output_changed"
    );
    let mut result = Vec::new();
    for (file, key, max) in [
        ("last.json", "last", CONFIRM_BYTES),
        ("stdout.jsonl", "stdout", 16 * 1024 * 1024),
        ("stderr.log", "stderr", 1024 * 1024),
    ] {
        if file == "last.json"
            && (step.body["provider_outcome"] == "limit" || step.body["exit"]["code"] != 0)
            && step.body["exit"][key].is_null()
        {
            continue;
        }
        let receipt: StreamEvidence = serde_json::from_value(step.body["exit"][key].clone())
            .context("confirm_output_changed")?;
        let path = attempt.join(file);
        let bytes = read_regular(&path, max).context("confirm_output_changed")?;
        ensure!(
            receipt.complete
                && receipt.bytes == bytes.len() as u64
                && receipt.sha256 == hash(&bytes)
                && std::fs::symlink_metadata(&path)?.permissions().mode() & 0o777 == 0o400,
            "confirm_output_changed"
        );
        review::scan_all_raw(&bytes)?;
        if file == "last.json" {
            result = bytes;
        }
    }
    review::scan_strings(&step.body)?;
    Ok(result)
}

pub(crate) async fn launch_check(
    owner: &crate::execution::RuntimeOwner,
    config: &crate::config::PipelineConfig,
    cursor: &crate::state::pipeline::PipelineCursor,
    input: &super::StepInput,
) -> Result<()> {
    let ctx = owner
        .store
        .lock()
        .map_err(|_| anyhow::anyhow!("state poisoned"))?
        .pipeline_round_context(owner, cursor)?;
    let nonce = ctx.step.body["confirm_nonce"]
        .as_str()
        .context("review_nonce_invalid")?;
    owner
        .store
        .lock()
        .map_err(|_| anyhow::anyhow!("state poisoned"))?
        .pipeline_round_nonce_check(owner, cursor, "confirm_nonce")?;
    ensure!(
        ctx.step.body["confirm_nonce_verified"].as_str() == Some(nonce),
        "confirm_nonce_unverified"
    );
    ensure!(
        input.work
            == super::review_snapshot::snapshot_path(
                &config.work_root,
                &ctx.run.id,
                ctx.step.seq,
                ctx.step.attempt
            )?,
        "review_snapshot_path"
    );
    ensure!(input.schema == CONFIRM_SCHEMA, "confirm_schema_mismatch");
    let (_, _, filtered) = crate::state::pipeline::round_source_review(&ctx)?;
    let git = super::PipelineGit::new(&ctx.project.root)?;
    let diff = git.diff(&ctx.run.base_commit, &ctx.step.step_base).await?;
    let expected = prompt(nonce, &filtered.blocking, &diff)?;
    ensure!(
        input.prompt == expected && hash(&expected) == ctx.step.input_sha256,
        "confirm_input_changed"
    );
    let tree =
        super::review_snapshot::verify(&git, &config.work_root, &input.work, &ctx.step.step_base)
            .await?;
    ensure!(
        ctx.step.body["snapshot_tree"] == tree,
        "review_snapshot_changed"
    );
    Ok(())
}

pub(crate) fn failure_reason(error: &anyhow::Error) -> &'static str {
    const REASONS: &[&str] = &[
        "review_snapshot_limit",
        "review_snapshot_changed",
        "confirm_schema",
        "confirm_label_ids",
        "confirm_agent_failed",
        "confirm_output_changed",
        "confirm_input_changed",
        "confirm_nonce_unverified",
        "confirm_schema_mismatch",
        "confirm_storage_limit",
        "review_nonce_invalid",
        "review_nonce_collision",
        "review_nonce_reused",
        "review_snapshot_path",
        "review_snapshot_unsupported",
        "review_prompt_too_large",
        "diff_too_large",
        "fix_input_changed",
        "fix_preflight_missing",
    ];
    error
        .chain()
        .find_map(|e| REASONS.iter().copied().find(|r| e.to_string() == *r))
        .unwrap_or("confirm_validation_failed")
}

#[cfg(test)]
mod tests;
