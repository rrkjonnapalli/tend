use super::*;
use crate::db::open_memory;

fn new(title: &str, kind: Option<&str>) -> NewWord {
    NewWord {
        title: title.into(),
        kind: kind.map(String::from),
        ..Default::default()
    }
}

#[test]
fn a_word_title_is_saved_in_lowercase_other_kinds_as_typed() {
    let conn = open_memory();
    let word = create(&conn, &new("  Ephemeral ", None)).unwrap();
    assert_eq!(
        (word.title.as_str(), word.kind.as_str()),
        ("ephemeral", WORD)
    );
    let abbr = create(&conn, &new("IT", Some("ABBR"))).unwrap();
    assert_eq!(abbr.title, "IT");
    let it = create(&conn, &new("It", None)).unwrap();
    assert_eq!(it.title, "it", "`it` and `IT` are different entries");
}

#[test]
fn titles_are_unique_by_exact_match() {
    let conn = open_memory();
    create(&conn, &new("ephemeral", None)).unwrap();
    let again = create(&conn, &new("EPHEMERAL", None)).unwrap_err();
    assert!(again.is(errors::WORD_TAKEN), "lowercased first, then taken");
    assert!(create(&conn, &new(" ", None))
        .unwrap_err()
        .is(errors::WORD_TITLE_EMPTY));
    assert!(create(&conn, &new("x", Some("SLANG")))
        .unwrap_err()
        .is(errors::WORD_KIND_UNKNOWN));
}

#[test]
fn moving_to_word_lowercases_and_refuses_a_clash() {
    let conn = open_memory();
    create(&conn, &new("nasa", None)).unwrap();
    let abbr = create(&conn, &new("NASA", Some("ABBR"))).unwrap();
    let to_word = Patch {
        kind: Some(WORD.into()),
        ..Default::default()
    };
    assert!(update(&conn, &abbr.id, &to_word)
        .unwrap_err()
        .is(errors::WORD_TAKEN));
    let other = create(&conn, &new("Petrichor", Some("TERM"))).unwrap();
    let moved = update(&conn, &other.id, &to_word).unwrap();
    assert_eq!(moved.title, "petrichor");
}

