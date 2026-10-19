use super::*;

fn discovery(limited: bool, matching_processes: usize) -> DeathDiscovery {
    let mut observation = DeathDiscovery::from(rrx_process_tracker::Discovery {
        processes: Vec::new(),
        coverage: rrx_process_tracker::Coverage {
            environment_unavailable: usize::from(limited),
            ..Default::default()
        },
    });
    observation.matching_processes = matching_processes;
    observation
}

/// Limited observations retry independently; the first complete empty scan wins.
#[tokio::test(start_paused = true)]
async fn b4_discovery_retry_complete() {
    let start = Instant::now();
    let mut scans = [discovery(true, 0), discovery(true, 0), discovery(false, 0)].into_iter();
    let attempts = discover_death(start + DEATH_BOUND, || {
        std::future::ready(Ok(scans.next().expect("unexpected extra scan")))
    })
    .await
    .unwrap();
    assert_eq!(attempts, 3);
    assert!(scans.next().is_none());
    assert!(Instant::now() >= start + DISCOVERY_RETRY_PAUSE * 2);
    let evidence = PipelineDeathEvidence {
        leader_exit_observed: true,
        group_kill_delivered: true,
        leader_reaped: true,
        group_empty: true,
        discovery_complete: true,
        matching_processes: 0,
        discovery_attempts: attempts,
    };
    assert_eq!(
        serde_json::to_value(evidence).unwrap()["discovery_attempts"],
        3
    );
}

/// A match refuses proof immediately, including when that scan is limited.
#[tokio::test(start_paused = true)]
async fn b4_discovery_retry_match() {
    for matching_attempt in 1..=3 {
        for limited in [false, true] {
            let start = Instant::now();
            let mut scans = 0;
            let result = discover_death(start + DEATH_BOUND, || {
                scans += 1;
                assert!(scans <= matching_attempt, "retried after a match");
                std::future::ready(Ok(discovery(
                    scans < matching_attempt || limited,
                    usize::from(scans == matching_attempt),
                )))
            })
            .await;
            assert!(result.is_err());
            assert_eq!(scans, matching_attempt);
            assert!(Instant::now() < start + DEATH_BOUND);
        }
    }
}

/// A limited scan never proves death and cannot start another scan at expiry.
#[tokio::test(start_paused = true)]
async fn b4_discovery_retry_deadline() {
    let start = Instant::now();
    let deadline = start + DEATH_BOUND;
    let mut scans = 0;
    let result = discover_death(deadline, || {
        assert!(Instant::now() < deadline, "scanned past the death bound");
        scans += 1;
        std::future::ready(Ok(discovery(true, 0)))
    })
    .await;
    assert!(result.is_err());
    assert!(scans > 1);
    assert_eq!(Instant::now(), deadline);
    assert!(
        discover_death(deadline, || async { panic!("scanned at expiry") })
            .await
            .is_err()
    );
}

/// A scan finishing at or after expiry cannot extend the shared death bound.
#[tokio::test(start_paused = true)]
async fn b4_discovery_retry_scan_bound() {
    for scan_time in [DEATH_BOUND, DEATH_BOUND * 2] {
        let deadline = Instant::now() + DEATH_BOUND;
        let mut scans = 0;
        let result = discover_death(deadline, || {
            scans += 1;
            async move {
                tokio::time::sleep(scan_time).await;
                Ok(discovery(false, 0))
            }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(scans, 1);
        assert_eq!(Instant::now(), deadline);
    }
}

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
