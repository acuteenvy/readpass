//! A tiny library for reading passwords without displaying them on the terminal.
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
//! write!(io::stderr(), "Please enter a password: ")?;
//! let passwd = readpass::from_tty()?;
//! # Ok::<(), io::Error>(())
//!```
//!
//! [`String`]s returned by `readpass` are wrapped in [`Zeroizing`]
//! to ensure the password is zeroized from memory after it's [`Drop`]ped.

use std::io::{self, BufRead};

use zeroize::Zeroizing;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use unix as sys;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows as sys;

pub use sys::from_tty;

const CTRL_U: char = char::from_u32(21).unwrap();

/// Reads a password from an `impl BufRead`.
///
/// This only reads the first line from the reader.
/// Newlines and carriage returns are trimmed from the end of the resulting [`String`].
fn from_bufread(reader: &mut impl BufRead) -> io::Result<Zeroizing<String>> {
    let mut password = Zeroizing::new(String::new());
    reader.read_line(&mut password)?;

    let len = password.trim_end_matches(&['\r', '\n'][..]).len();
    password.truncate(len);

    // Ctrl-U should remove the line in terminals.
    password = match password.rfind(CTRL_U) {
        Some(last_ctrl_u_index) => Zeroizing::new(password[last_ctrl_u_index + 1..].to_string()),
        None => password,
    };

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

    // These tests check whether or not we can read from a reader when
    // stdin is not a terminal.

    #[cfg(unix)]
    fn close_stdin() {
        unsafe {
            libc::close(libc::STDIN_FILENO);
        }
    }

    #[cfg(windows)]
    fn close_stdin() {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Console::{GetStdHandle, STD_INPUT_HANDLE};

        unsafe {
            CloseHandle(GetStdHandle(STD_INPUT_HANDLE));
        }
    }

    #[cfg(not(any(unix, windows)))]
    fn close_stdin() {
        unimplemented!()
    }

    #[test]
    fn can_read_from_redirected_input_many_times_nostdin() {
        close_stdin();
        can_read_from_redirected_input_many_times();
    }

    #[test]
    fn can_read_from_input_ctrl_u() {
        close_stdin();

        let s = format!(
            "A mocked response.{}Another mocked response.\n",
            super::CTRL_U
        );
        let mut reader_ctrl_u = Cursor::new(s.as_bytes());
        let response = super::from_bufread(&mut reader_ctrl_u).unwrap();
        assert_eq!(*response, "Another mocked response.");

        let s = format!("A mocked response.{}\n", super::CTRL_U);
        let mut reader_ctrl_u_at_end = Cursor::new(s.as_bytes());
        let response = super::from_bufread(&mut reader_ctrl_u_at_end).unwrap();
        assert_eq!(*response, "");
    }
}
