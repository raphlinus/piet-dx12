extern crate d3d12;
extern crate winapi;

pub fn error_if_failed<T>(result: d3d12::D3DResult<T>) -> Result<T, winapi::shared::winerror::HRESULT> {
    let (result_value, hresult) = result;

    if winapi::shared::winerror::SUCCEEDED(hresult) {
        Ok(result_value)
    } else {
        Err(hresult)
    }
}
