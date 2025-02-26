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

#[cfg(target_family = "unix")]
mod unix {
    use std::fs::File;
    use std::io::{self, BufRead, BufReader};
    use std::mem::MaybeUninit;
    use std::os::unix::io::AsRawFd;

    use libc::{c_int, tcsetattr, termios, ECHO, ECHONL, TCSANOW};

    struct HiddenInput {
        fd: i32,
        term_orig: termios,
    }

    impl HiddenInput {
        fn new(fd: i32) -> io::Result<HiddenInput> {
            // Make two copies of the terminal settings. The first one will be modified
            // and the second one will act as a backup for when we want to set the
            // terminal back to its original state.
            let mut term = safe_tcgetattr(fd)?;
            let term_orig = safe_tcgetattr(fd)?;

            // Hide the password. This is what makes this function useful.
            term.c_lflag &= !ECHO;

            // But don't hide the NL character when the user hits ENTER.
            term.c_lflag |= ECHONL;

            // Save the settings for now.
            io_result(unsafe { tcsetattr(fd, TCSANOW, &term) })?;

            Ok(HiddenInput { fd, term_orig })
        }
    }

    impl Drop for HiddenInput {
        fn drop(&mut self) {
            // Set the the mode back to normal
            unsafe {
                tcsetattr(self.fd, TCSANOW, &self.term_orig);
            }
        }
    }

    /// Turns a C function return into an IO Result.
    fn io_result(ret: c_int) -> io::Result<()> {
        match ret {
            0 => Ok(()),
            _ => Err(io::Error::last_os_error()),
        }
    }

    fn safe_tcgetattr(fd: c_int) -> io::Result<termios> {
        let mut term = MaybeUninit::<termios>::uninit();
        io_result(unsafe { ::libc::tcgetattr(fd, term.as_mut_ptr()) })?;
        Ok(unsafe { term.assume_init() })
    }

    /// Reads a password from the TTY.
    pub fn from_tty() -> io::Result<String> {
        let tty = File::open("/dev/tty")?;
        let fd = tty.as_raw_fd();
        let mut reader = BufReader::new(tty);

        from_fd_with_hidden_input(&mut reader, fd)
    }

    /// Reads a password from a given file descriptor.
    fn from_fd_with_hidden_input(reader: &mut impl BufRead, fd: i32) -> io::Result<String> {
        let mut password = String::new();
        let hidden_input = HiddenInput::new(fd)?;

        reader.read_line(&mut password)?;
        drop(hidden_input);

        super::fix_line_issues(password)
    }
}

#[cfg(target_family = "windows")]
mod windows {
    use std::fs::File;
    use std::io::{self, BufRead, BufReader};
    use std::os::windows::io::FromRawHandle;

    use windows_sys::core::PCSTR;
    use windows_sys::Win32::Foundation::{
        GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileA, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, SetConsoleMode, CONSOLE_MODE, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT,
    };

    struct HiddenInput {
        mode: u32,
        handle: HANDLE,
    }

    impl HiddenInput {
        fn new(handle: HANDLE) -> io::Result<HiddenInput> {
            let mut mode = 0;

            // Get the old mode so we can reset back to it when we are done
            if unsafe { GetConsoleMode(handle, &mut mode as *mut CONSOLE_MODE) } == 0 {
                return Err(std::io::Error::last_os_error());
            }

            // We want to be able to read line by line, and we still want backspace to work
            let new_mode_flags = ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT;
            if unsafe { SetConsoleMode(handle, new_mode_flags) } == 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(HiddenInput { mode, handle })
        }
    }

    impl Drop for HiddenInput {
        fn drop(&mut self) {
            // Set the the mode back to normal
            unsafe {
                SetConsoleMode(self.handle, self.mode);
            }
        }
    }

    /// Reads a password from the TTY.
    pub fn from_tty() -> io::Result<String> {
        let handle = unsafe {
            CreateFileA(
                b"CONIN$\x00".as_ptr() as PCSTR,
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                0,
                INVALID_HANDLE_VALUE,
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }

        let mut stream = BufReader::new(unsafe { File::from_raw_handle(handle as _) });
        from_handle_with_hidden_input(&mut stream, handle)
    }

    /// Reads a password from a given file handle.
    fn from_handle_with_hidden_input(
        reader: &mut impl BufRead,
        handle: HANDLE,
    ) -> io::Result<String> {
        let mut password = String::new();
        let hidden_input = HiddenInput::new(handle)?;

        let reader_return = reader.read_line(&mut password);

        // Newline for windows which otherwise prints on the same line.
        writeln!(io::stdout())?;

        if reader_return.is_err() {
            return Err(reader_return.unwrap_err());
        }

        drop(hidden_input);
        super::fix_line_issues(password)
    }
}

#[cfg(target_family = "unix")]
pub use unix::from_tty;
#[cfg(target_family = "windows")]
pub use windows::from_tty;

/// Reads a password from an `impl BufRead`.
///
/// This only reads the first line from the reader.
pub fn from_bufread(reader: &mut impl BufRead) -> io::Result<String> {
    let mut password = String::new();
    reader.read_line(&mut password)?;

    fix_line_issues(password)
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
