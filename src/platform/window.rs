use super::{
    error::get_last_error,
    input::{Key, MouseButton},
};
use core::{mem::size_of, ptr};
use std::{
    ffi::CString,
    ptr::{null, null_mut},
};
use winapi::{
    shared::{minwindef::*, windef::*},
    um::{libloaderapi::GetModuleHandleA, winuser::*},
};

const WINDOW_TITLE: *const i8 = b"Orbs\0".as_ptr() as *const i8;

pub struct Window {
    hwnd: HWND,
}

pub enum Event {
    CloseRequested,
    Resized(u32, u32),
    KeyPressed(Key, u32),
    KeyReleased(Key, u32),
    CharacterEntered(char),
    MouseMoved(u32, u32),
    MousePressed(MouseButton),
    MouseReleased(MouseButton),
    MouseWheel(i32),
}

impl Window {
    pub fn new() -> Window {
        unsafe {
            let wnd_class = WNDCLASSA {
                lpfnWndProc: Some(wnd_proc),
                lpszClassName: WINDOW_TITLE,
                hInstance: GetModuleHandleA(ptr::null()),
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
            };

            let hwnd = CreateWindowExA(
                0,
                WINDOW_TITLE,
                WINDOW_TITLE,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                ptr::null_mut(),
                ptr::null_mut(),
                GetModuleHandleA(ptr::null()),
                null_mut(),
            );

            if hwnd.is_null() {
                panic!("Failed to create window. {}", get_last_error());
            }

            if !cfg!(debug_assertions) {
                toggle_fullscreen(hwnd);
            }

            Window { hwnd }
        }
    }

