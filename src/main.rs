#![warn(clippy::all)]

use std::process::exit;

use orbs::platform::error::on_panic;
use orbs::platform::graphics::report_d3d11_leaks;
use orbs::platform::show_message_box;
use winapi::um::wincon::*;
use winapi::um::winuser::*;

fn main() {
    on_panic(|error_message| {
        show_message_box(format!(
            "The application has been terminated after a fatal error.\n\nThe error was: {error_message}"
        ));
    });

    unsafe {
        if AttachConsole(ATTACH_PARENT_PROCESS) != 0 {
            ShowWindow(GetConsoleWindow(), SW_HIDE);
        }
    }

    orbs::run();

    report_d3d11_leaks();
    exit(0);
}
