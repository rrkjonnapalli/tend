use super::*;
use crate::db::open_memory;

#[test]
fn start_list_extend_cancel_ring() {
    let conn = open_memory();
    assert!(start(&conn, "x", 0).is_err());
    let a = start(&conn, " Tea ", 300_000).unwrap();
    let b = start(&conn, "Motor", 60_000).unwrap();
    assert_eq!(a.title, "Tea");
    assert_eq!(
        list(&conn)
            .unwrap()
            .iter()
            .map(|t| t.id.clone())
            .collect::<Vec<_>>(),
        vec![b.id.clone(), a.id.clone()]
    );
    add_minute(&conn, &b.id).unwrap();
    let ends = list(&conn)
        .unwrap()
        .into_iter()
        .find(|t| t.id == b.id)
        .unwrap()
        .ends_at;
    assert_eq!(ends, b.ends_at + MINUTE_MS);
    cancel(&conn, &a.id).unwrap();
    ring(&conn, &b.id).unwrap();
    assert!(list(&conn).unwrap().is_empty());
}

#[test]
fn plus_a_minute_after_the_end_counts_from_now() {
    let conn = open_memory();
    let t = start(&conn, "", 1).unwrap();
    conn.execute("UPDATE timers SET ends_at = ends_at - 600000", [])
        .unwrap();
    add_minute(&conn, &t.id).unwrap();
    let ends = list(&conn).unwrap()[0].ends_at;
    assert!(ends >= now_ms() + MINUTE_MS - 1000);
}

#[test]
fn overdue_timers_show_once_after_launch() {
    let mut conn = open_memory();
    let gone = start(&conn, "Tea", 1000).unwrap();
    start(&conn, "Later", 600_000).unwrap();
    conn.execute("UPDATE timers SET ends_at = 1 WHERE id = ?1", [&gone.id])
        .unwrap();
    let done = finished_while_closed(&mut conn).unwrap();
    assert_eq!(
        done.iter().map(|t| t.title.as_str()).collect::<Vec<_>>(),
        vec!["Tea"]
    );
    assert!(finished_while_closed(&mut conn).unwrap().is_empty());
    assert_eq!(list(&conn).unwrap().len(), 1);
}

#[test]
fn a_timer_that_isnt_there_is_not_found() {
    let conn = open_memory();
    assert!(add_minute(&conn, "nope").unwrap_err().is(errors::NOT_FOUND));
    assert!(cancel(&conn, "nope").unwrap_err().is(errors::NOT_FOUND));
}
