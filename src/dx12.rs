extern crate winapi;
extern crate wio;

use std::{ffi, mem, ptr};
use winapi::shared::{dxgi, dxgi1_2, dxgi1_3, dxgi1_4, minwindef, windef, winerror, dxgiformat};
use winapi::um::{d3d12, d3dcommon, synchapi, winnt, d3d12sdklayers, dxgidebug};
use winapi::Interface;
use wio::com::ComPtr;

// everything is ripped from d3d12-rs, but wio::com::ComPtr, and winapi are used more directly

pub type D3DResult<T> = (T, winerror::HRESULT);

#[derive(Clone)]
pub struct Heap(pub ComPtr<d3d12::ID3D12Heap>);
pub type Subresource = u32;
#[derive(Clone)]
pub struct Resource(pub ComPtr<d3d12::ID3D12Resource>);
pub struct VertexBufferView(pub ComPtr<d3d12::D3D12_VERTEX_BUFFER_VIEW>);

#[derive(Clone)]
pub struct Adapter1(pub ComPtr<dxgi::IDXGIAdapter1>);
#[derive(Clone)]
pub struct Factory2(pub ComPtr<dxgi1_2::IDXGIFactory2>);
#[derive(Clone)]
pub struct Factory4(pub ComPtr<dxgi1_4::IDXGIFactory4>);
#[derive(Clone)]
pub struct SwapChain(pub ComPtr<dxgi::IDXGISwapChain>);
#[derive(Clone)]
pub struct SwapChain1(pub ComPtr<dxgi1_2::IDXGISwapChain1>);
#[derive(Clone)]
pub struct SwapChain3(pub ComPtr<dxgi1_4::IDXGISwapChain3>);

#[derive(Clone)]
pub struct QueryHeap(pub ComPtr<d3d12::ID3D12QueryHeap>);

#[derive(Clone)]
pub struct Device(pub ComPtr<d3d12::ID3D12Device>);

#[derive(Clone)]
pub struct CommandQueue(pub ComPtr<d3d12::ID3D12CommandQueue>);

#[derive(Clone)]
pub struct CommandAllocator(pub ComPtr<d3d12::ID3D12CommandAllocator>);

pub type CpuDescriptor = d3d12::D3D12_CPU_DESCRIPTOR_HANDLE;
pub type GpuDescriptor = d3d12::D3D12_GPU_DESCRIPTOR_HANDLE;

#[derive(Clone)]
pub struct DescriptorHeap(pub ComPtr<d3d12::ID3D12DescriptorHeap>);

pub type TextureAddressMode = [d3d12::D3D12_TEXTURE_ADDRESS_MODE; 3];

#[derive(Clone)]
pub struct RootSignature(pub ComPtr<d3d12::ID3D12RootSignature>);

#[derive(Clone)]
pub struct CommandSignature(pub ComPtr<d3d12::ID3D12CommandSignature>);
#[derive(Clone)]
pub struct CommandList(pub ComPtr<d3d12::ID3D12CommandList>);
#[derive(Clone)]
pub struct GraphicsCommandList(pub ComPtr<d3d12::ID3D12GraphicsCommandList>);

#[derive(Clone)]
pub struct Event(pub winnt::HANDLE);
#[derive(Clone)]
pub struct Fence(pub ComPtr<d3d12::ID3D12Fence>);

#[derive(Clone)]
pub struct PipelineState(pub ComPtr<d3d12::ID3D12PipelineState>);

#[derive(Clone)]
pub struct CachedPSO(d3d12::D3D12_CACHED_PIPELINE_STATE);

#[derive(Clone)]
pub struct Blob(pub ComPtr<d3dcommon::ID3DBlob>);

#[derive(Clone)]
pub struct ShaderByteCode {
    pub bytecode: d3d12::D3D12_SHADER_BYTECODE,
    blob: Option<Blob>,
}

pub struct DebugController(pub d3d12sdklayers::ID3D12Debug);

