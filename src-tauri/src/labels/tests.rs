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
    let script = add(
        &conn,
        &NewLabel {
            name: "script",
            value: "SCRIPT",
            color: "#1259d3",
        },
    )
    .unwrap();
    let review = add(
        &conn,
        &NewLabel {
            name: " review in progress ",
            value: "REVIEW",
            color: "#d57149",
        },
    )
    .unwrap();
    assert_eq!(review.name, "review in progress");
    assert_eq!(
        list(&conn)
            .unwrap()
            .iter()
            .map(|l| l.name.as_str())
            .collect::<Vec<_>>(),
        ["script", "review in progress"]
    );

    assert!(add(
        &conn,
        &NewLabel {
            name: "Script",
            value: "OTHER",
            color: "#000000"
        }
    )
    .unwrap_err()
    .is(errors::LABEL_NAME_TAKEN));
    assert!(add(
        &conn,
        &NewLabel {
            name: "  ",
            value: "BLANK",
            color: "#000000"
        }
    )
    .unwrap_err()
    .is(errors::LABEL_NAME_EMPTY));
    assert!(add(
        &conn,
        &NewLabel {
            name: "x",
            value: "X",
            color: "blue"
        }
    )
    .unwrap_err()
    .is(errors::COLOR_INVALID));
    assert!(rename(
        &conn,
        &Rename {
            id: &review.id,
            name: "SCRIPT"
        }
    )
    .unwrap_err()
    .is(errors::LABEL_NAME_TAKEN));

    assert_eq!(
        rename(
            &conn,
            &Rename {
                id: &script.id,
                name: "scripts"
            }
        )
        .unwrap()
        .name,
        "scripts"
    );
    assert_eq!(
        recolor(
            &conn,
            &Recolor {
                id: &script.id,
                color: "#1e7a55"
            }
        )
        .unwrap()
        .color,
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
    let a = add(
        &conn,
        &NewLabel {
            name: "script",
            value: "SCRIPT",
            color: "#1259d3",
        },
    )
    .unwrap();
    let b = add(
        &conn,
        &NewLabel {
            name: "pending approval",
            value: "PENDING_APPROVAL",
            color: "#d57149",
        },
    )
    .unwrap();
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
fn a_label_value_is_its_id_and_never_changes() {
    let conn = open_memory();
    let pr = add(
        &conn,
        &NewLabel {
            name: "PR review",
            value: "PR_REVIEW",
            color: "#f67280",
        },
    )
    .unwrap();
    assert_eq!(pr.id, "PR_REVIEW");
    // Another name, same value: the user is asked to change the value, not given a suffix.
    assert!(add(
        &conn,
        &NewLabel {
            name: "PR reviews",
            value: "PR_REVIEW",
            color: "#f67280"
        }
    )
    .unwrap_err()
    .is(errors::LABEL_VALUE_TAKEN));
    assert!(add(
        &conn,
        &NewLabel {
            name: "C#",
            value: "C_SHARP",
            color: "#f67280"
        }
    )
    .is_ok());
    for bad in ["db script", "db_script", "_X", "X__Y", ""] {
        assert!(
            add(
                &conn,
                &NewLabel {
                    name: "Odd",
                    value: bad,
                    color: "#f67280"
                }
            )
            .unwrap_err()
            .is(errors::LABEL_VALUE),
            "{bad}"
        );
    }
    let renamed = rename(
        &conn,
        &Rename {
            id: "PR_REVIEW",
            name: "Code review",
        },
    )
    .unwrap();
    assert_eq!(
        (renamed.id.as_str(), renamed.name.as_str()),
        ("PR_REVIEW", "Code review")
    );
}

#[test]
fn a_blocked_label_cant_be_put_on_a_task() {
    let mut conn = open_memory();
    let t = task(&mut conn);
    let new = NewLabel {
        name: "script",
        value: "SCRIPT",
        color: "#1259d3",
    };
    add(&conn, &new).unwrap();
    set_for_task(&mut conn, &t, &["SCRIPT".into()]).unwrap();
    in_transaction(&mut conn, |tx| {
        rows::set_status(tx, key("SCRIPT"), crate::enums::EnumStatus::Blocked)
    })
    .unwrap();
    assert!(of_task(&conn, &t).unwrap().is_empty());
    let refused = set_for_task(&mut conn, &t, &["SCRIPT".into()]).unwrap_err();
    assert!(refused.is(errors::ENUM_BLOCKED));
}

#[test]
fn a_label_that_isnt_there_is_not_found() {
    let conn = open_memory();
    let rename = rename(
        &conn,
        &Rename {
            id: "NOPE",
            name: "Nope",
        },
    );
    let recolor = recolor(
        &conn,
        &Recolor {
            id: "NOPE",
            color: "#1259d3",
        },
    );
    assert!(rename.unwrap_err().is(errors::NOT_FOUND));
    assert!(recolor.unwrap_err().is(errors::NOT_FOUND));
    assert!(delete(&conn, "NOPE").unwrap_err().is(errors::NOT_FOUND));
}
