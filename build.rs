use std::{
    env::set_current_dir,
    ffi::{CString, OsStr},
    os::windows::prelude::OsStrExt,
    ptr::{null, null_mut},
};
use winapi::um::d3dcompiler::{D3DCompileFromFile, D3DCOMPILE_DEBUG, D3DCOMPILE_ENABLE_STRICTNESS};

fn main() {
    set_current_dir("assets/").unwrap();

    vertex_shader("shaders/sprite.vs.hlsl");
}

fn vertex_shader(path: &str) {
    println!("Compiling vertex shader '{}'.", path);

    unsafe {
        let main = CString::new("Main").unwrap();
        let target = CString::new("vs_5_0").unwrap();
        let mut shader_blob = null_mut();
        let mut error_blob = null_mut();

        D3DCompileFromFile(
            OsStr::new(path).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>().as_ptr(),
            null(),
            null_mut(),
            main.as_ptr(),
            target.as_ptr(),
            if cfg!(debug_assertions) {
                D3DCOMPILE_DEBUG | D3DCOMPILE_ENABLE_STRICTNESS
            } else {
                D3DCOMPILE_ENABLE_STRICTNESS
            },
            0,
            &mut shader_blob,
            &mut error_blob,
        );

        if !error_blob.is_null() {
            let text = (*error_blob).GetBufferPointer() as *mut u8;
            let size = (*error_blob).GetBufferSize();
            let error = String::from_utf8_unchecked(Vec::from_raw_parts(text, size, size));
            panic!("Failed to compile vertex shader '{}': {}", path, error)
        }
    }
}
