extern crate winapi;

use crate::dx12;
use crate::window;
use std::{mem, ptr};
use winapi::shared::{dxgi, dxgi1_2, dxgitype, minwindef, winerror};
use winapi::um::d3d12;
use winapi::Interface;

const FRAME_COUNT: u32 = 2;
pub type TriangleVertexCoordinates = [f32; 3];
pub type TriangleVertexIndices = [f32; 4];
pub type TriangleVertex = (TriangleVertexCoordinates, TriangleVertexIndices);
const TRIANGLE_VERTICES: [TriangleVertex; 3] = [
    ([0.0, 0.25, 0.0], [1.0, 0.0, 0.0, 1.0]),
    ([0.25, -0.25, 0.0], [0.0, 1.0, 0.0, 1.0]),
    ([-0.25, -0.25, 0.0], [0.0, 0.0, 1.0, 1.0]),
];

pub struct GpuState {
    width: u32,
    height: u32,

    // pipeline stuff
    viewport: d3d12::D3D12_VIEWPORT,
    scissor_rect: d3d12::D3D12_RECT,
    swapchain: dx12::SwapChain3,
    device: dx12::Device,
    render_targets: Vec<dx12::Resource>,
    //compute_targets: Vec<dx12::Resource>,
    command_allocator: dx12::CommandAllocator,
    command_queue: dx12::CommandQueue,
    //compute_root_signature: dx12::RootSignature,
    graphics_root_signature: dx12::RootSignature,
    render_target_view_heap: dx12::DescriptorHeap,
    //compute_target_view_heap: dx12::DescriptorHeap,
    render_target_view_descriptor_size: u32,
    //compute_target_view_descriptor_size: u32,
    graphics_pipeline_state: dx12::PipelineState,
    //compute_pipeline_state: dx12::PipelineState,
    command_list: dx12::GraphicsCommandList,
    vertex_buffer: dx12::Resource,
    vertex_buffer_view: d3d12::D3D12_VERTEX_BUFFER_VIEW,

    // synchronizers
    frame_index: usize,
    fence_event: dx12::Event,
    fence: dx12::Fence,
    fence_value: u64,
}

impl GpuState {
    pub unsafe fn new(
        wnd: &window::Window,
        compute_shader_code: &[u8],
        compute_entry: String,
        vertex_shader_code: &[u8],
        vertex_entry: String,
        fragment_shader_code: &[u8],
        fragment_entry: String,
    ) -> GpuState {
        let width = wnd.get_width();
        let height = wnd.get_height();

        let viewport = d3d12::D3D12_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0 as f32,
            Width: width as f32,
            Height: height as f32,
            MinDepth: 0.0,
            MaxDepth: 0.0,
        };

        let scissor_rect = d3d12::D3D12_RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };

        #[cfg(debug_assertions)]
        dx12::enable_debug_layer();

        let (
            swapchain,
            device,
            render_targets,
            //compute_targets,
            command_allocator,
            command_queue,
            render_target_view_heap,
            //compute_target_view_heap,
            render_target_view_descriptor_size,
            //compute_target_view_descriptor_size,
            fence,
        ) = GpuState::create_pipeline_dependencies(width, height, wnd);

        let vertex_shader_code =
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
".as_bytes();

        let fragment_shader_code =
"struct PSInput
{
    float4 position : SV_POSITION;
    float4 color : COLOR;
};

