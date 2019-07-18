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

float circle_shader(uint2 pixel_pos, uint2 center_pos, float radius) {
    float d = distance(pixel_pos, center_pos);
    float alpha = clamp(radius - d, 0.0f, 1.0f);
    return alpha;
}

float4 calculate_pixel_color_due_to_circle(uint2 pixel_pos, uint4 circle_bbox, float4 circle_color) {
    uint2 circle_center = {lerp(circle_bbox[0], circle_bbox[1], 0.5f), lerp(circle_bbox[2], circle_bbox[3], 0.5f)};
    float radius = (circle_bbox[1] - circle_bbox[0])*0.5f;
    float position_based_alpha = circle_shader(pixel_pos, circle_center, radius);

    float4 pixel_color = {circle_color.r, circle_color.g, circle_color.b, circle_color.a*position_based_alpha};
    return pixel_color;
}

float4 blend_pd_over(float4 bg, float4 fg) {
    return lerp(bg, float4(fg.rgb, 1.0), fg.a);
}

#include "unpack.hlsl"

bool is_pixel_in_bbox(uint2 pixel_pos, uint4 bbox) {
    uint px = pixel_pos.x;
    uint py = pixel_pos.y;

    if (bbox[0] <= px && px <= bbox[1]) {
        if (bbox[2] <= py && py <= bbox[3]) {
            return 1;
        }
    }

    return 0;
}

bool in_rect(uint2 pixel_pos, uint2 origin, uint2 size) {
    uint px = pixel_pos.x;
    uint py = pixel_pos.y;
    uint left = origin.x;
    uint right = origin.x + size.x;
    uint top = origin.y - size.y;
    uint bot = origin.y;

    if (px < left || py > bot || py < top || px > right) {
        return 0;
    } else {
        return 1;
    }
}

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

        if (in_rect(pixel_pos, og, reversed_size)) {
            return 1.0;
        }
    }

    if (get_digit_code_1(digit)) {
        og.x = origin.x + size.x - size.y;
        og.y = origin.y;

        if (in_rect(pixel_pos, og, reversed_size)) {
            return 1.0;
        }
    }

    if (get_digit_code_2(digit)) {
        og.x = origin.x;
        og.y = origin.y;

        if (in_rect(pixel_pos, og, size)) {
            return 1.0;
        }
    }

    if (get_digit_code_3(digit)) {
        og.x = origin.x;
        og.y = origin.y;

        if (in_rect(pixel_pos, og, reversed_size)) {
            return 1.0;
        }
    }

    if (get_digit_code_4(digit)) {
        og.x = origin.x;
        og.y = origin.y - size.x;

        if (in_rect(pixel_pos, og, reversed_size)) {
            return 1.0;
        }
    }

    if (get_digit_code_5(digit)) {
        og.x = origin.x;
        og.y = origin.y - 2*size.x + size.y;

        if (in_rect(pixel_pos, og, size)) {
            return 1.0;
        }
    }

    if (get_digit_code_6(digit)) {
        og.x = origin.x;
        og.y = origin.y - size.x + 0.5*size.y;

        if (in_rect(pixel_pos, og, size)) {
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
        uint digit = round(fmod(fnumber, 10.0f));

        result = digit_display_shader(digit, pixel_pos, display_origin, size);

        if (result > 0.0) {
            break;
        }

        fnumber = trunc(fnumber/10.0f);
        display_origin.x = display_origin.x - delta;

        if (fnumber == 0.0f) {
            break;
        }
    }

    return result;
}

uint unpack_command(uint command_address) {
    return per_tile_command_list.Load(command_address);
}

[numthreads(16, 16, 1)]
void paint_objects(uint3 Gid: SV_GroupID, uint3 DTid : SV_DispatchThreadID) {
    float4 bg = {0.0f, 0.0f, 0.0f, 0.0f};
    float4 fg = {0.0f, 0.0f, 0.0f, 0.0f};

    uint2 pixel_pos = DTid.xy;

    // // uint casting is same as flooring (in general, casting is round to zero)
    uint tile_ix = Gid.y*num_tiles_x + Gid.x;
    uint command_address = num_circles*tile_ix*4;

    for (uint i = 0; i < num_circles; i++) {
        // do we want object index, bbox loading, color loading etc. to be in thread group shared memory?
        uint object_index = unpack_command(command_address);

        if (object_index == 4294967295) {
            break;
        } else {
            uint4 bbox = load_bbox_at_index(object_index);
            bool hit = is_pixel_in_bbox(pixel_pos, bbox);

            if (hit) {
                float4 color = load_color_at_index(object_index);
                float4 fg = calculate_pixel_color_due_to_circle(pixel_pos, bbox, color);
                bg = blend_pd_over(bg, fg);
            }
            //float4 fg = {0.5f, 0.5f, 0.5f, 1.0f};
            //bg = blend_pd_over(bg, fg);
        }

        command_address += 4;
    }

    // // boolean value display
    // bool test = (value == 6553800);
    // uint4 test_bbox = {100, 200, 100, 200};
    // float4 test_success_color = {0.0f, 1.0f, 0.0f, 1.0f};
    // float4 test_fail_color = {1.0f, 0.0f, 0.0f, 1.0f};

    // if (test) {
    //     fg = calculate_pixel_color_due_to_circle(pixel_pos, test_bbox, test_success_color);
    // } else {
    //     fg = calculate_pixel_color_due_to_circle(pixel_pos, test_bbox, test_fail_color);
    // }

    // unsigned integer value display
    // uint2 rect_origin = {800, 300};
    // uint2 rect_size = {50, 10};
    // fg.r = 1.0f;
    // fg.g = 1.0f;
    // fg.b = 1.0f;
    // // should print out 6553800
    // uint4 bbox0 = {0, 100, 0, 100};
    // uint4 bbox1 = generate_tile_bbox(66);
    // uint intersection_result = do_bbox_interiors_intersect(bbox0, bbox1);
    // fg.a = number_shader(bbox1[2], pixel_pos, rect_origin, rect_size);
    // bg = blend_pd_over(bg, fg);

    canvas[DTid.xy] = bg;
}
