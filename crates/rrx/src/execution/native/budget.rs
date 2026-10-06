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
        ("codex", SessionRole::Executor) => (234, 235, 248),
        ("codex", SessionRole::Reviewer) => (238, 239, 248),
        ("claude", SessionRole::Executor) => (238, 239, 252),
        ("claude", SessionRole::Reviewer) => (242, 243, 252),
        _ => anyhow::bail!("unsupported Native preparation provider or role"),
    };
    Ok(NativeEffectBudget {
        version,
        git,
        prepared,
    })
}
