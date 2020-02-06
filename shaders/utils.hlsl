bool bbox_interiors_intersect(BBox bbox0, BBox bbox1) {
    bool x_intersection = (bbox0.x_max >= bbox1.x_min && bbox1.x_max >= bbox0.x_min);
    bool y_intersection = (bbox0.y_max >= bbox1.y_min && bbox1.y_max >= bbox0.y_min);

    bool intersection = x_intersection && y_intersection;

    return intersection;
}

BBox generate_tile_bbox(uint2 tile_coord) {
    uint tile_x_ix = tile_coord.x;
    uint tile_y_ix = tile_coord.y;

    uint left = tile_side_length*tile_x_ix;
    uint top = tile_side_length*tile_y_ix;
    uint right = left + tile_side_length;
    uint bot = top + tile_side_length;

    BBox result;
    result.x_min = left;
    result.x_max = right;
    result.y_min = top;
    result.y_max = bot;
    return result;
}
