use super::ComPtr;
use std::ptr::{null, null_mut};
use winapi::{
    ctypes::c_void,
    shared::dxgi1_3::DXGIGetDebugInterface1,
    shared::{
        dxgi::*,
        dxgi1_2::*,
        dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM,
        dxgitype::{DXGI_SAMPLE_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT},
        windef::HWND,
        winerror::S_OK,
    },
    um::{d3d11::*, d3dcommon::*, dxgidebug::*, unknwnbase::IUnknown},
    Interface,
};

pub struct GraphicsDevice {
    device: ComPtr<ID3D11Device>,
    context: ComPtr<ID3D11DeviceContext>,
    swap_chain: Option<ComPtr<IDXGISwapChain1>>,
}

impl Default for GraphicsDevice {
    fn default() -> GraphicsDevice {
        unsafe {
            let mut feature_level = D3D_FEATURE_LEVEL_11_0;
            let device = ComPtr::<ID3D11Device>::new(
                |device| {
                    D3D11CreateDevice(
                        null_mut(),
                        D3D_DRIVER_TYPE_HARDWARE,
                        null_mut(),
                        if cfg!(debug_assertions) {
                            D3D11_CREATE_DEVICE_SINGLETHREADED | D3D11_CREATE_DEVICE_DEBUG
                        } else {
                            D3D11_CREATE_DEVICE_SINGLETHREADED
                        },
                        null(),
                        0,
                        D3D11_SDK_VERSION,
                        device,
                        &mut feature_level,
                        null_mut(),
                    )
                },
                "Failed to create Direct3D 11 device.",
            );

            if feature_level < D3D_FEATURE_LEVEL_11_0 {
                panic!("Incompatible graphics card: Feature level 11.0 is required.");
            }

            let context = ComPtr::<ID3D11DeviceContext>::new(
                |context| {
                    device.GetImmediateContext(context);
                    S_OK
                },
                "Failed to get context.",
            );

            GraphicsDevice {
                device,
                context,
                swap_chain: None,
            }
        }
    }
}

impl GraphicsDevice {
    pub fn initialize_swap_chain(&mut self, hwnd: HWND) {
        unsafe {
            let device = self.device.convert::<IDXGIDevice1>();
            let adapter = ComPtr::<IDXGIAdapter>::new(|adapter| device.GetAdapter(adapter), "Failed to retrieve DXGI adapter.");
            let factory = ComPtr::<IDXGIFactory2>::new(
                |factory| adapter.GetParent(&IDXGIFactory2::uuidof(), factory as *mut *mut c_void),
                "Failed to retrieve DXGI factory.",
            );

            // Initialize the swap chain with a default size because it will get resized later on anyway and we don't
            // know the final window size yet. We also defer the creation of the back buffer render target until then.
            let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: 800,
                Height: 600,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                    ..Default::default()
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                ..Default::default()
            };

            self.swap_chain = Some(ComPtr::<IDXGISwapChain1>::new(
                |swap_chain| {
                    factory.CreateSwapChainForHwnd(
                        self.device.as_ptr() as *mut IUnknown,
                        hwnd,
                        &swap_chain_desc,
                        null(),
                        null_mut(),
                        swap_chain,
                    )
                },
                "Unable to initialize swap chain.",
            ));

            // Do not allow DXGI to make fullscreen mode transitions on ALT + Enter because we handle fullscreen mode
            // ourselves with a borderless fullscreen window.
            let dxgi_mwa_no_alt_enter = 1 << 1;
            factory.MakeWindowAssociation(hwnd, dxgi_mwa_no_alt_enter);
        }
    }
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
