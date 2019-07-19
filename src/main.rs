//#[macro_use]
//extern crate log;
//extern crate env_logger;

// #![windows_subsystem = "windows"] (I think I want the console)
pub mod dx12;
pub mod error;
pub mod gpu;
pub mod scene;
pub mod window;

use std::os::windows::ffi::OsStrExt;

pub fn win32_string(value: &str) -> Vec<u16> {
    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}


fn main() {
    unsafe {
        println!("creating window...");
        let mut wnd =
            window::Window::new(win32_string("test"), win32_string("test"));

        let mut gpu_state = gpu::GpuState::new(
            &wnd,
            String::from("build_per_tile_command_list"),
            String::from("paint_objects"),
            String::from("VSMain"),
            String::from("PSMain"),
            16,
            32,
            1,
            1,
            1,
        );

        loop {
            if window::quit(&mut wnd) {
                gpu_state.destroy();
                break;
            } else {
                gpu_state.render();
            }
        }
    }
}
