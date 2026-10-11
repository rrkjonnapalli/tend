//! The names the database and the UI share: enum kinds (`tend_enums.kind`) and link types
//! (`tend_links.source_type` / `target_type`). Same values as `src/lib/enums/*.enum.ts`; nothing
//! else types them.

pub mod rows;

#[cfg(test)]
mod tests;

/// A list the user can change, one `kind` of `tend_enums` rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumKind {
    TaskStatus,
    TaskLabel,
    Tag,
}

impl EnumKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            EnumKind::TaskStatus => "TASK_STATUS",
            EnumKind::TaskLabel => "TASK_LABEL",
            EnumKind::Tag => "TAG",
        }
    }
}

/// The record a link starts from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Task,
    Note,
    Comment,
    Person,
}

impl SourceType {
    pub const ALL: [SourceType; 4] = [
        SourceType::Task,
        SourceType::Note,
        SourceType::Comment,
        SourceType::Person,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            SourceType::Task => "TASK",
            SourceType::Note => "NOTE",
            SourceType::Comment => "COMMENT",
            SourceType::Person => "PERSON",
        }
    }

    /// The table whose rows a link of this type starts from.
    pub const fn table(self) -> &'static str {
        match self {
            SourceType::Task => "tasks",
            SourceType::Note => "notes",
            SourceType::Comment => "task_comments",
            SourceType::Person => "people",
        }
    }

    pub fn parse(text: &str) -> Option<SourceType> {
        Self::ALL.into_iter().find(|s| s.as_str() == text)
    }
}

/// What a link points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    Enum,
    Attachment,
}

impl TargetType {
    pub const fn as_str(self) -> &'static str {
        match self {
            TargetType::Enum => "ENUM",
            TargetType::Attachment => "ATTACHMENT",
        }
    }
}
