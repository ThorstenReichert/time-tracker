use thiserror::Error;

/// Why a command was rejected. Every invalid save is rejected with one of these, never
/// adjusted silently (R12).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("invalid name: {0}")]
    InvalidName(NameProblem),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum NameProblem {
    #[error("it is empty")]
    Empty,
    #[error("it is longer than {max} characters")]
    TooLong { max: usize },
    #[error("it contains a control character")]
    ControlCharacter,
}
