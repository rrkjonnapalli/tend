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
    "An incompatible database exists in db/app.sqlite",
);
pub const NO_PERMISSION: ErrorDef = def(
    "NO_PERMISSION",
    "Tend doesn’t have permission to access this folder",
);
pub const IO: ErrorDef = def("IO", "A file couldn’t be read or written");
pub const DB: ErrorDef = def("DB", "The database couldn’t do that");
pub const WATCH: ErrorDef = def("WATCH", "The notes folder can’t be watched");
pub const JSON: ErrorDef = def("JSON", "That data couldn’t be read");
pub const COLOR_INVALID: ErrorDef = def("COLOR_INVALID", "Colors are #rrggbb.");
pub const OPEN_FAILED: ErrorDef = def("OPEN_FAILED", "The file couldn’t be opened");

// ---- names and folders
pub const NAME_EMPTY: ErrorDef = def("NAME_EMPTY", "Name can’t be empty");
pub const NAME_CHARS: ErrorDef = def("NAME_CHARS", "Use letters, digits, spaces and . , _ - only");
pub const NAME_LONG: ErrorDef = def("NAME_LONG", "Keep names under 200 characters");
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
pub const LABEL_ORDER: ErrorDef = def("LABEL_ORDER", "The order must list every label once.");

// ---- people
pub const FIRST_NAME_EMPTY: ErrorDef = def("FIRST_NAME_EMPTY", "A first name is needed");
pub const DOB_INVALID: ErrorDef = def("DOB_INVALID", "Use yyyy-mm-dd or mm-dd");
pub const TOB_INVALID: ErrorDef = def("TOB_INVALID", "Use a time like 6:30 AM");
pub const CONTACT_KIND: ErrorDef = def("CONTACT_KIND", "Unknown contact kind");
pub const CONTACT_EMPTY: ErrorDef = def("CONTACT_EMPTY", "Enter a value");

// ---- attachments, timers, feedback
pub const OWNER_TYPE: ErrorDef = def("OWNER_TYPE", "Unknown owner type");
pub const TIMER_DURATION: ErrorDef = def("TIMER_DURATION", "A timer must be longer than 0");
pub const FEEDBACK_KIND: ErrorDef = def("FEEDBACK_KIND", "Feedback is a bug, an idea or other.");
pub const FEEDBACK_EMPTY: ErrorDef = def("FEEDBACK_EMPTY", "Write something first.");
