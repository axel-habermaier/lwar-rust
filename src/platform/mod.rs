use std::{
    ops::Deref,
    ptr::{self, null_mut},
    slice,
};
use winapi::{
    ctypes::c_void,
    um::{
        errhandlingapi::GetLastError,
        unknwnbase::IUnknown,
        winbase::{
            FormatMessageW, LocalFree, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS,
            FORMAT_MESSAGE_MAX_WIDTH_MASK,
        },
        winnt::HRESULT,
    },
    Interface,
};

pub mod graphics;
pub mod input;
pub mod window;

fn get_error_messag_for(error: u32) -> String {
    unsafe {
        let mut buffer: *mut u16 = null_mut();
        let length = FormatMessageW(
            FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_MAX_WIDTH_MASK | FORMAT_MESSAGE_ALLOCATE_BUFFER,
            ptr::null(),
            error,
            0,
            &mut buffer as *mut *mut u16 as *mut u16,
            0,
            ptr::null_mut(),
        );

        let message = if length == 0 {
            "Unknown Windows error.".to_string()
        } else {
            String::from_utf16(slice::from_raw_parts(buffer, length as usize))
                .unwrap_or_else(|_| "Failed to retrieve Windows error message.".to_string())
                .trim()
                .to_string()
        };

        LocalFree(buffer as _);
        message
    }
}

fn get_last_error() -> String {
    unsafe { get_error_messag_for(GetLastError()) }
}

struct ComPtr<T: Interface> {
    ptr: *mut T,
}

impl<T: Interface> ComPtr<T> {
    fn new(func: impl FnOnce(*mut *mut T) -> HRESULT, error_message: &str) -> ComPtr<T> {
        let mut ptr: *mut T = null_mut();
        let result = func(&mut ptr);

        if result < 0 || ptr.is_null() {
            panic!("{} {}", error_message, get_error_messag_for(result as u32))
        }

        ComPtr { ptr }
    }

    fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    fn convert<U: Interface>(&self) -> ComPtr<U> {
        ComPtr::<U>::new(
            |obj| unsafe { (*(self.ptr as *const IUnknown)).QueryInterface(&U::uuidof(), obj as *mut *mut c_void) },
            "COM interface not implemented.",
        )
    }
}

impl<T: Interface> Deref for ComPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref().unwrap() }
    }
}

impl<T: Interface> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
            (*(self.ptr as *const IUnknown)).Release();
        }
    }
}
