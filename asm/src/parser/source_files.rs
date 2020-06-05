use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::ops::Range;
use std::cmp::max;
use std::iter::once;

use super::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileHandle {
    /// The index of the first byte of the file in `SourceFiles::source`
    start: usize,
    /// The number of bytes in the file
    len: usize,
}

/// The source for a file, represented as a slice of bytes and indexed from `start_index()` onwards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileSource<'a> {
    /// A slice of `SourceFiles::source`
    bytes: &'a [u8],
    /// The offset at which the slice of bytes was extracted from `SourceFiles::source`
    offset: usize,
}

impl<'a> FileSource<'a> {
    /// Returns the first index into this slice
    pub fn start_index(&self) -> usize {
        self.offset
    }

    /// Returns the number of bytes in the file
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns the byte in this file at the given index, if any
    ///
    /// Note that the index MUST be greater than or equal to `start_index()` for this method to
    /// return anything.
    pub fn get(&self, index: usize) -> Option<u8> {
        let index = index - self.offset;
        self.bytes.get(index).copied()
    }

    /// Slices from the bytes of this file's source
    pub fn slice(&self, range: Range<usize>) -> &'a [u8] {
        let Self {bytes, offset} = self;
        let Range {start, end} = range;

        &bytes[start-offset..end-offset]
    }

    /// Iterates over the bytes of this file's source, yielding the offset for each one
    pub fn iter_bytes(&self) -> impl Iterator<Item=(usize, u8)> + '_ {
        self.bytes.iter().copied().enumerate().map(move |(index, byte)| (self.offset + index, byte))
    }
}

#[derive(Debug)]
pub struct LineNumbers {
    /// The index in `SourceFiles::source` of the start of each line
    ///
    /// An index into this field is 1 less than the line number of
    /// the offset contained at that index.
    offsets: Vec<usize>,
}

impl LineNumbers {
    pub fn new(source: FileSource) -> Self {
        // There is always at least one line, starting at offset 0
        let offsets = once(source.start_index()).chain(source.iter_bytes().filter_map(|(offset, ch)| match ch {
            // Each line starts right *after* each newline
            b'\n' => Some(offset),
            _ => None,
        })).collect();

        Self {offsets}
    }

    /// Returns the line number corresponding to the given index in the source file
    pub fn number(&self, index: usize) -> usize {
        // Edge case: very first offset will give you a line number of zero, which we correct to 1
        max(self.offsets.binary_search(&index).unwrap_or_else(|index| index), 1)
    }
}

#[derive(Debug)]
struct File {
    path: PathBuf,
    /// The index into `SourceFiles::source` that represents the start of this file
    start_offset: usize,
    /// An index of the line numbers for all offsets in the file
    line_numbers: LineNumbers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePos<'a> {
    pub path: &'a Path,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Default)]
pub struct SourceFiles {
    /// The source code of all files concatenated together.
    ///
    /// This allows spans across files to be uniquely identifiable
    source: Vec<u8>,
    /// Metadata about each file stored in `source`
    ///
    /// Sorted by the offset
    files: Vec<File>,
}

impl SourceFiles {
    /// Reads a file and adds it to the set of source files. Returns a handle to that file's
    /// contents.
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<FileHandle> {
        let path = path.as_ref();
        let start = self.source.len();

        let mut file = fs::File::open(path)?;
        file.read_to_end(&mut self.source)?;

        let len = self.source.len() - start;
        Ok(self.create_handle(path, start, len))
    }

    /// Adds the given source to the set of source files. Returns a handle to that file's
    /// contents.
    pub fn add_source<P: AsRef<Path>>(&mut self, path: P, source: &[u8]) -> FileHandle {
        let path = path.as_ref();
        let start = self.source.len();

        self.source.extend(source);

        let len = source.len();
        self.create_handle(path, start, len)
    }

    fn create_handle(&mut self, path: &Path, start: usize, len: usize) -> FileHandle {
        let handle = FileHandle {start, len};
        let source = self.source(handle);
        let line_numbers = LineNumbers::new(source);
        self.files.push(File {
            path: path.to_path_buf(),
            start_offset: start,
            line_numbers,
        });
        handle
    }

    /// Returns the resolved file and position information for a span
    pub fn pos(&self, span: Span) -> FilePos {
        let file = self.file(span.start);
        FilePos {
            path: &file.path,
            start_line: file.line_numbers.number(span.start),
            end_line: file.line_numbers.number(span.end),
        }
    }

    /// Returns the path of the file whose source contains the given index
    pub fn path(&self, index: usize) -> &Path {
        &self.file(index).path
    }

    /// Returns the line number corresponding to the given index
    ///
    /// Automatically looks up the correct file and returns the line number in that file
    pub fn line(&self, index: usize) -> usize {
        self.file(index).line_numbers.number(index)
    }

    /// Returns the source for the given file handle
    pub fn source(&self, handle: FileHandle) -> FileSource {
        let FileHandle {start, len} = handle;
        FileSource {
            bytes: &self.source[start..start+len],
            offset: start,
        }
    }

    fn file(&self, index: usize) -> &File {
        let file_index = self.files.binary_search_by_key(&index, |file| file.start_offset)
            .unwrap_or_else(|index| index - 1);
        &self.files[file_index]
    }
}
