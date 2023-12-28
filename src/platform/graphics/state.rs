use super::GraphicsDevice;
use crate::primitives::Rectangle;
use winapi::um::{
    d3d11::{D3D11_RECT, D3D11_VIEWPORT},
    d3dcommon::{D3D11_PRIMITIVE_TOPOLOGY_POINTLIST, D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST},
};

#[repr(u32)]
pub enum PrimitiveType {
    Points = D3D11_PRIMITIVE_TOPOLOGY_POINTLIST,
    Triangles = D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
}

impl GraphicsDevice {
    pub fn set_viewport(&self, viewport: &Rectangle<u32>) {
        unsafe {
            self.context.RSSetViewports(
                1,
                &D3D11_VIEWPORT {
                    TopLeftX: viewport.left as f32,
                    TopLeftY: viewport.top as f32,
                    Width: viewport.width as f32,
                    Height: viewport.height as f32,
                    MaxDepth: 1.,
                    MinDepth: 0.,
                },
            );
        }
    }

    pub fn set_scissor_rect(&self, rectangle: &Rectangle<u32>) {
        unsafe {
            self.context.RSSetScissorRects(
                1,
                &D3D11_RECT {
                    left: rectangle.left as i32,
                    top: rectangle.top as i32,
                    right: (rectangle.left + rectangle.width) as i32,
                    bottom: (rectangle.top + rectangle.height) as i32,
                },
            );
        }
    }

    pub fn set_primitive_type(&self, primitive_type: PrimitiveType) {
        unsafe {
            self.context.IASetPrimitiveTopology(primitive_type as u32);
        }
    }
}
