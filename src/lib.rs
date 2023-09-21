#![warn(clippy::all)]

pub mod platform;
use platform::window::{execute_in_window, Event};

pub fn run() {
    execute_in_window(|event, exit| {
        if let Event::UpdateAndRender = event {
        } else {
            println!("Event: {:?}", event);
        }
        if let Event::CloseRequested = event {
            *exit = true;
        }
    });
}
