cbuffer Constants : register(b0)
{
	uint num_circles;
	uint tile_size;
    uint num_tiles_x;
    uint num_tiles_y;
};

ByteAddressBuffer circle_bbox_buffer : register(t0);
ByteAddressBuffer circle_color_buffer : register(t1);

RWByteAddressBuffer per_tile_command_list: register(u0);
RWTexture2D<float4> canvas : register(u1);

uint4 generate_tile_bbox(uint2 tile_coord) {
    uint tile_x_ix = tile_coord.x;
    uint tile_y_ix = tile_coord.y;

    uint left = tile_size*tile_x_ix;
    uint top = tile_size*tile_y_ix;
    uint right = left + tile_size;
    uint bot = top + tile_size;

    uint4 result = {left, right, top, bot};
    return result;
}

bool do_bbox_interiors_intersect(uint4 bbox0, uint4 bbox1) {

    uint right1 = bbox1[1];
    uint left0 = bbox0[0];
    uint left1 = bbox1[0];
    uint right0 = bbox0[1];
    
    if (right1 <= left0 || left1 >= right0) {
        return 0;
    }

    uint bot1 = bbox1[3];
    uint top0 = bbox0[2];
    uint top1 = bbox1[2];
    uint bot0 = bbox0[3];

    if (bot1 <= top0 || top1 >= bot0) {
        return 0;
    }

    return 1;
}

#include "unpack.hlsl"

uint pack_command(uint tile_ix) {
    return tile_ix;
}

[numthreads(32, 1, 1)]
void build_per_tile_command_list(uint3 DTid : SV_DispatchThreadID) {
    uint linear_tile_ix = num_tiles_x*DTid.y + DTid.x;
    uint current_command_address = num_circles*linear_tile_ix*4;
    uint next_tile_command_start_address = current_command_address + num_circles*4;
    uint num_commands = 0;
    uint4 tile_bbox = generate_tile_bbox(DTid.xy);

    for (uint i = 0; i < num_circles; i++) {
        uint4 object_bbox = load_bbox_at_index(i);
        bool hit = do_bbox_interiors_intersect(object_bbox, tile_bbox);

        if (hit) {
            per_tile_command_list.Store(current_command_address, pack_command(i));
            current_command_address += 4;
        }
    }

    // mark end of command list for this tile, if command list does not take up entire allocated space
    if (current_command_address < next_tile_command_start_address) {
        per_tile_command_list.Store(current_command_address, 4294967295);
    }
}

