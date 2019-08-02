uint load_packed_object_specific_data_from_cmd(uint command_address) {
    uint data_address = command_address;

    uint packed_data = per_tile_command_list.Load(data_address);

    return packed_data;
}

uint2 load_packed_in_atlas_bbox_from_cmd(uint command_address) {
    uint x_address = command_address + 4;
    uint y_address = x_address + 4;

    uint packed_bbox_x = per_tile_command_list.Load(x_address);
    uint packed_bbox_y = per_tile_command_list.Load(y_address);

    uint2 packed_bbox = {packed_bbox_x, packed_bbox_y};

    return packed_bbox;
}

uint2 load_packed_in_scene_bbox_from_cmd(uint command_address) {
    uint x_address = command_address + 12;
    uint y_address = x_address + 4;

    uint packed_bbox_x = per_tile_command_list.Load(x_address);
    uint packed_bbox_y = per_tile_command_list.Load(y_address);

    uint2 packed_bbox = {packed_bbox_x, packed_bbox_y};

    return packed_bbox;
}

uint load_packed_color_from_cmd(uint command_address) {
    uint color_address = command_address + 20;

    uint packed_color = per_tile_command_list.Load(color_address);

    return packed_color;
}

