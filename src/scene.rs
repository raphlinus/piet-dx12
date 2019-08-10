extern crate byteorder;
extern crate rand;

use crate::glyphs::{create_atlas, Atlas};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use rand::Rng;
use std::convert::TryFrom;
use std::io::Cursor;

// HLSL weirdness: bytes 0 1 2 3 will be mapped to 3 2 1 0

pub unsafe fn append_circle(
    object_data: &mut Vec<u8>,
    scene_bbox_x_min: u16,
    scene_bbox_y_min: u16,
    diameter: u16,
    color: [u8; 4],
) {
    // glyph_id
    object_data
        .write_u16::<LittleEndian>(0)
        .expect("could not convert u16 to bytes");
    // object_type
    object_data
        .write_u16::<LittleEndian>(0)
        .expect("could not convert u16 to bytes");

    // atlas_bbox_x_max
    object_data
        .write_u16::<LittleEndian>(0)
        .expect("could not convert u16 to bytes");
    // atlas_bbox_x_min
    object_data
        .write_u16::<LittleEndian>(0)
        .expect("could not convert u16 to bytes");

    // atlas_bbox_y_max
    object_data
        .write_u16::<LittleEndian>(0)
        .expect("could not convert u16 to bytes");
    // atlas_bbox_y_min
    object_data
        .write_u16::<LittleEndian>(0)
        .expect("could not convert u16 to bytes");

    // reverse order of each 4 bytes, so write component 2 first, in LE, then component 1 in LE
    object_data
        .write_u16::<LittleEndian>(scene_bbox_x_min + diameter)
        .expect("could not convert u16 to bytes");
    object_data
        .write_u16::<LittleEndian>(scene_bbox_x_min)
        .expect("could not convert u16 to bytes");

    // reverse order of each 4 bytes, so write component 2 first in LE, then component 1 in LE
    object_data
        .write_u16::<LittleEndian>(scene_bbox_y_min + diameter)
        .expect("could not convert u16 to bytes");
    object_data
        .write_u16::<LittleEndian>(scene_bbox_y_min)
        .expect("could not convert u16 to bytes");

    for component in color.iter() {
        object_data.push(*component);
    }
}

pub unsafe fn append_glyph(
    object_data: &mut Vec<u8>,
    glyph_id: u16,
    scene_bbox_x_min: u16,
    scene_bbox_y_min: u16,
    width: u16,
    height: u16,
    color: [u8; 4],
) {
    object_data
        .write_u16::<LittleEndian>(glyph_id)
        .expect("could not convert u16 to bytes");
    object_data
        .write_u16::<LittleEndian>(1)
        .expect("could not convert u16 to bytes");

    let atlas_bbox_x_min = glyph_id * 50;
    let atlas_bbox_y_min: u16 = 0;

    object_data
        .write_u16::<LittleEndian>(atlas_bbox_x_min + width)
        .expect("could not convert u16 to bytes");
    object_data
        .write_u16::<LittleEndian>(atlas_bbox_x_min)
        .expect("could not convert u16 to bytes");

    object_data
        .write_u16::<LittleEndian>(atlas_bbox_y_min + height)
        .expect("could not convert u16 to bytes");
    object_data
        .write_u16::<LittleEndian>(atlas_bbox_y_min)
        .expect("could not convert u16 to bytes");

    // reverse order of each 4 bytes, so write component 2 first, in LE, then component 1 in LE
    object_data
        .write_u16::<LittleEndian>(scene_bbox_x_min + width)
        .expect("could not convert u16 to bytes");
    object_data
        .write_u16::<LittleEndian>(scene_bbox_x_min)
        .expect("could not convert u16 to bytes");

    // reverse order of each 4 bytes, so write component 2 first in LE, then component 1 in LE
    object_data
        .write_u16::<LittleEndian>(scene_bbox_y_min + height)
        .expect("could not convert u16 to bytes");
    object_data
        .write_u16::<LittleEndian>(scene_bbox_y_min)
        .expect("could not convert u16 to bytes");

    for component in color.iter() {
        object_data.push(*component);
    }
}

