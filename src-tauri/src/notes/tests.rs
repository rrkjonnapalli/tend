use super::*;
use crate::db::open_memory;
use crate::error::errors;
use crate::util::now_ms;

struct Fixture {
    _tmp: tempfile::TempDir,
    dir: PathBuf,
    conn: Connection,
}

fn fixture(files: &[(&str, &str)]) -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("notes");
    fs::create_dir_all(&dir).unwrap();
    for (path, text) in files {
        let p = dir.join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
    let mut conn = open_memory();
    scan(&mut conn, &dir).unwrap();
    Fixture {
        _tmp: tmp,
        dir,
        conn,
    }
}

fn names_in(t: &Tree) -> Vec<String> {
    t.notes
        .iter()
        .map(|n| names::join(&n.folder, &n.name))
        .collect()
}

/// The index lists exactly what is on disk.
fn assert_matches_disk(f: &mut Fixture) {
    let before = tree(&f.conn).unwrap();
    scan(&mut f.conn, &f.dir).unwrap();
    let after = tree(&f.conn).unwrap();
    assert_eq!(before.folders, after.folders);
    assert_eq!(names_in(&before), names_in(&after));
}

#[test]
fn initial_scan_indexes_folders_and_notes() {
    let f = fixture(&[
        ("work/tauri/window_config.md", "# Window config\n\nbody"),
        ("work/release_checklist.md", "no heading"),
        ("scratch.md", ""),
    ]);
    let t = tree(&f.conn).unwrap();
    assert_eq!(t.folders, vec!["work", "work/tauri"]);
    assert_eq!(
        names_in(&t),
        vec![
            "scratch",
            "work/release_checklist",
            "work/tauri/window_config"
        ]
    );
    let wc = t.notes.iter().find(|n| n.name == "window_config").unwrap();
    assert_eq!(
        wc.title, "window_config",
        "a found file is titled by its name"
    );
}

#[test]
fn skips_invalid_names_without_touching_them() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    fs::create_dir_all(dir.join("My Folder")).unwrap();
    fs::write(dir.join("My Folder/a.md"), "x").unwrap();
    fs::write(dir.join("Bad Name.md"), "x").unwrap();
    fs::write(dir.join("image.png"), "x").unwrap();
    fs::write(dir.join(".DS_Store"), "x").unwrap();
    fs::write(dir.join("ok.md"), "x").unwrap();
    let mut conn = open_memory();
    let skipped = scan(&mut conn, &dir).unwrap();
    assert_eq!(skipped, vec!["Bad Name.md", "My Folder/", "image.png"]);
    assert_eq!(names_in(&tree(&conn).unwrap()), vec!["ok"]);
    assert!(dir.join("Bad Name.md").exists() && dir.join("My Folder/a.md").exists());
}

#[test]
fn external_rename_keeps_the_id() {
    let mut f = fixture(&[("a.md", "# A")]);
    let id = tree(&f.conn).unwrap().notes[0].id.clone();
    fs::create_dir_all(f.dir.join("x")).unwrap();
    fs::rename(f.dir.join("a.md"), f.dir.join("x/b.md")).unwrap();
    scan(&mut f.conn, &f.dir).unwrap();
    let t = tree(&f.conn).unwrap();
    assert_eq!(names_in(&t), vec!["x/b"]);
    assert_eq!(t.notes[0].id, id);
}

#[test]
fn external_edit_bumps_updated_at_but_keeps_the_title() {
    let mut f = fixture(&[("a.md", "# Old")]);
    let before = tree(&f.conn).unwrap().notes[0].clone();
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs::write(f.dir.join("a.md"), "# New title").unwrap();
    scan(&mut f.conn, &f.dir).unwrap();
    let after = tree(&f.conn).unwrap().notes[0].clone();
    assert_eq!(after.title, "a");
    assert!(after.modified_at > before.modified_at);
}

