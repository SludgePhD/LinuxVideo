//! Writes test data to a video output device.
//!
//! Uses the [`linuxvideo::stream::WriteStream`] returned by [`linuxvideo::VideoOutputDevice::into_stream`].

// TODO: does not seem to work with v4l2loopback

use std::{
    env,
    io::Write,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail};
use itertools::Itertools;
use linuxvideo::{
    format::{PixFormat, PixelFormat},
    CapabilityFlags, Device,
};

const WIDTH: u32 = 120;
const HEIGHT: u32 = 60;
const PIXFMT: PixelFormat = PixelFormat::RGB32;

const RED: [u8; 4] = [0xff, 0xff, 0, 0];
const GREEN: [u8; 4] = [0xff, 0, 0xff, 0];
const BLUE: [u8; 4] = [0xff, 0, 0, 0xff];
const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

fn main() -> anyhow::Result<()> {
    let mut args = env::args_os().skip(1);

    let path = args
        .next()
        .ok_or_else(|| anyhow!("usage: write <device>"))?;

    let device = Device::open(Path::new(&path))?;
    if !device
        .capabilities()?
        .device_capabilities()
        .contains(CapabilityFlags::VIDEO_OUTPUT)
    {
        bail!("cannot write data: selected device does not support `VIDEO_OUTPUT` capability");
    }

    let output = device.video_output(PixFormat::new(WIDTH, HEIGHT, PIXFMT))?;
    let fmt = output.format();
    println!("set format: {:?}", fmt);

    if fmt.pixel_format() != PIXFMT {
        bail!("driver does not support the requested parameters");
    }

    let mut image = (0..fmt.height())
        .cartesian_product(0..fmt.width())
        .flat_map(|(y, x)| {
            if x == y {
                RED
            } else if x < 5 || x > fmt.width() - 5 {
                GREEN
            } else if y < 5 || y > fmt.height() - 5 {
                BLUE
            } else {
                TRANSPARENT
            }
        })
        .collect::<Vec<_>>();

    assert_eq!(
        image.len(),
        fmt.width() as usize * fmt.height() as usize * 4
    );
    assert!(image.len() <= fmt.size_image() as usize);
    image.resize(fmt.size_image() as usize, 0xff);

    let mut stream = output.into_stream()?;

    println!("output started");
    let mut frames = 0;
    let mut time = Instant::now();
    loop {
        stream.enqueue(|mut buf| {
            buf.copy_from_slice(&image);
            Ok(())
        })?;

        frames += 1;
        print!(".");
        std::io::stdout().flush().ok();

        if time.elapsed() >= Duration::from_secs(1) {
            println!(" {} FPS", frames);

            time = Instant::now();
            frames = 0;
        }

        // This can run at 170000 FPS, so sleep a bit to limit that.
        thread::sleep(Duration::from_millis(5));
    }
}
