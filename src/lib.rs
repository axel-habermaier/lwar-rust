#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

pub mod platform;
mod primitives;
use platform::{graphics::GraphicsDevice, Event, Window};
use primitives::Color;

pub fn run() {
    let mut window = Window::new();
    let mut graphics_device = GraphicsDevice::new(&window);
    let mut should_exit = false;

    let _shader = include_bytes!("../target/assets/release/shaders/sprite.vs.hlsl");

    while !should_exit {
        window.handle_events(|event| match event {
            Event::CloseRequested => should_exit = true,
            Event::Resized(width, height) => graphics_device.resize_back_buffer(width, height),
            Event::KeyPressed(key, sc) => println!("{key:?}, {sc}"),
            _ => {}
        });

        graphics_device.clear(graphics_device.back_buffer(), Color::new(255, 0, 0, 255));
        graphics_device.present();
    }
}
