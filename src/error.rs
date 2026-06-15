use std::fmt;

/// Errors that can occur during CJK normalization or matching.
#[derive(Debug, PartialEq, Eq)]
pub enum CjkFuzzyError {
    /// The input string is not valid for the requested operation.
    InvalidInput(String),
    /// Normalization could not be completed.
    NormalizationFailed(String),
}

impl fmt::Display for CjkFuzzyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CjkFuzzyError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            CjkFuzzyError::NormalizationFailed(msg) => write!(f, "Normalization failed: {msg}"),
        }
    }
}

impl std::error::Error for CjkFuzzyError {}

/// Convenience alias for `Result<T, CjkFuzzyError>`.
pub type Result<T> = std::result::Result<T, CjkFuzzyError>;
