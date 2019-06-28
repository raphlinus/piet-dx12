extern crate winapi;

pub fn error_if_failed_else_unit(hresult: winapi::shared::winerror::HRESULT) -> Result<(), winapi::shared::winerror::HRESULT> {
    if winapi::shared::winerror::SUCCEEDED(hresult) {
        Ok(())
    } else {
        Err(hresult)
    }
}