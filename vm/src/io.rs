use std::char;
use std::io::{self, BufRead, Write};

#[derive(Debug, Default, PartialEq)]
pub struct Stdio {
    line: Vec<u8>,
    /// The current index into the line
    current: usize,
}

impl Stdio {
    /// Reads the next line of input from stdin
    ///
    /// Returns Ok(None) if EOF has been reached
    #[cfg(not(test))]
    pub fn read_byte(&mut self) -> io::Result<Option<u8>> {
        if self.current >= self.line.len() {
            let stdin = io::stdin();
            stdin.lock().read_until(b'\n', &mut self.line)?;
            self.current = 0;
        }

        Ok(self.line.get(self.current).copied().map(|byte| {
            // Found a character, advance the current index
            // This avoids `current` being incremented after EOF
            self.current += 1;
            byte
        }))
    }

    #[cfg(test)]
    pub fn read_byte(&mut self) -> io::Result<Option<u8>> {
        Ok(None)
    }

    /// Writes the given 4 bytes to stdout, printing the unicode replacement
    /// character if the bytes are not a valid `char`
    #[cfg(not(test))]
    pub fn write_bytes(&self, value: u32) -> io::Result<()> {
        let ch = char::from_u32(value)
            .unwrap_or(char::REPLACEMENT_CHARACTER);

        let mut stdout = io::stdout();
        write!(stdout, "{}", ch)
    }

    #[cfg(test)]
    pub fn write_bytes(&self, _value: u32) -> io::Result<()> {
        Ok(())
    }
}
