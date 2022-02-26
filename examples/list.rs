use livid::{controls::CtrlType, Device};

fn main() -> livid::Result<()> {
    for res in livid::list()? {
        match res.and_then(|device| list_device(device)) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("skipping device due to error: {}", e);
            }
        }
    }

    Ok(())
}

fn list_device(device: Device) -> livid::Result<()> {
    let caps = device.capabilities()?;
    println!("- {}: {}", device.path()?.display(), caps.card());
    println!("  driver: {}", caps.driver());
    println!("  bus info: {}", caps.bus_info());
    println!("  all capabilities:    {:?}", caps.all_capabilities());
    println!("  avail. capabilities: {:?}", caps.device_capabilities());
    for buf in device.supported_buf_types() {
        println!("  - supported formats for {:?} buffers:", buf);
        for res in device.formats(buf) {
            match res {
                Ok(fmt) => {
                    println!("    - [{}] {}", fmt.pixelformat(), fmt.description());
                    if !fmt.flags().is_empty() {
                        println!("      {:?}", fmt.flags());
                    }
                }
                Err(e) => {
                    println!("    - error: {}", e);
                }
            }
        }

        println!("  - active format for {:?} buffer:", buf);
        match device.format(buf) {
            Ok(format) => {
                println!("    {:?}", format);
            }
            Err(e) => {
                println!("    error: {}", e);
            }
        }
    }

    println!("  - inputs:");
    for res in device.inputs() {
        match res {
            Ok(input) => {
                println!("    - [{:?}] {}", input.input_type(), input.name());
                println!("      audioset: {:#b}", input.audioset());
                println!("      tuner: {}", input.tuner());
                println!("      std: {:?}", input.std());
                println!("      capabilities: {:?}", input.capabilities());
            }
            Err(e) => {
                println!("    - error: {}", e);
            }
        }
    }
    println!("  - outputs:");
    for res in device.outputs() {
        match res {
            Ok(output) => {
                println!("    - [{:?}] {}", output.output_type(), output.name());
                println!("      audioset: {:#b}", output.audioset());
                println!("      modulator: {}", output.modulator());
                println!("      std: {:?}", output.std());
                println!("      capabilities: {:?}", output.capabilities());
            }
            Err(e) => {
                println!("    - error: {}", e);
            }
        }
    }
    println!("  - controls:");
    for res in device.controls() {
        match res {
            Ok(desc) => {
                print!(
                    "    - [{:?}] \"{}\", {:?}",
                    desc.id(),
                    desc.name(),
                    desc.control_type()
                );

                match desc.control_type() {
                    CtrlType::INTEGER => {
                        print!(" [{}-{}", desc.minimum(), desc.maximum());
                        let step = desc.step();
                        if step != 1 {
                            print!(", step={step}");
                        }
                        print!(", default={}]", desc.default_value());
                    }
                    CtrlType::MENU => {
                        print!(" [{}-{}]", desc.minimum(), desc.maximum());
                    }
                    _ => {}
                }

                println!();
                if !desc.flags().is_empty() {
                    println!("      {:?}", desc.flags());
                }

                if desc.control_type() == CtrlType::MENU {
                    // Enumerate menu options.
                    for res in device.enumerate_menu(&desc) {
                        match res {
                            Ok(item) => {
                                println!("      {}: {}", item.index(), item.name());
                            }
                            Err(e) => {
                                println!("      error: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("    - error: {}", e);
            }
        }
    }
    println!();

    Ok(())
}
