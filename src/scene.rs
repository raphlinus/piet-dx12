extern crate byteorder;
extern crate rand;
extern crate kurbo;
extern crate font_rs;

use font_rs::font::Font;
use kurbo::{Circle, Rect, Point};

use crate::atlas::Atlas;
use byteorder::{LittleEndian, WriteBytesExt};
use rand::Rng;
use std::convert::TryFrom;
use std::mem;

pub enum ObjectType {
    Circle,
    Glyph,
}

pub struct GenericObject {
    object_type: u16,
    glyph_id: u16,
    in_atlas_bbox: (u16, u16, u16, u16),
    in_scene_bbox: (u16, u16, u16, u16),
    color: [u8; 4],
}

impl GenericObject {
    pub fn size_in_u32s() -> u32 {
        u32::try_from(mem::size_of::<GenericObject>() / mem::size_of::<u32>()).expect("could not convert GenericObject size in u32s into u32 value")
    }
    
    pub fn size_in_bytes() -> usize {
        mem::size_of::<GenericObject>()
    }
}

pub struct Scene {
    pub objects: Vec<GenericObject>,
    pub atlas: Atlas,
}

use std::collections::HashSet;
use std::hash::Hash;

// https://stackoverflow.com/a/47648303/3486684
fn dedup<T: Eq + Hash + Copy>(v: &mut Vec<T>) {
    // note the Copy constraint
    let mut uniques = HashSet::new();
    v.retain(|e| uniques.insert(*e));
}


impl Scene {
    pub fn new_empty(atlas_width: u16, atlas_height: u16) -> Scene {
        Scene {
            objects: Vec::new(),
            atlas: Atlas::create_empty_atlas(atlas_width, atlas_height),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut scene_in_bytes = Vec::<u8>::new();

        for object in self.objects.iter() {
            // glyph_id
            scene_in_bytes
                .write_u16::<LittleEndian>(object.glyph_id)
                .expect("could not convert u16 to bytes");
            // object_type
            scene_in_bytes
                .write_u16::<LittleEndian>(object.object_type as u16)
                .expect("could not convert u16 to bytes");

            // atlas_bbox_x_max
            scene_in_bytes
                .write_u16::<LittleEndian>(object.in_atlas_bbox.1)
                .expect("could not convert u16 to bytes");
            // atlas_bbox_x_min
            scene_in_bytes
                .write_u16::<LittleEndian>(object.in_atlas_bbox.0)
                .expect("could not convert u16 to bytes");

            // atlas_bbox_y_max
            scene_in_bytes
                .write_u16::<LittleEndian>(object.in_atlas_bbox.3)
                .expect("could not convert u16 to bytes");
            // atlas_bbox_y_min
            scene_in_bytes
                .write_u16::<LittleEndian>(object.in_atlas_bbox.2)
                .expect("could not convert u16 to bytes");

            // reverse order of each 4 bytes, so write component 2 first, in LE, then component 1 in LE
            scene_in_bytes
                .write_u16::<LittleEndian>(object.in_scene_bbox.1)
                .expect("could not convert u16 to bytes");
            scene_in_bytes
                .write_u16::<LittleEndian>(object.in_scene_bbox.0)
                .expect("could not convert u16 to bytes");

            // reverse order of each 4 bytes, so write component 2 first in LE, then component 1 in LE
            scene_in_bytes
                .write_u16::<LittleEndian>(object.in_scene_bbox.3)
                .expect("could not convert u16 to bytes");
            scene_in_bytes
                .write_u16::<LittleEndian>(object.in_scene_bbox.2)
                .expect("could not convert u16 to bytes");

            for component in object.color.iter().rev() {
                scene_in_bytes.push(*component);
            }
        }

        scene_in_bytes
    }

    pub fn append_circle(&mut self, circle: Circle, color: [u8; 4]) {
        self.objects.push(
            GenericObject {
                object_type: ObjectType::Circle as u16,
                glyph_id: 0,
                in_atlas_bbox: (0, 0, 0, 0),
                in_scene_bbox: (
                    (circle.center.x - circle.radius) as u16,
                    (circle.center.x + circle.radius) as u16,
                    (circle.center.y - circle.radius) as u16,
                    (circle.center.y + circle.radius) as u16,
                ),
                color,
            }
        );
    }

    pub fn append_glyph(
        &mut self,
        glyph_id: u16,
        in_atlas_bbox: Rect,
        in_scene_bbox: Rect,
        color: [u8; 4],
    ) {
        self.objects.push(
            GenericObject {
                object_type: ObjectType::Glyph as u16,
                glyph_id,
                in_atlas_bbox: (in_atlas_bbox.x0 as u16,
                                in_atlas_bbox.x1 as u16,
                                in_atlas_bbox.y0 as u16,
                                in_atlas_bbox.y1 as u16,),
                in_scene_bbox: (in_scene_bbox.x0 as u16,
                                in_scene_bbox.x1 as u16,
                                in_scene_bbox.y0 as u16,
                                in_scene_bbox.y1 as u16,),
                color,
            }
        );
    }

    pub fn initialize_test_scene0(
        &mut self,
        screen_width: u32,
        screen_height: u32,
    ) {
        self.objects = Vec::new();

        let (scene_bbox_x_min, scene_bbox_y_min): (u16, u16) = (100, 100);

        let mut color: [u8; 4] = [255, 255, 255, 255];

        let radius: f64 = 50.0;
        self.append_circle(
            Circle {
                center: Point {
                    x: radius + (scene_bbox_x_min as f64),
                    y: radius + (scene_bbox_y_min as f64),
                },
                radius,
            },
            color,
        );
    }

    pub fn populate_randomly(
        &mut self,
        screen_width: u32,
        screen_height: u32,
        num_objects: u32,
    ) {
        let mut rng = rand::thread_rng();

        for _ in 0..num_objects {
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
                let radius: f64 = rng.gen_range(10.0, 100.0);
                self.append_circle(
                    Circle {
                        center: Point {
                            x: radius + (scene_bbox_x_min as f64),
                            y: radius + (scene_bbox_y_min as f64),
                        },
                        radius,
                    },
                    color,
                );
            } else {
                let glyph_ix: usize = rng.gen_range(0, self.atlas.glyph_count as usize);

                match self.atlas.glyph_bboxes[glyph_ix] {
                    Some(glyph_bbox) => {
                        self.append_glyph(
                            glyph_ix as u16,
                            Rect {
                                x0: glyph_bbox.0 as f64,
                                x1: glyph_bbox.1 as f64,
                                y0: glyph_bbox.2 as f64,
                                y1: glyph_bbox.3 as f64,
                            },
                            Rect {
                                x0: scene_bbox_x_min as f64,
                                x1: (scene_bbox_x_min + (glyph_bbox.1 - glyph_bbox.0)) as f64,
                                y0: scene_bbox_y_min as f64,
                                y1: (scene_bbox_y_min + (glyph_bbox.3 -  glyph_bbox.2)) as f64,
                            },
                            color,
                        );
                    },
                    None => {},
                }
            }
        }
    }

