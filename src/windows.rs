use std::fs::File;
use std::io::{self, BufReader, Write};
use std::os::windows::io::FromRawHandle;

use windows_sys::core::PCSTR;
use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileA, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows_sys::Win32::System::Console::{
    GetConsoleMode, SetConsoleMode, CONSOLE_MODE, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT,
};
use zeroize::Zeroizing;

struct HiddenInput {
    mode: u32,
    handle: HANDLE,
}

impl HiddenInput {
    fn new(handle: HANDLE) -> io::Result<HiddenInput> {
        let mut mode = 0;

        // Get the old mode, so that we can reset back to it when we are done.
        if unsafe { GetConsoleMode(handle, &mut mode as *mut CONSOLE_MODE) } == 0 {
            return Err(std::io::Error::last_os_error());
        }

        // We want to be able to read line by line, and we still want backspace to work.
        let new_mode_flags = ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT;
        if unsafe { SetConsoleMode(handle, new_mode_flags) } == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(HiddenInput { mode, handle })
    }
}

impl Drop for HiddenInput {
    fn drop(&mut self) {
        // Set the mode back to normal.
        unsafe {
            SetConsoleMode(self.handle, self.mode);
        }
    }
}

/// Reads a password from the TTY.
///
/// Newlines and carriage returns are trimmed from the end of the resulting `String`.
///
/// # Errors
///
/// This function will return an I/O error if reading from the handle fails.
pub fn from_tty() -> io::Result<Zeroizing<String>> {
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

    let mut reader = BufReader::new(unsafe { File::from_raw_handle(handle as _) });

    let _hidden_input = HiddenInput::new(handle)?;
    let reader_return = crate::from_bufread(&mut reader);

    // Print a newline on Windows (otherwise whatever is printed next will be on the same line).
    io::stdout().write_all(b"\n")?;
    reader_return
}
