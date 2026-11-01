use std::fmt;

use serde::{Deserialize, Serialize};

use crate::DomainError;
use crate::name::clean;

/// The optional short form of a client, project or task name (R24): the same text rules as a
/// [`Name`](crate::Name), at most [`Abbreviation::MAX_CHARS`] characters. Names and
/// abbreviations share one namespace per parent (R1); [`Abbreviation::key`] is what's
/// compared.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Abbreviation {
    text: String,
    key: String,
}

impl Abbreviation {
    pub const MAX_CHARS: usize = 20;

    pub fn parse(input: &str) -> Result<Self, DomainError> {
        let text = clean(input, Self::MAX_CHARS).map_err(DomainError::InvalidAbbreviation)?;
        Ok(Self {
            text: text.to_owned(),
            key: text.to_lowercase(),
        })
    }

    /// The abbreviation typed into an optional field: missing or blank text means none.
    pub fn parse_optional(input: Option<&str>) -> Result<Option<Self>, DomainError> {
        match input {
            Some(text) if !text.trim().is_empty() => Self::parse(text).map(Some),
            _ => Ok(None),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The case-insensitive comparison key.
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl TryFrom<String> for Abbreviation {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<Abbreviation> for String {
    fn from(abbreviation: Abbreviation) -> Self {
        abbreviation.text
    }
}

impl fmt::Display for Abbreviation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NameProblem;

    #[test]
    fn trims_and_keeps_case_for_display() {
        let abbreviation = Abbreviation::parse(" ASCP ").unwrap();
        assert_eq!(abbreviation.as_str(), "ASCP");
        assert_eq!(abbreviation.key(), "ascp");
    }

    #[test]
    fn blank_optional_input_means_none() {
        assert_eq!(Abbreviation::parse_optional(None), Ok(None));
        assert_eq!(Abbreviation::parse_optional(Some("  ")), Ok(None));
        assert_eq!(
            Abbreviation::parse_optional(Some("NWI")),
            Ok(Some(Abbreviation::parse("NWI").unwrap()))
        );
    }

    #[test]
    fn length_is_limited_in_characters() {
        assert!(Abbreviation::parse(&"ä".repeat(Abbreviation::MAX_CHARS)).is_ok());
        assert_eq!(
            Abbreviation::parse(&"a".repeat(Abbreviation::MAX_CHARS + 1)),
            Err(DomainError::InvalidAbbreviation(NameProblem::TooLong {
                max: Abbreviation::MAX_CHARS
            }))
        );
    }

    #[test]
    fn rejects_control_characters() {
        assert_eq!(
            Abbreviation::parse("a\tb"),
            Err(DomainError::InvalidAbbreviation(
                NameProblem::ControlCharacter
            ))
        );
    }

    #[test]
    fn serialises_as_the_display_text_and_validates_on_load() {
        let abbreviation = Abbreviation::parse("Web").unwrap();
        assert_eq!(serde_json::to_string(&abbreviation).unwrap(), "\"Web\"");
        assert!(serde_json::from_str::<Abbreviation>("\" \"").is_err());
    }
}
