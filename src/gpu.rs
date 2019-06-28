extern crate winapi;

use crate::d3d12;
use crate::window;
use winapi::Interface;

const FRAME_COUNT: u32 = 2;

struct GpuState {
    // pipeline stuff
    swapchain: d3d12::SwapChain3,
    device: d3d12::Device,
    render_targets: Vec<d3d12::Resource>,
    command_allocator: d3d12::CommandAllocator,
    command_queue: d3d12::CommandQueue,
    root_signature: d3d12::RootSignature,
    rtv_heap: d3d12::DescriptorHeap,
    pipeline_state: d3d12::PipelineState,
    command_list: d3d12::CommandList,
    rtv_descriptor_size: u32,

    // synchronizers
    frame_index: usize,
    fence_event: d3d12::Event,
    fence: d3d12::Fence,
    fence_value: u64,
}

impl GpuState {
    unsafe fn new(width: u32, height: u32, name: &str, wnd: window::Window, shader_code: &[u8], entry: String) -> GpuState {
        let (
            swapchain,
            device,
            render_targets,
            command_allocator,
            command_queue,
            rtv_heap,
            rtv_descriptor_size,
            fence,
        ) = GpuState::create_pipeline_dependencies(width, height, wnd);

        let (root_signature, pipeline_state, command_list) = GpuState::create_pipeline_state(&device, shader_code, entry, command_allocator.clone());

        let fence_event = d3d12::Event::create(false, false);

        GpuState {
            swapchain,
            device,
            render_targets,
            command_allocator,
            command_queue,
            root_signature,
            rtv_heap,
            pipeline_state,
            command_list,
            rtv_descriptor_size,
            frame_index: 0,
            fence_event,
            fence,
            fence_value: 1,
        }
    }

    fn populate_command_list(&mut self) {
    }

    fn render(&mut self) {
        self.populate_command_list();

        // wait for rendering to complete
        self.wait_for_previous_frame();
    }

    fn wait_for_previous_frame(&mut self) {

    }


    fn destroy() {}

    unsafe fn create_pipeline_dependencies(
        width: u32,
        height: u32,
        wnd: window::Window,
    ) -> (
        d3d12::SwapChain3,
        d3d12::Device,
        Vec<d3d12::Resource>,
        d3d12::CommandAllocator,
        d3d12::CommandQueue,
        d3d12::DescriptorHeap,
        u32,
        d3d12::Fence,
    ) {
        #[cfg(debug_assertions)]
        // Enable debug layer
        {
            let mut debug_controller: *mut winapi::um::d3d12sdklayers::ID3D12Debug =
                std::ptr::null_mut();
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
        let mut factory4 = d3d12::error_if_failed_else_value(d3d12::Factory4::create(
            winapi::shared::dxgi1_3::DXGI_CREATE_FACTORY_DEBUG,
        ))
        .expect("could not create factory4");

        // create device
        let device = match d3d12::Device::create_device(&factory4) {
            Ok(device) => device,
            Err(hr) => {
                if hr[0] == winapi::shared::winerror::DXGI_ERROR_NOT_FOUND {
                    panic!("could not find adapter");
                } else {
                    panic!("could not find dx12 capable device");
                }
            }
        };

        let list_type = winapi::um::d3d12::D3D12_COMMAND_LIST_TYPE_COMPUTE;
        let command_queue = d3d12::error_if_failed_else_value(device.create_command_queue(
            list_type,
            0,
            winapi::um::d3d12::D3D12_COMMAND_QUEUE_FLAG_NONE,
            0,
        ))
        .expect("could not create command queue");

        // create swapchain
        let swapchain_desc = winapi::shared::dxgi1_2::DXGI_SWAP_CHAIN_DESC1 {
            AlphaMode: winapi::shared::dxgi1_2::DXGI_ALPHA_MODE_UNSPECIFIED,
            BufferCount: FRAME_COUNT,
            Width: width,
            Height: height,
            Format: winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            Flags: 0,
            BufferUsage: winapi::shared::dxgitype::DXGI_USAGE_UNORDERED_ACCESS,
            SampleDesc: winapi::shared::dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Scaling: winapi::shared::dxgitype::DXGI_MODE_SCALING_CENTERED,
            Stereo: winapi::shared::minwindef::FALSE,
            SwapEffect: winapi::shared::dxgi::DXGI_SWAP_EFFECT_DISCARD,
        };

        let swap_chain1 = d3d12::error_if_failed_else_value(factory4.as_factory2().create_swapchain_for_hwnd(
            command_queue.clone(),
            wnd.hwnd.clone(),
            swapchain_desc,
        ))
        .expect("could not create swap chain 1");
        let swap_chain3 = swap_chain1.cast_into_swap_chain3();
        // disable full screen transitions
        // winapi does not have DXGI_MWA_NO_ALT_ENTER?
        factory4.0.MakeWindowAssociation(wnd.hwnd, 1);

        let frame_index = swap_chain3.get_current_back_buffer_index();

        // create descriptor heap
        let descriptor_heap_desc = winapi::um::d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
            Type: winapi::um::d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: FRAME_COUNT,
            Flags: winapi::um::d3d12::D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            NodeMask: 0,
        };
        let rtv_heap = d3d12::error_if_failed_else_value(device.create_descriptor_heap(&descriptor_heap_desc))
            .expect("could not create descriptor heap");
        let rtv_descriptor_size =
            device.get_descriptor_increment_size(winapi::um::d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV);

        // create frame resources
        let mut cpu_descriptor = rtv_heap.start_cpu_descriptor();
        // create render target and render target view for each frame
        let mut render_targets: Vec<d3d12::Resource> = Vec::new();
        for ix in 0..FRAME_COUNT {
            let resource = d3d12::error_if_failed_else_value(swap_chain3.get_buffer(ix))
                .expect("could not create render target resource");
            device.create_render_target_view(resource.clone(), std::ptr::null(), cpu_descriptor);
            // TODO: is this correct?
            cpu_descriptor.ptr += rtv_descriptor_size as usize;
            render_targets.push(resource.clone());
        }

        let command_allocator = d3d12::error_if_failed_else_value(device.create_command_allocator(list_type))
            .expect("could not create command allocator");

        let fence = d3d12::error_if_failed_else_value(device.create_fence(0)).expect("could not create fence");

        (
            swap_chain3,
            device,
            render_targets,
            command_allocator,
            command_queue,
            rtv_heap,
            rtv_descriptor_size,
            fence
        )
    }

