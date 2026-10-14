//! Attachments: stored files and note links.

use tauri::State;

use crate::attachments;
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command(async)]
pub fn attachments_list(
    state: State<AppState>,
    owner: String,
    id: String,
) -> AppResult<Vec<attachments::Link>> {
    state.with(|db, _| {
        let owner = attachments::Owner {
            kind: &owner,
            id: &id,
        };
        attachments::list(db, &owner)
    })
}

/// Copies each file into the vault and links it. Every file is checked before any is copied, so
/// a folder or a too-big file in the drop adds nothing; the copying runs without the database
/// lock, so the app stays usable while a big file copies.
#[tauri::command(async)]
pub fn attachments_add(
    state: State<AppState>,
    owner: String,
    id: String,
    paths: Vec<String>,
) -> AppResult<()> {
    let sources: Vec<&std::path::Path> = paths.iter().map(std::path::Path::new).collect();
    let limit = state.with(|db, _| crate::files::Limit::of(db))?;
    sources
        .iter()
        .try_for_each(|src| attachments::check_file(src, limit))?;
    let root = state.vault()?.root.clone();
    for src in sources {
        let file = attachments::copy_in(&attachments::Incoming {
            root: &root,
            src,
            limit,
        })?;
        let copied = attachments::Copied {
            root: &root,
            file: &file,
            owner: &owner,
            owner_id: &id,
        };
        state.with(|db, _| attachments::link_copy(db, &copied))?;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn attachment_link_note(
    state: State<AppState>,
    owner: String,
    id: String,
    note: String,
) -> AppResult<()> {
    state.with(|db, _| {
        let link = attachments::NoteLink {
            owner: &owner,
            owner_id: &id,
            note_id: &note,
        };
        attachments::link_note(db, &link)
    })
}

#[tauri::command(async)]
pub fn attachment_unlink(state: State<AppState>, link: String) -> AppResult<()> {
    state.with(|db, _| attachments::unlink(db, &link))
}

fn opener_error(e: tauri_plugin_opener::Error) -> crate::error::AppError {
    crate::error::errors::OPEN_FAILED.with(e.to_string())
}

/// Opens the file with the OS default app.
#[tauri::command(async)]
pub fn attachment_open(state: State<AppState>, attachment: String) -> AppResult<()> {
    let path = state.with(|db, v| attachments::file_path(db, &v.root, &attachment))?;
    attachments::check_openable(&path)?;
    tauri_plugin_opener::open_path(path, None::<&str>).map_err(opener_error)
}

#[tauri::command(async)]
pub fn attachment_reveal(state: State<AppState>, attachment: String) -> AppResult<()> {
    let path = state.with(|db, v| attachments::file_path(db, &v.root, &attachment))?;
    tauri_plugin_opener::reveal_item_in_dir(path).map_err(opener_error)
}
