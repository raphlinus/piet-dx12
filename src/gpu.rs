extern crate winapi;
extern crate d3d12;

use winapi::Interface;
use crate::window;
use crate::error_utils::error_if_failed;

const FRAME_COUNT: u32 = 2;

struct GpuState {
    // pipeline stuff
    viewport: winapi::um::d3d12::D3D12_VIEWPORT,
    scissor_rect: winapi::um::d3d12::D3D12_RECT,
    swapchain: d3d12::dxgi::SwapChain3,
    device: d3d12::device::Device,
    render_targets: Vec<d3d12::resource::Resource>,
    command_allocator: d3d12::command_allocator::CommandAllocator,
    command_queue: d3d12::queue::CommandQueue,
    roots_signature: d3d12::descriptor::RootSignature,
    rtv_heap: d3d12::descriptor::DescriptorHeap,
    pipeline_state: d3d12::pso::PipelineState,
    command_list: d3d12::command_list::GraphicsCommandList,
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
    fn new(width: u32, height: u32, name: &str, wnd: window::Window) {
        GpuState::load_pipeline(width, height, wnd);
        GpuState::load_assets();
    }

    fn update() {

    }

    fn render() {

    }

    fn destroy() {

    }

    unsafe fn create_device(factory4: &d3d12::dxgi::Factory4) -> Result<d3d12::Device, Vec<winapi::shared::winerror::HRESULT>> {
        let mut id = 0;
        let mut errors: Vec<winapi::shared::winerror::HRESULT> = Vec::new();

        loop {
            let adapter = match error_if_failed(factory4.enumerate_adapters(id)) {
                Ok(a) => {
                    a
                },
                Err(hr) => {
                    errors.push(hr);
                    return Err(errors);
                }
            };
            
            id += 1;

            match error_if_failed(d3d12::Device::create(adapter, d3d12::FeatureLevel::L12_0)) {
                Ok(device) => {
                    adapter.destroy();
                    return Ok(device);
                },
                Err(hr) => {
                    errors.push(hr);
                    continue;
                }
            }
        }

        Err(errors)
    }

    fn load_pipeline(width: u32, height: u32, wnd: window::Window) {
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
        let mut factory4 = error_if_failed(d3d12::dxgi::Factory4::create(d3d12::dxgi::FactoryCreationFlags::DEBUG)).expect("could not create factory4");

        // create device
        let device = match GpuState::create_device(&factory4) {
            Ok(device) => {
                device
            },
            Err(hr) => {
                if hr == winapi::shared::winerror::DXGI_ERROR_NOT_FOUND {
                    panic!("could not find adapter");
                } else {
                    panic!("could not find dx12 capable device");
                }
            }
        };

        let command_queue = device.create_command_queue(d3d12::command_list::CmdListType::Direct, d3d12::queue::Priority::Normal, d3d12::queue::CommandQueueFlags::empty(), 0);

        // create swapchain
        let swapchain_desc = d3d12::dxgi::SwapchainDesc {
            width,
            height,
            format: winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            stereo: false,
            sample: d3d12::SampleDesc{
                count: 1,
                quality: 0,
            },
            buffer_usage: winapi::shared::dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            buffer_count: FRAME_COUNT,
            scaling: d3d12::dxgi::Scaling::Identity,
            swap_effect: d3d12::dxgi::SwapEffect::FlipDiscard,
            alpha_mode: d3d12::dxgi::AlphaMode::Unspecified,
            flags: 0,
        };

        let factory2 = factory4.as_factory2();
        let swapchain = factory2.create_swapchain_for_hwnd(command_queue, wnd.hwnd, &swapchain_desc);

        // disable full screen transitions
        // winapi does not have DXGI_MWA_NO_ALT_ENTER?
        factory4.MakeWindowAssociation(wnd.hwnd, 1);

        let frame_index = swapchain.get_current_back_buffer_index();

        // create descriptor heap
        let descriptor_heap = device.create_descriptor_heap(FRAME_COUNT, d3d12::descriptor::HeapType::Rtv, d3d12::descriptor::HeapFlags::empty());

        let rtv_descriptor_size = device.get_descriptor_increment_size(d3d12::descriptor::HeapType::Rtv);

        // create frame resources
        let cpu_descriptor = descriptor_heap.start_cpu_descriptor();
        // create render target and render target view for each frame
        let mut render_targets: Vec<d3d12::resource::Resource> = Vec::new();
        for ix in 0..FRAME_COUNT {
            let resource = swapchain.as_swapchain0().get_buffer().unwrap();
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
