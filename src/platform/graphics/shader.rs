use super::{com_ptr::ComPtr, GraphicsDevice, PixelShader, VertexShader};
use std::ptr;
use winapi::um::d3d11::D3D11_INPUT_ELEMENT_DESC;

impl GraphicsDevice {
    pub fn create_vertex_shader(&self, byte_code: &[u8], input_elements: &[D3D11_INPUT_ELEMENT_DESC]) -> VertexShader {
        unsafe {
            VertexShader {
                p: ComPtr::new(
                    |vertex_shader| {
                        self.device
                            .CreateVertexShader(byte_code.as_ptr() as _, byte_code.len(), ptr::null_mut(), vertex_shader)
                    },
                    "Failed to create vertex shader.",
                ),
                input_layout: ComPtr::new(
                    |layout| {
                        self.device.CreateInputLayout(
                            input_elements.as_ptr(),
                            input_elements.len() as u32,
                            byte_code.as_ptr() as _,
                            byte_code.len(),
                            layout,
                        )
                    },
                    "Failed to create input layout.",
                ),
            }
        }
    }

    pub fn create_pixel_shader(&self, byte_code: &[u8]) -> PixelShader {
        unsafe {
            PixelShader {
                p: ComPtr::new(
                    |pixel_shader| {
                        self.device
                            .CreatePixelShader(byte_code.as_ptr() as _, byte_code.len(), ptr::null_mut(), pixel_shader)
                    },
                    "Failed to create pixel shader.",
                ),
            }
        }
    }

    pub fn set_vertex_shader(&self, vertex_shader: &VertexShader) {
        unsafe {
            self.context.VSSetShader(vertex_shader.p.as_ptr(), ptr::null(), 0);
            self.context.IASetInputLayout(vertex_shader.input_layout.as_ptr());
        }
    }

    pub fn set_pixel_shader(&self, pixel_shader: &PixelShader) {
        unsafe {
            self.context.PSSetShader(pixel_shader.p.as_ptr(), ptr::null(), 0);
        }
    }
}
