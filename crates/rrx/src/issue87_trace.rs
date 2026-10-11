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
