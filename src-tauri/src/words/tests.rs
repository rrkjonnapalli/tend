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
    let mut conn = open_memory();
    create(&conn, &new("nasa", None)).unwrap();
    let abbr = create(&conn, &new("NASA", Some("ABBR"))).unwrap();
    let to_word = Patch {
        kind: Some(WORD.into()),
        ..Default::default()
    };
    assert!(update(&mut conn, &abbr.id, &to_word)
        .unwrap_err()
        .is(errors::WORD_TAKEN));
    let other = create(&conn, &new("Petrichor", Some("TERM"))).unwrap();
    let moved = update(&mut conn, &other.id, &to_word).unwrap();
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
    let edited = update(&mut conn, &word.id, &patch).unwrap();
    assert_eq!(edited.meaning, "brief");
    assert_eq!(edited.dictionary, Some(dictionary));
    let cleared = Patch {
        dictionary: Some(None),
        ..Default::default()
    };
    assert_eq!(
        update(&mut conn, &word.id, &cleared).unwrap().dictionary,
        None
    );
    crate::tags::set(&mut conn, SourceType::Word, &word.id, &["Reading".into()]).unwrap();
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
    assert_eq!(values, ["WORD", "IDIOM", "PHRASE", "ABBR", "TERM", "OTHER"]);
    let lookups: Vec<bool> = kinds.iter().map(|k| k.lookup).collect();
    assert_eq!(lookups, [true, true, true, false, false, false]);
}
