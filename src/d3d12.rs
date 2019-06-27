extern crate wio;
extern crate winapi;

use winapi::Interface;

// everything is ripped from d3d12-rs, but wio::com::ComPtr, and winapi are used more directly

pub type D3DResult<T> = (T, winapi::shared::winerror::HRESULT);

#[derive(Clone)]
pub struct Heap(pub wio::com::ComPtr<winapi::um::d3d12::ID3D12Heap>);
pub type Subresource = u32;
#[derive(Clone)]
pub struct Resource(pub wio::com::ComPtr<winapi::um::d3d12::ID3D12Resource>);
pub struct VertexBufferView(pub wio::com::ComPtr<winapi::um::d3d12::D3D12_VERTEX_BUFFER_VIEW>);

#[derive(Clone)]
pub struct Adapter1(pub wio::com::ComPtr<winapi::shared::dxgi::IDXGIAdapter1>);
#[derive(Clone)]
pub struct Factory2(pub wio::com::ComPtr<winapi::shared::dxgi1_2::IDXGIFactory2>);
#[derive(Clone)]
pub struct Factory4(pub wio::com::ComPtr<winapi::shared::dxgi1_4::IDXGIFactory4>);
#[derive(Clone)]
pub struct SwapChain(pub wio::com::ComPtr<winapi::shared::dxgi::IDXGISwapChain>);
#[derive(Clone)]
pub struct SwapChain1(pub wio::com::ComPtr<winapi::shared::dxgi1_2::IDXGISwapChain1>);
#[derive(Clone)]
pub struct SwapChain3(pub wio::com::ComPtr<winapi::shared::dxgi1_4::IDXGISwapChain3>);

#[derive(Clone)]
pub struct QueryHeap(pub wio::com::ComPtr<winapi::um::d3d12::ID3D12QueryHeap>);

#[derive(Clone)]
pub struct Device(pub wio::com::ComPtr<winapi::um::d3d12::ID3D12Device>);

#[derive(Clone)]
pub struct CommandQueue(pub wio::com::ComPtr<winapi::um::d3d12::ID3D12CommandQueue>);

#[derive(Clone)]
pub struct CommandAllocator(pub wio::com::ComPtr<winapi::um::d3d12::ID3D12CommandAllocator>);

pub type CpuDescriptor = winapi::um::d3d12::D3D12_CPU_DESCRIPTOR_HANDLE;
pub type GpuDescriptor = winapi::um::d3d12::D3D12_GPU_DESCRIPTOR_HANDLE;

#[derive(Clone)]
pub struct DescriptorHeap(pub wio::com::ComPtr<winapi::um::d3d12::ID3D12DescriptorHeap>);

pub type TextureAddressMode = [winapi::um::d3d12::D3D12_TEXTURE_ADDRESS_MODE; 3];

#[derive(Clone)]
pub struct RootSignature(pub wio::com::ComPtr<winapi::um::d3d12::ID3D12RootSignature>);

#[derive(Clone)]
pub struct CommandSignature(pub wio::com::ComPtr<winapi::um::d3d12::ID3D12CommandSignature>);
#[derive(Clone)]
pub struct CommandList(pub wio::com::ComPtr<winapi::um::d3d12::ID3D12CommandList>);
#[derive(Clone)]
pub struct GraphicsCommandList(pub wio::com::ComPtr<winapi::um::d3d12::ID3D12GraphicsCommandList>);

#[derive(Clone)]
pub struct Event(pub winapi::um::winnt::HANDLE);
#[derive(Clone)]
pub struct Fence(pub wio::com::ComPtr<winapi::um::d3d12::ID3D12Fence>);

#[derive(Clone)]
pub struct PipelineState(pub wio::com::ComPtr<winapi::um::d3d12::ID3D12PipelineState>);

#[derive(Clone)]
pub struct Shader(winapi::um::d3d12::D3D12_SHADER_BYTECODE);
#[derive(Clone)]
pub struct CachedPSO(winapi::um::d3d12::D3D12_CACHED_PIPELINE_STATE);

#[derive(Clone)]
pub struct Blob(pub wio::com::ComPtr<winapi::um::d3dcommon::ID3DBlob>);

#[derive(Clone)]
pub struct ErrorBlob(pub wio::com::ComPtr<winapi::um::d3dcommon::ID3DBlob>);