pub fn error_if_failed_else_value<T>(result: D3DResult<T>) -> Result<T, winerror::HRESULT> {
    let (result_value, hresult) = result;

    if winerror::SUCCEEDED(hresult) {
        Ok(result_value)
    } else {
        Err(hresult)
    }
}

pub fn error_if_failed_else_none(hresult: winerror::HRESULT) -> Result<(), winerror::HRESULT> {
    if winerror::SUCCEEDED(hresult) {
        Ok(())
    } else {
        Err(hresult)
    }
}

impl Resource {
    pub unsafe fn upload_data_to_resource<T>(&self, count: usize, data: *const T) {
        let mut mapped_memory = ptr::null_mut();
        let zero_range = d3d12::D3D12_RANGE {..mem::zeroed()};
        error_if_failed_else_none(self.0.Map(0, &zero_range as *const _, &mut mapped_memory as *mut _ as *mut _)).expect("could not get pointer to mapped memory");
        ptr::copy(data, mapped_memory, count);
        self.0.Unmap(0, ptr::null());
    }

    pub unsafe fn get_gpu_virtual_address(&self) -> d3d12::D3D12_GPU_VIRTUAL_ADDRESS {
        self.0.GetGPUVirtualAddress()
    }
}

impl Factory4 {
    pub unsafe fn create(flags: minwindef::UINT) -> Factory4 {
        let mut factory = ptr::null_mut();

        error_if_failed_else_none(dxgi1_3::CreateDXGIFactory2(
            0,
            &dxgi1_4::IDXGIFactory4::uuidof(),
            &mut factory as *mut _ as *mut _,
        ))
        .expect("could not create factory4");

        Factory4(ComPtr::from_raw(factory))
    }

    pub unsafe fn enumerate_adapters(
        &self,
        id: u32,
    ) -> (*mut dxgi::IDXGIAdapter1, winerror::HRESULT) {
        let mut adapter = ptr::null_mut();
        let hr = self.0.EnumAdapters1(id, &mut adapter as *mut _ as *mut _);

        (adapter, hr)
    }

    pub unsafe fn create_swapchain_for_hwnd(
        &self,
        command_queue: CommandQueue,
        hwnd: windef::HWND,
        desc: dxgi1_2::DXGI_SWAP_CHAIN_DESC1,
    ) -> SwapChain3 {
        let mut swap_chain = ptr::null_mut();
        error_if_failed_else_none(self.0.CreateSwapChainForHwnd(
            command_queue.0.as_raw() as *mut _,
            hwnd,
            &desc,
            ptr::null(),
            ptr::null_mut(),
            &mut swap_chain as *mut _ as *mut _,
        ))
            .expect("could not creation swapchain for hwnd");

        SwapChain3(ComPtr::from_raw(swap_chain))
    }
}

impl CommandQueue {
    pub unsafe fn signal(&self, fence: Fence, value: u64) -> winerror::HRESULT {
        self.0.Signal(fence.0.as_raw(), value)
    }

    pub unsafe fn execute_command_lists(
        &self,
        num_command_lists: u32,
        command_lists: &[*mut d3d12::ID3D12CommandList],
    ) {
        self.0
            .ExecuteCommandLists(num_command_lists, command_lists.as_ptr());
    }
}

impl SwapChain {
    pub unsafe fn get_buffer(&self, id: u32) -> Resource {
        let mut resource = ptr::null_mut();
        error_if_failed_else_none(self.0.GetBuffer(
            id,
            &d3d12::ID3D12Resource::uuidof(),
            &mut resource as *mut _ as *mut _,
        ))
        .expect("SwapChain could not get buffer");

        Resource(ComPtr::from_raw(resource))
    }

    // TODO: present flags
    pub unsafe fn present(&self, interval: u32, flags: u32) -> winerror::HRESULT {
        self.0.Present(interval, flags)
    }
}

impl SwapChain1 {
    pub unsafe fn cast_into_swap_chain3(&self) -> SwapChain3 {
        SwapChain3(
            self.0
                .cast::<dxgi1_4::IDXGISwapChain3>()
                .expect("could not cast into SwapChain3"),
        )
    }

