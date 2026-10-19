use super::*;

/// Last-message evidence records the exact retained bytes for adoption revalidation.
#[test]
fn b5_last_receipt() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("last.md"), b"done\n").unwrap();
    let receipt = check_last(dir.path(), false).unwrap().unwrap();
    assert_eq!(receipt.bytes, 5);
    assert_eq!(
        receipt.sha256,
        crate::execution::pipeline::launcher::hash(b"done\n")
    );
    assert!(receipt.complete);
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(dir.path().join("last.md"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o400
    );
}

/// Last-output sealing refuses hardlinked evidence without chmodding its alias.
#[test]
fn b5_f1_last_hardlink() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target");
    std::fs::write(&target, b"result").unwrap();
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::hard_link(&target, dir.path().join("last.md")).unwrap();
    assert!(check_last(dir.path(), false).is_err());
    assert_eq!(
        std::fs::metadata(target).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

/// Replacing last-output with a symlink after lstat cannot pass the fd open.
#[test]
fn b5_f1_last_open_race() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("last.md");
    let target = dir.path().join("target");
    std::fs::write(&path, b"result").unwrap();
    let meta = std::fs::symlink_metadata(&path).unwrap();
    std::fs::rename(&path, &target).unwrap();
    std::os::unix::fs::symlink(&target, &path).unwrap();
    assert!(open_last(&path, &meta).is_err());
}

/// Last-output chmod seals the opened inode even if the pathname changes.
#[test]
fn b5_f1_last_chmod_race() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("last.md");
    let retained = dir.path().join("retained");
    std::fs::write(&path, b"result").unwrap();
    let file = open_last(&path, &std::fs::symlink_metadata(&path).unwrap()).unwrap();
    std::fs::rename(&path, &retained).unwrap();
    std::fs::write(&path, b"replacement").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    seal_last(file).unwrap();
    assert_eq!(
        std::fs::metadata(retained).unwrap().permissions().mode() & 0o777,
        0o400
    );
    assert_eq!(
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

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

/// Stderr retains the exact bounded tail without growing capture files or memory.
#[test]
fn b4_stderr_tail() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let mut capture = Capture::new(STDERR_BYTES, true);
    capture.start(&root.join("stderr.pending")).unwrap();
    let input: Vec<u8> = (0..5 * STDERR_BYTES + 137)
        .map(|n| (n.wrapping_mul(31).wrapping_add(n / 251) % 256) as u8)
        .collect();
    let (large, rest) = input.split_at(2 * STDERR_BYTES + 19);
    for chunk in std::iter::once(large).chain(rest.chunks(STDERR_BYTES / 3 + 17)) {
        assert!(capture.append(chunk).unwrap());
        assert!(capture.tail_bytes.len() <= STDERR_BYTES);
        assert!(capture.tail_bytes.capacity() <= STDERR_BYTES);
        assert!(capture.bytes.is_empty());
        for file in std::fs::read_dir(&root).unwrap() {
            assert!(file.unwrap().metadata().unwrap().len() <= STDERR_BYTES as u64);
        }
        assert_eq!(
            std::fs::metadata(root.join("stderr.pending"))
                .unwrap()
                .len(),
            0
        );
    }
    let expected = &input[input.len() - STDERR_BYTES..];
    let evidence = capture.retain(&root, "stderr", true).unwrap();
    assert_eq!(evidence.bytes, STDERR_BYTES as u64);
    assert_eq!(evidence.sha256, format!("{:x}", Sha256::digest(expected)));
    assert_eq!(
        inspect(&root, "stderr", &evidence, STDERR_BYTES).unwrap(),
        expected
    );
    assert!(evidence.complete);
    assert!(!root.join("stderr.pending").exists());
    assert!(capture.truncated);
}

/// Both last-output names reject dangling and resolving symlinks.
#[test]
fn b4_last_output_refuses_symlinks() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for confirm in [false, true] {
        let last = root.join(if confirm { "last.json" } else { "last.md" });
        assert!(check_last(root, confirm).is_ok());
        let target = root.join("target");
        symlink(&target, &last).unwrap();
        assert!(check_last(root, confirm).is_err());
        std::fs::write(&target, b"result").unwrap();
        assert!(check_last(root, confirm).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"result");
        std::fs::remove_file(last).unwrap();
        std::fs::remove_file(target).unwrap();
    }
}

/// Private directories require the owner's full 0700 permissions.
#[test]
fn b4_private_directory_exact_mode() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("private");
    super::super::ensure_private_directory(&path).unwrap();
    for mode in [0o500, 0o600, 0o750, 0o770] {
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
        let result = super::super::ensure_private_directory(&path);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(result.is_err(), "accepted mode {mode:o}");
    }
    super::super::ensure_private_directory(&path).unwrap();
}

/// A poisoned Store still wakes the runtime when monitor fallback fails.
#[tokio::test]
async fn b4_monitor_poison_wakes() {
    let store = std::sync::Mutex::new(());
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _store = store.lock().unwrap();
            panic!("inject poisoned monitor fallback");
        }))
        .is_err()
    );
    let wake = Arc::new(Notify::new());
    let result = supervise_monitor(
        tokio::spawn(async { anyhow::bail!("monitor failed") }),
        wake.clone(),
        || {
            let _store = store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            Ok(())
        },
    )
    .await;
    assert!(result.unwrap_err().to_string().contains("state poisoned"));
    assert!(matches!(
        futures_util::poll!(Box::pin(wake.notified())),
        std::task::Poll::Ready(())
    ));
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
