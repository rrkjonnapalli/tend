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
    let script = add(&conn, "script", "SCRIPT", "#1259d3").unwrap();
    let review = add(&conn, " review in progress ", "REVIEW", "#d57149").unwrap();
    assert_eq!(review.name, "review in progress");
    assert_eq!(
        list(&conn)
            .unwrap()
            .iter()
            .map(|l| l.name.as_str())
            .collect::<Vec<_>>(),
        ["script", "review in progress"]
    );

    assert!(add(&conn, "Script", "OTHER", "#000000")
        .unwrap_err()
        .is(errors::LABEL_NAME_TAKEN));
    assert!(add(&conn, "  ", "BLANK", "#000000")
        .unwrap_err()
        .is(errors::LABEL_NAME_EMPTY));
    assert!(add(&conn, "x", "X", "blue")
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
    let a = add(&conn, "script", "SCRIPT", "#1259d3").unwrap();
    let b = add(&conn, "pending approval", "PENDING_APPROVAL", "#d57149").unwrap();
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
fn a_label_value_is_its_id_in_capitals_and_never_changes() {
    let conn = open_memory();
    let pr = add(&conn, "PR review", "PR_REVIEW", "#f67280").unwrap();
    assert_eq!(pr.id, "pr_review");
    // Another name, same value: the user is asked to change the value, not given a suffix.
    assert!(add(&conn, "PR reviews", "PR_REVIEW", "#f67280")
        .unwrap_err()
        .is(errors::LABEL_VALUE_TAKEN));
    assert!(add(&conn, "C#", "C_SHARP", "#f67280").is_ok());
    for bad in ["db script", "db_script", "_X", "X__Y", ""] {
        assert!(
            add(&conn, "Odd", bad, "#f67280")
                .unwrap_err()
                .is(errors::LABEL_VALUE),
            "{bad}"
        );
    }
    let renamed = rename(&conn, "pr_review", "Code review").unwrap();
    assert_eq!(
        (renamed.id.as_str(), renamed.name.as_str()),
        ("pr_review", "Code review")
    );
}
