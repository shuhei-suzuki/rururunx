//! Explicit Runtime/operator admission, never authority derived from Task text.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
};

pub const PROFILE_BYTES: usize = 1024 * 1024;
pub const STREAM_BYTES: usize = 32 * 1024 * 1024;
pub const RUN_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Tests,
    Typecheck,
    Lint,
    Build,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Applicability {
    Required,
    NotApplicable { reason: String },
}

/// Proposed command data. Only ManagedVerifier admission qualifies its actual program.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestCommand {
    pub id: String,
    pub category: Category,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub timeout_seconds: u64,
    pub drain_seconds: u64,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
}

/// An explicit admitted Project catalog; committing this DTO does not activate it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestsProfile {
    pub commands: Vec<TestCommand>,
    pub applicability: BTreeMap<Category, Applicability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AdmittedProfile {
    pub(crate) proposal: TestsProfile,
    pub(crate) programs: BTreeMap<String, String>,
}
pub(crate) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(crate) fn json_hash(value: &impl Serialize) -> Result<String> {
    Ok(hash(&serde_json::to_vec(value)?))
}

impl TestsProfile {
    pub(crate) fn validate(&self) -> Result<()> {
        ensure!(
            !self.commands.is_empty() && self.commands.len() <= 32,
            "verification requires a finite nonempty command plan"
        );
        ensure!(
            self.applicability.len() == 4
                && self.applicability.get(&Category::Tests) == Some(&Applicability::Required),
            "verification requires explicit four-category policy and relevant tests"
        );
        for category in [
            Category::Tests,
            Category::Typecheck,
            Category::Lint,
            Category::Build,
        ] {
            match self
                .applicability
                .get(&category)
                .context("verification applicability missing")?
            {
                Applicability::Required => ensure!(
                    self.commands.iter().any(|c| c.category == category),
                    "required verification command missing"
                ),
                Applicability::NotApplicable { reason } => ensure!(
                    !reason.trim().is_empty()
                        && reason.len() <= 1024
                        && !self.commands.iter().any(|c| c.category == category),
                    "invalid non-applicable verification category"
                ),
            }
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut planned_bytes = 0usize;
        for c in &self.commands {
            ensure!(
                !c.id.is_empty()
                    && c.id.len() <= 128
                    && c.id
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                    && ids.insert(&c.id),
                "invalid duplicate verification command identity"
            );
            ensure!(
                c.program.is_absolute()
                    && c.program.to_str().is_some()
                    && c.args.len() <= 128
                    && c.args.iter().all(|a| !a.contains('\0'))
                    && serde_json::to_vec(&c.args)?.len() <= 32 * 1024,
                "invalid verification argv"
            );
            ensure!(
                !c.cwd.is_absolute()
                    && c.cwd
                        .components()
                        .all(|p| matches!(p, Component::Normal(_) | Component::CurDir)),
                "verification cwd must stay in readonly input"
            );
            ensure!(
                (1..=7200).contains(&c.timeout_seconds)
                    && (1..=10).contains(&c.drain_seconds)
                    && (1..=STREAM_BYTES).contains(&c.stdout_bytes)
                    && (1..=STREAM_BYTES).contains(&c.stderr_bytes),
                "verification limits outside qualified envelope"
            );
            planned_bytes = planned_bytes
                .checked_add(c.stdout_bytes)
                .and_then(|n| n.checked_add(c.stderr_bytes))
                .context("verification plan output overflow")?;
            ensure!(
                !matches!(
                    c.program.file_name().and_then(|s| s.to_str()),
                    Some("sh" | "bash" | "zsh" | "fish" | "dash" | "ksh")
                ),
                "shell evaluation is not a verification command profile"
            );
        }
        ensure!(
            planned_bytes <= RUN_BYTES,
            "verification plan output budgets exceed run maximum"
        );
        ensure!(
            serde_json::to_vec(self)?.len() <= PROFILE_BYTES,
            "verification plan encoding limit"
        );
        Ok(())
    }
}

pub(crate) fn program_hash(program: &Path) -> Result<String> {
    ensure!(
        program.is_absolute() && std::fs::canonicalize(program)? == program && program.is_file(),
        "verification executable must be canonical regular file"
    );
    let metadata = std::fs::metadata(program)?;
    use std::os::unix::fs::PermissionsExt;
    ensure!(
        metadata.permissions().mode() & 0o111 != 0 && metadata.len() <= 512 * 1024 * 1024,
        "verification executable profile unavailable"
    );
    let mut file = File::open(program)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    let mut seen = 0u64;
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        seen = seen
            .checked_add(n as u64)
            .context("program length overflow")?;
        ensure!(
            seen <= 512 * 1024 * 1024,
            "verification executable exceeds bound"
        );
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

impl AdmittedProfile {
    pub(crate) fn admit(mut proposal: TestsProfile) -> Result<Self> {
        proposal.validate()?;
        let mut programs = BTreeMap::new();
        for c in &mut proposal.commands {
            c.program = std::fs::canonicalize(&c.program)?;
            programs.insert(c.id.clone(), program_hash(&c.program)?);
        }
        let admitted = Self { proposal, programs };
        admitted.validate()?;
        Ok(admitted)
    }
    pub(crate) fn validate(&self) -> Result<()> {
        self.proposal.validate()?;
        ensure!(
            self.programs.len() == self.proposal.commands.len()
                && self.proposal.commands.iter().all(|c| self
                    .programs
                    .get(&c.id)
                    .is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))),
            "verification admitted program inventory mismatch"
        );
        Ok(())
    }
    pub(crate) fn recheck(&self, c: &TestCommand) -> Result<()> {
        ensure!(
            self.proposal.commands.iter().any(|p| p == c)
                && self.programs.get(&c.id) == Some(&program_hash(&c.program)?),
            "verification program or plan changed after admission"
        );
        Ok(())
    }
}
