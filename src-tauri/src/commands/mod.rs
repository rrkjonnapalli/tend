//! Tauri command handlers, one file per area. Each one unwraps the open vault and calls into a
//! domain module; errors cross to the UI as `{ code, message }` (see error.rs).
//!
//! Commands are `async` (or `#[tauri::command(async)]`) so they run off the main thread: disk
//! and database work never blocks the window. Opening a file or revealing it in Finder stays on
//! the main thread, where the OS expects it.

pub mod app;
pub mod attachments;
pub mod comments;
pub mod notes;
pub mod people;
pub mod statuses;
pub mod tags;
pub mod tasks;
pub mod timers;
pub mod vault;