    pub unsafe fn get_buffer(&self, id: u32) -> Resource {
        let mut resource = ptr::null_mut();
        error_if_failed_else_none(self.0.GetBuffer(
            id,
            &d3d12::ID3D12Resource::uuidof(),
            &mut resource as *mut _ as *mut _,
        ))
        .expect("SwapChain1 could not get buffer");

        Resource(ComPtr::from_raw(resource))
    }
}

impl SwapChain3 {
    pub unsafe fn get_buffer(&self, id: u32) -> Resource {
        let mut resource = ptr::null_mut();
        error_if_failed_else_none(self.0.GetBuffer(
            id,
            &d3d12::ID3D12Resource::uuidof(),
            &mut resource as *mut _ as *mut _,
        ))
        .expect("SwapChain3 could not get buffer");

        Resource(ComPtr::from_raw(resource))
    }

    pub unsafe fn get_current_back_buffer_index(&self) -> u32 {
        self.0.GetCurrentBackBufferIndex()
    }

    pub unsafe fn present(&self, interval: u32, flags: u32) {
        println!("  asking swapchain to present...");
        error_if_failed_else_none(self.0.Present1(interval, flags, &dxgi1_2::DXGI_PRESENT_PARAMETERS{ ..mem::zeroed() } as *const _)).expect("could not present to swapchain");
        println!("  present successful.");
    }
}

impl Device {
    pub unsafe fn create_device(factory4: &Factory4) -> Result<Device, Vec<winerror::HRESULT>> {
        let mut id = 0;
        let mut errors: Vec<winerror::HRESULT> = Vec::new();

        loop {
            let adapter = {
                let (adapter, hr) = factory4.enumerate_adapters(id);

                if !winerror::SUCCEEDED(hr) {
                    errors.push(hr);
                    return Err(errors);
                }

                ComPtr::from_raw(adapter)
            };

            id += 1;

            let (device, hr) =
                Device::create_using_adapter(adapter.clone(), d3dcommon::D3D_FEATURE_LEVEL_12_0);

            if !winerror::SUCCEEDED(hr) {
                errors.push(hr);
                continue;
            } else {
                std::mem::drop(adapter);
                return Ok(Device(ComPtr::from_raw(device)));
            }
        }
    }

    pub unsafe fn create_using_adapter<I: Interface>(
        adapter: ComPtr<I>,
        feature_level: d3dcommon::D3D_FEATURE_LEVEL,
    ) -> (*mut d3d12::ID3D12Device, winerror::HRESULT) {
        let mut device = ptr::null_mut();
        let hr = d3d12::D3D12CreateDevice(
            adapter.as_raw() as *mut _,
            feature_level as _,
            &d3d12::ID3D12Device::uuidof(),
            &mut device as *mut _ as *mut _,
        );

        (device, hr)
    }

    pub unsafe fn create_command_allocator(
        &self,
        list_type: d3d12::D3D12_COMMAND_LIST_TYPE,
    ) -> CommandAllocator {
        let mut allocator = ptr::null_mut();
        error_if_failed_else_none(self.0.CreateCommandAllocator(
            list_type,
            &d3d12::ID3D12CommandAllocator::uuidof(),
            &mut allocator as *mut _ as *mut _,
        ))
        .expect("device could nto create command allocator");

        CommandAllocator(ComPtr::from_raw(allocator))
    }

    pub unsafe fn create_command_queue(
        &self,
        list_type: d3d12::D3D12_COMMAND_LIST_TYPE,
        priority: minwindef::INT,
        flags: d3d12::D3D12_COMMAND_QUEUE_FLAGS,
        node_mask: minwindef::UINT,
    ) -> CommandQueue {
        let desc = d3d12::D3D12_COMMAND_QUEUE_DESC {
            Type: list_type,
            Priority: priority,
            Flags: flags,
            NodeMask: node_mask,
        };

        let mut cmd_q = ptr::null_mut();
        error_if_failed_else_none(self.0.CreateCommandQueue(
            &desc,
            &d3d12::ID3D12CommandQueue::uuidof(),
            &mut cmd_q as *mut _ as *mut _,
        ))
        .expect("device could not create command queue");

        CommandQueue(ComPtr::from_raw(cmd_q))
    }

