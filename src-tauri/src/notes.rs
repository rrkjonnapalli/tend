//! The notes index. Files in `notes/` are the truth; `note_folders` and `notes` mirror them.
//! Every change made inside Tend writes to disk first, then to the database in one transaction,
//! and puts the disk back if the database step fails.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::names;
use crate::util::{new_id, now_ms};

/// Items missing from disk stay in the index this long, then they are purged.
pub const GRACE_MS: i64 = 7 * 24 * 60 * 60 * 1000;
const EXT: &str = ".md";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoteRow {
    pub id: String,
    pub folder: String,
    /// File name without `.md`.
    pub name: String,
    pub title: String,
    pub modified_at: i64,
}

#[derive(Debug, Serialize)]
pub struct Tree {
    pub folders: Vec<String>,
    pub notes: Vec<NoteRow>,
}

#[derive(Debug, Serialize)]
pub struct NoteDoc {
    #[serde(flatten)]
    pub row: NoteRow,
    pub text: String,
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------------------------
// Disk walk and scan

struct DiskNote {
    folder: String,
    file_name: String,
    mtime: i64,
}

#[derive(Default)]
struct Disk {
    folders: Vec<String>,
    notes: Vec<DiskNote>,
    skipped: Vec<String>,
}

fn mtime(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_millis() as i64)
}

fn walk(dir: &Path, rel: &str, out: &mut Disk) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let file = entry.file_name().to_string_lossy().into_owned();
        if file.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let shown = names::join(rel, &file);
        if path.is_dir() {
            if names::is_valid(&file) {
                out.folders.push(shown.clone());
                walk(&path, &shown, out);
            } else {
                out.skipped.push(format!("{shown}/"));
            }
        } else {
            match file.strip_suffix(EXT) {
                Some(stem) if names::is_valid(stem) => out.notes.push(DiskNote {
                    folder: rel.to_string(),
                    file_name: file,
                    mtime: mtime(&path),
                }),
                _ => out.skipped.push(shown),
            }
        }
    }
}

/// The first `# ` heading, or the file name without `.md`.
pub fn title_of(text: &str, name: &str) -> String {
    text.lines()
        .find_map(|l| l.strip_prefix("# ").map(str::trim))
        .filter(|t| !t.is_empty())
        .map_or_else(|| name.to_string(), str::to_string)
}

fn file_path(dir: &Path, folder: &str, file_name: &str) -> PathBuf {
    dir.join(names::join(folder, file_name))
}

fn read_title(dir: &Path, folder: &str, file_name: &str) -> String {
    let text = fs::read_to_string(file_path(dir, folder, file_name)).unwrap_or_default();
    title_of(&text, file_name.trim_end_matches(EXT))
}

struct Indexed {
    id: String,
    modified_at: i64,
    missing: bool,
    title: String,
}

