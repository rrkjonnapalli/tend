use super::*;
use crate::db::open_memory;

fn link(value: &str) -> NewRef {
    NewRef {
        kind: RefKind::Link,
        value: value.into(),
        title: None,
        why: String::new(),
    }
}

#[test]
fn url_keys_ignore_scheme_trailing_slash_and_case() {
    for same in [
        "https://Svelte.dev/blog/runes/",
        "http://svelte.dev/blog/runes",
        "svelte.dev/blog/runes",
        "  SVELTE.DEV/BLOG/RUNES  ",
    ] {
        assert_eq!(url_key(same), "svelte.dev/blog/runes", "{same}");
    }
    assert_ne!(url_key("www.svelte.dev"), url_key("svelte.dev"));
}

#[test]
fn a_link_can_be_saved_once_text_any_number_of_times() {
    let mut conn = open_memory();
    let first = create(&conn, &link("https://svelte.dev/blog/runes")).unwrap();
    assert_eq!(first.url_key.as_deref(), Some("svelte.dev/blog/runes"));
    assert!(create(&conn, &link("svelte.dev/blog/runes/"))
        .unwrap_err()
        .is(errors::REF_LINK_TAKEN));
    let film = NewRef {
        kind: RefKind::Text,
        value: "Perfect Days".into(),
        title: None,
        why: String::new(),
    };
    create(&conn, &film).unwrap();
    let again = create(&conn, &film).unwrap();
    assert_eq!(again.url_key, None);
    let to_link = Patch {
        kind: Some(RefKind::Link),
        value: Some("https://svelte.dev/blog/runes".into()),
        ..Default::default()
    };
    assert!(update(&mut conn, &again.id, &to_link)
        .unwrap_err()
        .is(errors::REF_LINK_TAKEN));
    assert!(create(&conn, &link("   "))
        .unwrap_err()
        .is(errors::REF_VALUE_EMPTY));
}

#[test]
fn titles_are_trimmed_and_blank_means_none() {
    let mut conn = open_memory();
    let r = create(
        &conn,
        &NewRef {
            title: Some("  What are runes? ".into()),
            ..link("svelte.dev")
        },
    )
    .unwrap();
    assert_eq!(r.title.as_deref(), Some("What are runes?"));
    let blank = Patch {
        title: Some(Some(" ".into())),
        why: Some("$state in one page".into()),
        ..Default::default()
    };
    let r = update(&mut conn, &r.id, &blank).unwrap();
    assert_eq!((r.title, r.why.as_str()), (None, "$state in one page"));
}

#[test]
fn the_list_is_newest_first() {
    let conn = open_memory();
    let old = create(&conn, &link("a.dev")).unwrap();
    conn.execute("UPDATE refs SET created_at = 1 WHERE id = ?1", [&old.id])
        .unwrap();
    let new = create(&conn, &link("b.dev")).unwrap();
    let ids: Vec<String> = list(&conn).unwrap().into_iter().map(|r| r.id).collect();
    assert_eq!(ids, vec![new.id, old.id]);
}
