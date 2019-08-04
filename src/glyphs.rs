extern crate image;
use self::image::GenericImageView;
use crate::dx12;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct RawAtlas {
    pub bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

impl RawAtlas {
    pub fn to_subresource_data(raw_atlas: RawAtlas) -> dx12::SubresourceData {
        let RawAtlas {
            bytes,
            width,
            height,
        } = raw_atlas;

        assert_eq!(width % 256, 0);

        dx12::SubresourceData {
            data: bytes,
            row_size: width as isize,
            column_size: height as isize,
        }
    }
}

pub fn load_raw_atlas() -> RawAtlas {
    let atlas_fp: PathBuf = ["resources", "glyph-atlas.png"].iter().collect();
    let atlas_fp_test: PathBuf = ["resources", "glyph-atlas-test.png"].iter().collect();

    let atlas_image = image::open(&atlas_fp)
        .expect(&format!(
            "failed to load image at: {}",
            atlas_fp
                .to_str()
                .expect("could not convert filepath to string for error message")
        ));
    let atlas_raw_pixels = atlas_image.raw_pixels();
    let mut grayscale_raw_pixels = Vec::new();
    for i in 0..atlas_raw_pixels.len() {
        if i % 4 == 3 {
            grayscale_raw_pixels.push(atlas_raw_pixels[i]);
        }
    }

    let raw_atlas = RawAtlas {
        bytes: grayscale_raw_pixels,
        width: {
            let w = atlas_image.width();
            //TODO: the spirit of the assert is valid, but is it the best way to do this chcek?
            assert!(w < std::u16::MAX as u32);
            w as u16
        },
        height: {
            let h = atlas_image.height();
            assert!(h < std::u16::MAX as u32);
            h as u16
        },
    };

    raw_atlas
}
