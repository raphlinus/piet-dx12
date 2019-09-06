// Copyright © 2019 piet-dx12 developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

ByteAddressBuffer object_data_buffer : register(t0);
RWByteAddressBuffer per_tile_command_list: register(u0);

cbuffer SceneConstants: register(b0) {
    uint num_objects_in_scene;
};
cbuffer GpuStateConstants : register(b1)
{
    uint max_objects_in_scene;
	uint object_size;
	uint tile_side_length_in_pixels;
    uint num_tiles_x;
    uint num_tiles_y;
};

#include "shaders/object_loaders.hlsl"
#include "shaders/unpack.hlsl"

bool do_bbox_interiors_intersect(uint4 bbox0, uint4 bbox1) {
    uint right1 = bbox1[1];
    uint left0 = bbox0[0];
    uint left1 = bbox1[0];
    uint right0 = bbox0[1];

    bool result = 1;

    if (right1 <= left0 || left1 >= right0) {
        result = 0;
    }

    uint bot1 = bbox1[3];
    uint top0 = bbox0[2];
    uint top1 = bbox1[2];
    uint bot0 = bbox0[3];

    if (result && (bot1 <= top0 || top1 >= bot0)) {
        result = 0;
    }

    return result;
}

uint4 generate_tile_bbox(uint2 tile_coord) {
    uint tile_x_ix = tile_coord.x;
    uint tile_y_ix = tile_coord.y;

    uint left = tile_side_length_in_pixels*tile_x_ix;
    uint top = tile_side_length_in_pixels*tile_y_ix;
    uint right = left + tile_side_length_in_pixels;
    uint bot = top + tile_side_length_in_pixels;

    uint4 result = {left, right, top, bot};
    return result;
}

[numthreads(32, 1, 1)]
void build_per_tile_command_list(uint3 DTid : SV_DispatchThreadID) {
    uint linear_tile_ix = num_tiles_x*DTid.y + DTid.x;
    uint size_of_command_list = 4 + max_objects_in_scene*object_size;
    uint command_list_init_address = size_of_command_list*linear_tile_ix;
    uint command_start_address = command_list_init_address + 4;

    uint num_stored_commands = 0;
    uint4 tile_bbox = generate_tile_bbox(DTid.xy);

    for (uint i = 0; i < num_objects_in_scene; i++) {
        uint2 packed_in_scene_bbox = load_packed_in_scene_bbox_at_object_index(i, object_size);
        uint4 in_scene_bbox = unpack_bbox(packed_in_scene_bbox);
        bool hit = do_bbox_interiors_intersect(in_scene_bbox, tile_bbox);

        if (hit) {
            uint packed_object_specific_data = load_packed_object_specific_data_at_object_index(i, object_size);
            uint2 packed_in_atlas_bbox = load_packed_in_atlas_bbox_at_object_index(i, object_size);
            uint packed_color = load_packed_color_at_object_index(i, object_size);

            uint current_address = command_start_address + num_stored_commands*object_size;
            per_tile_command_list.Store(current_address, packed_object_specific_data);
            per_tile_command_list.Store2(current_address + 4, packed_in_atlas_bbox);
            per_tile_command_list.Store2(current_address + 12, packed_in_scene_bbox);
            per_tile_command_list.Store(current_address + 20, packed_color);
            num_stored_commands += 1;
        }
    }

    per_tile_command_list.Store(command_list_init_address, num_stored_commands);
}

