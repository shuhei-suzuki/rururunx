//! Offline, owner-gated measurements at the production snapshot inventory limit.
use super::*;
use crate::execution::pipeline::{
    PipelineGit,
    review_snapshot::{self, SNAPSHOT_ENTRY_BUDGET},
};
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
    process::Stdio,
    time::Instant,
};
use tokio::io::AsyncWriteExt;

// Fixture construction has no production deadline or output cap. Only the
// measured rebuild/verify below use PipelineGit and its production limits.
async fn fixture_git(project: &Path, args: &[&str], bytes: &[u8]) -> Result<Vec<u8>> {
    let mut child = tokio::process::Command::new("/usr/bin/git")
        .arg("--no-replace-objects")
        .arg("-C")
        .arg(project)
        .args(["-c", "core.hooksPath=/dev/null", "-c", "gc.auto=0"])
        .args(args)
        .env_clear()
        .env("HOME", project)
        .env("PATH", "/usr/bin:/bin")
        .env("LANG", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "rrx")
        .env("GIT_COMMITTER_NAME", "rrx")
        .env("GIT_AUTHOR_EMAIL", "rrx@localhost")
        .env("GIT_COMMITTER_EMAIL", "rrx@localhost")
        .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00Z")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let mut stdin = child.stdin.take().context("fixture Git stdin missing")?;
    let write = async {
        stdin.write_all(bytes).await?;
        stdin.shutdown().await?;
        drop(stdin);
        Ok::<_, anyhow::Error>(())
    };
    let wait = async { Ok::<_, anyhow::Error>(child.wait_with_output().await?) };
    let (_, output) = tokio::try_join!(write, wait)?;
    ensure!(output.status.success(), "fixture Git command failed");
    Ok(output.stdout)
}

async fn object(project: &Path, args: &[&str], bytes: &[u8]) -> Result<String> {
    Ok(String::from_utf8(fixture_git(project, args, bytes).await?)?
        .trim()
        .into())
}

