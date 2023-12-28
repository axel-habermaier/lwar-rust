#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

pub mod platform;
mod primitives;
use platform::{
    graphics::{state::PrimitiveType, GraphicsDevice},
    Event, Window,
};
use primitives::{Color, Rectangle};
use winapi::{shared::dxgiformat::DXGI_FORMAT_R32G32B32A32_FLOAT, um::d3d11::D3D11_INPUT_ELEMENT_DESC};

pub fn run() {
    let mut window = Window::new();
    let mut graphics_device = GraphicsDevice::new(&window);
    let mut should_exit = false;

    let vertex_shader = graphics_device.create_vertex_shader(
        include_bytes!("../target/assets/debug/shaders/sprite.vs.hlsl"),
        &[D3D11_INPUT_ELEMENT_DESC {
            SemanticName: b"POSITION\0".as_ptr() as _,
            Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
            ..Default::default()
        }],
    );
    let pixel_shader = graphics_device.create_pixel_shader(include_bytes!("../target/assets/debug/shaders/sprite.ps.hlsl"));

    graphics_device.set_vertex_shader(&vertex_shader);
    graphics_device.set_pixel_shader(&pixel_shader);
    graphics_device.set_primitive_type(PrimitiveType::Triangles);

    while !should_exit {
        window.handle_events(|event| match event {
            Event::CloseRequested => should_exit = true,
            Event::Resized(width, height) => {
                graphics_device.resize_back_buffer(width, height);
                graphics_device.set_viewport(&Rectangle {
                    left: 0,
                    top: 0,
                    width,
                    height,
                })
            }
            Event::KeyPressed(key, sc) => println!("{key:?}, {sc}"),
            _ => {}
        });

        graphics_device.clear(graphics_device.back_buffer(), Color::new(0, 0, 0, 255));
        graphics_device.present();
    }
}
