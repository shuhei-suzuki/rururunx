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
/// A stall watchdog: if the guard is still alive after `after`, mark it and
/// (macOS, first two stalls per process) sample every thread's stack so a
/// worker blocked in a synchronous call can be named (measurement only).
pub(crate) struct Watch(std::sync::Arc<std::sync::atomic::AtomicBool>);
impl Drop for Watch {
    fn drop(&mut self) {
        self.0.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}
static SAMPLES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub(crate) fn watch(label: String, after: std::time::Duration) -> Watch {
    let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let seen = done.clone();
    let _ = std::thread::Builder::new()
        .name("issue87-watch".into())
        .spawn(move || {
            std::thread::sleep(after);
            if seen.load(std::sync::atomic::Ordering::SeqCst) {
                return;
            }
            mark(&format!("WATCHDOG {label} still pending after {after:?}"));
            if SAMPLES.fetch_add(1, std::sync::atomic::Ordering::SeqCst) >= 2
                || !std::path::Path::new("/usr/bin/sample").exists()
            {
                return;
            }
            let path = std::env::temp_dir().join(format!(
                "issue87-sample-{}-{}.txt",
                std::process::id(),
                SAMPLES.load(std::sync::atomic::Ordering::SeqCst)
            ));
            let status = std::process::Command::new("/usr/bin/sample")
                .arg(std::process::id().to_string())
                .arg("1")
                .arg("-file")
                .arg(&path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            mark(&format!(
                "WATCHDOG sample status={status:?} bytes={} (pending now={})",
                text.len(),
                !seen.load(std::sync::atomic::Ordering::SeqCst)
            ));
            for line in condense_sample(&text) {
                eprintln!("ISSUE87 SAMPLE {line}");
            }
        });
    Watch(done)
}
/// One line per sampled thread: its deepest frame and deepest rrx frame.
fn condense_sample(text: &str) -> Vec<String> {
    let mut out = vec![];
    type Thread = (String, (usize, String), (usize, String));
    fn flush(t: Option<Thread>, out: &mut Vec<String>) {
        if let Some((name, leaf, rrx)) = t {
            out.push(format!("{name} | leaf: {} | rrx: {}", leaf.1, rrx.1));
        }
    }
    let mut thread: Option<Thread> = None;
    for line in text.lines() {
        if line.starts_with("Total number in stack") {
            break;
        }
        let body = line.trim_start_matches(|c: char| " +!:|".contains(c));
        let depth = line.len() - body.len();
        let frame = body.split_once(' ').map_or(body, |(_, rest)| rest).trim();
        let frame: String = frame.chars().take(170).collect();
        if body.contains("Thread_") && depth <= 6 {
            flush(thread.take(), &mut out);
            thread = Some((frame, (0, String::new()), (0, String::new())));
            continue;
        }
        if let Some((_, leaf, rrx)) = thread.as_mut() {
            if depth > leaf.0 {
                *leaf = (depth, frame.clone());
            }
            if frame.contains("rrx::") && depth > rrx.0 {
                *rrx = (depth, frame);
            }
        }
    }
    flush(thread.take(), &mut out);
    out
}