pub fn error_if_failed<T>(result: D3DResult<T>) -> Result<T, winapi::shared::winerror::HRESULT> {
    let (result_value, hresult) = result;

    if winapi::shared::winerror::SUCCEEDED(hresult) {
        Ok(result_value)
    } else {
        Err(hresult)
    }
}

impl Resource {
}

impl Factory2 {
    // TODO: interface not complete
    pub unsafe fn create_swapchain_for_hwnd(
        &self,
        queue: CommandQueue,
        hwnd: winapi::shared::windef::HWND,
        desc: winapi::shared::dxgi1_2::DXGI_SWAP_CHAIN_DESC1,
    ) -> D3DResult<SwapChain1> {
        let mut swap_chain = std::ptr::null_mut();
        let hr = unsafe {
            self.0.CreateSwapChainForHwnd(
                queue.0.as_raw() as *mut _,
                hwnd,
                &desc,
                std::ptr::null(),
                std::ptr::null_mut(),
                &mut swap_chain as *mut *mut _,
            )
        };

        (SwapChain1(wio::com::ComPtr::from_raw(swap_chain)), hr)
    }
}


impl Factory4 {
    pub unsafe fn create(flags: winapi::shared::minwindef::UINT) -> D3DResult<Self> {
        let mut factory = std::ptr::null_mut();
        let hr = unsafe {
            winapi::shared::dxgi1_3::CreateDXGIFactory2(
                flags,
                &winapi::shared::dxgi1_4::IDXGIFactory4::uuidof(),
                factory as *mut *mut _,
            )
        };

        (Factory4(wio::com::ComPtr::from_raw(factory)), hr)
    }

    pub unsafe fn as_factory2(&self) -> Factory2 {
        Factory2(wio::com::ComPtr::from_raw(self.0.as_raw() as *mut _))
    }

    pub unsafe fn enumerate_adapters(&self, id: u32) -> D3DResult<Adapter1> {
        let mut adapter = std::ptr::null_mut();
        let hr = unsafe { self.0.EnumAdapters1(id, adapter as *mut *mut _) };

        (Adapter1(wio::com::ComPtr::from_raw(adapter)), hr)
    }
}

impl CommandQueue {
    pub fn signal(&self, fence: Fence, value: u64) -> winapi::shared::winerror::HRESULT {
        unsafe { self.0.Signal(fence.0.as_raw(), value) }
    }
}

impl SwapChain {
    pub unsafe fn get_buffer(&self, id: u32) -> D3DResult<Resource> {
        let mut resource = std::ptr::null_mut();
        let hr =
            unsafe { self.0.GetBuffer(id, &winapi::um::d3d12::ID3D12Resource::uuidof(), resource as *mut *mut _) };

        (Resource(wio::com::ComPtr::from_raw(resource)), hr)
    }

    // TODO: present flags
    pub fn present(&self, interval: u32, flags: u32) -> winapi::shared::winerror::HRESULT {
        unsafe { self.0.Present(interval, flags) }
    }
}

impl SwapChain1 {
    pub unsafe fn cast_into_swap_chain3(&self) -> SwapChain3 {
        SwapChain3(self.0.cast::<winapi::shared::dxgi1_4::IDXGISwapChain3>().expect("could not cast into SwapChain3"))
    }

    pub unsafe fn get_buffer(&self, id: u32) -> D3DResult<Resource> {
        let mut resource = std::ptr::null_mut();
        let hr =
            unsafe { self.0.GetBuffer(id, &winapi::um::d3d12::ID3D12Resource::uuidof(), resource as *mut *mut _) };

        (Resource(wio::com::ComPtr::from_raw(resource)), hr)
    }
}

impl SwapChain3 {
    pub unsafe fn get_buffer(&self, id: u32) -> D3DResult<Resource> {
        let mut resource = std::ptr::null_mut();
        let hr =
            unsafe { self.0.GetBuffer(id, &winapi::um::d3d12::ID3D12Resource::uuidof(), resource as *mut *mut _) };

        (Resource(wio::com::ComPtr::from_raw(resource)), hr)
    }

    pub fn get_current_back_buffer_index(&self) -> u32 {
        unsafe { self.0.GetCurrentBackBufferIndex() }
    }
}


