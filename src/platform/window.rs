use super::{
    get_last_error,
    input::{Key, MouseButton},
};
use core::{mem::size_of, ptr};
use std::{ffi::CString, mem::MaybeUninit, ptr::null_mut};
use winapi::{
    ctypes::c_void,
    shared::{
        minwindef::{HIWORD, LOWORD, LPARAM, LRESULT, UINT, WPARAM},
        windef::HWND,
    },
    um::{libloaderapi::GetModuleHandleA, winuser::*},
};

struct EventLoop<'a> {
    cursor_inside: bool,
    has_focus: bool,
    should_exit: bool,
    event_callback: &'a mut dyn FnMut(&Event, &mut bool),
}

impl<'a> EventLoop<'a> {
    fn handle_event(&mut self, event: &Event) {
        (self.event_callback)(event, &mut self.should_exit);
    }
}

#[derive(Debug)]
pub enum Event {
    Initialized,
    CloseRequested,
    Exiting,
    UpdateAndRender,
    KeyPressed(Key, u32),
    KeyReleased(Key, u32),
    CharacterEntered(char),
    Resized(u32, u32),
    Focused(bool),
    MouseEntered,
    MouseLeft,
    MouseMoved(u32, u32),
    MousePressed(MouseButton),
    MouseReleased(MouseButton),
    MouseWheel(i32),
}

pub fn execute_in_window(mut event_callback: impl FnMut(&Event, &mut bool)) {
    unsafe {
        let mut event_loop = EventLoop {
            cursor_inside: false,
            has_focus: false,
            should_exit: false,
            event_callback: &mut event_callback,
        };

        let hwnd = open_window(&mut event_loop);

        event_loop.handle_event(&Event::Initialized);

        while !event_loop.should_exit {
            // Make sure we don't emit the Focused event too often, i.e., only once per change.
            let has_focus = event_loop.has_focus;
            process_events(hwnd);
            if has_focus != event_loop.has_focus {
                event_loop.handle_event(&Event::Focused(event_loop.has_focus));
            }

            event_loop.handle_event(&Event::UpdateAndRender);
        }

        event_loop.handle_event(&Event::Exiting);
    }
}

unsafe fn open_window(event_loop: &mut EventLoop) -> HWND {
    let title = CString::new("Orbs").unwrap();

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
    }

    let hwnd = CreateWindowExA(
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
        event_loop as *mut _ as *mut c_void,
    );

    if hwnd.is_null() {
        panic!("Failed to create window. {}", get_last_error());
    }

    if !cfg!(debug_assertions) {
        toggle_fullscreen(hwnd);
    }

    hwnd
}

pub fn toggle_fullscreen(hwnd: HWND) {
    unsafe {
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
}

unsafe fn process_events(hwnd: HWND) {
    let mut msg: MSG = Default::default();
    while PeekMessageA(&mut msg, hwnd, 0, 0, PM_REMOVE) != 0 {
        TranslateMessage(&msg);
        DispatchMessageA(&msg);
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_CREATE {
        let event_loop_ptr = (*(lparam as *const CREATESTRUCTA)).lpCreateParams;
        SetWindowLongPtrA(hwnd, GWLP_USERDATA, event_loop_ptr as isize);
    }

    let event_loop_ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut c_void;
    if event_loop_ptr.is_null() {
        return DefWindowProcA(hwnd, msg, wparam, lparam);
    }

    let event_loop: &mut EventLoop = &mut *(event_loop_ptr as *mut EventLoop);

    match msg {
        WM_INPUT => handle_keyboard_input(lparam, event_loop),
        WM_SYSCOMMAND => {
            if wparam == SC_KEYMENU {
                return 0;
            }
        }
        WM_CLOSE => {
            event_loop.handle_event(&Event::CloseRequested);
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
        WM_ACTIVATE => event_loop.has_focus = LOWORD(wparam as u32) != WA_INACTIVE,
        WM_NCACTIVATE | WM_ACTIVATEAPP => event_loop.has_focus = wparam != 0,
        WM_MOUSEMOVE => {
            // If the cursor is entering the window, raise the mouse entered event and tell Windows to inform
            // us when the cursor leaves the window.
            if !event_loop.cursor_inside {
                let mut mouse_event = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    hwndTrack: hwnd,
                    dwFlags: TME_LEAVE,
                    ..Default::default()
                };
                TrackMouseEvent(&mut mouse_event);

                event_loop.cursor_inside = true;
                event_loop.handle_event(&Event::MouseEntered);
            } else {
                event_loop.handle_event(&Event::MouseMoved(LOWORD(lparam as u32) as u32, HIWORD(lparam as u32) as u32));
            }
        }
        WM_MOUSELEAVE => {
            event_loop.cursor_inside = false;
            event_loop.handle_event(&Event::MouseLeft);
        }
        WM_LBUTTONDOWN => event_loop.handle_event(&Event::MousePressed(MouseButton::Left)),
        WM_LBUTTONUP => event_loop.handle_event(&Event::MouseReleased(MouseButton::Left)),
        WM_RBUTTONDOWN => event_loop.handle_event(&Event::MousePressed(MouseButton::Right)),
        WM_RBUTTONUP => event_loop.handle_event(&Event::MouseReleased(MouseButton::Right)),
        WM_MBUTTONDOWN => event_loop.handle_event(&Event::MousePressed(MouseButton::Middle)),
        WM_MBUTTONUP => event_loop.handle_event(&Event::MouseReleased(MouseButton::Middle)),
        WM_XBUTTONDOWN => event_loop.handle_event(&Event::MousePressed(if HIWORD(wparam as u32) == XBUTTON1 {
            MouseButton::XButton1
        } else {
            MouseButton::XButton2
        })),
        WM_XBUTTONUP => event_loop.handle_event(&Event::MouseReleased(if HIWORD(wparam as u32) == XBUTTON1 {
            MouseButton::XButton1
        } else {
            MouseButton::XButton2
        })),
        WM_MOUSEWHEEL => event_loop.handle_event(&Event::MouseWheel((GET_WHEEL_DELTA_WPARAM(wparam) / WHEEL_DELTA) as i32)),
        WM_CHAR => {
            if let Some(character) = char::from_u32(wparam as u32) {
                event_loop.handle_event(&Event::CharacterEntered(character))
            }
        }
        WM_SIZE => event_loop.handle_event(&Event::Resized(LOWORD(lparam as u32) as u32, HIWORD(lparam as u32) as u32)),
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

unsafe fn handle_keyboard_input(lparam: LPARAM, event_loop: &mut EventLoop) {
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
            VK_CLEAR => {
                if !is_e0 {
                    Some(Key::Numpad5)
                } else {
                    None
                }
            }
            _ => translate_key(virtual_key),
        };

        if let Some(key) = key {
            if released {
                event_loop.handle_event(&Event::KeyReleased(key, scan_code));
            } else {
                event_loop.handle_event(&Event::KeyPressed(key, scan_code));
            }
        }
    }
}
