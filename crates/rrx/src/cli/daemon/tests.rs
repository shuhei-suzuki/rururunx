//! Daemon controls that need the parent in this process: C-S1b, C-S1b2,
//! stop outcomes and `serve` failure retention. The service is the compiled
//! `rrx` binary of this build; only the test's own wrapper may delay its exec.
use super::*;
use crate::{
    cli::endpoint::ControlEndpoint, execution::RuntimeOwner, runtime::stop::park::park_loop,
};
use std::sync::{Arc, Condvar, Mutex};

pub(super) const BEFORE_IDENTITY_MATCH: &str = "before identity match";

/// 0 armed, 1 reached, 2 released.
struct Pause {
    state: Mutex<u8>,
    changed: Condvar,
}
type Armed = (&'static str, PathBuf, Arc<Pause>);
static PAUSES: Mutex<Vec<Armed>> = Mutex::new(Vec::new());

fn arm(site: &'static str, state: &Path) -> Arc<Pause> {
    let pause = Arc::new(Pause {
        state: Mutex::new(0),
        changed: Condvar::new(),
    });
    PAUSES
        .lock()
        .unwrap()
        .push((site, state.to_path_buf(), pause.clone()));
    pause
}
impl Pause {
    fn wait_reached(&self) {
        let mut state = self.state.lock().unwrap();
        while *state == 0 {
            let (next, timeout) = self
                .changed
                .wait_timeout(state, Duration::from_secs(60))
                .unwrap();
            assert!(!timeout.timed_out(), "SETUP: pause never reached");
            state = next;
        }
    }
    fn release(&self) {
        *self.state.lock().unwrap() = 2;
        self.changed.notify_all();
    }
}

/// Parent-side pause, scoped to one site and one state path.
pub(super) fn pause(site: &'static str, state: &Path) {
    let pause = {
        let mut pauses = PAUSES.lock().unwrap();
        pauses
            .iter()
            .position(|(armed, path, _)| *armed == site && path == state)
            .map(|index| pauses.remove(index).2)
    };
    let Some(pause) = pause else { return };
    *pause.state.lock().unwrap() = 1;
    pause.changed.notify_all();
    tokio::task::block_in_place(|| {
        let mut state = pause.state.lock().unwrap();
        while *state != 2 {
            state = pause.changed.wait(state).unwrap();
        }
    });
}

/// The compiled service of this build (`target/<profile>/rrx`).
fn rrx() -> PathBuf {
    let binary = std::env::current_exe()
        .unwrap()
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .join("rrx");
    assert!(
        binary.is_file(),
        "SETUP: build the rrx binary first (cargo build -p rrx --tests)"
    );
    binary
}

struct Root {
    _dir: tempfile::TempDir,
    state: PathBuf,
    config: PathBuf,
}
impl Root {
    fn new() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("rrx-daemon-")
            .tempdir_in("/tmp")
            .unwrap();
        let base = dir.path().canonicalize().unwrap();
        let config = base.join("config.toml");
        std::fs::write(
            &config,
            "[agents.worker]\nprovider='claude'\ncommand=['/usr/bin/false']\n",
        )
        .unwrap();
        Self {
            state: base.join("state.db"),
            config,
            _dir: dir,
        }
    }
    fn base(&self) -> &Path {
        self.state.parent().unwrap()
    }
    async fn start(&self, launch: &Launch) -> StartOutcome {
        start_with(launch, &self.state, Some(&self.config)).await
    }
    /// Stop through the endpoint and wait for the owner lock to be released.
    async fn stop_and_wait(&self) {
        assert!(
            matches!(stop(&self.state).await, StopOutcome::Completed { .. }),
            "SETUP: stop did not complete"
        );
        self.wait_free().await;
    }
    async fn wait_free(&self) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        while probe_lock(&self.state) != OwnerLock::Free {
            assert!(
                tokio::time::Instant::now() < deadline,
                "SETUP: owner not released"
            );
            tokio::time::sleep(POLL).await;
        }
    }
    fn control_entries(&self) -> Vec<String> {
        let mut entries: Vec<_> = std::fs::read_dir(self.base().join("state.db.execution/control"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        entries.sort();
        entries
    }
}
fn direct() -> Launch {
    Launch::new(rrx(), Vec::new())
}
/// The parent's own child is this shell until the gate exists; it then
/// execs the same service, keeping the PID and the readiness pipe.
fn gated(gate: &Path) -> Launch {
    Launch::new(
        "/bin/sh".into(),
        vec![
            "-c".into(),
            format!(
                "while [ ! -e '{}' ]; do sleep 0.02; done; exec \"$0\" \"$@\"",
                gate.display()
            )
            .into(),
            rrx().into(),
        ],
    )
}

/// C-S1b: 8 concurrent starts, plus a loser whose own child is held before
/// `RuntimeOwner::open` until the winner has published. Exactly one runs,
/// one epoch is begun, one descriptor exists, and the held loser's parent
/// never reports the winner's endpoint as its own start.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn c_s1b_concurrent_starts_one_runs_and_held_loser_never_claims_running() {
    let root = Arc::new(Root::new());
    let gate = root.base().join("gate");
    let held = {
        let root = root.clone();
        let launch = gated(&gate);
        tokio::spawn(async move { root.start(&launch).await })
    };
    // The held parent has probed absent + free and spawned once its log exists.
    let log = root.base().join("state.db.execution/daemon/serve.log");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while !log.exists() {
        assert!(tokio::time::Instant::now() < deadline, "SETUP: no spawn");
        tokio::time::sleep(POLL).await;
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
    let starts: Vec<_> = (0..8)
        .map(|_| {
            let root = root.clone();
            tokio::spawn(async move { root.start(&direct()).await })
        })
        .collect();
    let mut outcomes = Vec::new();
    for start in starts {
        outcomes.push(start.await.unwrap());
    }
    let winners: Vec<_> = outcomes
        .iter()
        .filter_map(|o| match o {
            StartOutcome::Running { identity } => Some(identity.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(winners.len(), 1, "exactly one start runs: {outcomes:?}");
    let winner = winners[0].clone();
    for outcome in &outcomes {
        assert!(
            matches!(
                outcome,
                StartOutcome::Running { .. } | StartOutcome::OwnerBusy
            ) || matches!(outcome, StartOutcome::AlreadyRunning { identity } if *identity == winner),
            "typed refusal expected: {outcome:?}"
        );
    }
    std::fs::write(&gate, "").unwrap();
    let loser = held.await.unwrap();
    assert!(
        matches!(&loser, StartOutcome::AlreadyRunning { identity } if *identity == winner)
            || loser == StartOutcome::OwnerBusy,
        "held loser claimed the winner's start: {loser:?}"
    );
    assert_eq!(winner.epoch, 1, "a refused start began an epoch");
    assert_eq!(winner.protocol, transport::PROTOCOL_VERSION);
    assert_eq!(root.control_entries(), ["endpoint.json", "endpoint.lock"]);
    root.stop_and_wait().await;
    // The next owner is exactly the next epoch: none was begun in between.
    let next = root.start(&direct()).await;
    assert!(
        matches!(&next, StartOutcome::Running { identity } if identity.epoch == 2),
        "{next:?}"
    );
    root.stop_and_wait().await;
}

/// C-S1b2: the parent holds its own child's line A; A retires and a new epoch
/// B publishes before the identity match. The parent reports B as
/// `already running`, never as its own start.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn c_s1b2_identity_match_refuses_a_replacement_epoch() {
    let root = Arc::new(Root::new());
    let pause = arm(BEFORE_IDENTITY_MATCH, &root.state);
    let first = {
        let root = root.clone();
        tokio::spawn(async move { root.start(&direct()).await })
    };
    {
        let pause = pause.clone();
        tokio::task::spawn_blocking(move || pause.wait_reached())
            .await
            .unwrap();
    }
    match stop(&root.state).await {
        StopOutcome::Completed { epoch, .. } => assert_eq!(epoch, 1),
        other => panic!("SETUP: A did not stop: {other:?}"),
    }
    root.wait_free().await;
    let replacement = match root.start(&direct()).await {
        StartOutcome::Running { identity } => identity,
        other => panic!("SETUP: B did not start: {other:?}"),
    };
    assert_eq!(replacement.epoch, 2);
    pause.release();
    let outcome = first.await.unwrap();
    assert_eq!(
        outcome,
        StartOutcome::AlreadyRunning {
            identity: replacement
        },
        "the parent claimed a replacement epoch as its own start"
    );
    root.stop_and_wait().await;
}

/// C-S1d: a stop whose response never arrives is `unconfirmed`; a refused
/// stop is `failed`. Neither counts as stopped.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn c_s1d_stop_without_response_is_unconfirmed_and_refusal_is_failed() {
    let root = Root::new();
    let owner = RuntimeOwner::open(&root.state).unwrap();
    let endpoint = ControlEndpoint::bind(owner.clone()).unwrap();
    for (respond, expected) in [
        (false, StopOutcome::Unconfirmed),
        (true, StopOutcome::Failed),
    ] {
        let server = async {
            let accepted = endpoint.accept().await.unwrap();
            let mut reader = tokio::io::BufReader::new(accepted);
            let request: ControlRequest = transport::receive(&mut reader, transport::REQUEST_BYTES)
                .await
                .unwrap();
            assert!(matches!(request.action, ControlAction::RuntimeStop));
            if respond {
                transport::send(
                    reader.get_mut(),
                    &ControlResponse::Rejected {
                        request_id: Some(request.request_id),
                    },
                    transport::RESPONSE_BYTES,
                )
                .await
                .unwrap();
            }
            // Without a response the connection simply ends (EOF).
        };
        let ((), outcome) = tokio::join!(server, stop(&root.state));
        assert_eq!(outcome, expected);
    }
}

/// C-S1d: a pending stop is retained as failure by `serve` even when its own
/// later shutdown succeeds (the loop park opens right after the answer).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn c_s1d_serve_exits_unsuccessfully_after_a_pending_stop() {
    let root = Root::new();
    let park = park_loop(&root.state);
    let serve = {
        let state = root.state.clone();
        tokio::spawn(async move {
            crate::cli::service::serve(&state, crate::config::Config::default(), false).await
        })
    };
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while !matches!(status(&root.state).await, DaemonStatus::Running { .. }) {
        assert!(tokio::time::Instant::now() < deadline, "SETUP: no service");
        tokio::time::sleep(POLL).await;
    }
    match stop(&root.state).await {
        StopOutcome::Pending { site, .. } => assert_eq!(site, ShutdownSite::SupervisorJoin),
        other => panic!("stop was not pending: {other:?}"),
    }
    park.open();
    let result = tokio::time::timeout(Duration::from_secs(30), serve)
        .await
        .expect("serve did not exit")
        .unwrap();
    let error = result.expect_err("serve claimed success after a pending stop");
    assert!(
        format!("{error:#}").contains("Runtime control shutdown failed"),
        "{error:#}"
    );
}
