use serde::Serialize;

use crate::error::Result;

/// Serialize a `T` to an AYML string.
pub fn to_string<T: Serialize>(_value: &T) -> Result<String> {
    todo!()
}

/// Serialize a `T` to an AYML byte vector.
pub fn to_vec<T: Serialize>(_value: &T) -> Result<Vec<u8>> {
    todo!()
}

/// Serialize a `T` as AYML into a writer.
pub fn to_writer<W: std::io::Write, T: Serialize>(_writer: W, _value: &T) -> Result<()> {
    todo!()
}
