// Copyright © 2019 piet-dx12 developers.

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// a structure of arrays
// fields: scene_bbox_per_item, general_data_per_item, atlas_bbox_per_item, color_data_per_item
// suppose scene_bbox_per_item starts at address a_0
// then general_data_per_item starts at address a_1 = a_0 + bbox_size*num_items
// atlas_bbox_per_item starts at a_2 = a_1 + general_data_size*num_items
// ... and so on
ByteAddressBuffer item_scene_bboxes: register(t0);
ByteAddressBuffer item_data_buffer : register(t1);

RWByteAddressBuffer per_tile_command_list: register(u0);

cbuffer SceneConstants: register(b0) {
    uint num_items;
};

cbuffer GpuStateConstants : register(b1)
{
    uint max_items;
    uint tile_side_length;
    uint num_tiles_x;
    uint num_tiles_y;
};

~READERS~

~UTILS~

#define NUM_CMD_OFFSET 4

[numthreads(~PTCL_X~, ~PTCL_Y~, 1)]
void build_per_tile_command_list(uint3 DTid : SV_DispatchThreadID) {
    uint tile_ix = num_tiles_x*DTid.y + DTid.x;

    uint size_of_command_list = NUM_CMD_OFFSET + num_items*PIET_ITEM_SIZE;
    uint cmd_list_init = size_of_command_list*tile_ix;
    uint cmd_list_offset = cmd_list_init + NUM_CMD_OFFSET;
    uint item_offset = 0;
    uint item_bbox_offset = 0;

    uint num_commands = 0;
    BBox tile_bbox = generate_tile_bbox(DTid.xy);

    for (uint i = 0; i < num_items; i++) {
        BBoxPacked packed_scene_bbox = BBox_read(item_scene_bboxes, item_bbox_offset);
        BBox scene_bbox = BBox_unpack(packed_scene_bbox);
        bool hit = bbox_interiors_intersect(scene_bbox, tile_bbox);

        if (hit) {
            PietItem_read_into(item_data_buffer, item_offset, per_tile_command_list, cmd_list_offset);
            cmd_list_offset += PIET_ITEM_SIZE;
            num_commands += 1;
        }

        item_offset += PIET_ITEM_SIZE;
        item_bbox_offset += BBOX_SIZE;
    }
    per_tile_command_list.Store(cmd_list_init, num_commands);
}
