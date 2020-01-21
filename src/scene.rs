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

pub enum PietItemType {
    Circle,
    Glyph,
}

pub struct InSceneBBox {
    x_lims: (u16, u16),
    y_lims: (u16, u16),
}

impl InSceneBBox {
    fn new(x_min: u16, x_max: u16, y_min: u16, y_max: u16) -> InSceneBBox {
        InSceneBBox {
            x_lims: (x_min, x_max),
            y_lims: (y_min, y_max),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        // reverse order of each 4 bytes, so write component 2 first, in LE, then component 1 in LE
        // scene_bbox_x_max
        bytes
            .write_u16::<LittleEndian>(self.x_lims.1)
            .expect("could not convert u16 to bytes");
        // scene_bbox_x_min
        bytes
            .write_u16::<LittleEndian>(self.x_lims.0)
            .expect("could not convert u16 to bytes");

        // reverse order of each 4 bytes, so write component 2 first, in LE, then component 1 in LE
        // scene_bbox_x_max
        bytes
            .write_u16::<LittleEndian>(self.y_lims.1)
            .expect("could not convert u16 to bytes");
        // scene_bbox_x_min
        bytes
            .write_u16::<LittleEndian>(self.y_lims.0)
            .expect("could not convert u16 to bytes");

        bytes
    }
}

pub struct PietItem {
    tag: u32,
    in_atlas_bbox: (u16, u16, u16, u16),
    color: [u8; 4],
}

pub struct PlacedGlyph {
    pub atlas_glyph_index: u32,
    pub in_atlas_bbox: Rect,
    pub placed_bbox: Rect,
}

impl PietItem {
    pub fn size_in_u32s() -> u32 {
        let size_of_item_in_bytes = mem::size_of::<PietItem>();
        let size_of_u32_in_bytes = mem::size_of::<u32>();

        // item should always have a size that is an integer number of u32s
        assert_eq!(size_of_item_in_bytes % size_of_u32_in_bytes, 0);

        u32::try_from(size_of_item_in_bytes / size_of_u32_in_bytes)
            .expect("could not safely convert size of item in u32s into a u32 value")
    }

    pub fn size_in_bytes() -> usize {
        mem::size_of::<PietItem>()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.write_u32::<LittleEndian>(self.tag);

        // reverse order of each 4 bytes, so write component 2 first, in LE, then component 1 in LE
        // atlas_bbox_x_max
        bytes
            .write_u16::<LittleEndian>(self.in_atlas_bbox.1)
            .expect("could not convert u16 to bytes");
        // atlas_bbox_x_min
        bytes
            .write_u16::<LittleEndian>(self.in_atlas_bbox.0)
            .expect("could not convert u16 to bytes");

        // reverse order of each 4 bytes, so write component 2 first, in LE, then component 1 in LE
        // atlas_bbox_x_max
        bytes
            .write_u16::<LittleEndian>(self.in_atlas_bbox.1)
            .expect("could not convert u16 to bytes");
        // atlas_bbox_x_min
        bytes
            .write_u16::<LittleEndian>(self.in_atlas_bbox.0)
            .expect("could not convert u16 to bytes");

        for component in self.color.iter().rev() {
            bytes.push(*component);
        }

        bytes
    }
}

pub struct Scene {
    pub item_bboxes: Vec<InSceneBBox>,
    pub items: Vec<PietItem>,
}

impl Scene {
    pub fn new_empty() -> Scene {
        Scene {
            item_bboxes: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn to_bytes(&self) -> (Vec<u8>, Vec<u8>) {
        let mut in_scene_bboxes_in_bytes = Vec::<u8>::new();
        let mut items_in_bytes = Vec::<u8>::new();

        for (i, item) in self.items.iter().enumerate() {
            items_in_bytes.append(&mut item.to_bytes());
            in_scene_bboxes_in_bytes.append(&mut self.item_bboxes[i].to_bytes());
        }

        (in_scene_bboxes_in_bytes, items_in_bytes)
    }

    pub fn append_circle(&mut self, circle: Circle, color: [u8; 4]) {
        self.items.push(PietItem {
            tag: PietItemType::Circle as u32,
            in_atlas_bbox: (0, 0, 0, 0),
            color,
        });

        self.item_bboxes.push(InSceneBBox::new(
            (circle.center.x - circle.radius) as u16,
            (circle.center.x + circle.radius) as u16,
            (circle.center.y - circle.radius) as u16,
            (circle.center.y + circle.radius) as u16,
        ))
    }

    pub fn append_glyph(
        &mut self,
        glyph_id: u16,
        in_atlas_bbox: Rect,
        in_scene_bbox: Rect,
        color: [u8; 4],
    ) {
        self.items.push(PietItem {
            tag: PietItemType::Glyph as u32,
            in_atlas_bbox: (
                in_atlas_bbox.x0 as u16,
                in_atlas_bbox.x1 as u16,
                in_atlas_bbox.y0 as u16,
                in_atlas_bbox.y1 as u16,
            ),
            color,
        });

        self.item_bboxes.push(InSceneBBox::new(
            in_scene_bbox.x0 as u16,
            in_scene_bbox.x1 as u16,
            in_scene_bbox.y0 as u16,
            in_scene_bbox.y1 as u16,
        ))
    }

    pub fn initialize_test_scene0(&mut self) {
        self.items = Vec::new();

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
