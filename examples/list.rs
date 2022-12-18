use std::io;

use linuxvideo::Device;

fn main() -> io::Result<()> {
    for res in linuxvideo::list()? {
        match res.and_then(|device| list_device(device)) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("skipping device due to error: {}", e);
            }
        }
    }

    Ok(())
}

fn list_device(device: Device) -> io::Result<()> {
    let caps = device.capabilities()?;
    println!("- {}: {}", device.path()?.display(), caps.card());
    println!("  driver: {}", caps.driver());
    println!("  bus info: {}", caps.bus_info());
    println!("  all capabilities:    {:?}", caps.all_capabilities());
    println!("  avail. capabilities: {:?}", caps.device_capabilities());

    Ok(())
}
