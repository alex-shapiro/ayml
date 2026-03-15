#![warn(clippy::all, clippy::pedantic)]

mod commented;
mod commented_value;
pub mod de;
mod error;
mod read;
pub mod ser;
mod value;

pub use commented::Commented;
pub use commented_value::{CommentedValue, CommentedValueKind};
pub use error::{Error, Result};
pub use value::Value;

pub use de::{from_reader, from_slice, from_str};
pub use ser::{to_string, to_vec, to_writer};
