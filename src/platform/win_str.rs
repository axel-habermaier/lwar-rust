use alloc::vec::Vec;

pub struct WinStr {
    chars: Vec<i8>,
}

impl WinStr {
    pub fn from(s: &str) -> WinStr {
        WinStr {
            chars: s
                .as_bytes()
                .iter()
                .map(|c| *c as i8) // unsafe if the string does not consist of valid ASCII chars only
                .chain([0]) // append a terminating 0 to get a valid C string
                .collect(),
        }
    }

    pub fn as_ptr(&self) -> *const i8 {
        self.chars.as_ptr() as _
    }
}
