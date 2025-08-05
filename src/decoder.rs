use std::sync::mpsc::{Receiver, Sender};

use anyhow::Result;
use fast_image_resize::{self as fr, FilterType, ResizeAlg, ResizeOptions, SrcCropping};
use fr::{PixelType, Resizer};
use turbojpeg::Image;
use turbojpeg::{Decompressor, PixelFormat};

use crate::SETUP;
use crate::bgremoval::MlFrames;
use crate::viewer::Frame;

pub fn decode(rx: Receiver<Vec<u8>>, ml_tx: Sender<MlFrames>) -> Result<()> {
    let mut decompressor = Decompressor::new()?;
    let mut resizer = Resizer::new();

    let mut full_dec_buffer = vec![];
    full_dec_buffer.resize(
        SETUP.full_dec_width as usize * SETUP.full_dec_height as usize * SETUP.ful_dec_pixel_type.size(),
        0 as u8,
    );

    while let Ok(data) = rx.recv() {
        assert_eq!(
            SETUP.full_dec_width as usize * SETUP.ful_dec_pixel_type.size() % 4,
            0,
            "Pitch must be 4-byte aligned"
        );

        decompressor.decompress(
            &data,
            Image {
                pixels: &mut full_dec_buffer[..], // full_img_size
                width: SETUP.full_dec_width as usize,
                height: SETUP.full_dec_height as usize,
                format: PixelFormat::try_from(pixel_type_to_pixel_format(SETUP.ful_dec_pixel_type)).unwrap(),
                pitch: SETUP.full_dec_width as usize * SETUP.ful_dec_pixel_type.size(),
            }, // turbo image needed here
        )?;

        let full_img = fr::images::Image::from_slice_u8(
            SETUP.full_dec_width,
            SETUP.full_dec_height,
            &mut full_dec_buffer[..],
            SETUP.ful_dec_pixel_type,
        )?;

        let mut small_img = fr::images::Image::new(
            SETUP.small_dec_width,
            SETUP.small_dec_height,
            SETUP.small_dec_pixel_type,
        );

        let options = ResizeOptions {
            algorithm: ResizeAlg::Convolution(FilterType::Lanczos3),
            cropping: SrcCropping::None,
            mul_div_alpha: false,
        };

        resizer.resize(&full_img, &mut small_img, &options)?;

        let high_res = Frame {
            width: full_img.width() as i32,
            height: full_img.height() as i32,
            pixel_type: full_img.pixel_type(),
            data: full_img.into_vec(),
        };

        let low_res = Frame {
            width: small_img.width() as i32,
            height: small_img.height() as i32,
            pixel_type: small_img.pixel_type(),
            data: small_img.into_vec(),
        };

        ml_tx.send(MlFrames {
            high_res_frame: high_res.clone(),
            low_res_frame: low_res.clone(),
        })?;
    }

    Ok(())
}

pub fn pixel_type_to_pixel_format(pix_fmt: PixelType) -> PixelFormat {
    match pix_fmt {
        PixelType::U8x4 => PixelFormat::RGBA,
        PixelType::U8x3 => PixelFormat::RGB,
        _ => panic!("Unsupported pixel type: {:?}", pix_fmt),
    }
}
