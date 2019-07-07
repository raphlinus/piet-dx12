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
"#define BLOCK_SIZE 256

RWTexture2D<float4> canvas;
[numthreads(16, 16, 1)]
void CSMain(uint3 DTid : SV_DispatchThreadID) {
    float4 color = {0.0f, 0.0f, 1.0f, 1.0f};
    canvas[DTid.xy] = color;
}

float4 VSMain(float4 position: POSITION) : SV_Position
{
    return position;
}

float4 PSMain(float4 position: SV_Position) : SV_TARGET
{
    uint2 pos = position.xy;
    //float4 color = {frac(position.y), frac(position.y), frac(position.y), 1.0f};
    //return color;
    return canvas[pos.xy];
}
"
        .as_bytes();

        let mut gpu_state = gpu::GpuState::new(
            &wnd,
            shader_code,
            String::from("CSMain"),
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