impl Device {
    pub unsafe fn create_device(factory4: &Factory4) -> Result<Device, Vec<winapi::shared::winerror::HRESULT>> {
        let mut id = 0;
        let mut errors: Vec<winapi::shared::winerror::HRESULT> = Vec::new();

        loop {
            let adapter: Adapter1 = match error_if_failed(factory4.enumerate_adapters(id)) {
                Ok(a) => {
                    a
                },
                Err(hr) => {
                    errors.push(hr);
                    return Err(errors);
                }
            };

            id += 1;

            match error_if_failed(Device::create_using_adapter(adapter.0.clone(), winapi::um::d3dcommon::D3D_FEATURE_LEVEL_12_0)) {
                Ok(device) => {
                    std::mem::drop(adapter);
                    return Ok(device);
                },
                Err(hr) => {
                    errors.push(hr);
                    continue;
                }
            }
        }
    }

    pub unsafe fn create_using_adapter<I: Interface>(
        adapter: wio::com::ComPtr<I>,
        feature_level: winapi::um::d3dcommon::D3D_FEATURE_LEVEL,
    ) -> D3DResult<Self> {
        let mut device = std::ptr::null_mut();
        let hr = unsafe {
            winapi::um::d3d12::D3D12CreateDevice(
                adapter.as_raw() as *mut _,
                feature_level as _,
                &winapi::um::d3d12::ID3D12Device::uuidof(),
                device as *mut *mut _,
            )
        };

        (Device(wio::com::ComPtr::from_raw(device)), hr)
    }

    pub unsafe fn create_command_allocator(&self, list_type: winapi::um::d3d12::D3D12_COMMAND_LIST_TYPE) -> D3DResult<CommandAllocator> {
        let mut allocator = std::ptr::null_mut();
        let hr = unsafe {
            self.0.CreateCommandAllocator(
                list_type,
                &winapi::um::d3d12::ID3D12CommandAllocator::uuidof(),
                allocator as *mut *mut _,
            )
        };

        (CommandAllocator(wio::com::ComPtr::from_raw(allocator)), hr)
    }

    pub unsafe fn create_command_queue(
        &self,
        list_type: winapi::um::d3d12::D3D12_COMMAND_LIST_TYPE,
        priority: winapi::shared::minwindef::INT,
        flags: winapi::um::d3d12::D3D12_COMMAND_QUEUE_FLAGS,
        node_mask: winapi::shared::minwindef::UINT,
    ) -> D3DResult<CommandQueue> {
        let desc = winapi::um::d3d12::D3D12_COMMAND_QUEUE_DESC {
            Type: list_type,
            Priority: priority,
            Flags: flags,
            NodeMask: node_mask,
        };

        let mut cmd_q = std::ptr::null_mut();
        let hr = unsafe {
            self.0.CreateCommandQueue(
                &desc,
                &winapi::um::d3d12::ID3D12CommandQueue::uuidof(),
                cmd_q as *mut *mut _,
            )
        };

        (CommandQueue(wio::com::ComPtr::from_raw(cmd_q)), hr)
    }

    pub unsafe fn create_descriptor_heap(
        &self,
        heap_description: &winapi::um::d3d12::D3D12_DESCRIPTOR_HEAP_DESC,
    ) -> D3DResult<DescriptorHeap> {
        let mut heap = std::ptr::null_mut();
        let hr = unsafe {
            self.0.CreateDescriptorHeap(
                heap_description,
                &winapi::um::d3d12::ID3D12DescriptorHeap::uuidof(),
                heap as *mut *mut _,
            )
        };

        (DescriptorHeap(wio::com::ComPtr::from_raw(heap)), hr)
    }

    pub fn get_descriptor_increment_size(&self, heap_type: winapi::um::d3d12::D3D12_DESCRIPTOR_HEAP_TYPE) -> u32 {
        unsafe { self.0.GetDescriptorHandleIncrementSize(heap_type) }
    }

    pub unsafe fn create_graphics_command_list(
        &self,
        list_type: winapi::um::d3d12::D3D12_COMMAND_LIST_TYPE,
        allocator: CommandAllocator,
        initial: PipelineState,
        node_mask: winapi::shared::minwindef::UINT,
    ) -> D3DResult<GraphicsCommandList> {
        let mut command_list = std::ptr::null_mut();
        let hr =
            self.0.CreateCommandList(
                node_mask,
                list_type,
                allocator.0.as_raw(),
                initial.0.as_raw(),
                &winapi::um::d3d12::ID3D12GraphicsCommandList::uuidof(),
                command_list as *mut *mut _,
            );

        (GraphicsCommandList(wio::com::ComPtr::from_raw(command_list)), hr)
    }