#[test]
fn touch_without_change_keeps_updated_at() {
    let mut f = fixture(&[("a.md", "same")]);
    let before = tree(&f.conn).unwrap().notes[0].modified_at;
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs::write(f.dir.join("a.md"), "same").unwrap();
    scan(&mut f.conn, &f.dir).unwrap();
    assert_eq!(tree(&f.conn).unwrap().notes[0].modified_at, before);
}

#[test]
fn external_delete_marks_missing_then_purges() {
    let mut f = fixture(&[("gone/a.md", "# A"), ("b.md", "# B")]);
    fs::remove_dir_all(f.dir.join("gone")).unwrap();
    scan(&mut f.conn, &f.dir).unwrap();
    assert_eq!(names_in(&tree(&f.conn).unwrap()), vec!["b"]);
    assert!(tree(&f.conn).unwrap().folders.is_empty());
    let rows: i64 = f
        .conn
        .query_row("SELECT count(*) FROM notes", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 2, "missing note stays during the grace period");

    let old = now_ms() - GRACE_MS - 1;
    f.conn
        .execute(
            "UPDATE attachments SET missing_at = ?1 WHERE missing_at IS NOT NULL",
            [old],
        )
        .unwrap();
    f.conn
        .execute(
            "UPDATE note_folders SET missing_at = ?1 WHERE missing_at IS NOT NULL",
            [old],
        )
        .unwrap();
    scan(&mut f.conn, &f.dir).unwrap();
    let rows: i64 = f
        .conn
        .query_row("SELECT count(*) FROM notes", [], |r| r.get(0))
        .unwrap();
    let folders: i64 = f
        .conn
        .query_row("SELECT count(*) FROM note_folders", [], |r| r.get(0))
        .unwrap();
    assert_eq!((rows, folders), (1, 1));
}

#[test]
fn create_write_and_read() {
    let mut f = fixture(&[]);
    let n = create(&mut f.conn, &f.dir, "work/ideas", " Release checklist! ").unwrap();
    assert_eq!(
        (n.title.as_str(), n.name.as_str()),
        ("Release checklist!", "release_checklist")
    );
    assert!(f.dir.join("work/ideas/release_checklist.md").is_file());
    let n = write(&f.conn, &f.dir, &n.id, "# Ship it\n\nnow").unwrap();
    assert_eq!(
        n.title, "Release checklist!",
        "the body never sets the title"
    );
    assert_eq!(
        read(&f.conn, &f.dir, &n.id).unwrap().text,
        "# Ship it\n\nnow"
    );
    assert_eq!(tree(&f.conn).unwrap().folders, vec!["work", "work/ideas"]);
    assert_matches_disk(&mut f);
}

#[test]
fn titles_pick_free_file_names() {
    let mut f = fixture(&[("work/a.md", "")]);
    let empty = create(&mut f.conn, &f.dir, "", "  ").unwrap_err();
    assert!(empty.is(errors::NOTE_TITLE_EMPTY));
    let name = |f: &mut Fixture, folder: &str, title: &str| {
        create(&mut f.conn, &f.dir, folder, title).unwrap().name
    };
    assert_eq!(
        name(&mut f, "", "Work"),
        "work_2",
        "a folder holds the name"
    );
    assert_eq!(name(&mut f, "work", "A"), "a_2");
    assert_eq!(name(&mut f, "work", "a"), "a_3");
    assert_eq!(name(&mut f, "", "!!!"), "note");
    assert!(folder_create(&mut f.conn, &f.dir, "work").is_err());
    assert!(folder_create(&mut f.conn, &f.dir, "Work2").is_err());
    assert!(folder_create(&mut f.conn, &f.dir, "a__b").is_err());
}

