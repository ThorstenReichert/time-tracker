use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{DomainError, NameProblem};

/// The name of a client, project or task (R1): trimmed, 1 to [`Name::MAX_CHARS`] characters,
/// no control characters. Names are unique within their parent, ignoring case; [`Name::key`]
/// is what's compared.
///
/// The key is the full Unicode lowercase form. That is stricter than SQLite's `NOCASE` (ASCII
/// only), so the database backstop never rejects a name the core accepted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Name {
    text: String,
    key: String,
}

impl Name {
    pub const MAX_CHARS: usize = 100;

    pub fn parse(input: &str) -> Result<Self, DomainError> {
        let text = clean(input, Self::MAX_CHARS).map_err(DomainError::InvalidName)?;
        Ok(Self {
            text: text.to_owned(),
            key: text.to_lowercase(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The case-insensitive comparison key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Same name, ignoring case.
    pub fn matches(&self, other: &Name) -> bool {
        self.key == other.key
    }
}

/// The text rules shared by names and abbreviations (R1, R24): trimmed, 1 to `max`
/// characters, no control characters.
pub(crate) fn clean(input: &str, max: usize) -> Result<&str, NameProblem> {
    let text = input.trim();
    if text.is_empty() {
        return Err(NameProblem::Empty);
    }
    if text.chars().count() > max {
        return Err(NameProblem::TooLong { max });
    }
    if text.chars().any(char::is_control) {
        return Err(NameProblem::ControlCharacter);
    }
    Ok(text)
}

impl TryFrom<String> for Name {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<Name> for String {
    fn from(name: Name) -> Self {
        name.text
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_keeps_case_for_display() {
        let name = Name::parse("  Acme GmbH ").unwrap();
        assert_eq!(name.as_str(), "Acme GmbH");
        assert_eq!(name.key(), "acme gmbh");
    }

    #[test]
    fn rejects_empty_and_blank() {
        assert_eq!(
            Name::parse("   "),
            Err(DomainError::InvalidName(NameProblem::Empty))
        );
    }

    #[test]
    fn length_is_counted_in_characters() {
        assert!(Name::parse(&"ä".repeat(Name::MAX_CHARS)).is_ok());
        assert_eq!(
            Name::parse(&"a".repeat(Name::MAX_CHARS + 1)),
            Err(DomainError::InvalidName(NameProblem::TooLong {
                max: Name::MAX_CHARS
            }))
        );
    }

    #[test]
    fn rejects_control_characters() {
        assert_eq!(
            Name::parse("a\nb"),
            Err(DomainError::InvalidName(NameProblem::ControlCharacter))
        );
    }

    #[test]
    fn case_insensitive_including_non_ascii() {
        let a = Name::parse("Müller").unwrap();
        assert!(a.matches(&Name::parse("MÜLLER").unwrap()));
        assert!(!a.matches(&Name::parse("Mueller").unwrap()));
    }

    #[test]
    fn serialises_as_the_display_text_and_validates_on_load() {
        let name = Name::parse("Web").unwrap();
        assert_eq!(serde_json::to_string(&name).unwrap(), "\"Web\"");
        assert!(serde_json::from_str::<Name>("\" \"").is_err());
    }
}