    pub unsafe fn create_descriptor_heap(
        &self,
        heap_description: &d3d12::D3D12_DESCRIPTOR_HEAP_DESC,
    ) -> DescriptorHeap {
        let mut heap = ptr::null_mut();
        error_if_failed_else_none(self.0.CreateDescriptorHeap(
            heap_description,
            &d3d12::ID3D12DescriptorHeap::uuidof(),
            &mut heap as *mut _ as *mut _,
        ))
        .expect("device could not create descriptor heap");

        DescriptorHeap(ComPtr::from_raw(heap))
    }

    pub unsafe fn get_descriptor_increment_size(
        &self,
        heap_type: d3d12::D3D12_DESCRIPTOR_HEAP_TYPE,
    ) -> u32 {
        self.0.GetDescriptorHandleIncrementSize(heap_type)
    }

    pub unsafe fn create_graphics_pipeline_state(
        &self,
        graphics_pipeline_desc: &d3d12::D3D12_GRAPHICS_PIPELINE_STATE_DESC,
    ) -> PipelineState {
        let mut pipeline_state = ptr::null_mut();

        error_if_failed_else_none(self.0.CreateGraphicsPipelineState(
            graphics_pipeline_desc as *const _,
            &d3d12::ID3D12PipelineState::uuidof(),
            &mut pipeline_state as *mut _ as *mut _,
        ))
        .expect("device could not create graphics pipeline state");

        PipelineState(ComPtr::from_raw(pipeline_state))
    }

    pub unsafe fn create_compute_pipeline_state(
        &self,
        compute_pipeline_desc: &d3d12::D3D12_COMPUTE_PIPELINE_STATE_DESC,
    ) -> PipelineState {
        let mut pipeline_state = ptr::null_mut();

        error_if_failed_else_none(self.0.CreateComputePipelineState(
            compute_pipeline_desc as *const _,
            &d3d12::ID3D12PipelineState::uuidof(),
            &mut pipeline_state as *mut _ as *mut _,
        ))
        .expect("device could not create compute pipeline state");

        PipelineState(ComPtr::from_raw(pipeline_state))
    }

    pub unsafe fn create_root_signature(
        &self,
        node_mask: minwindef::UINT,
        blob: Blob,
    ) -> RootSignature {
        let mut signature = ptr::null_mut();
        error_if_failed_else_none(self.0.CreateRootSignature(
            node_mask,
            blob.0.GetBufferPointer(),
            blob.0.GetBufferSize(),
            &d3d12::ID3D12RootSignature::uuidof(),
            &mut signature as *mut _ as *mut _,
        ))
        .expect("device could not create root signature");

        RootSignature(ComPtr::from_raw(signature))
    }

    pub unsafe fn create_command_signature(
        &self,
        root_signature: RootSignature,
        arguments: &[d3d12::D3D12_INDIRECT_ARGUMENT_DESC],
        stride: u32,
        node_mask: minwindef::UINT,
    ) -> CommandSignature {
        let mut signature = ptr::null_mut();
        let desc = d3d12::D3D12_COMMAND_SIGNATURE_DESC {
            ByteStride: stride,
            NumArgumentDescs: arguments.len() as _,
            pArgumentDescs: arguments.as_ptr() as *const _,
            NodeMask: node_mask,
        };

        error_if_failed_else_none(self.0.CreateCommandSignature(
            &desc,
            root_signature.0.as_raw(),
            &d3d12::ID3D12CommandSignature::uuidof(),
            &mut signature as *mut _ as *mut _,
        ))
        .expect("device could not create command signature");

        CommandSignature(ComPtr::from_raw(signature))
    }