float4 PSMain(PSInput input) : SV_TARGET
{
    return input.color;
}
".as_bytes();


        let (
            //compute_root_signature,
            graphics_root_signature,
            //compute_pipeline_state,
            graphics_pipeline_state,
            command_list,
            vertex_buffer,
            vertex_buffer_view,
        ) = GpuState::create_pipeline_state(
            &device,
            compute_shader_code,
            vertex_shader_code,
            fragment_shader_code,
            compute_entry,
            vertex_entry,
            fragment_entry,
            command_allocator.clone(),
        );

        let fence_event = dx12::Event::create(false, false);

        GpuState {
            width,
            height,
            viewport,
            scissor_rect,
            swapchain,
            device,
            //compute_targets,
            render_targets,
            command_allocator,
            command_queue,
            //compute_root_signature,
            graphics_root_signature,
            render_target_view_heap,
            //compute_target_view_heap,
            //compute_pipeline_state,
            graphics_pipeline_state,
            command_list,
            render_target_view_descriptor_size,
            //compute_target_view_descriptor_size,
            frame_index: 0,
            fence_event,
            fence,
            fence_value: 1,
            vertex_buffer,
            vertex_buffer_view,
        }
    }

    unsafe fn populate_command_list(&mut self) {
        self.command_allocator.reset();

        // compute pipeline call
//        self.command_list.reset(
//            self.command_allocator.clone(),
//            self.compute_pipeline_state.clone(),
//        );
//        self.command_list
//            .set_compute_root_signature(self.compute_root_signature.clone());
//        let transition_intermediate_to_unordered_access = dx12::create_transition_resource_barrier(
//            self.compute_targets[self.frame_index].0.as_raw(),
//            d3d12::D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
//            d3d12::D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
//        );
//        self.command_list
//            .set_resource_barrier(1, [transition_intermediate_to_unordered_access].as_ptr());
//        let ct_gpu_virtual_address =
//            self.compute_targets[self.frame_index].get_gpu_virtual_address();
//        self.command_list
//            .set_compute_root_unordered_access_view(0, ct_gpu_virtual_address);
//        self.command_list.dispatch(10, 10, 10);

        // graphics pipeline call
//        self.command_list
//            .set_pipeline_state(self.graphics_pipeline_state.clone());
        self.command_list.reset(self.command_allocator.clone(), self.graphics_pipeline_state.clone());
        self.command_list
            .set_graphics_root_signature(self.graphics_root_signature.clone());
        self.command_list.set_viewport(&self.viewport);
        self.command_list.set_scissor_rect(&self.scissor_rect);
//        let transition_intermediate_to_pixel_shader_resource =
//            dx12::create_transition_resource_barrier(
//                self.compute_targets[self.frame_index].0.as_raw(),
//                d3d12::D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
//                d3d12::D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
//            );
        let transition_render_target_from_present = dx12::create_transition_resource_barrier(
            self.render_targets[self.frame_index].0.as_raw(),
            d3d12::D3D12_RESOURCE_STATE_PRESENT,
            d3d12::D3D12_RESOURCE_STATE_RENDER_TARGET,
        );
        self.command_list.set_resource_barrier(
            2,
            [
                //transition_intermediate_to_pixel_shader_resource,
                transition_render_target_from_present,
            ]
            .as_ptr(),
        );
//        self.command_list
//            .set_graphics_root_shader_resource_view(0, ct_gpu_virtual_address);
        let mut rt_descriptor = self.render_target_view_heap.start_cpu_descriptor();
        rt_descriptor.ptr += self.frame_index;
        self.command_list.set_render_target(rt_descriptor);
        self.command_list.draw(0, 0, 0, 0);
        let transition_render_target_to_present = dx12::create_transition_resource_barrier(
            self.render_targets[self.frame_index].0.as_raw(),
            d3d12::D3D12_RESOURCE_STATE_RENDER_TARGET,
            d3d12::D3D12_RESOURCE_STATE_PRESENT,
        );
        self.command_list
            .set_resource_barrier(1, [transition_render_target_to_present].as_ptr());

        self.command_list.close();
    }

    unsafe fn execute_command_list(&mut self) {
        let raw_command_list = self.command_list.as_raw_list();
        self.command_queue
            .execute_command_lists(1, &[raw_command_list.0.as_raw()]);
    }

    unsafe fn render(&mut self) {
        self.populate_command_list();

        self.execute_command_list();

        // TODO: what should the present flags be?
        dx12::error_if_failed_else_none(self.swapchain.present(0, 0)).expect("presentation failed");

        self.wait_for_render_completion();
    }

    unsafe fn wait_for_render_completion(&mut self) {
        self.command_queue
            .signal(self.fence.clone(), self.fence_value);
        self.fence_value = !self.fence_value;

        if self.fence.get_value() != self.fence_value {
            dx12::error_if_failed_else_none(
                self.fence
                    .set_event_on_completion(self.fence_event.clone(), self.fence_value),
            )
            .expect("error setting fence event on render completion");
            //TODO: handle return value?
            self.fence_event.wait(std::u32::MAX);
        }

        self.frame_index = self.swapchain.get_current_back_buffer_index() as usize;
    }

    pub unsafe fn destroy(&mut self) {
        self.wait_for_render_completion();
        if winapi::um::handleapi::CloseHandle(self.fence_event.0) == 0 {
            panic!("could not close fence event properly")
        }
    }

    unsafe fn create_pipeline_dependencies(
        width: u32,
        height: u32,
        wnd: &window::Window,
    ) -> (
        dx12::SwapChain3,
        dx12::Device,
        Vec<dx12::Resource>,
        //Vec<dx12::Resource>,
        dx12::CommandAllocator,
        dx12::CommandQueue,
        dx12::DescriptorHeap,
        //dx12::DescriptorHeap,
        u32,
        //u32,
        dx12::Fence,
    ) {
        // create factory4
        let factory4 = dx12::Factory4::create(0);

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

        //TODO: command list type okay?
        let list_type = d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT;
        let command_queue =
            device.create_command_queue(list_type, 0, d3d12::D3D12_COMMAND_QUEUE_FLAG_NONE, 0);

        // create compute resource descriptions
//        let heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
//            //for GPU access only
//            Type: d3d12::D3D12_HEAP_TYPE_DEFAULT,
//            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
//            //TODO: what should MemoryPoolPreference flag be?
//            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
//            //we don't care about multi-adapter operation, so these next two will be zero
//            CreationNodeMask: 0,
//            VisibleNodeMask: 0,
//        };
//        //TODO: consider flag D3D12_HEAP_FLAG_ALLOW_SHADER_ATOMICS?
//        let heap_usage_flags = d3d12::D3D12_HEAP_FLAG_NONE;
//        let resource_description = d3d12::D3D12_RESOURCE_DESC {
//            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_TEXTURE2D,
//            //TODO: what alignment should be chosen?
//            Alignment: 0,
//            Width: width as u64,
//            Height: height,
//            DepthOrArraySize: 1,
//            //TODO: what should MipLevels be?
//            MipLevels: 1,
//            Format: winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
//            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
//                Count: 1,
//                Quality: 0,
//            },
//            //essentially we're letting the adapter decide the layout
//            Layout: d3d12::D3D12_TEXTURE_LAYOUT_UNKNOWN,
//            Flags: d3d12::D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
//        };
//        let mut clear_value: d3d12::D3D12_CLEAR_VALUE = mem::zeroed();
//        *clear_value.u.Color_mut() = [0.0, 0.0, 0.0, 0.0];
//
//        // create compute descriptor heap
//        let compute_heap_type = d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV;
//        let ct_descriptor_heap_desc = d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
//            Type: compute_heap_type,
//            NumDescriptors: FRAME_COUNT,
//            Flags: heap_usage_flags,
//            NodeMask: 0,
//        };
//        let ctv_heap = device.create_descriptor_heap(&ct_descriptor_heap_desc);
//        let ctv_descriptor_size = device.get_descriptor_increment_size(compute_heap_type);

        // create swapchain
        let swapchain_desc = dxgi1_2::DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            AlphaMode: dxgi1_2::DXGI_ALPHA_MODE_IGNORE,
            BufferCount: FRAME_COUNT,
            Format: winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            Flags: 0,
            BufferUsage: dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Scaling: dxgi1_2::DXGI_SCALING_STRETCH,
            Stereo: winapi::shared::minwindef::FALSE,
            SwapEffect: dxgi::DXGI_SWAP_EFFECT_FLIP_DISCARD,
        };

        let swap_chain3 = factory4.create_swapchain_for_hwnd(
            command_queue.clone(),
            wnd.hwnd.clone(),
            swapchain_desc,
        );

        //let swap_chain3 = swap_chain1.cast_into_swap_chain3();
        // disable full screen transitions
        // winapi does not have DXGI_MWA_NO_ALT_ENTER?
        factory4.0.MakeWindowAssociation(wnd.hwnd, 1);

        // create graphics descriptor heap
        let rt_descriptor_heap_desc = d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
            Type: d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: FRAME_COUNT,
            Flags: d3d12::D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            NodeMask: 0,
        };
        let rtv_heap = device.create_descriptor_heap(&rt_descriptor_heap_desc);
        let rtv_descriptor_size =
            device.get_descriptor_increment_size(d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV);

        // create frame resources
        //let mut ct_cpu_descriptor = ctv_heap.start_cpu_descriptor();
        let mut rt_cpu_descriptor = rtv_heap.start_cpu_descriptor();
        // create render target and render target view for each frame
        //let mut compute_targets: Vec<dx12::Resource> = Vec::new();
        let mut render_targets: Vec<dx12::Resource> = Vec::new();

        // just work on getting rasterization working
        for ix in 0..FRAME_COUNT {
//            let compute_target_resource = device.create_committed_resource(
//                &heap_properties,
//                heap_usage_flags,
//                &resource_description,
//                d3d12::D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
//                ptr::null(),
//            );
            let render_target_resource = swap_chain3.get_buffer(ix);

//            device.create_unordered_access_view(compute_target_resource.clone(), ct_cpu_descriptor);
            device.create_render_target_view(
                render_target_resource.clone(),
                ptr::null(),
                rt_cpu_descriptor,
            );

            // TODO: is this correct?
            //ct_cpu_descriptor.ptr += ctv_descriptor_size as usize;
            rt_cpu_descriptor.ptr += rtv_descriptor_size as usize;

            //compute_targets.push(compute_target_resource.clone());
            render_targets.push(render_target_resource.clone());
        }

        let command_allocator = device.create_command_allocator(list_type);

        let fence = device.create_fence(0);

        (
            swap_chain3,
            device,
            //compute_targets,
            render_targets,
            command_allocator,
            command_queue,
            rtv_heap,
            //ctv_heap,
            rtv_descriptor_size,
            //ctv_descriptor_size,
            fence,
        )
    }

    unsafe fn create_pipeline_state(
        device: &dx12::Device,
        compute_shader_code: &[u8],
        vertex_shader_code: &[u8],
        fragment_shader_code: &[u8],
        compute_entry: String,
        vertex_entry: String,
        fragment_entry: String,
        command_allocator: dx12::CommandAllocator,
    ) -> (
        //dx12::RootSignature,
        dx12::RootSignature,
        //dx12::PipelineState,
        dx12::PipelineState,
        dx12::GraphicsCommandList,
        dx12::Resource,
        d3d12::D3D12_VERTEX_BUFFER_VIEW,
    ) {
        // create empty root signature for compute
        // create 1 parameter root signature for graphics
//        let graphics_root_parameter = d3d12::D3D12_ROOT_PARAMETER {
//            ParameterType: d3d12::D3D12_ROOT_PARAMETER_TYPE_CBV,
//            ShaderVisibility: d3d12::D3D12_SHADER_VISIBILITY_PIXEL,
//            ..mem::zeroed()
//        };
//        let compute_root_signature_desc = d3d12::D3D12_ROOT_SIGNATURE_DESC {
//            NumParameters: 0,
//            pParameters: ptr::null(),
//            NumStaticSamplers: 0,
//            pStaticSamplers: ptr::null(),
//            Flags: d3d12::D3D12_ROOT_SIGNATURE_FLAG_NONE,
//        };
//        let graphics_root_signature_desc = d3d12::D3D12_ROOT_SIGNATURE_DESC {
//            NumParameters: 1,
//            pParameters: &graphics_root_parameter as *const _,
//            NumStaticSamplers: 0,
//            pStaticSamplers: ptr::null(),
//            Flags: d3d12::D3D12_ROOT_SIGNATURE_FLAG_NONE,
//        };
//        // serialize root signature description and create compute root signature
//        let blob = dx12::RootSignature::serialize_description(
//            &compute_root_signature_desc,
//            d3d12::D3D_ROOT_SIGNATURE_VERSION_1,
//        );
//        let compute_root_signature = device.create_root_signature(0, blob);
        let graphics_root_signature_desc = d3d12::D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: 0,
            pParameters: ptr::null(),
            NumStaticSamplers: 0,
            pStaticSamplers: ptr::null(),
            Flags: d3d12::D3D12_ROOT_SIGNATURE_FLAG_NONE,
        };
        // serialize root signature description and create graphics root signature
        let blob= dx12::RootSignature::serialize_description(
            &graphics_root_signature_desc,
            d3d12::D3D_ROOT_SIGNATURE_VERSION_1,
        );
        let graphics_root_signature = device.create_root_signature(0, blob);

        let vertex_buffer_stride = mem::size_of::<TriangleVertex>();
        let vertex_buffer_size = vertex_buffer_stride*TRIANGLE_VERTICES.len();
        let vertex_buffer_heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            Type: d3d12::D3D12_HEAP_TYPE_UPLOAD,
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        let vertex_buffer_resource_description = d3d12::D3D12_RESOURCE_DESC {
            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: vertex_buffer_size as u64,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: d3d12::D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            ..mem::zeroed()
        };
        let vertex_buffer = device.create_committed_resource(&vertex_buffer_heap_properties, d3d12::D3D12_HEAP_FLAG_NONE, &vertex_buffer_resource_description, d3d12::D3D12_RESOURCE_STATE_GENERIC_READ, ptr::null());
        vertex_buffer.upload_data_to_resource(vertex_buffer_size, TRIANGLE_VERTICES.as_ptr());
        let vertex_buffer_view = d3d12::D3D12_VERTEX_BUFFER_VIEW {
          BufferLocation: vertex_buffer.get_gpu_virtual_address(),
          SizeInBytes: vertex_buffer_size as u32,
          StrideInBytes: vertex_buffer_stride as u32,
        };

        let mut flags: minwindef::DWORD = 0;

        #[cfg(debug_assertions)]
        {
            flags = winapi::um::d3dcompiler::D3DCOMPILE_DEBUG
                | winapi::um::d3dcompiler::D3DCOMPILE_SKIP_OPTIMIZATION;
        }

