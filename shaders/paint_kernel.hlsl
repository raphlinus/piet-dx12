// Copyright © 2019 piet-dx12 developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

RWByteAddressBuffer per_tile_command_list: register(u0);

cbuffer SceneConstants: register(b0) {
    uint num_objects_in_scene;
};

cbuffer GpuStateConstants : register(b1)
{
    uint max_objects_in_scene;
	uint tile_side_length_in_pixels;
    uint num_tiles_x;
    uint num_tiles_y;
};

cbuffer DataSpecificationConstants : register(b2)
{
    uint object_size;
    uint init_in_scene_bbox_address;
    uint init_general_data_address;
    uint init_in_atlas_bbox_address;
    uint init_color_data_address;
    uint bbox_data_size;
    uint general_data_size;
    uint color_data_size;
}

Texture2D<float> glyph_atlas : register(t1);
RWTexture2D<float4> canvas : register(u1);

#include "C:/Users/bhmer/Desktop/piet-dx12/shaders/command_loaders.hlsl"
#include "C:/Users/bhmer/Desktop/piet-dx12/shaders/unpack.hlsl"

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

bool is_pixel_in_rect(uint2 pixel_pos, uint2 origin, uint2 size) {
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

float circle_shader(uint2 pixel_pos, uint2 center_pos, float radius) {
    float d = distance(pixel_pos, center_pos);
    float alpha = clamp(radius - d, 0.0, 1.0);
    return alpha;
}

float calculate_pixel_alpha_due_to_circle(uint2 pixel_pos, uint4 circle_bbox, float color_alpha) {
    uint2 circle_center = {lerp(circle_bbox[0], circle_bbox[1], 0.5), lerp(circle_bbox[2], circle_bbox[3], 0.5)};
    float radius = (circle_bbox[1] - circle_bbox[0])*0.5;
    float position_based_alpha = circle_shader(pixel_pos, circle_center, radius);

    float pixel_alpha = color_alpha*position_based_alpha;

    return pixel_alpha;
}

float calculate_pixel_alpha_due_to_glyph(uint2 pixel_pos, uint4 in_atlas_bbox, uint4 in_scene_bbox, float color_alpha) {
    uint2 atlas_pixel_pos = {in_atlas_bbox[0] + (pixel_pos.x - in_scene_bbox[0]), in_atlas_bbox[2] + (pixel_pos.y - in_scene_bbox[2])};
    float glyph_alpha = glyph_atlas[atlas_pixel_pos];

    float pixel_alpha = color_alpha*glyph_alpha;

    return pixel_alpha;
}

float4 blend_pd_over(float4 bg, float4 fg) {
    return lerp(bg, float4(fg.rgb, 1.0), fg.a);
}

// simple number printing
bool get_digit_code_0(uint digit) {
	if (digit == 5 || digit == 6) {
		return 0;
	}

	return 1;
}

bool get_digit_code_1(uint digit) {
	if (digit == 2) {
		return 0;
	}

	return 1;
}

bool get_digit_code_2(uint digit) {
	if (digit == 1 || digit == 4 || digit == 7) {
		return 0;
	}

	return 1;
}

bool get_digit_code_3(uint digit) {
	if (digit == 0 || digit == 2 || digit == 6 || digit == 8) {
		return 1;
	}

	return 0;
}

bool get_digit_code_4(uint digit) {
	if (digit == 1 || digit == 2 || digit == 3 || digit == 7) {
		return 0;
	}

	return 1;
}

bool get_digit_code_5(uint digit) {
	if (digit == 1 || digit == 4) {
		return 0;
	}

	return 1;
}

bool get_digit_code_6(uint digit) {
	if (digit == 0 || digit == 1 || digit == 7) {
		return 0;
	}

	return 1;
}

float digit_display_shader(uint digit, uint2 pixel_pos, uint2 origin, uint2 size) {
    uint2 reversed_size = size.yx;
    uint2 og = {0, 0};

    if (get_digit_code_0(digit)) {
        og.x = origin.x + size.x - size.y;
        og.y = origin.y - size.x;

        if (is_pixel_in_rect(pixel_pos, og, reversed_size)) {
            return 1.0;
        }
    }

    if (get_digit_code_1(digit)) {
        og.x = origin.x + size.x - size.y;
        og.y = origin.y;

        if (is_pixel_in_rect(pixel_pos, og, reversed_size)) {
            return 1.0;
        }
    }

    if (get_digit_code_2(digit)) {
        og.x = origin.x;
        og.y = origin.y;

        if (is_pixel_in_rect(pixel_pos, og, size)) {
            return 1.0;
        }
    }

    if (get_digit_code_3(digit)) {
        og.x = origin.x;
        og.y = origin.y;

        if (is_pixel_in_rect(pixel_pos, og, reversed_size)) {
            return 1.0;
        }
    }

    if (get_digit_code_4(digit)) {
        og.x = origin.x;
        og.y = origin.y - size.x;

        if (is_pixel_in_rect(pixel_pos, og, reversed_size)) {
            return 1.0;
        }
    }

    if (get_digit_code_5(digit)) {
        og.x = origin.x;
        og.y = origin.y - 2*size.x + size.y;

        if (is_pixel_in_rect(pixel_pos, og, size)) {
            return 1.0;
        }
    }

    if (get_digit_code_6(digit)) {
        og.x = origin.x;
        og.y = origin.y - size.x + 0.5*size.y;

        if (is_pixel_in_rect(pixel_pos, og, size)) {
            return 1.0;
        }
    }

    return 0.0;
}

float number_shader(uint number, uint2 pixel_pos, uint2 init_display_origin, uint2 size) {
    uint2 display_origin = init_display_origin;
    uint delta = size.x + 10;

    uint num_digits = 0;
    float fnumber = number;
    float result = 0.0;

    while (1) {
        uint digit = round(fmod(fnumber, 10.0));

        result = digit_display_shader(digit, pixel_pos, display_origin, size);

        if (result > 0.0) {
            break;
        }

        fnumber = trunc(fnumber/10.0);
        display_origin.x = display_origin.x - delta;

        if (fnumber == 0.0) {
            break;
        }
    }

    return result;
}

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

[numthreads(16, 16, 1)]
void paint_objects(uint3 Gid: SV_GroupID, uint3 DTid : SV_DispatchThreadID) {
    float4 bg = {0.0, 0.0, 0.0, 0.0};
    float4 fg = {0.0, 0.0, 0.0, 0.0};

    uint2 pixel_pos = DTid.xy;

    uint linear_tile_ix = Gid.y*num_tiles_x + Gid.x;
    uint size_of_command_list = 4 + num_objects_in_scene*object_size;
    uint num_commands_address = size_of_command_list*linear_tile_ix;

    uint this_tile_num_commands = per_tile_command_list.Load(num_commands_address);

    uint cmd_general_data_start = num_commands_address + 4;
    uint cmd_in_scene_bbox_start = cmd_general_data_start + num_objects_in_scene*general_data_size;
    uint cmd_in_atlas_bbox_start = cmd_in_scene_bbox_start + num_objects_in_scene*bbox_data_size;
    uint cmd_color_start = cmd_in_atlas_bbox_start + num_objects_in_scene*bbox_data_size;

    for (uint i = 0; i < this_tile_num_commands; i++) {
        /**
        fg.g = 1.0;
        fg.a = 1.0;
        bg = blend_pd_over(bg, fg);
        **/

        uint2 packed_in_scene_bbox = load_packed_in_scene_bbox_from_cmd(cmd_in_scene_bbox_start + i*bbox_data_size);
        uint4 in_scene_bbox = unpack_bbox(packed_in_scene_bbox);

        bool hit = is_pixel_in_bbox(pixel_pos, in_scene_bbox);

        if (hit) {
            uint packed_general_data = load_packed_general_data_from_cmd(cmd_general_data_start + i*general_data_size);
            uint2 general_data = unpack_general_data(packed_general_data);
            uint object_type = general_data.x;

            uint packed_color = load_packed_color_from_cmd(cmd_color_start + i*color_data_size);
            fg = unpack_color(packed_color);

            if (object_type == 0) {
                //float4 green = {0.0, 1.0, 0.0, 1.0};
                fg.a = calculate_pixel_alpha_due_to_circle(pixel_pos, in_scene_bbox, fg.a);
            } else {
                //float4 blue = {0.0, 0.0, 1.0, 1.0};
                uint2 packed_in_atlas_bbox = load_packed_in_atlas_bbox_from_cmd(cmd_in_atlas_bbox_start + i*bbox_data_size);
                uint4 in_atlas_bbox = unpack_bbox(packed_in_atlas_bbox);

                fg.a = calculate_pixel_alpha_due_to_glyph(pixel_pos, in_atlas_bbox, in_scene_bbox, fg.a);
            }

            bg = blend_pd_over(bg, fg);
        }
    }
    
    /**
    uint2 rect_origin = {200, 200};
    uint2 rect_size = {50, 10};
    fg.r = 1.0;
    fg.g = 1.0;
    fg.b = 1.0;
    fg.a = 0.0;

    uint2 packed_in_scene_bbox = load_packed_in_scene_bbox_from_cmd(cmd_in_scene_bbox_start);
    uint4 in_scene_bbox = unpack_bbox(packed_in_scene_bbox);

    uint packed_general_data = load_packed_general_data_from_cmd(cmd_general_data_start);
    uint2 general_data = unpack_general_data(packed_general_data);
    uint object_type = general_data.x;

    uint packed_color = load_packed_color_from_cmd(cmd_color_start);
    uint4 int_color = extract_u8s_from_uint(packed_color);

    fg.a = number_shader(int_color[0], pixel_pos, rect_origin, rect_size);

    bg = blend_pd_over(bg, fg);
    **/

    /**
    fg.r = 1.0;
    fg.g = 0.0;
    fg.b = 0.0;
    fg.a = 0.0;

    if (this_tile_num_commands > 0) {
        fg.a = 1.0;
    }

    bg = blend_pd_over(bg, fg);
    **/

    canvas[DTid.xy] = bg;
}
