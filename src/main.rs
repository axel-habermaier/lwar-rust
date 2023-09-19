#![warn(clippy::all)]
#![windows_subsystem = "windows"] // Don't open up a console window when the app starts.

use std::{ffi::CString, panic, process::exit, ptr::null_mut};
use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};
use winapi::um::winuser::{MessageBoxA, MB_ICONERROR, MB_OK};

fn main() {
    // Display a nice little message box when the app panics.
    panic::set_hook(Box::new(|panic_info| {
        let caption = CString::new("Orbs: Fatal Error").unwrap();
        let error_message = if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s
        } else if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s
        } else {
            "An unknown error occurred."
        };
        let message = CString::new(format!(
            "The application has been terminated after a fatal error.\n\nThe error was: {}",
            error_message
        ))
        .unwrap();

        unsafe {
            MessageBoxA(null_mut(), message.as_ptr(), caption.as_ptr(), MB_ICONERROR | MB_OK);
        }

        exit(-1);
    }));

    // Try to attach to the parent console so that we get log output to stdout when starting the app from the command line.
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    orbs::run();
    exit(0);
}
