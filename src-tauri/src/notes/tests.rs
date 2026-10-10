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
fn without_consent_bad_names_are_reported_not_touched() {
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
    let pairs: Vec<_> = skipped
        .iter()
        .map(|s| (s.path.as_str(), s.fix.as_str()))
        .collect();
    assert_eq!(
        pairs,
        vec![(" padded.md", "padded.md"), ("Bad: Folder/", "bad_folder/")],
        "other files are ignored, not reported"
    );
    assert_eq!(names_in(&tree(&conn).unwrap()), vec!["ok"]);
    assert!(dir.join(" padded.md").exists() && dir.join("Bad: Folder/a.md").exists());
}

#[test]
fn folders_without_notes_are_ignored_unless_tend_made_them() {
    let mut f = fixture(&[("photos/a.png", "x"), ("app/node_modules/x/index.js", "x")]);
    assert!(tree(&f.conn).unwrap().folders.is_empty());
    folder_create(&mut f.conn, &f.dir, "Empty").unwrap();
    scan(&mut f.conn, &f.dir, false).unwrap();
    assert_eq!(tree(&f.conn).unwrap().folders, vec!["empty"]);
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
    let n = create(&mut f.conn, &f.dir, "Work/Ideas", " Release checklist! ").unwrap();
    assert_eq!(
        (n.folder.as_str(), n.name.as_str()),
        ("work/ideas", "release_checklist")
    );
    let file = f.dir.join("work/ideas/release_checklist.md");
    assert_eq!(fs::read_to_string(&file).unwrap(), "# Release checklist!\n");
    let n = write(&f.conn, &f.dir, &n.id, "# Ship it\n\nnow").unwrap();
    assert_eq!(
        n.name, "release_checklist",
        "editing the heading never renames the file"
    );
    assert_eq!(
        read(&f.conn, &f.dir, &n.id).unwrap().text,
        "# Ship it\n\nnow"
    );
    assert_eq!(tree(&f.conn).unwrap().folders, vec!["work", "work/ideas"]);
    assert_matches_disk(&mut f);
}

#[test]
fn titles_become_slugs_and_clashes_get_a_suffix() {
    let mut f = fixture(&[("work/a.md", "")]);
    let empty = create(&mut f.conn, &f.dir, "", "  ").unwrap_err();
    assert!(empty.is(errors::NOTE_TITLE_EMPTY));
    let name = |f: &mut Fixture, folder: &str, title: &str| {
        create(&mut f.conn, &f.dir, folder, title).unwrap().name
    };
    assert_eq!(name(&mut f, "", "Meeting"), "meeting");
    assert_eq!(name(&mut f, "", "meeting!"), "meeting_1");
    assert_eq!(
        name(&mut f, "Work", "A"),
        "a_1",
        "a typed folder is slugged too"
    );
    assert_eq!(name(&mut f, "", "日本"), "note");
    assert_eq!(
        name(&mut f, "", "work"),
        "work",
        "a note and a folder may share a name"
    );
    assert_eq!(
        folder_create(&mut f.conn, &f.dir, "Work").unwrap(),
        "work_1"
    );
    assert!(folder_create(&mut f.conn, &f.dir, "!!")
        .unwrap_err()
        .is(errors::NAME_CHARS));
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
    assert_eq!(renamed.name, "b_side");
    assert!(f.dir.join("b_side.md").is_file() && !f.dir.join("a.md").exists());
    assert_eq!(
        fs::read_to_string(f.dir.join("b_side.md")).unwrap(),
        "# A",
        "the body is untouched"
    );
    let same = rename_note(&mut f.conn, &f.dir, &id, "B SIDE").unwrap();
    assert_eq!(same.name, "b_side");
    let moved = move_note(&mut f.conn, &f.dir, &id, "work/new").unwrap();
    assert_eq!(moved.folder, "work/new");
    assert!(f.dir.join("work/new/b_side.md").is_file());
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
    let log: Vec<(String, String, String)> = f
        .conn
        .prepare("SELECT reason, old_path, new_path FROM path_log ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let expected = [
        ("rename", "a.md", "b_side.md"),
        ("move", "b_side.md", "work/new/b_side.md"),
        ("move", "work/x.md", "x_1.md"),
    ];
    let expected: Vec<_> = expected
        .iter()
        .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
        .collect();
    assert_eq!(log, expected);
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
    let before = f.conn.total_changes();
    scan(&mut f.conn, &f.dir, false).unwrap();
    assert_eq!(f.conn.total_changes(), before);
}

