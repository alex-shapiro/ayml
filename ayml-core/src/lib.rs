mod error;
mod value;

pub mod parser;
pub mod emitter;

pub use error::{Error, ErrorKind, Span};
pub use value::{MapKey, Node, Value};

/// Parse an AYML document from a string.
pub fn parse(input: &str) -> Result<Node, Error> {
    parser::parse(input)
}

/// Emit an AYML document to a string.
pub fn emit(node: &Node) -> String {
    emitter::emit(node)
}
