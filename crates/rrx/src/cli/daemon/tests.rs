//! Daemon start controls that need a parent-side pause (C-S1b, C-S1b2).
use super::*;
use std::sync::{Arc, Condvar, Mutex};

pub(super) const BEFORE_IDENTITY_MATCH: &str = "before identity match";

/// 0 armed, 1 reached, 2 released.
struct Pause {
    state: Mutex<u8>,
    changed: Condvar,
}
static PAUSES: Mutex<Vec<(&'static str, Arc<Pause>)>> = Mutex::new(Vec::new());

fn arm(site: &'static str) -> Arc<Pause> {
    let pause = Arc::new(Pause {
        state: Mutex::new(0),
        changed: Condvar::new(),
    });
    PAUSES.lock().unwrap().push((site, pause.clone()));
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
            assert!(!timeout.timed_out(), "pause never reached");
            state = next;
        }
    }
    fn release(&self) {
        *self.state.lock().unwrap() = 2;
        self.changed.notify_all();
    }
}

pub(super) fn pause(site: &'static str) {
    let pause = {
        let mut pauses = PAUSES.lock().unwrap();
        pauses
            .iter()
            .position(|(armed, _)| *armed == site)
            .map(|index| pauses.remove(index).1)
    };
    let Some(pause) = pause else { return };
    *pause.state.lock().unwrap() = 1;
    pause.changed.notify_all();
    let wait = || {
        let mut state = pause.state.lock().unwrap();
        while *state != 2 {
            state = pause.changed.wait(state).unwrap();
        }
    };
    tokio::task::block_in_place(wait);
}
