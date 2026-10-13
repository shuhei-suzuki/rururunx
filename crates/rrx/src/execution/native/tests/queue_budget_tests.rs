//! Shared Lines controls use original child pipe bytes and both queue budgets.
//! They neither register a Session nor qualify installed S/P or task custody.
use super::*;
use crate::execution::{process::OwnedProcess, strict_json};

const BUDGET: usize = 8 * 1024 * 1024;
const FRAME: usize = 1024 * 1024;

fn parsed(raw: &[u8]) -> Value {
    strict_json::decode(
        raw,
        strict_json::Limits {
            frame_bytes: FRAME,
            ..strict_json::Limits::CEILINGS
        },
    )
    .expect("SETUP: frame must meet original JSON/frame/node/string/container limits")
}

async fn pipe_lines(raw_frames: &[Vec<u8>]) -> (tempfile::TempDir, OwnedProcess, Lines) {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("bounded-queue-frames.jsonl");
    let bytes = raw_frames.iter().flatten().copied().collect::<Vec<_>>();
    std::fs::write(&input, bytes).unwrap();
    let mut command = Command::new("/usr/bin/python3");
    command
        .args([
            "-I",
            "-S",
            "-c",
            "import sys\nwith open(sys.argv[1], 'rb') as source:\n for line in source:\n  sys.stdout.buffer.write(line)\n  sys.stdout.buffer.flush()\nsys.stdin.buffer.read()\n",
        ])
        .arg(&input)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut process = OwnedProcess::spawn(&mut command).unwrap();
    let stdin = process.child.stdin.take().unwrap();
    let stdout = process.child.stdout.take().unwrap();
    (directory, process, Lines::new(stdin, stdout, FRAME))
}

async fn read(wire: &mut Lines, expected: &[u8]) -> Value {
    let value = tokio::time::timeout(Duration::from_secs(10), wire.line())
        .await
        .expect("SETUP: actual pipe frame absent")
        .expect("SETUP: actual pipe frame rejected");
    assert_eq!(wire.original().unwrap(), expected);
    value
}

#[tokio::test]
async fn encoded_limit_is_retained_for_short_original_exponents() {
    let tokens = vec!["1e15"; 4096].join(",");
    let frame = format!(
        "{{\"type\":\"auth_status\",\"isAuthenticating\":false,\"output\":[],\"extra\":[{tokens}]}}\n"
    )
    .into_bytes();
    let value = parsed(&frame);
    let encoded = serde_json::to_vec(&value).unwrap().len();
    assert!(
        encoded > frame.len(),
        "SETUP: encoded expansion not observed"
    );
    let count = BUDGET / encoded;
    let remaining = BUDGET % encoded;
    let empty = json!({"type":"auth_status","isAuthenticating":false,"output":[],"pad":""});
    let empty_size = serde_json::to_vec(&empty).unwrap().len();
    assert!(
        remaining >= empty_size && count + 2 < 128,
        "SETUP: wrong boundary fixture"
    );
    let mut tail = empty;
    tail["pad"] = json!("x".repeat(remaining - empty_size));
    let mut boundary = serde_json::to_vec(&tail).unwrap();
    assert_eq!(boundary.len(), remaining);
    boundary.push(b'\n');
    parsed(&boundary);
    let overflow =
        b"{\"type\":\"auth_status\",\"isAuthenticating\":false,\"output\":[]}\n".to_vec();
    parsed(&overflow);
    let mut frames = vec![frame.clone(); count];
    frames.push(boundary.clone());
    frames.push(overflow.clone());
    let accepted_raw = count * frame.len() + boundary.len();
    assert!(
        accepted_raw + overflow.len() < BUDGET,
        "SETUP: raw guard masks encoded boundary"
    );
    let (_directory, mut child, mut wire) = pipe_lines(&frames).await;
    for body in &frames[..count + 1] {
        let value = read(&mut wire, body).await;
        wire.queue(value).unwrap();
    }
    let boundary_counts = (
        wire.queued_bytes,
        wire.queued_original_bytes,
        wire.queued.len(),
    );
    let value = read(&mut wire, &overflow).await;
    let overflow_rejected = wire.queue(value).is_err();
    // The entire peer output has been read. Recover the real child before
    // budget assertions, including when an omission mutant accepted overflow.
    let receipt = child.stop_and_reap().await.unwrap();
    assert!(receipt.group_error.is_none());
    println!(
        "M0 QUEUE encoded-boundary child wait returned {:?}",
        receipt.status
    );
    assert_eq!(boundary_counts, (BUDGET, accepted_raw, count + 1));
    assert!(overflow_rejected, "encoded queue overflow accepted");
    assert_eq!(wire.queued_bytes, BUDGET);
    assert_eq!(wire.queued_original_bytes, accepted_raw);
    assert_eq!(wire.original().unwrap(), overflow);

    let first = wire.receive().await.unwrap();
    assert_eq!(wire.original().unwrap(), frame);
    assert_eq!(wire.queued_bytes, BUDGET - encoded);
    assert_eq!(wire.queued_original_bytes, accepted_raw - frame.len());
    let checked = claude_wire::decoder_selfcheck().unwrap();
    let exact = claude_wire::ValidatedClaudeFrame::decode_bytes(wire.original().unwrap(), &checked)
        .unwrap();
    assert_eq!(exact.raw()["extra"].as_array().unwrap().len(), 4096);
    assert!(wire.original().unwrap().windows(4).any(|x| x == b"1e15"));
    wire.queue(first).unwrap();
    assert_eq!(wire.queued_bytes, BUDGET);
    assert_eq!(wire.queued_original_bytes, accepted_raw);
    for _ in 0..count + 1 {
        wire.receive().await.unwrap();
    }
    assert!(wire.queued.is_empty());
    assert_eq!((wire.queued_bytes, wire.queued_original_bytes), (0, 0));
    drop(wire);
}

