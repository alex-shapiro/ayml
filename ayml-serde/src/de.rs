use serde::de::DeserializeOwned;

use crate::error::Result;

/// Deserialize a `T` from a string of AYML text.
pub fn from_str<T: DeserializeOwned>(_s: &str) -> Result<T> {
    todo!()
}

/// Deserialize a `T` from a slice of AYML bytes.
pub fn from_slice<T: DeserializeOwned>(_bytes: &[u8]) -> Result<T> {
    todo!()
}

/// Deserialize a `T` from an AYML reader.
pub fn from_reader<R: std::io::Read, T: DeserializeOwned>(_rdr: R) -> Result<T> {
    todo!()
}
