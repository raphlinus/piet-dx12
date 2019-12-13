// Copyright © 2019 piet-dx12 developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate winapi;

use winapi::shared::winerror;

pub type D3DResult<T> = (T, winerror::HRESULT);

pub fn convert_hresult_to_lower_hex(hresult: winerror::HRESULT) -> String {
    format!("{:x}", hresult as i32)
}

pub fn error_if_failed_else_value<T>(result: D3DResult<T>) -> Result<T, String> {
    let (result_value, hresult) = result;

    if winerror::SUCCEEDED(hresult) {
        Ok(result_value)
    } else {
        Err(convert_hresult_to_lower_hex(hresult))
    }
}

pub fn error_if_failed_else_unit(hresult: winerror::HRESULT) -> Result<(), String> {
    if winerror::SUCCEEDED(hresult) {
        Ok(())
    } else {
        Err(convert_hresult_to_lower_hex(hresult))
    }
}

pub fn get_human_readable_error(hresult_as_hex: &str) -> String {
    match hresult_as_hex {
        "887a0005" => {
            String::from(format!("`DXGI_ERROR_DEVICE_REMOVED` (`0x{}`): The GPU device instance has been suspended. Use `GetDeviceRemovedReason` to determine the appropriate action.", hresult_as_hex))
        },
        "887a0006" => {
            String::from(format!("`DXG_ERROR_DEVICE_HUNG` (`0x{}`): The GPU will not respond to more commands, most likely because of an invalid command passed by the calling application.", hresult_as_hex))
        },
        _ => {
            String::from(format!("unknown error: {}", hresult_as_hex))
        }
    }
}
