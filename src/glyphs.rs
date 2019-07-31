extern crate image;
use self::image::GenericImageView;
use std::path::{Path, PathBuf};

pub struct RawGlyph {
    pub bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

pub fn load_raw_glyphs() -> Vec<RawGlyph> {
    let mut raw_glyphs = Vec::<RawGlyph>::new();

    for i in 0..10 {
        let digit_string = i.to_string();

        let mut filepath: PathBuf = ["/resources", &digit_string, ".png"].iter().collect();

        let digit_image = image::open(&filepath)
            .expect(&format!(
                "failed to load iamge at: {}",
                filepath
                    .to_str()
                    .expect("could not convert filepath to string for error message")
            ))
            .grayscale();

        let raw_glyph = RawGlyph {
            bytes: digit_image.raw_pixels(),
            width: {
                let w = digit_image.width();
                assert!(w < std::u16::MAX);
                w as u16
            },
            height: {
                let h = digit_image.height();
                assert!(h < std::u16::MAX);
                h as u16
            },
        };

        raw_glyphs.push(raw_glyph);
    }

    raw_glyphs
}
