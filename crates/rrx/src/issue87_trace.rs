//! TEMPORARY Issue #87 measurement trace (measurement branch only; removed
//! before any fix is proposed). Prints to the capturing test's output.
use std::{sync::OnceLock, time::Instant};
static START: OnceLock<Instant> = OnceLock::new();
pub(crate) fn mark(label: &str) {
    let t = START.get_or_init(Instant::now).elapsed();
    eprintln!("ISSUE87 t={:.3}s {label}", t.as_secs_f64());
}