    pub unsafe fn create_graphics_command_list(
        &self,
        list_type: d3d12::D3D12_COMMAND_LIST_TYPE,
        allocator: CommandAllocator,
        initial_ps: PipelineState,
        node_mask: minwindef::UINT,
    ) -> GraphicsCommandList {
        let mut command_list = ptr::null_mut();

        error_if_failed_else_none(self.0.CreateCommandList(
            node_mask,
            list_type,
            allocator.0.as_raw(),
            initial_ps.0.as_raw(),
            &d3d12::ID3D12GraphicsCommandList::uuidof(),
            &mut command_list as *mut _ as *mut _,
        ))
        .expect("device could not create graphics command list");

        GraphicsCommandList(ComPtr::from_raw(command_list))
    }

    pub unsafe fn create_unordered_access_view(
        &self,
        resource: Resource,
        descriptor: CpuDescriptor,
    ) {
        self.0.CreateUnorderedAccessView(
            resource.0.as_raw(),
            ptr::null_mut(),
            ptr::null(),
            descriptor,
        )
    }

    pub unsafe fn create_render_target_view(
        &self,
        resource: Resource,
        desc: *const d3d12::D3D12_RENDER_TARGET_VIEW_DESC,
        descriptor: CpuDescriptor,
    ) {
        self.0
            .CreateRenderTargetView(resource.0.as_raw(), desc, descriptor);
    }

    // TODO: interface not complete
    pub unsafe fn create_fence(&self, initial: u64) -> Fence {
        let mut fence = ptr::null_mut();
        error_if_failed_else_none(self.0.CreateFence(
            initial,
            d3d12::D3D12_FENCE_FLAG_NONE,
            &d3d12::ID3D12Fence::uuidof(),
            &mut fence as *mut _ as *mut _,
        ))
        .expect("device could not create fence");

        Fence(ComPtr::from_raw(fence))
    }

    pub unsafe fn create_committed_resource(
        &self,
        heap_properties: &d3d12::D3D12_HEAP_PROPERTIES,
        flags: d3d12::D3D12_HEAP_FLAGS,
        resource_description: &d3d12::D3D12_RESOURCE_DESC,
        initial_resource_state: d3d12::D3D12_RESOURCE_STATES,
        optimized_clear_value: *const d3d12::D3D12_CLEAR_VALUE,
    ) -> Resource {
        let mut resource = ptr::null_mut();

        error_if_failed_else_none(self.0.CreateCommittedResource(
            heap_properties as *const _,
            flags,
            resource_description as *const _,
            initial_resource_state,
            optimized_clear_value,
            &d3d12::ID3D12Resource::uuidof(),
            &mut resource as *mut _ as *mut _,
        ))
        .expect("device could not create committed resource");

        Resource(ComPtr::from_raw(resource))
    }
}

impl CommandAllocator {
    pub unsafe fn reset(&self) {
        self.0.Reset();
    }
}

impl DescriptorHeap {
    pub unsafe fn start_cpu_descriptor(&self) -> CpuDescriptor {
        self.0.GetCPUDescriptorHandleForHeapStart()
    }

    pub unsafe fn start_gpu_descriptor(&self) -> GpuDescriptor {
        self.0.GetGPUDescriptorHandleForHeapStart()
    }
}

#[repr(transparent)]
pub struct DescriptorRange(d3d12::D3D12_DESCRIPTOR_RANGE);
impl DescriptorRange {}

impl RootSignature {
    pub unsafe fn serialize_description(
        desc: &d3d12::D3D12_ROOT_SIGNATURE_DESC,
        version: d3d12::D3D_ROOT_SIGNATURE_VERSION,
    ) -> Blob {
        let mut blob = ptr::null_mut();
        //TODO: properly use error blob
        let mut _error = ptr::null_mut();

        error_if_failed_else_none(d3d12::D3D12SerializeRootSignature(
            desc as *const _,
            version,
            &mut blob as *mut _ as *mut _,
            &mut _error as *mut _ as *mut _,
        )).expect("could not serialize root signature description");

        Blob(ComPtr::from_raw(blob))
    }
}

