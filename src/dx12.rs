extern crate winapi;
extern crate wio;

use std::{ffi, ptr};
use winapi::shared::{dxgi, dxgi1_2, dxgi1_3, dxgi1_4, minwindef, windef, winerror};
use winapi::um::{d3d12, d3dcommon, synchapi, winnt};
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
pub struct ErrorBlob(pub ComPtr<d3dcommon::ID3DBlob>);

#[derive(Copy, Clone)]
pub struct ShaderByteCode(pub d3d12::D3D12_SHADER_BYTECODE);

pub fn error_if_failed_else_value<T>(result: D3DResult<T>) -> Result<T, winerror::HRESULT> {
    let (result_value, hresult) = result;

    if winerror::SUCCEEDED(hresult) {
        Ok(result_value)
    } else {
        Err(hresult)
    }
}

impl Resource {}

impl Factory2 {
    // TODO: interface not complete
    pub unsafe fn create_swapchain_for_hwnd(
        &self,
        queue: CommandQueue,
        hwnd: windef::HWND,
        desc: dxgi1_2::DXGI_SWAP_CHAIN_DESC1,
    ) -> D3DResult<SwapChain1> {
        let mut swap_chain = ptr::null_mut();
        let hr = unsafe {
            self.0.CreateSwapChainForHwnd(
                queue.0.as_raw() as *mut _,
                hwnd,
                &desc,
                ptr::null(),
                ptr::null_mut(),
                &mut swap_chain as *mut *mut _,
            )
        };

        (SwapChain1(ComPtr::from_raw(swap_chain)), hr)
    }
}

impl Factory4 {
    pub unsafe fn create(flags: minwindef::UINT) -> D3DResult<Self> {
        let mut factory = ptr::null_mut();
        let hr = unsafe {
            dxgi1_3::CreateDXGIFactory2(
                flags,
                &dxgi1_4::IDXGIFactory4::uuidof(),
                factory as *mut *mut _,
            )
        };

        (Factory4(ComPtr::from_raw(factory)), hr)
    }

    pub unsafe fn as_factory2(&self) -> Factory2 {
        Factory2(ComPtr::from_raw(self.0.as_raw() as *mut _))
    }

    pub unsafe fn enumerate_adapters(&self, id: u32) -> D3DResult<Adapter1> {
        let mut adapter = ptr::null_mut();
        let hr = unsafe { self.0.EnumAdapters1(id, adapter as *mut *mut _) };

        (Adapter1(ComPtr::from_raw(adapter)), hr)
    }
}

impl CommandQueue {
    pub fn signal(&self, fence: Fence, value: u64) -> winerror::HRESULT {
        unsafe { self.0.Signal(fence.0.as_raw(), value) }
    }
}

impl SwapChain {
    pub unsafe fn get_buffer(&self, id: u32) -> D3DResult<Resource> {
        let mut resource = ptr::null_mut();
        let hr = unsafe {
            self.0.GetBuffer(
                id,
                &d3d12::ID3D12Resource::uuidof(),
                resource as *mut *mut _,
            )
        };

        (Resource(ComPtr::from_raw(resource)), hr)
    }

    // TODO: present flags
    pub fn present(&self, interval: u32, flags: u32) -> winerror::HRESULT {
        unsafe { self.0.Present(interval, flags) }
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

    pub unsafe fn get_buffer(&self, id: u32) -> D3DResult<Resource> {
        let mut resource = ptr::null_mut();
        let hr = unsafe {
            self.0.GetBuffer(
                id,
                &d3d12::ID3D12Resource::uuidof(),
                resource as *mut *mut _,
            )
        };

        (Resource(ComPtr::from_raw(resource)), hr)
    }
}

impl SwapChain3 {
    pub unsafe fn get_buffer(&self, id: u32) -> D3DResult<Resource> {
        let mut resource = ptr::null_mut();
        let hr = unsafe {
            self.0.GetBuffer(
                id,
                &d3d12::ID3D12Resource::uuidof(),
                resource as *mut *mut _,
            )
        };

        (Resource(ComPtr::from_raw(resource)), hr)
    }

    pub fn get_current_back_buffer_index(&self) -> u32 {
        unsafe { self.0.GetCurrentBackBufferIndex() }
    }
}

impl Device {
    pub unsafe fn create_device(factory4: &Factory4) -> Result<Device, Vec<winerror::HRESULT>> {
        let mut id = 0;
        let mut errors: Vec<winerror::HRESULT> = Vec::new();

        loop {
            let adapter: Adapter1 =
                match error_if_failed_else_value(factory4.enumerate_adapters(id)) {
                    Ok(a) => a,
                    Err(hr) => {
                        errors.push(hr);
                        return Err(errors);
                    }
                };

            id += 1;

            match error_if_failed_else_value(Device::create_using_adapter(
                adapter.0.clone(),
                d3dcommon::D3D_FEATURE_LEVEL_12_0,
            )) {
                Ok(device) => {
                    std::mem::drop(adapter);
                    return Ok(device);
                }
                Err(hr) => {
                    errors.push(hr);
                    continue;
                }
            }
        }
    }

