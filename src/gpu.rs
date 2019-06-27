extern crate winapi;

use winapi::Interface;
use crate::window;
use crate::d3d12;

const FRAME_COUNT: u32 = 2;

struct GpuState {
    // pipeline stuff
    viewport: winapi::um::d3d12::D3D12_VIEWPORT,
    scissor_rect: winapi::um::d3d12::D3D12_RECT,
    swapchain: d3d12::SwapChain3,
    device: d3d12::Device,
    render_targets: Vec<d3d12::Resource>,
    command_allocator: d3d12::CommandAllocator,
    command_queue: d3d12::CommandQueue,
    roots_signature: d3d12::RootSignature,
    rtv_heap: d3d12::DescriptorHeap,
    pipeline_state: d3d12::PipelineState,
    command_list: d3d12::GraphicsCommandList,
    rtv_descriptor_size: usize,

    // "app" stuff?
    vertex_buffer: d3d12::resource::Resource,
    vertex_buffer_view: Vec<d3d12::WeakPtr<winapi::um::d3d12::D3D12_VERTEX_BUFFER_VIEW>>,

    // synchronizers
    frame_index: usize,
    fence_event: d3d12::sync::Event,
    fence: d3d12::sync::Fence,
    fence_value: u64,
}

impl GpuState {
    unsafe fn new(width: u32, height: u32, name: &str, wnd: window::Window) {
        GpuState::load_pipeline(width, height, wnd);
        GpuState::load_assets();
    }

    fn update() {

    }

    fn render() {

    }

    fn destroy() {

    }

    unsafe fn load_pipeline(width: u32, height: u32, wnd: window::Window) {
        #[cfg(debug_assertions)]
        // Enable debug layer
        {

            let mut debug_controller: *mut winapi::um::d3d12sdklayers::ID3D12Debug = std::ptr::null_mut();
            let hr = unsafe {
                winapi::um::d3d12::D3D12GetDebugInterface(
                    &winapi::um::d3d12sdklayers::ID3D12Debug::uuidof(),
                    &mut debug_controller as *mut *mut _ as *mut *mut _,
                )
            };

            if winapi::shared::winerror::SUCCEEDED(hr) {
                unsafe {
                    (*debug_controller).EnableDebugLayer();
                    (*debug_controller).Release();
                }
            }
        }

        // create factory4
        let mut factory4 = d3d12::error_if_failed(d3d12::dxgi::Factory4::create(winapi::shared::dxgi1_3::DXGI_CREATE_FACTORY_DEBUG)).expect("could not create factory4");

        // create device
        let device = match GpuState::create_device(&factory4) {
            Ok(device) => {
                device
            },
            Err(hr) => {
                if hr[0] == winapi::shared::winerror::DXGI_ERROR_NOT_FOUND {
                    panic!("could not find adapter");
                } else {
                    panic!("could not find dx12 capable device");
                }
            }
        };

        let command_queue = d3d12::error_if_failed(device.create_command_queue(d3d12::command_list::CmdListType::Direct, d3d12::queue::Priority::Normal, d3d12::queue::CommandQueueFlags::empty(), 0)).expect("could not create command queue");

        // create swapchain
        let swapchain_desc = winapi::um::dxgi1_2::DXGI_SWAP_CHAIN_DESC1 {
            AlphaMode: desc.alpha_mode as _,
            BufferCount: FRAME_COUNT,
            Width: width,
            Height: height,
            Format: winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            Flags: 0,
            BufferUsage: winapi::shared::dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Scaling: winapi::shared::dxgitype::DXGI_MODE_SCALING_CENTERED,
            Stereo: winapi::shared::ntdef::FALSE,
            SwapEffect: winapi::shared::dxgi::DXGI_SWAP_EFFECT_DISCARD,
        };

        let swap_chain1 = factory4.as_factory2().create_swapchain_for_hwnd(command_queue.clone(), wnd.hwnd.clone(), swapchain_desc);
        let swap_chain3 = d3d12::error_if_failed(swap_chain1.cast::<winapi::um::dxgi1_4::IDXGISwapChain3>());
        // disable full screen transitions
        // winapi does not have DXGI_MWA_NO_ALT_ENTER?
        factory4.MakeWindowAssociation(wnd.hwnd, 1);

        let frame_index = swap_chain3.get_current_back_buffer_index();

        // create descriptor heap
        let descriptor_heap = device.create_descriptor_heap(FRAME_COUNT, d3d12::descriptor::HeapType::Rtv, d3d12::descriptor::HeapFlags::empty());

        let rtv_descriptor_size = device.get_descriptor_increment_size(d3d12::descriptor::HeapType::Rtv);

        // create frame resources
        let cpu_descriptor = descriptor_heap.start_cpu_descriptor();
        // create render target and render target view for each frame
        let mut render_targets: Vec<d3d12::resource::Resource> = Vec::new();
        for ix in 0..FRAME_COUNT {
            let resource = swap_chain3.get_buffer().unwrap();
            device.create_render_target_view(resource, &render_target_view_desc, cpu_descriptor);
            render_targets.push(resource);
        }
    }

    fn load_assets() {

    }

    fn populate_command_list() {

    }

    fn wait_for_previous_frame() {

    }
}
