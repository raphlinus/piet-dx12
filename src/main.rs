// Copyright © 2019 piet-dx12 developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate kurbo;
extern crate piet;
extern crate rand;
extern crate font_rs;

pub mod atlas;
pub mod dx12;
pub mod error;
pub mod gpu;
pub mod scene;
pub mod window;

use std::os::windows::ffi::OsStrExt;
use std::borrow::Cow;
use kurbo::{Affine, Point, Rect, Shape};
use rand::Rng;
use piet::{
    Color, Error, FixedGradient, Font, FontBuilder, ImageFormat, InterpolationMode, IntoBrush,
    RenderContext, StrokeStyle, Text, TextLayout, TextLayoutBuilder,
};
use font_rs::font::{parse, Font as RawFont};
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::collections::HashSet;
use std::hash::Hash;
use atlas::Atlas;
use std::convert::TryFrom;

#[derive(Clone)]
pub struct ColorValue {
    color_u32: u32,
    color_u8s: [u8; 4],
}

#[derive(Clone)]
pub enum DX12Brush {
    Solid(ColorValue),
    Gradient,
}

pub struct DX12Image;

//TODO: fix font-rs Font so that it's signature is Font<T: AsRef<[u8]>> instead of Font<'a>
// `font_bytes` + `generate_font_rs_object` form a band-aid solution.
#[derive(Clone)]
pub struct RawFontGenerator {
    font_bytes: Vec<u8>,
}

impl RawFontGenerator {
    pub fn load_notomono() -> RawFontGenerator {
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
            Ok(_) => RawFontGenerator { font_bytes: data },
        }
    }

    pub fn generate_raw_font<'a>(&'a self) -> RawFont<'a> {
        match parse(&self.font_bytes) {
            Ok(font) => font,
            Err(_) => panic!("failed to parse bytes as font!"),
        }
    }
}

pub struct DX12Font;

pub struct DX12FontBuilder;

pub struct DX12TextLayout {
    placed_glyphs: Vec<scene::PlacedGlyph>,
}

pub struct DX12TextLayoutBuilder {
    atlas: Arc<atlas::Atlas>,
    raw_font_generator: Arc<RawFontGenerator>,
    font_size: u32,
    text: String,
}

pub struct DX12Text;

pub struct DX12RenderContext {
    scene: scene::Scene,
    atlas: Arc<atlas::Atlas>,
    inner_text: DX12Text,
}

impl DX12RenderContext {
    pub unsafe fn new(atlas_width: u16, atlas_height: u16) -> DX12RenderContext {
        DX12RenderContext {
            scene: scene::Scene::new_empty(),
            atlas: Arc::new(Atlas::create_empty_atlas(atlas_width, atlas_height)),
            inner_text: DX12Text,
        }
    }
}

impl RenderContext for DX12RenderContext {
    type Brush = DX12Brush;
    type Image = DX12Image;
    type Text = DX12Text;
    type TextLayout = DX12TextLayout;

