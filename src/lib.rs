#![warn(clippy::all)]

pub mod platform;
use platform::window::{execute_in_window, ControlFlow, Event};

pub fn run() {
    execute_in_window(|event| {
        println!("Event: {:?}", event);
        if let Event::CloseRequested = event {
            ControlFlow::Exit
        } else {
            ControlFlow::Continue
        }
    });
}
