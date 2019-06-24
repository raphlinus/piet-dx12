extern crate winapi;
extern crate user32;

use std::os::windows::ffi::OsStrExt;

fn win32_string(value: &str) -> Vec<u16> {
    std::ffi::OsStr::new( value).encode_wide().chain(std::iter::once(0)).collect()
}

struct Window {
    name: usize,
    class: winapi::um::winuser::WNDCLASSW,
}

impl Window {
    unsafe fn new() -> Window {
        let name = win32_string("piet-dx12");
        let title = win32_string("piet-dx12");

        use winapi::um::{winuser, libloaderapi};

        let hinstance = winapi::um::libloaderapi::GetModuleHandleW(std::ptr::null());
        
        let class = winuser::WNDCLASSW {
            style: winuser::CS_HREDRAW | winuser::CS_VREDRAW | winuser::CS_OWNDC,
            lpfnWndProc: Some(winuser::DefWindowProcW()),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: winuser::LoadIconW(std::ptr::null_mut(), winuser::IDI_WINLOGO),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name_wstr.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };

        unsafe {
            winuser::RegisterClassExW(&class);
            winuser::IsGUIThread(1);
        }

        let handle = winuser::CreateWindowExW(
            0,
            name.as_ptr(),
            title.as_ptr(),
            winuser::WS_OVERLAPPEDWINDOW | winuser::WS_VISIBLE,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            class.hinstance,
            std::ptr::null_mut() );
    }
}

