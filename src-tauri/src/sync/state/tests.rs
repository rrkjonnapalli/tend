use super::*;
use crate::db::open_memory;

fn times(conn: &Connection) -> (Option<i64>, Option<String>, Option<i64>) {
    conn.query_row(
        "SELECT last_run_at, last_result, last_ok_at FROM sync_state WHERE id = 1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
    .unwrap()
}

#[test]
fn a_failure_keeps_the_last_successful_time() {
    let conn = open_memory();
    record(&conn, OK).unwrap();
    let (run, _, ok) = times(&conn);
    assert_eq!(ok, run);
    std::thread::sleep(std::time::Duration::from_millis(5));
    record(&conn, "SYNC_OFFLINE").unwrap();
    let (failed_at, result, still_ok) = times(&conn);
    assert_eq!(result.as_deref(), Some("SYNC_OFFLINE"));
    assert_eq!(still_ok, ok);
    assert!(failed_at > ok);
}
