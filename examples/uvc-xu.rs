//! Queries UVC Extension Units (XUs).

use std::{env, path::Path};

use linuxvideo::{uvc::UvcExt, Device};

fn main() -> linuxvideo::Result<()> {
    env_logger::init();

    let mut args = env::args_os().skip(1);

    let (path, unit_id) = match (args.next(), args.next()) {
        (Some(path), Some(unit_id)) => (path, unit_id),
        _ => {
            println!("usage: uvc-xu <device> <extension unit ID>");
            std::process::exit(1);
        }
    };

    let unit_id: u8 = unit_id
        .to_str()
        .and_then(|id| id.parse().ok())
        .expect("unit ID must be an integer");

    let device = Device::open(Path::new(&path))?;

    let uvc = UvcExt::new(&device);
    let xu = uvc.extension_unit(unit_id);

    for sel in 0..=0xff {
        let res = xu.control_info(sel);
        println!("{:#04x}: {:?}", sel, res);
    }

    Ok(())
}
