//! Synthetic peers exercise genuine Session/Core consumers. No CLI, S/P,
//! installed ingress, Task Waiting/Held or full-M0 qualification is claimed.
use super::*;
use crate::execution::native_result::AcquisitionStatus;

fn peer(root: &Path, mode: &str) -> std::path::PathBuf {
    let path = root.join("confirmed-findings-peer");
    let script = format!(
        "#!/usr/bin/python3\nMODE = {mode:?}\n{}",
        r#"
import json
import os
import sys
if sys.argv[1:] == ['--version']:
    print('2.1.294 (Claude Code)', flush=True)
    sys.exit(0)
native = sys.argv[sys.argv.index('--session-id') + 1]
uuid = '12345678-1234-1234-1234-123456789abc'
queued_uuid = '12345678-1234-1234-1234-123456789abd'
def send(value):
    print(json.dumps(value), flush=True)
def number(value, key, text):
    value[key] = '__synthetic_number__'
    print(json.dumps(value).replace('"__synthetic_number__"', text), flush=True)
def init(extra=None):
    value = {'type':'system', 'subtype':'init', 'session_id':native, 'cwd':os.getcwd(), 'tools':[], 'mcp_servers':[]}
    if extra: value.update(extra)
    send(value)
for line in sys.stdin:
    value = json.loads(line)
    if value['type'] == 'control_request':
        if MODE == 'queued-init': init()
        if MODE == 'queued-counter-fraction':
            number({'type':'system','subtype':'api_retry','max_retries':1,'retry_delay_ms':0,'error_status':None}, 'attempt', '9007199254740992.5')
        if MODE == 'queued-uuid-collision':
            number({'type':'system','subtype':'session_state_changed','state':'running','uuid':queued_uuid}, 'extra', '9007199254740992.1')
        response = {'type':'control_response','uuid':uuid,'response':{'request_id':value['request_id'],'subtype':'success','response':{}}}
        if MODE == 'bootstrap-owned': response['session_id'] = native
        if MODE == 'bootstrap-foreign': response['session_id'] = 'synthetic-foreign-session'
        if MODE == 'bootstrap-wrong-request': response['response']['request_id'] = 'synthetic-other-request'
        if MODE == 'bootstrap-malformed': response['response']['response'] = []
        send(response)
    elif value['type'] == 'user':
        if MODE != 'queued-init':
            init({'uuid':uuid} if MODE == 'bootstrap-uuid-init' else None)
        if MODE.startswith('late-'):
            send({'type':'control_request','request_id':'late-barrier','request':{'subtype':'can_use_tool','tool_use_id':'synthetic-tool','tool_name':'Bash','input':{}}})
            release = os.path.join(os.environ['RRX_OUTPUT_DIR'], 'confirmed-findings-release')
            import time
            while not os.path.exists(release): time.sleep(0.01)
        if MODE == 'queued-uuid-collision':
            number({'type':'system','subtype':'session_state_changed','state':'running','uuid':queued_uuid}, 'extra', '9007199254740992.2')
        if MODE.startswith('status-'):
            number({'type':'result','session_id':native,'subtype':'error_during_execution','is_error':True,'errors':['ordinary synthetic failure']}, 'api_error_status', MODE[len('status-'):])
        elif MODE == 'counter-fraction':
            frame = {'type':'result','session_id':native,'subtype':'success','is_error':False,'result':'synthetic answer','usage':{'input_tokens':'__synthetic_number__'}}
            print(json.dumps(frame).replace('"__synthetic_number__"', '9007199254740992.5'), flush=True)
        elif MODE == 'counter-max':
            frame = {'type':'result','session_id':native,'subtype':'success','is_error':False,'result':'synthetic answer','usage':{'input_tokens':'__synthetic_number__'}}
            print(json.dumps(frame).replace('"__synthetic_number__"', '18446744073709551615.0'), flush=True)
        else:
            frame = {'type':'result','session_id':native,'subtype':'success','is_error':False,'result':'synthetic answer','usage':{'input_tokens':7}}
            if MODE == 'late-failure': frame['usage']['input_tokens'] = 'wrongtype'
            if MODE == 'bootstrap-uuid-result': frame['uuid'] = uuid
            send(frame)
"#
    );
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}

