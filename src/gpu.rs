extern crate winapi;

use crate::dx12;
use crate::window;
use std::{mem, ptr};
use winapi::shared::{dxgi, dxgi1_2, dxgiformat, dxgitype, minwindef, winerror};
use winapi::um::{d3d12, d3dcommon};
use winapi::Interface;

const FRAME_COUNT: u32 = 2;
pub type VertexCoordinates = [f32; 3];
pub type VertexColor = [f32; 4];
pub type Vertex = VertexCoordinates;

pub struct Quad {
    ox: f32,
    oy: f32,
    width: f32,
    height: f32,
}

impl Quad {
    fn new(ox: f32, oy: f32, width: f32, height: f32) -> Quad {
        Quad {
            ox,
            oy,
            width,
            height,
        }
    }

    fn as_vertices(&self) -> [Vertex; 4] {
        [
            [self.ox, self.oy, 0.0],
            [self.ox, self.oy + self.height, 0.0],
            [self.ox + self.width, self.oy, 0.0],
            [self.ox + self.width, self.oy + self.height, 0.0],
        ]
    }
}

const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

pub struct GpuState {
    width: u32,
    height: u32,

    // pipeline stuff
    viewport: d3d12::D3D12_VIEWPORT,
    scissor_rect: d3d12::D3D12_RECT,
    swapchain: dx12::SwapChain3,
    device: dx12::Device,
    render_targets: Vec<dx12::Resource>,
    compute_targets: Vec<dx12::Resource>,
    command_allocators: Vec<dx12::CommandAllocator>,
    command_queue: dx12::CommandQueue,
    compute_root_signature: dx12::RootSignature,
    graphics_root_signature: dx12::RootSignature,
    rtv_descriptor_heap: dx12::DescriptorHeap,
    compute_target_descriptor_heap: dx12::DescriptorHeap,
    rtv_descriptor_size: u32,
    graphics_pipeline_state: dx12::PipelineState,
    compute_pipeline_state: dx12::PipelineState,
    command_list: dx12::GraphicsCommandList,
    vertex_buffer: dx12::Resource,
    vertex_buffer_view: d3d12::D3D12_VERTEX_BUFFER_VIEW,

    // synchronizers
    frame_index: usize,
    fence_event: dx12::Event,
    fence: dx12::Fence,
    fence_values: Vec<u64>,

    num_dispatch_threadgroups_x: u32,
    num_dispatch_threadgroups_y: u32,
}

impl GpuState {
    pub unsafe fn new(
        wnd: &window::Window,
        shader_code: &[u8],
        compute_entry: String,
        vertex_entry: String,
        fragment_entry: String,
    ) -> GpuState {
        let width = wnd.get_width();
        let height = wnd.get_height();

        let canvas_quad_width = ((width as f32) / 16.0).ceil() * 16.0;
        let canvas_quad_height = ((height as f32) / 16.0).ceil() * 16.0;
        let num_dispatch_threadgroups_x = (canvas_quad_width / 16.0) as u32;
        let num_dispatch_threadgroups_y = (canvas_quad_height / 16.0) as u32;
        let canvas_quad = Quad::new(
            -1.0 * (canvas_quad_width / 2.0),
            -1.0 * (canvas_quad_height / 2.0),
            canvas_quad_width,
            canvas_quad_height,
        );

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
            compute_targets,
            render_targets,
            command_allocators,
            command_queue,
            compute_target_descriptor_heap,
            rtv_descriptor_heap,
            rtv_descriptor_size,
            fence,
        ) = GpuState::create_pipeline_dependencies(width, height, wnd);

        let (
            compute_root_signature,
            compute_pipeline_state,
            graphics_root_signature,
            graphics_pipeline_state,
            vertex_buffer,
            vertex_buffer_view,
            command_list,
        ) = GpuState::create_pipeline_states(
            &device,
            shader_code,
            compute_entry,
            vertex_entry,
            fragment_entry,
            &command_allocators,
            canvas_quad,
        );

        let fence_event = dx12::Event::create(false, false);

