use crate::platform::error::handle_hresult_error;
use std::{
    ops::Deref,
    ptr::{null_mut, NonNull},
};
use winapi::{
    um::{unknwnbase::IUnknown, winnt::HRESULT},
    Interface,
};

pub struct ComPtr<T: Interface> {
    p: NonNull<T>,
}

impl<T: Interface> ComPtr<T> {
    pub fn new(func: impl FnOnce(*mut *mut T) -> HRESULT, error_message: &str) -> ComPtr<T> {
        let mut ptr: *mut T = null_mut();
        handle_hresult_error(func(&mut ptr), error_message);

        ComPtr {
            p: NonNull::new(ptr).expect("Failed to allocate COM object."),
        }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.p.as_ptr()
    }

    pub fn convert<U: Interface>(&self) -> ComPtr<U> {
        ComPtr::<U>::new(
            |obj| unsafe { (*self.p.as_ptr().cast::<IUnknown>()).QueryInterface(&U::uuidof(), obj as *mut *mut _) },
            "COM interface not implemented.",
        )
    }
}

impl<T: Interface> Deref for ComPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.p.as_ref() }
    }
}

impl<T: Interface> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
            (*self.p.as_ptr().cast::<IUnknown>()).Release();
        }
    }
}
