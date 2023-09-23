#![warn(clippy::all)]

pub mod platform;
use platform::{
    graphics::GraphicsDevice,
    window::{execute_in_window, Event},
};

pub fn run() {
    let mut graphics_device = GraphicsDevice::default();

    execute_in_window(|event, exit| match event {
        &Event::Initialized(hwnd) => graphics_device.initialize_swap_chain(hwnd),
        &Event::Resized(width, height) => graphics_device.resize_back_buffer(width, height),
        Event::UpdateAndRender => graphics_device.present(),
        Event::CloseRequested => *exit = true,
        _ => println!("Event: {:?}", event),
    });
}
