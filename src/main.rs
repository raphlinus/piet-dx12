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

        let compute_shader_code =
"#define BLOCK_SIZE 16

".as_bytes();
        let shader_code =
"struct PSInput
{
    float4 position : SV_POSITION;
};

PSInput VSMain(float4 position : POSITION)
{
    PSInput result;

    result.position = position;

    return result;
}

float4 PSMain(PSInput input) : SV_TARGET
{
    float4 color = {1.0f, 0.0f, 0.0f, 1.0f};
    return color;
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
                println!("rendering...");
                gpu_state.render();
                println!("looping...");
            }
        }
    }
}
