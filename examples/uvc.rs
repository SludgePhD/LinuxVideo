//! Dumps UVC metadata frames.

use std::{env, path::Path};

use anyhow::{anyhow, bail};
use linuxvideo::{
    format::{MetaFormat, PixelFormat},
    uvc::UvcMetadata,
    CapabilityFlags, Device,
};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut args = env::args_os().skip(1);

    let path = args.next().ok_or_else(|| anyhow!("usage: uvc <device>"))?;

    let device = Device::open(Path::new(&path))?;

    if !device
        .capabilities()?
        .device_capabilities()
        .contains(CapabilityFlags::META_CAPTURE)
    {
        bail!("device does not support `META_CAPTURE` capability");
    }

    let meta = device.meta_capture(MetaFormat::new(PixelFormat::UVC))?;

    let mut stream = meta.into_stream()?;

    println!("stream started, waiting for data");
    loop {
        stream.dequeue(|view| {
            let meta = UvcMetadata::from_bytes(&view);
            eprintln!("{:?}", meta);
            Ok(())
        })?;
    }
}
