use rrx::{
    cli::client,
    runtime::{
        control::{ControlAction, ControlResponse},
        startup::StartupState,
    },
};
use std::{path::Path, time::Duration};

/// Whole-state baselines must follow public completion of the startup journal producer.
pub async fn settled(state: &Path) {
    wait_for_settlement(async || {
        let ControlResponse::RuntimeMetadata { startup, .. } =
            client::request(state, ControlAction::RuntimeStatus)
                .await
                .unwrap()
        else {
            panic!("startup metadata absent")
        };
        startup.state
    })
    .await;
}

async fn wait_for_settlement(mut observe: impl AsyncFnMut() -> StartupState) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let state = observe().await;
            if state == StartupState::Refused {
                return;
            }
            if state == StartupState::Passed {
                return;
            }
            assert_eq!(
                state,
                StartupState::Pending,
                "startup did not settle normally"
            );
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("startup did not settle before whole-state baseline");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A baseline waits through pending public observations until journal settlement.
    #[tokio::test]
    async fn whole_state_baseline_waits_for_public_startup_settlement() {
        for terminal in [StartupState::Refused, StartupState::Passed] {
            let mut observations =
                [StartupState::Pending, StartupState::Pending, terminal].into_iter();
            wait_for_settlement(async || observations.next().expect("polled after settlement"))
                .await;
            assert!(
                observations.next().is_none(),
                "baseline preceded settlement"
            );
        }
    }
}
