#![warn(clippy::all)]
#![windows_subsystem = "windows"] // Don't open up a console window when the app starts.

use std::process::exit;

use orbs::platform::error::{setup_panic_handler, ShowMessageBox};
use orbs::platform::graphics::report_d3d11_leaks;
use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};

fn main() {
    setup_panic_handler(ShowMessageBox::Yes);

    // Try to attach to the parent console so that we get log output to stdout when starting the app from the command line.
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    orbs::run();

    report_d3d11_leaks();
    exit(0);
}
