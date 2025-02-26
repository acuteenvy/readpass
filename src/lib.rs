//! Read passwords without displaying them on the terminal.
//! Works on Unix-like OSes and Windows.
//!
//! # Usage
//!
//! Read a password:
//!
//!```rust,no_run
//! let passwd = readpass::from_tty()?;
//! # Ok::<(), std::io::Error>(())
//!```
//!
//! If you want to display a prompt, print it to stdout or stderr before reading:
//!
//!```rust,no_run
//! use std::io::{self, Write};
//!
//! writeln!(io::stderr(), "Please enter a password: ")?;
//! let passwd = readpass::from_tty()?;
//! # Ok::<(), io::Error>(())
//!```

use std::io::{self, BufRead};

use zeroize::Zeroizing;

#[cfg(target_family = "unix")]
mod unix;
#[cfg(target_family = "unix")]
use unix as sys;

#[cfg(target_family = "windows")]
mod windows;
#[cfg(target_family = "windows")]
use windows as sys;

pub use sys::from_tty;

/// Reads a password from an `impl BufRead`.
///
/// This only reads the first line from the reader.
pub fn from_bufread(reader: &mut impl BufRead) -> io::Result<Zeroizing<String>> {
    let mut password = Zeroizing::new(String::new());
    reader.read_line(&mut password)?;

    let len = password.trim_end_matches(&['\r', '\n'][..]).len();
    password.truncate(len);

    // Ctrl-U should remove the line in terminals.
    if password.contains('') {
        password = match password.rfind('') {
            Some(last_ctrl_u_index) => password[last_ctrl_u_index + 1..].to_string().into(),
            None => password,
        };
    }

    Ok(password)
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
        assert_eq!(*response, "A mocked response.");
        let response = super::from_bufread(&mut reader_crlf).unwrap();
        assert_eq!(*response, "Another mocked response.");

        let mut reader_lf = mock_input_lf();
        let response = super::from_bufread(&mut reader_lf).unwrap();
        assert_eq!(*response, "A mocked response.");
        let response = super::from_bufread(&mut reader_lf).unwrap();
        assert_eq!(*response, "Another mocked response.");
    }
}
