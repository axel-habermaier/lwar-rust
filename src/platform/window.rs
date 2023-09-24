use super::{
    error::get_last_error,
    input::{Key, MouseButton},
};
use core::{mem::size_of, ptr};
use std::{
    ffi::CString,
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

struct WindowData {
    cursor_inside: bool,
    has_focus: bool,
    width: u32,
    height: u32,
    events: Vec<Event>,
    hwnd: HWND,
}

pub struct Window {
    data: Box<WindowData>,
    class_name: CString,
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

impl Window {
    pub fn new(title: &str) -> Window {
        unsafe {
            let title = CString::new(title).unwrap();

            let wnd_class = WNDCLASSA {
                lpfnWndProc: Some(wnd_proc),
                lpszClassName: title.as_ptr(),
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

            let mut data = Box::new(WindowData {
                cursor_inside: false,
                has_focus: false,
                width: 0,
                height: 0,
                events: Vec::with_capacity(32),
                hwnd: null_mut(),
            });

            data.hwnd = CreateWindowExA(
                0,
                title.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                ptr::null_mut(),
                ptr::null_mut(),
                GetModuleHandleA(ptr::null()),
                &mut *data as *mut _ as *mut c_void,
            );

            if data.hwnd.is_null() {
                panic!("Failed to create window. {}", get_last_error());
            }

            if !cfg!(debug_assertions) {
                toggle_fullscreen(data.hwnd);
            }

            Window { data, class_name: title }
        }
    }

    pub fn pending_events(&mut self) -> impl Iterator<Item = &Event> {
        self.data.events.clear();
        let width = self.data.width;
        let height = self.data.height;
        let has_focus = self.data.has_focus;

        unsafe {
            let mut msg: MSG = Default::default();
            while PeekMessageA(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }

        if has_focus != self.data.has_focus {
            self.data.events.push(Event::Focused(self.data.has_focus));
        }

        if width != self.data.width || height != self.data.height {
            self.data.events.push(Event::Resized(self.data.width, self.data.height));
        }

        self.data.events.iter()
    }

    pub fn hwnd(&self) -> HWND {
        self.data.hwnd
    }

    pub fn size(&self) -> (u32, u32) {
        (self.data.width, self.data.height)
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            CloseWindow(self.data.hwnd);
            UnregisterClassA(self.class_name.as_ptr(), GetModuleHandleA(null()));
        };
    }
}

unsafe fn toggle_fullscreen(hwnd: HWND) {
    let style = GetWindowLongPtrA(hwnd, GWL_STYLE);
    if style == 0 {
        panic!("Failed to retrieve Window style. {}", get_last_error());
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
    if msg == WM_CREATE {
        let data_ptr = (*(lparam as *const CREATESTRUCTA)).lpCreateParams;
        SetWindowLongPtrA(hwnd, GWLP_USERDATA, data_ptr as isize);
    }

    let data_ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut c_void;
    if data_ptr.is_null() {
        return DefWindowProcA(hwnd, msg, wparam, lparam);
    }

    let data: &mut WindowData = &mut *(data_ptr as *mut WindowData);

    match msg {
        WM_INPUT => handle_keyboard_input(lparam, &mut data.events),
        WM_SYSCOMMAND if wparam == SC_KEYMENU => {
            return 0;
        }
        // Toggle fullscreen on ALT + ENTER.
        WM_SYSKEYDOWN if wparam == VK_RETURN as usize && (lparam & 0x60000000) == 0x20000000 => toggle_fullscreen(hwnd),
        WM_CLOSE => {
            data.events.push(Event::CloseRequested);
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
        WM_ACTIVATE => data.has_focus = LOWORD(wparam as u32) != WA_INACTIVE,
        WM_NCACTIVATE | WM_ACTIVATEAPP => data.has_focus = wparam != 0,
        WM_MOUSEMOVE => {
            // If the cursor is entering the window, raise the mouse entered event and tell Windows to inform
            // us when the cursor leaves the window.
            if !data.cursor_inside {
                let mut mouse_event = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    hwndTrack: hwnd,
                    dwFlags: TME_LEAVE,
                    ..Default::default()
                };
                TrackMouseEvent(&mut mouse_event);

                data.cursor_inside = true;
                data.events.push(Event::MouseEntered);
            } else {
                data.events.push(Event::MouseMoved(LOWORD(lparam as u32) as u32, HIWORD(lparam as u32) as u32));
            }
        }
        WM_MOUSELEAVE => {
            data.cursor_inside = false;
            data.events.push(Event::MouseLeft);
        }
        WM_LBUTTONDOWN => data.events.push(Event::MousePressed(MouseButton::Left)),
        WM_LBUTTONUP => data.events.push(Event::MouseReleased(MouseButton::Left)),
        WM_RBUTTONDOWN => data.events.push(Event::MousePressed(MouseButton::Right)),
        WM_RBUTTONUP => data.events.push(Event::MouseReleased(MouseButton::Right)),
        WM_MBUTTONDOWN => data.events.push(Event::MousePressed(MouseButton::Middle)),
        WM_MBUTTONUP => data.events.push(Event::MouseReleased(MouseButton::Middle)),
        WM_XBUTTONDOWN => data.events.push(Event::MousePressed(if HIWORD(wparam as u32) == XBUTTON1 {
            MouseButton::XButton1
        } else {
            MouseButton::XButton2
        })),
        WM_XBUTTONUP => data.events.push(Event::MouseReleased(if HIWORD(wparam as u32) == XBUTTON1 {
            MouseButton::XButton1
        } else {
            MouseButton::XButton2
        })),
        WM_MOUSEWHEEL => data.events.push(Event::MouseWheel((GET_WHEEL_DELTA_WPARAM(wparam) / WHEEL_DELTA) as i32)),
        WM_CHAR => {
            if let Some(character) = char::from_u32(wparam as u32) {
                data.events.push(Event::CharacterEntered(character))
            }
        }
        WM_SIZE => {
            data.width = LOWORD(lparam as u32) as u32;
            data.height = HIWORD(lparam as u32) as u32;
        }
        _ => (),
    };

    DefWindowProcA(hwnd, msg, wparam, lparam)
}

fn translate_key(virtual_key: i32) -> Option<Key> {
    match virtual_key {
        VK_OEM_102 => Some(Key::BackSlash2),
        VK_SCROLL => Some(Key::Scroll),
        VK_SNAPSHOT => Some(Key::Print),
        VK_NUMLOCK => Some(Key::NumLock),
        VK_DECIMAL => Some(Key::NumpadDecimal),
        VK_LSHIFT => Some(Key::LeftShift),
        VK_RSHIFT => Some(Key::RightShift),
        VK_LWIN => Some(Key::LeftSystem),
        VK_RWIN => Some(Key::RightSystem),
        VK_APPS => Some(Key::Menu),
        VK_OEM_1 => Some(Key::Semicolon),
        VK_OEM_2 => Some(Key::Slash),
        VK_OEM_PLUS => Some(Key::Equal),
        VK_OEM_MINUS => Some(Key::Dash),
        VK_OEM_4 => Some(Key::LeftBracket),
        VK_OEM_6 => Some(Key::RightBracket),
        VK_OEM_COMMA => Some(Key::Comma),
        VK_OEM_PERIOD => Some(Key::Period),
        VK_OEM_7 => Some(Key::Quote),
        VK_OEM_5 => Some(Key::BackSlash),
        VK_OEM_3 => Some(Key::Grave),
        VK_ESCAPE => Some(Key::Escape),
        VK_SPACE => Some(Key::Space),
        VK_RETURN => Some(Key::Return),
        VK_BACK => Some(Key::Back),
        VK_TAB => Some(Key::Tab),
        VK_PRIOR => Some(Key::PageUp),
        VK_NEXT => Some(Key::PageDown),
        VK_END => Some(Key::End),
        VK_HOME => Some(Key::Home),
        VK_INSERT => Some(Key::Insert),
        VK_DELETE => Some(Key::Delete),
        VK_ADD => Some(Key::Add),
        VK_SUBTRACT => Some(Key::Subtract),
        VK_MULTIPLY => Some(Key::Multiply),
        VK_DIVIDE => Some(Key::Divide),
        VK_PAUSE => Some(Key::Pause),
        VK_F1 => Some(Key::F1),
        VK_F2 => Some(Key::F2),
        VK_F3 => Some(Key::F3),
        VK_F4 => Some(Key::F4),
        VK_F5 => Some(Key::F5),
        VK_F6 => Some(Key::F6),
        VK_F7 => Some(Key::F7),
        VK_F8 => Some(Key::F8),
        VK_F9 => Some(Key::F9),
        VK_F10 => Some(Key::F10),
        VK_F11 => Some(Key::F11),
        VK_F12 => Some(Key::F12),
        VK_F13 => Some(Key::F13),
        VK_F14 => Some(Key::F14),
        VK_F15 => Some(Key::F15),
        VK_LEFT => Some(Key::Left),
        VK_RIGHT => Some(Key::Right),
        VK_UP => Some(Key::Up),
        VK_DOWN => Some(Key::Down),
        VK_CAPITAL => Some(Key::CapsLock),
        VK_NUMPAD0 => Some(Key::Numpad0),
        VK_NUMPAD1 => Some(Key::Numpad1),
        VK_NUMPAD2 => Some(Key::Numpad2),
        VK_NUMPAD3 => Some(Key::Numpad3),
        VK_NUMPAD4 => Some(Key::Numpad4),
        VK_NUMPAD5 => Some(Key::Numpad5),
        VK_NUMPAD6 => Some(Key::Numpad6),
        VK_NUMPAD7 => Some(Key::Numpad7),
        VK_NUMPAD8 => Some(Key::Numpad8),
        VK_NUMPAD9 => Some(Key::Numpad9),
        0x30 => Some(Key::Num0),
        0x31 => Some(Key::Num1),
        0x32 => Some(Key::Num2),
        0x33 => Some(Key::Num3),
        0x34 => Some(Key::Num4),
        0x35 => Some(Key::Num5),
        0x36 => Some(Key::Num6),
        0x37 => Some(Key::Num7),
        0x38 => Some(Key::Num8),
        0x39 => Some(Key::Num9),
        0x41 => Some(Key::A),
        0x42 => Some(Key::B),
        0x43 => Some(Key::C),
        0x44 => Some(Key::D),
        0x45 => Some(Key::E),
        0x46 => Some(Key::F),
        0x47 => Some(Key::G),
        0x48 => Some(Key::H),
        0x49 => Some(Key::I),
        0x4A => Some(Key::J),
        0x4B => Some(Key::K),
        0x4C => Some(Key::L),
        0x4D => Some(Key::M),
        0x4E => Some(Key::N),
        0x4F => Some(Key::O),
        0x50 => Some(Key::P),
        0x51 => Some(Key::Q),
        0x52 => Some(Key::R),
        0x53 => Some(Key::S),
        0x54 => Some(Key::T),
        0x55 => Some(Key::U),
        0x56 => Some(Key::V),
        0x57 => Some(Key::W),
        0x58 => Some(Key::X),
        0x59 => Some(Key::Y),
        0x5A => Some(Key::Z),
        _ => {
            println!("An unknown key was pressed. Virtual key code: '{}'.", virtual_key);
            None
        }
    }
}

unsafe fn handle_keyboard_input(lparam: LPARAM, events: &mut Vec<Event>) {
    let mut input = MaybeUninit::<RAWINPUT>::uninit();
    let mut size = size_of::<RAWINPUT>() as u32;
    let success = GetRawInputData(
        lparam as HRAWINPUT,
        RID_INPUT,
        input.as_mut_ptr() as *mut c_void,
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
            _ => translate_key(virtual_key),
        };

        if let Some(key) = key {
            if released {
                events.push(Event::KeyReleased(key, scan_code));
            } else {
                events.push(Event::KeyPressed(key, scan_code));
            }
        }
    }
}
