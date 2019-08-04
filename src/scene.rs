extern crate byteorder;
extern crate rand;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use rand::Rng;
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
        .expect("could not convert u32 to bytes");
    // object_type
    object_data
        .write_u16::<LittleEndian>(0)
        .expect("could not convert u32 to bytes");

    // atlas_bbox_x_max
    object_data
        .write_u16::<LittleEndian>(0)
        .expect("could not convert u32 to bytes");
    // atlas_bbox_x_min
    object_data
        .write_u16::<LittleEndian>(0)
        .expect("could not convert u32 to bytes");

    // atlas_bbox_y_max
    object_data
        .write_u16::<LittleEndian>(0)
        .expect("could not convert u32 to bytes");
    // atlas_bbox_y_min
    object_data
        .write_u16::<LittleEndian>(0)
        .expect("could not convert u32 to bytes");

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
        .expect("could not convert u32 to bytes");
    object_data
        .write_u16::<LittleEndian>(1)
        .expect("could not convert u32 to bytes");

    let atlas_bbox_x_min = glyph_id * 50;
    let atlas_bbox_y_min: u16 = 0;

    object_data
        .write_u16::<LittleEndian>(atlas_bbox_x_min + width)
        .expect("could not convert u32 to bytes");
    object_data
        .write_u16::<LittleEndian>(atlas_bbox_x_min)
        .expect("could not convert u32 to bytes");

    object_data
        .write_u16::<LittleEndian>(atlas_bbox_y_min + height)
        .expect("could not convert u32 to bytes");
    object_data
        .write_u16::<LittleEndian>(atlas_bbox_y_min)
        .expect("could not convert u32 to bytes");

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

pub unsafe fn create_constant_scene(screen_width: u32, screen_height: u32) -> (u32, Vec<u8>) {
    let mut rng = rand::thread_rng();

    let mut object_data: Vec<u8> = Vec::new();
    let object_size = 24;

    let diameter: u16 = 200;
    let (scene_bbox_x_min, scene_bbox_y_min): (u16, u16) = (100, 100);
    let color: [u8; 4] = [255, 255, 255, 255];

    append_circle(&mut object_data, scene_bbox_x_min, scene_bbox_y_min, diameter, color);
    append_glyph(
        &mut object_data,
        0,
        scene_bbox_x_min + 500,
        scene_bbox_y_min,
        50,
        50,
        color,
    );

    (object_size, object_data)
}
