use anyhow::Result;
use std::sync::mpsc::Receiver;
use v4l::io::traits::OutputStream;
use v4l::video::Output;

use v4l::buffer::Type;
use v4l::{FourCC, prelude::*};

use crate::SETUP;
use crate::viewer::Frame;

// Create a virtual camera device
//sudo v4l2loopback-ctl add -n "BGR Virtual Cam"

pub fn virtual_cam(vcam_rx: Receiver<Vec<u8>>) -> Result<()> {
    println!("Creating virtual camera...");
    let virtual_vam_path = "/dev/video3";
    let node = v4l::context::Node::new(virtual_vam_path);
    println!(
        "Virtual camera : {} - {}",
        node.name().unwrap(),
        node.path().to_str().unwrap()
    );

    let mut device = Device::with_path(virtual_vam_path).unwrap();
    let mut fmt = device.format()?;
    fmt.fourcc = FourCC::new(b"BGR4");
    fmt.width = 1920;
    fmt.height = 1080;
    device.set_format(&fmt)?;
    let mut out_stream = MmapStream::with_buffers(&mut device, Type::VideoOutput, 4)?;

    while let Ok(frame) = vcam_rx.recv() {
        let (buf, buf_out_meta) = OutputStream::next(&mut out_stream)?;
        let output_frame = Frame {
            width: 1920,
            height: 1080,
            pixel_type: SETUP.ful_dec_pixel_type,
            data: frame,
        };

        // let mut output_buffer = OutputBuf::try_from(buf)?;
        buf.copy_from_slice(&output_frame.as_bgra());
        buf_out_meta.bytesused = buf.len() as u32;

        println!("Sending frame to virtual camera");
    }
    Ok(())
}
