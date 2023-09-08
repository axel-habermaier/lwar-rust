use core::ptr::null_mut;

use winapi::um::{
    processthreadsapi::{GetCurrentProcess, TerminateProcess},
    winuser::{MessageBoxA, MB_ICONERROR, MB_OK},
};

use super::win_str::WinStr;

pub fn exit() -> ! {
    terminate(0)
}

pub fn fatal(message: &str) -> ! {
    unsafe {
        MessageBoxA(
            null_mut(),
            WinStr::from(message).as_ptr(),
            WinStr::from("Orbs: Fatal Error").as_ptr(),
            MB_ICONERROR | MB_OK,
        );
    }

    terminate(1)
}

fn terminate(exit_code: u32) -> ! {
    unsafe {
        TerminateProcess(GetCurrentProcess(), exit_code);

        // This panic is never executed, but necessary to satisfy the type checker.
        panic!("")
    }
}
