//! Writes test data to a video device.
//!
//! TODO: this is very rough and doesn't do much

use core::slice;
use std::{env, mem, path::Path, thread, time::Duration};

use livid::{
    format::{Format, PixFormat},
    CapabilityFlags, Device, Pixelformat,
};

const WIDTH: u32 = 120;
const HEIGHT: u32 = 60;
const PIXFMT: Pixelformat = Pixelformat::RGB32;

const RED: [u8; 4] = [0xff, 0xff, 0, 0];
const BLACK: [u8; 4] = [0xff, 0, 0, 0];
const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

static DATA: [[[u8; 4]; 3]; 3] = [
    [RED, RED, RED],
    [BLACK, BLACK, BLACK],
    [TRANSPARENT, TRANSPARENT, TRANSPARENT],
];

fn main() -> livid::Result<()> {
    let mut args = env::args_os().skip(1);

    let path = args
        .next()
        .ok_or_else(|| format!("usage: write <device>"))?;

    let mut device = Device::open(Path::new(&path))?;
    if !device
        .capabilities()?
        .device_capabilities()
        .contains(CapabilityFlags::VIDEO_OUTPUT)
    {
        return Err(format!(
            "cannot write data: selected device does not support `VIDEO_OUTPUT` capability"
        )
        .into());
    }

    let format = Format::VideoOutput(PixFormat::new(WIDTH, HEIGHT, PIXFMT));
    let format = device.set_format_raw(format)?;
    println!("set format: {:?}", format);

    let fmt = match format {
        Format::VideoOutput(fmt) => fmt,
        _ => unreachable!(),
    };
    if fmt.width() != WIDTH || fmt.height() != HEIGHT || fmt.pixelformat() != PIXFMT {
        return Err(format!("driver does not support the requested parameters").into());
    }

    let image_bytes: &[u8] =
        unsafe { slice::from_raw_parts(&DATA as *const _ as *const _, mem::size_of_val(&DATA)) };
    let mut data = image_bytes.to_vec();
    data.resize(fmt.size_image() as usize, 0xff);

    assert_eq!(data.len(), fmt.size_image() as usize);

    loop {
        device.write(image_bytes)?;
        thread::sleep(Duration::from_millis(16));
    }
}
