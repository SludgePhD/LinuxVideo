//! Captures video and ignores the video data (printing a `.` to the screen for every frame
//! received).
//!
//! Uses the [`linuxvideo::stream::ReadStream`] returned by [`linuxvideo::VideoCaptureDevice::into_stream`]
//! to read image data.

use std::{
    env,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

use linuxvideo::{
    format::{PixFormat, Pixelformat},
    Device,
};

fn main() -> linuxvideo::Result<()> {
    env_logger::init();

    let mut args = env::args_os().skip(1);

    let path = args
        .next()
        .ok_or_else(|| format!("usage: drain-stream <device>"))?;

    let device = Device::open(Path::new(&path))?;

    println!(
        "capabilities: {:?}",
        device.capabilities()?.device_capabilities()
    );

    let capture = device.video_capture(PixFormat::new(u32::MAX, u32::MAX, Pixelformat::YUYV))?;
    println!("negotiated format: {:?}", capture.format());

    let mut stream = capture.into_stream(2)?;

    println!("stream started, waiting for data");
    let mut frames = 0;
    let mut time = Instant::now();
    loop {
        stream.dequeue(|_buf| Ok(()))?;

        frames += 1;
        print!(".");
        std::io::stdout().flush().ok();

        if time.elapsed() >= Duration::from_secs(1) {
            println!(" {} FPS", frames);

            time = Instant::now();
            frames = 0;
        }
    }
}
