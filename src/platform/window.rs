use core::{mem::size_of, ptr, ptr::null_mut};

use winapi::{
    shared::{
        minwindef::{LPARAM, LRESULT, UINT, WPARAM},
        windef::HWND,
    },
    um::{libloaderapi::GetModuleHandleA, winuser::*},
};

use super::{
    process::{exit, fatal},
    win_str::WinStr,
};

static mut _HWND: u64 = 0;

fn hwnd() -> HWND {
    unsafe { _HWND as HWND }
}

extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_CLOSE {
        exit();
    }
    if msg == WM_CHAR {
        toggle_mode();
    }
    unsafe { DefWindowProcA(hwnd, msg, wparam, lparam) }
}

pub fn open_window() {
    unsafe {
        let name = WinStr::from("WndClass");
        let wnd_class = WNDCLASSA {
            lpfnWndProc: Some(wnd_proc),
            lpszClassName: name.as_ptr(),
            hInstance: GetModuleHandleA(ptr::null()),
            style: CS_DBLCLKS,
            ..Default::default()
        };

        if RegisterClassA(&wnd_class) == 0 {
            fatal("Failed to register window class.")
        }

        let device = RAWINPUTDEVICE {
            usUsagePage: 0x01, // keyboard
            usUsage: 0x06,     // keyboard
            ..Default::default()
        };

        if RegisterRawInputDevices(&device, 1, size_of::<RAWINPUTDEVICE>() as u32) == 0 {
            fatal("Failed to register raw input device.");
        }

        _HWND = CreateWindowExA(
            0,
            name.as_ptr(),
            WinStr::from("Orbs").as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            ptr::null_mut(),
            ptr::null_mut(),
            GetModuleHandleA(ptr::null()),
            ptr::null_mut(),
        ) as u64;

        if hwnd().is_null() {
            fatal("Failed to create window.");
        }
    }
}

pub fn process_messages() {
    unsafe {
        let mut msg: MSG = Default::default();
        while PeekMessageA(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }
}

pub fn toggle_mode() {
    unsafe {
        let style = GetWindowLongPtrA(hwnd(), GWL_STYLE);
        let is_fullscreen = (style & WS_THICKFRAME as isize) != WS_THICKFRAME as isize;

        if is_fullscreen {
            let style = style | WS_OVERLAPPEDWINDOW as isize;
            if SetWindowLongPtrA(hwnd(), GWL_STYLE, style) == 0 {
                fatal("Failed to set new window style.");
            }

            if ShowWindow(hwnd(), SW_RESTORE) == 0 {
                fatal("Failed to restore windowed mode.");
            }
        } else {
            let style = style & !WS_OVERLAPPEDWINDOW as isize;
            if SetWindowLongPtrA(hwnd(), GWL_STYLE, style) == 0 {
                fatal("Failed to set fullscreen window style.");
            }

            // We have to resize the window manually if it is maximized to get rid of the taskbar.
            if IsZoomed(hwnd()) != 0 {
                let monitor = MonitorFromWindow(hwnd(), MONITOR_DEFAULTTONEAREST);
                let mut monitor_info = MONITORINFO {
                    cbSize: size_of::<MONITORINFO>() as u32,
                    ..Default::default()
                };

                if GetMonitorInfoA(monitor, &mut monitor_info) == 0 {
                    fatal("Failed to get monitor info.");
                }

                let x = monitor_info.rcMonitor.left;
                let y = monitor_info.rcMonitor.top;
                let width = monitor_info.rcMonitor.right - x;
                let height = monitor_info.rcMonitor.bottom - y;

                if SetWindowPos(hwnd(), null_mut(), x, y, width, height, SWP_FRAMECHANGED | SWP_SHOWWINDOW) == 0 {
                    fatal("Failed to change to fullscreen window style.");
                }
            } else if ShowWindow(hwnd(), SW_SHOWMAXIMIZED) == 0 {
                fatal("Failed to maximize fullscreen window.");
            }
        }
    }
}
