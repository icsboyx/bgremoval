use anyhow::Result;
use fast_image_resize::PixelType;
use raylib::{ffi::MeasureText, prelude::*, texture::Image};
use std::{sync::mpsc::Receiver, time::Instant};
use tracing_subscriber::fmt::format;

use crate::SETUP;

#[derive(Clone, Debug)]
pub struct Frame {
    pub width: i32,
    pub height: i32,
    pub pixel_type: PixelType,
    pub data: Vec<u8>, // Only store the canonical buffer!
}

impl Frame {
    pub fn as_rgb(&self) -> Vec<u8> {
        match self.pixel_type {
            PixelType::U8x3 => self.data.clone(),
            PixelType::U8x4 => {
                let mut rgb = Vec::with_capacity(self.width as usize * self.height as usize * 3);
                for px in self.data.chunks_exact(4) {
                    rgb.extend_from_slice(&px[0..3]);
                }
                rgb
            }
            _ => panic!("Unsupported pixel type"),
        }
    }

    pub fn as_rgba(&self) -> Vec<u8> {
        match self.pixel_type {
            PixelType::U8x4 => self.data.clone(),
            PixelType::U8x3 => {
                let mut out = Vec::with_capacity(self.width as usize * self.height as usize * 4);
                for px in self.data.chunks_exact(3) {
                    out.extend_from_slice(px);
                    out.push(255);
                }
                out
            }
            _ => panic!("Unsupported pixel type"),
        }
    }

    pub fn to_hwc_f32(&self) -> ndarray::Array3<f32> {
        let rgb = self.as_rgb();
        let width = self.width as usize;
        let height = self.height as usize;
        assert_eq!(rgb.len(), width * height * 3);
        let floats: Vec<f32> = rgb.iter().map(|&v| v as f32 / 255.0).collect();
        ndarray::Array3::from_shape_vec((height, width, 3), floats).unwrap()
    }

    pub fn to_chw_f32(&self) -> ndarray::Array3<f32> {
        self.to_hwc_f32().permuted_axes([2, 0, 1])
    }

    pub fn to_nhwc_f32(&self) -> ndarray::Array4<f32> {
        self.to_hwc_f32().insert_axis(ndarray::Axis(0))
    }

    pub fn to_nchw_f32(&self) -> ndarray::Array4<f32> {
        self.to_chw_f32().insert_axis(ndarray::Axis(0))
    }
}
#[derive(Clone)]
pub struct RaylibFrames {
    pub high_res_frame: Frame,
    pub low_res_frame: Frame,
    pub ml_low_frame: Frame,
    pub ml_high_frame: Frame,
    pub instant: Instant,
}