    fn status(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn solid_brush(&mut self, color: Color) -> Self::Brush {
        let color: u32 = color.as_rgba_u32();
        let (r, g, b, a): (u8, u8, u8, u8) = {
            (
                (color & (255 << 24) >> 24) as u8,
                (color & (255 << 16) >> 16) as u8,
                (color & (255 << 8) >> 8) as u8,
                (color & 255) as u8,
            )
        };
        Self::Brush::Solid(ColorValue {
            color_u32: color,
            color_u8s: [r, g, b, a],
        })
    }

    fn gradient(&mut self, _gradient: impl Into<FixedGradient>) -> Result<Self::Brush, Error> {
        Ok(Self::Brush::Gradient)
    }

    fn clear(&mut self, _color: Color) {}

    fn stroke(&mut self, _shape: impl Shape, _brush: &impl IntoBrush<Self>, _width: f64) {}

    fn stroke_styled(
        &mut self,
        _shape: impl Shape,
        _brush: &impl IntoBrush<Self>,
        _width: f64,
        _style: &StrokeStyle,
    ) {
    }

    fn fill(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>) {
        let dummy_closure = || Rect {
            x0: 0.0,
            x1: 0.0,
            y0: 0.0,
            y1: 0.0,
        };
        let brush = brush.make_brush(self, dummy_closure).into_owned();
        match shape.as_circle() {
            Some(circle) => match brush {
                DX12Brush::Solid(cv) => {
                    self.scene.append_circle(circle, cv.color_u8s);
                }
                _ => {}
            },
            None => {}
        }
    }

    fn fill_even_odd(&mut self, _shape: impl Shape, _brush: &impl IntoBrush<Self>) {}

    fn clip(&mut self, _shape: impl Shape) {}

    fn text(&mut self) -> &mut Self::Text {
        &mut self.inner_text
    }

    fn draw_text(
        &mut self,
        layout: &Self::TextLayout,
        pos: impl Into<Point>,
        brush: &impl IntoBrush<Self>,
    ) {
        let pos = Point::from(pos);
        match brush {
            DX12Brush::Solid(cv) => {
                self.scene.add_text(pos.x as u16, pos.y as u16, &layout.placed_glyphs, cv.color_u8s);
            }
            _ => {}
        }
    }

    fn save(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn restore(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn finish(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn transform(&mut self, _transform: Affine) {}

    fn make_image(
        &mut self,
        _width: usize,
        _height: usize,
        _buf: &[u8],
        _format: ImageFormat,
    ) -> Result<Self::Image, Error> {
        Ok(DX12Image)
    }

    fn draw_image(
        &mut self,
        _image: &Self::Image,
        _rect: impl Into<Rect>,
        _interp: InterpolationMode,
    ) {
    }
}

impl Text for DX12Text {
    type Font = DX12Font;
    type FontBuilder = DX12FontBuilder;
    type TextLayout = DX12TextLayout;
    type TextLayoutBuilder = DX12TextLayoutBuilder;

    fn new_font_by_name(&mut self, _name: &str, size: f64) -> Result<Self::FontBuilder, Error> {
        unimplemented!();
    }

    fn new_text_layout(
        &mut self,
        font: &Self::Font,
        text: &str,
    ) -> Result<Self::TextLayoutBuilder, Error> {
        unimplemented!();
    }
}

impl Font for DX12Font {}

impl FontBuilder for DX12FontBuilder {
    type Out = DX12Font;

    fn build(self) -> Result<Self::Out, Error> {
        unimplemented!();
    }
}

// https://stackoverflow.com/a/47648303/3486684
fn dedup<T: Eq + Hash + Copy>(v: &mut Vec<T>) {
    // note the Copy constraint
    let mut uniques = HashSet::new();
    v.retain(|e| uniques.insert(*e));
}

impl TextLayoutBuilder for DX12TextLayoutBuilder {
    type Out = DX12TextLayout;

    fn build(self) -> Result<Self::Out, Error> {
        let mut placed_glyphs = Vec::new();

        let string_chars: Vec<char> = self.text.chars().collect();

        let glyph_chars: Vec<char> = {
            let mut result = string_chars.clone();
            dedup(&mut result);
            result
        };

        let mut atlas = self.atlas;
        {
            let mut_atlas_ref = Arc::get_mut(&mut atlas).expect("atlas is poisoned");
            for &gc in glyph_chars.iter() {
                let raw_font = self.raw_font_generator.generate_raw_font();
                mut_atlas_ref.insert_character(gc, self.font_size, &raw_font);
            }
        }

        let mut x_cursor: u16 = 0;
        let mut y_cursor: i32 = 0;

        for &c in string_chars.iter() {
            let gix = atlas.get_glyph_index_of_char(c, self.font_size);
            let glyph_advance = u16::try_from(self.atlas.glyph_advances[gix])
                .expect("could not safely convert u32 glyph advance into u16");
            let y_offset = u16::try_from(y_cursor + self.atlas.glyph_top_offsets[gix])
                .expect("could not safely convert i32 glyph y offset into u16");

            match atlas.glyph_bboxes[gix] {
                Some(in_atlas_bbox) => {
                    let (w, h) = {
                        (
                            in_atlas_bbox.1 - in_atlas_bbox.0,
                            in_atlas_bbox.3 - in_atlas_bbox.2,
                        )
                    };

                    let placed_bbox = Rect {
                        x0: x_cursor as f64,
                        x1: (x_cursor + w) as f64,
                        y0: y_offset as f64,
                        y1: (y_offset + h) as f64,
                    };

                    let in_atlas_bbox = Rect {
                        x0: in_atlas_bbox.0 as f64,
                        x1: in_atlas_bbox.1 as f64,
                        y0: in_atlas_bbox.2 as f64,
                        y1: in_atlas_bbox.3 as f64,
                    };

                    placed_glyphs.push(scene::PlacedGlyph {
                        atlas_glyph_index: gix as u32,
                        in_atlas_bbox,
                        placed_bbox,
                    });

                    x_cursor += w + (0.2 * glyph_advance as f32).round() as u16;
                }
                None => {
                    x_cursor += glyph_advance;
                }
            }
        }

        Ok(DX12TextLayout {
            placed_glyphs,
        })
    }
}

impl TextLayout for DX12TextLayout {
    fn width(&self) -> f64 {
        let x0 = self.placed_glyphs.iter().map(|pg| pg.placed_bbox.x0).min_by(|a, b| a.partial_cmp(b).expect("could not compare some x0 values")).unwrap_or(0.0);
        let x1 = self.placed_glyphs.iter().map(|pg| pg.placed_bbox.x0).min_by(|a, b| a.partial_cmp(b).expect("could not compare some x0 values")).unwrap_or(0.0);

        (x1 - x0)
    }
}

impl IntoBrush<DX12RenderContext> for DX12Brush {
    fn make_brush<'b>(
        &'b self,
        _piet: &mut DX12RenderContext,
        _bbox: impl FnOnce() -> Rect,
    ) -> std::borrow::Cow<'b, DX12Brush> {
        Cow::Borrowed(self)
    }
}

pub fn win32_string(value: &str) -> Vec<u16> {
    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn generate_random_circles(
    num_circles: u32,
    screen_size: Rect,
) -> Vec<(kurbo::Circle, DX12Brush)> {
    let mut rng = rand::thread_rng();

    let mut random_circles = Vec::<(kurbo::Circle, DX12Brush)>::new();

    for _ in 0..num_circles {
        let circle = kurbo::Circle {
            center: kurbo::Point {
                x: rng.gen_range(screen_size.x0, screen_size.x1),
                y: rng.gen_range(screen_size.y0, screen_size.y1),
            },
            radius: rng.gen_range(0.0, 100.0),
        };

        let mut color_u8s: [u8; 4] = [0; 4];
        for i in 0..4 {
            color_u8s[i] = rng.gen_range(0_u16, 256_u16) as u8;
        }
        let brush = DX12Brush::Solid(ColorValue {
            color_u32: 0,
            color_u8s,
        });

        random_circles.push((circle, brush));
    }

    random_circles
}

fn generate_random_text(
    num_strings: u32,
    screen_size: Rect,
) -> Vec<(String, kurbo::Point, u32, DX12Brush)> {
    let mut rng = rand::thread_rng();
    let possible_strings = ["wow", "much fast", "so gpu", "endgame now", "60 fps", "very piet"].iter().map(|&s| String::from(s)).collect::<Vec<String>>();

    let mut random_text = Vec::<(String, kurbo::Point, u32, DX12Brush)>::new();

    for i in 0..num_strings {
        let chosen_string = {
            let rand_ix = rng.gen_range(0, possible_strings.len());
            possible_strings[rand_ix]
        };

        let font_size = rng.gen_range(12_u32, 50_u32);

        let pos = kurbo::Point {
            x: rng.gen_range(0 as f64, screen_size.x1 as f64),
            y: rng.gen_range(0 as f64, screen_size.y1 as f64),
        };

        let mut color_u8s: [u8; 4] = [0; 4];
        for i in 0..4 {
            color_u8s[i] = rng.gen_range(0_u16, 256_u16) as u8;
        }
        let brush = DX12Brush::Solid(ColorValue {
            color_u32: 0,
            color_u8s,
        });

        random_text.push((chosen_string, pos, font_size, brush));
    }

    random_text
}

fn populate_render_context(render_context: &mut DX12RenderContext, raw_text_generator: &Arc<RawFontGenerator>, random_circles: &[(kurbo::Circle, DX12Brush)], random_text: &[(String, kurbo::Point, u32, DX12Brush)]) {
    random_circles.iter().map(|(circle, brush)| render_context.fill(circle, brush));

    random_text.iter().map(|(text, position, font_size, brush)| {
        let layout_builder = DX12TextLayoutBuilder {
            atlas: Arc::clone(&render_context.atlas),
            text: "".to_string(),
            raw_font_generator: Arc::clone(raw_text_generator),
            font_size: *font_size,
        };
        let text_layout = layout_builder.build().expect("could not layout text");
        render_context.draw_text(&text_layout, position, brush);
    });
}

fn main() {
    unsafe {
        println!("creating window...");
        let wnd = window::Window::new(win32_string("test"), win32_string("test"));
        let screen_size = Rect {
            x0: 0.0,
            x1: wnd.get_width() as f64,
            y0: 0.0,
            y1: wnd.get_height() as f64,
        };

        let num_objects: u32 = 1000;
        let num_renders: u32 = 1000;
        let atlas_width: u16 = 512;
        let atlas_height: u16 = 512;
        let tile_side_length_in_pixels: u32 = 16;

        let mut gpu_state = gpu::GpuState::new(
            &wnd,
            String::from("build_per_tile_command_list"),
            String::from("paint_objects"),
            String::from("VSMain"),
            String::from("PSMain"),
            num_objects,
            tile_side_length_in_pixels,
            32,
            1,
            1,
            1,
            atlas_width as u64,
            atlas_height as u32,
            (atlas_width as u64) * (atlas_height as u64),
            num_renders,
        );

        let constants = gpu::Constants {
            num_objects,
            object_size: scene::GenericObject::size_in_bytes() as u32,
            tile_size: tile_side_length_in_pixels,
            num_tiles_x: gpu_state.num_tiles_x,
            num_tiles_y: gpu_state.num_tiles_y,
        };

        let random_circles = generate_random_circles(500, screen_size);
        let random_text = generate_random_text(500, screen_size);
        let raw_font_generator = Arc::new(RawFontGenerator::load_notomono());

        for i in 0..num_renders {
            let mut render_context = DX12RenderContext::new(atlas_width, atlas_height);
            // let custom_font = atlas::FontBytes::new();
            // let font_rs_obj = custom_font.generate_font_rs_object();
            // render_context.scene.add_characters_to_atlas("0123456789", 50, &font_rs_obj);
            //render_context.scene.atlas.dump_bytes_as_rgba_image();
            // render_context.scene.populate_randomly(wnd.get_width(), wnd.get_height(), 1000);
            //render_context.scene.initialize_test_scene0();
            populate_render_context(&mut render_context, &raw_font_generator, &random_circles, &random_text);
            let tile_side_length_in_pixels = 16;

            let num_objects: u32 = render_context.scene.objects.len() as u32;

            gpu_state.upload_data(
                Some(constants),
                Some(render_context.scene.to_bytes()),
                Some(&render_context.atlas.bytes),
            );

            gpu_state.render(i);
        }

        gpu_state.print_stats();
        gpu_state.destroy();
    }
}
