//! Captures a video frame and writes it to a file.
//!
//! Uses the [`linuxvideo::stream::ReadStream`] returned by [`linuxvideo::VideoCaptureDevice::into_stream`]
//! to read image data.

use std::{
    env,
    ffi::{OsStr, OsString},
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail};
use linuxvideo::{
    format::{PixFormat, PixelFormat},
    BufType, Device,
};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut args = env::args_os().skip(1);

    let device = args
        .next()
        .ok_or_else(|| anyhow!("usage: save-jpeg <device> <file> [<count>]"))?;

    let file_path = args
        .next()
        .ok_or_else(|| anyhow!("usage: save-jpeg <device> <file> [<count>]"))?;

    let count: u32 = args.next().map_or(1, |osstr| {
        osstr
            .to_str()
            .expect("invalid UTF-8")
            .parse()
            .expect("invalid value for <count>")
    });

    let device = Device::open(Path::new(&device))?;
    println!(
        "capabilities: {:?}",
        device.capabilities()?.device_capabilities()
    );

    let formats = device
        .formats(BufType::VIDEO_CAPTURE)
        .map(|res| res.map(|f| f.pixel_format()))
        .collect::<io::Result<Vec<_>>>()?;
    let format = if formats.contains(&PixelFormat::MJPG) {
        PixelFormat::MJPG
    } else if formats.contains(&PixelFormat::JPEG) {
        PixelFormat::JPEG
    } else {
        bail!(
            "webcam does not support JPEG (supported formats are {:?})",
            formats
        );
    };

    let capture = device.video_capture(PixFormat::new(u32::MAX, u32::MAX, format))?;
    println!("negotiated format: {:?}", capture.format());

    let mut stream = capture.into_stream()?;
    println!("stream started, waiting for data");
    for i in 0..count {
        let mut path = PathBuf::from(&file_path);
        let stem = path.file_stem().unwrap_or(OsStr::new("image"));
        let ext = path.extension().unwrap_or(OsStr::new("jpg"));
        let number = if count == 1 {
            String::new()
        } else {
            format!("-{i}")
        };
        let filename = [stem, OsStr::new(&number), OsStr::new("."), ext]
            .into_iter()
            .collect::<OsString>();
        path.set_file_name(filename);

        let mut file = File::create(&path)?;
        stream.dequeue(|buf| {
            if buf.is_error() {
                eprintln!("WARNING: error flag is set on buffer");
            }
            file.write_all(&*buf)?;
            println!(
                "wrote {} bytes to {} (raw buffer size: {} bytes)",
                buf.len(),
                path.display(),
                buf.raw_buffer().len(),
            );
            Ok(())
        })?;
    }

    Ok(())
}
