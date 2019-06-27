extern crate wio;
extern crate winapi;

// everything is ripped from d3d12-rs, but wio::com::ComPtr, and winapi are used more directly

pub type D3DResult<T> = (T, winapi::shared::winerror::HRESULT);

pub type Heap = wio::com::ComPtr<winapi::um::d3d12::ID3D12Heap>;
pub type Subresource = u32;
pub type Resource = wio::com::ComPtr<winapi::um::d3d12::ID3D12Resource>;

pub type Adapter1 = wio::com::ComPtr<winapi::shared::dxgi::IDXGIAdapter1>;
pub type Factory2 = wio::com::ComPtr<winapi::shared::dxgi1_2::IDXGIFactory2>;
pub type Factory4 = wio::com::ComPtr<winapi::shared::dxgi1_4::IDXGIFactory4>;
pub type SwapChain = wio::com::ComPtr<dxgi::IDXGISwapChain>;
pub type SwapChain1 = wio::com::ComPtr<dxgi1_2::IDXGISwapChain1>;
pub type SwapChain3 = wio::com::ComPtr<dxgi1_4::IDXGISwapChain3>;

pub type QueryHeap = wio::com::ComPtr<winapi::um::d3d12::ID3D12QueryHeap>;

pub type Device = wio::com::ComPtr<winapi::um::d3d12::ID3D12Device>;

pub type CommandQueue = wio::com::ComPtr<winapi::um::d3d12::ID3D12CommandQueue>;

pub type CommandAllocator = wio::com::ComPtr<winapi::um::d3d12::ID3D12CommandAllocator>;

pub type CpuDescriptor = winapi::um::d3d12::D3D12_CPU_DESCRIPTOR_HANDLE;
pub type GpuDescriptor = winapi::um::d3d12::D3D12_GPU_DESCRIPTOR_HANDLE;

pub type DescriptorHeap = wio::com::ComPtr<winapi::um::d3d12::ID3D12DescriptorHeap>;

pub type TextureAddressMode = [winapi::um::d3d12::D3D12_TEXTURE_ADDRESS_MODE; 3];

pub type RootSignature = wio::com::ComPtr<winapi::um::d3d12::ID3D12RootSignature>;

pub type Error = wio::com::ComPtr<winapi::um::d3dcommon::ID3DBlob>;

pub type CommandSignature = wio::com::ComPtr<winapi::um::d3d12::ID3D12CommandSignature>;
pub type CommandList = wio::com::ComPtr<winapi::um::d3d12::ID3D12CommandList>;
pub type GraphicsCommandList = wio::com::ComPtr<winapi::um::d3d12::ID3D12GraphicsCommandList>;

#[repr(transparent)]
pub struct RenderTargetViewDesc(winapi::um::d3d12::D3D12_RENDER_TARGET_VIEW_DESC);

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
    pub fn create_swapchain_for_hwnd(
        &self,
        queue: CommandQueue,
        hwnd: winapi::shared::windef::HWND,
        desc: winapi::um::dxgi1_2::DXGI_SWAP_CHAIN_DESC1,
    ) -> D3DResult<SwapChain1> {
        let mut swap_chain = std::ptr::null_mut();
        let hr = unsafe {
            self.CreateSwapChainForHwnd(
                queue.as_mut_ptr() as *mut _,
                hwnd,
                &desc,
                std::ptr::null(),
                std::ptr::null_mut(),
                swap_chain.mut_void() as *mut *mut _,
            )
        };

        (swap_chain, hr)
    }
}


impl Factory4 {
    pub fn create(flags: winapi::shared::minwindef::UINT) -> D3DResult<Self> {
        let mut factory = std::ptr::null_mut();
        let hr = unsafe {
            winapi::um::dxgi1_3::CreateDXGIFactory2(
                flags,
                &dxgi1_4::IDXGIFactory4::uuidof(),
                factory.mut_void() as *mut *mut _,
            )
        };

        (factory, hr)
    }

    pub fn as_factory2(&self) -> Factory2 {
        unsafe { Factory2::from_raw(self.as_mut_ptr() as *mut _) }
    }

