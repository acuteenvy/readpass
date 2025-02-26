use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::os::windows::io::FromRawHandle;

use windows_sys::core::PCSTR;
use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE};
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
fn from_handle_with_hidden_input(reader: &mut impl BufRead, handle: HANDLE) -> io::Result<String> {
    let _hidden_input = HiddenInput::new(handle)?;
    let reader_return = super::from_bufread(reader);

    // Newline for windows which otherwise prints on the same line.
    io::stdout().write_all(b"\n")?;
    reader_return
}
