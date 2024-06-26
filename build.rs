include!("src/platform/error.rs");
use std::{env::set_current_dir, ffi::OsStr, fs, os::windows::prelude::OsStrExt, path::Path, ptr::null};
use winapi::{
    shared::winerror::{E_FAIL, S_OK},
    um::d3dcompiler::{D3DCompileFromFile, D3DCOMPILE_DEBUG, D3DCOMPILE_ENABLE_STRICTNESS},
};

fn main() {
    on_panic(|_| {});
    set_current_dir("assets/").unwrap();

    unsafe {
        vertex_shader("shaders/sprite.vs.hlsl");
        pixel_shader("shaders/sprite.ps.hlsl");
    }
}

unsafe fn vertex_shader(path: &str) {
    println!("Compiling vertex shader '{path}'.");
    shader(path, b"vs_5_0\0");
}

unsafe fn pixel_shader(path: &str) {
    println!("Compiling pixel shader '{path}'.");
    shader(path, b"ps_5_0\0");
}

unsafe fn shader(path: &str, target: &[u8]) {
    let mut shader_blob = null_mut();
    let mut error_blob = null_mut();

    let hr = D3DCompileFromFile(
        OsStr::new(path).encode_wide().chain([0]).collect::<Vec<_>>().as_ptr(),
        null(),
        null_mut(),
        b"main\0".as_ptr() as *const _,
        target.as_ptr() as _,
        if cfg!(debug_assertions) {
            D3DCOMPILE_DEBUG | D3DCOMPILE_ENABLE_STRICTNESS
        } else {
            D3DCOMPILE_ENABLE_STRICTNESS
        },
        0,
        &mut shader_blob,
        &mut error_blob,
    );

    if hr == E_FAIL || !error_blob.is_null() {
        let text = (*error_blob).GetBufferPointer() as *mut u8;
        let size = (*error_blob).GetBufferSize();
        let error = String::from_utf8_unchecked(Vec::from_raw_parts(text, size, size));
        panic!("{error}")
    } else if hr != S_OK {
        handle_hresult_error(hr, "Failed to compile shader.");
    }

    write_file(
        path,
        std::slice::from_raw_parts((*shader_blob).GetBufferPointer() as *const u8, (*shader_blob).GetBufferSize()),
    );

    (*shader_blob).Release();
}

fn write_file<C: AsRef<[u8]>>(path: &str, content: C) {
    let out_dir = {
        if cfg!(debug_assertions) {
            "debug/"
        } else {
            "release/"
        }
    };
    let asset_path = format!("../target/assets/{out_dir}{path}");
    let file_path = Path::new(&asset_path);
    let directory = file_path.parent().unwrap();

    if !directory.is_dir() {
        fs::create_dir_all(directory).unwrap_or_else(|e| panic!("Failed to create directory '{directory:?}': {e}."));
    }

    fs::write(file_path, content).unwrap_or_else(|e| panic!("Failed to write file '{asset_path}': {e}."));
}