    pub fn enumerate_adapters(&self, id: u32) -> D3DResult<Adapter1> {
        let mut adapter = Adapter1::null();
        let hr = unsafe { self.EnumAdapters1(id, adapter.mut_void() as *mut *mut _) };

        (adapter, hr)
    }
}

impl CommandQueue {
    pub fn signal(&self, fence: Fence, value: u64) -> winapi::shared::winerror::HRESULT {
        unsafe { self.Signal(fence.as_mut_ptr(), value) }
    }
}

impl SwapChain {
    pub fn get_buffer(&self, id: u32) -> D3DResult<Resource> {
        let mut resource = std::ptr::null_mut();
        let hr =
            unsafe { self.GetBuffer(id, &winapi::um::d3d12::ID3D12Resource::uuidof(), resource.mut_void() as *mut *mut _) };

        (resource, hr)
    }

    // TODO: present flags
    pub fn present(&self, interval: u32, flags: u32) -> winapi::shared::winerror::HRESULT {
        unsafe { self.Present(interval, flags) }
    }
}

impl SwapChain1 {
    pub fn get_buffer(&self, id: u32) -> D3DResult<Resource> {
        let mut resource = std::ptr::null_mut();
        let hr =
            unsafe { self.GetBuffer(id, &winapi::um::d3d12::ID3D12Resource::uuidof(), resource.mut_void() as *mut *mut _) };

        (resource, hr)
    }
}

impl SwapChain3 {
    pub fn get_buffer(&self, id: u32) -> D3DResult<Resource> {
        let mut resource = std::ptr::null_mut();
        let hr =
            unsafe { self.GetBuffer(id, &winapi::um::d3d12::ID3D12Resource::uuidof(), resource.mut_void() as *mut *mut _) };

        (resource, hr)
    }

    pub fn get_current_back_buffer_index(&self) -> u32 {
        unsafe { self.GetCurrentBackBufferIndex() }
    }
}


unsafe fn create_device(factory4: &Factory4) -> Result<wio::com::ComPtr<winapi::um::d3d12::ID3D12Device>, Vec<winapi::shared::winerror::HRESULT>> {
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
}

impl Device {
    pub fn create<I: winapi::Interface>(
        adapter: wio::com::ComPtr<I>,
        feature_level: FeatureLevel,
    ) -> D3DResult<Self> {
        let mut device = Device::null();
        let hr = unsafe {
            winapi::um::d3d12::D3D12CreateDevice(
                adapter.as_unknown() as *const _ as *mut _,
                feature_level as _,
                &d3d12::ID3D12Device::uuidof(),
                device.mut_void(),
            )
        };

        (device, hr)
    }

    pub fn create_command_allocator(&self, list_type: CmdListType) -> D3DResult<CommandAllocator> {
        let mut allocator = CommandAllocator::null();
        let hr = unsafe {
            self.CreateCommandAllocator(
                list_type as _,
                &d3d12::ID3D12CommandAllocator::uuidof(),
                allocator.mut_void(),
            )
        };

        (allocator, hr)
    }

    pub fn create_command_queue(
        &self,
        list_type: CmdListType,
        priority: queue::Priority,
        flags: queue::CommandQueueFlags,
        node_mask: NodeMask,
    ) -> D3DResult<CommandQueue> {
        let desc = winapi::um::d3d12::D3D12_COMMAND_QUEUE_DESC {
            Type: list_type as _,
            Priority: priority as _,
            Flags: flags.bits(),
            NodeMask: node_mask,
        };

        let mut queue = CommandQueue::null();
        let hr = unsafe {
            self.CreateCommandQueue(
                &desc,
                &d3d12::ID3D12CommandQueue::uuidof(),
                queue.mut_void(),
            )
        };

        (queue, hr)
    }

    pub fn create_descriptor_heap(
        &self,
        num_descriptors: u32,
        heap_type: HeapType,
        flags: HeapFlags,
        node_mask: NodeMask,
    ) -> D3DResult<DescriptorHeap> {
        let desc = winapi::um::d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
            Type: heap_type as _,
            NumDescriptors: num_descriptors,
            Flags: flags.bits(),
            NodeMask: node_mask,
        };

