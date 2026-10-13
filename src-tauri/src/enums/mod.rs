//! The names the database and the UI share: enum kinds (`tend_enums.kind`), link types
//! (`tend_links.source_type` / `target_type`), snap papers and reference kinds. Same values as
//! `src/lib/enums/*.enum.ts`; nothing else types them.

pub mod rows;

use serde::{Deserialize, Serialize};

use crate::db::tables;
use crate::error::{errors, AppError};

#[cfg(test)]
mod tests;

/// A list the user can change, one `kind` of `tend_enums` rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumKind {
    TaskStatus,
    TaskLabel,
    WordKind,
    Tag,
}

impl EnumKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            EnumKind::TaskStatus => "TASK_STATUS",
            EnumKind::TaskLabel => "TASK_LABEL",
            EnumKind::WordKind => "WORD_KIND",
            EnumKind::Tag => "TAG",
        }
    }

    /// The error for a row of this kind that isn't there.
    pub fn not_found(self) -> AppError {
        let message = match self {
            EnumKind::TaskStatus => "Status was not found",
            EnumKind::TaskLabel => "Label was not found",
            EnumKind::WordKind => "Word kind was not found",
            EnumKind::Tag => "Tag was not found",
        };
        errors::NOT_FOUND.with(message)
    }

    /// The error for using a blocked row of this kind.
    pub fn blocked(self) -> AppError {
        match self {
            EnumKind::Tag => errors::TAG_BLOCKED.err(),
            _ => errors::ENUM_BLOCKED.err(),
        }
    }
}

/// Whether a row can be used. A blocked row stays in its list but can't be put on anything new,
/// and blocking it takes it off everything that carried it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnumStatus {
    Active,
    Blocked,
}

impl EnumStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            EnumStatus::Active => "ACTIVE",
            EnumStatus::Blocked => "BLOCKED",
        }
    }

    /// A stored value; anything but blocked reads as active.
    pub fn parse(text: &str) -> EnumStatus {
        if text == EnumStatus::Blocked.as_str() {
            EnumStatus::Blocked
        } else {
            EnumStatus::Active
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
    Word,
    Snap,
    Ref,
}

impl SourceType {
    pub const ALL: [SourceType; 7] = [
        SourceType::Task,
        SourceType::Note,
        SourceType::Comment,
        SourceType::Person,
        SourceType::Word,
        SourceType::Snap,
        SourceType::Ref,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            SourceType::Task => "TASK",
            SourceType::Note => "NOTE",
            SourceType::Comment => "COMMENT",
            SourceType::Person => "PERSON",
            SourceType::Word => "WORD",
            SourceType::Snap => "SNAP",
            SourceType::Ref => "REF",
        }
    }

    /// The table whose rows a link of this type starts from.
    pub const fn table(self) -> &'static str {
        match self {
            SourceType::Task => tables::TASKS,
            SourceType::Note => tables::NOTES,
            SourceType::Comment => tables::TASK_COMMENTS,
            SourceType::Person => tables::PEOPLE,
            SourceType::Word => tables::WORDS,
            SourceType::Snap => tables::SNAPS,
            SourceType::Ref => tables::REFS,
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

/// The paper a snap is written on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Paper {
    Scrap,
    Slip,
    Lined,
    Plain,
    Sticky,
}

impl Paper {
    const ALL: [Paper; 5] = [
        Paper::Scrap,
        Paper::Slip,
        Paper::Lined,
        Paper::Plain,
        Paper::Sticky,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Paper::Scrap => "SCRAP",
            Paper::Slip => "SLIP",
            Paper::Lined => "LINED",
            Paper::Plain => "PLAIN",
            Paper::Sticky => "STICKY",
        }
    }

    /// A stored value; one this app doesn't know (from a newer app) reads as scrap.
    pub fn parse(text: &str) -> Paper {
        let known = Self::ALL.into_iter().find(|p| p.as_str() == text);
        known.unwrap_or(Paper::Scrap)
    }
}

/// What a reference is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RefKind {
    Link,
    Text,
}

impl RefKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            RefKind::Link => "LINK",
            RefKind::Text => "TEXT",
        }
    }

    /// A stored value; anything but a link reads as text.
    pub fn parse(text: &str) -> RefKind {
        if text == RefKind::Link.as_str() {
            RefKind::Link
        } else {
            RefKind::Text
        }
    }
}
