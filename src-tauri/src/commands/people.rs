//! People and their contacts.

use tauri::State;

use crate::error::AppResult;
use crate::people;
use crate::state::AppState;

#[tauri::command(async)]
pub fn people_list(state: State<AppState>) -> AppResult<Vec<people::PersonRow>> {
    state.with(|db, _| people::list(db))
}

#[tauri::command(async)]
pub fn person_create(state: State<AppState>, first: String, last: String) -> AppResult<String> {
    state.with(|db, _| {
        let new = people::NewPerson {
            first: &first,
            last: &last,
        };
        people::create(db, &new)
    })
}

#[tauri::command(async)]
pub fn person_get(state: State<AppState>, id: String) -> AppResult<people::Person> {
    state.with(|db, _| people::get(db, &id))
}

#[tauri::command(async)]
pub fn person_update(
    state: State<AppState>,
    id: String,
    patch: people::Patch,
) -> AppResult<people::Person> {
    state.with(|db, _| people::update(db, &id, &patch))
}

#[tauri::command(async)]
pub fn person_delete(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| people::delete(db, &id))
}

#[tauri::command(async)]
pub fn contact_add(
    state: State<AppState>,
    person: String,
    kind: String,
    value: String,
    label: String,
) -> AppResult<()> {
    state.with(|db, _| {
        let new = people::NewContact {
            person: &person,
            kind: &kind,
            value: &value,
            label: &label,
        };
        people::contact_add(db, &new)
    })
}

#[tauri::command(async)]
pub fn contact_update(
    state: State<AppState>,
    id: String,
    value: String,
    label: String,
) -> AppResult<()> {
    state.with(|db, _| {
        let edit = people::ContactEdit {
            id: &id,
            value: &value,
            label: &label,
        };
        people::contact_update(db, &edit)
    })
}

#[tauri::command(async)]
pub fn contact_delete(state: State<AppState>, id: String) -> AppResult<()> {
    state.with(|db, _| people::contact_delete(db, &id))
}
