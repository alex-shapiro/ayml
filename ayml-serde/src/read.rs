use crate::error::{Error, Result};

/// Trait abstracting over input sources for the deserializer.
///
/// Provides lazy access to a `&str` buffer and a byte offset cursor.
/// For [`StrRead`], all data is available immediately. For [`IoRead`],
/// data is read from the underlying reader on demand.
pub(crate) trait Read<'de> {
    /// Ensure at least `pos` bytes of input are available.
    /// Returns `Ok(true)` if `pos` bytes are available, `Ok(false)` if
    /// EOF was reached first.
    fn fill_to(&mut self, pos: usize) -> Result<bool>;

    /// The full available input text (up to what has been filled so far).
    fn input(&self) -> &str;

    /// Current byte offset into the input.
    fn offset(&self) -> usize;

    /// Set the byte offset (for save/restore backtracking within already-available data).
    fn set_offset(&mut self, offset: usize);
}

// ── StrRead ──────────────────────────────────────────────────────

/// Read implementation over a borrowed `&str`.
/// All data is available immediately; [`fill_to`](Read::fill_to) is a no-op.
pub(crate) struct StrRead<'a> {
    input: &'a str,
    offset: usize,
}

impl<'a> StrRead<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, offset: 0 }
    }
}

impl<'a> Read<'a> for StrRead<'a> {
    #[inline]
    fn fill_to(&mut self, _pos: usize) -> Result<bool> {
        Ok(true)
    }

    #[inline]
    fn input(&self) -> &str {
        self.input
    }

    #[inline]
    fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }
}

// ── IoRead ───────────────────────────────────────────────────────

/// Read implementation over an `io::Read`, buffering lazily.
///
/// Data is read from the underlying reader in chunks as the deserializer
/// advances. The internal buffer grows monotonically and is never compacted
/// during deserialization, so backtracking within already-read data is safe.
pub(crate) struct IoRead<R> {
    reader: R,
    buf: String,
    offset: usize,
    done: bool,
    /// Leftover bytes from incomplete UTF-8 sequence at chunk boundary.
    pending: Vec<u8>,
}

impl<R: std::io::Read> IoRead<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: String::new(),
            offset: 0,
            done: false,
            pending: Vec::new(),
        }
    }
}

/// Maximum buffer size for `IoRead` (256 MiB). Prevents OOM from
/// unbounded input when reading from an `io::Read` source.
const MAX_IO_BUF: usize = 256 * 1024 * 1024;

impl<R: std::io::Read> Read<'_> for IoRead<R> {
    fn fill_to(&mut self, pos: usize) -> Result<bool> {
        if pos > MAX_IO_BUF {
            return Err(Error::Message(format!(
                "input exceeds maximum size ({MAX_IO_BUF} bytes)"
            )));
        }
        while self.buf.len() < pos && !self.done {
            let mut tmp = [0u8; 4096];
            let n = self.reader.read(&mut tmp).map_err(Error::from)?;
            if n == 0 {
                self.done = true;
                if !self.pending.is_empty() {
                    return Err(Error::Message(
                        "invalid UTF-8: incomplete sequence at end of input".into(),
                    ));
                }
                break;
            }

            // Combine any pending bytes from a previous incomplete UTF-8 sequence
            // with the new chunk.
            let mut to_decode = std::mem::take(&mut self.pending);
            to_decode.extend_from_slice(&tmp[..n]);

            match std::str::from_utf8(&to_decode) {
                Ok(s) => {
                    self.buf.push_str(s);
                }
                Err(e) => {
                    let valid_up_to = e.valid_up_to();
                    // Safety: from_utf8 guarantees bytes[..valid_up_to] is valid UTF-8.
                    self.buf.push_str(unsafe {
                        std::str::from_utf8_unchecked(&to_decode[..valid_up_to])
                    });

                    if e.error_len().is_some() {
                        // Genuinely invalid byte (not just an incomplete sequence).
                        return Err(Error::Message(format!(
                            "invalid UTF-8 at byte {}",
                            self.buf.len()
                        )));
                    }
                    // Incomplete sequence at end of chunk — save for next read.
                    self.pending = to_decode[valid_up_to..].to_vec();
                }
            }
        }
        Ok(self.buf.len() >= pos)
    }

    #[inline]
    fn input(&self) -> &str {
        &self.buf
    }

    #[inline]
    fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }
}
