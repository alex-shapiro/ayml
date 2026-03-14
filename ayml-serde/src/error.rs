use std::fmt;

/// Error type for ayml-serde serialization and deserialization.
#[derive(Debug)]
pub enum Error {
    /// An error from the ayml-core parser.
    Parse(ayml_core::Error),
    /// A custom message from serde or this crate.
    Message(String),
    /// An I/O error (from `from_reader` / `to_writer`).
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "{e}"),
            Error::Message(msg) => f.write_str(msg),
            Error::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Parse(e) => Some(e),
            Error::Message(_) => None,
            Error::Io(e) => Some(e),
        }
    }
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

impl From<ayml_core::Error> for Error {
    fn from(e: ayml_core::Error) -> Self {
        Error::Parse(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

/// Alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
