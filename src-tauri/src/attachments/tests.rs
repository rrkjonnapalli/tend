use super::*;
use crate::db::open_memory;

#[test]
fn files_are_copied_hashed_and_linked() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("vault");
    let src = tmp.path().join("Invoice March.PDF");
    fs::write(&src, b"hello").unwrap();
    let mut conn = open_memory();
    let t = crate::tasks::create(
        &mut conn,
        &crate::tasks::NewTask {
            title: "T".into(),
            ..Default::default()
        },
    )
    .unwrap();

    add_file(&mut conn, &root, "task", &t.id, &src).unwrap();
    let links = list(&conn, "task", &t.id).unwrap();
    assert_eq!(links.len(), 1);
    let l = &links[0];
    assert_eq!(
        (l.kind.as_str(), l.name.as_str(), l.size_bytes),
        ("file", "Invoice March.PDF", Some(5))
    );
    assert_eq!(l.mime_type.as_deref(), Some("application/pdf"));
    let path = file_path(&conn, &root, l.attachment_id.as_deref().unwrap()).unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"hello");
    assert!(path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .ends_with(".pdf"));
    let hash: String = conn
        .query_row("SELECT sha256 FROM attachments", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        hash,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );

    unlink(&conn, &l.id).unwrap();
    assert!(list(&conn, "task", &t.id).unwrap().is_empty());
    assert!(path.exists(), "unlinking keeps the file");
    assert!(add_file(&mut conn, &root, "nope", &t.id, &src).is_err());
}

#[test]
fn notes_link_once() {
    let mut conn = open_memory();
    let t = crate::tasks::create(
        &mut conn,
        &crate::tasks::NewTask {
            title: "T".into(),
            ..Default::default()
        },
    )
    .unwrap();
    crate::notes::seed(&conn, "n", "notes/Work/Q4 plan.md");
    link_note(&conn, "task", &t.id, "n").unwrap();
    link_note(&conn, "task", &t.id, "n").unwrap();
    let links = list(&conn, "task", &t.id).unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(
        (links[0].name.as_str(), links[0].folder.as_deref()),
        ("Q4 plan", Some("Work"))
    );
}
