use std::io::{self, Write};

use termcolor::{StandardStream, StandardStreamLock, ColorSpec, Color, WriteColor};

use crate::parser::FilePos;

pub trait DiagnosticsWriter {
    fn write_error(&mut self, pos: Option<FilePos>, message: &str) -> io::Result<()>;
    fn write_warning(&mut self, pos: Option<FilePos>, message: &str) -> io::Result<()>;
    fn write_info(&mut self, pos: Option<FilePos>, message: &str) -> io::Result<()>;
    fn write_note(&mut self, pos: Option<FilePos>, message: &str) -> io::Result<()>;
    fn write_help(&mut self, pos: Option<FilePos>, message: &str) -> io::Result<()>;
    fn write_newline(&mut self) -> io::Result<()>;
}

impl DiagnosticsWriter for StandardStream {
    fn write_error(&mut self, pos: Option<FilePos>, message: &str) -> io::Result<()> {
        write_message(self.lock(), pos, "error:", Color::Red, message)
    }

    fn write_warning(&mut self, pos: Option<FilePos>, message: &str) -> io::Result<()> {
        write_message(self.lock(), pos, "warning:", Color::Yellow, message)
    }

    fn write_info(&mut self, pos: Option<FilePos>, message: &str) -> io::Result<()> {
        write_message(self.lock(), pos, "info:", Color::White, message)
    }

    fn write_note(&mut self, pos: Option<FilePos>, message: &str) -> io::Result<()> {
        write_message(self.lock(), pos, "note:", Color::Green, message)
    }

    fn write_help(&mut self, pos: Option<FilePos>, message: &str) -> io::Result<()> {
        write_message(self.lock(), pos, "help:", Color::Blue, message)
    }

    fn write_newline(&mut self) -> io::Result<()> {
        writeln!(self.lock())
    }
}

fn write_message(
    mut out: StandardStreamLock,
    pos: Option<FilePos>,
    prefix: &str,
    prefix_color: Color,
    message: &str,
) -> io::Result<()> {
    if let Some(FilePos {path, start_line, start_offset, end_line, end_offset}) = pos {
        if start_line == end_line && start_offset == end_offset-1 {
            write!(out, "[{}:{}:{}] ", path.display(), start_line, start_offset)?;
        } else {
            // end offset is always one past the end
            write!(out, "[{}:{}:{}-{}:{}] ", path.display(), start_line, start_offset, end_line, end_offset-1)?;
        }
    }

    out.set_color(ColorSpec::new().set_fg(Some(prefix_color)).set_bold(true))?;
    write!(out, "{} ", prefix)?;
    out.reset()?;

    writeln!(out, "{}", message)
}

#[cfg(test)]
pub struct NullWriter;

#[cfg(test)]
impl NullWriter {
    pub fn new(_color_choice: termcolor::ColorChoice) -> Self {
        // This impl exists to silence an unused parameter warning
        NullWriter
    }
}

#[cfg(test)]
impl DiagnosticsWriter for NullWriter {
    fn write_error(&mut self, _pos: Option<FilePos>, _message: &str) -> io::Result<()> {
        Ok(())
    }

    fn write_warning(&mut self, _pos: Option<FilePos>, _message: &str) -> io::Result<()> {
        Ok(())
    }

    fn write_info(&mut self, _pos: Option<FilePos>, _message: &str) -> io::Result<()> {
        Ok(())
    }

    fn write_note(&mut self, _pos: Option<FilePos>, _message: &str) -> io::Result<()> {
        Ok(())
    }

    fn write_help(&mut self, _pos: Option<FilePos>, _message: &str) -> io::Result<()> {
        Ok(())
    }

    fn write_newline(&mut self) -> io::Result<()> {
        Ok(())
    }
}
