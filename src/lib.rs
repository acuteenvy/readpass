//! Read passwords without displaying them on the terminal.
//! Works on Unix OSes and Windows.
//!
//! # Usage
//!
//! Read a password:
//!
//!```rust,no_run
//! let passwd = readpass::from_tty()?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! If you want to display a prompt, print it to stdout or stderr before reading:
//! ```rust,no_run
//! use std::io::{self, Write};
//!
//! writeln!(io::stderr(), "Please enter a password: ")?;
//! let passwd = readpass::from_tty()?;
//! # Ok::<(), io::Error>(())
//! ```

use std::io::{self, BufRead};

#[cfg(target_family = "unix")]
mod unix;
#[cfg(target_family = "unix")]
use unix as sys;

#[cfg(target_family = "windows")]
mod windows;
#[cfg(target_family = "windows")]
use windows as sys;

/// Reads a password from the TTY.
pub use sys::from_tty;

/// Reads a password from an `impl BufRead`.
///
/// This only reads the first line from the reader.
pub fn from_bufread(reader: &mut impl BufRead) -> io::Result<String> {
    let mut password = String::new();
    reader.read_line(&mut password)?;

    fix_line_issues(password)
}

/// Normalizes the return of `read_line()` in the context of a CLI application.
fn fix_line_issues(mut line: String) -> io::Result<String> {
    if !line.ends_with('\n') {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "unexpected end of file",
        ));
    }

    // Remove the \n from the line.
    line.pop();

    // Remove the \r from the line if present
    if line.ends_with('\r') {
        line.pop();
    }

    // Ctrl-U should remove the line in terminals
    if line.contains('') {
        line = match line.rfind('') {
            Some(last_ctrl_u_index) => line[last_ctrl_u_index + 1..].to_string(),
            None => line,
        };
    }

    Ok(line)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    fn mock_input_crlf() -> Cursor<&'static [u8]> {
        Cursor::new(&b"A mocked response.\r\nAnother mocked response.\r\n"[..])
    }

    fn mock_input_lf() -> Cursor<&'static [u8]> {
        Cursor::new(&b"A mocked response.\nAnother mocked response.\n"[..])
    }

    #[test]
    fn can_read_from_redirected_input_many_times() {
        let mut reader_crlf = mock_input_crlf();

        let response = super::from_bufread(&mut reader_crlf).unwrap();
        assert_eq!(response, "A mocked response.");
        let response = super::from_bufread(&mut reader_crlf).unwrap();
        assert_eq!(response, "Another mocked response.");

        let mut reader_lf = mock_input_lf();
        let response = super::from_bufread(&mut reader_lf).unwrap();
        assert_eq!(response, "A mocked response.");
        let response = super::from_bufread(&mut reader_lf).unwrap();
        assert_eq!(response, "Another mocked response.");
    }
}
