use std::{
    env::set_current_dir,
    ffi::{CString, OsStr},
    fs,
    os::windows::prelude::OsStrExt,
    panic::set_hook,
    path::Path,
    process::exit,
    ptr::{null, null_mut},
};
use winapi::um::d3dcompiler::{D3DCompileFromFile, D3DCOMPILE_DEBUG, D3DCOMPILE_ENABLE_STRICTNESS};

fn main() {
    setup_panic_handler();
    set_current_dir("assets/").unwrap();
    create_dir("../target/assets/shaders/");

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
            OsStr::new(path).encode_wide().chain([0]).collect::<Vec<_>>().as_ptr(),
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
            panic!("{}", error)
        }

        fs::write(
            format!(
                "../target/assets/shaders/{}.rs",
                Path::new(path).file_stem().unwrap().to_str().unwrap().to_owned()
            ),
            std::slice::from_raw_parts((*shader_blob).GetBufferPointer() as *const u8, (*shader_blob).GetBufferSize()),
        )
        .unwrap();
    }
}

fn setup_panic_handler() {
    set_hook(Box::new(|panic_info| {
        let error_message = {
            // Formatted strings such as `panic!("{}", 1)` are `String` instances.
            if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s
            // Constant strings such as `panic!("") are `&str` instances.
            } else if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s
            } else {
                "An unknown error occurred."
            }
        };

        println!("{}", error_message);
        exit(-1);
    }));
}

fn create_dir(dir: &str) {
    if !Path::new(dir).is_dir() {
        fs::create_dir_all(dir).unwrap();
    }
}
