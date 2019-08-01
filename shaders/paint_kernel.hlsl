cbuffer Constants : register(b0)
{
	uint num_objects;
	uint object_size;
	uint tile_size;
    uint num_tiles_x;
    uint num_tiles_y;
};


ByteAddressBuffer object_data_buffer : register(t0);
Texture2D<float> glyphs[10] : register(t1);

RWByteAddressBuffer per_tile_command_list: register(u0);
RWTexture2D<float4> canvas : register(u1);

#include "shaders/unpack.hlsl"
#include "shaders/debug.hlsl"

float circle_shader(uint2 pixel_pos, uint2 center_pos, float radius) {
    float d = distance(pixel_pos, center_pos);
    float alpha = clamp(radius - d, 0.0, 1.0);
    return alpha;
}

float4 calculate_pixel_color_due_to_circle(uint2 pixel_pos, uint4 circle_bbox, float4 circle_color) {
    uint2 circle_center = {lerp(circle_bbox[0], circle_bbox[1], 0.5), lerp(circle_bbox[2], circle_bbox[3], 0.5)};
    float radius = (circle_bbox[1] - circle_bbox[0])*0.5;
    float position_based_alpha = circle_shader(pixel_pos, circle_center, radius);

    float4 pixel_color = {circle_color.r, circle_color.g, circle_color.b, circle_color.a*position_based_alpha};
    return pixel_color;
}

float4 calculate_pixel_color_due_to_glyph(uint2 pixel_pos, uint glyph_index, uint4 glyph_bbox, float4 color) {
    uint2 glyph_pixel_pos = {pixel_pos.x - glyph_bbox[0], pixel_pos.y - glyph_bbox[2]};
    float glyph_alpha = glyphs[NonUniformResourceIndex(glyph_index)][glyph_pixel_pos];

    float4 pixel_color = {0.0, 0.0, 0.0, 0.0};

    if (glyph_alpha > 0.0) {
        float4 pixel_color = {color.r, color.g, color.b, color.a*glyph_alpha};
    }

    return pixel_color;
}

float4 blend_pd_over(float4 bg, float4 fg) {
    return lerp(bg, float4(fg.rgb, 1.0), fg.a);
}

bool is_pixel_in_bbox(uint2 pixel_pos, uint4 bbox) {
    uint px = pixel_pos.x;
    uint py = pixel_pos.y;

    // use of explicit result to avoid the following warning:
    // warning X4000: use of potentially uninitialized variable
    bool result = 0;

    uint left = bbox[0];
    uint right = bbox[1];
    uint top = bbox[2];
    uint bot = bbox[3];

    if (left <= px && px <= right) {
        if (top <= py && py <= bot) {
            result = 1;
        }
    }

    return result;
}

bool in_rect(uint2 pixel_pos, uint2 origin, uint2 size) {
    uint px = pixel_pos.x;
    uint py = pixel_pos.y;
    uint left = origin.x;
    uint right = origin.x + size.x;
    uint top = origin.y - size.y;
    uint bot = origin.y;

    bool result = 1;

    if (px < left || py > bot || py < top || px > right) {
        result = 0;
    }

    return result;
}

[numthreads(16, 16, 1)]
void paint_objects(uint3 Gid: SV_GroupID, uint3 DTid : SV_DispatchThreadID) {
    float4 bg = {0.0, 0.0, 0.0, 0.0};
    float4 fg = {0.0, 0.0, 0.0, 0.0};

    uint2 pixel_pos = DTid.xy;

    // uint casting is same as flooring (in general, casting is round to zero)
    uint linear_tile_ix = Gid.y*num_tiles_x + Gid.x;
    uint size_of_command_list = 4 + num_objects*object_size;
    uint init_address = size_of_command_list*linear_tile_ix;

    uint this_tile_num_commands = per_tile_command_list.Load(init_address);

    for (uint i = 0; i < this_tile_num_commands; i++) {
        uint command_address = i*object_size + init_adddress + 4;
        uint4 packed_command = per_tile_command_list.Load4(address);
        uint2 packed_bbox_data = packed_command.yz;

        uint4 bbox = unpack_bbox(packed_bbox_data);
        bool hit = is_pixel_in_bbox(pixel_pos, bbox);

        if (hit) {
            uint packed_object_specific_data = packed_command.x;
            uint2 object_specific_data = unpack_object_specific_data(packed_object_specific_data);
            uint object_type = object_specific_data.x;
            uint glyph_index = object_specific_data.y;
            float4 color = unpack_color(packed_command.w);

            float4 fg = {0.0, 0.0, 0.0, 0.0};
            if (object_type == 0) {
                float4 fg = calculate_pixel_color_due_to_circle(pixel_pos, bbox, color);

            } else {
                float4 fg = calculate_pixel_color_due_to_glyph(pixel_pos, glyph_index, bbox, color);
            }

            bg = blend_pd_over(bg, fg);
        }
    }

    // // boolean value display
    // bool test = (value == 6553800);
    // uint4 test_bbox = {100, 200, 100, 200};
    // float4 test_success_color = {0.0, 1.0, 0.0, 1.0};
    // float4 test_fail_color = {1.0, 0.0, 0.0, 1.0};

    // if (test) {
    //     fg = calculate_pixel_color_due_to_circle(pixel_pos, test_bbox, test_success_color);
    // } else {
    //     fg = calculate_pixel_color_due_to_circle(pixel_pos, test_bbox, test_fail_color);
    // }

    // unsigned integer value display
    // uint2 rect_origin = {800, 300};
    // uint2 rect_size = {50, 10};
    // fg.r = 1.0;
    // fg.g = 1.0;
    // fg.b = 1.0;
    // // should print out 6553800
    // uint4 bbox0 = {0, 100, 0, 100};
    // uint4 bbox1 = generate_tile_bbox(66);
    // uint intersection_result = do_bbox_interiors_intersect(bbox0, bbox1);
    // fg.a = number_shader(bbox1[2], pixel_pos, rect_origin, rect_size);
    // bg = blend_pd_over(bg, fg);

    canvas[DTid.xy] = bg;
}
