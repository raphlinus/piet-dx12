extern crate image;
use self::image::GenericImageView;
use std::path::{Path, PathBuf};
use crate::dx12;

#[derive(Clone)]
pub struct RawAtlas {
    pub bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

impl RawAtlas {
    pub fn to_subresource_data(raw_atlas: RawAtlas) -> dx12::SubresourceData {
        let RawAtlas { bytes, width, height} = raw_atlas;

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

    let atlas_image = image::open(&atlas_fp)
        .expect(&format!(
            "failed to load image at: {}",
            atlas_fp
                .to_str()
                .expect("could not convert filepath to string for error message")
        ))
        .grayscale();

    let raw_atlas = RawAtlas {
        bytes: atlas_image.raw_pixels(),
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