#[test]
fn fixing_slugs_bad_names_keeps_titles_and_logs_old_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    fs::create_dir_all(dir.join("Café (old)")).unwrap();
    fs::write(dir.join("Café (old)/Résumé!.md"), "x").unwrap();
    fs::write(dir.join("plan.md"), "x").unwrap();
    fs::write(dir.join("Plan?.md"), "y").unwrap();
    fs::write(dir.join("日本.md"), "z").unwrap();
    fs::write(dir.join(".hidden.md"), "h").unwrap();
    fs::create_dir_all(dir.join("Photos (2024)")).unwrap();
    fs::write(dir.join("Photos (2024)/a.png"), "p").unwrap();
    let mut conn = open_memory();
    let skipped = scan(&mut conn, &dir, true).unwrap();
    assert!(skipped.is_empty());
    let t = tree(&conn).unwrap();
    assert_eq!(t.folders, vec!["cafe_old"]);
    let found: Vec<_> = t
        .notes
        .iter()
        .map(|n| names::join(&n.folder, &n.name))
        .collect();
    assert_eq!(found, vec!["cafe_old/resume", "note", "plan", "plan_1"]);
    assert!(
        dir.join(".hidden.md").exists(),
        "hidden files are never touched"
    );
    assert!(
        dir.join("Photos (2024)/a.png").exists(),
        "folders without notes are left alone"
    );
    let logged: Vec<(String, String)> = conn
        .prepare("SELECT old_path, new_path FROM path_log WHERE reason = 'fix' ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert!(logged.contains(&("Café (old)".into(), "cafe_old".into())));
    assert!(logged.contains(&("Plan?.md".into(), "plan_1.md".into())));
    assert_eq!(logged.len(), 4);
}

#[test]
fn a_capitalised_name_keeps_its_word_and_loses_only_its_case() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    fs::write(dir.join("Ideas.md"), "x").unwrap();
    fs::create_dir_all(dir.join("Inbox")).unwrap();
    fs::write(dir.join("Inbox/a.md"), "x").unwrap();
    let mut conn = open_memory();
    scan(&mut conn, &dir, true).unwrap();
    let t = tree(&conn).unwrap();
    assert_eq!(t.folders, vec!["inbox"]);
    assert_eq!(names_in(&t), vec!["ideas", "inbox/a"]);
}

#[test]
fn a_note_of_30_mb_or_more_is_never_indexed() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    fs::write(dir.join("small.md"), "x").unwrap();
    let big = fs::File::create(dir.join("big.md")).unwrap();
    big.set_len(crate::files::MAX_FILE_BYTES).unwrap();
    let mut conn = open_memory();
    scan(&mut conn, &dir, true).unwrap();
    assert_eq!(names_in(&tree(&conn).unwrap()), vec!["small"]);
}

#[test]
fn an_indexed_note_that_grows_over_the_limit_keeps_its_row() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    fs::write(dir.join("grows.md"), "x").unwrap();
    let mut conn = open_memory();
    scan(&mut conn, &dir, true).unwrap();
    let file = fs::OpenOptions::new()
        .write(true)
        .open(dir.join("grows.md"))
        .unwrap();
    file.set_len(crate::files::MAX_FILE_BYTES).unwrap();
    scan(&mut conn, &dir, true).unwrap();
    // A missing row would be purged later and sync as a deletion to the other devices.
    let missing: Option<i64> = conn
        .query_row(
            "SELECT missing_at FROM attachments WHERE path = 'grows.md'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(missing, None);
}

#[cfg(unix)]
#[test]
fn symlinks_are_skipped_so_a_loop_cannot_hang_the_scan() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    fs::create_dir_all(dir.join("work")).unwrap();
    fs::write(dir.join("work/a.md"), "x").unwrap();
    std::os::unix::fs::symlink(&dir, dir.join("work/loop")).unwrap();
    let mut conn = open_memory();
    scan(&mut conn, &dir, true).unwrap();
    assert_eq!(names_in(&tree(&conn).unwrap()), vec!["work/a"]);
}