pub fn start_raylib_viewer(rx: Receiver<RaylibFrames>) -> Result<()> {
    let scale_factor = 0.5 as f32;

    let (mut rl, thread) = raylib::init()
        .size(
            (SETUP.full_dec_width as f32 * scale_factor) as i32,
            ((SETUP.full_dec_height + SETUP.small_dec_height) as f32 * scale_factor) as i32,
        )
        .title("Camera Stream")
        .log_level(raylib::consts::TraceLogLevel::LOG_ALL)
        .build();
    rl.set_target_fps(60);

    let font = rl.load_font(&thread, "fonts/Roboto-Regular.ttf").unwrap();
    font.texture()
        .set_texture_filter(&thread, raylib::consts::TextureFilter::TEXTURE_FILTER_BILINEAR);

    let Ok(RaylibFrames {
        high_res_frame,
        low_res_frame,
        ml_low_frame: ml_frame,
        ml_high_frame: _ml_high_frame,
        instant,
    }) = rx.recv()
    else {
        return Err(anyhow::anyhow!("Failed to receive initial setup frame"));
    };

    println!(
        "Initial frame received in {} ms, starting viewer...",
        instant.elapsed().as_millis()
    );

    // Create high resolution image
    let mut high_res_texture = rl.load_texture_from_image(
        &thread,
        &Image::gen_image_color(high_res_frame.width, high_res_frame.height, Color::WHITE),
    )?;
    high_res_texture.set_texture_filter(&thread, raylib::consts::TextureFilter::TEXTURE_FILTER_BILINEAR);
    high_res_texture.update_texture(&blend(&high_res_frame.data, &_ml_high_frame.data))?;

    // Create high resolution image
    let mut low_res_texture = rl.load_texture_from_image(
        &thread,
        &Image::gen_image_color(low_res_frame.width, low_res_frame.height, Color::WHITE),
    )?;
    low_res_texture.set_texture_filter(&thread, raylib::consts::TextureFilter::TEXTURE_FILTER_BILINEAR);
    low_res_texture.update_texture(&low_res_frame.data)?;

    // Create ML processed image
    let mut ml_res_texture = rl.load_texture_from_image(
        &thread,
        &Image::gen_image_color(ml_frame.width, ml_frame.height, Color::WHITE),
    )?;
    ml_res_texture.set_texture_filter(&thread, raylib::consts::TextureFilter::TEXTURE_FILTER_BILINEAR);
    ml_res_texture.update_texture(&ml_frame.as_rgba())?;

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        d.draw_texture_ex(
            &high_res_texture,
            Vector2::new(0.0, 0.0),
            0.0,
            scale_factor,
            Color::WHITE,
        );
        d.draw_texture_ex(
            &low_res_texture,
            Vector2::new(0 as f32, high_res_frame.height as f32 * scale_factor),
            0.0,
            scale_factor,
            Color::WHITE,
        );
        d.draw_texture_ex(
            &ml_res_texture,
            Vector2::new(
                low_res_frame.width as f32 * scale_factor,
                high_res_frame.height as f32 * scale_factor,
            ),
            0.0,
            scale_factor,
            Color::WHITE,
        );
        d.draw_text_ex(
            &font,
            &format!(
                "High Res: {}x{} with upscaled mask: from {}x{}",
                high_res_frame.width, high_res_frame.height, ml_frame.width, ml_frame.height
            ),
            Vector2::new(10.0, 10.0),
            30.0,
            1.0,
            Color::BLUE,
        );
        match rx.recv() {
            Ok(RaylibFrames {
                high_res_frame,
                low_res_frame,
                ml_low_frame,
                ml_high_frame,
                instant,
            }) => {
                // Create high resolution image
                high_res_texture.update_texture(&blend(&high_res_frame.as_rgba(), &ml_high_frame.as_rgba()))?;
                d.draw_text_ex(
                    &font,
                    &format!(
                        "Total elaboration and render time: {} ms",
                        instant.elapsed().as_millis(),
                    ),
                    Vector2::new(10.0, 40.0),
                    30.0,
                    1.0,
                    Color::BLUE,
                );

                // SAFELY grab the default Font from the C API:

                let scaling_text = format!("Raylib UI scaling factor: {}", scale_factor);
                d.draw_text_ex(
                    &font,
                    &scaling_text,
                    Vector2::new(
                        (d.get_render_width() - raylib::core::RaylibHandle::measure_text(&d, &scaling_text, 30)) as f32,
                        (d.get_render_height() - 35) as f32,
                    ),
                    30.0,
                    1.0,
                    Color::BLUE,
                );

                // Create low resolution image
                low_res_texture.update_texture(&low_res_frame.as_rgba())?;
                // Create ML processed image
                ml_res_texture.update_texture(&ml_low_frame.as_rgba())?;
            }
            Err(_) => {}
        };
    }
    Ok(())
}

pub fn blend(image: &[u8], mask: &[u8]) -> Vec<u8> {
    assert_eq!(image.len(), mask.len());
    let mut blended = Vec::with_capacity(image.len());
    for (px, m) in image.chunks_exact(4).zip(mask.chunks_exact(4)) {
        if m[3] == 0 {
            // Transparent mask alpha = person pixel: keep original pixel
            blended.extend_from_slice(px);
        } else {
            // Opaque mask alpha = background pixel: use mask color (green here)
            blended.extend_from_slice(&[0, 0, 0, 0]);
        }
    }
    blended
}
