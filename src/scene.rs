extern crate byteorder;
extern crate rand;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use rand::Rng;
use std::io::Cursor;

// HLSL weirdness: bytes 0 1 2 3 will be mapped to 3 2 1 0

pub unsafe fn create_random_scene(
    screen_width: u32,
    screen_height: u32,
    num_objects: u32,
) -> (u32, Vec<u8>) {
    let mut rng = rand::thread_rng();

    let mut object_data: Vec<u8> = Vec::new();
    let mut object_size = 0;

    for n in 0..num_objects {
        let object_type: u16 = rng.gen_range(0, 2);

        let (
            width,
            height,
            glyph_index,
            atlas_bbox_x_min,
            atlas_bbox_x_max,
            atlas_bbox_y_min,
            atlas_bbox_y_max,
        ): (u16, u16, u16, u16, u16, u16, u16) = if (object_type == 0) {
            let diameter: u16 = rng.gen_range(20, 200);
            (diameter, diameter, 0, 0, 0, 0, 0)
        } else {
            let glyph_index: u16 = rng.gen_range(0, 10);
            // use knowledge that toy texture atlas has only 50x50 glyphs
            let (left, right, top, bot) = {
                let left = glyph_index * 50;
                let right = left + 50;
                let top = 0;
                let bot = top + glyph_index;

                (left, right, top, bot)
            };

            (50, 50, glyph_index, left, right, top, bot)
        };

        let scene_bbox_min_x = rng.gen_range(0, screen_width as u16);
        let scene_bbox_min_y = rng.gen_range(0, screen_height as u16);

        object_data
            .write_u16::<LittleEndian>(glyph_index)
            .expect("could not convert u32 to bytes");
        object_data
            .write_u16::<LittleEndian>(object_type)
            .expect("could not convert u32 to bytes");
        object_size += 4;

        object_data
            .write_u16::<LittleEndian>(atlas_bbox_x_max)
            .expect("could not convert u32 to bytes");
        object_data
            .write_u16::<LittleEndian>(atlas_bbox_x_min)
            .expect("could not convert u32 to bytes");
        object_size += 4;

        object_data
            .write_u16::<LittleEndian>(atlas_bbox_y_max)
            .expect("could not convert u32 to bytes");
        object_data
            .write_u16::<LittleEndian>(atlas_bbox_y_min)
            .expect("could not convert u32 to bytes");
        object_size += 4;

        // reverse order of each 4 bytes, so write component 2 first, in LE, then component 1 in LE
        object_data
            .write_u16::<LittleEndian>(scene_bbox_min_x + width)
            .expect("could not convert u16 to bytes");
        object_data
            .write_u16::<LittleEndian>(scene_bbox_min_x)
            .expect("could not convert u16 to bytes");
        object_size += 4;

        // reverse order of each 4 bytes, so write component 2 first in LE, then component 1 in LE
        object_data
            .write_u16::<LittleEndian>(scene_bbox_min_y + height)
            .expect("could not convert u16 to bytes");
        object_data
            .write_u16::<LittleEndian>(scene_bbox_min_y)
            .expect("could not convert u16 to bytes");
        object_size += 4;

        // order doesn't matter for randomly generated color values;
        // for real color values order will have to be reversed
        for i in 0..4 {
            object_data.push(rng.gen());
        }
        object_size += 4;
    }

    (object_size, object_data)
}
