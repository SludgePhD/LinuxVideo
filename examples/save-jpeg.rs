//! Captures a video frame and writes it to a file.
//!
//! Uses the [`linuxvideo::stream::ReadStream`] returned by [`linuxvideo::VideoCaptureDevice::into_stream`]
//! to read image data.

use std::{env, fs::File, io::Write, path::Path};

use anyhow::anyhow;
use linuxvideo::{
    format::{PixFormat, PixelFormat},
    Device,
};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut args = env::args_os().skip(1);

    let device = args
        .next()
        .ok_or_else(|| anyhow!("usage: save-stream <device> <file>"))?;

    let file_path = args
        .next()
        .ok_or_else(|| anyhow!("usage: save-stream <device> <file>"))?;
    let mut file = File::create(file_path)?;

    let device = Device::open(Path::new(&device))?;

    println!(
        "capabilities: {:?}",
        device.capabilities()?.device_capabilities()
    );

    let capture = device.video_capture(PixFormat::new(u32::MAX, u32::MAX, PixelFormat::JPEG))?;
    println!("negotiated format: {:?}", capture.format());

    let mut stream = capture.into_stream()?;

    println!("stream started, waiting for data");
    stream.dequeue(|buf| {
        file.write_all(&*buf)?;
        println!("wrote file");
        Ok(())
    })?;

    Ok(())
}
