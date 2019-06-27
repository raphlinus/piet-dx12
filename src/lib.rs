// #![windows_subsystem = "windows"] (I think I want the console)

#[macro_use]
extern crate bitflags;

pub mod window;
pub mod gpu;
pub mod d3d12;

//fn main() {
//    unsafe {
//        println!("creating window...");
//        let mut wnd = window::Window::new(window::win32_string("test"), window::win32_string("test"));
//
//        println!("beginning loop...");
//        loop {
//            if window::quit(&mut wnd) {
//                println!("quitting...");
//                break;
//            }
//            println!("    looping...");
//        }
//    }
//}
