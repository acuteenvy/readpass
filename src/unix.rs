use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::mem::MaybeUninit;
use std::os::unix::io::AsRawFd;

use libc::{c_int, tcsetattr, termios, ECHO, ECHONL, TCSANOW};
use zeroize::Zeroizing;

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
///
/// Newlines and carriage returns are trimmed from the end of the resulting `String`.
pub fn from_tty() -> io::Result<Zeroizing<String>> {
    let tty = File::open("/dev/tty")?;
    let fd = tty.as_raw_fd();
    let mut reader = BufReader::new(tty);

    from_fd_with_hidden_input(&mut reader, fd)
}

/// Reads a password from a given file descriptor.
fn from_fd_with_hidden_input(reader: &mut impl BufRead, fd: i32) -> io::Result<Zeroizing<String>> {
    let _hidden_input = HiddenInput::new(fd)?;
    super::from_bufread(reader)
}
