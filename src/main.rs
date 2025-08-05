pub mod bgremoval;
pub mod capture;
pub mod decoder;
pub mod viewer;

use crate::bgremoval::MlFrames;
use crate::capture::capture;
use crate::decoder::decode;
use crate::viewer::RaylibFrames;
use anyhow::Result;
use fast_image_resize::PixelType;
use std::any::Any;
use std::thread;
use v4l::Device;
use v4l::Format;
use v4l::FourCC;
use v4l::buffer::Type;
use v4l::context::enum_devices;
use v4l::prelude::MmapStream;
use v4l::video::Capture;

pub static SETUP: Setup = Setup {
    camera_device: 0,                      // Default to first camera
    capture_width: 1920,                   // Default width
    capture_res_height: 1080,              // Default height
    full_dec_width: 1920,                  // Width for high resolution
    full_dec_height: 1080,                 // Height for high resolution
    ful_dec_pixel_type: PixelType::U8x4,   // Pixel type for high
    small_dec_width: 512,                  // Width for low resolution
    small_dec_height: 512,                 // Height for low resolution
    small_dec_pixel_type: PixelType::U8x4, // Pixel type for low resolution
};

pub struct Setup {
    camera_device: usize,
    capture_width: u32,
    capture_res_height: u32,
    full_dec_width: u32,
    full_dec_height: u32,
    ful_dec_pixel_type: PixelType,
    small_dec_width: u32,
    small_dec_height: u32,
    small_dec_pixel_type: PixelType,
}

fn main() -> Result<()> {
    println!("Starting camera stream...");
    // List all present devices
    for dev in enum_devices() {
        println!("Found device: {:?}, {:?}, {:?}", dev.path(), dev.name(), dev.type_id());
    }

    let dev = match Device::new(SETUP.camera_device) {
        Ok(device) => device,
        Err(e) => {
            eprintln!("Failed to open device: {:#?}", e);
            return Err(e.into());
        }
    }; // /dev/video0
    let mut join_handles = Vec::new();

    println!("=== Supported Formats ===");
    for format in dev.enum_formats()? {
        println!("Pixel format: {}, description: {}", format.fourcc, format.description);

        for size in dev.enum_framesizes(format.fourcc)? {
            println!("  {:?}", size);
        }
    }

    let fmt = Format::new(SETUP.capture_width, SETUP.capture_res_height, FourCC::new(b"MJPG"));
    dev.set_format(&fmt)?;

    let stream = MmapStream::with_buffers(&dev, Type::VideoCapture, 4)?;
    println!("Selected format: {:?}", fmt);
    println!("Starting video capture...");

    let (tx, rx) = std::sync::mpsc::channel();
    let (ml_tx, ml_rx) = std::sync::mpsc::channel::<MlFrames>();
    let (raylib_tx, raylib_rx) = std::sync::mpsc::channel::<RaylibFrames>();

    println!("Starting capture...");

    // check_model()?;
    // exit(1);

    join_handles.push(
        thread::Builder::new()
            .name("capture".into())
            .spawn(move || -> Result<()> { capture(tx, stream) })?,
    );
    join_handles.push(
        thread::Builder::new()
            .name("decoder".into())
            .spawn(move || -> Result<()> { decode(rx, ml_tx) })?,
    );

    join_handles.push(
        thread::Builder::new()
            .name("bgremoval".into())
            .spawn(move || -> Result<()> { bgremoval::bgremoval(ml_rx, raylib_tx) })?,
    );

    join_handles.push(
        thread::Builder::new()
            .name("raylib_viewer".into())
            .spawn(move || -> Result<()> { viewer::start_raylib_viewer(raylib_rx) })?,
    );

    for handle in join_handles {
        let thread_name = handle.thread().name().unwrap_or("unknown").to_owned();
        match handle.join() {
            Ok(Ok(())) => {} // Thread OK
            Ok(Err(e)) => eprintln!("Thread {:?} returned error: {:?}", &thread_name, e),
            Err(e) => eprintln!("Thread panicked: {:?}", e),
        }
    }
    Ok(())
}