impl ShaderByteCode {
    // empty byte code
    pub unsafe fn empty() -> ShaderByteCode {
        ShaderByteCode {
            bytecode: d3d12::D3D12_SHADER_BYTECODE {
                BytecodeLength: 0,
                pShaderBytecode: ptr::null(),
            },
            blob: None,
        }
    }

    // `blob` may not be null.
    pub unsafe fn from_blob(blob: Blob) -> ShaderByteCode {
        ShaderByteCode {
            bytecode: d3d12::D3D12_SHADER_BYTECODE {
                BytecodeLength: blob.0.GetBufferSize(),
                pShaderBytecode: blob.0.GetBufferPointer(),
            },
            blob: Some(blob),
        }
    }

    /// Compile a shader from raw HLSL.
    ///
    /// * `target`: example format: `ps_5_1`.
    pub unsafe fn compile(
        code: &[u8],
        target: String,
        entry: String,
        flags: minwindef::DWORD,
    ) -> Blob {
        let mut shader = ptr::null_mut();
        //TODO: use error blob properly
        let mut _error = ptr::null_mut();

        let target = ffi::CString::new(target)
            .expect("could not convert target format string into ffi::CString");
        let entry = ffi::CString::new(entry)
            .expect("could not convert entry name String into ffi::CString");

        error_if_failed_else_none(winapi::um::d3dcompiler::D3DCompile(
            code.as_ptr() as *const _,
            code.len(),
            ptr::null(), // defines
            ptr::null(), // include
            ptr::null_mut(),
            entry.as_ptr() as *const _,
            target.as_ptr() as *const _,
            flags,
            0,
            &mut shader as *mut _ as *mut _,
            &mut _error as *mut _ as *mut _,
        ))
        .expect("could not compile shader code");

        Blob(ComPtr::from_raw(shader))
    }

    pub unsafe fn compile_from_file(
        file_path: String,
        target: String,
        entry: String,
        flags: minwindef::DWORD,
    ) -> Blob {
        let mut shader = ptr::null_mut();
        //TODO: use error blob properly
        let mut _error = ptr::null_mut();

        let target = ffi::CString::new(target)
            .expect("could not convert target format string into ffi::CString");
        let entry = ffi::CString::new(entry)
            .expect("could not convert entry name String into ffi::CString");

        error_if_failed_else_none(winapi::um::d3dcompiler::D3DCompileFromFile(
            file_path.as_ptr() as *const _,
            ptr::null(),
            ptr::null_mut(),
            entry.as_ptr() as *const _,
            target.as_ptr() as *const _,
            flags,
            0,
            &mut shader as *mut _ as *mut _,
            &mut _error as *mut _ as *mut _,
        ))
            .expect("could not compile shader code");

        Blob(ComPtr::from_raw(shader))
    }
}

impl Fence {
    pub unsafe fn set_event_on_completion(&self, event: Event, value: u64) -> winerror::HRESULT {
        self.0.SetEventOnCompletion(value, event.0)
    }

    pub unsafe fn get_value(&self) -> u64 {
        self.0.GetCompletedValue()
    }

    pub unsafe fn signal(&self, value: u64) -> winerror::HRESULT {
        self.0.Signal(value)
    }
}

impl Event {
    pub unsafe fn create(manual_reset: bool, initial_state: bool) -> Self {
        Event(synchapi::CreateEventA(
            ptr::null_mut(),
            manual_reset as _,
            initial_state as _,
            ptr::null(),
        ))
    }

    pub unsafe fn wait(&self, timeout_ms: u32) -> u32 {
        synchapi::WaitForSingleObject(self.0, timeout_ms)
    }
}

impl GraphicsCommandList {
    pub unsafe fn as_raw_list(&self) -> CommandList {
        CommandList(ComPtr::from_raw(self.0.as_raw() as *mut _))
    }

    pub unsafe fn close(&self) -> winerror::HRESULT {
        self.0.Close()
    }