/// Exact-limit nested trees measure production preparation and post-death verification offline.
#[tokio::test]
#[ignore = "owner only: creates a snapshot with the full production entry budget"]
async fn snapshot_budget_timing() {
    if std::env::var("RRX_SNAPSHOT_BUDGET_TIMING").as_deref() != Ok("1") {
        return;
    }
    let out = std::path::PathBuf::from(
        std::env::var_os("RRX_SNAPSHOT_BUDGET_OUT").expect("RRX_SNAPSHOT_BUDGET_OUT required"),
    );
    assert!(out.is_absolute(), "absolute timing output path required");
    let mut summary = json!({"entry_budget":SNAPSHOT_ENTRY_BUDGET,"entry_count":null,
        "preparation":null,"post_death_verification":null});
    let mut directory = None;
    let mut snapshot = None;
    let mut failure_reason = "fixture_build_failed";
    let measured: Result<()> = async {
        let dir = directory.insert(tempfile::tempdir()?);
        let root = dir.path().canonicalize()?;
        let project = root.join("project");
        fs::create_dir(&project)?;
        let git = PipelineGit::new(&project)?;
        let work = root.join("work");
        let path = review_snapshot::snapshot_path(&work, "budget", 1, 1)?;
        snapshot = Some(path.clone());
        fixture_git(&project, &["init", "-b", "main"], &[]).await?;
        let blob = object(
            &project,
            &["hash-object", "-w", "--no-filters", "--stdin"],
            b"x\n",
        )
        .await?;
        // digest() counts the root directory, every intermediate directory and every
        // file. Two nested directories + the root leave budget - 3 small files.
        let files = SNAPSHOT_ENTRY_BUDGET
            .checked_sub(3)
            .context("budget needs nested directories")?;
        let mut rows = Vec::new();
        for i in 0..files {
            rows.extend_from_slice(format!("100644 blob {blob}\tf{i:06}\n").as_bytes());
        }
        let leaf = object(&project, &["mktree"], &rows).await?;
        let middle = object(
            &project,
            &["mktree"],
            format!("040000 tree {leaf}\tb\n").as_bytes(),
        )
        .await?;
        let tree = object(
            &project,
            &["mktree"],
            format!("040000 tree {middle}\ta\n").as_bytes(),
        )
        .await?;
        let commit = object(&project, &["commit-tree", &tree], b"snapshot budget\n").await?;
        fixture_git(&project, &["update-ref", "refs/heads/main", &commit], &[]).await?;
        // Include tree entries (-t) and root without production fixture limits.
        let entries = fixture_git(&project, &["ls-tree", "-r", "-t", "-z", &commit], &[]).await?;
        let count = 1 + entries
            .split(|b| *b == 0)
            .filter(|entry| !entry.is_empty())
            .count();
        summary["entry_count"] = json!(count);
        ensure!(
            count == SNAPSHOT_ENTRY_BUDGET,
            "measurement inventory differs from production budget"
        );
        let start = Instant::now();
        let prepared = review_snapshot::rebuild(&git, &work, &path, &commit).await;
        summary["preparation"] = json!({"duration_ms":start.elapsed().as_millis(),
            "state":if prepared.is_ok() {"verified"} else {"failed"},"reason":reason(&prepared)});
        if prepared.is_err() {
            // Still measure the full verifier when the production Git output cap
            // refuses preparation. This synthetic fixture never authorizes a run;
            // preparation remains failed and the test fails after writing evidence.
            review_snapshot::remove(&path)?;
            fs::create_dir_all(path.join("a/b"))?;
            for i in 0..files {
                let file = path.join(format!("a/b/f{i:06}"));
                fs::write(&file, b"x\n")?;
                fs::set_permissions(&file, fs::Permissions::from_mode(0o444))?;
            }
            for directory in [path.join("a/b"), path.join("a"), path.clone()] {
                fs::set_permissions(directory, fs::Permissions::from_mode(0o555))?;
            }
            summary["verification_fixture"] =
                json!("independent_synthetic_tree_after_preparation_failure");
        } else {
            summary["verification_fixture"] = json!("production_preparation");
        }
        failure_reason = "snapshot_measurement_failed";
        // No Agent runs here: this is the same independent verify used after death.
        let start = Instant::now();
        let verified = review_snapshot::verify(&git, &work, &path, &commit).await;
        summary["post_death_verification"] = json!({"duration_ms":start.elapsed().as_millis(),
            "state":if verified.is_ok() {"verified"} else {"failed"},"reason":reason(&verified)});
        ensure!(
            prepared.as_deref().ok() == Some(tree.as_str())
                && verified.as_deref().ok() == Some(tree.as_str()),
            "production snapshot did not verify at entry budget"
        );
        Ok::<_, anyhow::Error>(())
    }
    .await
    .map_err(|error| error.context(failure_reason));
    let cleanup = snapshot
        .as_deref()
        .map(review_snapshot::remove)
        .unwrap_or(Ok(()))
        .context("snapshot_cleanup_failed");
    summary["cleanup"] = json!(if cleanup.is_err() {
        "failed"
    } else if snapshot.is_some() {
        "removed"
    } else {
        "not_needed"
    });
    summary["cleanup_reason"] = json!(reason(&cleanup));
    summary["reason"] = json!(reason(&measured).or_else(|| reason(&cleanup)));
    let passed = measured.is_ok() && cleanup.is_ok();
    summary["state"] = json!(if passed { "verified" } else { "failed" });
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(out)
        .unwrap();
    file.write_all(&serde_json::to_vec_pretty(&summary).unwrap())
        .unwrap();
    file.sync_all().unwrap();
    assert!(
        passed,
        "budget measurement failed; inspect RRX_SNAPSHOT_BUDGET_OUT"
    );
}

fn reason<T>(result: &Result<T>) -> Option<&'static str> {
    result.as_ref().err().map(|error| {
        error
            .chain()
            .find_map(|cause| {
                [
                    "fixture_build_failed",
                    "snapshot_cleanup_failed",
                    "review_snapshot_limit",
                    "review_snapshot_changed",
                    "command output exceeded its bound",
                    "Git combined output exceeds bound",
                    "Git timeout",
                ]
                .into_iter()
                .find(|name| cause.to_string() == *name)
            })
            .unwrap_or("snapshot_measurement_failed")
    })
}
