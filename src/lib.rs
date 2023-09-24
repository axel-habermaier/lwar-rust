#![warn(clippy::all)]

pub mod platform;
mod primitives;
use crate::primitives::Color;
use platform::{graphics::GraphicsDevice, Event, Window};

pub fn run() {
    let mut window = Window::new("Orbs");
    let mut graphics_device = GraphicsDevice::new(&window);
    let mut should_exit = false;

    while !should_exit {
        window.pending_events().for_each(|event| match event {
            Event::CloseRequested => should_exit = true,
            &Event::Resized(width, height) => graphics_device.resize_back_buffer(width, height),
            _ => {}
        });

        graphics_device.clear(graphics_device.back_buffer(), Color::new(255, 0, 0, 255));
        graphics_device.present();
    }
}
