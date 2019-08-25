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

pub fn error_if_failed_else_value<T>(result: D3DResult<T>) -> Result<T, winerror::HRESULT> {
    let (result_value, hresult) = result;

    if winerror::SUCCEEDED(hresult) {
        Ok(result_value)
    } else {
        Err(hresult)
    }
}

pub fn error_if_failed_else_unit(hresult: winerror::HRESULT) -> Result<(), winerror::HRESULT> {
    if winerror::SUCCEEDED(hresult) {
        Ok(())
    } else {
        Err(hresult)
    }
}
