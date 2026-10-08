use super::*;
use crate::db::open_memory;

#[test]
fn set_reuses_tags_case_insensitively() {
    let mut conn = open_memory();
    crate::notes::seed(&conn, "n", "notes/n.md");
    let tags = set(
        &mut conn,
        Owner::Note,
        "n",
        &["UI".into(), "tauri".into(), " ".into(), "ui".into()],
    )
    .unwrap();
    assert_eq!(tags, vec!["tauri", "UI"]);
    assert_eq!(all(&conn).unwrap(), vec!["tauri", "UI"]);
    let tags = set(&mut conn, Owner::Note, "n", &["Tauri".into()]).unwrap();
    assert_eq!(tags, vec!["tauri"], "existing tag keeps its spelling");
}
