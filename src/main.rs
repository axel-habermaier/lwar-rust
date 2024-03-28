#![warn(clippy::all)]
#![windows_subsystem = "windows"]

use lwar::platform::error::on_panic;
use lwar::platform::graphics::report_d3d11_leaks;
use lwar::platform::show_message_box;
use std::process::exit;

fn main() {
    on_panic(|error_message| {
        show_message_box(format!(
            "The application has been terminated after a fatal error.\n\nThe error was: {error_message}"
        ));
    });

    lwar::run();

    report_d3d11_leaks();
    exit(0);
}
