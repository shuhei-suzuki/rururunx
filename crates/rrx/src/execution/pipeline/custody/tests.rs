use super::*;

/// Stdout overflow retains only its bounded prefix and refuses complete capture.
#[test]
fn b4_stdout_bound() {
    let mut capture = Capture::new(4, false);
    assert!(capture.append(b"abc").unwrap());
    assert!(!capture.append(b"def").unwrap());
    assert_eq!(capture.bytes, b"abcd");
    assert!(capture.truncated);
}

/// Stderr overflow preserves the bounded tail without rejecting the process.
#[test]
fn b4_stderr_tail() {
    let mut capture = Capture::new(4, true);
    assert!(capture.append(b"abc").unwrap());
    assert!(capture.append(b"def").unwrap());
    assert_eq!(capture.bytes, b"cdef");
    assert!(capture.truncated);
}

/// Retained evidence is read back only when its full digest and length agree.
#[test]
fn b4_retained_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let mut capture = Capture::new(4, false);
    capture.start(&root.join("pending")).unwrap();
    capture.append(b"abcd").unwrap();
    assert_eq!(std::fs::read(root.join("pending")).unwrap(), b"abcd");
    let evidence = capture.retain(&root, "stdout", true).unwrap();
    assert_eq!(inspect(&root, "stdout", &evidence, 4).unwrap(), b"abcd");
    let mut wrong = evidence;
    wrong.sha256 = "0".repeat(64);
    assert!(inspect(&root, "stdout", &wrong, 4).is_err());
}