    pub fn add_characters_to_atlas(&mut self, characters: &str, font_size: u32, font: &Font) {
        for c in characters.chars() {
            self.atlas.insert_character(c, font_size, font);
        }
    }

    pub fn add_text(&mut self, screen_x_offset: u16, screen_y_offset: u16, text_string: &str, font_size: u32, font: &Font) {
        let string_chars: Vec<char> = text_string.chars().collect();

        let glyph_chars: Vec<char> = {
            let mut result = string_chars.clone();
            dedup(&mut result);
            result
        };

        for &gc in glyph_chars.iter() {
            self.atlas.insert_character(gc, font_size, font);
        }

        let mut x_cursor: u16 = screen_x_offset;
        let screen_y_offset_i32 = screen_y_offset as i32;

        for &c in string_chars.iter(){
            let gix = self.atlas.get_glyph_index_of_char(c, font_size);
            let glyph_advance = u16::try_from(self.atlas.glyph_advances[gix])
                .expect("could not safely convert u32 glyph advance into u16");
            let y_offset = u16::try_from(screen_y_offset_i32 + self.atlas.glyph_top_offsets[gix])
                .expect("could not safely convert i32 glyph y offset into u16");

            match self.atlas.glyph_bboxes[gix] {
                Some(in_atlas_bbox) => {
                    let (w, h) = {
                        (in_atlas_bbox.1 - in_atlas_bbox.0, in_atlas_bbox.3 - in_atlas_bbox.2)
                    };

                    let in_scene_bbox_rect = Rect {
                        x0: x_cursor as f64,
                        x1: (x_cursor + w) as f64,
                        y0: y_offset as f64,
                        y1: (y_offset + h) as f64,
                    };

                    let in_atlas_bbox_rect = Rect {
                      x0: in_atlas_bbox.0 as f64,
                        x1: in_atlas_bbox.1 as f64,
                        y0: in_atlas_bbox.2 as f64,
                        y1: in_atlas_bbox.3 as f64,
                    };

                    self.append_glyph(
                        gix as u16,
                        in_atlas_bbox_rect,
                        in_scene_bbox_rect,
                        [255, 255, 255, 255],
                    );
                    x_cursor += (in_atlas_bbox.1 - in_atlas_bbox.0) + (0.2 * glyph_advance as f32).round() as u16;
                }
                None => {
                    x_cursor += glyph_advance;
                }
            }
        }
    }
}
