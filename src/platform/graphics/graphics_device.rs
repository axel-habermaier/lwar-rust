use super::{com_ptr::ComPtr, GraphicsDevice};
use crate::platform::Window;
use std::ptr::{null, null_mut};
use winapi::{
    shared::{
        dxgi::{IDXGIAdapter, IDXGIDevice1, DXGI_SWAP_EFFECT_FLIP_DISCARD},
        dxgi1_2::{IDXGIFactory2, IDXGISwapChain1, DXGI_SWAP_CHAIN_DESC1},
        dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM,
        dxgitype::{DXGI_SAMPLE_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT},
        winerror::S_OK,
    },
    um::{d3d11::*, d3dcommon::*},
    Interface,
};

impl GraphicsDevice {
    pub fn new(window: &Window) -> GraphicsDevice {
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

            let factory = {
                let device = device.convert::<IDXGIDevice1>();
                let adapter =
                    ComPtr::<IDXGIAdapter>::new(|adapter| device.GetAdapter(adapter), "Failed to retrieve DXGI adapter.");

                ComPtr::<IDXGIFactory2>::new(
                    |factory| adapter.GetParent(&IDXGIFactory2::uuidof(), factory as *mut *mut _),
                    "Failed to retrieve DXGI factory.",
                )
            };

            let (width, height) = window.size();
            let hwnd = window.hwnd();

            let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: width,
                Height: height,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                ..Default::default()
            };

            let swap_chain = ComPtr::<IDXGISwapChain1>::new(
                |swap_chain| {
                    factory.CreateSwapChainForHwnd(
                        device.as_ptr() as *mut _,
                        hwnd,
                        &swap_chain_desc,
                        null(),
                        null_mut(),
                        swap_chain,
                    )
                },
                "Unable to initialize swap chain.",
            );

            // Do not allow DXGI to make fullscreen mode transitions on ALT + Enter because we handle fullscreen mode
            // ourselves with a borderless fullscreen window.
            let dxgi_mwa_no_alt_enter = 1 << 1; // this is missing in the winapi crate
            factory.MakeWindowAssociation(hwnd, dxgi_mwa_no_alt_enter);

            let mut device = GraphicsDevice {
                device,
                context,
                swap_chain,
                back_buffer: None,
            };

            device.resize_back_buffer(width, height);
            device
        }
    }
}
