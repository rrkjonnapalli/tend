//! Table names, in one place. Anything that passes a table name around as data (a snapshot key, a
//! rule, a piece of SQL built at run time) takes it from here; only whole SQL statements written
//! out as text name their table themselves.

pub const SETTINGS: &str = "settings";
pub const ENUMS: &str = "tend_enums";
pub const LINKS: &str = "tend_links";
pub const ATTACHMENTS: &str = "attachments";
pub const NOTE_FOLDERS: &str = "note_folders";
pub const NOTES: &str = "notes";
pub const PEOPLE: &str = "people";
pub const PERSON_CONTACTS: &str = "person_contacts";
pub const TASK_FOLDERS: &str = "task_folders";
pub const TASKS: &str = "tasks";
pub const MY_DAY: &str = "my_day";
pub const TASK_COMMENTS: &str = "task_comments";
pub const WORDS: &str = "words";
pub const SNAP_STACKS: &str = "snap_stacks";
pub const SNAPS: &str = "snaps";
pub const REFS: &str = "refs";
