use crate::SETUP;
use crate::viewer::{Frame, RaylibFrames};
use anyhow::Result;
use fast_image_resize::images::Image;
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer, SrcCropping};
use std::ops::Mul;
use std::time::Instant;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use ort::session::Session;
use ort::value::Tensor;
use ort::{execution_providers::*, inputs};
use std::sync::mpsc::{Receiver, Sender};

pub struct MlFrames {
    pub high_res_frame: Frame,
    pub low_res_frame: Frame,
    pub instant: Instant,
}

pub fn bgremoval(ml_rx: Receiver<MlFrames>, raylib_tx: Sender<RaylibFrames>) -> Result<()> {
    // Initialize tracing to receive debug messages from `ort`

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,ort=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // let ep = TensorRTExecutionProvider::default().with_device_id(0).build();
    let ep = CUDAExecutionProvider::default().with_device_id(0).build();

    ort::init()
        .with_execution_providers([ep])
        .with_name("BGRemoval")
        .commit()?;

    let mut session = Session::builder()?
        .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)?
        .commit_from_file("models/model.onnx")?;

    let mask_threshold = 235 as u8;
    let mask_per_frame = 0; // use 0 to process every frame
    let mut mask_per_frame_count = 0;
    let mut mask = vec![];
    // Loop
    while let Ok(MlFrames {
        high_res_frame,
        low_res_frame,
        instant,
    }) = ml_rx.recv()
    {
        if mask_per_frame == 0 || mask_per_frame_count == 0 || mask_per_frame_count % mask_per_frame == 0 {
            let tensor = Tensor::from_array(low_res_frame.to_nchw_f32())?;
            let outputs = session.run(inputs![tensor])?;
            let output = outputs["output"].try_extract_array::<f32>()?;
            let output = output.mul(255.0).map(|x| *x as u8);
            let output = output.into_raw_vec_and_offset();

            mask = output
                .0
                .iter()
                .flat_map(|&mask_val| {
                    if mask_val > mask_threshold {
                        vec![0, 0, 0, 0] // Transparent pixel (person)
                    } else {
                        vec![0, 255, 0, 255] // Green pixel, fully opaque (background)
                    }
                })
                .collect::<Vec<u8>>()
        }
        mask_per_frame_count += 1;

        let full_mask = resize_mask(
            SETUP.small_dec_width,
            SETUP.small_dec_height,
            mask.clone().as_mut_slice(),
            SETUP.full_dec_width,
            SETUP.full_dec_height,
        )?;

        let ml_high_frame = Frame {
            width: SETUP.full_dec_width as i32,
            height: SETUP.full_dec_height as i32,
            pixel_type: PixelType::U8x4,
            data: full_mask,
        };

        //Send

        let ml_low_frame = Frame {
            width: low_res_frame.width,
            height: low_res_frame.height,
            pixel_type: PixelType::U8x4,
            data: mask.clone(),
        };

        // Send all frames
        raylib_tx.send(RaylibFrames {
            high_res_frame,
            low_res_frame,
            ml_low_frame,
            ml_high_frame,
            instant,
        })?;
    }
    Ok(())
}

fn resize_mask(
    src_width: u32,
    src_height: u32,
    mut src_data: &mut [u8],
    dst_width: u32,
    dst_height: u32,
) -> Result<Vec<u8>, anyhow::Error> {
    let mut resizer = Resizer::new();

    // Create source image
    let src_img = Image::from_slice_u8(src_width, src_height, &mut src_data, PixelType::U8x4)?;

    // Create destination image
    let mut dst_img = Image::new(dst_width, dst_height, PixelType::U8x4);

    let options = ResizeOptions {
        algorithm: ResizeAlg::Convolution(FilterType::Box),
        cropping: SrcCropping::None,
        mul_div_alpha: false,
    };

    resizer.resize(&src_img, &mut dst_img, &options)?;

    Ok(dst_img.into_vec())
}

pub fn run_inference(ml_rx: Receiver<MlFrames>, raylib_tx: Sender<RaylibFrames>) -> Result<()> {
    bgremoval(ml_rx, raylib_tx)?;
    Ok(())
}