    pub unsafe fn create_using_adapter<I: Interface>(
        adapter: ComPtr<I>,
        feature_level: d3dcommon::D3D_FEATURE_LEVEL,
    ) -> D3DResult<Self> {
        let mut device = ptr::null_mut();
        let hr = unsafe {
            d3d12::D3D12CreateDevice(
                adapter.as_raw() as *mut _,
                feature_level as _,
                &d3d12::ID3D12Device::uuidof(),
                device as *mut *mut _,
            )
        };

        (Device(ComPtr::from_raw(device)), hr)
    }

    pub unsafe fn create_command_allocator(
        &self,
        list_type: d3d12::D3D12_COMMAND_LIST_TYPE,
    ) -> D3DResult<CommandAllocator> {
        let mut allocator = ptr::null_mut();
        let hr = unsafe {
            self.0.CreateCommandAllocator(
                list_type,
                &d3d12::ID3D12CommandAllocator::uuidof(),
                allocator as *mut *mut _,
            )
        };

        (CommandAllocator(ComPtr::from_raw(allocator)), hr)
    }

    pub unsafe fn create_command_queue(
        &self,
        list_type: d3d12::D3D12_COMMAND_LIST_TYPE,
        priority: minwindef::INT,
        flags: d3d12::D3D12_COMMAND_QUEUE_FLAGS,
        node_mask: minwindef::UINT,
    ) -> D3DResult<CommandQueue> {
        let desc = d3d12::D3D12_COMMAND_QUEUE_DESC {
            Type: list_type,
            Priority: priority,
            Flags: flags,
            NodeMask: node_mask,
        };

        let mut cmd_q = ptr::null_mut();
        let hr = unsafe {
            self.0.CreateCommandQueue(
                &desc,
                &d3d12::ID3D12CommandQueue::uuidof(),
                cmd_q as *mut *mut _,
            )
        };

        (CommandQueue(ComPtr::from_raw(cmd_q)), hr)
    }

    pub unsafe fn create_descriptor_heap(
        &self,
        heap_description: &d3d12::D3D12_DESCRIPTOR_HEAP_DESC,
    ) -> D3DResult<DescriptorHeap> {
        let mut heap = ptr::null_mut();
        let hr = unsafe {
            self.0.CreateDescriptorHeap(
                heap_description,
                &d3d12::ID3D12DescriptorHeap::uuidof(),
                heap as *mut *mut _,
            )
        };

        (DescriptorHeap(ComPtr::from_raw(heap)), hr)
    }

    pub fn get_descriptor_increment_size(
        &self,
        heap_type: d3d12::D3D12_DESCRIPTOR_HEAP_TYPE,
    ) -> u32 {
        unsafe { self.0.GetDescriptorHandleIncrementSize(heap_type) }
    }

    pub unsafe fn create_compute_pipeline_state(
        &self,
        compute_pipeline_desc: &d3d12::D3D12_COMPUTE_PIPELINE_STATE_DESC,
    ) -> D3DResult<PipelineState> {
        let mut pipeline = ptr::null_mut();

        let hr = unsafe {
            self.0.CreateComputePipelineState(
                compute_pipeline_desc as *const _,
                &d3d12::ID3D12PipelineState::uuidof(),
                pipeline as *mut *mut _,
            )
        };

        (PipelineState(ComPtr::from_raw(pipeline)), hr)
    }

    pub unsafe fn create_root_signature(
        &self,
        node_mask: minwindef::UINT,
        blob: Blob,
    ) -> D3DResult<RootSignature> {
        let mut signature = ptr::null_mut();
        let hr = unsafe {
            self.0.CreateRootSignature(
                node_mask,
                blob.0.GetBufferPointer(),
                blob.0.GetBufferSize(),
                &d3d12::ID3D12RootSignature::uuidof(),
                signature as *mut *mut _,
            )
        };

        (RootSignature(ComPtr::from_raw(signature)), hr)
    }

    pub unsafe fn create_command_signature(
        &self,
        root_signature: RootSignature,
        arguments: &[d3d12::D3D12_INDIRECT_ARGUMENT_DESC],
        stride: u32,
        node_mask: minwindef::UINT,
    ) -> D3DResult<CommandSignature> {
        let mut signature = ptr::null_mut();
        let desc = d3d12::D3D12_COMMAND_SIGNATURE_DESC {
            ByteStride: stride,
            NumArgumentDescs: arguments.len() as _,
            pArgumentDescs: arguments.as_ptr() as *const _,
            NodeMask: node_mask,
        };

        let hr = unsafe {
            self.0.CreateCommandSignature(
                &desc,
                root_signature.0.as_raw(),
                &d3d12::ID3D12CommandSignature::uuidof(),
                signature as *mut *mut _,
            )
        };

        (CommandSignature(ComPtr::from_raw(signature)), hr)
    }

