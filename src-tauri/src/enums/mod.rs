//! The names the database and the UI share: enum kinds (`tend_enums.kind`), link types
//! (`tend_links.source_type` / `target_type`), snap papers and reference kinds. Same values as
//! `src/lib/enums/*.enum.ts`; nothing else types them.

pub mod rows;

use serde::{Deserialize, Serialize};

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
            SourceType::Task => "tasks",
            SourceType::Note => "notes",
            SourceType::Comment => "task_comments",
            SourceType::Person => "people",
            SourceType::Word => "words",
            SourceType::Snap => "snaps",
            SourceType::Ref => "refs",
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
