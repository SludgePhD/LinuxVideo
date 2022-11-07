use std::{env, path::Path};

use linuxvideo::Device;

fn main() -> linuxvideo::Result<()> {
    env_logger::init();

    let mut args = env::args_os().skip(1);

    let path = match args.next() {
        Some(path) => path,
        None => {
            println!("usage: control <device> <control> [<value>]");
            std::process::exit(1);
        }
    };

    let control_name = args.next();
    let control_name = match control_name.as_ref() {
        Some(name) => Some(name.to_str().expect("control name must be UTF-8")),
        None => None,
    };
    let value = args.next();
    let value = match value.as_ref() {
        Some(value) => Some(
            value
                .to_str()
                .and_then(|id| id.parse().ok())
                .expect("control name must be UTF-8"),
        ),
        None => None,
    };

    let mut device = Device::open(Path::new(&path))?;

    let control_name = match control_name {
        Some(name) => name,
        None => {
            for desc in device.controls() {
                let desc = desc?;
                println!("{:?}", desc);
            }
            return Ok(());
        }
    };

    let mut cid = None;
    for desc in device.controls() {
        let desc = desc?;
        if desc.name().eq_ignore_ascii_case(control_name) {
            println!(
                "'{}' matches control {:?} [{}-{}, step {}, default {}]",
                control_name,
                desc.id(),
                desc.minimum(),
                desc.maximum(),
                desc.step(),
                desc.default_value(),
            );
            cid = Some(desc.id());
            break;
        }
    }

    match cid {
        Some(cid) => match value {
            Some(value) => {
                device.write_control_raw(cid, value)?;
            }
            None => {
                let value = device.read_control_raw(cid)?;
                println!("{:?} control value: {}", cid, value);
            }
        },
        None => panic!("device does not have control named {}", control_name),
    }

    Ok(())
}