pub unsafe fn append_glyph2(
    object_data: &mut Vec<u8>,
    glyph_id: u16,
    in_atlas_glyph_bbox: (u16, u16, u16, u16),
    scene_bbox_x_min: u16,
    scene_bbox_y_min: u16,
    color: [u8; 4],
) -> u32 {
    let mut object_size = 0;

    object_data
        .write_u16::<LittleEndian>(glyph_id)
        .expect("could not convert u16 to bytes");
    object_data
        .write_u16::<LittleEndian>(1)
        .expect("could not convert u16 to bytes");
    object_size += 4;

    let (width, height) = {
        (
            in_atlas_glyph_bbox.1 - in_atlas_glyph_bbox.0,
            in_atlas_glyph_bbox.3 - in_atlas_glyph_bbox.2,
        )
    };

    object_data
        .write_u16::<LittleEndian>(in_atlas_glyph_bbox.1)
        .expect("could not convert u16 to bytes");
    object_data
        .write_u16::<LittleEndian>(in_atlas_glyph_bbox.0)
        .expect("could not convert u16 to bytes");
    object_size += 4;

    object_data
        .write_u16::<LittleEndian>(in_atlas_glyph_bbox.3)
        .expect("could not convert u16 to bytes");
    object_data
        .write_u16::<LittleEndian>(in_atlas_glyph_bbox.2)
        .expect("could not convert u16 to bytes");
    object_size += 4;

    // reverse order of each 4 bytes, so write component 2 first, in LE, then component 1 in LE
    object_data
        .write_u16::<LittleEndian>(scene_bbox_x_min + width)
        .expect("could not convert u16 to bytes");
    object_data
        .write_u16::<LittleEndian>(scene_bbox_x_min)
        .expect("could not convert u16 to bytes");
    object_size += 4;

    // reverse order of each 4 bytes, so write component 2 first in LE, then component 1 in LE
    object_data
        .write_u16::<LittleEndian>(scene_bbox_y_min + height)
        .expect("could not convert u16 to bytes");
    object_data
        .write_u16::<LittleEndian>(scene_bbox_y_min)
        .expect("could not convert u16 to bytes");
    object_size += 4;

    for component in color.iter() {
        object_data.push(*component);
    }
    object_size += 4;

    object_size
}

pub unsafe fn create_random_scene(
    screen_width: u32,
    screen_height: u32,
    num_objects: u32,
) -> (u32, Vec<u8>) {
    let mut rng = rand::thread_rng();

    let mut object_data: Vec<u8> = Vec::new();
    let object_size = 24;

    for n in 0..num_objects {
        let object_type: u16 = rng.gen_range(0, 2);
        let (scene_bbox_x_min, scene_bbox_y_min): (u16, u16) = (
            rng.gen_range(0, screen_width) as u16,
            rng.gen_range(0, screen_height) as u16,
        );
        let mut color: [u8; 4] = [0; 4];
        for i in 0..4 {
            color[i] = rng.gen_range(0, 256) as u8;
        }

        if object_type == 0 {
            let diameter: u16 = rng.gen_range(20, 200);
            append_circle(
                &mut object_data,
                scene_bbox_x_min,
                scene_bbox_y_min,
                diameter,
                color,
            );
        } else {
            let glyph_id: u16 = rng.gen_range(0, 10);
            append_glyph(
                &mut object_data,
                glyph_id,
                scene_bbox_x_min,
                scene_bbox_y_min,
                50,
                50,
                color,
            );
        }
    }

    (object_size, object_data)
}

pub unsafe fn create_constant_scene(
    screen_width: u32,
    screen_height: u32,
) -> (u32, u32, Vec<u8>, Atlas) {
    let atlas = create_atlas(vec!['0', '1'], 50, 512, 50);

    let mut rng = rand::thread_rng();

    let mut object_data: Vec<u8> = Vec::new();
    let object_size = 24;

    let diameter: u16 = 200;
    let (scene_bbox_x_min, scene_bbox_y_min): (u16, u16) = (100, 100);
    let color: [u8; 4] = [255, 255, 255, 255];
    let mut num_objects: u32 = 0;
    let mut object_size: u32 = 0;

    append_circle(
        &mut object_data,
        scene_bbox_x_min,
        scene_bbox_y_min,
        diameter,
        color,
    );
    num_objects += 1;

    let x_cursor = 500;
    let gix = atlas.get_glyph_index_of_char('0');
    let in_atlas_bbox =
        atlas.glyph_bboxes[gix].expect("constant scene: character does not have a bbox");
    let glyph_advance = u16::try_from(atlas.glyph_advances[gix])
        .expect("could not safely convert u32 glyph advance into u16");
    object_size = append_glyph2(
        &mut object_data,
        gix as u16,
        in_atlas_bbox,
        x_cursor,
        scene_bbox_y_min,
        [255, 255, 255, 255],
    );
    num_objects += 1;

    (num_objects, object_size, object_data, atlas)
}

