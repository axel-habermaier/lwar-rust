#![warn(clippy::all)]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;
pub mod platform;
use platform::window::*;

pub fn run() -> ! {
    open_window();
    toggle_mode();

    loop {
        process_messages();
        // panic!("Gnah");
    }

    platform::process::exit()
}