//        // load compute shader
//        let compute_shader_blob = dx12::ShaderByteCode::compile(
//            compute_shader_code,
//            String::from("cs_5_0"),
//            compute_entry,
//            flags,
//        );
//        let compute_shader_bytecode = dx12::ShaderByteCode::from_blob(compute_shader_blob);

        // load graphics shaders
        println!("compiling vertex shader code...");
        let graphics_vertex_shader_blob= dx12::ShaderByteCode::compile(
            vertex_shader_code,
            String::from("cs_5_0"),
            vertex_entry,
            flags,
        );
        let graphics_vertex_shader_bytecode =
            dx12::ShaderByteCode::from_blob(graphics_vertex_shader_blob);
        println!("compiling fragment shader code...");
        let graphics_fragment_shader_blob= dx12::ShaderByteCode::compile(
            fragment_shader_code,
            String::from("cs_5_0"),
            fragment_entry,
            flags,
        );
        let graphics_fragment_shader_bytecode =
            dx12::ShaderByteCode::from_blob(graphics_fragment_shader_blob);

        // create compute pipeline state
//        let compute_ps_desc = d3d12::D3D12_COMPUTE_PIPELINE_STATE_DESC {
//            pRootSignature: compute_root_signature.0.as_raw(),
//            CS: compute_shader_bytecode.0,
//            NodeMask: 0,
//            CachedPSO: d3d12::D3D12_CACHED_PIPELINE_STATE {
//                pCachedBlob: ptr::null(),
//                CachedBlobSizeInBytes: 0,
//            },
//            Flags: d3d12::D3D12_PIPELINE_STATE_FLAG_NONE,
//        };
//        let compute_pipeline_state = device.create_compute_pipeline_state(&compute_ps_desc);

        // create graphics pipeline state
        let graphics_ps_desc = d3d12::D3D12_GRAPHICS_PIPELINE_STATE_DESC {
            pRootSignature: graphics_root_signature.0.as_raw(),
            VS: graphics_vertex_shader_bytecode.0,
            PS: graphics_fragment_shader_bytecode.0,
            DS: dx12::ShaderByteCode::empty().0,
            HS: dx12::ShaderByteCode::empty().0,
            GS: dx12::ShaderByteCode::empty().0,
            StreamOutput: d3d12::D3D12_STREAM_OUTPUT_DESC {
                pSODeclaration: ptr::null(),
                NumEntries: 0,
                pBufferStrides: ptr::null(),
                NumStrides: 0,
                RasterizedStream: 0,
            },
            //TODO: confirm do nothing blend desc is correct
            BlendState: dx12::do_nothing_blend_desc(),
            SampleMask: 0,
            // TODO: could ..mem::zeroed() work here?
            RasterizerState: d3d12::D3D12_RASTERIZER_DESC {
                FillMode: d3d12::D3D12_FILL_MODE_SOLID,
                CullMode: d3d12::D3D12_CULL_MODE_NONE,
                FrontCounterClockwise: true as _,
                DepthBias: 0,
                DepthBiasClamp: 0.0,
                SlopeScaledDepthBias: 0.0,
                DepthClipEnable: false as _,
                MultisampleEnable: false as _,
                AntialiasedLineEnable: false as _,
                ForcedSampleCount: 0,
                ConservativeRaster: d3d12::D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
            },
            // TODO: could ..mem::zeroed() work here?
            DepthStencilState: d3d12::D3D12_DEPTH_STENCIL_DESC {
                DepthEnable: false as _,
                DepthWriteMask: 0,
                DepthFunc: 0,
                StencilEnable: false as _,
                StencilReadMask: 0,
                StencilWriteMask: 0,
                FrontFace: d3d12::D3D12_DEPTH_STENCILOP_DESC { ..mem::zeroed() },
                BackFace: d3d12::D3D12_DEPTH_STENCILOP_DESC { ..mem::zeroed() },
            },
            InputLayout: d3d12::D3D12_INPUT_LAYOUT_DESC { ..mem::zeroed() },
            IBStripCutValue: 0,
            PrimitiveTopologyType: 0,
            NumRenderTargets: 1,
            RTVFormats: [
                winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
                winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
                winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
                winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
                winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
                winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
                winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
                winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            ],
            DSVFormat: winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            NodeMask: 0,
            CachedPSO: d3d12::D3D12_CACHED_PIPELINE_STATE {
                pCachedBlob: ptr::null(),
                CachedBlobSizeInBytes: 0,
            },
            Flags: d3d12::D3D12_PIPELINE_STATE_FLAG_NONE,
        };
        let graphics_pipeline_state = device.create_graphics_pipeline_state(&graphics_ps_desc);

        // create command list
//        let command_list = device.create_graphics_command_list(
//            d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT,
//            command_allocator.clone(),
//            compute_pipeline_state.clone(),
//            0,
//        );
        let command_list = device.create_graphics_command_list(
            d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT,
            command_allocator.clone(),
            graphics_pipeline_state.clone(),
            0,
        );
        command_list.close();

        (
            //compute_root_signature,
            graphics_root_signature,
            //compute_pipeline_state,
            graphics_pipeline_state,
            command_list,
            vertex_buffer,
            vertex_buffer_view,
        )
    }
}