    unsafe fn create_pipeline_state(device: &d3d12::Device, shader_code: &[u8], entry: String, command_allocator: d3d12::CommandAllocator) -> (d3d12::RootSignature, d3d12::PipelineState, d3d12::CommandList) {
        // create empty root signature
        let root_signature_desc = winapi::um::d3d12::D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: 0,
            pParameters: std::ptr::null(),
            NumStaticSamplers: 0,
            pStaticSamplers: std::ptr::null(),
            Flags: winapi::um::d3d12::D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
        };
        let (blob, error_blob) = d3d12::error_if_failed_else_value(d3d12::RootSignature::serialize(&root_signature_desc, winapi::um::d3d12::D3D_ROOT_SIGNATURE_VERSION_1)).expect("could not serialize root signature");
        let root_signature = d3d12::error_if_failed_else_value(device.create_root_signature(0, blob)).expect("could not create root signature");

        let mut flags: winapi::shared::minwindef::DWORD = 0;

        #[cfg(debug_assertions)]
        {
            flags = winapi::um::d3dcompiler::D3DCOMPILE_DEBUG | winapi::um::d3dcompiler::D3DCOMPILE_SKIP_OPTIMIZATION;
        }

        // load shader
        let (compute_shader_blob, compile_error_blob) = d3d12::error_if_failed_else_value(d3d12::ShaderByteCode::compile(shader_code, String::from("cs_5_0"), entry, flags)).expect("could not compile compute shader");
        let compute_shader_bytecode = d3d12::ShaderByteCode::from_blob(compute_shader_blob);

        // create compute pipeline state
        let compute_ps_desc = winapi::um::d3d12::D3D12_COMPUTE_PIPELINE_STATE_DESC {
            pRootSignature: root_signature.0.as_raw(),
            CS: compute_shader_bytecode.0,
            NodeMask: 0,
            CachedPSO: winapi::um::d3d12::D3D12_CACHED_PIPELINE_STATE {
                pCachedBlob: std::ptr::null(),
                CachedBlobSizeInBytes: 0,
            },
            Flags: winapi::um::d3d12::D3D12_PIPELINE_STATE_FLAG_NONE,
        };
        let compute_pipeline_state = d3d12::error_if_failed_else_value(device.create_compute_pipeline_state(&compute_ps_desc)).expect("could not create compute pipeline state");

        // create command list
        let command_list = d3d12::error_if_failed_else_value(device.create_command_list(winapi::um::d3d12::D3D12_COMMAND_LIST_TYPE_COMPUTE, command_allocator.clone(), compute_pipeline_state.clone(), 0)).expect("could not create compute pipeline list");
        //command_list.close();

        (root_signature, compute_pipeline_state, command_list)
    }
}
