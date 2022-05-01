//! Dumps UVC metadata frames.

use std::{env, path::Path};

use linuxvideo::{
    format::{MetaFormat, Pixelformat},
    uvc::UvcMetadata,
    CapabilityFlags, Device,
};

fn main() -> linuxvideo::Result<()> {
    env_logger::init();

    let mut args = env::args_os().skip(1);

    let path = args.next().ok_or_else(|| format!("usage: uvc <device>"))?;

    let device = Device::open(Path::new(&path))?;

    if !device
        .capabilities()?
        .device_capabilities()
        .contains(CapabilityFlags::META_CAPTURE)
    {
        return Err("device does not support `META_CAPTURE` capability".into());
    }

    let meta = device.meta_capture(MetaFormat::new(Pixelformat::UVC))?;

    let mut stream = meta.into_stream(4)?;

    println!("stream started, waiting for data");
    loop {
        stream.dequeue(|view| {
            let meta = UvcMetadata::from_bytes(&view);
            eprintln!("{:?}", meta);
            Ok(())
        })?;
    }
}
