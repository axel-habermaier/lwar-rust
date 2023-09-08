use alloc::vec::Vec;
use core::iter;

pub struct WinStr {
    chars: Vec<i8>,
}

impl WinStr {
    pub fn from(s: &str) -> WinStr {
        WinStr {
            chars: s
                .as_bytes()
                .iter()
                .map(|c| *c as i8) // unsafe if the string does not consists of valid ASCII chars only
                .chain(iter::once(0)) // append a terminating 0 to get a valid C string
                .collect(),
        }
    }

    pub fn as_ptr(&self) -> *const i8 {
        self.chars.as_ptr() as _
    }
}
