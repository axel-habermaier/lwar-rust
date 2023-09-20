#![warn(clippy::all)]

pub mod platform;
use platform::window::{execute_in_window, Event};

pub fn run() {
    execute_in_window(|event, exit| {
        println!("Event: {:?}", event);
        if let Event::CloseRequested = event {
            *exit = true;
        }
    });
}
