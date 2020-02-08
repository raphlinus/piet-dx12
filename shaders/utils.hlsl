bool bbox_interiors_intersect(BBox bbox0, BBox bbox1) {
    bool x_intersection = (bbox0.x1 >= bbox1.x0 && bbox1.x1 >= bbox0.x0);
    bool y_intersection = (bbox0.y1 >= bbox1.y0 && bbox1.y1 >= bbox0.y0);

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
    result.x0 = left;
    result.x1 = right;
    result.y0 = top;
    result.y1 = bot;
    return result;
}
