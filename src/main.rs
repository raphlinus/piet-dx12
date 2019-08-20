//#[macro_use]
//extern crate log;
//extern crate env_logger;

// #![windows_subsystem = "windows"] (I think I want the console)

extern crate piet;
extern crate kurbo;

pub mod dx12;
pub mod error;
pub mod atlas;
pub mod gpu;
pub mod scene;
pub mod window;

use std::os::windows::ffi::OsStrExt;

use std::borrow::Cow;

use kurbo::{Affine, Point, Rect, Shape};

use piet::{
    Color, Error, FixedGradient, Font, FontBuilder, ImageFormat, InterpolationMode, IntoBrush,
    RenderContext, StrokeStyle, Text, TextLayout, TextLayoutBuilder,
};

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
pub struct DX12Font;
pub struct DX12FontBuilder;

pub struct DX12TextLayout;
pub struct DX12TextLayoutBuilder;

pub struct DX12Text;


// let mut gpu_state = gpu::GpuState::new(
//            wnd,
//            String::from("build_per_tile_command_list"),
//            String::from("paint_objects"),
//            String::from("VSMain"),
//            String::from("PSMain"),
//            1000,
//            16,
//            32,
//            1,
//            1,
//            1,
//            512,
//            512,
//            512*512,
//            1000,
//        );

pub struct DX12RenderContext {
    scene: scene::Scene,
    text: DX12Text,
}

impl DX12RenderContext {
    pub unsafe fn new(atlas_width: u16, atlas_height: u16) -> DX12RenderContext {
        DX12RenderContext {
            scene: scene::Scene::new_empty(atlas_width, atlas_height),
            text: DX12Text,
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
        Self::Brush::Solid(ColorValue{
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

    fn fill(&mut self, _shape: impl Shape, _brush: &impl IntoBrush<Self>) {

    }

    fn fill_even_odd(&mut self, _shape: impl Shape, _brush: &impl IntoBrush<Self>) {}

    fn clip(&mut self, _shape: impl Shape) {}

    fn text(&mut self) -> &mut Self::Text {
        &mut self.text
    }

    fn draw_text(
        &mut self,
        _layout: &Self::TextLayout,
        _pos: impl Into<Point>,
        _brush: &impl IntoBrush<Self>,
    ) {
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

    fn new_font_by_name(&mut self, _name: &str, _size: f64) -> Result<Self::FontBuilder, Error> {
        Ok(DX12FontBuilder)
    }

    fn new_text_layout(
        &mut self,
        _font: &Self::Font,
        _text: &str,
    ) -> Result<Self::TextLayoutBuilder, Error> {
        Ok(DX12TextLayoutBuilder)
    }
}

impl Font for DX12Font {}

impl FontBuilder for DX12FontBuilder {
    type Out = DX12Font;

    fn build(self) -> Result<Self::Out, Error> {
        Ok(DX12Font)
    }
}

impl TextLayoutBuilder for DX12TextLayoutBuilder {
    type Out = DX12TextLayout;

    fn build(self) -> Result<Self::Out, Error> {
        Ok(DX12TextLayout)
    }
}

impl TextLayout for DX12TextLayout {
    fn width(&self) -> f64 {
        42.0
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

fn main() {
    unsafe {
        println!("creating window...");
        let wnd = window::Window::new(win32_string("test"), win32_string("test"));

        let num_renders: u32 = 1000;
        let atlas_width: u16 = 512;
        let atlas_height: u16 = 512;

        let mut render_context = DX12RenderContext::new(atlas_width, atlas_height);
        //let custom_font = atlas::FontBytes::new();
        //let font_rs_obj = custom_font.generate_font_rs_object();
        //render_context.scene.add_characters_to_atlas("0123456789", 50, &font_rs_obj);
        //render_context.scene.atlas.dump_bytes_as_rgba_image();
        //render_context.scene.populate_randomly(wnd.get_width(), wnd.get_height(), num_objects);
        render_context.scene.initialize_test_scene0(wnd.get_width(), wnd.get_height());
        let tile_side_length_in_pixels = 16;

        let num_objects: u32 = render_context.scene.objects.len() as u32;
        println!("num_objects: {}", num_objects);
        println!("scene in bytes: {:?}", render_context.scene.to_bytes());

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
            (atlas_width as u64)*(atlas_height as u64),
            num_renders,
        );

        let constants = gpu::Constants {
            num_objects: num_objects,
            object_size: scene::GenericObject::size_in_bytes() as u32,
            tile_size: tile_side_length_in_pixels,
            num_tiles_x: gpu_state.num_tiles_x,
            num_tiles_y: gpu_state.num_tiles_y,
        };

        gpu_state.upload_data(Some(constants), None, None);

        //panic!("stop");

        for i in 0..num_renders {
            gpu_state.render(i, &render_context.scene.atlas.bytes);
        }

        gpu_state.print_stats();
        gpu_state.destroy();
    }
}
