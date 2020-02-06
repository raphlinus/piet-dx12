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

use kurbo::{Circle, Rect};

use byteorder::{LittleEndian, WriteBytesExt};
use std::convert::TryFrom;
use std::mem;

pub enum PietItem {
    Circle(PietCircle),
    Glyph(PietGlyph),
}

#[derive(Clone)]
pub struct BBox {
    x0: u16,
    x1: u16,
    y0: u16,
    y1: u16,
}

impl BBox {
    fn from(bbox: &Rect) -> BBox {
        // TODO: should more attention be paid to f64 to u16 conversion?
        BBox {
            x0: bbox.x0 as u16,
            x1: bbox.x1 as u16,
            y0: bbox.y0 as u16,
            y1: bbox.y1 as u16,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes
            .write_u16::<LittleEndian>(self.x0)
            .expect("could not convert u16 to bytes");
        bytes
            .write_u16::<LittleEndian>(self.x1)
            .expect("could not convert u16 to bytes");

        bytes
            .write_u16::<LittleEndian>(self.y0)
            .expect("could not convert u16 to bytes");
        bytes
            .write_u16::<LittleEndian>(self.y1)
            .expect("could not convert u16 to bytes");

        bytes
    }
}

pub struct SRGBColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl SRGBColor {
    fn from(color: &[u8; 4]) -> SRGBColor {
        SRGBColor {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        [self.a, self.b, self.g, self.r].iter().map(|&b| b).collect()
    }
}

pub struct PietGlyph {
    scene_bbox: BBox,
    atlas_bbox: BBox,
    color: SRGBColor,
}

impl PietGlyph {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.append(&mut self.scene_bbox.to_bytes());
        bytes.append(&mut self.atlas_bbox.to_bytes());
        bytes.append(&mut self.color.to_bytes());
        bytes.append(&mut Self::padding());

        bytes
    }

    pub fn size_in_bytes() -> usize {
        mem::size_of::<Self>()
    }

    pub fn padding() -> Vec<u8> {
        // println!("Glyph. PietItem_size: {}, Self_size: {}", PietItem::size_in_bytes(), Self::size_in_bytes());
        let padding_size = PietItem::size_in_bytes() - Self::size_in_bytes() - 4;
        vec![0; padding_size]
    }
}

pub struct PietCircle {
    scene_bbox: BBox,
    color: SRGBColor,
}

impl PietCircle {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.append(&mut self.scene_bbox.to_bytes());
        bytes.append(&mut self.color.to_bytes());
        bytes.append(&mut Self::padding());

        bytes
    }

    pub fn size_in_bytes() -> usize {
        mem::size_of::<Self>()
    }

    pub fn padding() -> Vec<u8> {
        // println!("Circle. PietItem_size: {}, Self_size: {}", PietItem::size_in_bytes(), Self::size_in_bytes());
        let padding_size = PietItem::size_in_bytes() - Self::size_in_bytes() - 4;
        vec![0; padding_size]
    }
}

impl PietItem {
    pub fn size_in_u32s() -> u32 {
        let size_in_bytes = Self::size_in_bytes();
        let size_of_u32_in_bytes = mem::size_of::<u32>();

        // item should always have a size that is an integer number of u32s
        assert_eq!(size_in_bytes % size_of_u32_in_bytes, 0);

        u32::try_from(size_in_bytes / size_of_u32_in_bytes)
            .expect("could not safely convert size of item in u32s into a u32 value")
    }

    pub fn size_in_bytes() -> usize {
        *([PietCircle::size_in_bytes(), PietGlyph::size_in_bytes()].iter().max().expect("could not determine size of PietItem")) + 4
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        match self {
            PietItem::Circle(circle) => {
                bytes
                    .write_u32::<LittleEndian>(0)
                    .expect("could not convert u32 to bytes");
                bytes.append(&mut circle.to_bytes());
            }
            PietItem::Glyph(glyph) => {
                bytes
                    .write_u32::<LittleEndian>(1)
                    .expect("could not convert u32 to bytes");
                bytes.append(&mut glyph.to_bytes());
            }
        }
        bytes
    }
}


pub struct PlacedGlyph {
    pub atlas_bbox: Rect,
    pub placed_bbox: Rect,
}

pub struct Scene {
    pub item_bboxes: Vec<BBox>,
    pub items: Vec<PietItem>,
}

impl Scene {
    pub fn new_empty() -> Scene {
        Scene {
            item_bboxes: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn to_bytes(&mut self) -> (Vec<u8>, Vec<u8>) {
        let mut item_bboxes_in_bytes = Vec::<u8>::new();
        let mut items_in_bytes = Vec::<u8>::new();

        for (bbox, item) in self.item_bboxes.iter().zip(self.items.iter()) {
            item_bboxes_in_bytes.append(&mut bbox.to_bytes());
            //println!("{:?}", &mut bbox.to_bytes());
            items_in_bytes.append(&mut item.to_bytes());
            //println!("{:?}", &mut item.to_bytes());
        }

        (item_bboxes_in_bytes, items_in_bytes)
    }

    pub fn append_circle(&mut self, circle: Circle, color: [u8; 4]) {
        let bbox = BBox {
            x0: (circle.center.x - circle.radius) as u16,
            x1: (circle.center.x + circle.radius) as u16,
            y0: (circle.center.y - circle.radius) as u16,
            y1: (circle.center.y + circle.radius) as u16,
        };

        self.items.push(PietItem::Circle(PietCircle {
            scene_bbox: bbox.clone(),
            color: SRGBColor::from(&color),
        }));

        self.item_bboxes.push(bbox.clone())
    }

    pub fn append_glyph(
        &mut self,
        scene_bbox: Rect,
        atlas_bbox: Rect,
        color: [u8; 4],
    ) {
        self.items.push(PietItem::Glyph(PietGlyph {
            scene_bbox: BBox::from(&scene_bbox),
            atlas_bbox: BBox::from(&atlas_bbox),
            color: SRGBColor::from(&color),
        }));

        self.item_bboxes.push(BBox::from(&scene_bbox))
    }

    pub fn add_text(
        &mut self,
        screen_x_offset: u16,
        screen_y_offset: u16,
        placed_glyphs: &[PlacedGlyph],
        color: [u8; 4],
    ) {
        for pg in placed_glyphs.iter() {
            let scene_bbox = Rect {
                x0: pg.placed_bbox.x0 + (screen_x_offset as f64),
                x1: pg.placed_bbox.x1 + (screen_x_offset as f64),
                y0: pg.placed_bbox.y0 + (screen_y_offset as f64),
                y1: pg.placed_bbox.y1 + (screen_y_offset as f64),
            };
            // println!("scene | x0: {}, x1: {}, y0: {}, y1: {}", scene_bbox.x0, scene_bbox.x1, scene_bbox.y0, scene_bbox.y1);
            // println!("atlas | x0: {}, x1: {}, y0: {}, y1: {}", pg.atlas_bbox.x0, pg.atlas_bbox.x1, pg.atlas_bbox.y0, pg.atlas_bbox.y1);
            self.append_glyph(
                scene_bbox,
                pg.atlas_bbox,
                color,
            );
        }
    }
}
