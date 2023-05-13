use std::str::Utf8Error;
use std::string::FromUtf8Error;
use thiserror::Error;

/// Error type for rchain.
#[derive(Error, Debug)]
pub enum Error {
    /// Serialization or deserialization error.
    #[error("serde_ron error: {0}")]
    Serde(#[from] ron::error::Error),

    /// From the error which sled operation return.
    #[error("sled error: {0}")]
    Sled(#[from] sled::Error),

    /// From convert [u8] to str
    #[error("utf8 error: {0}")]
    Utf8(#[from] Utf8Error),

    /// From convert [u8] to String
    #[error("utf8 to String error: {0}")]
    FromUtf8(#[from] FromUtf8Error),

    /// We can arbitrarily define content.
    #[error("{0}")]
    StringError(String),

    /// There is no enough balance in the account.
    #[error("no enough balance")]
    NoEnoughBalance,

    /// Base58 decode error.
    #[error("base58 decode error: {0}")]
    Base58Decode(#[from] bs58::decode::Error),

    /// Invalid transaction.
    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),
}

/// Alias for a Result with the error type Error.
pub type Result<T> = std::result::Result<T, Error>;
