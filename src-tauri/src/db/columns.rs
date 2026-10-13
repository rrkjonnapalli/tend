//! Column names that are passed around as data in more than one place (sync reads rows as
//! column → value maps; updates are built from the columns a patch names).

pub const ID: &str = "id";
pub const UPDATED_AT: &str = "updated_at";
pub const PATH: &str = "path";
pub const NAME: &str = "name";
pub const TITLE: &str = "title";
pub const KIND: &str = "kind";
pub const VALUE: &str = "value";
pub const LABEL: &str = "label";
pub const META: &str = "meta";
pub const STATUS: &str = "status";
pub const NOTES: &str = "notes";
pub const PARENT_ID: &str = "parent_id";
pub const STACK_ID: &str = "stack_id";
pub const TASK_ID: &str = "task_id";
pub const ATTACHMENT_ID: &str = "attachment_id";
pub const SHA256: &str = "sha256";
pub const URL_KEY: &str = "url_key";
pub const ORDER_KEY: &str = "order_key";
pub const SOURCE_TYPE: &str = "source_type";
pub const SOURCE_ID: &str = "source_id";
pub const TARGET_TYPE: &str = "target_type";
pub const TARGET_KIND: &str = "target_kind";
pub const TARGET_VALUE: &str = "target_value";
