extern crate font_rs;
extern crate image;
use crate::dx12;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use font_rs::font::{parse, Font};

#[derive(Clone, Copy)]
pub struct AtlasCursor {
    x: u16,
    y: u16,
    strip_height: u16,
}

impl fmt::Display for AtlasCursor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AtlasCursor {{x: {}, y: {}, strip_height: {}}}",
            self.x, self.y, self.strip_height
        )
    }
}

#[derive(Clone)]
pub struct Atlas {
    pub glyph_count: usize,
    pub bytes: Vec<u8>,
    pub width: u16,
    row_stride: usize,
    pub height: u16,
    pub glyph_bboxes: Vec<Option<(u16, u16, u16, u16)>>,
    cursor: u16,
    //TODO: distinguish by character, font size, and font; not just character and font size!
    char_to_ix_map: HashMap<(char, u32), usize>,
    pub glyph_advances: Vec<u32>,
    pub glyph_top_offsets: Vec<i32>,
    pub strip_free_lists: Vec<Vec<AtlasCursor>>,
}

#[inline]
fn approx_log2(x: u16) -> u16 {
    if x < 4 {
        x
    } else {
        let y = x - 1;
        16_u16 - (y.leading_zeros() as u16)
    }
}

pub enum AllocationError {
    TooTall,
    TooWide,
    MassiveImage,
    AtlasFull,
}

impl fmt::Display for AllocationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error_string = match self {
            AllocationError::TooTall => "too tall",
            AllocationError::TooWide => "too wide",
            AllocationError::MassiveImage => "massive image",
            AllocationError::AtlasFull => "atlas full",
        };

        write!(f, "{}", error_string)
    }
}

impl Atlas {
    pub fn create_empty_atlas(atlas_width: u16, atlas_height: u16) -> Atlas {
        let free_list: Vec<Vec<AtlasCursor>> = (0..16).map(|_| Vec::<AtlasCursor>::new()).collect();
        let row_stride = (atlas_width as usize) * std::mem::size_of::<u8>();

        Atlas {
            glyph_count: 0,
            bytes: vec![
                0;
                (atlas_width as usize) * (atlas_height as usize) * std::mem::size_of::<u8>()
            ],
            width: atlas_width,
            row_stride,
            height: atlas_height,
            glyph_bboxes: Vec::new(),
            cursor: 0,
            char_to_ix_map: HashMap::new(),
            glyph_advances: Vec::<u32>::new(),
            glyph_top_offsets: Vec::<i32>::new(),
            strip_free_lists: free_list,
        }
    }

    pub fn insert_character(&mut self, c: char, font_size: u32, font: &Font) {
        //TODO: distinguish by character, font size, and font; not just character and font size!
        if self.char_to_ix_map.contains_key(&(c, font_size)) {
            return;
        }

        let glyph_code_point = c as u32;
        let glyph_id = font.lookup_glyph_id(glyph_code_point).expect(&format!("error looking up glyph id of character: {}", c));

        let glyph_advance = font.get_h_metrics(glyph_id, font_size)
            .expect(&format!(
                "could not retrieve h metrics for glyph '{}' in font",
                c
            ))
            .advance_width
            .round() as u32;

        let glyph_bitmap = font.render_glyph(glyph_id, font_size);
        let glyph_top_offset = match &glyph_bitmap {
            Some(gb) => gb.top,
            None => 0,
        };

        self.char_to_ix_map.insert((c, font_size), self.glyph_count);
        self.glyph_count += 1;

        let in_atlas_bbox = match &glyph_bitmap {
            Some(gb) => {
                let gw = u16::try_from(gb.width)
                    .expect("could not safely convert glyph bitmap width to u16");
                let gh = u16::try_from(gb.height)
                    .expect("could not safely convert glyph bitmap width to u16");
                // println!("atlas: allocating space for glyph of character '{}'", gc);
                let tl = match self.allocate_rect(gw, gh) {
                    Ok(result) => {
                        result
                    },
                    Err(err) => {
                        panic!("atlas: could not allocate space for glyph of character '{}', with size ({}, {}), error: {}", c, gw, gh, err);
                    }
                };

                let gbbox = (tl.0, tl.0 + gw, tl.1, tl.1 + gh);
                // println!("new glyph bbox: {}, {}, {}, {}", gbbox.0, gbbox.1, gbbox.2, gbbox.3);

                let glyph_row_stride = gb.width * std::mem::size_of::<u8>();

                let x_offset_in_atlas = (gbbox.0 as usize) * std::mem::size_of::<u8>();
                for hix in 0..gb.height {
                    let start_address =
                        x_offset_in_atlas + (hix + (gbbox.2 as usize)) * self.row_stride;
                    for ro in 0..glyph_row_stride {
                        self.bytes[start_address + ro] = gb.data[hix * glyph_row_stride + ro];
                    }
                }

                Some(gbbox)
            },
            None => {
                None
            }
        };
        self.glyph_bboxes.push(in_atlas_bbox);

        self.glyph_advances.push(glyph_advance);

        self.glyph_top_offsets.push(glyph_top_offset);
    }

