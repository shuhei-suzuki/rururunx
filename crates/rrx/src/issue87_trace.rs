//! TEMPORARY Issue #87 measurement trace (measurement branch only; removed
//! before any fix is proposed). Prints to the capturing test's output.
use std::{sync::OnceLock, time::Instant};
static START: OnceLock<Instant> = OnceLock::new();
/// Measurement-only mutant switch (`ISSUE87_MUTANT=mN`), restored by unsetting it.
pub(crate) fn mutant(name: &str) -> bool {
    std::env::var("ISSUE87_MUTANT").as_deref() == Ok(name)
}
pub(crate) fn mark(label: &str) {
    let t = START.get_or_init(Instant::now).elapsed();
    eprintln!("ISSUE87 t={:.3}s {label}", t.as_secs_f64());
}
/// The service sweep's current stage per state root, so a failing setup wait
/// can name where a blocked sweep is (measurement only).
static STAGES: std::sync::Mutex<
    Option<std::collections::BTreeMap<std::path::PathBuf, (String, Instant)>>,
> = std::sync::Mutex::new(None);
pub(crate) fn stage(root: &std::path::Path, label: &str) {
    let mut stages = STAGES.lock().unwrap_or_else(|e| e.into_inner());
    stages
        .get_or_insert_with(Default::default)
        .insert(root.to_path_buf(), (label.to_owned(), Instant::now()));
}
pub(crate) fn dump_stage(root: &std::path::Path) {
    let stages = STAGES.lock().unwrap_or_else(|e| e.into_inner());
    match stages.as_ref().and_then(|s| s.get(root)) {
        Some((label, at)) => mark(&format!("sweep stage now: {label} for {:?}", at.elapsed())),
        None => mark("sweep stage now: none recorded"),
    }
}
