use std::{ptr, slice};
use winapi::um::{
    errhandlingapi::GetLastError,
    winbase::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS, FORMAT_MESSAGE_MAX_WIDTH_MASK},
};

pub mod input;
pub mod window;

fn get_last_error() -> String {
    let buffer_size = 2048;
    let mut buffer: Vec<u16> = Vec::with_capacity(buffer_size);

    unsafe {
        let length = FormatMessageW(
            FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_MAX_WIDTH_MASK,
            ptr::null(),
            GetLastError(),
            0,
            buffer.as_mut_ptr(),
            buffer_size as u32,
            ptr::null_mut(),
        );

        if length == 0 {
            "Unknown Windows error".to_string()
        } else {
            String::from_utf16(slice::from_raw_parts(buffer.as_ptr(), length as usize))
                .unwrap_or_else(|_| "Failed to retrieve Windows error message.".to_string())
        }
    }
}