    pub fn handle_events(&mut self, mut handle_event: impl FnMut(Event)) {
        let old_size = self.size();

        unsafe {
            let mut handler: &mut dyn FnMut(Event) = &mut handle_event;
            SetWindowLongPtrA(self.hwnd, GWLP_USERDATA, &mut handler as *mut _ as isize);

            let mut msg: MSG = Default::default();
            while PeekMessageA(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }

            SetWindowLongPtrA(self.hwnd, GWLP_USERDATA, 0);
        }

        let new_size = self.size();
        if old_size != new_size && unsafe { IsIconic(self.hwnd) } == 0 {
            handle_event(Event::Resized(new_size.0, new_size.1));
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn size(&self) -> (u32, u32) {
        let mut rect = RECT::default();
        if unsafe { GetClientRect(self.hwnd, &mut rect) } == 0 {
            panic!("Failed to retrieve window size. {}", get_last_error());
        }

        (rect.right as u32 - rect.left as u32, rect.bottom as u32 - rect.top as u32)
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            SetWindowLongPtrA(self.hwnd, GWLP_USERDATA, 0);
            CloseWindow(self.hwnd);
            UnregisterClassA(WINDOW_TITLE, GetModuleHandleA(null()));
        };
    }
}

unsafe fn toggle_fullscreen(hwnd: HWND) {
    let style = GetWindowLongPtrA(hwnd, GWL_STYLE);
    if style == 0 {
        panic!("Failed to retrieve window style. {}", get_last_error());
    }

    let is_fullscreen = (style & WS_THICKFRAME as isize) != WS_THICKFRAME as isize;

    if is_fullscreen {
        let style = style | WS_OVERLAPPEDWINDOW as isize;
        if SetWindowLongPtrA(hwnd, GWL_STYLE, style) == 0 {
            panic!("Failed to set new window style. {}", get_last_error());
        }

        ShowWindow(hwnd, SW_RESTORE);
    } else {
        let style = style & !WS_OVERLAPPEDWINDOW as isize;
        if SetWindowLongPtrA(hwnd, GWL_STYLE, style) == 0 {
            panic!("Failed to set fullscreen window style. {}", get_last_error());
        }

        if IsZoomed(hwnd) != 0 {
            // Necessary to get rid of the taskbar.
            ShowWindow(hwnd, SW_RESTORE);
        }

        ShowWindow(hwnd, SW_SHOWMAXIMIZED);
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let event_ptr = if msg == WM_CREATE {
        (*(lparam as *const CREATESTRUCTA)).lpCreateParams
    } else {
        GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut _
    };

    if event_ptr.is_null() {
        return DefWindowProcA(hwnd, msg, wparam, lparam);
    }

    let handle_event: &mut &mut dyn FnMut(Event) = &mut *(event_ptr as *mut _);

    match msg {
        WM_INPUT => handle_keyboard_input(lparam, handle_event),
        WM_SYSCOMMAND if wparam == SC_KEYMENU => {
            return 0;
        }
        // Toggle fullscreen on ALT + ENTER.
        WM_SYSKEYDOWN if wparam == VK_RETURN as usize && (lparam & 0x60000000) == 0x20000000 => toggle_fullscreen(hwnd),
        WM_CLOSE => {
            handle_event(Event::CloseRequested);
            return 0;
        }
        WM_GETMINMAXINFO => {
            let info = lparam as *mut MINMAXINFO;
            (*info).ptMinTrackSize.x = 640;
            (*info).ptMinTrackSize.y = 480;
        }
        WM_MOUSEMOVE => handle_event(Event::MouseMoved(LOWORD(lparam as u32) as u32, HIWORD(lparam as u32) as u32)),
        WM_LBUTTONDOWN => handle_event(Event::MousePressed(MouseButton::Left)),
        WM_LBUTTONUP => handle_event(Event::MouseReleased(MouseButton::Left)),
        WM_RBUTTONDOWN => handle_event(Event::MousePressed(MouseButton::Right)),
        WM_RBUTTONUP => handle_event(Event::MouseReleased(MouseButton::Right)),
        WM_MBUTTONDOWN => handle_event(Event::MousePressed(MouseButton::Middle)),
        WM_MBUTTONUP => handle_event(Event::MouseReleased(MouseButton::Middle)),
        WM_XBUTTONDOWN => handle_event(Event::MousePressed(if HIWORD(wparam as u32) == XBUTTON1 {
            MouseButton::XButton1
        } else {
            MouseButton::XButton2
        })),
        WM_XBUTTONUP => handle_event(Event::MouseReleased(if HIWORD(wparam as u32) == XBUTTON1 {
            MouseButton::XButton1
        } else {
            MouseButton::XButton2
        })),
        WM_MOUSEWHEEL => handle_event(Event::MouseWheel((GET_WHEEL_DELTA_WPARAM(wparam) / WHEEL_DELTA) as i32)),
        WM_CHAR => {
            if let Some(character) = char::from_u32(wparam as u32) {
                handle_event(Event::CharacterEntered(character))
            }
        }
        _ => (),
    };

    DefWindowProcA(hwnd, msg, wparam, lparam)
}

unsafe fn handle_keyboard_input(lparam: LPARAM, handle_event: &mut dyn FnMut(Event)) {
    let mut input = RAWINPUT::default();
    let mut size = size_of::<RAWINPUT>() as u32;
    let success = GetRawInputData(
        lparam as HRAWINPUT,
        RID_INPUT,
        &mut input as *mut _ as *mut _,
        &mut size as *mut _,
        size_of::<RAWINPUTHEADER>() as u32,
    );

    if success == u32::MAX {
        panic!("Failed to read raw keyboard input. {}", get_last_error());
    }

    // Extract keyboard raw input data; see http://molecularmusings.wordpress.com/2011/09/05/properly-handling-keyboard-input/.
    if input.header.dwType == RIM_TYPEKEYBOARD {
        let mut virtual_key = input.data.keyboard().VKey as i32;
        let mut scan_code = input.data.keyboard().MakeCode as u32;
        let flags = input.data.keyboard().Flags as u32;

        let released = (flags & RI_KEY_BREAK) != 0;

        if virtual_key == 255 {
            return;
        }

        if virtual_key == VK_SHIFT {
            virtual_key = MapVirtualKeyA(scan_code, MAPVK_VSC_TO_VK_EX) as i32;
        } else if virtual_key == VK_NUMLOCK {
            scan_code = MapVirtualKeyA(virtual_key as u32, MAPVK_VK_TO_VSC) | 0x100;
        }

        let is_e0 = (flags & RI_KEY_E0) != 0;
        let is_e1 = (flags & RI_KEY_E1) != 0;

        if is_e1 {
            if virtual_key == VK_PAUSE {
                scan_code = 0x45;
            } else {
                scan_code = MapVirtualKeyA(virtual_key as u32, MAPVK_VK_TO_VSC);
            }
        }

        let key = match virtual_key {
            VK_CONTROL => Some(if is_e0 { Key::RightControl } else { Key::LeftControl }),
            VK_MENU => Some(if is_e0 { Key::RightAlt } else { Key::LeftAlt }),
            VK_RETURN => Some(if is_e0 { Key::NumpadEnter } else { Key::Return }),
            VK_INSERT => Some(if !is_e0 { Key::Numpad0 } else { Key::Insert }),
            VK_DELETE => Some(if !is_e0 { Key::NumpadDecimal } else { Key::Delete }),
            VK_HOME => Some(if !is_e0 { Key::Numpad7 } else { Key::Home }),
            VK_END => Some(if !is_e0 { Key::Numpad1 } else { Key::End }),
            VK_PRIOR => Some(if !is_e0 { Key::Numpad9 } else { Key::PageUp }),
            VK_NEXT => Some(if !is_e0 { Key::Numpad3 } else { Key::PageDown }),
            VK_LEFT => Some(if !is_e0 { Key::Numpad4 } else { Key::Left }),
            VK_RIGHT => Some(if !is_e0 { Key::Numpad6 } else { Key::Right }),
            VK_UP => Some(if !is_e0 { Key::Numpad8 } else { Key::Up }),
            VK_DOWN => Some(if !is_e0 { Key::Numpad2 } else { Key::Down }),
            VK_CLEAR if !is_e0 => Some(Key::Numpad5),
            VK_CLEAR if is_e0 => None,
            _ => match Key::try_from(virtual_key) {
                Ok(key) => Some(key),
                Err(_) => {
                    println!("An unknown key was pressed. Virtual key code: '{virtual_key}'.");
                    None
                }
            },
        };

        if let Some(key) = key {
            if released {
                handle_event(Event::KeyReleased(key, scan_code));
            } else {
                handle_event(Event::KeyPressed(key, scan_code));
            }
        }
    }
}

pub fn show_message_box<T: AsRef<str>>(content: T) {
    let message = CString::new(content.as_ref()).unwrap();

    unsafe {
        MessageBoxA(
            null_mut(),
            message.as_ptr(),
            WINDOW_TITLE,
            MB_ICONERROR | MB_OK | MB_TASKMODAL | MB_TOPMOST,
        );
    }
}
