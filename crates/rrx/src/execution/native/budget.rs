//! Fixed first-input headroom. This pure policy grants no execution authority.
use super::*;

#[derive(Clone, Copy)]
pub(crate) struct NativeEffectBudget {
    pub(crate) version: usize,
    pub(crate) git: usize,
    pub(crate) prepared: usize,
}
impl NativeEffectBudget {
    pub(crate) fn check_prepared(self, rows: usize) -> Result<()> {
        ensure!(
            rows <= self.prepared,
            "prepared provider effect reserve exhausted"
        );
        Ok(())
    }
}
pub(crate) fn native_effect_budget(
    provider: &str,
    role: SessionRole,
) -> Result<NativeEffectBudget> {
    let (version, git, prepared) = match (provider, role) {
        ("codex", SessionRole::Executor) => (230, 233, 246),
        ("codex", SessionRole::Reviewer) => (232, 235, 248),
        ("claude", SessionRole::Executor) => (237, 238, 252),
        ("claude", SessionRole::Reviewer) => (237, 238, 252),
        _ => anyhow::bail!("unsupported Native preparation provider or role"),
    };
    Ok(NativeEffectBudget {
        version,
        git,
        prepared,
    })
}
