extern crate font_rs;
extern crate image;

use self::image::GenericImageView;
use crate::dx12;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use font_rs::font::{parse, Font, GlyphBitmap, VMetrics};

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
    pub bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub glyph_bboxes: Vec<Option<(u16, u16, u16, u16)>>,
    cursor: u16,
    char_to_ix_map: HashMap<char, usize>,
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
    pub fn generate_atlas(
        atlas_width: u16,
        atlas_height: u16,
        glyph_chars: Vec<char>,
        glyph_ids: Vec<u16>,
        glyph_bitmaps: Vec<Option<GlyphBitmap>>,
        glyph_advances: Vec<u32>,
        glyph_top_offsets: Vec<i32>,
    ) -> Atlas {
        let mut char_to_ix_map = HashMap::new();

        for (i, &c) in glyph_chars.iter().enumerate() {
            char_to_ix_map.insert(c, i);
        }

        let free_list: Vec<Vec<AtlasCursor>> = (0..16).map(|_| Vec::<AtlasCursor>::new()).collect();

        let mut atlas = Atlas {
            bytes: vec![
                0;
                (atlas_width as usize) * (atlas_height as usize) * std::mem::size_of::<u8>()
            ],
            width: atlas_width,
            height: atlas_height,
            glyph_bboxes: Vec::new(),
            cursor: 0,
            char_to_ix_map,
            glyph_advances,
            glyph_top_offsets,
            strip_free_lists: free_list,
        };

        let glyph_bboxes: Vec<Option<(u16, u16, u16, u16)>> = {
            glyph_chars.iter().zip(&glyph_bitmaps)
                .map(|(&gc, ogb)| {
                    match ogb {
                        Some(gb) => {
                            let gw = u16::try_from(gb.width)
                                .expect("could not safely convert glyph bitmap width to u16");
                            let gh = u16::try_from(gb.height)
                                .expect("could not safely convert glyph bitmap width to u16");
                            println!("atlas: allocating space for glyph of character '{}'", gc);
                            let tl = match atlas.allocate_rect(gw, gh) {
                                Ok(result) => {
                                    result
                                },
                                Err(err) => {
                                    panic!("atlas: could not allocate space for glyph of character '{}', with size ({}, {}), error: {}", gc, gw, gh, err);
                                }
                            };

                            let bbox = (tl.0, tl.0 + gw, tl.1, tl.1 + gh);
                            println!("new glyph bbox: {}, {}, {}, {}", bbox.0, bbox.1, bbox.2, bbox.3);
                            Some(bbox)
                        },
                        None => {
                            None
                        }
                    }
                })
                .collect()
        };
        atlas.glyph_bboxes = glyph_bboxes;

        let atlas_row_stride = (atlas.width as usize) * std::mem::size_of::<u8>();

        for i in 0..glyph_chars.len() {
            match &glyph_bitmaps[i] {
                Some(gb) => {
                    let gbbox = atlas.glyph_bboxes[i].expect(&format!(
                        "glyph bitmap exists, but not bbox, for character: {}",
                        glyph_chars[i]
                    ));

                    let glyph_row_stride = gb.width * std::mem::size_of::<u8>();

                    let x_offset_in_atlas = (gbbox.0 as usize) * std::mem::size_of::<u8>();
                    for hix in 0..gb.height {
                        let start_address =
                            x_offset_in_atlas + (hix + (gbbox.2 as usize)) * atlas_row_stride;
                        for ro in 0..glyph_row_stride {
                            atlas.bytes[start_address + ro] = gb.data[hix * glyph_row_stride + ro];
                        }
                        // atlas.bytes[start_address..(start_address + glyph_row_stride)] = gb.data[hix*glyph_row_stride..(hix + 1)*glyph_row_stride];
                    }
                }
                None => {}
            }
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

    pub fn get_glyph_index_of_char(&self, c: char) -> usize {
        match self.char_to_ix_map.get(&c) {
            Some(v) => *v,
            None => panic!("atlas: could not find char '{}'", c),
        }
    }
}

pub fn create_atlas(
    glyph_chars: Vec<char>,
    font_size: u32,
    atlas_width: u16,
    atlas_height: u16,
) -> Atlas {
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

    let rendered_glyphs: Vec<Option<GlyphBitmap>> = glyph_chars
        .iter()
        .zip(&glyph_ids)
        .map(|(gc, &gid)| font.render_glyph(gid, font_size))
        .collect();

    let glyph_top_offsets: Vec<i32> = glyph_chars
        .iter()
        .zip(&rendered_glyphs)
        .map(|(gc, ogb)| match ogb {
            Some(gb) => gb.top,
            None => 0,
        })
        .collect();

    let atlas = Atlas::generate_atlas(
        atlas_width,
        atlas_height,
        glyph_chars,
        glyph_ids,
        rendered_glyphs,
        glyph_advances,
        glyph_top_offsets,
    );

    atlas
}
