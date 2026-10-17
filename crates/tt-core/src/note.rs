use std::fmt;

use serde::{Deserialize, Serialize};

use crate::DomainError;

/// An entry's optional free-text note (R13): trimmed, non-empty, at most
/// [`Note::MAX_CHARS`] characters. Line breaks are allowed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Note(String);

impl Note {
    pub const MAX_CHARS: usize = 2000;

    /// Blank text means "no note".
    pub fn parse(input: &str) -> Result<Option<Self>, DomainError> {
        let text = input.trim();
        if text.is_empty() {
            return Ok(None);
        }
        if text.chars().count() > Self::MAX_CHARS {
            return Err(DomainError::NoteTooLong {
                max: Self::MAX_CHARS,
            });
        }
        Ok(Some(Self(text.to_owned())))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Note {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match Note::parse(&value) {
            Ok(Some(note)) => Ok(note),
            Ok(None) => Err("a stored note is never blank".to_owned()),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl From<Note> for String {
    fn from(note: Note) -> Self {
        note.0
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
