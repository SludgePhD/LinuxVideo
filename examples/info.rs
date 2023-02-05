//! Prints detailed device information.

use std::{env, io, path::Path};

use anyhow::anyhow;
use linuxvideo::{controls::CtrlType, format::FrameSizes, BufType, Device};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut args = env::args_os().skip(1);

    let path = args.next().ok_or_else(|| anyhow!("usage: info <device>"))?;

    let device = Device::open(Path::new(&path))?;

    list_device(device)?;

    Ok(())
}

fn list_device(device: Device) -> io::Result<()> {
    let caps = device.capabilities()?;
    println!("card: {}", caps.card());
    println!("driver: {}", caps.driver());
    println!("bus info: {}", caps.bus_info());
    println!("all capabilities:    {:?}", caps.all_capabilities());
    println!("avail. capabilities: {:?}", caps.device_capabilities());
    for buf in device.supported_buf_types() {
        println!("- supported formats for {:?} buffers:", buf);
        for res in device.formats(buf) {
            match res {
                Ok(fmt) => {
                    println!("  - [{}] {}", fmt.pixel_format(), fmt.description());
                    if !fmt.flags().is_empty() {
                        println!("    {:?}", fmt.flags());
                    }

                    if buf == BufType::META_CAPTURE || buf == BufType::META_OUTPUT {
                        // Metadata formats like `UVCH` will return an error from `frame_sizes`.
                        continue;
                    }

                    let sizes = match device.frame_sizes(fmt.pixel_format()) {
                        Ok(sizes) => sizes,
                        Err(e) => {
                            println!("  - error while fetching supported frame sizes: {}", e);
                            continue;
                        }
                    };
                    match sizes {
                        FrameSizes::Discrete(iter) => {
                            for size in iter {
                                let intervals = device.frame_intervals(
                                    fmt.pixel_format(),
                                    size.width(),
                                    size.height(),
                                )?;
                                println!(
                                    "    - [{:2}] {}x{} @ {}",
                                    size.index(),
                                    size.width(),
                                    size.height(),
                                    intervals,
                                );
                            }
                        }
                        FrameSizes::Stepwise(s) => {
                            let min_ivals = device.frame_intervals(
                                fmt.pixel_format(),
                                s.min_width(),
                                s.min_height(),
                            )?;
                            let max_ivals = device.frame_intervals(
                                fmt.pixel_format(),
                                s.max_width(),
                                s.max_height(),
                            )?;
                            println!(
                                "    - {}x{} to {}x{} (step {}x{}) @ {} to {}",
                                s.min_width(),
                                s.min_height(),
                                s.max_width(),
                                s.max_height(),
                                s.step_width(),
                                s.step_height(),
                                min_ivals,
                                max_ivals,
                            );
                        }
                        FrameSizes::Continuous(s) => {
                            let min_ivals = device.frame_intervals(
                                fmt.pixel_format(),
                                s.min_width(),
                                s.min_height(),
                            )?;
                            let max_ivals = device.frame_intervals(
                                fmt.pixel_format(),
                                s.max_width(),
                                s.max_height(),
                            )?;
                            println!(
                                "    - {}x{} to {}x{} @ {} to {}",
                                s.min_width(),
                                s.min_height(),
                                s.max_width(),
                                s.max_height(),
                                min_ivals,
                                max_ivals,
                            );
                        }
                    }
                }
                Err(e) => {
                    println!("  - error: {}", e);
                }
            }
        }

        println!("- active format for {:?} buffer:", buf);
        match device.format(buf) {
            Ok(format) => {
                println!("  {:?}", format);
            }
            Err(e) => {
                println!("  error: {}", e);
            }
        }
    }

    println!("- inputs:");
    for res in device.inputs() {
        match res {
            Ok(input) => {
                println!("  - [{:?}] {}", input.input_type(), input.name());
                println!("    audioset: {:#b}", input.audioset());
                println!("    tuner: {}", input.tuner());
                println!("    std: {:?}", input.std());
                println!("    capabilities: {:?}", input.capabilities());
            }
            Err(e) => {
                println!("  - error: {}", e);
            }
        }
    }
    println!("- outputs:");
    for res in device.outputs() {
        match res {
            Ok(output) => {
                println!("  - [{:?}] {}", output.output_type(), output.name());
                println!("    audioset: {:#b}", output.audioset());
                println!("    modulator: {}", output.modulator());
                println!("    std: {:?}", output.std());
                println!("    capabilities: {:?}", output.capabilities());
            }
            Err(e) => {
                println!("  - error: {}", e);
            }
        }
    }
    println!("- controls:");
    for res in device.controls() {
        match res {
            Ok(desc) => {
                print!(
                    "  - [{:?}] \"{}\", {:?}",
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
                    println!("    {:?}", desc.flags());
                }

                if desc.control_type() == CtrlType::MENU {
                    // Enumerate menu options.
                    for res in device.enumerate_menu(&desc) {
                        match res {
                            Ok(item) => {
                                println!("    {}: {}", item.index(), item.name());
                            }
                            Err(e) => {
                                println!("    error: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("  - error: {}", e);
            }
        }
    }
    println!();

    Ok(())
}
