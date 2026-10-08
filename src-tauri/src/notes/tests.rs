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
    scan(&mut conn, &dir, false).unwrap();
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
    scan(&mut f.conn, &f.dir, false).unwrap();
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
}

#[test]
fn readable_names_are_indexed() {
    let f = fixture(&[
        ("Work Stuff/Q4 plan - draft.md", ""),
        ("notes, v1.2.md", ""),
    ]);
    let t = tree(&f.conn).unwrap();
    assert_eq!(t.folders, vec!["Work Stuff"]);
    assert_eq!(
        names_in(&t),
        vec!["notes, v1.2", "Work Stuff/Q4 plan - draft"]
    );
}

#[test]
fn skips_invalid_names_without_touching_them() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    fs::create_dir_all(dir.join("Bad: Folder")).unwrap();
    fs::write(dir.join("Bad: Folder/a.md"), "x").unwrap();
    fs::write(dir.join(" padded.md"), "x").unwrap();
    fs::write(dir.join("image.png"), "x").unwrap();
    fs::write(dir.join(".DS_Store"), "x").unwrap();
    fs::write(dir.join("ok.md"), "x").unwrap();
    let mut conn = open_memory();
    let skipped = scan(&mut conn, &dir, false).unwrap();
    let paths: Vec<_> = skipped.iter().map(|s| s.path.as_str()).collect();
    assert_eq!(paths, vec![" padded.md", "Bad: Folder/", "image.png"]);
    let fixes: Vec<_> = skipped.iter().map(|s| s.fix.as_deref()).collect();
    assert_eq!(fixes, vec![Some("padded.md"), Some("Bad Folder/"), None]);
    assert_eq!(names_in(&tree(&conn).unwrap()), vec!["ok"]);
    assert!(dir.join(" padded.md").exists() && dir.join("Bad: Folder/a.md").exists());
}

#[test]
fn external_rename_keeps_the_id() {
    let mut f = fixture(&[("a.md", "# A")]);
    let id = tree(&f.conn).unwrap().notes[0].id.clone();
    fs::create_dir_all(f.dir.join("x")).unwrap();
    fs::rename(f.dir.join("a.md"), f.dir.join("x/b.md")).unwrap();
    scan(&mut f.conn, &f.dir, false).unwrap();
    let t = tree(&f.conn).unwrap();
    assert_eq!(names_in(&t), vec!["x/b"]);
    assert_eq!(t.notes[0].id, id);
}

#[test]
fn external_edit_bumps_updated_at() {
    let mut f = fixture(&[("a.md", "# Old")]);
    let before = tree(&f.conn).unwrap().notes[0].clone();
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs::write(f.dir.join("a.md"), "# New title").unwrap();
    scan(&mut f.conn, &f.dir, false).unwrap();
    let after = tree(&f.conn).unwrap().notes[0].clone();
    assert_eq!(after.name, "a");
    assert!(after.modified_at > before.modified_at);
}

#[test]
fn touch_without_change_keeps_updated_at() {
    let mut f = fixture(&[("a.md", "same")]);
    let before = tree(&f.conn).unwrap().notes[0].modified_at;
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs::write(f.dir.join("a.md"), "same").unwrap();
    scan(&mut f.conn, &f.dir, false).unwrap();
    assert_eq!(tree(&f.conn).unwrap().notes[0].modified_at, before);
}

#[test]
fn external_delete_marks_missing_then_purges() {
    let mut f = fixture(&[("gone/a.md", "# A"), ("b.md", "# B")]);
    fs::remove_dir_all(f.dir.join("gone")).unwrap();
    scan(&mut f.conn, &f.dir, false).unwrap();
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
    scan(&mut f.conn, &f.dir, false).unwrap();
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
    let n = create(&mut f.conn, &f.dir, "Work/Ideas", "Release checklist").unwrap();
    assert_eq!(n.name, "Release checklist");
    assert!(f.dir.join("Work/Ideas/Release checklist.md").is_file());
    let n = write(&f.conn, &f.dir, &n.id, "# Ship it\n\nnow").unwrap();
    assert_eq!(
        n.name, "Release checklist",
        "the body never renames the note"
    );
    assert_eq!(
        read(&f.conn, &f.dir, &n.id).unwrap().text,
        "# Ship it\n\nnow"
    );
    assert_eq!(tree(&f.conn).unwrap().folders, vec!["Work", "Work/Ideas"]);
    assert_matches_disk(&mut f);
}