#[test]
fn an_edit_keeps_what_it_leaves_out_and_the_list_carries_tags() {
    let mut conn = open_memory();
    let dictionary = serde_json::json!({ "senses": [{ "def": "Lasting a short time." }] });
    let word = create(
        &conn,
        &NewWord {
            title: "ephemeral".into(),
            dictionary: Some(dictionary.clone()),
            meaning: "brief".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let patch = Patch {
        notes: Some("- seen in a book".into()),
        ..Default::default()
    };
    let edited = update(&conn, &word.id, &patch).unwrap();
    assert_eq!(edited.meaning, "brief");
    assert_eq!(edited.dictionary, Some(dictionary));
    let cleared = Patch {
        dictionary: Some(None),
        ..Default::default()
    };
    assert_eq!(update(&conn, &word.id, &cleared).unwrap().dictionary, None);
    let source = crate::links::Source {
        kind: SourceType::Word,
        id: &word.id,
    };
    crate::tags::set(&mut conn, source, &["Reading".into()]).unwrap();
    assert_eq!(list(&conn).unwrap()[0].tags, vec!["reading"]);
    delete(&conn, &word.id).unwrap();
    let links: i64 = conn
        .query_row("SELECT count(*) FROM tend_links", [], |r| r.get(0))
        .unwrap();
    assert_eq!(links, 0);
}

#[test]
fn kinds_come_in_order_with_their_lookup() {
    let conn = open_memory();
    let kinds = kinds(&conn).unwrap();
    let values: Vec<&str> = kinds.iter().map(|k| k.value.as_str()).collect();
    assert_eq!(
        values,
        ["WORD", "IDIOM", "ABBR", "CONCEPT", "PHRASE", "TERM", "OTHER"]
    );
    let lookups: Vec<bool> = kinds.iter().map(|k| k.lookup).collect();
    assert_eq!(lookups, [true, true, false, false, true, false, false]);
}

#[test]
fn kinds_can_be_added_renamed_ordered_and_deleted_when_unused() {
    let mut conn = open_memory();
    let slang = kind_add(
        &conn,
        &KindNamed {
            value: "SLANG",
            label: " Slang ",
        },
    )
    .unwrap();
    assert_eq!(
        (slang.label.as_str(), slang.position, slang.lookup),
        ("Slang", 7, false)
    );
    assert!(kind_add(
        &conn,
        &KindNamed {
            value: "SLANG_2",
            label: "slang"
        }
    )
    .unwrap_err()
    .is(errors::WORD_KIND_LABEL_TAKEN));
    assert!(kind_add(
        &conn,
        &KindNamed {
            value: "SLANG",
            label: "Other slang"
        }
    )
    .unwrap_err()
    .is(errors::WORD_KIND_VALUE_TAKEN));
    assert!(kind_add(
        &conn,
        &KindNamed {
            value: "odd value",
            label: "Odd"
        }
    )
    .unwrap_err()
    .is(errors::WORD_KIND_VALUE));
    assert_eq!(
        kind_rename(
            &conn,
            &KindNamed {
                value: "SLANG",
                label: "Street"
            }
        )
        .unwrap()
        .label,
        "Street"
    );

    let mut order: Vec<String> = kinds(&conn).unwrap().into_iter().map(|k| k.value).collect();
    order.rotate_right(1);
    kind_reorder(&mut conn, &order).unwrap();
    assert_eq!(kinds(&conn).unwrap()[0].value, "SLANG");
    assert!(kind_reorder(&mut conn, &order[1..])
        .unwrap_err()
        .is(errors::WORD_KIND_ORDER));

    let word = create(&conn, &new("Gnarly", Some("SLANG"))).unwrap();
    assert!(kind_delete(&conn, "SLANG")
        .unwrap_err()
        .is(errors::WORD_KIND_IN_USE));
    assert!(kind_delete(&conn, WORD)
        .unwrap_err()
        .is(errors::WORD_KIND_DEFAULT));
    delete(&conn, &word.id).unwrap();
    kind_delete(&conn, "SLANG").unwrap();
    assert_eq!(kinds(&conn).unwrap().len(), 7);
}

#[test]
fn a_blocked_kind_cant_be_given_to_a_word() {
    let conn = open_memory();
    let word = create(&conn, &new("ephemeral", None)).unwrap();
    conn.execute(
        "UPDATE tend_enums SET status = 'BLOCKED' WHERE kind = 'WORD_KIND' AND value = 'ABBR'",
        [],
    )
    .unwrap();
    let refused = create(&conn, &new("IT", Some("ABBR"))).unwrap_err();
    assert!(refused.is(errors::ENUM_BLOCKED));
    let patch = Patch {
        kind: Some("ABBR".into()),
        ..Default::default()
    };
    assert!(update(&conn, &word.id, &patch)
        .unwrap_err()
        .is(errors::ENUM_BLOCKED));
}

#[test]
fn a_word_or_kind_that_isnt_there_is_not_found() {
    let conn = open_memory();
    let notes = Patch {
        notes: Some("x".into()),
        ..Default::default()
    };
    let title = Patch {
        title: Some("x".into()),
        ..Default::default()
    };
    assert!(update(&conn, "nope", &notes)
        .unwrap_err()
        .is(errors::NOT_FOUND));
    assert!(update(&conn, "nope", &title)
        .unwrap_err()
        .is(errors::NOT_FOUND));
    assert!(delete(&conn, "nope").unwrap_err().is(errors::NOT_FOUND));
    let kind = KindNamed {
        value: "NOPE",
        label: "Nope",
    };
    assert!(kind_rename(&conn, &kind).unwrap_err().is(errors::NOT_FOUND));
    assert!(kind_delete(&conn, "NOPE")
        .unwrap_err()
        .is(errors::NOT_FOUND));
}
