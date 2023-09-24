use super::error::get_error_messag_for;
use std::{ops::Deref, ptr::null_mut};
use winapi::{
    ctypes::c_void,
    um::{unknwnbase::IUnknown, winnt::HRESULT},
    Interface,
};

pub struct ComPtr<T: Interface> {
    ptr: *mut T,
}

impl<T: Interface> ComPtr<T> {
    pub fn null() -> ComPtr<T> {
        ComPtr { ptr: null_mut() }
    }

    pub fn new(func: impl FnOnce(*mut *mut T) -> HRESULT, error_message: &str) -> ComPtr<T> {
        let mut ptr: *mut T = null_mut();
        let result = func(&mut ptr);

        if result < 0 || ptr.is_null() {
            panic!("{} {}", error_message, get_error_messag_for(result as u32))
        }

        ComPtr { ptr }
    }

    pub fn as_ptr(&self) -> *mut T {
        debug_assert!(!self.ptr.is_null());
        self.ptr
    }

    pub fn convert<U: Interface>(&self) -> ComPtr<U> {
        ComPtr::<U>::new(
            |obj| unsafe { (*self.as_ptr().cast::<IUnknown>()).QueryInterface(&U::uuidof(), obj as *mut *mut c_void) },
            "COM interface not implemented.",
        )
    }
}

impl<T: Interface> Deref for ComPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.as_ptr().as_ref().unwrap() }
    }
}

impl<T: Interface> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                (*self.ptr.cast::<IUnknown>()).Release();
            }
        }
    }
}
