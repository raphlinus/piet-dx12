extern crate font_rs;
extern crate image;

use self::image::GenericImageView;
use crate::dx12;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use font_rs::font::{parse, Font, GlyphBitmap};

#[derive(Clone)]
pub struct Atlas {
    pub bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub glyph_bboxes: Vec<(u16, u16, u16, u16)>,
    cursor: u16,
    char_to_ix_map: HashMap<char, usize>,
    pub glyph_advances: Vec<u32>,
}

impl Atlas {
    pub fn generate_atlas(
        atlas_width: u16,
        atlas_height: u16,
        glyph_chars: Vec<char>,
        glyph_ids: Vec<u16>,
        glyph_bitmaps: Vec<GlyphBitmap>,
        glyph_advances: Vec<u32>,
    ) -> Atlas {
        let mut char_to_ix_map = HashMap::new();

        for (i, &c) in glyph_chars.iter().enumerate() {
            char_to_ix_map.insert(c, i);
        }

        let mut atlas = Atlas {
            bytes: vec![0; (atlas_width as usize) * (atlas_height as usize) * std::mem::size_of::<u8>()],
            width: atlas_width,
            height: atlas_height,
            glyph_bboxes: Vec::new(),
            cursor: 0,
            char_to_ix_map,
            glyph_advances,
        };

        let glyph_bboxes: Vec<(u16, u16, u16, u16)> = {
            glyph_chars.iter().zip(&glyph_bitmaps)
                .map(|(&gc, gb)| {
                    let gw = u16::try_from(gb.width)
                        .expect("could not safely convert glyph bitmap width to u16");
                    let gh = u16::try_from(gb.height)
                        .expect("could not safely convert glyph bitmap width to u16");
                    println!("atlas: allocating space for glyph of character '{}'", gc);
                    let tl = atlas
                        .allocate_rect(gw, gh)
                        .expect(&format!("atlas: could not allocate space for glyph of character '{}', with size ({}, {})", gc, gw, gh));
                    (tl.0, tl.0 + gw, tl.1, tl.1 + gh)
                })
                .collect()
        };
        atlas.glyph_bboxes = glyph_bboxes;

        let atlas_row_stride = (atlas.width as usize) * std::mem::size_of::<u8>();

        for i in 0..glyph_chars.len() {
            let gb = &glyph_bitmaps[i];

            //            let gb_num_pixels = gb.width * gb.height;
            //            println!("gb num pixels: {}", gb_num_pixels);
            //            println!("gb expected num bytes: {}", gb_num_pixels*std::mem::size_of::<u8>());
            //            println!("gb actual num bytes: {}", gb.data.len());

            let gbbox = atlas.glyph_bboxes[i];

            let glyph_row_stride = gb.width * std::mem::size_of::<u8>();

            let x_offset_in_atlas = (gbbox.0 as usize) * std::mem::size_of::<u8>();
            for hix in 0..gb.height {
                let start_address = x_offset_in_atlas + hix * atlas_row_stride;
                for ro in 0..glyph_row_stride {
                    atlas.bytes[start_address + ro] = gb.data[hix * glyph_row_stride + ro];
                }
                // atlas.bytes[start_address..(start_address + glyph_row_stride)] = gb.data[hix*glyph_row_stride..(hix + 1)*glyph_row_stride];
            }
        }

        atlas
    }

    pub fn allocate_rect(&mut self, w: u16, h: u16) -> Result<(u16, u16), ()> {
        assert!(h < self.height);
        if self.cursor + w > self.width {
            Err(())
        } else {
            let result: (u16, u16) = (self.cursor, 0);
            self.cursor += w;

            Ok(result)
        }
    }

    pub fn to_subresource_data(&self) -> dx12::SubresourceData {
        assert_eq!(self.width % 256, 0);

        dx12::SubresourceData {
            data: self.bytes.clone(),
            row_size: self.width as isize,
            column_size: self.height as isize,
        }
    }

    fn expand_bytes_to_rgba(&self) -> Vec<u8> {
        let mut expanded_bytes: Vec<u8> = vec![0; self.bytes.len() * 4];

        for i in 0..self.bytes.len() {
            expanded_bytes[i * 4 + 3] = self.bytes[i];
        }

        expanded_bytes
    }

    pub fn dump_bytes_as_rgba_image(&self) {
        let expanded_bytes = self.expand_bytes_to_rgba();
        let filename: PathBuf = ["resources", "raw-atlas-dump.png"].iter().collect();
        image::save_buffer(
            &filename,
            &expanded_bytes,
            self.width as u32,
            self.height as u32,
            image::RGBA(8),
        )
        .expect("failed to dump raw atlas as png image");
    }

    pub fn get_glyph_index_of_char(&self, c: char) -> usize {
        match self.char_to_ix_map.get(&c) {
            Some(v) => *v,
            None => panic!("atlas: could not find char '{}'", c),
        }
    }
}

pub fn create_atlas(glyph_chars: Vec<char>, font_size: u32, atlas_width: u16, atlas_height: u16) -> Atlas {
    let filename: PathBuf = ["resources", "notomono", "NotoMono-Regular.ttf"]
        .iter()
        .collect();
    let mut f = File::open(&filename).unwrap();
    let mut data = Vec::new();

    let str_filename = filename
        .to_str()
        .expect("could not convert filename to string");

    let font = match f.read_to_end(&mut data) {
        Err(e) => panic!("failed to read {}, {}", str_filename, e),
        Ok(_) => match parse(&data) {
            Ok(font) => font,
            Err(_) => panic!("failed to parse {}", str_filename),
        },
    };

    //    let glyph_chars: Vec<char> = (0_u32..10_u32)
    //        .map(|d| {
    //            std::char::from_digit(d, 10).expect(&format!("could not convert digit {} to char", d))
    //        })
    //        .collect();
    let glyph_cps: Vec<u32> = glyph_chars.iter().map(|&c| c as u32).collect();
    let glyph_ids: Vec<u16> = glyph_chars
        .iter()
        .zip(glyph_cps.iter())
        .map(|(gc, &gcp)| {
            font.lookup_glyph_id(gcp)
                .expect(&format!("error looking up glyph id of character: {}", gc))
        })
        .collect();
    let glyph_advances: Vec<u32> = glyph_chars
        .iter()
        .zip(&glyph_ids)
        .map(|(gc, &gid)| {
            font.get_h_metrics(gid, font_size)
                .expect(&format!(
                    "could not retrieve h metrics for glyph '{}' in font",
                    gc
                ))
                .advance_width
                .round() as u32
        })
        .collect();
    let rendered_glyphs: Vec<GlyphBitmap> = glyph_chars
        .iter()
        .zip(&glyph_ids)
        .map(|(gc, &gid)| {
            font.render_glyph(gid, font_size)
                .expect(&format!("error rendering glyph: {}", gc))
        })
        .collect();

    let atlas = Atlas::generate_atlas(
        atlas_width,
        atlas_height,
        glyph_chars,
        glyph_ids,
        rendered_glyphs,
        glyph_advances,
    );

    atlas
}