/// Makes the index match the disk. Returns the skipped items (names that break the rule).
pub fn scan(conn: &mut Connection, dir: &Path) -> AppResult<Vec<String>> {
    let mut disk = Disk::default();
    walk(dir, "", &mut disk);
    let now = now_ms();
    let tx = conn.transaction()?;

    let on_disk: HashSet<&str> = disk.folders.iter().map(String::as_str).collect();
    for f in &disk.folders {
        tx.execute(
            "INSERT INTO note_folders (path, created_at) VALUES (?1, ?2)
             ON CONFLICT(path) DO UPDATE SET missing_at = NULL",
            params![f, now],
        )?;
    }
    let known: Vec<String> = tx
        .prepare("SELECT path FROM note_folders WHERE path <> '' AND missing_at IS NULL")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    for f in known.iter().filter(|f| !on_disk.contains(f.as_str())) {
        tx.execute("UPDATE note_folders SET missing_at = ?1 WHERE path = ?2", params![now, f])?;
    }

    let mut indexed: HashMap<(String, String), Indexed> = HashMap::new();
    {
        let mut stmt = tx.prepare("SELECT folder, file_name, id, modified_at, missing_at, title FROM notes")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                (r.get::<_, String>(0)?, r.get::<_, String>(1)?),
                Indexed {
                    id: r.get(2)?,
                    modified_at: r.get(3)?,
                    missing: r.get::<_, Option<i64>>(4)?.is_some(),
                    title: r.get(5)?,
                },
            ))
        })?;
        for row in rows {
            let (key, value) = row?;
            indexed.insert(key, value);
        }
    }

    let present: HashSet<(String, String)> =
        disk.notes.iter().map(|n| (n.folder.clone(), n.file_name.clone())).collect();
    // Notes that just vanished. A new file with the same mtime and title is the same note,
    // renamed or moved outside Tend, so it keeps its id (and its tags and links).
    let mut vanished: Vec<(String, i64, String)> = indexed
        .iter()
        .filter(|(k, v)| !v.missing && !present.contains(*k))
        .map(|(_, v)| (v.id.clone(), v.modified_at, v.title.clone()))
        .collect();

    for n in &disk.notes {
        let key = (n.folder.clone(), n.file_name.clone());
        match indexed.get(&key) {
            Some(ix) if ix.modified_at == n.mtime && !ix.missing => {}
            Some(ix) => {
                let title = read_title(dir, &n.folder, &n.file_name);
                tx.execute(
                    "UPDATE notes SET title = ?1, modified_at = ?2, missing_at = NULL WHERE id = ?3",
                    params![title, n.mtime, ix.id],
                )?;
            }
            None => {
                let title = read_title(dir, &n.folder, &n.file_name);
                let renamed = vanished.iter().position(|(_, m, t)| *m == n.mtime && *t == title);
                match renamed {
                    Some(i) => {
                        let (id, _, _) = vanished.swap_remove(i);
                        tx.execute(
                            "UPDATE notes SET folder = ?1, file_name = ?2, missing_at = NULL WHERE id = ?3",
                            params![n.folder, n.file_name, id],
                        )?;
                    }
                    None => {
                        tx.execute(
                            "INSERT INTO notes (id, folder, file_name, title, created_at, modified_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                            params![new_id(), n.folder, n.file_name, title, now, n.mtime],
                        )?;
                    }
                }
            }
        }
    }
    for (id, _, _) in &vanished {
        tx.execute("UPDATE notes SET missing_at = ?1 WHERE id = ?2", params![now, id])?;
    }

    purge(&tx, now)?;
    tx.commit()?;
    disk.skipped.sort();
    Ok(disk.skipped)
}

