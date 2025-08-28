//! Example: Parse and display HID report descriptors
//!
//! This example shows how to get and interpret HID report descriptors,
//! which define the structure of data sent to/from HID devices.

use hidraw_rs::{Error, HidDevice, Result, enumerate};

/// Basic HID report descriptor item parser
fn parse_report_descriptor(data: &[u8]) {
    println!("\nReport Descriptor Analysis:");
    println!("===========================");

    let mut i = 0;
    let mut indent: u32 = 0;

    while i < data.len() {
        let item = data[i];

        // Get item size (bits 0-1)
        let size = match item & 0x03 {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 4,
            _ => unreachable!(),
        };

        // Get item type (bits 2-3)
        let item_type = (item >> 2) & 0x03;

        // Get item tag (bits 4-7)
        let tag = (item >> 4) & 0x0F;

        // Print indentation
        for _ in 0..indent {
            print!("  ");
        }

        // Parse based on type
        match item_type {
            0 => {
                // Main items
                match tag {
                    0x8 => print!("Input"),
                    0x9 => print!("Output"),
                    0xA => {
                        print!("Collection");
                        indent += 1;
                    }
                    0xB => print!("Feature"),
                    0xC => {
                        indent = indent.saturating_sub(1);
                        print!("End Collection");
                    }
                    _ => print!("Main[{tag:X}]"),
                }
            }
            1 => {
                // Global items
                match tag {
                    0x0 => print!("Usage Page"),
                    0x1 => print!("Logical Minimum"),
                    0x2 => print!("Logical Maximum"),
                    0x3 => print!("Physical Minimum"),
                    0x4 => print!("Physical Maximum"),
                    0x5 => print!("Unit Exponent"),
                    0x6 => print!("Unit"),
                    0x7 => print!("Report Size"),
                    0x8 => print!("Report ID"),
                    0x9 => print!("Report Count"),
                    0xA => print!("Push"),
                    0xB => print!("Pop"),
                    _ => print!("Global[{tag:X}]"),
                }
            }
            2 => {
                // Local items
                match tag {
                    0x0 => print!("Usage"),
                    0x1 => print!("Usage Minimum"),
                    0x2 => print!("Usage Maximum"),
                    0x3 => print!("Designator Index"),
                    0x4 => print!("Designator Minimum"),
                    0x5 => print!("Designator Maximum"),
                    0x7 => print!("String Index"),
                    0x8 => print!("String Minimum"),
                    0x9 => print!("String Maximum"),
                    0xA => print!("Delimiter"),
                    _ => print!("Local[{tag:X}]"),
                }
            }
            _ => print!("Reserved[{tag:X}]"),
        }

        // Print data bytes
        if size > 0 {
            print!(" (");
            for j in 1..=size {
                if i + j < data.len() {
                    if j > 1 {
                        print!(" ");
                    }
                    print!("{:02X}", data[i + j]);
                }
            }
            print!(")");

            // Try to interpret common values
            if size <= 4 && i + size < data.len() {
                let mut value: u32 = 0;
                for j in 0..size {
                    if i + j + 1 < data.len() {
                        value |= (data[i + j + 1] as u32) << (j * 8);
                    }
                }

                // Special interpretations
                match (item_type, tag) {
                    (1, 0) => {
                        // Usage Page
                        let page_name = match value {
                            0x01 => " - Generic Desktop",
                            0x02 => " - Simulation Controls",
                            0x03 => " - VR Controls",
                            0x04 => " - Sport Controls",
                            0x05 => " - Game Controls",
                            0x06 => " - Generic Device Controls",
                            0x07 => " - Keyboard/Keypad",
                            0x08 => " - LED",
                            0x09 => " - Button",
                            0x0A => " - Ordinal",
                            0x0B => " - Telephony Device",
                            0x0C => " - Consumer",
                            0x0D => " - Digitizers",
                            0x0E => " - Haptics",
                            0x0F => " - Physical Input Device",
                            0x10 => " - Unicode",
                            0x14 => " - Alphanumeric Display",
                            0x40 => " - Medical Instrument",
                            0xFF00..=0xFFFF => " - Vendor Defined",
                            _ => "",
                        };
                        print!("{page_name}");
                    }
                    (0, 0xA) => {
                        // Collection
                        let coll_name = match value {
                            0x00 => " - Physical",
                            0x01 => " - Application",
                            0x02 => " - Logical",
                            0x03 => " - Report",
                            0x04 => " - Named Array",
                            0x05 => " - Usage Switch",
                            0x06 => " - Usage Modifier",
                            _ => "",
                        };
                        print!("{coll_name}");
                    }
                    _ => {}
                }

                print!(" = {value}");
            }
        }

        println!();

        // Move to next item
        i += 1 + size;
    }
}

fn main() -> Result<()> {
    println!("HID Report Descriptor Parser Example");
    println!("===================================");

    // Get all HID devices
    let devices = enumerate()?;

    if devices.is_empty() {
        println!("\nNo HID devices found.");
        println!("Note: You may need to run this as root.");
        return Ok(());
    }

    println!("\nFound {} HID device(s)", devices.len());

    // Find an interesting device (keyboard, mouse, or game controller)
    let interesting_device = devices.iter().find(|d| {
        // Look for common HID devices
        d.product
            .as_ref()
            .map(|p| {
                p.to_lowercase().contains("keyboard")
                    || p.to_lowercase().contains("mouse")
                    || p.to_lowercase().contains("gamepad")
                    || p.to_lowercase().contains("controller")
            })
            .unwrap_or(false)
    });

    let device_info = interesting_device.unwrap_or(&devices[0]);

    println!("\nAnalyzing device: {}", device_info.display_name());
    println!("Path: {}", device_info.path.display());

    // Open the device
    let device = match HidDevice::open(device_info) {
        Ok(d) => d,
        Err(Error::PermissionDenied) => {
            println!("\nPermission denied. Try running with sudo.");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    // Get and parse the report descriptor
    match device.get_report_descriptor() {
        Ok(desc) => {
            println!("\nReport Descriptor Size: {} bytes", desc.size);

            if !desc.is_empty() {
                let desc_data = desc.as_bytes();

                // Show raw hex dump
                println!("\nRaw Report Descriptor:");
                for (i, chunk) in desc_data.chunks(16).enumerate() {
                    print!("{:04X}: ", i * 16);
                    for byte in chunk {
                        print!("{byte:02X} ");
                    }
                    println!();
                }

                // Parse the descriptor
                parse_report_descriptor(desc_data);
            }
        }
        Err(e) => {
            println!("\nFailed to get report descriptor: {e}");
        }
    }

    Ok(())
}
