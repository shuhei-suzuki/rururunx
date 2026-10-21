//! Offline, owner-gated measurements at the production snapshot inventory limit.
use super::*;
use crate::execution::pipeline::{
    PipelineGit,
    review_snapshot::{self, SNAPSHOT_ENTRY_BUDGET},
};
use anyhow::{Result, ensure};
use std::{
    fs,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    time::Instant,
};

async fn object(git: &PipelineGit, args: &[&str], bytes: &[u8], identity: bool) -> Result<String> {
    Ok(
        String::from_utf8(git.invoke(args, bytes, None, identity, 4096).await?)?
            .trim()
            .into(),
    )
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
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let project = root.join("project");
    fs::create_dir(&project).unwrap();
    let git = PipelineGit::new(&project).unwrap();
    let work = root.join("work");
    let path = review_snapshot::snapshot_path(&work, "budget", 1, 1).unwrap();
    let mut summary = json!({"entry_budget":SNAPSHOT_ENTRY_BUDGET,"entry_count":null,
        "preparation":null,"post_death_verification":null});
    let measured: Result<()> = async {
        git.run(&["init", "-b", "main"]).await?;
        let blob = object(
            &git,
            &["hash-object", "-w", "--no-filters", "--stdin"],
            b"x\n",
            false,
        )
        .await?;
        // digest() counts the root directory, every intermediate directory and every
        // file. Two nested directories + the root leave budget - 3 small files.
        let files = SNAPSHOT_ENTRY_BUDGET
            .checked_sub(3)
            .expect("budget needs nested directories");
        let mut rows = Vec::new();
        for i in 0..files {
            rows.extend_from_slice(format!("100644 blob {blob}\tf{i:06}\n").as_bytes());
        }
        let leaf = object(&git, &["mktree"], &rows, false).await?;
        let middle = object(
            &git,
            &["mktree"],
            format!("040000 tree {leaf}\tb\n").as_bytes(),
            false,
        )
        .await?;
        let tree = object(
            &git,
            &["mktree"],
            format!("040000 tree {middle}\ta\n").as_bytes(),
            false,
        )
        .await?;
        let commit = object(&git, &["commit-tree", &tree], b"snapshot budget\n", true).await?;
        git.run(&["update-ref", "refs/heads/main", &commit]).await?;
        // Include tree entries (-t) and root, using a generous measurement-only
        // read bound. The measured production rebuild/verify keep their own limits.
        let entries = git
            .invoke(
                &["ls-tree", "-r", "-t", "-z", &commit],
                &[],
                None,
                false,
                16 * 1024 * 1024,
            )
            .await?;
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
        Ok(())
    }
    .await;
    let cleanup = review_snapshot::remove(&path);
    summary["cleanup"] = json!(if cleanup.is_ok() { "removed" } else { "failed" });
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

fn reason(result: &Result<String>) -> Option<&'static str> {
    result.as_ref().err().map(|error| {
        error
            .chain()
            .find_map(|cause| {
                [
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
