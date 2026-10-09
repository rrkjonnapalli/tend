use super::*;

const MINUTE: Duration = Duration::from_secs(60);

fn failed(key: crate::error::ErrorDef) -> AppResult<Outcome> {
    Err(key.err())
}

#[test]
fn a_success_waits_the_interval_and_zero_waits_for_a_trigger() {
    let mut backoff = Backoff::default();
    backoff.after(&Ok(Outcome::Idle));
    assert_eq!(backoff.wait(15), Some(15 * MINUTE));
    assert_eq!(backoff.wait(0), None);
}

#[test]
fn failures_double_the_wait_up_to_an_hour() {
    let mut backoff = Backoff::default();
    let mut waits = Vec::new();
    for _ in 0..8 {
        backoff.after(&failed(errors::SYNC_OFFLINE));
        waits.push(backoff.wait(15).unwrap());
    }
    let minutes: Vec<u64> = waits.iter().map(|w| w.as_secs() / 60).collect();
    assert_eq!(minutes, [1, 2, 4, 8, 16, 32, 60, 60]);
    backoff.after(&Ok(Outcome::Pushed));
    assert_eq!(
        backoff.wait(15),
        Some(15 * MINUTE),
        "a success resets the backoff"
    );
}

#[test]
fn errors_the_user_must_fix_stop_automatic_runs() {
    for key in [
        errors::SYNC_AUTH,
        errors::SYNC_BASE_LOST,
        errors::SNAPSHOT_TOO_NEW,
    ] {
        let mut backoff = Backoff::default();
        backoff.after(&failed(key));
        assert_eq!(backoff.wait(15), None, "{}", key.key);
    }
}
