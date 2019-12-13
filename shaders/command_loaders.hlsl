// Copyright © 2019 piet-dx12 developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

uint load_packed_general_data_from_cmd(uint data_address) {
    uint packed_data = per_tile_command_list.Load(data_address);

    return packed_data;
}

uint2 load_packed_in_scene_bbox_from_cmd(uint data_address) {
    uint x_address = data_address;
    uint y_address = x_address + 4;

    uint packed_bbox_x = per_tile_command_list.Load(x_address);
    uint packed_bbox_y = per_tile_command_list.Load(y_address);

    uint2 packed_bbox = {packed_bbox_x, packed_bbox_y};

    return packed_bbox;
}

uint2 load_packed_in_atlas_bbox_from_cmd(uint data_address) {
    uint x_address = data_address;
    uint y_address = x_address + 4;

    uint packed_bbox_x = per_tile_command_list.Load(x_address);
    uint packed_bbox_y = per_tile_command_list.Load(y_address);

    uint2 packed_bbox = {packed_bbox_x, packed_bbox_y};

    return packed_bbox;
}

uint load_packed_color_from_cmd(uint data_address) {
    uint packed_color = per_tile_command_list.Load(data_address);

    return packed_color;
}