        let mut gpu_state = GpuState {
            width,
            height,
            viewport,
            scissor_rect,
            swapchain,
            device,
            compute_targets,
            render_targets,
            command_allocators,
            command_queue,
            compute_root_signature,
            graphics_root_signature,
            rtv_descriptor_heap,
            compute_target_descriptor_heap,
            compute_pipeline_state,
            graphics_pipeline_state,
            command_list,
            rtv_descriptor_size,
            frame_index: 0,
            fence_event,
            fence,
            fence_values: (0..FRAME_COUNT).into_iter().map(|_| 1).collect(),
            vertex_buffer,
            vertex_buffer_view,
            num_dispatch_threadgroups_x,
            num_dispatch_threadgroups_y,
        };

        // wait for upload of any resources to gpu
        gpu_state.wait_for_gpu();

        gpu_state
    }

    unsafe fn populate_command_list(&mut self) {
        println!("  populating command list...");
        println!("      resetting relevant command allocator...");
        self.command_allocators[self.frame_index].reset();

        // compute pipeline call
        println!("      resetting command list...");
        self.command_list.reset(
            self.command_allocators[self.frame_index].clone(),
            self.compute_pipeline_state.clone(),
        );
        println!("      command list: set compute root signature...");
        self.command_list
            .set_compute_root_signature(self.compute_root_signature.clone());
        self.command_list.set_descriptor_heaps(vec![self.compute_target_descriptor_heap.clone()]);
        self.command_list.set_compute_root_descriptor_table(0, self.compute_target_descriptor_heap.get_gpu_descriptor_handle_for_heap_start());
        self.command_list.dispatch(self.num_dispatch_threadgroups_x, self.num_dispatch_threadgroups_y, 1);

        // graphics pipeline call
        println!("      setting command list pipeline state to graphics...");
        self.command_list
            .set_pipeline_state(self.graphics_pipeline_state.clone());
        println!("      command list: set graphics root signature...");
        self.command_list
            .set_graphics_root_signature(self.graphics_root_signature.clone());
        println!("      command list: set viewport...");
        self.command_list.set_descriptor_heaps(vec![self.compute_target_descriptor_heap.clone()]);
        self.command_list.set_graphics_root_descriptor_table(0, self.compute_target_descriptor_heap.get_gpu_descriptor_handle_for_heap_start());
        self.command_list.set_viewport(&self.viewport);
        println!("      command list: set scissor rect...");
        self.command_list.set_scissor_rect(&self.scissor_rect);
        let transition_render_target_from_present = dx12::create_transition_resource_barrier(
            self.render_targets[self.frame_index].0.as_raw(),
            d3d12::D3D12_RESOURCE_STATE_PRESENT,
            d3d12::D3D12_RESOURCE_STATE_RENDER_TARGET,
        );
        println!("      command list: set pre-draw resource barrier...");
        self.command_list.set_resource_barrier(
            1,
            [
                //transition_intermediate_to_pixel_shader_resource,
                transition_render_target_from_present,
            ]
                .as_ptr(),
        );
        let mut rt_descriptor = self.rtv_descriptor_heap.get_cpu_descriptor_handle_for_heap_start();
        let rt_descriptor_size = self
            .device
            .get_descriptor_increment_size(d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV);
        rt_descriptor.ptr += self.frame_index * (rt_descriptor_size as usize);
        println!("      command list: set render target...");
        self.command_list.set_render_target(rt_descriptor);

        // Record drawing commands.
        println!("      command list: record draw commands...");
        self.command_list
            .clear_render_target_view(rt_descriptor, &CLEAR_COLOR);
        self.command_list
            .set_primitive_topology(d3dcommon::D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
        self.command_list
            .set_vertex_buffer(0, 1, &self.vertex_buffer_view);
        self.command_list.draw_instanced(4, 1, 0, 0);

        let transition_render_target_to_present = dx12::create_transition_resource_barrier(
            self.render_targets[self.frame_index].0.as_raw(),
            d3d12::D3D12_RESOURCE_STATE_RENDER_TARGET,
            d3d12::D3D12_RESOURCE_STATE_PRESENT,
        );
        println!("      command list: set post-draw resource barrier...");
        self.command_list
            .set_resource_barrier(1, [transition_render_target_to_present].as_ptr());

        println!("      command list: close...");
        self.command_list.close();
    }

    unsafe fn execute_command_list(&mut self) {
        println!("  executing command list...");
        self.command_queue
            .execute_command_lists(1, &[self.command_list.as_raw_list()]);
    }

    pub unsafe fn render(&mut self) {
        self.populate_command_list();

        self.execute_command_list();

        // TODO: what should the present flags be?
        self.swapchain.present(1, 0);

        self.move_to_next_frame();
    }

    unsafe fn wait_for_gpu(&mut self) {
        self.command_queue
            .signal(self.fence.clone(), self.fence_values[self.frame_index]);

        //TODO: handle return value
        self.fence.set_event_on_completion(
            self.fence_event.clone(),
            self.fence_values[self.frame_index],
        );
        self.fence_event
            .wait_ex(winapi::um::winbase::INFINITE, false);

        self.fence_values[self.frame_index] = self.fence_values[self.frame_index] + 1;

        self.fence_values[self.frame_index] = self.fence_values[self.frame_index] + 1;
    }

    unsafe fn move_to_next_frame(&mut self) {
        let current_fence_value = self.fence_values[self.frame_index];
        self.command_queue
            .signal(self.fence.clone(), current_fence_value);

        self.frame_index = self.swapchain.get_current_back_buffer_index() as usize;

        if self.fence.get_value() < self.fence_values[self.frame_index] {
            self.fence.set_event_on_completion(
                self.fence_event.clone(),
                self.fence_values[self.frame_index],
            );
            //TODO: handle return value
            self.fence_event
                .wait_ex(winapi::um::winbase::INFINITE, false);
        }

        self.fence_values[self.frame_index] = current_fence_value + 1;
    }

    pub unsafe fn destroy(&mut self) {
        self.wait_for_gpu();

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
        Vec<dx12::Resource>,
        Vec<dx12::CommandAllocator>,
        dx12::CommandQueue,
        dx12::DescriptorHeap,
        dx12::DescriptorHeap,
        u32,
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

        //create compute resource descriptions
        let compute_heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            //for GPU access only
            Type: d3d12::D3D12_HEAP_TYPE_DEFAULT,
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        //TODO: consider flag D3D12_HEAP_FLAG_ALLOW_SHADER_ATOMICS?
        let compute_resource_heap_flags = d3d12::D3D12_HEAP_FLAG_NONE;
        let compute_resource_desc = d3d12::D3D12_RESOURCE_DESC {
            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            //TODO: what alignment should be chosen?
            Alignment: 0,
            Width: width as u64,
            Height: height,
            DepthOrArraySize: 1,
            //TODO: what should MipLevels be?
            MipLevels: 1,
            Format: winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            //essentially we're letting the adapter decide the layout
            Layout: d3d12::D3D12_TEXTURE_LAYOUT_UNKNOWN,
            Flags: compute_resource_heap_flags,
        };
        let mut clear_value: d3d12::D3D12_CLEAR_VALUE = mem::zeroed();
        *clear_value.u.Color_mut() = [0.0, 0.0, 0.0, 0.0];

        // create compute descriptor heap
        let ct_descriptor_heap_desc = d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
            Type: d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            NumDescriptors: FRAME_COUNT,
            Flags: d3d12::D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
            NodeMask: 0,
        };
        let compute_target_descriptor_heap = device.create_descriptor_heap(&ct_descriptor_heap_desc);

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
        let rtv_descriptor_heap_desc = d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
            Type: d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: FRAME_COUNT,
            Flags: d3d12::D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            NodeMask: 0,
        };
        let rtv_descriptor_heap = device.create_descriptor_heap(&rtv_descriptor_heap_desc);
        let rtv_descriptor_size =
            device.get_descriptor_increment_size(d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV);

        // create frame resources
        let mut rt_cpu_descriptor = rtv_descriptor_heap.get_cpu_descriptor_handle_for_heap_start();
        // create render target and render target view for each frame
        let mut compute_targets: Vec<dx12::Resource> = Vec::new();
        let mut render_targets: Vec<dx12::Resource> = Vec::new();

        let mut command_allocators: Vec<dx12::CommandAllocator> = Vec::new();

        for ix in 0..FRAME_COUNT {
            let compute_target_resource = device.create_committed_resource(
                &compute_heap_properties,
                compute_resource_heap_flags,
                &compute_resource_desc,
                d3d12::D3D12_RESOURCE_STATE_COMMON,
                ptr::null(),
            );
            let render_target_resource = swap_chain3.get_buffer(ix);

            device.create_render_target_view(
                render_target_resource.clone(),
                ptr::null(),
                rt_cpu_descriptor,
            );

            rt_cpu_descriptor.ptr += rtv_descriptor_size as usize;

            compute_targets.push(compute_target_resource.clone());
            render_targets.push(render_target_resource.clone());

            command_allocators.push(device.create_command_allocator(list_type));
        }

        let fence = device.create_fence(0);

        (
            swap_chain3,
            device,
            compute_targets,
            render_targets,
            command_allocators,
            command_queue,
            compute_target_descriptor_heap,
            rtv_descriptor_heap,
            rtv_descriptor_size,
            fence,
        )
    }

    unsafe fn create_compute_pipeline_state(
        device: &dx12::Device,
        shader_code: &[u8],
        shader_compile_flags: minwindef::DWORD,
        compute_entry: String,
    ) -> (dx12::RootSignature, dx12::PipelineState) {
        // create 1 root parameter for compute
        let canvas_uav_descriptor_range = d3d12::D3D12_DESCRIPTOR_RANGE {
            RangeType: d3d12::D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
            NumDescriptors: 1,
            ..mem::zeroed()
        };
        let canvas_uav_table = d3d12::D3D12_ROOT_DESCRIPTOR_TABLE {
            NumDescriptorRanges: 1,
            pDescriptorRanges: &canvas_uav_descriptor_range as *const _,
        };
        let mut compute_root_parameter = d3d12::D3D12_ROOT_PARAMETER {
            ParameterType: d3d12::D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
            ShaderVisibility: d3d12::D3D12_SHADER_VISIBILITY_ALL,
            ..mem::zeroed()
        };
        *compute_root_parameter.u.DescriptorTable_mut() = canvas_uav_table;

        let compute_root_signature_desc = d3d12::D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: 1,
            pParameters: &compute_root_parameter as *const _,
            NumStaticSamplers: 0,
            pStaticSamplers: ptr::null(),
            Flags: d3d12::D3D12_ROOT_SIGNATURE_FLAG_NONE,
        };

        // serialize root signature description and create compute root signature
        let blob = dx12::RootSignature::serialize_description(
            &compute_root_signature_desc,
            d3d12::D3D_ROOT_SIGNATURE_VERSION_1,
        );
        let compute_root_signature = device.create_root_signature(0, blob);

        // load compute shader
        println!("compiling compute shader code...");
        let compute_shader_blob = dx12::ShaderByteCode::compile(
            shader_code,
            String::from("cs_5_1"),
            compute_entry,
            shader_compile_flags,
        );
        let compute_shader_bytecode = dx12::ShaderByteCode::from_blob(compute_shader_blob);

        // create compute pipeline state
        println!("creating compute pipeline state...");
        let compute_ps_desc = d3d12::D3D12_COMPUTE_PIPELINE_STATE_DESC {
            pRootSignature: compute_root_signature.0.as_raw(),
            CS: compute_shader_bytecode.bytecode,
            NodeMask: 0,
            CachedPSO: d3d12::D3D12_CACHED_PIPELINE_STATE {
                pCachedBlob: ptr::null(),
                CachedBlobSizeInBytes: 0,
            },
            Flags: d3d12::D3D12_PIPELINE_STATE_FLAG_NONE,
        };
        let compute_pipeline_state = device.create_compute_pipeline_state(&compute_ps_desc);

        (compute_root_signature, compute_pipeline_state)
    }

    unsafe fn create_graphics_pipeline_state(
        device: &dx12::Device,
        shader_code: &[u8],
        shader_compile_flags: minwindef::DWORD,
        vertex_entry: String,
        fragment_entry: String,
        screen_quad: Quad,
    ) -> (
        dx12::RootSignature,
        dx12::PipelineState,
        dx12::Resource,
        d3d12::D3D12_VERTEX_BUFFER_VIEW,
    ) {
        // create 1 root parameter for graphics
        let frag_shader_srv_descriptor_range = d3d12::D3D12_DESCRIPTOR_RANGE {
            RangeType: d3d12::D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
            NumDescriptors: 1,
            ..mem::zeroed()
        };
        let frag_shader_srv_table = d3d12::D3D12_ROOT_DESCRIPTOR_TABLE {
            NumDescriptorRanges: 1,
            pDescriptorRanges: &frag_shader_srv_descriptor_range as *const _,
        };
        let mut graphics_root_parameter = d3d12::D3D12_ROOT_PARAMETER {
            ParameterType: d3d12::D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
            ShaderVisibility: d3d12::D3D12_SHADER_VISIBILITY_PIXEL,
            ..mem::zeroed()
        };
        *graphics_root_parameter.u.DescriptorTable_mut() = frag_shader_srv_table;

        let graphics_root_signature_desc = d3d12::D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: 1,
            pParameters: &graphics_root_parameter as *const _,
            NumStaticSamplers: 0,
            pStaticSamplers: ptr::null(),
            Flags: d3d12::D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
        };

        // serialize root signature description and create graphics root signature
        let blob = dx12::RootSignature::serialize_description(
            &graphics_root_signature_desc,
            d3d12::D3D_ROOT_SIGNATURE_VERSION_1,
        );
        let graphics_root_signature = device.create_root_signature(0, blob);

        let vertices = screen_quad.as_vertices();
        let vertex_buffer_stride = mem::size_of::<Vertex>();
        let vertex_buffer_size = vertex_buffer_stride * vertices.len();
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

        let vertex_buffer = device.create_committed_resource(
            &vertex_buffer_heap_properties,
            d3d12::D3D12_HEAP_FLAG_NONE,
            &vertex_buffer_resource_description,
            d3d12::D3D12_RESOURCE_STATE_GENERIC_READ,
            ptr::null(),
        );
        vertex_buffer.upload_data_to_resource(vertex_buffer_size, vertices.as_ptr());
        let vertex_buffer_view = d3d12::D3D12_VERTEX_BUFFER_VIEW {
            BufferLocation: vertex_buffer.get_gpu_virtual_address(),
            SizeInBytes: vertex_buffer_size as u32,
            StrideInBytes: vertex_buffer_stride as u32,
        };

        // load graphics shaders from byte string
        println!("compiling vertex shader code...");
        let graphics_vertex_shader_blob = dx12::ShaderByteCode::compile(
            shader_code,
            String::from("vs_5_1"),
            vertex_entry,
            shader_compile_flags,
        );
        let graphics_vertex_shader_bytecode =
            dx12::ShaderByteCode::from_blob(graphics_vertex_shader_blob);

        println!("compiling fragment shader code...");
        let graphics_fragment_shader_blob = dx12::ShaderByteCode::compile(
            shader_code,
            String::from("ps_5_1"),
            fragment_entry,
            shader_compile_flags,
        );
        let graphics_fragment_shader_bytecode =
            dx12::ShaderByteCode::from_blob(graphics_fragment_shader_blob);

        // load graphics shaders from file
        //        println!("compiling vertex shader code...");
        //        let graphics_vertex_shader_blob= dx12::ShaderByteCode::compile_from_file(
        //            String::from("A:\\piet-dx12\\resources\\shaders.hlsl"),
        //            String::from("vs_5_0"),
        //            vertex_entry,
        //            flags,
        //        );
        //        let graphics_vertex_shader_bytecode =
        //            dx12::ShaderByteCode::from_blob(graphics_vertex_shader_blob);
        //        println!("compiling fragment shader code...");
        //        let graphics_fragment_shader_blob= dx12::ShaderByteCode::compile_from_file(
        //            String::from("A:\\piet-dx12\\resources\\shaders.hlsl"),
        //            String::from("ps_5_0"),
        //            fragment_entry,
        //            flags,
        //        );
        //        let graphics_fragment_shader_bytecode =
        //            dx12::ShaderByteCode::from_blob(graphics_fragment_shader_blob);

        // create graphics pipeline state
        let position_ied = dx12::InputElementDesc {
            semantic_name: String::from("POSITION"),
            semantic_index: 0,
            format: dxgiformat::DXGI_FORMAT_R32G32B32_FLOAT,
            input_slot: 0,
            aligned_byte_offset: 0,
            input_slot_class: d3d12::D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            instance_data_step_rate: 0,
        };

        let ieds = [position_ied.as_winapi_struct()];

        let graphics_ps_desc = d3d12::D3D12_GRAPHICS_PIPELINE_STATE_DESC {
            pRootSignature: graphics_root_signature.0.as_raw(),
            VS: graphics_vertex_shader_bytecode.bytecode,
            PS: graphics_fragment_shader_bytecode.bytecode,
            DS: dx12::ShaderByteCode::empty().bytecode,
            HS: dx12::ShaderByteCode::empty().bytecode,
            GS: dx12::ShaderByteCode::empty().bytecode,
            StreamOutput: d3d12::D3D12_STREAM_OUTPUT_DESC {
                pSODeclaration: ptr::null(),
                NumEntries: 0,
                pBufferStrides: ptr::null(),
                NumStrides: 0,
                RasterizedStream: 0,
            },
            //TODO: confirm do nothing blend desc is correct
            BlendState: dx12::default_blend_desc(),
            SampleMask: std::u32::MAX,
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
            InputLayout: d3d12::D3D12_INPUT_LAYOUT_DESC {
                pInputElementDescs: ieds.as_ptr(),
                NumElements: ieds.len() as u32,
            },
            IBStripCutValue: 0,
            PrimitiveTopologyType: d3d12::D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            NumRenderTargets: 1,
            RTVFormats: [
                winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
                winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN,
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
        println!("creating graphics pipeline state...");
        let graphics_pipeline_state = device.create_graphics_pipeline_state(&graphics_ps_desc);

        (
            graphics_root_signature,
            graphics_pipeline_state,
            vertex_buffer,
            vertex_buffer_view,
        )
    }

    unsafe fn create_pipeline_states(
        device: &dx12::Device,
        shader_code: &[u8],
        compute_entry: String,
        vertex_entry: String,
        fragment_entry: String,
        command_allocators: &Vec<dx12::CommandAllocator>,
        screen_quad: Quad,
    ) -> (
        dx12::RootSignature,
        dx12::PipelineState,
        dx12::RootSignature,
        dx12::PipelineState,
        dx12::Resource,
        d3d12::D3D12_VERTEX_BUFFER_VIEW,
        dx12::GraphicsCommandList,
    ) {
        #[cfg(not(debug_assertions))]
        let shader_compile_flags: minwindef::DWORD = 0;

        #[cfg(debug_assertions)]
        let shader_compile_flags: minwindef::DWORD = winapi::um::d3dcompiler::D3DCOMPILE_DEBUG
            | winapi::um::d3dcompiler::D3DCOMPILE_SKIP_OPTIMIZATION;

        let (compute_root_signature, compute_pipeline_state) =
            GpuState::create_compute_pipeline_state(
                device,
                shader_code,
                shader_compile_flags,
                compute_entry,
            );

        let (graphics_root_signature, graphics_pipeline_state, vertex_buffer, vertex_buffer_view) =
            GpuState::create_graphics_pipeline_state(
                device,
                shader_code,
                shader_compile_flags,
                vertex_entry,
                fragment_entry,
                screen_quad,
            );

        // create command list
        let command_list = device.create_graphics_command_list(
            d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT,
            command_allocators[0].clone(),
            compute_pipeline_state.clone(),
            0,
        );

        command_list.close();

        (
            compute_root_signature,
            compute_pipeline_state,
            graphics_root_signature,
            graphics_pipeline_state,
            vertex_buffer,
            vertex_buffer_view,
            command_list,
        )
    }
}
