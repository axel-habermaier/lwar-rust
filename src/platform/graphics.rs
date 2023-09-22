use super::ComPtr;
use std::ptr::{null, null_mut};
use winapi::{
    ctypes::c_void,
    shared::{dxgi::CreateDXGIFactory, winerror::S_OK},
    shared::{dxgi1_3::DXGIGetDebugInterface1, dxgi1_5::IDXGIFactory5},
    um::{
        d3d11::{
            D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_CREATE_DEVICE_DEBUG, D3D11_CREATE_DEVICE_SINGLETHREADED, D3D11_SDK_VERSION,
        },
        d3dcommon::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0},
        dxgidebug::{IDXGIDebug, DXGI_DEBUG_ALL, DXGI_DEBUG_RLO_ALL},
    },
    Interface,
};

pub struct GraphicsDevice {
    device: ComPtr<ID3D11Device>,
    context: ComPtr<ID3D11DeviceContext>,
}

impl Default for GraphicsDevice {
    fn default() -> GraphicsDevice {
        unsafe {
            let factory = ComPtr::<IDXGIFactory5>::new(
                |factory| CreateDXGIFactory(&IDXGIFactory5::uuidof(), factory as *mut *mut c_void),
                "Failed to create DXGI factory.",
            );

            let flags = if cfg!(debug_assertions) {
                D3D11_CREATE_DEVICE_SINGLETHREADED | D3D11_CREATE_DEVICE_DEBUG
            } else {
                D3D11_CREATE_DEVICE_SINGLETHREADED
            };

            let mut feature_level = D3D_FEATURE_LEVEL_11_0;
            let device = ComPtr::<ID3D11Device>::new(
                |device| {
                    D3D11CreateDevice(
                        null_mut(),
                        D3D_DRIVER_TYPE_HARDWARE,
                        null_mut(),
                        flags,
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

            GraphicsDevice { device, context }
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
