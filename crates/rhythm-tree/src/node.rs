//! A node of a rhythm tree: an equal split into `n` slots.

use crate::issue::ParseError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Node {
    pub(crate) n: u32,
}

/// The most slots one node splits into.
pub(crate) const MAX_SLOTS: u32 = 64;

/// A read position in link text.
pub(crate) struct Cursor<'a> {
    s: &'a str,
    at: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(s: &'a str) -> Cursor<'a> {
        Cursor { s, at: 0 }
    }

    /// The digit run at the cursor, moving past it.
    pub(crate) fn int(&mut self) -> Result<u32, ParseError> {
        let digits = self.rest().bytes().take_while(u8::is_ascii_digit).count();
        let n = int(&self.rest()[..digits]).ok_or_else(|| ParseError::ExpectedNumber {
            at: self.at,
            text: self.s.to_string(),
        })?;
        self.at += digits;
        Ok(n)
    }

    /// Moves past `b` when it is next.
    pub(crate) fn eat(&mut self, b: u8) -> bool {
        let next = self.rest().as_bytes().first() == Some(&b);
        if next {
            self.at += 1;
        }
        next
    }

    pub(crate) fn rest(&self) -> &str {
        &self.s[self.at..]
    }
}

/// A run of digits, saturating at `u32::MAX`.
pub(crate) fn int(s: &str) -> Option<u32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(s.bytes().fold(0u32, |n, b| {
        n.saturating_mul(10).saturating_add(u32::from(b - b'0'))
    }))
}
