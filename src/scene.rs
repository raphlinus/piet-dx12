extern crate rand;
extern crate byteorder;

use rand::Rng;
use byteorder::{BigEndian, LittleEndian, WriteBytesExt, ReadBytesExt};
use std::io::Cursor;

// HLSL weirdness: bytes 0 1 2 3 will be mapped to 3 2 1 0

pub unsafe fn create_random_scene(screen_width: u32, screen_height: u32, num_circles: u32) -> (Vec<u8>, Vec<u8>) {
    let mut rng = rand::thread_rng();

    let mut bbox_data: Vec<u8> = Vec::new();
    let mut color_data: Vec<u8> = Vec::new();

    for n in 0..num_circles {
        let diameter: u16 = rng.gen_range(20, 200);
        let bbox_min_x: u16 = rng.gen_range(0, screen_width as u16);
        let bbox_min_y: u16 = rng.gen_range(0, screen_height as u16);

        // reverse order of each 4 bytes, so write component 2 first, in LE, then component 1 in LE
        bbox_data.write_u16::<LittleEndian>(bbox_min_x + diameter).expect("could not convert u16 to bytes");
        bbox_data.write_u16::<LittleEndian>(bbox_min_x).expect("could not convert u16 to bytes");

        // reverse order of each 4 bytes, so write component 2 first in LE, then component 1 in LE
        bbox_data.write_u16::<LittleEndian>(bbox_min_y + diameter).expect("could not convert u16 to bytes");
        bbox_data.write_u16::<LittleEndian>(bbox_min_y).expect("could not convert u16 to bytes");


        for i in 0..4 {
            color_data.push(rng.gen());
        }
    }

    // order doesn't matter for randomly generated color values; for real color values order will have to be reversed
    (bbox_data, color_data)
}

pub unsafe fn create_constant_scene(screen_width: u32, screen_height: u32, num_circles: u32) -> (Vec<u8>, Vec<u8>) {
    let mut rng = rand::thread_rng();

    let mut bbox_data: Vec<u8> = Vec::new();
    let mut color_data: Vec<u8> = Vec::new();

    let diameter: u16 = 100;
    let bbox_min_x: u16 = 100;
    let bbox_min_y: u16 = 100;

    // reverse order of each 4 bytes, so write component 2 first, in LE, then component 1 in LE
    bbox_data.write_u16::<LittleEndian>(bbox_min_x + diameter).expect("could not convert u16 to bytes");
    bbox_data.write_u16::<LittleEndian>(bbox_min_x).expect("could not convert u16 to bytes");

    // reverse order of each 4 bytes, so write component 2 first in LE, then component 1 in LE
    bbox_data.write_u16::<LittleEndian>(bbox_min_y + diameter).expect("could not convert u16 to bytes");
    bbox_data.write_u16::<LittleEndian>(bbox_min_y).expect("could not convert u16 to bytes");

    // order doesn't matter for randomly generated color values; for real color values order will have to be reversed
    for i in 0..4 {
        color_data.push(255);
    }

    (bbox_data, color_data)
}