    pub unsafe fn create_graphics_command_list(
        &self,
        list_type: d3d12::D3D12_COMMAND_LIST_TYPE,
        allocator: CommandAllocator,
        initial_ps: PipelineState,
        node_mask: minwindef::UINT,
    ) -> D3DResult<GraphicsCommandList> {
        let mut command_list = ptr::null_mut();

        let hr = self.0.CreateCommandList(
            node_mask,
            list_type,
            allocator.0.as_raw(),
            initial_ps.0.as_raw(),
            &d3d12::ID3D12GraphicsCommandList::uuidof(),
            command_list as *mut *mut _,
        );

        (GraphicsCommandList(ComPtr::from_raw(command_list)), hr)
    }

    pub unsafe fn create_render_target_view(
        &self,
        resource: Resource,
        desc: *const d3d12::D3D12_RENDER_TARGET_VIEW_DESC,
        descriptor: CpuDescriptor,
    ) {
        unsafe {
            self.0
                .CreateRenderTargetView(resource.0.as_raw(), desc, descriptor);
        }
    }

    // TODO: interface not complete
    pub unsafe fn create_fence(&self, initial: u64) -> D3DResult<Fence> {
        let mut fence = ptr::null_mut();
        let hr = self.0.CreateFence(
            initial,
            d3d12::D3D12_FENCE_FLAG_NONE,
            &d3d12::ID3D12Fence::uuidof(),
            fence as *mut *mut _,
        );

        (Fence(ComPtr::from_raw(fence)), hr)
    }
}

impl CommandAllocator {
    pub unsafe fn reset(&self) {
        self.0.Reset();
    }
}

impl DescriptorHeap {
    pub fn start_cpu_descriptor(&self) -> CpuDescriptor {
        unsafe { self.0.GetCPUDescriptorHandleForHeapStart() }
    }
}

#[repr(transparent)]
pub struct DescriptorRange(d3d12::D3D12_DESCRIPTOR_RANGE);
impl DescriptorRange {}

impl RootSignature {
    pub unsafe fn serialize(
        desc: &d3d12::D3D12_ROOT_SIGNATURE_DESC,
        version: d3d12::D3D_ROOT_SIGNATURE_VERSION,
    ) -> D3DResult<(Blob, ErrorBlob)> {
        let mut blob = ptr::null_mut();
        let mut error = ptr::null_mut();

        let hr = unsafe {
            d3d12::D3D12SerializeRootSignature(
                desc as *const _,
                version,
                blob as *mut *mut _,
                error as *mut *mut _,
            )
        };

        (
            (
                Blob(ComPtr::from_raw(blob)),
                ErrorBlob(ComPtr::from_raw(error)),
            ),
            hr,
        )
    }
}

impl ShaderByteCode {
    // `blob` may not be null.
    pub unsafe fn from_blob(blob: Blob) -> Self {
        ShaderByteCode(d3d12::D3D12_SHADER_BYTECODE {
            BytecodeLength: blob.0.GetBufferSize(),
            pShaderBytecode: blob.0.GetBufferPointer(),
        })
    }

    /// Compile a shader from raw HLSL.
    ///
    /// * `target`: example format: `ps_5_1`.
    pub unsafe fn compile(
        code: &[u8],
        target: String,
        entry: String,
        flags: minwindef::DWORD,
    ) -> D3DResult<(Blob, ErrorBlob)> {
        let mut shader = ptr::null_mut();
        let mut error = ptr::null_mut();

        let target = ffi::CString::new(target)
            .expect("could not convert target format string into ffi::CString");
        let entry = ffi::CString::new(entry)
            .expect("could not convert entry name String into ffi::CString");

        let hr = unsafe {
            winapi::um::d3dcompiler::D3DCompile(
                code.as_ptr() as *const _,
                code.len(),
                ptr::null(), // defines
                ptr::null(), // include
                ptr::null_mut(),
                entry.as_ptr() as *const _,
                target.as_ptr() as *const _,
                flags,
                0,
                shader as *mut *mut _,
                error as *mut *mut _,
            )
        };

        (
            (
                Blob(ComPtr::from_raw(shader)),
                ErrorBlob(ComPtr::from_raw(error)),
            ),
            hr,
        )
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
    pub unsafe fn as_list(&self) -> CommandList {
        CommandList(ComPtr::from_raw(self.0.as_raw() as *mut _))
    }

    pub unsafe fn close(&self) -> winerror::HRESULT {
        self.0.Close()
    }

    pub unsafe fn reset(
        &self,
        allocator: CommandAllocator,
        initial_pso: PipelineState,
    ) -> winerror::HRESULT {
        self.0.Reset(allocator.0.as_raw(), initial_pso.0.as_raw())
    }

    pub unsafe fn set_compute_root_signature(&self, signature: RootSignature) {
        self.0.SetComputeRootSignature(signature.0.as_raw());
    }

    pub unsafe fn set_resource_barrier(&self, num_barriers: u32, resource_barriers: *const d3d12::D3D12_RESOURCE_BARRIER) {
        self.0.ResourceBarrier(num_barriers, resource_barriers);
    }
}
