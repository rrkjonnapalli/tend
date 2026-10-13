use super::*;
use crate::db::open_memory;

fn task_owner(id: &str) -> Owner<'_> {
    Owner { kind: "TASK", id }
}

/// Copies `src` in and links it, as the `attachments_add` command does.
fn add_file(
    conn: &mut Connection,
    root: &Path,
    owner: &str,
    owner_id: &str,
    src: &Path,
) -> AppResult<()> {
    let limit = crate::files::Limit::DEFAULT;
    let file = copy_in(&Incoming { root, src, limit })?;
    link_copy(
        conn,
        &Copied {
            root,
            file: &file,
            owner,
            owner_id,
        },
    )
}

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

    add_file(&mut conn, &root, "TASK", &t.id, &src).unwrap();
    let links = list(&conn, &task_owner(&t.id)).unwrap();
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
    assert!(list(&conn, &task_owner(&t.id)).unwrap().is_empty());
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
    crate::notes::seed(&conn, "n", "work/q4_plan.md");
    link_note(
        &conn,
        &NoteLink {
            owner: "TASK",
            owner_id: &t.id,
            note_id: "n",
        },
    )
    .unwrap();
    link_note(
        &conn,
        &NoteLink {
            owner: "TASK",
            owner_id: &t.id,
            note_id: "n",
        },
    )
    .unwrap();
    let links = list(&conn, &task_owner(&t.id)).unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(
        (links[0].name.as_str(), links[0].folder.as_deref()),
        ("q4_plan", Some("work"))
    );
}

#[test]
fn a_file_of_30_mb_or_more_or_a_folder_is_refused_before_anything_is_copied() {
    let tmp = tempfile::tempdir().unwrap();
    let big = tmp.path().join("big.mov");
    fs::File::create(&big)
        .unwrap()
        .set_len(crate::files::MAX_FILE_BYTES)
        .unwrap();
    assert!(check_file(&big, crate::files::Limit::DEFAULT)
        .unwrap_err()
        .is(errors::FILE_TOO_BIG));
    assert!(check_file(tmp.path(), crate::files::Limit::DEFAULT).is_err());
}

#[test]
fn only_paths_inside_the_vault_are_opened() {
    let conn = crate::db::open_memory();
    conn.execute_batch(
        "INSERT INTO attachments (id, path, name, mime_type, size_bytes, sha256, created_at, updated_at)
         VALUES ('bad', '/System/Applications/Calculator.app', 'x', 'x', 0, 'x', 0, 0);",
    )
    .unwrap();
    assert!(file_path(&conn, Path::new("/v"), "bad").is_err());
}
