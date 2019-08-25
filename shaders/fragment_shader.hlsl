// Copyright © 2019 piet-dx12 developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

RWTexture2D<float4> canvas : register(u1);

float4 PSMain(float4 position: SV_Position) : SV_TARGET
{
    uint2 pos = position.xy;
    return canvas[pos.xy];
}
