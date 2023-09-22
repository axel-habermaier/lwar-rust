#![warn(clippy::all)]

pub mod platform;
use platform::{
    graphics::GraphicsDevice,
    window::{execute_in_window, Event},
};

pub fn run() {
    let graphics_device = GraphicsDevice::default();

    execute_in_window(|event, exit| match event {
        Event::Initialized(hwnd) => {
            //graphics_device = Some(GraphicsDevice::default());
        }
        Event::UpdateAndRender => {}
        Event::CloseRequested => *exit = true,
        _ => println!("Event: {:?}", event),
    });
}
