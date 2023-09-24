use std::{
    ffi::CString,
    panic,
    process::exit,
    ptr::{self, null_mut},
    slice,
};
use winapi::um::{
    errhandlingapi::GetLastError,
    winbase::{FormatMessageW, LocalFree, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS, FORMAT_MESSAGE_MAX_WIDTH_MASK},
    winnt::HRESULT,
    winuser::{MessageBoxA, MB_ICONERROR, MB_OK, MB_TASKMODAL, MB_TOPMOST},
};

pub fn get_error_message_for(error: u32) -> String {
    unsafe {
        let mut buffer: *mut u16 = null_mut();
        let length = FormatMessageW(
            FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_MAX_WIDTH_MASK | FORMAT_MESSAGE_ALLOCATE_BUFFER,
            ptr::null(),
            error,
            0,
            &mut buffer as *mut *mut u16 as *mut u16,
            0,
            ptr::null_mut(),
        );

        let message = if length == 0 {
            "Unknown Windows error.".to_string()
        } else {
            String::from_utf16(slice::from_raw_parts(buffer, length as usize))
                .unwrap_or_else(|_| "Failed to retrieve Windows error message.".to_string())
                .trim()
                .to_string()
        };

        LocalFree(buffer as _);
        message
    }
}

pub fn get_last_error() -> String {
    unsafe { get_error_message_for(GetLastError()) }
}

pub fn handle_hresult_error(hr: HRESULT, error_message: &str) {
    if hr < 0 {
        panic!("{} {}", error_message, get_error_message_for(hr as u32));
    }
}

pub fn setup_panic_handler() {
    // Display a nice little message box when the app panics.
    panic::set_hook(Box::new(|panic_info| unsafe {
        let caption = CString::new("Orbs: Fatal Error").unwrap();

        let error_message = {
            // Formatted strings such as `panic!("{}", 1)` are `String` instances.
            if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s
            // Constant strings such as `panic!("") are `&str` instances.
            } else if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s
            } else {
                "An unknown error occurred."
            }
        };

        let message = CString::new(format!(
            "The application has been terminated after a fatal error.\n\nThe error was: {}",
            error_message
        ))
        .unwrap();

        MessageBoxA(null_mut(), message.as_ptr(), caption.as_ptr(), MB_ICONERROR | MB_OK | MB_TASKMODAL | MB_TOPMOST);
        exit(-1);
    }));
}