    pub fn generate_atlas(
        atlas_width: u16,
        atlas_height: u16,
        glyph_chars: Vec<char>,
        glyph_font_sizes: Vec<u32>,
        fonts: Vec<&Font>,
    ) -> Atlas {
        let mut atlas = Atlas::create_empty_atlas(atlas_width, atlas_height);

        for i in 0..glyph_chars.len() {
            atlas.insert_character(glyph_chars[i], glyph_font_sizes[i], fonts[i]);
        }

        atlas
    }

    fn find_new_strip_cursor_y(&self) -> u16 {
        self.strip_free_lists
            .iter()
            .map(|l| match l.last() {
                Some(ac) => ac.y + ac.strip_height,
                None => 0,
            })
            .max()
            .expect("could not determine where to place new cursor!")
    }

    pub fn allocate_rect(&mut self, w: u16, h: u16) -> Result<(u16, u16), AllocationError> {
        println!("===========");
        println!("h: {}", h);
        if h > self.height {
            println!("===========");
            Err(AllocationError::TooTall)
        } else if w > self.width {
            println!("===========");
            Err(AllocationError::TooWide)
        } else {
            let log_h = approx_log2(h);
            println!("log_h: {}", log_h);
            let fli = (log_h - 1) as usize;
            println!("fli: {}", fli);

            if fli > 15 {
                println!("===========");
                Err(AllocationError::MassiveImage)
            } else {
                match self.strip_free_lists[fli].last() {
                    Some(&ac) => {
                        println!("existing ac: {}", ac);
                        if (ac.x + w) > self.width {
                            println!("ac.x + w > self.width...determining new strip");
                            // calculate 2^log_h
                            let strip_height = 1 << log_h;
                            println!("strip height: {}", strip_height);
                            let new_cursor_y = self.find_new_strip_cursor_y();
                            println!("found new cursor y: {}", new_cursor_y);

                            if new_cursor_y + strip_height > self.height {
                                println!("===========");
                                Err(AllocationError::AtlasFull)
                            } else {
                                let new_cursor = AtlasCursor {
                                    x: w,
                                    y: new_cursor_y,
                                    strip_height,
                                };

                                println!("appending new cursor: {}", new_cursor);
                                self.strip_free_lists[fli].push(new_cursor);

                                println!("===========");
                                Ok((0, new_cursor_y + (strip_height - h)))
                            }
                        } else {
                            let new_cursor = AtlasCursor {
                                x: ac.x + w,
                                y: ac.y,
                                strip_height: ac.strip_height,
                            };

                            println!("appending new cursor: {}", new_cursor);
                            self.strip_free_lists[fli].push(new_cursor);

                            let tl = (ac.x, ac.y + (ac.strip_height - h));
                            println!("tl: {}, {}", tl.0, tl.1);

                            println!("===========");
                            Ok(tl)
                        }
                    }
                    None => {
                        // calculate 2^log_h
                        let strip_height = 1 << log_h;
                        println!("strip height: {}", strip_height);
                        let new_cursor_y = self.find_new_strip_cursor_y();

                        if new_cursor_y + strip_height > self.height {
                            println!("===========");
                            Err(AllocationError::AtlasFull)
                        } else {
                            let new_cursor = AtlasCursor {
                                x: w,
                                y: new_cursor_y,
                                strip_height,
                            };

                            println!("appending new cursor: {}", new_cursor);
                            self.strip_free_lists[fli].push(new_cursor);

                            let tl = (0, new_cursor_y + (strip_height - h));
                            println!("tl: {}, {}", tl.0, tl.1);

                            println!("===========");
                            Ok(tl)
                        }
                    }
                }
            }
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

    pub fn get_glyph_index_of_char(&self, c: char, font_size: u32) -> usize {
        //TODO: distinguish by character, font size, and font; not just character and font size!
        match self.char_to_ix_map.get(&(c, font_size)) {
            Some(v) => *v,
            None => panic!("atlas: could not find char '{}'", c),
        }
    }
}

pub fn load_font<'a>() -> Font<'a> {
    let filename: PathBuf = ["resources", "notomono", "NotoMono-Regular.ttf"]
        .iter()
        .collect();
    let mut f = File::open(&filename).unwrap();
    let mut data = Vec::new();

    let str_filename = filename
        .to_str()
        .expect("could not convert filename to string");

    match f.read_to_end(&mut data) {
        Err(e) => panic!("failed to read {}, {}", str_filename, e),
        Ok(_) => match parse(&data) {
            Ok(font) => font,
            Err(_) => panic!("failed to parse {}", str_filename),
        },
    }
}
