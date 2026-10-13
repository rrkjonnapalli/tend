use super::*;
use crate::db::open_memory;

fn note(conn: &Connection, text: &str, stack: Option<&str>) -> Snap {
    let new = NewSnap {
        text: text.into(),
        stack_id: stack.map(String::from),
        paper: Paper::Scrap,
        color: None,
        pinned: false,
    };
    create(conn, &new).unwrap()
}

#[test]
fn text_is_kept_as_written_and_only_stickies_keep_a_colour() {
    let conn = open_memory();
    let snap = note(&conn, "  late train —\n", None);
    assert_eq!(snap.text, "  late train —\n");
    assert!(create(
        &conn,
        &NewSnap {
            text: " \n".into(),
            stack_id: None,
            paper: Paper::Scrap,
            color: None,
            pinned: false
        }
    )
    .unwrap_err()
    .is(errors::SNAP_TEXT_EMPTY));
    let sticky = Patch {
        paper: Some(Paper::Sticky),
        color: Some(Some("#d4b23b".into())),
        ..Default::default()
    };
    let snap = update(&conn, &snap.id, &sticky).unwrap();
    assert_eq!(snap.color.as_deref(), Some("#d4b23b"));
    let slip = Patch {
        paper: Some(Paper::Slip),
        ..Default::default()
    };
    assert_eq!(update(&conn, &snap.id, &slip).unwrap().color, None);
    let bad = Patch {
        paper: Some(Paper::Sticky),
        color: Some(Some("yellow".into())),
        ..Default::default()
    };
    assert!(update(&conn, &snap.id, &bad)
        .unwrap_err()
        .is(errors::COLOR_INVALID));
}

#[test]
fn the_board_leads_with_pinned_then_the_latest_edit() {
    let conn = open_memory();
    let first = note(&conn, "first", None);
    let second = note(&conn, "second", None);
    let pin = Patch {
        pinned: Some(true),
        ..Default::default()
    };
    update(&conn, &first.id, &pin).unwrap();
    conn.execute(
        "UPDATE snaps SET updated_at = 1 WHERE id = ?1",
        [&second.id],
    )
    .unwrap();
    let third = note(&conn, "third", None);
    let order: Vec<String> = board(&conn)
        .unwrap()
        .snaps
        .into_iter()
        .map(|s| s.id)
        .collect();
    assert_eq!(order, vec![first.id, third.id, second.id]);
}

#[test]
fn a_stack_cant_go_inside_itself_or_its_sub_stacks() {
    let conn = open_memory();
    let work = stack_create(
        &conn,
        &NewStack {
            parent: None,
            name: "Work rules",
        },
    )
    .unwrap();
    let reviews = stack_create(
        &conn,
        &NewStack {
            parent: Some(&work.id),
            name: " Code reviews ",
        },
    )
    .unwrap();
    assert_eq!(reviews.name, "Code reviews");
    let deep = stack_create(
        &conn,
        &NewStack {
            parent: Some(&reviews.id),
            name: "",
        },
    )
    .unwrap();
    for target in [&work.id, &reviews.id, &deep.id] {
        assert!(stack_move(
            &conn,
            &StackMove {
                id: &work.id,
                parent: Some(target)
            }
        )
        .unwrap_err()
        .is(errors::STACK_INTO_SELF));
    }
    let out = stack_move(
        &conn,
        &StackMove {
            id: &deep.id,
            parent: None,
        },
    )
    .unwrap();
    assert_eq!(out.parent_id, None);
    assert_eq!(
        stack_rename(
            &conn,
            &StackRename {
                id: &deep.id,
                name: "Monsoon"
            }
        )
        .unwrap()
        .name,
        "Monsoon"
    );
}

#[test]
fn removing_a_stack_moves_its_contents_up_one_level() {
    let mut conn = open_memory();
    let haiku = stack_create(
        &conn,
        &NewStack {
            parent: None,
            name: "Haiku",
        },
    )
    .unwrap();
    let monsoon = stack_create(
        &conn,
        &NewStack {
            parent: Some(&haiku.id),
            name: "Monsoon",
        },
    )
    .unwrap();
    let rain = stack_create(
        &conn,
        &NewStack {
            parent: Some(&monsoon.id),
            name: "Rain",
        },
    )
    .unwrap();
    let snap = note(&conn, "first rain", Some(&monsoon.id));
    stack_remove(&mut conn, &monsoon.id).unwrap();
    let moved = board(&conn).unwrap().snaps;
    assert_eq!(moved[0].id, snap.id);
    assert_eq!(moved[0].stack_id, Some(haiku.id.clone()));
    let stacks = board(&conn).unwrap().stacks;
    let rain = stacks.iter().find(|s| s.id == rain.id).unwrap();
    assert_eq!(rain.parent_id, Some(haiku.id));
    assert_eq!(stacks.len(), 2);
}

#[test]
fn moving_a_snap_needs_a_real_stack() {
    let conn = open_memory();
    let snap = note(&conn, "x", None);
    let into = |stack: Option<&str>| Patch {
        stack_id: Some(stack.map(String::from)),
        ..Default::default()
    };
    assert!(update(&conn, &snap.id, &into(Some("nope")))
        .unwrap_err()
        .is(errors::NOT_FOUND));
    let s = stack_create(
        &conn,
        &NewStack {
            parent: None,
            name: "S",
        },
    )
    .unwrap();
    assert_eq!(
        update(&conn, &snap.id, &into(Some(&s.id)))
            .unwrap()
            .stack_id,
        Some(s.id.clone())
    );
    assert_eq!(update(&conn, &snap.id, &into(None)).unwrap().stack_id, None);
}

#[test]
fn a_snap_or_stack_that_isnt_there_is_not_found() {
    let mut conn = open_memory();
    let not_found = |e: crate::error::AppError| assert!(e.is(errors::NOT_FOUND), "{e}");
    let pin = Patch {
        pinned: Some(true),
        ..Default::default()
    };
    let paper = Patch {
        paper: Some(Paper::Lined),
        ..Default::default()
    };
    not_found(update(&conn, "nope", &pin).unwrap_err());
    not_found(update(&conn, "nope", &paper).unwrap_err());
    not_found(delete(&conn, "nope").unwrap_err());
    let rename = StackRename {
        id: "nope",
        name: "x",
    };
    let to_board = StackMove {
        id: "nope",
        parent: None,
    };
    not_found(stack_rename(&conn, &rename).unwrap_err());
    not_found(stack_move(&conn, &to_board).unwrap_err());
    not_found(stack_remove(&mut conn, "nope").unwrap_err());
}