pub unsafe fn create_constant_scene2(
    screen_width: u32,
    screen_height: u32,
) -> (u32, u32, Vec<u8>, Atlas) {
    let atlas = create_atlas(vec!['0', '1'], 50, 512, 50);

    let mut rng = rand::thread_rng();

    let mut object_data: Vec<u8> = Vec::new();
    let object_size = 24;

    let diameter: u16 = 200;
    let (scene_bbox_x_min, scene_bbox_y_min): (u16, u16) = (100, 100);
    let color: [u8; 4] = [255, 255, 255, 255];
    let mut num_objects: u32 = 0;
    let mut object_size: u32 = 0;

    append_circle(
        &mut object_data,
        scene_bbox_x_min,
        scene_bbox_y_min,
        diameter,
        color,
    );
    num_objects += 1;

    let mut x_cursor = 500;

    let gix = atlas.get_glyph_index_of_char('0');
    let in_atlas_bbox =
        atlas.glyph_bboxes[gix].expect("constant scene: character does not have a bbox");
    let glyph_advance = u16::try_from(atlas.glyph_advances[gix])
        .expect("could not safely convert u32 glyph advance into u16");
    object_size = append_glyph2(
        &mut object_data,
        gix as u16,
        in_atlas_bbox,
        x_cursor,
        scene_bbox_y_min,
        [255, 255, 255, 255],
    );
    x_cursor += (in_atlas_bbox.1 - in_atlas_bbox.0) + glyph_advance;
    num_objects += 1;

    let gix = atlas.get_glyph_index_of_char('1');
    let in_atlas_bbox =
        atlas.glyph_bboxes[gix].expect("constant scene: character does not have a bbox");
    let glyph_advance = u16::try_from(atlas.glyph_advances[gix])
        .expect("could not safely convert u32 glyph advance into u16");
    object_size = append_glyph2(
        &mut object_data,
        gix as u16,
        in_atlas_bbox,
        x_cursor,
        scene_bbox_y_min,
        [255, 255, 255, 255],
    );
    x_cursor += (in_atlas_bbox.1 - in_atlas_bbox.0) + glyph_advance;
    num_objects += 1;

    (num_objects, object_size, object_data, atlas)
}

use std::collections::HashSet;
use std::hash::Hash;

// https://stackoverflow.com/a/47648303/3486684
fn dedup<T: Eq + Hash + Copy>(v: &mut Vec<T>) {
    // note the Copy constraint
    let mut uniques = HashSet::new();
    v.retain(|e| uniques.insert(*e));
}

pub unsafe fn create_text_string_scene(
    screen_x_offset: u16,
    screen_y_offset: u16,
    text_string: &str,
    font_size: u32,
) -> (u32, u32, Vec<u8>, Atlas) {
    let string_chars: Vec<char> = text_string.chars().collect();

    let glyph_chars: Vec<char> = {
        let mut result = string_chars.clone();
        dedup(&mut result);
        result
    };

    let atlas = create_atlas(glyph_chars, font_size, 512, 512);

    let mut object_data: Vec<u8> = Vec::new();
    let mut object_size: u32 = 24;

    let mut x_cursor: u16 = screen_x_offset;
    let screen_y_offset_i32 = screen_y_offset as i32;
    let mut num_objects: u32 = 0;
    for (i, &c) in string_chars.iter().enumerate() {
        let gix = atlas.get_glyph_index_of_char(c);
        let some_in_atlas_bbox = atlas.glyph_bboxes[gix];
        let glyph_advance = u16::try_from(atlas.glyph_advances[gix])
            .expect("could not safely convert u32 glyph advance into u16");
        let y_offset = u16::try_from(screen_y_offset_i32 + atlas.glyph_top_offsets[gix])
            .expect("could not safely convert i32 glyph y offset into u16");

        match some_in_atlas_bbox {
            Some(in_atlas_bbox) => {
                object_size = append_glyph2(
                    &mut object_data,
                    gix as u16,
                    in_atlas_bbox,
                    x_cursor,
                    y_offset,
                    [255, 255, 255, 255],
                );
                x_cursor += (in_atlas_bbox.1 - in_atlas_bbox.0) + (0.2 * glyph_advance as f32).round() as u16;
            }
            None => {
                x_cursor += glyph_advance;
            }
        }
        num_objects += 1;
    }

    (num_objects, object_size, object_data, atlas)
}
