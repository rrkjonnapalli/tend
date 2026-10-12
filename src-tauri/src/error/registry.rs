//! Every error the app can return, in one place. Throw sites use an entry with `.err()`, or
//! `.with(…)` when the message needs details from that case.

use super::ErrorDef;

const fn def(key: &'static str, message: &'static str) -> ErrorDef {
    ErrorDef { key, message }
}

// ---- general
pub const NOT_FOUND: ErrorDef = def("NOT_FOUND", "Not found");
pub const NO_VAULT: ErrorDef = def("NO_VAULT", "No vault is open");
pub const INCOMPATIBLE_DB: ErrorDef = def(
    "INCOMPATIBLE_DB",
    "An incompatible database exists in .tend/tend.db",
);
pub const NO_PERMISSION: ErrorDef = def(
    "NO_PERMISSION",
    "Tend doesn’t have permission to access this folder",
);
pub const VAULT_GONE: ErrorDef = def(
    "VAULT_GONE",
    "The folder isn’t there any more. It may have been moved or renamed, or it’s on a drive that isn’t connected.",
);
pub const NOT_A_VAULT: ErrorDef = def("NOT_A_VAULT", "This folder isn’t a Tend vault.");
pub const VAULT_OPEN_FAILED: ErrorDef = def("VAULT_OPEN_FAILED", "The vault couldn’t be opened.");
pub const IO: ErrorDef = def("IO", "A file couldn’t be read or written");
pub const DB: ErrorDef = def("DB", "The database couldn’t do that");
pub const WATCH: ErrorDef = def("WATCH", "The notes folder can’t be watched");
pub const EVENT: ErrorDef = def("EVENT", "The window couldn’t be told about a change");
pub const JSON: ErrorDef = def("JSON", "That data couldn’t be read");
pub const COLOR_INVALID: ErrorDef = def("COLOR_INVALID", "Colors are #rrggbb.");
pub const OPEN_FAILED: ErrorDef = def("OPEN_FAILED", "The file couldn’t be opened");
pub const FILE_TOO_BIG: ErrorDef = def("FILE_TOO_BIG", "This file is over the size limit.");
pub const FILE_LIMIT: ErrorDef = def("FILE_LIMIT", "The size limit is 1 to 90 MB.");

// ---- names and folders
pub const NAME_EMPTY: ErrorDef = def("NAME_EMPTY", "Name can’t be empty");
pub const NAME_CHARS: ErrorDef = def("NAME_CHARS", "Use at least one letter or digit");
pub const NOTE_TITLE_EMPTY: ErrorDef = def("NOTE_TITLE_EMPTY", "A note needs a title");
pub const NAME_TAKEN: ErrorDef = def("NAME_TAKEN", "Too many items with this name here");
pub const SAME_FOLDER: ErrorDef = def("SAME_FOLDER", "It’s already in that folder");
pub const INTO_SELF: ErrorDef = def("INTO_SELF", "A folder can’t move into itself");
pub const ROOT_FOLDER: ErrorDef = def("ROOT_FOLDER", "The root folder can’t be deleted");

// ---- tasks and comments
pub const TASK_TITLE_EMPTY: ErrorDef = def("TASK_TITLE_EMPTY", "A task needs a title");
pub const SUBTASK_MOVE: ErrorDef = def("SUBTASK_MOVE", "Subtasks move with their parent");
pub const SUBTASK_DEPTH: ErrorDef =
    def("SUBTASK_DEPTH", "A subtask can't have subtasks of its own");
pub const COMMENT_EMPTY: ErrorDef = def("COMMENT_EMPTY", "A comment can’t be empty");

// ---- statuses
pub const STATUS_VALUE: ErrorDef = def("STATUS_VALUE", "Give the status a value (A–Z, 0–9 and _).");
pub const STATUS_RESERVED: ErrorDef = def(
    "STATUS_RESERVED",
    "That value is reserved for a default status.",
);
pub const STATUS_VALUE_TAKEN: ErrorDef = def(
    "STATUS_VALUE_TAKEN",
    "That value is already used by another status.",
);
pub const STATUS_LABEL_EMPTY: ErrorDef = def("STATUS_LABEL_EMPTY", "Label can’t be empty.");
pub const STATUS_LABEL_TAKEN: ErrorDef = def(
    "STATUS_LABEL_TAKEN",
    "Another status already uses this label.",
);
pub const STATUS_ORDER: ErrorDef = def(
    "STATUS_ORDER",
    "Not started stays first and Done stays last.",
);
pub const STATUS_DEFAULT: ErrorDef = def(
    "STATUS_DEFAULT",
    "Default statuses can be renamed, not deleted.",
);
pub const STATUS_IN_USE: ErrorDef = def(
    "STATUS_IN_USE",
    "Some tasks still have this status. Move them to another status first.",
);

// ---- labels
pub const LABEL_NAME_EMPTY: ErrorDef = def("LABEL_NAME_EMPTY", "A label needs a name.");
pub const LABEL_NAME_TAKEN: ErrorDef =
    def("LABEL_NAME_TAKEN", "Another label already uses this name.");
