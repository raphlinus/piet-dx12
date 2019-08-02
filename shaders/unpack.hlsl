uint2 extract_ushort2_from_uint(uint input_value) {
    // https://www.wolframalpha.com/input/?i=1111111111111111_2
    uint right_mask = 65535;
    uint left_mask = right_mask << 16;

    uint left_value = (left_mask & input_value) >> 16;
    uint right_value = right_mask & input_value;

    uint2 result = {left_value, right_value};

    return result;
}

uint4 extract_u8s_from_uint(uint input_value) {
    uint r_shift = 24;
    uint g_shift = 16;
    uint b_shift = 8;

    uint mask_a = 255;
    uint mask_b = mask_a << b_shift;
    uint mask_g = mask_a << g_shift;
    uint mask_r = mask_a << r_shift;

    uint r = (mask_r & input_value) >> r_shift;
    uint g = (mask_g & input_value) >> g_shift;
    uint b = (mask_b & input_value) >> b_shift;
    uint a = (mask_a & input_value);

    uint4 result = {r, g, b, a};
    return result;
}

uint2 unpack_object_specific_data(uint packed_data) {
    uint2 unpacked_data = extract_ushort2_from_uint(packed_data);

    return unpacked_data;
}

uint4 unpack_bbox(uint2 packed_bbox) {
    uint2 bbox_x = extract_ushort2_from_uint(packed_bbox.x);
    uint2 bbox_y = extract_ushort2_from_uint(packed_bbox.y);

    uint4 bbox = {bbox_x, bbox_y};

    return bbox;
}


float4 unpack_color(uint packed_color) {
    uint4 int_colors = extract_u8s_from_uint(packed_color);
    float4 float_int_colors = int_colors;

    float r = float_int_colors.r/255.0f;
    float g = float_int_colors.g/255.0f;
    float b = float_int_colors.b/255.0f;
    float a = float_int_colors.a/255.0f;

    float4 result = {r, g, b, a};
    return result;
}
