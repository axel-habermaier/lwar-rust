use super::{com_ptr::ComPtr, GraphicsDevice, Texture2D};
use crate::platform::error::handle_hresult_error;
use winapi::{shared::dxgi1_2::DXGI_SWAP_CHAIN_DESC1, um::d3d11::*, Interface};

impl GraphicsDevice {
    pub fn resize_back_buffer(&mut self, width: u32, height: u32) {
        unsafe {
            // We're not allowed to reference the old buffers anymore anywhere, so let's reset all
            // D3D11 state to the default values.
            self.back_buffer = None;
            self.context.ClearState();
            self.context.Flush();

            let mut desc = DXGI_SWAP_CHAIN_DESC1::default();
            self.swap_chain.GetDesc1(&mut desc);

            let hr = self.swap_chain.ResizeBuffers(desc.BufferCount, width, height, desc.Format, 0);
            handle_hresult_error(hr, "Failed to resize swap chain buffers.");

            let texture = Texture2D {
                width,
                height,
                p: ComPtr::<ID3D11Texture2D>::new(
                    |texture| self.swap_chain.GetBuffer(0, &ID3D11Texture2D::uuidof(), texture as *mut *mut _),
                    "Failed to retrieve back buffer texture.",
                ),
            };

            self.back_buffer = Some(self.create_render_target(&texture));
        }
    }

    pub fn present(&self) {
        unsafe {
            let hr = self.swap_chain.Present(1 /* wait for VSYNC */, 0);
            handle_hresult_error(hr, "Failed to present back buffer.");
        }
    }
}
