use super::{
    error::get_last_error,
    input::{Key, MouseButton},
};
use core::{mem::size_of, ptr};
use std::{
    mem::MaybeUninit,
    ptr::{null, null_mut},
};
use winapi::{
    ctypes::c_void,
    shared::{
        minwindef::{HIWORD, LOWORD, LPARAM, LRESULT, UINT, WPARAM},
        windef::HWND,
    },
    um::{libloaderapi::GetModuleHandleA, winuser::*},
};

const WINDOW_TITLE: *const u8 = b"Orbs\0".as_ptr();

struct EventHandler<'a, 'b> {
    window: &'a mut Window,
    handle_event: &'b mut dyn FnMut(&Event),
}

pub struct Window {
    cursor_inside: bool,
    has_focus: bool,
    width: u32,
    height: u32,
    hwnd: HWND,
}

#[derive(Debug)]
pub enum Event {
    CloseRequested,
    Resized(u32, u32),
    Focused(bool),
    KeyPressed(Key, u32),
    KeyReleased(Key, u32),
    CharacterEntered(char),
    MouseEntered,
    MouseLeft,
    MouseMoved(u32, u32),
    MousePressed(MouseButton),
    MouseReleased(MouseButton),
    MouseWheel(i32),
}

impl Default for Window {
    fn default() -> Window {
        unsafe {
            let mut window = Window {
                cursor_inside: false,
                has_focus: false,
                width: 0,
                height: 0,
                hwnd: null_mut(),
            };

            let wnd_class = WNDCLASSA {
                lpfnWndProc: Some(wnd_proc),
                lpszClassName: WINDOW_TITLE as *const _,
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

            let mut event_handler = EventHandler {
                window: &mut window,
                handle_event: &mut |_| {},
            };

            let hwnd = CreateWindowExA(
                0,
                WINDOW_TITLE as *const _,
                WINDOW_TITLE as *const _,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                ptr::null_mut(),
                ptr::null_mut(),
                GetModuleHandleA(ptr::null()),
                // Some window events are important to keep the window's internal state
                // up-to-date immediately after opening. So let's capture them here.
                &mut event_handler as *mut _ as *mut _,
            );

            if hwnd.is_null() {
                panic!("Failed to create window. {}", get_last_error());
            }

            if !cfg!(debug_assertions) {
                toggle_fullscreen(hwnd);
            }

            window
        }
    }
}

impl Window {
    pub fn handle_events(&mut self, mut handle_event: impl FnMut(&Event)) {
        let width = self.width;
        let height = self.height;
        let has_focus = self.has_focus;
        let hwnd = self.hwnd;

        let mut event_handler = EventHandler {
            window: self,
            handle_event: &mut handle_event,
        };

        unsafe {
            SetWindowLongPtrA(hwnd, GWLP_USERDATA, &mut event_handler as *mut _ as isize);

            let mut msg: MSG = Default::default();
            while PeekMessageA(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }

        if has_focus != self.has_focus {
            handle_event(&Event::Focused(self.has_focus));
        }

        if width != self.width || height != self.height {
            handle_event(&Event::Resized(self.width, self.height));
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            SetWindowLongPtrA(self.hwnd, GWLP_USERDATA, 0);
            CloseWindow(self.hwnd);
            UnregisterClassA(WINDOW_TITLE as *const _, GetModuleHandleA(null()));
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
            // If the Window is already maximized, we have to un-maximize it first to get rid of the taskbar.
            ShowWindow(hwnd, SW_RESTORE);
        }

        ShowWindow(hwnd, SW_SHOWMAXIMIZED);
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let event_ptr = if msg == WM_CREATE {
        (*(lparam as *const CREATESTRUCTA)).lpCreateParams
    } else {
        GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut c_void
    };

    if event_ptr.is_null() {
        return DefWindowProcA(hwnd, msg, wparam, lparam);
    }

    let handler: &mut EventHandler = &mut *(event_ptr as *mut EventHandler);
    let handle_event = &mut handler.handle_event;
    let window = &mut handler.window;

    match msg {
        WM_CREATE => window.hwnd = hwnd,
        WM_INPUT => handle_keyboard_input(lparam, handle_event),
        WM_SYSCOMMAND if wparam == SC_KEYMENU => {
            return 0;
        }
        // Toggle fullscreen on ALT + ENTER.
        WM_SYSKEYDOWN if wparam == VK_RETURN as usize && (lparam & 0x60000000) == 0x20000000 => toggle_fullscreen(hwnd),
        WM_CLOSE => {
            handle_event(&Event::CloseRequested);
            // Do not forward the message to the default wnd proc, as we want full control over when the window is actually closed.
            return 0;
        }
        WM_GETMINMAXINFO => {
            // Restrict the minimum allowed window size.
            let info = lparam as *mut MINMAXINFO;
            (*info).ptMinTrackSize.x = 600;
            (*info).ptMinTrackSize.y = 400;
        }
        // Check WM_ACTIVATE, WM_NCACTIVATE, WM_ACTIVATEAPP in order to ensure that we do not miss an activation or deactivation.
        WM_ACTIVATE => window.has_focus = LOWORD(wparam as u32) != WA_INACTIVE,
        WM_NCACTIVATE | WM_ACTIVATEAPP => window.has_focus = wparam != 0,
        WM_MOUSEMOVE => {
            // If the cursor is entering the window, raise the mouse entered event and tell Windows to inform
            // us when the cursor leaves the window.
            if !window.cursor_inside {
                let mut mouse_event = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    hwndTrack: hwnd,
                    dwFlags: TME_LEAVE,
                    ..Default::default()
                };
                TrackMouseEvent(&mut mouse_event);

                window.cursor_inside = true;
                handle_event(&Event::MouseEntered);
            } else {
                handle_event(&Event::MouseMoved(LOWORD(lparam as u32) as u32, HIWORD(lparam as u32) as u32));
            }
        }
        WM_MOUSELEAVE => {
            window.cursor_inside = false;
            handle_event(&Event::MouseLeft);
        }
        WM_LBUTTONDOWN => handle_event(&Event::MousePressed(MouseButton::Left)),
        WM_LBUTTONUP => handle_event(&Event::MouseReleased(MouseButton::Left)),
        WM_RBUTTONDOWN => handle_event(&Event::MousePressed(MouseButton::Right)),
        WM_RBUTTONUP => handle_event(&Event::MouseReleased(MouseButton::Right)),
        WM_MBUTTONDOWN => handle_event(&Event::MousePressed(MouseButton::Middle)),
        WM_MBUTTONUP => handle_event(&Event::MouseReleased(MouseButton::Middle)),
        WM_XBUTTONDOWN => handle_event(&Event::MousePressed(if HIWORD(wparam as u32) == XBUTTON1 {
            MouseButton::XButton1
        } else {
            MouseButton::XButton2
        })),
        WM_XBUTTONUP => handle_event(&Event::MouseReleased(if HIWORD(wparam as u32) == XBUTTON1 {
            MouseButton::XButton1
        } else {
            MouseButton::XButton2
        })),
        WM_MOUSEWHEEL => handle_event(&Event::MouseWheel((GET_WHEEL_DELTA_WPARAM(wparam) / WHEEL_DELTA) as i32)),
        WM_CHAR => {
            if let Some(character) = char::from_u32(wparam as u32) {
                handle_event(&Event::CharacterEntered(character))
            }
        }
        WM_SIZE => {
            window.width = LOWORD(lparam as u32) as u32;
            window.height = HIWORD(lparam as u32) as u32;
        }
        _ => (),
    };

    DefWindowProcA(hwnd, msg, wparam, lparam)
}

unsafe fn handle_keyboard_input(lparam: LPARAM, handle_event: &mut dyn FnMut(&Event)) {
    let mut input = MaybeUninit::<RAWINPUT>::uninit();
    let mut size = size_of::<RAWINPUT>() as u32;
    let success = GetRawInputData(
        lparam as HRAWINPUT,
        RID_INPUT,
        input.as_mut_ptr() as *mut _,
        &mut size as *mut _,
        size_of::<RAWINPUTHEADER>() as u32,
    );

    if success == u32::MAX {
        panic!("Failed to read raw keyboard input. {}", get_last_error());
    }

    // Extract keyboard raw input data; see http://molecularmusings.wordpress.com/2011/09/05/properly-handling-keyboard-input/
    // for an explanation of what's going on.
    let input = input.assume_init();
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
                handle_event(&Event::KeyReleased(key, scan_code));
            } else {
                handle_event(&Event::KeyPressed(key, scan_code));
            }
        }
    }
}
