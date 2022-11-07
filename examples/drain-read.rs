//! Captures video and ignores the video data (printing a `.` to the screen for every frame
//! received).
//!
//! Uses the [`std::io::Read`] implementation of [`linuxvideo::VideoCaptureDevice`] to capture image
//! data.

use std::{
    env,
    io::{Read, Write},
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

    let path = match args.next() {
        Some(path) => path,
        None => {
            println!("usage: drain-read <device>");
            std::process::exit(1);
        }
    };

    let device = Device::open(Path::new(&path))?;

    println!(
        "capabilities: {:?}",
        device.capabilities()?.device_capabilities()
    );

    let mut capture =
        device.video_capture(PixFormat::new(u32::MAX, u32::MAX, Pixelformat::YUYV))?;
    println!("negotiated format: {:?}", capture.format());
    let size = capture.format().size_image() as usize;
    let mut buf = vec![0; size];

    println!("stream started, waiting for data");
    let mut frames = 0;
    let mut time = Instant::now();
    loop {
        capture.read(&mut buf)?;

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
