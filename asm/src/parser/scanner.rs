use std::str;

use super::span::Span;
use super::source_files::FileSource;

#[derive(Debug)]
pub struct Scanner<'a> {
    source: FileSource<'a>,
    current: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: FileSource<'a>) -> Self {
        Self {
            source,
            current: 0,
        }
    }

    /// Returns the current position (in bytes) in the source
    pub fn current_pos(&self) -> usize {
        self.current
    }

    /// Returns the next character in the source text or returns None if there are no more left
    pub fn next(&mut self) -> Option<u8> {
        let ch = self.peek()?;
        self.current += 1;
        Some(ch)
    }

    /// Returns the next character in the source text, but does not advance the scanner
    pub fn peek(&self) -> Option<u8> {
        self.source.get(self.current)
    }

    /// Creates a new span that is empty (from `index` to `index`)
    pub fn empty_span(&self, index: usize) -> Span {
        self.span(index, index)
    }

    /// Creates a new span for a single byte
    pub fn byte_span(&self, index: usize) -> Span {
        self.span(index, index+1)
    }

    /// Creates a new span between the given byte indexes
    ///
    /// `start` is included in the range, `end` is not.
    pub fn span(&self, start: usize, end: usize) -> Span {
        Span {start, end}
    }

    /// Creates a new slice of the source between the given byte indexes and parse it as unicode
    ///
    /// `start` is included in the range, `end` is not.
    ///
    /// # Panics
    ///
    /// Panics if the sliced source bytes are not valid unicode.
    pub fn slice(&self, start: usize, end: usize) -> &'a str {
        str::from_utf8(self.source.slice(start..end))
            .expect("bug: not valid unicode")
    }
}