    pub unsafe fn create_compute_pipeline_state(
        &self,
        root_signature: RootSignature,
        cs: Shader,
        node_mask: winapi::shared::minwindef::UINT,
        cached_pso: CachedPSO,
        flags: winapi::um::d3d12::D3D12_PIPELINE_STATE_FLAGS,
    ) -> D3DResult<PipelineState> {
        let mut pipeline = std::ptr::null_mut();
        let desc = winapi::um::d3d12::D3D12_COMPUTE_PIPELINE_STATE_DESC {
            pRootSignature: root_signature.0.as_raw(),
            CS: cs.0,
            NodeMask: node_mask,
            CachedPSO: cached_pso.0,
            Flags: flags,
        };

        let hr = unsafe {
            self.0.CreateComputePipelineState(
                &desc,
                &winapi::um::d3d12::ID3D12PipelineState::uuidof(),
                pipeline as *mut *mut _,
            )
        };

        (PipelineState(wio::com::ComPtr::from_raw(pipeline)), hr)
    }

    pub unsafe fn create_root_signature(
        &self,
        blob: Blob,
        node_mask: winapi::shared::minwindef::UINT,
    ) -> D3DResult<RootSignature> {
        let mut signature = std::ptr::null_mut();
        let hr = unsafe {
            self.0.CreateRootSignature(
                node_mask,
                blob.0.GetBufferPointer(),
                blob.0.GetBufferSize(),
                &winapi::um::d3d12::ID3D12RootSignature::uuidof(),
                signature as *mut *mut _,
            )
        };

        (RootSignature(wio::com::ComPtr::from_raw(signature)), hr)
    }

    pub unsafe fn create_command_signature(
        &self,
        root_signature: RootSignature,
        arguments: &[winapi::um::d3d12::D3D12_INDIRECT_ARGUMENT_DESC],
        stride: u32,
        node_mask: winapi::shared::minwindef::UINT,
    ) -> D3DResult<CommandSignature> {
        let mut signature = std::ptr::null_mut();
        let desc = winapi::um::d3d12::D3D12_COMMAND_SIGNATURE_DESC {
            ByteStride: stride,
            NumArgumentDescs: arguments.len() as _,
            pArgumentDescs: arguments.as_ptr() as *const _,
            NodeMask: node_mask,
        };

        let hr = unsafe {
            self.0.CreateCommandSignature(
                &desc,
                root_signature.0.as_raw(),
                &winapi::um::d3d12::ID3D12CommandSignature::uuidof(),
                signature as *mut *mut _,
            )
        };

        (CommandSignature(wio::com::ComPtr::from_raw(signature)), hr)
    }

    pub unsafe fn create_render_target_view(
        &self,
        resource: Resource,
        desc: *const winapi::um::d3d12::D3D12_RENDER_TARGET_VIEW_DESC,
        descriptor: CpuDescriptor,
    ) {
        unsafe {
            self.0.CreateRenderTargetView(resource.0.as_raw(), desc, descriptor);
        }
    }

    // TODO: interface not complete
    pub unsafe fn create_fence(&self, initial: u64) -> D3DResult<Fence> {
        let mut fence = std::ptr::null_mut();
        let hr = unsafe {
            self.0.CreateFence(
                initial,
                winapi::um::d3d12::D3D12_FENCE_FLAG_NONE,
                &winapi::um::d3d12::ID3D12Fence::uuidof(),
                fence as *mut *mut _,
            )
        };

        (Fence(wio::com::ComPtr::from_raw(fence)), hr)
    }
}



impl CommandAllocator {
}


impl DescriptorHeap {
    pub fn start_cpu_descriptor(&self) -> CpuDescriptor {
        unsafe { self.0.GetCPUDescriptorHandleForHeapStart() }
    }
}

#[repr(transparent)]
pub struct DescriptorRange(winapi::um::d3d12::D3D12_DESCRIPTOR_RANGE);
impl DescriptorRange {

}

#[repr(transparent)]
pub struct RootParameter(winapi::um::d3d12::D3D12_ROOT_PARAMETER);
impl RootParameter {
}


impl RootSignature {
}



