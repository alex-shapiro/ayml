use std::collections::VecDeque;
use std::io::BufRead;

use crate::error::{Error, Result};

/// Byte-level peekable reader trait for the deserializer.
pub(crate) trait Read {
    /// Peek at the next byte without consuming it.
    fn peek(&mut self) -> Result<Option<u8>>;

    /// Peek at the byte `n` positions ahead (0 = same as `peek()`).
    fn peek_at(&mut self, n: usize) -> Result<Option<u8>>;

    /// Consume and return the next byte.
    fn next(&mut self) -> Result<Option<u8>>;

    /// Current byte offset from the start of input.
    fn byte_offset(&self) -> usize;
}

// ── SliceRead ──────────────────────────────────────────────────

/// Read implementation over a borrowed `&[u8]` slice.
pub(crate) struct SliceRead<'a> {
    slice: &'a [u8],
    index: usize,
}

impl<'a> SliceRead<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        Self { slice, index: 0 }
    }
}

impl Read for SliceRead<'_> {
    #[inline]
    fn peek(&mut self) -> Result<Option<u8>> {
        Ok(self.slice.get(self.index).copied())
    }

    #[inline]
    fn peek_at(&mut self, n: usize) -> Result<Option<u8>> {
        Ok(self.slice.get(self.index + n).copied())
    }

    #[inline]
    fn next(&mut self) -> Result<Option<u8>> {
        match self.slice.get(self.index) {
            Some(&b) => {
                self.index += 1;
                Ok(Some(b))
            }
            None => Ok(None),
        }
    }

    #[inline]
    fn byte_offset(&self) -> usize {
        self.index
    }
}

// ── IoRead ─────────────────────────────────────────────────────

/// Read implementation over an `io::Read`, using a small lookahead buffer.
pub(crate) struct IoRead<R> {
    reader: std::io::BufReader<R>,
    /// Lookahead buffer for `peek_at` support.
    buf: VecDeque<u8>,
    /// Total bytes consumed (returned via next/discard) so far.
    offset: usize,
    done: bool,
}

impl<R: std::io::Read> IoRead<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: std::io::BufReader::new(reader),
            buf: VecDeque::new(),
            offset: 0,
            done: false,
        }
    }

    /// Ensure `buf` has at least `n` bytes (if available from the reader).
    fn fill_buf_to(&mut self, n: usize) -> Result<()> {
        while self.buf.len() < n && !self.done {
            let available = self.reader.fill_buf().map_err(Error::from)?;
            if available.is_empty() {
                self.done = true;
                break;
            }
            let need = n - self.buf.len();
            let take = need.min(available.len());
            self.buf.extend(&available[..take]);
            self.reader.consume(take);
        }
        Ok(())
    }
}

impl<R: std::io::Read> Read for IoRead<R> {
    #[inline]
    fn peek(&mut self) -> Result<Option<u8>> {
        if self.buf.is_empty() {
            self.fill_buf_to(1)?;
        }
        Ok(self.buf.front().copied())
    }

    #[inline]
    fn peek_at(&mut self, n: usize) -> Result<Option<u8>> {
        if self.buf.len() <= n {
            self.fill_buf_to(n + 1)?;
        }
        Ok(self.buf.get(n).copied())
    }

    #[inline]
    fn next(&mut self) -> Result<Option<u8>> {
        if self.buf.is_empty() {
            self.fill_buf_to(1)?;
        }
        match self.buf.pop_front() {
            Some(b) => {
                self.offset += 1;
                Ok(Some(b))
            }
            None => Ok(None),
        }
    }

    #[inline]
    fn byte_offset(&self) -> usize {
        self.offset
    }
}
