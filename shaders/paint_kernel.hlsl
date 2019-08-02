cbuffer Constants : register(b0)
{
	uint num_objects;
	uint object_size;
	uint tile_size;
    uint num_tiles_x;
    uint num_tiles_y;
};


ByteAddressBuffer object_data_buffer : register(t0);
Texture2D<float> glyph_atlas : register(t1);

RWByteAddressBuffer per_tile_command_list: register(u0);
RWTexture2D<float4> canvas : register(u1);

#include "shaders/geometry.hlsl"
#include "shaders/command_loaders.hlsl"
#include "shaders/unpack.hlsl"
//#include "shaders/debug.hlsl"

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

float4 calculate_pixel_color_due_to_glyph(uint2 pixel_pos, uint4 glyph_atlas_bbox, uint4 glyph_in_scene_bbox, float4 color) {
    uint2 atlas_pixel_pos = {glyph_atlas_bbox[0] + (pixel_pos.x - glyph_in_scene_bbox[0]), glyph_atlas_bbox[2] + (pixel_pos.y - glyph_in_scene_bbox[2])};
    float glyph_alpha = glyph_atlas[atlas_pixel_pos];

    float4 pixel_color = {0.0, 0.0, 0.0, 0.0};

    if (glyph_alpha > 0.0) {
        float4 pixel_color = {color.r, color.g, color.b, color.a*glyph_alpha};
    }

    return pixel_color;
}

float4 blend_pd_over(float4 bg, float4 fg) {
    return lerp(bg, float4(fg.rgb, 1.0), fg.a);
}

[numthreads(16, 16, 1)]
void paint_objects(uint3 Gid: SV_GroupID, uint3 DTid : SV_DispatchThreadID) {
    float4 bg = {0.0, 0.0, 0.0, 0.0};
    float4 fg = {0.0, 0.0, 0.0, 0.0};

    uint2 pixel_pos = DTid.xy;

    uint linear_tile_ix = Gid.y*num_tiles_x + Gid.x;
    uint size_of_command_list = 4 + num_objects*object_size;
    uint num_commands_address = size_of_command_list*linear_tile_ix;

    uint this_tile_num_commands = per_tile_command_list.Load(num_commands_address);

    uint init_address = num_commands_address + 4;

    for (uint i = 0; i < this_tile_num_commands; i++) {
        uint command_address = i*object_size + init_address;
        uint2 packed_in_scene_bbox = load_packed_in_scene_bbox_from_cmd(command_address);
        uint4 in_scene_bbox = unpack_bbox(packed_in_scene_bbox);
        
        bool hit = is_pixel_in_bbox(pixel_pos, in_scene_bbox);

        if (hit) {
            uint packed_object_specific_data = load_packed_object_specific_data_from_cmd(command_address);
            uint2 object_specific_data = unpack_object_specific_data(packed_object_specific_data);
            uint object_type = object_specific_data.x;

            uint2 packed_color = load_packed_color_from_cmd(command_address);
            float4 color = unpack_color(packed_color);
            float4 fg = {0.0, 0.0, 0.0, 0.0};

            if (object_type == 0) {
                float4 fg = calculate_pixel_color_due_to_circle(pixel_pos, in_scene_bbox, color);
            } else {
                uint2 packed_in_atlas_bbox = load_packed_in_atlas_bbox_from_cmd(command_address);
                uint4 in_atlas_bbox = unpack_bbox(packed_in_atlas_bbox);
                float4 fg = calculate_pixel_color_due_to_glyph(pixel_pos, in_atlas_bbox, in_scene_bbox, color);
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
