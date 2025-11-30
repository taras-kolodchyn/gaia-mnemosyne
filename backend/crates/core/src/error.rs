use thiserror::Error;

/// Common error type shared across mnemo crates.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MnemoError {
    /// Fallback variant for yet-to-be-classified errors.
    #[error("{0}")]
    Message(String),
}

/// Result type alias using `MnemoError`.
pub type MnemoResult<T> = Result<T, MnemoError>;
