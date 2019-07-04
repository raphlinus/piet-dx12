//#[macro_use]
//extern crate log;
//extern crate env_logger;

// #![windows_subsystem = "windows"] (I think I want the console)
pub mod dx12;
pub mod error;
pub mod gpu;
pub mod window;

fn main() {
    unsafe {
        //env_logger::init();

        println!("creating window...");
        let mut wnd =
            window::Window::new(window::win32_string("test"), window::win32_string("test"));

        let shader_code =
"struct PSInput
{
    float4 position : SV_POSITION;
    float4 color : COLOR;
};

PSInput VSMain(float4 position : POSITION, float4 color : COLOR)
{
    PSInput result;

    result.position = position;
    result.color = color;

    return result;
}

float4 PSMain(PSInput input) : SV_TARGET
{
    return input.color;
}
".as_bytes();

        let mut gpu_state = gpu::GpuState::new(
            &wnd,
            shader_code,
            String::from("VSMain"),
            String::from("PSMain"),
        );

        println!("beginning loop...");
        loop {
            if window::quit(&mut wnd) {
                println!("quitting...");
                gpu_state.destroy();
                break;
            } else {
                gpu_state.render();
                println!("    looping...");
            }
        }
    }
}
