use super::{com_ptr::ComPtr, GraphicsDevice, RenderTarget, Texture2D};
use crate::primitives::Color;
use std::ptr::null;
use winapi::um::d3d11::{ID3D11RenderTargetView, ID3D11Resource};

impl GraphicsDevice {
    pub fn back_buffer(&self) -> &RenderTarget {
        self.back_buffer.as_ref().expect("Back buffer not initialized.")
    }

    pub fn create_render_target(&self, texture: &Texture2D) -> RenderTarget {
        unsafe {
            RenderTarget {
                width: texture.width,
                height: texture.height,
                p: ComPtr::<ID3D11RenderTargetView>::new(
                    |back_buffer| {
                        self.device
                            .CreateRenderTargetView(texture.p.as_ptr() as *mut ID3D11Resource, null(), back_buffer)
                    },
                    "Failed to create render target.",
                ),
            }
        }
    }

    pub fn clear(&self, render_target: &RenderTarget, color: Color) {
        unsafe {
            let color = [
                color.r as f32 / 255.,
                color.g as f32 / 255.,
                color.b as f32 / 255.,
                color.a as f32 / 255.,
            ];
            self.context.ClearRenderTargetView(render_target.p.as_ptr(), &color);
        }
    }
}
