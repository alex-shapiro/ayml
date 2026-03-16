use std::fmt;

/// Error type for ayml-serde serialization and deserialization.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error
    #[error(transparent)]
    Io(std::io::Error),
    /// UTF-8 Error
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    /// String From UTF-8 Error
    #[error(transparent)]
    FromUtf8(#[from] std::string::FromUtf8Error),
    /// Unexpected Error
    #[error("unexpected")]
    Unexpected,
    /// A custom message from serde or this crate.
    #[error("{0}")]
    Message(String),
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

/// Alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
