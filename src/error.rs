use thiserror::Error;

/// Error type for rchain.
#[derive(Error, Debug)]
pub enum Error {

    /// Serialization or deserialization error.
    #[error("serde_ron error: {0}")]
    Serde(#[from] ron::error::Error)
}

/// Alias for a Result with the error type Error.
pub type Result<T> = std::result::Result<T, Error>;