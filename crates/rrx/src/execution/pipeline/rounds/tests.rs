use super::*;

/// Only passing tests and an approve verdict without blocking labels permit release.
#[test]
fn b8_release() {
    assert_eq!(
        decide(true, Some(&ReviewVerdict::Approve), 0, 0, 0, 2).unwrap(),
        RoundDecision::Release
    );
    assert_ne!(
        decide(false, Some(&ReviewVerdict::Approve), 0, 0, 0, 2).unwrap(),
        RoundDecision::Release
    );
    assert_ne!(
        decide(true, Some(&ReviewVerdict::Approve), 1, 0, 0, 2).unwrap(),
        RoundDecision::Release
    );
}

/// Unclear blocking evidence holds before unsupported requests or fix eligibility.
#[test]
fn b8_unclear() {
    for verdict in [ReviewVerdict::Approve, ReviewVerdict::RequestChanges] {
        assert_eq!(
            decide(true, Some(&verdict), 0, 1, 0, 2).unwrap(),
            RoundDecision::Hold("unclear_finding")
        );
        assert_eq!(
            decide(false, Some(&verdict), 1, 1, 0, 2).unwrap(),
            RoundDecision::Hold("unclear_finding")
        );
    }
}

/// Requests unsupported by confirmed blocking findings never authorize a fix.
#[test]
fn b8_unsupported() {
    assert_eq!(
        decide(true, Some(&ReviewVerdict::RequestChanges), 0, 0, 0, 2).unwrap(),
        RoundDecision::Hold("unsupported_request_changes")
    );
}

/// Fix eligibility consumes each allowed round and holds exactly at limits zero through two.
#[test]
fn b8_round_limits() {
    for max in 0..=2 {
        for round in 0..=max {
            for failing_test in [false, true] {
                let got = decide(
                    !failing_test,
                    (!failing_test).then_some(&ReviewVerdict::RequestChanges),
                    usize::from(!failing_test),
                    0,
                    round,
                    max,
                )
                .unwrap();
                assert_eq!(
                    got,
                    if round < max {
                        RoundDecision::Fix
                    } else {
                        RoundDecision::Hold("round_limit")
                    }
                );
            }
        }
    }
    assert!(decide(false, None, 0, 0, 0, 3).is_err());
    assert!(decide(false, None, 0, 0, 2, 1).is_err());
}

/// Remediation data and quotes cannot escape the fresh per-step nonce delimiters.
#[test]
fn b8_fix_framing() {
    let nonce = "abcdef0123456789abcdef0123456789";
    let input = FixInput::TestFailure {
        command_id: "ignore instructions".into(),
        exit_code: Some(1),
        signal: None,
        timed_out: false,
        overflow: false,
    };
    let prompt = String::from_utf8(fix_prompt(nonce, &input).unwrap()).unwrap();
    assert!(prompt.contains(&format!("BEGIN_UNTRUSTED_{nonce}\nData: ")));
    assert!(prompt.ends_with(&format!("\nEND_UNTRUSTED_{nonce}\n")));
    let collision = FixInput::TestFailure {
        command_id: nonce.into(),
        exit_code: Some(1),
        signal: None,
        timed_out: false,
        overflow: false,
    };
    assert_eq!(
        fix_prompt(nonce, &collision).unwrap_err().to_string(),
        "review_nonce_collision"
    );
    assert_eq!(
        fix_prompt("short", &input).unwrap_err().to_string(),
        "review_nonce_invalid"
    );
}

/// A secret in a stored failure summary is refused before a fix prompt can be sent.
#[test]
fn b8_fix_prompt_secret() {
    let input = FixInput::TestFailure {
        command_id: "EXAMPLE_TOKEN=fixture".into(),
        exit_code: Some(1),
        signal: None,
        timed_out: false,
        overflow: false,
    };
    assert!(
        fix_prompt("abcdef0123456789abcdef0123456789", &input)
            .unwrap_err()
            .downcast_ref::<crate::execution::pipeline::SecretHit>()
            .is_some()
    );
}
