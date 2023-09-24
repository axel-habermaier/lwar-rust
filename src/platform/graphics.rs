use com_ptr::ComPtr;
use winapi::{
    ctypes::c_void,
    shared::{dxgi1_2::IDXGISwapChain1, dxgi1_3::DXGIGetDebugInterface1},
    um::{
        d3d11::{ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D},
        dxgidebug::{IDXGIDebug, DXGI_DEBUG_ALL, DXGI_DEBUG_RLO_ALL},
    },
    Interface,
};

mod com_ptr;
pub mod graphics_device;
pub mod render_target;
pub mod state;
pub mod swap_chain;
pub mod texture;

pub struct GraphicsDevice {
    device: ComPtr<ID3D11Device>,
    context: ComPtr<ID3D11DeviceContext>,
    swap_chain: ComPtr<IDXGISwapChain1>,
    back_buffer: Option<RenderTarget>,
}

pub struct RenderTarget {
    p: ComPtr<ID3D11RenderTargetView>,
    pub width: u32,
    pub height: u32,
}

pub struct Texture2D {
    p: ComPtr<ID3D11Texture2D>,
    pub width: u32,
    pub height: u32,
}

pub fn report_d3d11_leaks() {
    if cfg!(debug_assertions) {
        unsafe {
            let debug = ComPtr::<IDXGIDebug>::new(
                |debug| DXGIGetDebugInterface1(0, &IDXGIDebug::uuidof(), debug as *mut *mut c_void),
                "Failed to instantiate the IDXGIDebug interface.",
            );
            debug.ReportLiveObjects(DXGI_DEBUG_ALL, DXGI_DEBUG_RLO_ALL);
        }
    }
}
