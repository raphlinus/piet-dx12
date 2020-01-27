// Copyright © 2019 piet-dx12 developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

ByteAddressBuffer per_tile_command_list: register(t2);

cbuffer SceneConstants: register(b0) {
    uint num_items;
};

cbuffer GpuStateConstants : register(b1)
{
    uint max_items_scene;
	uint tile_side_length;
    uint num_tiles_x;
    uint num_tiles_y;
};

Texture2D<float> glyph_atlas : register(t3);
RWTexture2D<float4> canvas : register(u1);

~READERS~

~UTILS~

bool is_pixel_in_bbox(uint2 pixel_pos, BBox bbox) {
    uint px = pixel_pos.x;
    uint py = pixel_pos.y;

    // use of explicit result to avoid the following warning:
    // warning X4000: use of potentially uninitialized variable
    bool result = 0;

    uint left = bbox.x_min;
    uint right = bbox.x_max;
    uint top = bbox.y_min;
    uint bot = bbox.y_max;

    if (left <= px && px <= right) {
        if (top <= py && py <= bot) {
            result = 1;
        }
    }

    return result;
}

float circle_alpha(uint2 pixel_pos, BBox circle_bbox, float color_alpha) {
    uint2 circle_center = {lerp(circle_bbox.x_min, circle_bbox.x_max, 0.5), lerp(circle_bbox.y_min, circle_bbox.y_max, 0.5)};
    float radius = (circle_bbox.x_max - circle_bbox.x_min)*0.5;
    float d = distance(pixel_pos, circle_center);
    float position_alpha = clamp(radius - d, 0.0, 1.0);

    float pixel_alpha = color_alpha*position_alpha;

    return pixel_alpha;
}

float glyph_alpha(uint2 pixel_pos, BBox scene_bbox, BBox atlas_bbox, float color_alpha) {
    uint2 atlas_pixel_pos = {atlas_bbox.x_min + (pixel_pos.x - scene_bbox.x_min), atlas_bbox.y_min + (pixel_pos.y - scene_bbox.y_min)};
    float glyph_alpha = glyph_atlas[atlas_pixel_pos];

    float pixel_alpha = color_alpha*glyph_alpha;

    return pixel_alpha;
}

float4 blend_pd_over(float4 bg, float4 fg) {
    return lerp(bg, float4(fg.rgb, 1.0), fg.a);
}

#define NUM_CMD_OFFSET 4

[numthreads(~P_X~, ~P_Y~, 1)]
void paint_items(uint3 Gid: SV_GroupID, uint3 DTid : SV_DispatchThreadID) {
    float4 bg = {0.0, 0.0, 0.0, 0.0};
    float4 fg = {0.0, 0.0, 0.0, 0.0};

    uint2 pixel_pos = DTid.xy;

    uint tile_ix = Gid.y*num_tiles_x + Gid.x;
    uint cmd_list_size = NUM_CMD_OFFSET + num_items*PIET_ITEM_SIZE;
    uint cmd_init_offset = cmd_list_size*tile_ix;
    uint num_cmd = per_tile_command_list.Load(cmd_init_offset);
    uint cmd_item_start = cmd_init_offset + NUM_CMD_OFFSET;

    /*
    if (num_items == 2) {
        fg.g = 1.0;
        fg.a = 1.0;
    } else {
        fg.r = 1.0;
        fg.a = 1.0;
    }
    */

    /*
    for (uint i = 0; i < num_cmd; i++) {
        uint item_offset = cmd_item_start + PIET_ITEM_SIZE*i;
        uint tag = PietItem_tag(per_tile_command_list, item_offset);

        PietCirclePacked packed_circle = PietCirclePacked_read(per_tile_command_list, cmd_item_start);
        SRGBColor color = SRGBColorPacked_unpack(packed_circle.color);
        BBox scene_bbox = BBoxPacked_unpack(packed_circle.scene_bbox);

        if (tag == PietItem_Circle) {
            fg.g = 1.0;
            fg.a = 0.5;
        } else {
            fg.r = 1.0;
            fg.a = 0.5;
        }

        bg = blend_pd_over(bg, fg);
    }
    */

    for (uint i = 0; i < num_cmd; i++) {
        uint item_offset = cmd_item_start + PIET_ITEM_SIZE*i;
        uint tag = PietItem_tag(per_tile_command_list, item_offset);

        if (tag == PietItem_Circle) {
            BBoxPacked packed_scene_bbox = PietCirclePacked_scene_bbox(per_tile_command_list, item_offset);
            BBox scene_bbox = BBoxPacked_unpack(packed_scene_bbox);
            SRGBColorPacked packed_color = PietCirclePacked_color(per_tile_command_list, item_offset);
            SRGBColor color = SRGBColorPacked_unpack(packed_color);

            fg.r = color.r/255.0;
            fg.g = color.g/255.0;
            fg.b = color.b/255.0;
            fg.a = color.a/255.0;
            fg.a = circle_alpha(pixel_pos, scene_bbox, color.a/255.0);
            bg = blend_pd_over(bg, fg);
        } else if (tag == PietItem_Glyph) {
            BBoxPacked packed_scene_bbox = PietGlyphPacked_scene_bbox(per_tile_command_list, item_offset);
            BBox scene_bbox = BBoxPacked_unpack(packed_scene_bbox);

            if (is_pixel_in_bbox(pixel_pos, scene_bbox)) {
                BBoxPacked packed_atlas_bbox = PietGlyphPacked_atlas_bbox(per_tile_command_list, item_offset);
                BBox atlas_bbox = BBoxPacked_unpack(packed_atlas_bbox);

                SRGBColorPacked packed_color = PietGlyphPacked_color(per_tile_command_list, item_offset);
                SRGBColor color = SRGBColorPacked_unpack(packed_color);

                fg.r = color.r/255.0;
                fg.g = color.g/255.0;
                fg.b = color.b/255.0;
                fg.a = glyph_alpha(pixel_pos, scene_bbox, atlas_bbox, color.a/255.0);
                bg = blend_pd_over(bg, fg);
            }
        }
    }

    canvas[DTid.xy] = bg;
}
