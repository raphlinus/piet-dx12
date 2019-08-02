uint load_packed_object_specific_data_at_object_index(uint ix) {
    uint data_address = ix*24;

    uint packed_data = object_data_buffer.Load(data_address);

    return packed_data;
}

uint2 load_packed_in_atlas_bbox_at_object_index(uint ix) {
    uint x_address = ix*24 + 4;
    uint y_address = x_address + 4;

    uint packed_bbox_x = object_data_buffer.Load(x_address);
    uint packed_bbox_y = object_data_buffer.Load(y_address);

    uint2 packed_bbox = {packed_bbox_x, packed_bbox_y};

    return packed_bbox;
}

uint2 load_packed_in_scene_bbox_at_object_index(uint ix) {
    uint x_address = ix*24 + 12;
    uint y_address = x_address + 4;

    uint packed_bbox_x = object_data_buffer.Load(x_address);
    uint packed_bbox_y = object_data_buffer.Load(y_address);

    uint2 packed_bbox = {packed_bbox_x, packed_bbox_y};

    return packed_bbox;
}

uint load_packed_color_at_object_index(uint ix) {
    uint color_address = ix*24 + 20;

    uint packed_color = object_data_buffer.Load(color_address);

    return packed_color;
}