pub const LABEL_VALUE: ErrorDef = def(
    "LABEL_VALUE",
    "A value is capital letters, digits and single _ (like DB_SCRIPT).",
);
pub const LABEL_VALUE_TAKEN: ErrorDef = def(
    "LABEL_VALUE_TAKEN",
    "Another label already uses this value. Change the value.",
);
pub const LABEL_ORDER: ErrorDef = def("LABEL_ORDER", "The order must list every label once.");

// ---- people
pub const FIRST_NAME_EMPTY: ErrorDef = def("FIRST_NAME_EMPTY", "A first name is needed");
pub const DOB_INVALID: ErrorDef = def("DOB_INVALID", "Use yyyy-mm-dd or mm-dd");
pub const TOB_INVALID: ErrorDef = def("TOB_INVALID", "Use a time like 6:30 AM");
pub const DUE_DATE_INVALID: ErrorDef = def("DUE_DATE_INVALID", "That isn’t a real date.");
pub const DUE_TIME_INVALID: ErrorDef = def("DUE_TIME_INVALID", "Use a time like 6:30 AM");
pub const CONTACT_KIND: ErrorDef = def("CONTACT_KIND", "Unknown contact kind");
pub const CONTACT_EMPTY: ErrorDef = def("CONTACT_EMPTY", "Enter a value");

// ---- library
pub const WORD_TITLE_EMPTY: ErrorDef = def("WORD_TITLE_EMPTY", "Give it a title.");
pub const WORD_TAKEN: ErrorDef = def("WORD_TAKEN", "Already saved.");
pub const WORD_KIND_UNKNOWN: ErrorDef = def("WORD_KIND_UNKNOWN", "Pick one of the word kinds.");
pub const SNAP_TEXT_EMPTY: ErrorDef = def("SNAP_TEXT_EMPTY", "Write something first.");
pub const STACK_INTO_SELF: ErrorDef = def("STACK_INTO_SELF", "A stack can’t go inside itself.");
pub const REF_VALUE_EMPTY: ErrorDef = def("REF_VALUE_EMPTY", "Paste a link or type a name.");
pub const REF_LINK_TAKEN: ErrorDef = def("REF_LINK_TAKEN", "You saved this link already.");
pub const REF_TITLE_FETCH: ErrorDef = def("REF_TITLE_FETCH", "The page title couldn’t be fetched.");

// ---- attachments, timers, feedback
pub const OWNER_TYPE: ErrorDef = def("OWNER_TYPE", "Unknown owner type");
pub const TIMER_DURATION: ErrorDef = def("TIMER_DURATION", "A timer must be longer than 0");
pub const FEEDBACK_KIND: ErrorDef = def("FEEDBACK_KIND", "Feedback is a bug, an idea or other.");
pub const FEEDBACK_EMPTY: ErrorDef = def("FEEDBACK_EMPTY", "Write something first.");

// ---- sync
pub const SNAPSHOT_TOO_NEW: ErrorDef = def(
    "SNAPSHOT_TOO_NEW",
    "This vault was synced by a newer Tend. Update Tend to keep syncing.",
);
pub const SYNC_APPLY_FAILED: ErrorDef = def("SYNC_APPLY_FAILED", "Sync couldn’t apply the changes");
pub const SYNC_AUTH: ErrorDef = def(
    "SYNC_AUTH",
    "The sync token was refused. Replace it in Settings → Sync.",
);
pub const SYNC_OFFLINE: ErrorDef = def("SYNC_OFFLINE", "Couldn’t reach the sync remote.");
pub const SYNC_REJECTED: ErrorDef = def(
    "SYNC_REJECTED",
    "Another device synced at the same time. Sync again.",
);
pub const SYNC_GIT: ErrorDef = def("SYNC_GIT", "Git couldn’t complete the sync.");
pub const SYNC_NOT_CONNECTED: ErrorDef = def(
    "SYNC_NOT_CONNECTED",
    "This vault isn’t connected to a remote.",
);
pub const SYNC_REMOTE_NOT_EMPTY: ErrorDef = def(
    "SYNC_REMOTE_NOT_EMPTY",
    "This repo already has a vault. Restore it into an empty folder instead.",
);
pub const SYNC_BASE_LOST: ErrorDef = def(
    "SYNC_BASE_LOST",
    "The remote’s history changed. Restore the vault into an empty folder to keep syncing.",
);
pub const SYNC_NOT_A_VAULT: ErrorDef =
    def("SYNC_NOT_A_VAULT", "This repo doesn’t hold a Tend vault.");
pub const SYNC_TOKEN: ErrorDef = def(
    "SYNC_TOKEN",
    "Couldn’t use the Keychain for the sync token.",
);
pub const RESTORE_NOT_EMPTY: ErrorDef = def(
    "RESTORE_NOT_EMPTY",
    "Choose an empty folder to restore into.",
);
pub const SYNC_INTERVAL: ErrorDef = def(
    "SYNC_INTERVAL",
    "Sync every 0 to 1440 minutes (0 turns it off).",
);
