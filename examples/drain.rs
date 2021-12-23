//! Captures video and ignores the video data (printing a `.` to the screen for every frame
//! received).

use std::{
    env,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

use livid::{format::PixFormat, Device, Pixelformat};

fn main() -> livid::Result<()> {
    env_logger::init();

    let mut args = env::args_os().skip(1);

    let path = args
        .next()
        .ok_or_else(|| format!("usage: drain <device>"))?;

    let device = Device::open(Path::new(&path))?;

    let meta = device.video_capture(PixFormat::new(u32::MAX, u32::MAX, Pixelformat::YUYV))?;
    println!("negotiated format: {:?}", meta.format());
    let mut stream = meta.into_stream(1)?;
    stream.stream_on()?;

    println!("stream started, waiting for data");
    let mut frames = 0;
    let mut time = Instant::now();
    loop {
        stream.dequeue(|_view| {
            frames += 1;
            print!(".");
            std::io::stdout().flush().ok();

            if time.elapsed() >= Duration::from_secs(1) {
                println!(" {} FPS", frames);

                time = Instant::now();
                frames = 0;
            }

            Ok(())
        })?;
    }
}
