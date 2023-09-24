use super::{com_ptr::ComPtr, error::get_error_messag_for};
use std::ptr::{null, null_mut};
use winapi::{
    ctypes::c_void,
    shared::{
        dxgi::*,
        dxgi1_2::*,
        dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM,
        dxgitype::{DXGI_SAMPLE_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT},
        windef::HWND,
        winerror::*,
    },
    shared::{dxgi1_3::DXGIGetDebugInterface1, dxgiformat::DXGI_FORMAT},
    um::{d3d11::*, d3dcommon::*, dxgidebug::*, unknwnbase::IUnknown},
    Interface,
};

pub struct GraphicsDevice {
    device: ComPtr<ID3D11Device>,
    context: ComPtr<ID3D11DeviceContext>,
    swap_chain: ComPtr<IDXGISwapChain1>,
    back_buffer: ComPtr<ID3D11RenderTargetView>,
    hwnd: HWND,
    size: (u32, u32),
}

const BACK_BUFFER_COUNT: u32 = 2;
const BACK_BUFFER_FORMAT: DXGI_FORMAT = DXGI_FORMAT_B8G8R8A8_UNORM;

fn initialize_device_and_context() -> (ComPtr<ID3D11Device>, ComPtr<ID3D11DeviceContext>) {
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

        (device, context)
    }
}

impl Default for GraphicsDevice {
    fn default() -> GraphicsDevice {
        let (device, context) = initialize_device_and_context();

        GraphicsDevice {
            device,
            context,
            swap_chain: ComPtr::null(),
            hwnd: null_mut(),
            back_buffer: ComPtr::null(),
            size: (800, 600), // this will be replace later on with the real window size.
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
            let (width, height) = self.size;
            let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: width,
                Height: height,
                Format: BACK_BUFFER_FORMAT,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                    ..Default::default()
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: BACK_BUFFER_COUNT,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                ..Default::default()
            };

            self.hwnd = hwnd;
            self.swap_chain = ComPtr::<IDXGISwapChain1>::new(
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
            );

            // Do not allow DXGI to make fullscreen mode transitions on ALT + Enter because we handle fullscreen mode
            // ourselves with a borderless fullscreen window.
            let dxgi_mwa_no_alt_enter = 1 << 1; // this is missing in the winapi crate
            factory.MakeWindowAssociation(hwnd, dxgi_mwa_no_alt_enter);
        }
    }

    pub fn resize_back_buffer(&mut self, width: u32, height: u32) {
        unsafe {
            // Unset the currently bound render target just in case it's set to the back buffer render target.
            // We should no longer reference the old render target now that we're going to resize the buffers.
            // We also flush the device context to ensure that the command has been sent to the GPU.
            let null_views: [*mut ID3D11RenderTargetView; 1] = [null_mut()];
            self.context.OMSetRenderTargets(0, null_views.as_ptr(), null_mut());
            self.context.Flush();
            self.back_buffer = ComPtr::null();

            let hr = self.swap_chain.ResizeBuffers(BACK_BUFFER_COUNT, width, height, BACK_BUFFER_FORMAT, 0);
            self.handle_swap_chain_result(hr);

            let texture = ComPtr::<ID3D11Texture2D>::new(
                |texture| self.swap_chain.GetBuffer(0, &ID3D11Texture2D::uuidof(), texture as *mut *mut c_void),
                "Failed to retrieve back buffer texture.",
            );

            self.back_buffer = ComPtr::<ID3D11RenderTargetView>::new(
                |back_buffer| {
                    self.device
                        .CreateRenderTargetView(texture.as_ptr() as *mut ID3D11Resource, null(), back_buffer)
                },
                "Failed to create render target for back buffer.",
            );
        }
    }

    pub fn present(&mut self) {
        unsafe {
            let hr = self.swap_chain.Present(1 /* wait for VSYNC */, 0);
            self.handle_swap_chain_result(hr);
        }
    }

    fn handle_swap_chain_result(&mut self, hr: HRESULT) {
        // If the device was reset for any reason, we must reinitialize everything.
        if hr == DXGI_ERROR_DEVICE_REMOVED || hr == DXGI_ERROR_DEVICE_RESET {
            self.back_buffer = ComPtr::null();
            self.swap_chain = ComPtr::null();

            let (device, context) = initialize_device_and_context();
            self.device = device;
            self.context = context;
            self.initialize_swap_chain(self.hwnd);
            let (width, height) = self.size;
            self.resize_back_buffer(width, height);
        } else if hr < 0 {
            panic!("An error related to the swap chain occurred. {}", get_error_messag_for(hr as u32));
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
