use super::*;
use crate::db::open_memory;

fn patch(json: &str) -> Patch {
    serde_json::from_str(json).unwrap()
}

#[test]
fn create_update_and_list() {
    let mut conn = open_memory();
    let id = create(&conn, "Ravi", "Kumar").unwrap();
    assert!(create(&conn, "  ", "x").is_err());
    let p = update(
        &mut conn,
        &id,
        &patch(r#"{"nickname":"Ravs","dob":{"year":1995,"month":10,"day":9},"tob":"06:30"}"#),
    )
    .unwrap();
    assert_eq!(p.row.nickname.as_deref(), Some("Ravs"));
    assert_eq!(
        (p.row.dob_year, p.row.dob_month, p.row.dob_day),
        (Some(1995), Some(10), Some(9))
    );
    assert_eq!(p.tob.as_deref(), Some("06:30"));
    let p = update(
        &mut conn,
        &id,
        &patch(r#"{"dob":{"year":null,"month":2,"day":29},"nickname":"","tob":null}"#),
    )
    .unwrap();
    assert_eq!(
        (p.row.dob_year, p.row.dob_month, p.row.dob_day),
        (None, Some(2), Some(29))
    );
    assert_eq!((p.row.nickname, p.tob), (None, None));
}

#[test]
fn invalid_dob_and_time_are_rejected_and_not_saved() {
    let mut conn = open_memory();
    let id = create(&conn, "A", "").unwrap();
    for bad in [
        r#"{"dob":{"year":2023,"month":2,"day":29}}"#,
        r#"{"dob":{"year":null,"month":13,"day":1}}"#,
        r#"{"dob":{"year":null,"month":4,"day":31}}"#,
        r#"{"tob":"25:00"}"#,
        r#"{"tob":"6:30"}"#,
    ] {
        assert!(
            update(&mut conn, &id, &patch(bad)).is_err(),
            "{bad} should fail"
        );
    }
    let p = get(&conn, &id).unwrap();
    assert_eq!((p.row.dob_month, p.tob), (None, None));
}

#[test]
fn contacts_keep_order_and_feed_the_list() {
    let mut conn = open_memory();
    let id = create(&conn, "A", "B").unwrap();
    contact_add(&conn, &id, "phone", "+91 1", "personal").unwrap();
    contact_add(&conn, &id, "phone", "+91 2", "work").unwrap();
    contact_add(&conn, &id, "email", "a@b.c", "").unwrap();
    assert!(contact_add(&conn, &id, "fax", "1", "").is_err());
    assert!(contact_add(&conn, &id, "email", " ", "").is_err());
    let row = &list(&conn).unwrap()[0];
    assert_eq!(
        (row.phone.as_deref(), row.email.as_deref()),
        (Some("+91 1"), Some("a@b.c"))
    );
    let p = get(&conn, &id).unwrap();
    let first = p
        .contacts
        .iter()
        .find(|c| c.value == "+91 1")
        .unwrap()
        .id
        .clone();
    contact_update(&conn, &first, "+91 9", "home").unwrap();
    contact_delete(
        &conn,
        &p.contacts.iter().find(|c| c.value == "+91 2").unwrap().id,
    )
    .unwrap();
    let p = get(&conn, &id).unwrap();
    let phones: Vec<_> = p
        .contacts
        .iter()
        .filter(|c| c.kind == "phone")
        .map(|c| (c.value.as_str(), c.label.as_str()))
        .collect();
    assert_eq!(phones, vec![("+91 9", "home")]);
    let t = crate::tasks::create(
        &mut conn,
        &crate::tasks::NewTask {
            title: "T".into(),
            ..Default::default()
        },
    )
    .unwrap();
    crate::tasks::update(
        &mut conn,
        &t.id,
        &serde_json::from_str(&format!(r#"{{"assigneeId":"{id}"}}"#)).unwrap(),
    )
    .unwrap();
    delete(&conn, &id).unwrap();
    assert!(crate::tasks::get(&conn, &t.id)
        .unwrap()
        .assignee_id
        .is_none());
}
