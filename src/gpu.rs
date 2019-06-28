extern crate winapi;

use crate::dx12;
use crate::window;
use winapi::Interface;
use winapi::um::d3d12;
use winapi::shared::{winerror, dxgitype, dxgi, dxgi1_2, minwindef};

const FRAME_COUNT: u32 = 2;

struct GpuState {
    // pipeline stuff
    swapchain: dx12::SwapChain3,
    device: dx12::Device,
    render_targets: Vec<dx12::Resource>,
    command_allocator: dx12::CommandAllocator,
    command_queue: dx12::CommandQueue,
    root_signature: dx12::RootSignature,
    rtv_heap: dx12::DescriptorHeap,
    pipeline_state: dx12::PipelineState,
    command_list: dx12::GraphicsCommandList,
    rtv_descriptor_size: u32,

    // synchronizers
    frame_index: usize,
    fence_event: dx12::Event,
    fence: dx12::Fence,
    fence_value: u64,
}

impl GpuState {
    unsafe fn new(
        width: u32,
        height: u32,
        name: &str,
        wnd: window::Window,
        shader_code: &[u8],
        entry: String,
    ) -> GpuState {
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

        let (root_signature, pipeline_state, command_list) =
            GpuState::create_pipeline_state(&device, shader_code, entry, command_allocator.clone());

        let fence_event = dx12::Event::create(false, false);

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

    unsafe fn populate_command_list(&mut self) {
        self.command_allocator.reset();
        self.command_list.reset(self.command_allocator.clone(), self.pipeline_state.clone());

        self.command_list.set_compute_root_signature(self.root_signature.clone());
        let transition_barrier = d3d12::D3D12_RESOURCE_TRANSITION_BARRIER {
            pResource: self.render_targets[self.frame_index].clone().0.Get(),
            Subresource: d3d12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
            StateBefore: d3d12::D3D12_RESOURCE_STATE_PRESENT,
            StateAfter: d3d12::D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
        };
        self.command_list.set_resource_barrier(1, &transition_barrier as *const _);

    }

    unsafe fn render(&mut self) {
        self.populate_command_list();

        // wait for rendering to complete
        self.wait_for_previous_frame();
    }

    fn wait_for_previous_frame(&mut self) {}

    fn destroy() {}

    unsafe fn create_pipeline_dependencies(
        width: u32,
        height: u32,
        wnd: window::Window,
    ) -> (
        dx12::SwapChain3,
        dx12::Device,
        Vec<dx12::Resource>,
        dx12::CommandAllocator,
        dx12::CommandQueue,
        dx12::DescriptorHeap,
        u32,
        dx12::Fence,
    ) {
        #[cfg(debug_assertions)]
        // Enable debug layer
        {
            let mut debug_controller: *mut winapi::um::d3d12sdklayers::ID3D12Debug =
                std::ptr::null_mut();
            let hr = unsafe {
                d3d12::D3D12GetDebugInterface(
                    &winapi::um::d3d12sdklayers::ID3D12Debug::uuidof(),
                    &mut debug_controller as *mut *mut _ as *mut *mut _,
                )
            };

            if winerror::SUCCEEDED(hr) {
                unsafe {
                    (*debug_controller).EnableDebugLayer();
                    (*debug_controller).Release();
                }
            }
        }

        // create factory4
        let mut factory4 = dx12::error_if_failed_else_value(dx12::Factory4::create(
            winapi::shared::dxgi1_3::DXGI_CREATE_FACTORY_DEBUG,
        ))
        .expect("could not create factory4");

        // create device
        let device = match dx12::Device::create_device(&factory4) {
            Ok(device) => device,
            Err(hr) => {
                if hr[0] == winerror::DXGI_ERROR_NOT_FOUND {
                    panic!("could not find adapter");
                } else {
                    panic!("could not find dx12 capable device");
                }
            }
        };

        let list_type = d3d12::D3D12_COMMAND_LIST_TYPE_COMPUTE;
        let command_queue = dx12::error_if_failed_else_value(device.create_command_queue(
            list_type,
            0,
            d3d12::D3D12_COMMAND_QUEUE_FLAG_NONE,
            0,
        ))
        .expect("could not create command queue");

        // create swapchain
        let swapchain_desc = dxgi1_2::DXGI_SWAP_CHAIN_DESC1 {
            AlphaMode: dxgi1_2::DXGI_ALPHA_MODE_UNSPECIFIED,
            BufferCount: FRAME_COUNT,
            Width: width,
            Height: height,
            Format: winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            Flags: 0,
            BufferUsage: dxgitype::DXGI_USAGE_UNORDERED_ACCESS,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Scaling: dxgitype::DXGI_MODE_SCALING_CENTERED,
            Stereo: minwindef::FALSE,
            SwapEffect: dxgi::DXGI_SWAP_EFFECT_DISCARD,
        };

        let swap_chain1 =
            dx12::error_if_failed_else_value(factory4.as_factory2().create_swapchain_for_hwnd(
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
        let descriptor_heap_desc = d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
            Type: d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: FRAME_COUNT,
            Flags: d3d12::D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            NodeMask: 0,
        };
        let rtv_heap =
            dx12::error_if_failed_else_value(device.create_descriptor_heap(&descriptor_heap_desc))
                .expect("could not create descriptor heap");
        let rtv_descriptor_size =
            device.get_descriptor_increment_size(d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV);

        // create frame resources
        let mut cpu_descriptor = rtv_heap.start_cpu_descriptor();
        // create render target and render target view for each frame
        let mut render_targets: Vec<dx12::Resource> = Vec::new();
        for ix in 0..FRAME_COUNT {
            let resource = dx12::error_if_failed_else_value(swap_chain3.get_buffer(ix))
                .expect("could not create render target resource");
            device.create_render_target_view(resource.clone(), std::ptr::null(), cpu_descriptor);
            // TODO: is this correct?
            cpu_descriptor.ptr += rtv_descriptor_size as usize;
            render_targets.push(resource.clone());
        }

        let command_allocator =
            dx12::error_if_failed_else_value(device.create_command_allocator(list_type))
                .expect("could not create command allocator");

        let fence = dx12::error_if_failed_else_value(device.create_fence(0))
            .expect("could not create fence");

        (
            swap_chain3,
            device,
            render_targets,
            command_allocator,
            command_queue,
            rtv_heap,
            rtv_descriptor_size,
            fence,
        )
    }

    unsafe fn create_pipeline_state(
        device: &dx12::Device,
        shader_code: &[u8],
        entry: String,
        command_allocator: dx12::CommandAllocator,
    ) -> (
        dx12::RootSignature,
        dx12::PipelineState,
        dx12::GraphicsCommandList,
    ) {
        // create empty root signature
        let root_signature_desc = d3d12::D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: 0,
            pParameters: std::ptr::null(),
            NumStaticSamplers: 0,
            pStaticSamplers: std::ptr::null(),
            Flags: d3d12::D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
        };
        let (blob, error_blob) = dx12::error_if_failed_else_value(dx12::RootSignature::serialize(
            &root_signature_desc,
            d3d12::D3D_ROOT_SIGNATURE_VERSION_1,
        ))
        .expect("could not serialize root signature");
        let root_signature =
            dx12::error_if_failed_else_value(device.create_root_signature(0, blob))
                .expect("could not create root signature");

        let mut flags: minwindef::DWORD = 0;

        #[cfg(debug_assertions)]
        {
            flags = winapi::um::d3dcompiler::D3DCOMPILE_DEBUG
                | winapi::um::d3dcompiler::D3DCOMPILE_SKIP_OPTIMIZATION;
        }

        // load shader
        let (compute_shader_blob, compile_error_blob) = dx12::error_if_failed_else_value(
            dx12::ShaderByteCode::compile(shader_code, String::from("cs_5_0"), entry, flags),
        )
        .expect("could not compile compute shader");
        let compute_shader_bytecode = dx12::ShaderByteCode::from_blob(compute_shader_blob);

        // create compute pipeline state
        let compute_ps_desc = d3d12::D3D12_COMPUTE_PIPELINE_STATE_DESC {
            pRootSignature: root_signature.0.as_raw(),
            CS: compute_shader_bytecode.0,
            NodeMask: 0,
            CachedPSO: d3d12::D3D12_CACHED_PIPELINE_STATE {
                pCachedBlob: std::ptr::null(),
                CachedBlobSizeInBytes: 0,
            },
            Flags: d3d12::D3D12_PIPELINE_STATE_FLAG_NONE,
        };
        let compute_pipeline_state = dx12::error_if_failed_else_value(
            device.create_compute_pipeline_state(&compute_ps_desc),
        )
        .expect("could not create compute pipeline state");

        // create command list
        let command_list = dx12::error_if_failed_else_value(device.create_graphics_command_list(
            d3d12::D3D12_COMMAND_LIST_TYPE_COMPUTE,
            command_allocator.clone(),
            compute_pipeline_state.clone(),
            0,
        ))
        .expect("could not create compute pipeline list");
        command_list.close();

        (root_signature, compute_pipeline_state, command_list)
    }
}
