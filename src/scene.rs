extern crate byteorder;
extern crate rand;
extern crate kurbo;

use kurbo::{Shape, Circle, Rect, Point};

use crate::glyphs::{create_atlas, Atlas};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use rand::Rng;
use std::convert::TryFrom;
use std::io::Cursor;

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
pub struct Scene {
    objects: Vec<GenericObject>,
}

impl Scene {
    pub fn empty_new() -> Scene {
        Scene {
            objects: Vec::new(),
        }
    }

    pub unsafe fn to_bytes(&self) -> Vec<u8> {
        let mut scene_in_bytes = Vec::<u8>::new();

        for object in self.objects {
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

            for component in color.iter().rev() {
                scene_in_bytes.push(*component);
            }
        }

        scene_in_bytes
    }
    pub unsafe fn append_circle(&mut self, circle: Circle, color: [u8; 4]) {
        self.objects.push(
            GenericObject {
                object_type: ObjectType::Circle as u16,
                glyph_id: 0,
                in_atlas_bbox: (0, 0, 0, 0),
                in_scene_bbox: (u16::try_from(circle.center.0 - circle.radius).expect("could not convert circle bbox x_min to u16"),
                                u16::try_from(circle.center.0 + circle.radius).expect("could not convert circle bbox x_max to u16"),
                                u16::try_from(circle.center.1 - circle.radius).expect("could not convert circle bbox y_min to u16"),
                                u16::try_from(circle.center.1 + circle.radius).expect("could not convert circle bbox y_max to u16")),
                color,
            }
        );
    }

    pub unsafe fn append_glyph(
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
                in_atlas_bbox: (u16::try_from(in_atlas_bbox.x0).expect("could not convert glyph's in atlas bbox x_min to u16"),
                                u16::try_from(in_atlas_bbox.x1).expect("could not convert glyph's in atlas bbox x_max to u16"),
                                u16::try_from(in_atlas_bbox.y0).expect("could not convert glyph's in atlas bbox y_min to u16"),
                                u16::try_from(in_atlas_bbox.y1).expect("could not convert glyph's in atlas bbox y_max to u16")),
                in_scene_bbox: (u16::try_from(in_scene_bbox.x0).expect("could not convert glyph's in scene bbox x_min to u16"),
                                u16::try_from(in_scene_bbox.x1).expect("could not convert glyph's in scene bbox x_max to u16"),
                                u16::try_from(in_scene_bbox.y0).expect("could not convert glyph's in scene bbox y_min to u16"),
                                u16::try_from(in_scene_bbox.y1).expect("could not convert glyph's in scene bbox y_max to u16")),
                color,
            }
        );
    }

    pub unsafe fn populate_randomly(
        &mut self,
        screen_width: u32,
        screen_height: u32,
        num_objects: u32,
        glyph_atlas: Atlas,
    ) {
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
                let glyph_ix: u16 = rng.gen_range(0, glyph_atlas.glyph_bboxes.len());
                let glyph_bbox = glyph_atlas.glyph_bboxes[glyph_ix];

                self.append_glyph(
                    glyph_ix,
                    Rect {
                        x0: glyph_atlas.glyph_bboxes[glyph_ix].0 as f64,
                        x1: glyph_atlas.glyph_bboxes[glyph_ix].1 as f64,
                        y0: glyph_atlas.glyph_bboxes[glyph_ix].2 as f64,
                        y1: glyph_atlas.glyph_bboxes[glyph_ix].3 as f64,
                    },
                    Rect {
                        x0: scene_bbox_x_min as f64,
                        x1: (glyph_atlas.glyph_bboxes[glyph_ix].1 - glyph_atlas.glyph_bboxes[glyph_ix].0) as f64,
                        y0: scene_bbox_y_min as f64,
                        y1: (glyph_atlas.glyph_bboxes[glyph_ix].3 -  glyph_atlas.glyph_bboxes[glyph_ix].2) as f64,
                    },
                    color,
                );
            }
        }
    }

    pub unsafe fn add_text(&mut self, screen_x_offset: u16, screen_y_offset: u16, text_string: &str, font_size: u32) {

    }
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