fn purge(tx: &Transaction, now: i64) -> AppResult<()> {
    let cutoff = now - GRACE_MS;
    tx.execute("DELETE FROM notes WHERE missing_at < ?1", [cutoff])?;
    tx.execute(
        "DELETE FROM note_folders WHERE missing_at < ?1
         AND NOT EXISTS (SELECT 1 FROM notes WHERE notes.folder = note_folders.path)",
        [cutoff],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Queries

const ROW_SQL: &str = "SELECT id, folder, file_name, title, modified_at FROM notes";

fn to_row(r: &rusqlite::Row) -> rusqlite::Result<NoteRow> {
    let file: String = r.get(2)?;
    Ok(NoteRow {
        id: r.get(0)?,
        folder: r.get(1)?,
        name: file.trim_end_matches(EXT).to_string(),
        title: r.get(3)?,
        modified_at: r.get(4)?,
    })
}

pub fn tree(conn: &Connection) -> AppResult<Tree> {
    let folders = conn
        .prepare("SELECT path FROM note_folders WHERE missing_at IS NULL AND path <> '' ORDER BY path")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    let notes = conn
        .prepare(&format!("{ROW_SQL} WHERE missing_at IS NULL ORDER BY folder, file_name"))?
        .query_map([], to_row)?
        .collect::<Result<_, _>>()?;
    Ok(Tree { folders, notes })
}

pub fn row(conn: &Connection, id: &str) -> AppResult<NoteRow> {
    conn.query_row(&format!("{ROW_SQL} WHERE id = ?1 AND missing_at IS NULL"), [id], to_row)
        .optional()?
        .ok_or_else(|| AppError::NotFound("Note".into()))
}

pub fn read(conn: &Connection, dir: &Path, id: &str) -> AppResult<NoteDoc> {
    let row = row(conn, id)?;
    let text = fs::read_to_string(file_path(dir, &row.folder, &format!("{}{EXT}", row.name)))?;
    let tags = crate::tags::of(conn, crate::tags::Owner::Note, id)?;
    Ok(NoteDoc { row, text, tags })
}

pub fn write(conn: &Connection, dir: &Path, id: &str, text: &str) -> AppResult<NoteRow> {
    let row = row(conn, id)?;
    let path = file_path(dir, &row.folder, &format!("{}{EXT}", row.name));
    fs::write(&path, text)?;
    conn.execute(
        "UPDATE notes SET title = ?1, modified_at = ?2 WHERE id = ?3",
        params![title_of(text, &row.name), mtime(&path), id],
    )?;
    self::row(conn, id)
}

fn folder_exists(conn: &Connection, path: &str) -> AppResult<bool> {
    Ok(path.is_empty()
        || conn
            .query_row("SELECT 1 FROM note_folders WHERE path = ?1 AND missing_at IS NULL", [path], |_| Ok(()))
            .optional()?
            .is_some())
}

/// A folder or note called `name` already sits in `folder` (on disk or in the index).
fn clash(conn: &Connection, dir: &Path, folder: &str, name: &str) -> AppResult<bool> {
    let file = format!("{name}{EXT}");
    let in_db = folder_exists(conn, &names::join(folder, name))?
        || conn
            .query_row(
                "SELECT 1 FROM notes WHERE folder = ?1 AND file_name = ?2 AND missing_at IS NULL",
                params![folder, file],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
    Ok(in_db || dir.join(names::join(folder, name)).exists() || file_path(dir, folder, &file).exists())
}

fn check_new(conn: &Connection, dir: &Path, folder: &str, name: &str) -> AppResult<()> {
    names::check(name)?;
    if clash(conn, dir, folder, name)? {
        return Err(AppError::InvalidName(names::CLASH.into()));
    }
    Ok(())
}

/// Drops index rows left behind by missing items at `path`, so a new item can take the name.
fn clear_missing(tx: &Transaction, path: &str) -> AppResult<()> {
    tx.execute(
        "DELETE FROM notes WHERE missing_at IS NOT NULL AND (folder = ?1 OR substr(folder, 1, length(?1) + 1) = ?1 || '/')",
        [path],
    )?;
    tx.execute(
        "DELETE FROM note_folders WHERE missing_at IS NOT NULL AND (path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/')",
        [path],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Changes (disk first, then the database)

/// Runs the database step; if it fails, `undo` puts the disk back.
fn then_db<T>(conn: &mut Connection, undo: impl FnOnce(), db: impl FnOnce(&Transaction) -> AppResult<T>) -> AppResult<T> {
    let result = (|| {
        let tx = conn.transaction()?;
        let out = db(&tx)?;
        tx.commit()?;
        Ok(out)
    })();
    if result.is_err() {
        undo();
    }
    result
}

/// Creates `path` and any missing parents. Each segment must follow the name rule.
pub fn folder_create(conn: &mut Connection, dir: &Path, path: &str) -> AppResult<()> {
    names::check_path(path)?;
    if path.is_empty() {
        return Ok(());
    }
    check_new(conn, dir, names::parent(path), names::base(path))?;
    let mut missing = Vec::new();
    let mut cur = String::new();
    for seg in path.split('/') {
        cur = names::join(&cur, seg);
        if !folder_exists(conn, &cur)? {
            missing.push(cur.clone());
        }
    }
    fs::create_dir_all(dir.join(path))?;
    let now = now_ms();
    then_db(conn, || {}, |tx| {
        for p in &missing {
            clear_missing(tx, p)?;
            tx.execute("INSERT INTO note_folders (path, created_at) VALUES (?1, ?2)", params![p, now])?;
        }
        Ok(())
    })
}

/// The heading a new note starts with: `release_checklist` → `# Release checklist`.
fn heading_for(name: &str) -> String {
    let words = name.replace('_', " ");
    let mut chars = words.chars();
    let title = chars.next().map_or(String::new(), |c| c.to_uppercase().chain(chars).collect());
    format!("# {title}\n\n")
}

pub fn create(conn: &mut Connection, dir: &Path, folder: &str, name: &str) -> AppResult<NoteRow> {
    if !folder_exists(conn, folder)? {
        folder_create(conn, dir, folder)?;
    }
    check_new(conn, dir, folder, name)?;
    let file = format!("{name}{EXT}");
    let path = file_path(dir, folder, &file);
    let text = heading_for(name);
    fs::write(&path, &text)?;
    let id = new_id();
    let (now, modified) = (now_ms(), mtime(&path));
    then_db(conn, || drop(fs::remove_file(&path)), |tx| {
        tx.execute(
            "INSERT INTO notes (id, folder, file_name, title, created_at, modified_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, folder, file, title_of(&text, name), now, modified],
        )?;
        Ok(())
    })?;
    row(conn, &id)
}

pub fn rename_note(conn: &mut Connection, dir: &Path, id: &str, name: &str) -> AppResult<NoteRow> {
    let note = row(conn, id)?;
    if note.name == name {
        return Ok(note);
    }
    check_new(conn, dir, &note.folder, name)?;
    relocate_note(conn, dir, &note, &note.folder, name)
}

pub fn move_note(conn: &mut Connection, dir: &Path, id: &str, target: &str) -> AppResult<NoteRow> {
    let note = row(conn, id)?;
    if note.folder == target {
        return Err(AppError::rule("same_folder", "It’s already in that folder"));
    }
    if !folder_exists(conn, target)? {
        folder_create(conn, dir, target)?;
    }
    check_new(conn, dir, target, &note.name)?;
    relocate_note(conn, dir, &note, target, &note.name)
}

fn relocate_note(conn: &mut Connection, dir: &Path, note: &NoteRow, folder: &str, name: &str) -> AppResult<NoteRow> {
    let from = file_path(dir, &note.folder, &format!("{}{EXT}", note.name));
    let file = format!("{name}{EXT}");
    let to = file_path(dir, folder, &file);
    fs::rename(&from, &to)?;
    then_db(conn, || drop(fs::rename(&to, &from)), |tx| {
        tx.execute(
            "UPDATE notes SET folder = ?1, file_name = ?2 WHERE id = ?3",
            params![folder, file, note.id],
        )?;
        Ok(())
    })?;
    row(conn, &note.id)
}

pub fn rename_folder(conn: &mut Connection, dir: &Path, path: &str, name: &str) -> AppResult<String> {
    if names::base(path) == name {
        return Ok(path.to_string());
    }
    let parent = names::parent(path);
    check_new(conn, dir, parent, name)?;
    relocate_folder(conn, dir, path, &names::join(parent, name))
}

pub fn move_folder(conn: &mut Connection, dir: &Path, path: &str, target: &str) -> AppResult<String> {
    if names::is_under(target, path) {
        return Err(AppError::rule("into_self", "A folder can’t move into itself"));
    }
    if names::parent(path) == target {
        return Err(AppError::rule("same_folder", "It’s already in that folder"));
    }
    if !folder_exists(conn, target)? {
        folder_create(conn, dir, target)?;
    }
    let name = names::base(path);
    check_new(conn, dir, target, name)?;
    relocate_folder(conn, dir, path, &names::join(target, name))
}

fn relocate_folder(conn: &mut Connection, dir: &Path, old: &str, new: &str) -> AppResult<String> {
    if old.is_empty() || !folder_exists(conn, old)? {
        return Err(AppError::NotFound("Folder".into()));
    }
    let (from, to) = (dir.join(old), dir.join(new));
    fs::rename(&from, &to)?;
    then_db(conn, || drop(fs::rename(&to, &from)), |tx| {
        clear_missing(tx, new)?;
        // Notes follow through ON UPDATE CASCADE.
        tx.execute(
            "UPDATE note_folders SET path = ?2 || substr(path, length(?1) + 1)
             WHERE path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'",
            params![old, new],
        )?;
        Ok(())
    })?;
    Ok(new.to_string())
}

pub fn delete_note(conn: &mut Connection, dir: &Path, id: &str) -> AppResult<()> {
    let note = row(conn, id)?;
    fs::remove_file(file_path(dir, &note.folder, &format!("{}{EXT}", note.name)))?;
    then_db(conn, || {}, |tx| {
        tx.execute("DELETE FROM notes WHERE id = ?1", [id])?;
        Ok(())
    })
}

/// Deletes the folder and everything inside it, from disk and from the index.
pub fn delete_folder(conn: &mut Connection, dir: &Path, path: &str) -> AppResult<()> {
    if path.is_empty() {
        return Err(AppError::rule("root", "The root folder can’t be deleted"));
    }
    if !folder_exists(conn, path)? {
        return Err(AppError::NotFound("Folder".into()));
    }
    fs::remove_dir_all(dir.join(path))?;
    then_db(conn, || {}, |tx| {
        tx.execute(
            "DELETE FROM notes WHERE folder = ?1 OR substr(folder, 1, length(?1) + 1) = ?1 || '/'",
            [path],
        )?;
        tx.execute(
            "DELETE FROM note_folders WHERE path = ?1 OR substr(path, 1, length(?1) + 1) = ?1 || '/'",
            [path],
        )?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_memory;

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
        Fixture { _tmp: tmp, dir, conn }
    }

    fn names_in(t: &Tree) -> Vec<String> {
        t.notes.iter().map(|n| names::join(&n.folder, &n.name)).collect()
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
        assert_eq!(names_in(&t), vec!["scratch", "work/release_checklist", "work/tauri/window_config"]);
        let wc = t.notes.iter().find(|n| n.name == "window_config").unwrap();
        assert_eq!(wc.title, "Window config");
        let rc = t.notes.iter().find(|n| n.name == "release_checklist").unwrap();
        assert_eq!(rc.title, "release_checklist");
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
    fn external_edit_updates_title() {
        let mut f = fixture(&[("a.md", "# Old")]);
        std::thread::sleep(std::time::Duration::from_millis(20));
        fs::write(f.dir.join("a.md"), "# New title").unwrap();
        scan(&mut f.conn, &f.dir).unwrap();
        assert_eq!(tree(&f.conn).unwrap().notes[0].title, "New title");
    }

    #[test]
    fn external_delete_marks_missing_then_purges() {
        let mut f = fixture(&[("gone/a.md", "# A"), ("b.md", "# B")]);
        fs::remove_dir_all(f.dir.join("gone")).unwrap();
        scan(&mut f.conn, &f.dir).unwrap();
        assert_eq!(names_in(&tree(&f.conn).unwrap()), vec!["b"]);
        assert!(tree(&f.conn).unwrap().folders.is_empty());
        let rows: i64 = f.conn.query_row("SELECT count(*) FROM notes", [], |r| r.get(0)).unwrap();
        assert_eq!(rows, 2, "missing note stays during the grace period");

        let old = now_ms() - GRACE_MS - 1;
        f.conn.execute("UPDATE notes SET missing_at = ?1 WHERE missing_at IS NOT NULL", [old]).unwrap();
        f.conn.execute("UPDATE note_folders SET missing_at = ?1 WHERE missing_at IS NOT NULL", [old]).unwrap();
        scan(&mut f.conn, &f.dir).unwrap();
        let rows: i64 = f.conn.query_row("SELECT count(*) FROM notes", [], |r| r.get(0)).unwrap();
        let folders: i64 = f.conn.query_row("SELECT count(*) FROM note_folders", [], |r| r.get(0)).unwrap();
        assert_eq!((rows, folders), (1, 1));
    }

    #[test]
    fn create_write_and_read() {
        let mut f = fixture(&[]);
        let n = create(&mut f.conn, &f.dir, "work/ideas", "release_checklist").unwrap();
        assert_eq!(n.title, "Release checklist");
        assert!(f.dir.join("work/ideas/release_checklist.md").is_file());
        let n = write(&f.conn, &f.dir, &n.id, "# Ship it\n\nnow").unwrap();
        assert_eq!(n.title, "Ship it");
        assert_eq!(read(&f.conn, &f.dir, &n.id).unwrap().text, "# Ship it\n\nnow");
        assert_eq!(tree(&f.conn).unwrap().folders, vec!["work", "work/ideas"]);
        assert_matches_disk(&mut f);
    }

    #[test]
    fn name_validation_and_clashes() {
        let mut f = fixture(&[("work/a.md", "")]);
        let err = |r: AppResult<NoteRow>| match r {
            Err(AppError::InvalidName(m)) => m,
            other => panic!("expected invalid name, got {other:?}"),
        };
        assert_eq!(err(create(&mut f.conn, &f.dir, "", "Bad Name")), names::CHARS);
        assert_eq!(err(create(&mut f.conn, &f.dir, "", "")), names::EMPTY);
        assert_eq!(err(create(&mut f.conn, &f.dir, "", "work")), names::CLASH);
        assert_eq!(err(create(&mut f.conn, &f.dir, "work", "a")), names::CLASH);
        assert!(folder_create(&mut f.conn, &f.dir, "work").is_err());
        assert!(folder_create(&mut f.conn, &f.dir, "Work2").is_err());
    }

    #[test]
    fn rename_and_move_a_folder_with_children() {
        let mut f = fixture(&[("work/tauri/a.md", "# A"), ("work/b.md", "# B"), ("home/c.md", "# C")]);
        let a = tree(&f.conn).unwrap().notes.iter().find(|n| n.name == "a").unwrap().id.clone();

        assert_eq!(rename_folder(&mut f.conn, &f.dir, "work", "job").unwrap(), "job");
        assert!(f.dir.join("job/tauri/a.md").is_file());
        assert_eq!(tree(&f.conn).unwrap().folders, vec!["home", "job", "job/tauri"]);
        assert_eq!(row(&f.conn, &a).unwrap().folder, "job/tauri");

        assert_eq!(move_folder(&mut f.conn, &f.dir, "job/tauri", "home").unwrap(), "home/tauri");
        assert!(f.dir.join("home/tauri/a.md").is_file());
        assert_eq!(row(&f.conn, &a).unwrap().folder, "home/tauri");

        assert!(move_folder(&mut f.conn, &f.dir, "home", "home/tauri").is_err(), "into itself");
        assert!(move_folder(&mut f.conn, &f.dir, "home/tauri", "home").is_err(), "same folder");
        assert_matches_disk(&mut f);
    }

    #[test]
    fn rename_and_move_a_note() {
        let mut f = fixture(&[("a.md", "# A"), ("work/x.md", "")]);
        let id = tree(&f.conn).unwrap().notes.iter().find(|n| n.name == "a").unwrap().id.clone();
        rename_note(&mut f.conn, &f.dir, &id, "b").unwrap();
        assert!(f.dir.join("b.md").is_file() && !f.dir.join("a.md").exists());
        let moved = move_note(&mut f.conn, &f.dir, &id, "work/new").unwrap();
        assert_eq!(moved.folder, "work/new");
        assert!(f.dir.join("work/new/b.md").is_file());
        assert!(rename_note(&mut f.conn, &f.dir, &id, "Bad").is_err());
        assert_matches_disk(&mut f);
    }

    #[test]
    fn delete_note_and_folder() {
        let mut f = fixture(&[("work/tauri/a.md", ""), ("work/b.md", ""), ("c.md", "")]);
        let c = tree(&f.conn).unwrap().notes.iter().find(|n| n.name == "c").unwrap().id.clone();
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
        f.conn.execute("DELETE FROM notes", []).unwrap();
        f.conn.execute("INSERT INTO note_folders (path, created_at) VALUES ('ghost', 0)", []).unwrap();
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
}