        let mut heap = DescriptorHeap::null();
        let hr = unsafe {
            self.CreateDescriptorHeap(
                &desc,
                &d3d12::ID3D12DescriptorHeap::uuidof(),
                heap.mut_void(),
            )
        };

        (heap, hr)
    }

    pub fn get_descriptor_increment_size(&self, heap_type: HeapType) -> u32 {
        unsafe { self.GetDescriptorHandleIncrementSize(heap_type as _) }
    }

    pub fn create_graphics_command_list(
        &self,
        list_type: CmdListType,
        allocator: CommandAllocator,
        initial: PipelineState,
        node_mask: NodeMask,
    ) -> D3DResult<GraphicsCommandList> {
        let mut command_list = GraphicsCommandList::null();
        let hr = unsafe {
            self.CreateCommandList(
                node_mask,
                list_type as _,
                allocator.as_mut_ptr(),
                initial.as_mut_ptr(),
                &d3d12::ID3D12GraphicsCommandList::uuidof(),
                command_list.mut_void(),
            )
        };

        (command_list, hr)
    }

    pub fn create_compute_pipeline_state(
        &self,
        root_signature: RootSignature,
        cs: Shader,
        node_mask: NodeMask,
        cached_pso: CachedPSO,
        flags: pso::PipelineStateFlags,
    ) -> D3DResult<PipelineState> {
        let mut pipeline = PipelineState::null();
        let desc = winapi::um::d3d12::D3D12_COMPUTE_PIPELINE_STATE_DESC {
            pRootSignature: root_signature.as_mut_ptr(),
            CS: *cs,
            NodeMask: node_mask,
            CachedPSO: *cached_pso,
            Flags: flags.bits(),
        };

        let hr = unsafe {
            self.CreateComputePipelineState(
                &desc,
                &d3d12::ID3D12PipelineState::uuidof(),
                pipeline.mut_void(),
            )
        };

        (pipeline, hr)
    }

    pub fn create_root_signature(
        &self,
        blob: Blob,
        node_mask: NodeMask,
    ) -> D3DResult<RootSignature> {
        let mut signature = RootSignature::null();
        let hr = unsafe {
            self.CreateRootSignature(
                node_mask,
                blob.GetBufferPointer(),
                blob.GetBufferSize(),
                &d3d12::ID3D12RootSignature::uuidof(),
                signature.mut_void(),
            )
        };

        (signature, hr)
    }

    pub fn create_command_signature(
        &self,
        root_signature: RootSignature,
        arguments: &[IndirectArgument],
        stride: u32,
        node_mask: NodeMask,
    ) -> D3DResult<CommandSignature> {
        let mut signature = CommandSignature::null();
        let desc = winapi::um::d3d12::D3D12_COMMAND_SIGNATURE_DESC {
            ByteStride: stride,
            NumArgumentDescs: arguments.len() as _,
            pArgumentDescs: arguments.as_ptr() as *const _,
            NodeMask: node_mask,
        };

        let hr = unsafe {
            self.CreateCommandSignature(
                &desc,
                root_signature.as_mut_ptr(),
                &d3d12::ID3D12CommandSignature::uuidof(),
                signature.mut_void(),
            )
        };

        (signature, hr)
    }

    pub fn create_render_target_view(
        &self,
        resource: Resource,
        desc: &RenderTargetViewDesc,
        descriptor: CpuDescriptor,
    ) {
        unsafe {
            self.CreateRenderTargetView(resource.as_mut_ptr(), &desc.0 as *const _, descriptor);
        }
    }

    // TODO: interface not complete
    pub fn create_fence(&self, initial: u64) -> D3DResult<Fence> {
        let mut fence = Fence::null();
        let hr = unsafe {
            self.CreateFence(
                initial,
                winapi::um::d3d12::D3D12_FENCE_FLAG_NONE,
                &d3d12::ID3D12Fence::uuidof(),
                fence.mut_void(),
            )
        };

        (fence, hr)
    }
}



impl CommandAllocator {
}

pub struct Binding {
    pub register: u32,
    pub space: u32,
}


impl DescriptorHeap {
    pub fn start_cpu_descriptor(&self) -> CpuDescriptor {
        unsafe { self.GetCPUDescriptorHandleForHeapStart() }
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


impl StaticSampler {
}



impl RootSignature {
}