struct PeerRelease(std::path::PathBuf);
impl Drop for PeerRelease {
    fn drop(&mut self) {
        let _ = std::fs::write(&self.0, "release");
    }
}
struct StatusRelease(Option<std::sync::mpsc::Sender<()>>);
impl Drop for StatusRelease {
    fn drop(&mut self) {
        if let Some(send) = self.0.take() {
            let _ = send.send(());
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn i01_actual_status_reads_publication_after_no_frozen_snapshot() {
    for failed in [false, true] {
        let (dir, owner, task) = results::tests::fixture().await;
        let sessions = Arc::new(NativeSessions::new(owner.clone()).unwrap());
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, "claude", "Implement", None)
            .await
            .unwrap();
        let mode = if failed {
            "late-failure"
        } else {
            "late-success"
        };
        let NativeStart::Launched(handle) = sessions
            .start_inner(
                input(&unit, "synthetic input"),
                None,
                None,
                Some(peer(dir.path(), mode)),
            )
            .await
            .unwrap()
        else {
            panic!("synthetic launch queued")
        };
        let mut updates = sessions.subscribe(&handle).unwrap();
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if updates
                    .borrow()
                    .pending
                    .iter()
                    .any(|p| p["request_id"] == "late-barrier")
                {
                    break;
                }
                updates.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        assert!(updates.borrow().receipt.is_none());
        let output = owner
            .root
            .join("units")
            .join(unit.id.to_string())
            .join("output");
        let peer_release = PeerRelease(output.join("confirmed-findings-release"));
        let (arrived_send, arrived) = tokio::sync::oneshot::channel();
        let arrived_send = Arc::new(Mutex::new(Some(arrived_send)));
        let (release_send, release_receive) = std::sync::mpsc::channel();
        let release_receive = Arc::new(Mutex::new(release_receive));
        sessions.set_status_observer(Some(Arc::new(move || {
            let send = arrived_send.lock().unwrap().take();
            if let Some(send) = send {
                let _ = send.send(());
                release_receive
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap();
            }
        })));
        let release_status = StatusRelease(Some(release_send));
        let status_sessions = sessions.clone();
        let status_handle = handle.clone();
        let read = tokio::task::spawn_blocking(move || status_sessions.status(&status_handle));
        tokio::time::timeout(Duration::from_secs(5), arrived)
            .await
            .unwrap()
            .unwrap();
        // Actual status is paused after observing no retained terminal. Allow
        // genuine Core capture, persistence, child reap and watch publication.
        drop(peer_release);
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let done = {
                    let observed = updates.borrow();
                    observed.receipt.is_some()
                        && observed.pending.is_empty()
                        && observed.failure == failed.then_some(NativeFailure::ProtocolFailure)
                };
                if done {
                    break;
                }
                updates.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        assert!(
            sessions
                .entries
                .lock()
                .unwrap()
                .get(&handle.session)
                .unwrap()
                .terminal
                .lock()
                .unwrap()
                .is_none()
        );
        let published = updates.borrow().clone();
        drop(release_status);
        let actual = read.await.unwrap().unwrap();
        sessions.set_status_observer(None);
        let receipt = owner
            .store
            .lock()
            .unwrap()
            .native_session_result(handle.session)
            .unwrap()
            .unwrap();
        assert_eq!(actual.receipt, Some(receipt.id));
        assert_eq!(actual.receipt, published.receipt);
        assert_eq!(actual.failure, published.failure);
        assert_eq!(actual.diagnostic, published.diagnostic);
        assert!(actual.pending.is_empty());
        assert_eq!(actual.work, Some(receipt.observed_work));
        if failed {
            assert_eq!(actual.failure, Some(NativeFailure::ProtocolFailure));
            assert_eq!(
                actual.diagnostic,
                Some(NativeFailure::ProtocolFailure.diagnostic())
            );
            assert_eq!(actual.disposition, Disposition::ProtocolError);
            assert_eq!(actual.session.state, SessionState::Lost);
            assert!(actual.metrics.is_none());
            assert!(receipt.text.is_none());
            assert_ne!(receipt.acquisition, AcquisitionStatus::Complete);
        } else {
            assert_eq!(actual.failure, None);
            assert_eq!(actual.disposition, Disposition::Completed);
            assert_eq!(actual.metrics.as_ref().unwrap().input(), Some(7));
            assert_eq!(published.metrics.as_ref().unwrap().input(), Some(7));
            assert_eq!(receipt.acquisition, AcquisitionStatus::Complete);
            assert_eq!(receipt.text.as_deref(), Some("synthetic answer"));
        }
        sessions.release(&handle).unwrap();
    }
}

#[tokio::test]
async fn i02_actual_bootstrap_ownership_history_and_queued_init() {
    for (mode, rejected, input_written) in [
        ("bootstrap-absent", false, true),
        ("bootstrap-owned", false, true),
        ("queued-init", false, true),
        ("bootstrap-foreign", true, false),
        ("bootstrap-wrong-request", true, false),
        ("bootstrap-malformed", true, false),
        ("bootstrap-uuid-init", true, true),
        ("bootstrap-uuid-result", true, true),
    ] {
        let (dir, owner, task) = results::tests::fixture().await;
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, "claude", "Implement", None)
            .await
            .unwrap();
        let NativeStart::Launched(handle) = sessions
            .start_inner(
                input(&unit, "synthetic input"),
                None,
                None,
                Some(peer(dir.path(), mode)),
            )
            .await
            .unwrap()
        else {
            panic!("synthetic launch queued")
        };
        let actual = terminal(&sessions, &handle).await;
        let store = owner.store.lock().unwrap();
        let invocation = store.native_session_invocation(handle.session).unwrap();
        assert_eq!(
            invocation.input_operation.is_some(),
            input_written,
            "{mode}"
        );
        let receipt = store
            .native_session_result(handle.session)
            .unwrap()
            .unwrap();
        if rejected {
            let (failure, disposition) = if mode == "bootstrap-foreign" {
                (NativeFailure::AuthorityUnavailable, Disposition::Refused)
            } else {
                (NativeFailure::ProtocolFailure, Disposition::ProtocolError)
            };
            assert_eq!(actual.failure, Some(failure), "{mode}");
            assert_eq!(actual.disposition, disposition, "{mode}");
            assert_eq!(actual.work, Some(WorkOutcome::Unknown), "{mode}");
            assert!(actual.metrics.is_none(), "{mode}");
            assert!(
                receipt.text.is_none() && receipt.answer_sha256.is_none(),
                "{mode}"
            );
            assert_ne!(receipt.acquisition, AcquisitionStatus::Complete, "{mode}");
        } else {
            assert_eq!(actual.work, Some(WorkOutcome::Success), "{mode}");
            assert_eq!(actual.metrics.as_ref().unwrap().input(), Some(7), "{mode}");
            assert_eq!(receipt.acquisition, AcquisitionStatus::Complete, "{mode}");
        }
        drop(store);
        sessions.release(&handle).unwrap();
    }
}

#[tokio::test]
async fn i03_actual_native_status_consumes_checked_api_classification() {
    for (forms, failure, disposition) in [
        (
            ["401", "401.0", "4.01e2"],
            Some(NativeFailure::AuthenticationUnavailable),
            Disposition::Refused,
        ),
        (
            ["429", "429.0", "4.29e2"],
            None,
            Disposition::CapacityInterrupted,
        ),
        (
            ["503", "503.0", "5.03e2"],
            None,
            Disposition::CapacityInterrupted,
        ),
        (["200", "200.0", "2e2"], None, Disposition::Completed),
    ] {
        for form in forms {
            let (dir, owner, task) = results::tests::fixture().await;
            let sessions = NativeSessions::new(owner.clone()).unwrap();
            let (unit, _) = attempts::AttemptManager::new(owner.clone())
                .prepare(task.id, "claude", "Implement", None)
                .await
                .unwrap();
            let NativeStart::Launched(handle) = sessions
                .start_inner(
                    input(&unit, "synthetic input"),
                    None,
                    None,
                    Some(peer(dir.path(), &format!("status-{form}"))),
                )
                .await
                .unwrap()
            else {
                panic!("synthetic launch queued")
            };
            let actual = terminal(&sessions, &handle).await;
            assert_eq!(actual.failure, failure, "{form}");
            assert_eq!(actual.disposition, disposition, "{form}");
            assert_eq!(
                actual.work,
                Some(if disposition == Disposition::Completed {
                    WorkOutcome::Failure
                } else {
                    WorkOutcome::Unknown
                }),
                "{form}"
            );
            assert!(actual.metrics.is_none());
            let store = owner.store.lock().unwrap();
            assert!(
                store
                    .quota_observations("claude", "unknown")
                    .unwrap()
                    .is_empty()
            );
            if disposition == Disposition::CapacityInterrupted {
                assert_eq!(
                    store.execution_unit(unit.id).unwrap().wait_reason,
                    Some(WaitReason::Capacity)
                );
            }
            drop(store);
            sessions.release(&handle).unwrap();
        }
    }
}

#[tokio::test]
async fn i04_lines_and_queue_deliver_original_numbers_to_real_consumers() {
    for (mode, rejected) in [
        ("counter-max", false),
        ("counter-fraction", true),
        ("queued-counter-fraction", true),
        ("queued-uuid-collision", true),
    ] {
        let (dir, owner, task) = results::tests::fixture().await;
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, "claude", "Implement", None)
            .await
            .unwrap();
        let NativeStart::Launched(handle) = sessions
            .start_inner(
                input(&unit, "synthetic input"),
                None,
                None,
                Some(peer(dir.path(), mode)),
            )
            .await
            .unwrap()
        else {
            panic!("synthetic launch queued")
        };
        let actual = terminal(&sessions, &handle).await;
        let receipt = owner
            .store
            .lock()
            .unwrap()
            .native_session_result(handle.session)
            .unwrap()
            .unwrap();
        if rejected {
            assert_eq!(
                actual.failure,
                Some(NativeFailure::ProtocolFailure),
                "{mode}"
            );
            assert_eq!(actual.disposition, Disposition::ProtocolError, "{mode}");
            assert!(actual.metrics.is_none(), "{mode}");
            assert!(
                receipt.text.is_none() && receipt.answer_sha256.is_none(),
                "{mode}"
            );
            assert_ne!(receipt.acquisition, AcquisitionStatus::Complete, "{mode}");
        } else {
            assert_eq!(actual.work, Some(WorkOutcome::Success));
            assert_eq!(actual.metrics.as_ref().unwrap().input(), Some(u64::MAX));
            assert_eq!(receipt.acquisition, AcquisitionStatus::Complete);
        }
        sessions.release(&handle).unwrap();
    }
}