#[test]
fn rename_and_move_a_folder_with_children() {
    let mut f = fixture(&[
        ("work/tauri/a.md", "# A"),
        ("work/b.md", "# B"),
        ("home/c.md", "# C"),
    ]);
    let a = tree(&f.conn)
        .unwrap()
        .notes
        .iter()
        .find(|n| n.name == "a")
        .unwrap()
        .id
        .clone();

    assert_eq!(
        rename_folder(&mut f.conn, &f.dir, "work", "job").unwrap(),
        "job"
    );
    assert!(f.dir.join("job/tauri/a.md").is_file());
    assert_eq!(
        tree(&f.conn).unwrap().folders,
        vec!["home", "job", "job/tauri"]
    );
    assert_eq!(row(&f.conn, &a).unwrap().folder, "job/tauri");

    assert_eq!(
        move_folder(&mut f.conn, &f.dir, "job/tauri", "home").unwrap(),
        "home/tauri"
    );
    assert!(f.dir.join("home/tauri/a.md").is_file());
    assert_eq!(row(&f.conn, &a).unwrap().folder, "home/tauri");

    assert!(
        move_folder(&mut f.conn, &f.dir, "home", "home/tauri").is_err(),
        "into itself"
    );
    assert!(
        move_folder(&mut f.conn, &f.dir, "home/tauri", "home").is_err(),
        "same folder"
    );
    assert_matches_disk(&mut f);
}

#[test]
fn rename_and_move_a_note() {
    let mut f = fixture(&[("a.md", "# A"), ("work/x.md", "")]);
    let id = tree(&f.conn)
        .unwrap()
        .notes
        .iter()
        .find(|n| n.name == "a")
        .unwrap()
        .id
        .clone();
    let renamed = rename_note(&mut f.conn, &f.dir, &id, "B side").unwrap();
    assert_eq!(
        (renamed.title.as_str(), renamed.name.as_str()),
        ("B side", "b_side")
    );
    assert!(f.dir.join("b_side.md").is_file() && !f.dir.join("a.md").exists());
    let same = rename_note(&mut f.conn, &f.dir, &id, "B SIDE").unwrap();
    assert_eq!(same.name, "b_side", "same slug keeps the file");
    let moved = move_note(&mut f.conn, &f.dir, &id, "work/new").unwrap();
    assert_eq!(moved.folder, "work/new");
    assert!(f.dir.join("work/new/b_side.md").is_file());
    assert!(rename_note(&mut f.conn, &f.dir, &id, " ").is_err());
    assert_matches_disk(&mut f);
}

#[test]
fn delete_note_and_folder() {
    let mut f = fixture(&[("work/tauri/a.md", ""), ("work/b.md", ""), ("c.md", "")]);
    let c = tree(&f.conn)
        .unwrap()
        .notes
        .iter()
        .find(|n| n.name == "c")
        .unwrap()
        .id
        .clone();
    delete_note(&mut f.conn, &f.dir, &c).unwrap();
    assert!(!f.dir.join("c.md").exists());
    delete_folder(&mut f.conn, &f.dir, "work").unwrap();
    assert!(!f.dir.join("work").exists());
    let t = tree(&f.conn).unwrap();
    assert!(t.folders.is_empty() && t.notes.is_empty());
    assert!(delete_folder(&mut f.conn, &f.dir, "").is_err());
    assert_matches_disk(&mut f);
}

#[test]
fn rescan_repairs_drift() {
    let mut f = fixture(&[("a.md", "# A")]);
    f.conn.execute("DELETE FROM attachments", []).unwrap();
    f.conn
        .execute(
            "INSERT INTO note_folders (id, path, created_at, updated_at) VALUES ('g', 'ghost', 0, 0)",
            [],
        )
        .unwrap();
    scan(&mut f.conn, &f.dir).unwrap();
    let t = tree(&f.conn).unwrap();
    assert_eq!(names_in(&t), vec!["a"]);
    assert!(t.folders.is_empty());
}

#[test]
fn underscore_is_not_a_like_wildcard() {
    let mut f = fixture(&[("a_b/x.md", ""), ("acb/y.md", "")]);
    rename_folder(&mut f.conn, &f.dir, "a_b", "z").unwrap();
    let t = tree(&f.conn).unwrap();
    assert_eq!(t.folders, vec!["acb", "z"]);
    assert_eq!(names_in(&t), vec!["acb/y", "z/x"]);
}
