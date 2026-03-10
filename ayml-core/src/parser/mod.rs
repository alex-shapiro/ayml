mod grammar;
mod scanner;

use crate::error::Error;
use crate::value::Node;

/// Parse an AYML document from a string, returning the root node.
pub fn parse(input: &str) -> Result<Node, Error> {
    let mut parser = grammar::Parser::new(input);
    parser.parse_document()
}
