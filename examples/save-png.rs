//! Captures BGRA video frames and encodes them as a PNG file (animated if more than one frame is
//! captured).
//!
//! Uses the [`linuxvideo::stream::ReadStream`] returned by [`linuxvideo::VideoCaptureDevice::into_stream`]
//! to read image data.

use std::{
    env,
    fs::File,
    io::{self, stdout, BufWriter, Write},
    path::Path,
    time::Instant,
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
        .ok_or_else(|| anyhow!("usage: save-png <device> <file> [<count>]"))?;

    let file_path = args
        .next()
        .ok_or_else(|| anyhow!("usage: save-png <device> <file> [<count>]"))?;

    let count: u32 = args.next().map_or(1, |osstr| {
        osstr
            .to_str()
            .expect("invalid UTF-8")
            .parse()
            .expect("invalid value for <count>")
    });
    if count == 0 {
        bail!("'capture zero frames', statements dreamed up by the utterly deranged");
    }

    let device = Device::open(Path::new(&device))?;
    println!(
        "capabilities: {:?}",
        device.capabilities()?.device_capabilities()
    );

    let formats = device
        .formats(BufType::VIDEO_CAPTURE)
        .map(|res| res.map(|f| f.pixel_format()))
        .collect::<io::Result<Vec<_>>>()?;
    let format = if formats.contains(&PixelFormat::ABGR32) {
        PixelFormat::ABGR32
    } else {
        bail!(
            "save-png does not support any of the device's formats (device supports {:?})",
            formats
        );
    };

    let capture = device.video_capture(PixFormat::new(u32::MAX, u32::MAX, format))?;
    println!("negotiated format: {:?}", capture.format());

    let width = capture.format().width();
    let height = capture.format().height();
    let mut stream = capture.into_stream()?;
    println!("stream started, waiting for data");

    let file = File::create(&file_path)?;
    let mut enc = png::Encoder::new(BufWriter::new(file), width, height);
    enc.set_animated(count, 0)?;
    enc.set_color(png::ColorType::Rgba);
    enc.validate_sequence(true);
    // We're not using the stream writer since the basic `Writer` is already streaming on a per-frame
    // basis, which is enough for this.
    let mut writer = enc.write_header()?;
    let mut cur_frame = vec![0; (width * height * 4) as usize];
    let mut prev_frame: Option<(Vec<u8>, _)> = None;
    for _ in 0..count + 1 {
        let arrival = stream.dequeue(|buf| {
            let arrival = Instant::now(); // FIXME: use buffer timecode instead
            if buf.is_error() {
                eprintln!("WARNING: error flag is set on buffer");
            }

            match format {
                PixelFormat::ABGR32 => {
                    // Source order: B G R A
                    assert_eq!(cur_frame.len(), buf.len());
                    for (dest, src) in cur_frame.chunks_exact_mut(4).zip(buf.chunks_exact(4)) {
                        assert_eq!(dest.len(), 4);
                        let &[b, g, r, a] = src else { unreachable!() };
                        dest.copy_from_slice(&[r, g, b, a]);
                    }
                }
                _ => unreachable!(),
            }

            Ok(arrival)
        })?;
        if let Some((frame, prev_arrival)) = &prev_frame {
            let millis = arrival.saturating_duration_since(*prev_arrival).as_millis();
            writer.set_frame_delay(millis as u16, 1000)?;
            writer.write_image_data(frame)?;
        }
        prev_frame = Some((cur_frame.clone(), arrival));
        print!(".");
        stdout().flush()?;
    }
    println!();

    writer.finish()?;

    Ok(())
}
