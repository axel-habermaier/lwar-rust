#![warn(clippy::all)]

use std::process::exit;

use orbs::platform::error::{setup_panic_handler, ShowMessageBox};
use orbs::platform::graphics::report_d3d11_leaks;
use winapi::um::wincon::*;
use winapi::um::winuser::*;

fn main() {
    setup_panic_handler(ShowMessageBox::Yes);

    unsafe {
        if AttachConsole(ATTACH_PARENT_PROCESS) != 0 {
            ShowWindow(GetConsoleWindow(), SW_HIDE);
        }
    }

    orbs::run();

    report_d3d11_leaks();

    exit(0);
}