#[test]
fn typed_names_are_cleaned_and_clashes_get_a_suffix() {
    let mut f = fixture(&[("work/a.md", "")]);
    let fails = |r: AppResult<NoteRow>, def| r.unwrap_err().is(def);
    assert!(fails(
        create(&mut f.conn, &f.dir, "", " . "),
        errors::NAME_EMPTY
    ));
    assert!(fails(
        create(&mut f.conn, &f.dir, "", "a:b"),
        errors::NAME_CHARS
    ));
    let name = |f: &mut Fixture, folder: &str, typed: &str| {
        create(&mut f.conn, &f.dir, folder, typed).unwrap().name
    };
    assert_eq!(name(&mut f, "", " .Plan. "), "Plan");
    assert_eq!(name(&mut f, "", "plan"), "plan_1", "clashes ignore case");
    assert_eq!(name(&mut f, "", "Plan"), "Plan_2");
    assert_eq!(name(&mut f, "work", "A"), "A_1");
    assert_eq!(
        name(&mut f, "", "work"),
        "work",
        "a note and a folder may share a name"
    );
    let made = create(&mut f.conn, &f.dir, "WORK", "b").unwrap();
    assert_eq!(
        made.folder, "work",
        "a typed path takes the existing spelling"
    );
    assert_eq!(
        folder_create(&mut f.conn, &f.dir, "Work").unwrap(),
        "work_1"
    );
    assert_eq!(folder_create(&mut f.conn, &f.dir, "x/.y").unwrap(), "x/y");
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
    assert_eq!(renamed.name, "B side");
    assert!(f.dir.join("B side.md").is_file() && !f.dir.join("a.md").exists());
    let recased = rename_note(&mut f.conn, &f.dir, &id, "B Side").unwrap();
    assert_eq!(recased.name, "B Side", "a case-only rename is allowed");
    let moved = move_note(&mut f.conn, &f.dir, &id, "work/new").unwrap();
    assert_eq!(moved.folder, "work/new");
    assert!(f.dir.join("work/new/B Side.md").is_file());
    fs::write(f.dir.join("x.md"), "").unwrap();
    scan(&mut f.conn, &f.dir, false).unwrap();
    let x = tree(&f.conn)
        .unwrap()
        .notes
        .into_iter()
        .find(|n| n.folder == "work" && n.name == "x")
        .unwrap();
    let moved = move_note(&mut f.conn, &f.dir, &x.id, "").unwrap();
    assert_eq!(moved.name, "x_1", "the target already has x");
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
    scan(&mut f.conn, &f.dir, false).unwrap();
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

#[test]
fn rescan_without_changes_writes_nothing() {
    let mut f = fixture(&[("work/a.md", "# A")]);
    let before: i64 = f
        .conn
        .query_row("SELECT count(*) FROM activity", [], |r| r.get(0))
        .unwrap();
    scan(&mut f.conn, &f.dir, false).unwrap();
    let after: i64 = f
        .conn
        .query_row("SELECT count(*) FROM activity", [], |r| r.get(0))
        .unwrap();
    assert_eq!(before, after);
}

#[test]
fn fixing_renames_bad_names_and_indexes_them() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    fs::create_dir_all(dir.join("Café (old)")).unwrap();
    fs::write(dir.join("Café (old)/Résumé!.md"), "x").unwrap();
    fs::write(dir.join("plan.md"), "x").unwrap();
    fs::write(dir.join("Plan?.md"), "y").unwrap();
    fs::write(dir.join("日本.md"), "z").unwrap();
    fs::write(dir.join(".hidden.md"), "h").unwrap();
    let mut conn = open_memory();
    let skipped = scan(&mut conn, &dir, true).unwrap();
    let t = tree(&conn).unwrap();
    assert_eq!(t.folders, vec!["Cafe old"]);
    assert_eq!(names_in(&t), vec!["Cafe old/Resume", "plan", "Plan_1"]);
    assert!(dir.join("Cafe old/Resume.md").is_file() && dir.join("Plan_1.md").is_file());
    let left: Vec<_> = skipped
        .iter()
        .map(|s| (s.path.as_str(), s.fix.is_none()))
        .collect();
    assert_eq!(
        left,
        vec![("日本.md", true)],
        "nothing valid can be made from it"
    );
    assert!(
        dir.join(".hidden.md").exists(),
        "hidden files are never touched"
    );
    let logged: i64 = conn
        .query_row(
            "SELECT count(*) FROM activity WHERE action = 'rename'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(logged, 3);
}
