extern crate winapi;
extern crate d3d12;

use crate::window;

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
    fn new(width: u32, height: u32, name: &str) {
        GpuState::load_pipeline(width, height);
        GpuState::load_assets();
    }

    fn update() {

    }

    fn render() {

    }

    fn destroy() {

    }

    fn create_device(factory: d3d12::dxgi::Factory4) -> d3d12::Device {
        let mut id = 0;
        loop {
            let (adapter, hr) = factory.enumerate_adapters(id);
            if hr == winapi::shared::winerror::DXGI_ERROR_NOT_FOUND {
                panic!("unable to find adapter")
            }
            id += 1;

            let (device, hr) = d3d12::Device::create(adapter, d3d12::FeatureLevel::L12_0);
            if !winapi::shared::winerror::SUCCEEDED(hr) {
                continue;
            }
            unsafe { adapter.destroy() };
            return device;
        }
    }

    fn load_pipeline(width: u32, height: u32, wnd: crate::window::Window) {
        #[cfg(debug_assertions)]
        // Enable debug layer
        {

            let mut debug_controller: *mut winapi::um::d3d12sdklayers::ID3D12Debug = std::ptr::null_mut();
            let hr = unsafe {
                winapi::um::d3d12::D3D12GetDebugInterface(
                    &d3d12sdklayers::ID3D12Debug::uuidof(),
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
        let mut factory4 = d3d12::dxgi::Factory4::create(d3d12::dxgi::FactoryCreationFlags::DEBUG);

        // create device
        let device = create_device(factory4);

        let command_queue = device.create_command_queue(d3d12::command_list::CmdListType::Direct, d3d12::queue::Priority::Normal, d3d12::queue::CommandQueueFlags::empty(), 0);

        // create swapchain
        let swapchain_desc = d3d12::dxgi::SwapchainDesc {
            width,
            height,
            format: d3d12::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
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