    pub unsafe fn reset(
        &self,
        allocator: CommandAllocator,
        initial_pso: PipelineState,
    )  {
        assert!(!allocator.0.is_null());
        error_if_failed_else_none(self.0.Reset(allocator.0.as_raw(), initial_pso.0.as_raw())).expect("could not reset command list");
    }

    pub unsafe fn set_compute_root_signature(&self, signature: RootSignature) {
        self.0.SetComputeRootSignature(signature.0.as_raw());
    }

    pub unsafe fn set_graphics_root_signature(&self, signature: RootSignature) {
        self.0.SetGraphicsRootSignature(signature.0.as_raw());
    }

    pub unsafe fn set_resource_barrier(
        &self,
        num_barriers: u32,
        resource_barriers: *const d3d12::D3D12_RESOURCE_BARRIER,
    ) {
        self.0.ResourceBarrier(num_barriers, resource_barriers);
    }

    pub unsafe fn set_viewport(&self, viewport: &d3d12::D3D12_VIEWPORT) {
        self.0.RSSetViewports(1, viewport as *const _);
    }

    pub unsafe fn set_scissor_rect(&self, scissor_rect: &d3d12::D3D12_RECT) {
        self.0.RSSetScissorRects(1, scissor_rect as *const _);
    }

    pub unsafe fn dispatch(&self, count_x: u32, count_y: u32, count_z: u32) {
        self.0.Dispatch(count_x, count_y, count_z);
    }

    pub unsafe fn draw_instanced(
        &self,
        num_vertices: u32,
        num_instances: u32,
        start_vertex: u32,
        start_instance: u32,
    ) {
        self.0
            .DrawInstanced(num_vertices, num_instances, start_vertex, start_instance);
    }

    pub unsafe fn set_pipeline_state(&self, pipeline_state: PipelineState) {
        self.0.SetPipelineState(pipeline_state.0.as_raw());
    }

    pub unsafe fn set_compute_root_unordered_access_view(
        &self,
        root_parameter_index: u32,
        buffer_location: d3d12::D3D12_GPU_VIRTUAL_ADDRESS,
    ) {
        self.0
            .SetComputeRootUnorderedAccessView(root_parameter_index, buffer_location);
    }

    pub unsafe fn set_graphics_root_shader_resource_view(
        &self,
        root_parameter_index: u32,
        buffer_location: d3d12::D3D12_GPU_VIRTUAL_ADDRESS,
    ) {
        self.0
            .SetGraphicsRootShaderResourceView(root_parameter_index, buffer_location);
    }

    pub unsafe fn set_render_target(
        &self,
        render_target_descriptor: d3d12::D3D12_CPU_DESCRIPTOR_HANDLE,
    ) {
        self.0.OMSetRenderTargets(
            1,
            &render_target_descriptor as *const _,
            false as _,
            ptr::null(),
        );
    }

    pub unsafe fn clear_render_target_view(&self, render_target_descriptor: d3d12::D3D12_CPU_DESCRIPTOR_HANDLE, clear_color: &[f32; 4]) {
        self.0.ClearRenderTargetView(render_target_descriptor, clear_color as *const _, 0, ptr::null());
    }

    pub unsafe fn set_primitive_topology(&self, primitive_topology: d3dcommon::D3D_PRIMITIVE_TOPOLOGY) {
        self.0.IASetPrimitiveTopology(primitive_topology);
    }

    pub unsafe fn set_vertex_buffer(&self, start_slot: u32, num_views: u32, vertex_buffer_view: &d3d12::D3D12_VERTEX_BUFFER_VIEW) {
        self.0.IASetVertexBuffers(start_slot, num_views, vertex_buffer_view as *const _);
    }
}

