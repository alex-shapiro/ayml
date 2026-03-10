#![warn(clippy::all, clippy::pedantic)]

mod error;
mod value;

pub mod emitter;
pub mod parser;

pub use error::{Error, ErrorKind, Span};
pub use value::{MapKey, Node, Value};

/// Parse an AYML document from a string.
///
/// # Errors
/// Returns an [`Error`] if the input is not valid AYML.
pub fn parse(input: &str) -> Result<Node, Error> {
    parser::parse(input)
}

/// Emit an AYML document to a string.
#[must_use]
pub fn emit(node: &Node) -> String {
    emitter::emit(node)
}
