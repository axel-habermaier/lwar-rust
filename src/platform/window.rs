use core::{mem::size_of, ptr};
use std::{cell::Cell, ffi::CString, ptr::null_mut};

use winapi::{
    ctypes::c_void,
    shared::{
        minwindef::{LPARAM, LRESULT, UINT, WPARAM},
        windef::HWND,
    },
    um::{libloaderapi::GetModuleHandleA, winuser::*},
};

use super::get_last_error;

#[derive(Debug)]
pub enum Event {
    Initialized,
    CloseRequested,
    Exiting,
    UpdateAndRender,
}

pub fn execute_in_window(mut event_callback: impl FnMut(&Event, &mut bool)) {
    unsafe {
        let exit_cell = Cell::new(false);
        let mut handle_event = |event: &Event| {
            let mut exit = exit_cell.get();
            event_callback(event, &mut exit);
            exit_cell.set(exit);
        };

        let mut handle_event_ref: &mut dyn FnMut(&Event) = &mut handle_event;
        let hwnd = open_window(&mut handle_event_ref as *mut _ as *mut c_void);

        handle_event(&Event::Initialized);

        while !exit_cell.get() {
            process_events(hwnd);
            handle_event(&Event::UpdateAndRender);
        }

        handle_event(&Event::Exiting);
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_CREATE {
        let handle_event_ptr = (*(lparam as *const CREATESTRUCTA)).lpCreateParams;
        SetWindowLongPtrA(hwnd, GWLP_USERDATA, handle_event_ptr as isize);
    }

    let handle_event_ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut c_void;
    if handle_event_ptr.is_null() {
        return DefWindowProcA(hwnd, msg, wparam, lparam);
    }

    let handle_event: &mut &mut dyn FnMut(&Event) = &mut *(handle_event_ptr as *mut _);

    match msg {
        WM_CHAR => toggle_mode(hwnd),
        WM_CLOSE => handle_event(&Event::CloseRequested),
        _ => (),
    };

    DefWindowProcA(hwnd, msg, wparam, lparam)
}

unsafe fn open_window(event_callback: *mut c_void) -> HWND {
    let name = CString::new("WndClass").unwrap();
    let title = CString::new("Orbs").unwrap();

    let wnd_class = WNDCLASSA {
        lpfnWndProc: Some(wnd_proc),
        lpszClassName: name.as_ptr(),
        hInstance: GetModuleHandleA(ptr::null()),
        style: CS_DBLCLKS,
        ..Default::default()
    };

    if RegisterClassA(&wnd_class) == 0 {
        panic!("Failed to register window class. {}", get_last_error());
    }

    let device = RAWINPUTDEVICE {
        usUsagePage: 0x01, // keyboard
        usUsage: 0x06,     // keyboard
        ..Default::default()
    };

    if RegisterRawInputDevices(&device, 1, size_of::<RAWINPUTDEVICE>() as u32) == 0 {
        panic!("Failed to register raw input device. {}", get_last_error());
    }

    let hwnd = CreateWindowExA(
        0,
        name.as_ptr(),
        title.as_ptr(),
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        ptr::null_mut(),
        ptr::null_mut(),
        GetModuleHandleA(ptr::null()),
        event_callback,
    );

    if hwnd.is_null() {
        panic!("Failed to create window. {}", get_last_error());
    }

    hwnd
}

unsafe fn process_events(hwnd: HWND) {
    let mut msg: MSG = Default::default();
    while PeekMessageA(&mut msg, hwnd, 0, 0, PM_REMOVE) != 0 {
        TranslateMessage(&msg);
        DispatchMessageA(&msg);
    }
}

unsafe fn toggle_mode(hwnd: HWND) {
    let style = GetWindowLongPtrA(hwnd, GWL_STYLE);
    let is_fullscreen = (style & WS_THICKFRAME as isize) != WS_THICKFRAME as isize;

    if is_fullscreen {
        let style = style | WS_OVERLAPPEDWINDOW as isize;
        if SetWindowLongPtrA(hwnd, GWL_STYLE, style) == 0 {
            panic!("Failed to set new window style. {}", get_last_error());
        }

        if ShowWindow(hwnd, SW_RESTORE) == 0 {
            panic!("Failed to restore windowed mode. {}", get_last_error());
        }
    } else {
        let style = style & !WS_OVERLAPPEDWINDOW as isize;
        if SetWindowLongPtrA(hwnd, GWL_STYLE, style) == 0 {
            panic!("Failed to set fullscreen window style. {}", get_last_error());
        }

        // We have to resize the window manually if it is maximized to get rid of the taskbar.
        if IsZoomed(hwnd) != 0 {
            let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            let mut monitor_info = MONITORINFO {
                cbSize: size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };

            if GetMonitorInfoA(monitor, &mut monitor_info) == 0 {
                panic!("Failed to get monitor info. {}", get_last_error());
            }

            let x = monitor_info.rcMonitor.left;
            let y = monitor_info.rcMonitor.top;
            let width = monitor_info.rcMonitor.right - x;
            let height = monitor_info.rcMonitor.bottom - y;

            if SetWindowPos(hwnd, null_mut(), x, y, width, height, SWP_FRAMECHANGED | SWP_SHOWWINDOW) == 0 {
                panic!("Failed to change to fullscreen window style. {}", get_last_error());
            }
        } else if ShowWindow(hwnd, SW_SHOWMAXIMIZED) == 0 {
            panic!("Failed to maximize fullscreen window. {}", get_last_error());
        }
    }
}
