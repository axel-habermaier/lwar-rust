#![warn(clippy::all)]
#![windows_subsystem = "windows"]
#![no_std]
#![no_main]
#![feature(lang_items)]

#[allow(dead_code)]
#[no_mangle]
pub extern "stdcall" fn orbs_main() -> ! {
    orbs::run()
}

#[no_mangle]
pub static _fltused: i32 = 1;

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic_handler(_panic_info: &core::panic::PanicInfo) -> ! {
    orbs::platform::process::fatal("Unknown error.")
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn __CxxFrameHandler3(
    _record: usize,
    _frame: usize,
    _context: usize,
    _dispatcher: usize,
) -> u32 {
    1
}
