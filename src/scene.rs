// Copyright © 2019 piet-dx12 developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate byteorder;
extern crate font_rs;
extern crate kurbo;
extern crate rand;

use kurbo::{Circle, Point, Rect};

use byteorder::{LittleEndian, WriteBytesExt};
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

pub struct PlacedGlyph {
    atlas_glyph_index: u32,
    in_atlas_bbox: Rect,
    pub placed_bbox: Rect,
}

impl GenericObject {
    pub fn size_in_u32s() -> u32 {
        u32::try_from(mem::size_of::<GenericObject>() / mem::size_of::<u32>())
            .expect("could not convert GenericObject size in u32s into u32 value")
    }

    pub fn size_in_bytes() -> usize {
        mem::size_of::<GenericObject>()
    }
}

pub struct Scene {
    pub objects: Vec<GenericObject>,
}

impl Scene {
    pub fn new_empty() -> Scene {
        Scene {
            objects: Vec::new()
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
        self.objects.push(GenericObject {
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
        });
    }

    pub fn append_glyph(
        &mut self,
        glyph_id: u16,
        in_atlas_bbox: Rect,
        in_scene_bbox: Rect,
        color: [u8; 4],
    ) {
        self.objects.push(GenericObject {
            object_type: ObjectType::Glyph as u16,
            glyph_id,
            in_atlas_bbox: (
                in_atlas_bbox.x0 as u16,
                in_atlas_bbox.x1 as u16,
                in_atlas_bbox.y0 as u16,
                in_atlas_bbox.y1 as u16,
            ),
            in_scene_bbox: (
                in_scene_bbox.x0 as u16,
                in_scene_bbox.x1 as u16,
                in_scene_bbox.y0 as u16,
                in_scene_bbox.y1 as u16,
            ),
            color,
        });
    }

    pub fn initialize_test_scene0(&mut self) {
        self.objects = Vec::new();

        let (scene_bbox_x_min, scene_bbox_y_min): (u16, u16) = (100, 100);

        let color: [u8; 4] = [255, 255, 255, 255];

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

    pub fn add_text(
        &mut self,
        screen_x_offset: u16,
        screen_y_offset: u16,
        placed_glyphs: &[PlacedGlyph],
        color: [u8; 4],
    ) {
        for pg in placed_glyphs.iter() {
            self.append_glyph(
                pg.atlas_glyph_index as u16,
                pg.in_atlas_bbox,
                Rect {
                    x0: pg.placed_bbox.x0 + (screen_x_offset as f64),
                    x1: pg.placed_bbox.x1 + (screen_x_offset as f64),
                    y0: pg.placed_bbox.y0 + (screen_y_offset as f64),
                    y1: pg.placed_bbox.y1 + (screen_y_offset as f64),
                },
                color,
            );
        }
    }
}
