extern crate wio;
extern crate winapi;

use crate::error_utils::error_if_failed;


// everything is heavily inspired by d3d12-rs, and goal is to get these into d3d12-rs, and then use d3d12-rs directly

pub type D3DResult<T> = (T, winapi::shared::winerror::HRESULT);
pub type Adapter1 = wio::com::ComPtr<winapi::shared::dxgi::IDXGIAdapter1>;
pub type Factory2 = wio::com::ComPtr<winapi::shared::dxgi1_2::IDXGIFactory2>;
pub type Factory4 = wio::com::ComPtr<winapi::shared::dxgi1_4::IDXGIFactory4>;

impl Factory2 {
    // TODO: interface not complete
    pub fn create_swapchain_for_hwnd(
        &self,
        queue: CommandQueue,
        hwnd: HWND,
        desc: &SwapchainDesc,
    ) -> D3DResult<SwapChain1> {
        let desc = dxgi1_2::DXGI_SWAP_CHAIN_DESC1 {
            AlphaMode: desc.alpha_mode as _,
            BufferCount: desc.buffer_count,
            Width: desc.width,
            Height: desc.height,
            Format: desc.format,
            Flags: desc.flags,
            BufferUsage: desc.buffer_usage,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Scaling: desc.scaling as _,
            Stereo: desc.stereo as _,
            SwapEffect: desc.swap_effect as _,
        };

        let mut swap_chain = SwapChain1::null();
        let hr = unsafe {
            self.CreateSwapChainForHwnd(
                queue.as_mut_ptr() as *mut _,
                hwnd,
                &desc,
                ptr::null(),
                ptr::null_mut(),
                swap_chain.mut_void() as *mut *mut _,
            )
        };

        (swap_chain, hr)
    }
}

impl Factory4 {
    pub fn create(flags: FactoryCreationFlags) -> D3DResult<Self> {
        let mut factory = Factory4::null();
        let hr = unsafe {
            winapi::shared::dxgi1_3::CreateDXGIFactory2(
                flags.bits(),
                &dxgi1_4::IDXGIFactory4::uuidof(),
                factory.mut_void(),
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