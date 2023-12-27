#![warn(clippy::all)]
#![windows_subsystem = "windows"]

use orbs::platform::error::on_panic;
use orbs::platform::graphics::report_d3d11_leaks;
use orbs::platform::show_message_box;
use std::process::exit;

fn main() {
    on_panic(|error_message| {
        show_message_box(format!(
            "The application has been terminated after a fatal error.\n\nThe error was: {error_message}"
        ));
    });

    orbs::run();

    report_d3d11_leaks();
    exit(0);
}
