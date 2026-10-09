use super::*;
use crate::db::open_memory;
use crate::tasks::{create, NewTask};

fn task(conn: &mut Connection) -> String {
    create(
        conn,
        &NewTask {
            title: "T".into(),
            ..Default::default()
        },
    )
    .unwrap()
    .id
}

#[test]
fn add_rename_recolor_and_order() {
    let mut conn = open_memory();
    let script = add(&conn, "script", "#1259d3").unwrap();
    let review = add(&conn, " review in progress ", "#d57149").unwrap();
    assert_eq!(review.name, "review in progress");
    assert_eq!(
        list(&conn)
            .unwrap()
            .iter()
            .map(|l| l.name.as_str())
            .collect::<Vec<_>>(),
        ["script", "review in progress"]
    );

    assert!(add(&conn, "Script", "#000000")
        .unwrap_err()
        .is(errors::LABEL_NAME_TAKEN));
    assert!(add(&conn, "  ", "#000000")
        .unwrap_err()
        .is(errors::LABEL_NAME_EMPTY));
    assert!(add(&conn, "x", "blue")
        .unwrap_err()
        .is(errors::COLOR_INVALID));
    assert!(rename(&conn, &review.id, "SCRIPT")
        .unwrap_err()
        .is(errors::LABEL_NAME_TAKEN));

    assert_eq!(
        rename(&conn, &script.id, "scripts").unwrap().name,
        "scripts"
    );
    assert_eq!(
        recolor(&conn, &script.id, "#1e7a55").unwrap().color,
        "#1e7a55"
    );

    reorder(&mut conn, &[review.id.clone(), script.id.clone()]).unwrap();
    assert_eq!(list(&conn).unwrap()[0].id, review.id);
    assert!(reorder(&mut conn, std::slice::from_ref(&review.id))
        .unwrap_err()
        .is(errors::LABEL_ORDER));
}

#[test]
fn tasks_carry_labels_until_the_label_is_deleted() {
    let mut conn = open_memory();
    let a = add(&conn, "script", "#1259d3").unwrap();
    let b = add(&conn, "pending approval", "#d57149").unwrap();
    let t = task(&mut conn);

    let set = set_for_task(&mut conn, &t, &[b.id.clone(), a.id.clone()]).unwrap();
    assert_eq!(set, vec![a.id.clone(), b.id.clone()], "in the list's order");
    assert_eq!(usage(&conn, &a.id).unwrap(), 1);
    assert!(set_for_task(&mut conn, &t, &["nope".into()])
        .unwrap_err()
        .is(errors::NOT_FOUND));

    delete(&conn, &a.id).unwrap();
    assert_eq!(of_task(&conn, &t).unwrap(), vec![b.id.clone()]);
    assert_eq!(links(&conn).unwrap(), vec![(t, b.id)]);
}

#[test]
fn a_label_id_is_its_first_names_slug_and_never_changes() {
    let conn = open_memory();
    let pr = add(&conn, "PR review", "#f67280").unwrap();
    assert_eq!(pr.id, "pr_review");
    assert!(add(&conn, "pr-review", "#f67280")
        .unwrap_err()
        .is(errors::LABEL_NAME_TAKEN));
    assert!(add(&conn, "!!", "#f67280")
        .unwrap_err()
        .is(errors::NAME_CHARS));
    let renamed = rename(&conn, "pr_review", "Code review").unwrap();
    assert_eq!(
        (renamed.id.as_str(), renamed.name.as_str()),
        ("pr_review", "Code review")
    );
}
