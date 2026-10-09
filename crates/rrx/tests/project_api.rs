//! #81 S3 controls (HOW §3.4, §3.5): `rrx project …` through the control API
//! while a daemon runs, the offline path under the exclusive owner lock, and
//! the typed refusals between them. Every Project row is registered by the
//! public CLI; held Git preflights come from a `git` wrapper first on the
//! daemon's `PATH`, which pauses only inside a repository carrying a marker.
use rrx::{
    cli::client,
    domain::{ProjectId, ProjectState},
    runtime::control::{
        ControlAction, ControlResponse, ProjectOptions, ReconcileMark, UnavailableReason,
    },
};
use serde_json::Value;
use std::{
    fs::OpenOptions,
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{Duration, Instant},
};

fn git(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
fn repository(path: &Path) {
    std::fs::create_dir_all(path).unwrap();
    git(path, &["init", "-b", "main"]);
    git(path, &["config", "user.email", "fixture@example.invalid"]);
    git(path, &["config", "user.name", "Fixture"]);
    git(path, &["config", "commit.gpgsign", "false"]);
    std::fs::write(path.join("rules.md"), "rules").unwrap();
    git(path, &["add", "rules.md"]);
    git(path, &["commit", "-m", "fixture"]);
}

struct Fixture {
    _dir: tempfile::TempDir,
    base: PathBuf,
    state: PathBuf,
    config: PathBuf,
    bin: PathBuf,
    log: PathBuf,
    a: PathBuf,
    b: PathBuf,
    c: PathBuf,
    started: std::cell::Cell<bool>,
}
impl Fixture {
    fn new() -> Self {
        Self::with_state(|base| base.join("state.db"))
    }
    fn with_state(state: impl FnOnce(&Path) -> PathBuf) -> Self {
        let dir = tempfile::Builder::new()
            .prefix("rrx-p-")
            .tempdir_in("/tmp")
            .unwrap();
        let base = dir.path().canonicalize().unwrap();
        let (a, b, c) = (base.join("a"), base.join("b"), base.join("c"));
        for repo in [&a, &b, &c] {
            repository(repo);
        }
        let config = base.join("config.toml");
        std::fs::write(
            &config,
            "[agents.worker]\nprovider='claude'\ncommand=['/usr/bin/false']\n",
        )
        .unwrap();
        let bin = base.join("bin");
        std::fs::create_dir(&bin).unwrap();
        let log = base.join("daemon-git.log");
        let real = String::from_utf8(
            Command::new("sh")
                .args(["-c", "command -v git"])
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();
        // Fails with a secret-looking stderr while `<repo>/.git/rrx-fail`
        // exists. Pauses while `<repo>/.git/rrx-hold` exists, after recording its own
        // pid; with `rrx-escape` it first starts a descendant in a new
        // session that keeps the output pipes open for a while.
        let wrapper = format!(
            "#!/bin/sh\nhere=$(pwd -P)\necho \"$here $*\" >> '{log}'\n\
             if [ -e \"$here/.git/rrx-fail\" ]; then echo 'fatal: SECRET-TOKEN-81' >&2; exit 128; fi\n\
             if [ -e \"$here/.git/rrx-hold\" ]; then\n\
             echo $$ >> \"$here/.git/rrx-held\"\n\
             if [ -e \"$here/.git/rrx-escape\" ]; then setsid sleep 14 & fi\n\
             while [ -e \"$here/.git/rrx-hold\" ]; do sleep 0.05; done\nfi\n\
             exec '{real}' \"$@\"\n",
            log = log.display(),
            real = real.trim(),
        );
        std::fs::write(bin.join("git"), wrapper).unwrap();
        std::fs::set_permissions(bin.join("git"), std::fs::Permissions::from_mode(0o755)).unwrap();
        Self {
            state: state(&base),
            base,
            config,
            bin,
            log,
            a,
            b,
            c,
            started: std::cell::Cell::new(false),
            _dir: dir,
        }
    }
    fn command(&self, cwd: &Path) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rrx"));
        command
            .current_dir(cwd)
            .arg("--state")
            .arg(&self.state)
            .arg("--config")
            .arg(&self.config);
        command
    }
    fn cli(&self, cwd: &Path, args: &[&str]) -> Output {
        self.command(cwd).args(args).output().unwrap()
    }
    fn ok(&self, cwd: &Path, args: &[&str]) -> String {
        let out = self.cli(cwd, args);
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }
    fn add(&self, repo: &Path) -> ProjectId {
        let line = self.ok(&self.base, &["project", "add", repo.to_str().unwrap()]);
        line.split('\t').next().unwrap().parse().unwrap()
    }
    fn daemon(&self, verb: &str) -> Value {
        let path = format!(
            "{}:{}",
            self.bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let out = self
            .command(&self.base)
            .env("PATH", path)
            .args(["daemon", verb, "--json"])
            .output()
            .unwrap();
        serde_json::from_slice(&out.stdout).unwrap_or_else(|_| {
            panic!(
                "daemon {verb}: {}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )
        })
    }
    fn start(&self) {
        let started = self.daemon("start");
        assert_eq!(started["start"], "running", "{started}");
        self.started.set(true);
    }
    fn api(&self, action: ControlAction) -> ControlResponse {
        api(&self.state, action)
    }
    fn hold(&self, repo: &Path) {
        std::fs::write(repo.join(".git/rrx-hold"), "").unwrap();
    }
    fn release(&self, repo: &Path) {
        let _ = std::fs::remove_file(repo.join(".git/rrx-hold"));
    }
    fn held(&self, repo: &Path) -> Vec<u32> {
        std::fs::read_to_string(repo.join(".git/rrx-held"))
            .unwrap_or_default()
            .lines()
            .map(|l| l.parse().unwrap())
            .collect()
    }
    fn wait_held(&self, repo: &Path) -> u32 {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(pid) = self.held(repo).first() {
                return *pid;
            }
            assert!(Instant::now() < deadline, "SETUP: preflight never held");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    fn daemon_git(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap_or_default()
    }
    /// The row's version and body, read-only.
    fn row(&self, id: ProjectId) -> Option<(i64, String)> {
        let db = rusqlite::Connection::open_with_flags(
            &self.state,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        db.query_row(
            "SELECT version,body FROM projects WHERE id=?1",
            [id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok()
    }
    fn audit(&self, id: ProjectId) -> i64 {
        let db = rusqlite::Connection::open_with_flags(
            &self.state,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        db.query_row(
            "SELECT count(*) FROM audit WHERE project_id=?1",
            [id.to_string()],
            |r| r.get(0),
        )
        .unwrap()
    }
    fn projects(&self) -> i64 {
        let db = rusqlite::Connection::open_with_flags(
            &self.state,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        db.query_row("SELECT count(*) FROM projects", [], |r| r.get(0))
            .unwrap()
    }
    fn execution(&self) -> PathBuf {
        let mut root = self.state.clone().into_os_string();
        root.push(".execution");
        PathBuf::from(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        for repo in [&self.a, &self.b, &self.c] {
            self.release(repo);
        }
        if self.started.get() {
            let _ = self.daemon("stop");
        }
    }
}
fn api(state: &Path, action: ControlAction) -> ControlResponse {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(client::request(state, action))
        .unwrap()
}
fn reason(response: &ControlResponse) -> Option<UnavailableReason> {
    match response {
        ControlResponse::Unavailable { reason, .. } => Some(*reason),
        _ => None,
    }
}
fn lookup(state: &Path, root: &Path) -> (ProjectId, u64, ProjectState) {
    match api(
        state,
        ControlAction::ProjectLookupRoot {
            root: root.to_path_buf(),
        },
    ) {
        ControlResponse::ProjectLookup { found: Some(found) } => {
            (found.id, found.version, found.state)
        }
        other => panic!("lookup: {other:?}"),
    }
}
fn status_of(state: &Path, id: ProjectId) -> ControlResponse {
    api(
        state,
        ControlAction::ProjectStatus {
            selector: Some(id.to_string()),
            cwd: PathBuf::from("/"),
        },
    )
}
fn renamed(name: &str) -> ProjectOptions {
    ProjectOptions {
        name: Some(name.into()),
        ..Default::default()
    }
}
fn alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}
fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// C-S3a: with a daemon running, add/list/status/remove all go through the
/// API: each one runs Git in the daemon (its `PATH` wrapper logs the root),
/// never in the client. Mutant: the CLI opens the Store directly → the
/// daemon's Git never sees the root → FAIL.
#[test]
fn c_s3a_every_subcommand_goes_through_the_api_while_a_daemon_runs() {
    let f = Fixture::new();
    f.start();
    let a = f.a.to_str().unwrap();
    let mut id = None;
    for step in ["add", "list", "status", "remove"] {
        let id_text = id.map(|id: ProjectId| id.to_string()).unwrap_or_default();
        let step: Vec<&str> = match step {
            "add" => vec!["project", "add", "."],
            "list" => vec!["project", "list"],
            "status" => vec!["project", "status", "--json"],
            _ => vec!["project", "remove", &id_text],
        };
        let before = f.daemon_git().matches(a).count();
        let out = f.ok(&f.a, &step);
        assert!(
            f.daemon_git().matches(a).count() > before,
            "{step:?} did not reach the daemon: {out}"
        );
        id.get_or_insert(lookup(&f.state, &f.a).0);
    }
    let (id, _, state) = lookup(&f.state, &f.a);
    assert_eq!(state, ProjectState::Removed);
    assert!(
        f.ok(&f.base, &["project", "list", "--all"])
            .contains(&id.to_string())
    );
}

/// C-S3b: an offline write is held after `Store::open` (its own Git pauses
/// under the exclusive lock); `daemon start` meanwhile is `owner_busy`.
/// After release the start succeeds and the write is visible exactly once.
#[test]
fn c_s3b_offline_write_holds_the_owner_lock_until_its_last_write() {
    let f = Fixture::new();
    f.hold(&f.a);
    let path = format!(
        "{}:{}",
        f.bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut writer = f
        .command(&f.base)
        .env("PATH", path)
        .args(["project", "add", f.a.to_str().unwrap()])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    f.wait_held(&f.a);
    assert!(f.state.exists(), "SETUP: the Store was not opened yet");
    let busy = f.daemon("start");
    assert_eq!(busy["start"], "owner_busy", "{busy}");
    f.release(&f.a);
    assert!(writer.wait().unwrap().success());
    f.start();
    assert_eq!(f.projects(), 1);
    let (id, _, state) = lookup(&f.state, &f.a);
    assert_eq!(state, ProjectState::Registered);
    assert_eq!(
        f.ok(&f.base, &["project", "list"])
            .matches(&id.to_string())
            .count(),
        1
    );
}

/// C-S3c (M5): clients A and B both observed version v; A updates, then B's
/// update and B's remove are each typed `ProjectCurrencyChanged` with the
/// row and audit unchanged. Mutant: the client sends the latest version
/// instead of the observed one → B's writes land → FAIL.
#[test]
fn c_s3c_a_stale_client_version_is_refused_typed_without_change() {
    let f = Fixture::new();
    f.add(&f.a);
    f.start();
    let (id, v, _) = lookup(&f.state, &f.a);
    let (_, observed_by_b, _) = lookup(&f.state, &f.a);
    assert_eq!(v, observed_by_b);
    match f.api(ControlAction::ProjectUpdate {
        project: id,
        expected_project: v,
        options: renamed("first"),
    }) {
        ControlResponse::ProjectSaved { changed: true, .. } => {}
        other => panic!("A's update: {other:?}"),
    }
    let (row, audit) = (f.row(id), f.audit(id));
    let update = f.api(ControlAction::ProjectUpdate {
        project: id,
        expected_project: observed_by_b,
        options: renamed("second"),
    });
    assert_eq!(
        reason(&update),
        Some(UnavailableReason::ProjectCurrencyChanged),
        "{update:?}"
    );
    let remove = f.api(ControlAction::ProjectRemove {
        project: id,
        expected_project: observed_by_b,
    });
    assert_eq!(
        reason(&remove),
        Some(UnavailableReason::ProjectCurrencyChanged),
        "{remove:?}"
    );
    assert_eq!(f.row(id), row);
    assert_eq!(f.audit(id), audit);
}

/// C-S3d (M6): A's preflight is held; B's status and a `RuntimeStop` still
/// complete within 5 s, and the stop cuts A's preflight off. Mutant: hold
/// the Store lock or `control_admission` across the preflight → FAIL.
#[test]
fn c_s3d_a_held_preflight_never_blocks_other_projects_or_stop() {
    let f = Fixture::new();
    let a = f.add(&f.a);
    let b = f.add(&f.b);
    f.start();
    f.hold(&f.a);
    let state = f.state.clone();
    let held = std::thread::spawn(move || status_of(&state, a));
    let pid = f.wait_held(&f.a);
    let started = Instant::now();
    match status_of(&f.state, b) {
        ControlResponse::ProjectStatusFacts { status, reconcile } => {
            assert_eq!(status.project.id, b);
            assert_eq!(reconcile, ReconcileMark::Checked);
        }
        other => panic!("B's status: {other:?}"),
    }
    assert!(started.elapsed() < Duration::from_secs(5));
    let stopped = f.api(ControlAction::RuntimeStop);
    assert!(
        matches!(
            stopped,
            ControlResponse::RuntimeStopped { .. } | ControlResponse::RuntimeStopPending { .. }
        ),
        "{stopped:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(5));
    let cut = held.join().unwrap();
    assert_eq!(
        reason(&cut),
        Some(UnavailableReason::ProjectPreflightUnavailable),
        "{cut:?}"
    );
    assert!(!alive(pid), "the held Git child outlived the stop");
    assert!(started.elapsed() < Duration::from_secs(5));
    f.started.set(false);
}

/// C-S3d2 (M4): with A's Git held past the deadline and nothing in flight,
/// B's status returns B's facts within 5 s and runs no Git in A. Mutant:
/// status reconciles every Registered Project → waits on A → FAIL.
#[test]
fn c_s3d2_status_reconciles_only_its_own_project() {
    let f = Fixture::new();
    f.add(&f.a);
    let b = f.add(&f.b);
    f.start();
    f.hold(&f.a);
    let started = Instant::now();
    match status_of(&f.state, b) {
        ControlResponse::ProjectStatusFacts { status, reconcile } => {
            assert_eq!(status.project.id, b);
            assert_eq!(reconcile, ReconcileMark::Checked);
        }
        other => panic!("B's status: {other:?}"),
    }
    assert!(started.elapsed() < Duration::from_secs(5));
    assert!(f.held(&f.a).is_empty(), "B's status ran Git in A");
}

/// C-S3d3: A is observed in flight; a List started while A is still held
/// finishes within 5 s with A still in flight: A `skipped_in_flight`, B
/// checked. Mutant: List waits for, or joins, the in-flight preflight →
/// FAIL on the deadline and on the finish order.
#[test]
fn c_s3d3_list_never_waits_for_an_in_flight_preflight() {
    let f = Fixture::new();
    let a = f.add(&f.a);
    let b = f.add(&f.b);
    f.start();
    f.hold(&f.a);
    let state = f.state.clone();
    let held = std::thread::spawn(move || status_of(&state, a));
    f.wait_held(&f.a);
    let started = Instant::now();
    let rows = match f.api(ControlAction::ProjectList { all: false }) {
        ControlResponse::ProjectRows { rows } => rows,
        other => panic!("list: {other:?}"),
    };
    assert!(started.elapsed() < Duration::from_secs(5));
    assert!(!held.is_finished(), "A finished before the List returned");
    let mark = |id| rows.iter().find(|r| r.project.id == id).unwrap().reconcile;
    assert_eq!(mark(a), ReconcileMark::SkippedInFlight);
    assert_eq!(mark(b), ReconcileMark::Checked);
    f.release(&f.a);
    assert!(matches!(
        held.join().unwrap(),
        ControlResponse::ProjectStatusFacts { .. }
    ));
}

/// C-S3d4: nothing in flight; List's own preflight for A runs past the
/// deadline, so A is `unavailable` (not `skipped_in_flight`), B is checked,
/// and A's Git child is reaped before the response arrives, even though a
/// descendant that left its group still holds the child's output pipes.
/// Mutant: reap only after the pipes drain → the child is still a zombie
/// when the response arrives → FAIL.
#[test]
fn c_s3d4_list_own_timeout_is_unavailable_and_reaped_before_response() {
    let f = Fixture::new();
    let a = f.add(&f.a);
    let b = f.add(&f.b);
    f.start();
    std::fs::write(f.a.join(".git/rrx-escape"), "").unwrap();
    f.hold(&f.a);
    let rows = match f.api(ControlAction::ProjectList { all: false }) {
        ControlResponse::ProjectRows { rows } => rows,
        other => panic!("list: {other:?}"),
    };
    let held = f.held(&f.a);
    assert_eq!(held.len(), 1, "SETUP: A's preflight was not held once");
    assert!(
        !alive(held[0]),
        "A's Git child was not reaped before the response"
    );
    let mark = |id| rows.iter().find(|r| r.project.id == id).unwrap().reconcile;
    assert_eq!(mark(a), ReconcileMark::Unavailable);
    assert_ne!(mark(a), ReconcileMark::SkippedInFlight);
    assert_eq!(mark(b), ReconcileMark::Checked);
}

/// C-S3e (M7): the client's CWD is A, the daemon's is B, and the client runs
/// `project add .`: only A is registered. Mutant: no client-side resolution
/// → B, or a refusal → FAIL.
#[test]
fn c_s3e_add_dot_resolves_against_the_client_cwd() {
    let f = Fixture::with_state(|base| base.join("b/state.db"));
    f.start();
    f.ok(&f.a, &["project", "add", "."]);
    let (_, _, state) = lookup(&f.state, &f.a);
    assert_eq!(state, ProjectState::Registered);
    assert_eq!(f.projects(), 1);
    let status: Value =
        serde_json::from_str(&f.ok(&f.a, &["project", "status", "--json"])).unwrap();
    assert_eq!(status["project"]["root"], f.a.to_str().unwrap());
}

/// C-S3e2 (M2): with the client CWD outside the root, `--project-config
/// project.toml` selects `<root>/project.toml` through the API and offline;
/// a reference outside the root is refused on both.
#[test]
fn c_s3e2_project_config_is_relative_to_the_source_root() {
    let f = Fixture::new();
    for repo in [&f.a, &f.b] {
        std::fs::write(repo.join("project.toml"), "[context]\nenabled = false\n").unwrap();
    }
    std::fs::write(f.base.join("outside.toml"), "[context]\nenabled = false\n").unwrap();
    let config = |repo: &Path| -> Value {
        let list: Value =
            serde_json::from_str(&f.ok(&f.base, &["project", "list", "--json"])).unwrap();
        list.as_array()
            .unwrap()
            .iter()
            .find(|p| p["root"] == repo.to_str().unwrap())
            .unwrap()["config_ref"]
            .clone()
    };
    let add = |repo: &Path, reference: &str| {
        f.cli(
            &f.base,
            &[
                "--project-config",
                reference,
                "project",
                "add",
                repo.to_str().unwrap(),
            ],
        )
    };
    // Offline (no daemon), then through the API.
    for (repo, online) in [(&f.a, false), (&f.b, true)] {
        if online {
            f.start();
        }
        let refused = add(repo, "../outside.toml");
        assert!(!refused.status.success(), "online={online}");
        let out = add(repo, "project.toml");
        assert!(out.status.success(), "online={online}: {}", stderr(&out));
        assert_eq!(
            config(repo),
            repo.join("project.toml").to_str().unwrap(),
            "online={online}"
        );
    }
}

/// C-S3f (D5): a preflight whose Git child is held past the deadline is
/// `ProjectPreflightUnavailable`; the child is reaped before the response,
/// and while the entry is still held (its pipes are still open) a commit
/// for the same Project is `ProjectPreflightInFlight` and another Project
/// commits. Mutant: release the entry before the reap → A's update lands
/// → FAIL.
#[test]
fn c_s3f_in_flight_entry_clears_only_after_the_reap() {
    let f = Fixture::new();
    let a = f.add(&f.a);
    let b = f.add(&f.b);
    f.start();
    std::fs::write(f.a.join(".git/rrx-escape"), "").unwrap();
    f.hold(&f.a);
    let cut = status_of(&f.state, a);
    assert_eq!(
        reason(&cut),
        Some(UnavailableReason::ProjectPreflightUnavailable),
        "{cut:?}"
    );
    let pid = f.held(&f.a)[0];
    assert!(!alive(pid), "the held Git child was not reaped first");
    let (_, va, _) = lookup(&f.state, &f.a);
    let (row, audit) = (f.row(a), f.audit(a));
    let update = f.api(ControlAction::ProjectUpdate {
        project: a,
        expected_project: va,
        options: renamed("during"),
    });
    assert_eq!(
        reason(&update),
        Some(UnavailableReason::ProjectPreflightInFlight),
        "{update:?}"
    );
    assert_eq!((f.row(a), f.audit(a)), (row, audit));
    let (_, vb, _) = lookup(&f.state, &f.b);
    match f.api(ControlAction::ProjectUpdate {
        project: b,
        expected_project: vb,
        options: renamed("other"),
    }) {
        ControlResponse::ProjectSaved { changed: true, .. } => {}
        other => panic!("B's update: {other:?}"),
    }
    f.release(&f.a);
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let ControlResponse::ProjectStatusFacts {
            reconcile: ReconcileMark::Checked,
            ..
        } = status_of(&f.state, a)
        {
            break;
        }
        assert!(Instant::now() < deadline, "A's entry never cleared");
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// C-S3g (D2, L1): (i) an invalid descriptor, (ii) a probe holding the
/// exclusive lock and (iii) a live daemon whose Git sleeps past the
/// deadline. Each subcommand fails typed with its own reason and changes
/// no row, version or audit. M-a (direct-Store fallback after an API error)
/// and M-b (offline fallback, which is `owner busy` under a live daemon)
/// both lose (iii)'s typed API reason → FAIL.
#[test]
fn c_s3g_refused_routes_are_typed_and_write_nothing() {
    let f = Fixture::new();
    let a = f.add(&f.a);
    let (a_id, a_path) = (a.to_string(), f.a.to_str().unwrap().to_owned());
    let c_path = f.c.to_str().unwrap().to_owned();
    let commands: Vec<Vec<&str>> = vec![
        vec!["project", "add", &c_path],
        vec!["project", "add", &a_path],
        vec!["project", "remove", &a_id],
        vec!["project", "list"],
        vec!["project", "status", &a_id],
    ];
    let unchanged = |case: &str, before: &(Option<(i64, String)>, i64, i64)| {
        assert_eq!(
            &(f.row(a), f.audit(a), f.projects()),
            before,
            "{case} changed state"
        );
    };
    let before = (f.row(a), f.audit(a), f.projects());
    let check = |case: &str, code: Option<i32>, text: &str| {
        for command in &commands {
            let out = f.cli(&f.base, command);
            assert!(!out.status.success(), "{case} {command:?} succeeded");
            if code.is_some() {
                assert_eq!(out.status.code(), code, "{case} {command:?}");
            }
            assert!(
                stderr(&out).contains(text),
                "{case} {command:?}: {}",
                stderr(&out)
            );
        }
    };
    // (i) D Invalid: a strictly placed but corrupt descriptor.
    let control = f.execution().join("control");
    for (dir, mode) in [(f.execution(), 0o755), (control.clone(), 0o700)] {
        match std::fs::DirBuilder::new().mode(mode).create(&dir) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => panic!("{e}"),
        }
    }
    let descriptor = control.join("endpoint.json");
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&descriptor)
        .unwrap()
        .write_all(b"{not json")
        .unwrap();
    check("(i)", Some(4), "discovery unavailable");
    unchanged("(i)", &before);
    std::fs::remove_file(&descriptor).unwrap();
    // (ii) the exclusive owner lock held by a probe; no descriptor.
    {
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(f.execution().join("owner.lock"))
            .unwrap();
        rustix::fs::flock(&lock, rustix::fs::FlockOperation::NonBlockingLockExclusive).unwrap();
        check("(ii)", Some(4), "owner busy");
        unchanged("(ii)", &before);
    }
    // (iii) D Valid; every Project Git in the daemon sleeps past the deadline.
    f.start();
    let before = (f.row(a), f.audit(a), f.projects());
    f.hold(&f.a);
    f.hold(&f.c);
    check("(iii)", None, "project_preflight_unavailable");
    unchanged("(iii)", &before);
}

/// C-S3h (M3): remove A, then add A through the API reactivates the same ID;
/// a client whose looked-up version went stale is `ProjectCurrencyChanged`
/// with row and audit unchanged.
#[test]
fn c_s3h_add_reactivates_a_removed_row_and_refuses_a_stale_lookup() {
    let f = Fixture::new();
    f.start();
    let id = f.add(&f.a);
    f.ok(&f.base, &["project", "remove", &id.to_string()]);
    assert_eq!(lookup(&f.state, &f.a).2, ProjectState::Removed);
    assert_eq!(f.add(&f.a), id);
    let (found, v, state) = lookup(&f.state, &f.a);
    assert_eq!((found, state), (id, ProjectState::Registered));
    f.ok(
        &f.base,
        &["project", "add", f.a.to_str().unwrap(), "--name", "renamed"],
    );
    let (row, audit) = (f.row(id), f.audit(id));
    let stale = f.api(ControlAction::ProjectUpdate {
        project: id,
        expected_project: v,
        options: renamed("stale"),
    });
    assert_eq!(
        reason(&stale),
        Some(UnavailableReason::ProjectCurrencyChanged),
        "{stale:?}"
    );
    assert_eq!((f.row(id), f.audit(id)), (row, audit));
}

/// Sol 6079932858 M2: List checks B at version v while A holds the List;
/// B is updated before the List returns. B is reported at its new version
/// as `changed`, never as `checked`. Mutant: keep the snapshot mark → FAIL.
#[test]
fn m2_a_row_changed_after_its_check_is_not_reported_checked() {
    let f = Fixture::new();
    let a = f.add(&f.a);
    let b = f.add(&f.b);
    f.start();
    f.hold(&f.a);
    let state = f.state.clone();
    let list = std::thread::spawn(move || api(&state, ControlAction::ProjectList { all: false }));
    f.wait_held(&f.a);
    // B's own check has finished: its preflight is no longer in flight.
    let deadline = Instant::now() + Duration::from_secs(5);
    let vb = loop {
        let (_, vb, _) = lookup(&f.state, &f.b);
        match f.api(ControlAction::ProjectUpdate {
            project: b,
            expected_project: vb,
            options: renamed("moved"),
        }) {
            ControlResponse::ProjectSaved { project, .. } => break project.version,
            other => {
                assert_eq!(
                    reason(&other),
                    Some(UnavailableReason::ProjectPreflightInFlight)
                );
                assert!(Instant::now() < deadline, "B stayed in flight");
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    };
    f.release(&f.a);
    let rows = match list.join().unwrap() {
        ControlResponse::ProjectRows { rows } => rows,
        other => panic!("list: {other:?}"),
    };
    let row = |id| rows.iter().find(|r| r.project.id == id).unwrap();
    assert_eq!(row(b).project.version, vb);
    assert_eq!(row(b).reconcile, ReconcileMark::Changed);
    assert_eq!(row(a).reconcile, ReconcileMark::Checked);
}

/// Sol 6079932858 M4: a Git failure's stderr reaches neither a typed
/// refusal, nor a Blocked reason in any API response, nor the stored row.
#[test]
fn m4_git_stderr_never_reaches_api_projections_or_stored_reasons() {
    let f = Fixture::new();
    let a = f.add(&f.a);
    f.start();
    for repo in [&f.a, &f.c] {
        std::fs::write(repo.join(".git/rrx-fail"), "").unwrap();
    }
    let secret = "SECRET-TOKEN-81";
    let refused = f.api(ControlAction::ProjectRegister {
        path: f.c.clone(),
        options: ProjectOptions::default(),
    });
    match &refused {
        ControlResponse::ProjectRefused { reason } => {
            assert!(reason.contains("failed"), "{reason}");
            assert!(!reason.contains(secret), "{reason}");
        }
        other => panic!("register: {other:?}"),
    }
    let cli = f.cli(&f.base, &["project", "add", f.c.to_str().unwrap()]);
    assert!(!cli.status.success());
    assert!(!stderr(&cli).contains(secret), "{}", stderr(&cli));
    let status = status_of(&f.state, a);
    let ControlResponse::ProjectStatusFacts { status, .. } = &status else {
        panic!("status: {status:?}");
    };
    assert_eq!(status.project.state, ProjectState::Blocked);
    let reason = status.project.blocked_reason.clone().unwrap();
    assert!(
        reason.contains("failed") && !reason.contains(secret),
        "{reason}"
    );
    let ControlResponse::ProjectRows { rows } = f.api(ControlAction::ProjectList { all: true })
    else {
        panic!("list");
    };
    for row in rows {
        assert!(!format!("{:?}", row.project).contains(secret));
    }
    let stored = f.row(a).unwrap().1;
    assert!(!stored.contains(secret), "{stored}");
    for out in [
        f.cli(&f.base, &["project", "status", &a.to_string(), "--json"]),
        f.cli(&f.base, &["project", "list", "--all", "--json"]),
    ] {
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(!text.contains(secret), "{text}");
    }
}
