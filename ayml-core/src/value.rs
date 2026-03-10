use std::collections::HashMap;
use std::fmt;

/// A node in the AYML document tree.
///
/// Wraps a [`Value`] with optional comment metadata. The reference
/// implementation preserves comments; other implementations may discard them.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Comments on the line(s) immediately above this node.
    /// Multi-line comments are joined with newlines. The `#` prefix and
    /// leading whitespace are stripped.
    pub comment: Option<String>,
    /// Inline comment trailing this node on the same line.
    pub inline_comment: Option<String>,
    /// The value of this node.
    pub value: Value,
}

impl Node {
    /// Create a node with no comments.
    pub fn new(value: Value) -> Self {
        Self {
            comment: None,
            inline_comment: None,
            value,
        }
    }

    /// Create a node with a top comment.
    pub fn with_comment(value: Value, comment: impl Into<String>) -> Self {
        Self {
            comment: Some(comment.into()),
            inline_comment: None,
            value,
        }
    }
}

/// A key in an AYML mapping.
///
/// Restricted to hashable, equality-comparable types: bool, integer, and
/// string. Float and null keys are not allowed per the spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapKey {
    Bool(bool),
    Int(i64),
    String(String),
}

impl fmt::Display for MapKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MapKey::Bool(b) => write!(f, "{b}"),
            MapKey::Int(i) => write!(f, "{i}"),
            MapKey::String(s) => write!(f, "{s}"),
        }
    }
}

/// An AYML value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Sequence(Vec<Node>),
    Mapping(HashMap<MapKey, Node>),
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_scalar(&self) -> bool {
        !matches!(self, Value::Sequence(_) | Value::Mapping(_))
    }

    pub fn is_collection(&self) -> bool {
        matches!(self, Value::Sequence(_) | Value::Mapping(_))
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_sequence(&self) -> Option<&[Node]> {
        match self {
            Value::Sequence(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_mapping(&self) -> Option<&HashMap<MapKey, Node>> {
        match self {
            Value::Mapping(m) => Some(m),
            _ => None,
        }
    }
}