pub fn default_render_target_blend_desc() -> d3d12::D3D12_RENDER_TARGET_BLEND_DESC {
    d3d12::D3D12_RENDER_TARGET_BLEND_DESC {
        BlendEnable: minwindef::FALSE,
        LogicOpEnable: minwindef::FALSE,
        SrcBlend: d3d12::D3D12_BLEND_ONE,
        DestBlend: d3d12::D3D12_BLEND_ZERO,
        // enum variant 0
        BlendOp: d3d12::D3D12_BLEND_OP_ADD,
        SrcBlendAlpha: d3d12::D3D12_BLEND_ONE,
        DestBlendAlpha: d3d12::D3D12_BLEND_ZERO,
        BlendOpAlpha: d3d12::D3D12_BLEND_OP_ADD,
        // enum variant 0
        LogicOp: d3d12::D3D12_LOGIC_OP_NOOP,
        RenderTargetWriteMask: d3d12::D3D12_COLOR_WRITE_ENABLE_ALL as u8,
    }
}

pub fn default_blend_desc() -> d3d12::D3D12_BLEND_DESC {
    // see default description here: https://docs.microsoft.com/en-us/windows/win32/direct3d12/cd3dx12-blend-desc
    d3d12::D3D12_BLEND_DESC {
        AlphaToCoverageEnable: minwindef::FALSE,
        IndependentBlendEnable: minwindef::FALSE,
        RenderTarget: [
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
        ],
    }
}

pub unsafe fn create_transition_resource_barrier(
    resource: *mut d3d12::ID3D12Resource,
    state_before: d3d12::D3D12_RESOURCE_STATES,
    state_after: d3d12::D3D12_RESOURCE_STATES,
) -> d3d12::D3D12_RESOURCE_BARRIER {
    let transition = d3d12::D3D12_RESOURCE_TRANSITION_BARRIER {
        pResource: resource,
        Subresource: d3d12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
        StateBefore: state_before,
        StateAfter: state_after,
    };

    let mut resource_barrier: d3d12::D3D12_RESOURCE_BARRIER = mem::zeroed();
    resource_barrier.Type = d3d12::D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;
    resource_barrier.Flags = d3d12::D3D12_RESOURCE_BARRIER_FLAG_NONE;
    *resource_barrier.u.Transition_mut() = transition;

    resource_barrier
}

pub unsafe fn enable_debug_layer() {
    let mut debug_controller: *mut d3d12sdklayers::ID3D12Debug1 = ptr::null_mut();
    error_if_failed_else_none(d3d12::D3D12GetDebugInterface(
        &d3d12sdklayers::ID3D12Debug1::uuidof(),
        &mut debug_controller as *mut _ as *mut _,
    )).expect("could not create debug controller");

    (*debug_controller).EnableDebugLayer();

//    let mut queue = ptr::null_mut();
//    let hr =
//        dxgi1_3::DXGIGetDebugInterface1(
//            0,
//            &dxgidebug::IDXGIInfoQueue::uuidof(),
//            &mut queue as *mut _ as *mut _,
//        );
//
//    if winerror::SUCCEEDED(hr) {
//        (*debug_controller).SetEnableGPUBasedValidation(minwindef::TRUE);
//    }

    (*debug_controller).Release();
}

pub struct InputElementDesc {
    pub semantic_name: String,
    pub semantic_index: u32,
    pub format: dxgiformat::DXGI_FORMAT,
    pub input_slot: u32,
    pub aligned_byte_offset: u32,
    pub input_slot_class: d3d12::D3D12_INPUT_CLASSIFICATION,
    pub instance_data_step_rate: u32,
}

impl InputElementDesc {
    pub fn as_winapi_struct(&self) -> d3d12::D3D12_INPUT_ELEMENT_DESC {
        d3d12::D3D12_INPUT_ELEMENT_DESC {
            SemanticName: std::ffi::CString::new(self.semantic_name.as_str()).unwrap().into_raw() as *const _,
            SemanticIndex: self.semantic_index,
            Format: self.format,
            InputSlot: self.input_slot,
            AlignedByteOffset: self.aligned_byte_offset,
            InputSlotClass: self.input_slot_class,
            InstanceDataStepRate: self.instance_data_step_rate,
        }
    }
}