#[tokio::test]
async fn original_bytes_have_an_independent_limit_and_dequeue_restores_both() {
    let body = b"{\"type\":\"auth_status\",\"isAuthenticating\":false,\"output\":[]}\n";
    let mut frame = vec![b' '; FRAME - body.len()];
    frame.extend_from_slice(body);
    assert_eq!(frame.len(), FRAME);
    let value = parsed(&frame);
    let encoded = serde_json::to_vec(&value).unwrap().len();
    assert!(encoded * 9 < BUDGET);
    let frames = vec![frame.clone(); 9];
    let (_directory, mut child, mut wire) = pipe_lines(&frames).await;
    for body in &frames[..8] {
        let value = read(&mut wire, body).await;
        wire.queue(value).unwrap();
    }
    let boundary_counts = (
        wire.queued_bytes,
        wire.queued_original_bytes,
        wire.queued.len(),
    );
    let value = read(&mut wire, &frame).await;
    let overflow_rejected = wire.queue(value).is_err();
    let receipt = child.stop_and_reap().await.unwrap();
    assert!(receipt.group_error.is_none());
    println!(
        "M0 QUEUE original-boundary child wait returned {:?}",
        receipt.status
    );
    assert_eq!(boundary_counts, (encoded * 8, BUDGET, 8));
    assert!(overflow_rejected, "original queue overflow accepted");
    assert_eq!(wire.queued_original_bytes, BUDGET);
    assert_eq!(wire.queued_bytes, encoded * 8);
    assert_eq!(wire.queued.len(), 8);
    let value = wire.receive().await.unwrap();
    assert_eq!(wire.original().unwrap(), frame);
    assert_eq!(wire.queued_original_bytes, BUDGET - FRAME);
    assert_eq!(wire.queued_bytes, encoded * 7);
    wire.queue(value).unwrap();
    assert_eq!(wire.queued_original_bytes, BUDGET);
    assert_eq!(wire.queued_bytes, encoded * 8);
    for _ in 0..8 {
        wire.receive().await.unwrap();
    }
    assert_eq!((wire.queued_bytes, wire.queued_original_bytes), (0, 0));
    assert!(wire.queued.is_empty());
    drop(wire);
}
