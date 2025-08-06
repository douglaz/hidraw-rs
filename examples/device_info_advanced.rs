//! Example: Get advanced device information using the public API
//!
//! This example demonstrates how to use HidDevice's methods to get
//! detailed device information including physical location, unique ID, and
//! report descriptors.

use hidraw_rs::{enumerate, Error, HidDevice, Result};

fn print_hex_dump(data: &[u8], max_bytes: usize) {
    let bytes_to_show = data.len().min(max_bytes);
    for (i, chunk) in data[..bytes_to_show].chunks(16).enumerate() {
        print!("{:04x}: ", i * 16);

        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                print!(" ");
            }
            print!("{byte:02x} ");
        }

        // Padding
        for j in chunk.len()..16 {
            if j == 8 {
                print!(" ");
            }
            print!("   ");
        }

        // ASCII representation
        print!(" |");
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == b' ' {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }

    if bytes_to_show < data.len() {
        println!("... ({} more bytes)", data.len() - bytes_to_show);
    }
}

fn show_device_info(device_info: &hidraw_rs::DeviceInfo) -> Result<()> {
    println!("\n=== Device: {} ===", device_info.path.display());

    // Open the device
    let device = match HidDevice::open(device_info) {
        Ok(d) => d,
        Err(Error::PermissionDenied) => {
            println!("Permission denied. Try running with sudo.");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    // Display basic info from DeviceInfo
    println!("\nBasic Info:");
    println!("  Vendor ID: 0x{:04x}", device_info.vendor_id);
    println!("  Product ID: 0x{:04x}", device_info.product_id);
    if let Some(name) = &device_info.product {
        println!("  Product Name: {name}");
    }
    if let Some(mfr) = &device_info.manufacturer {
        println!("  Manufacturer: {mfr}");
    }
    if let Some(serial) = &device_info.serial_number {
        println!("  Serial Number: {serial}");
    }

    // Now use the public API for advanced info

    // Get physical location
    match device.get_physical_info() {
        Ok(phys) => {
            println!("\nPhysical Location: {phys}");
        }
        Err(e) => println!("\nFailed to get physical location: {e}"),
    }

    // Get unique ID
    match device.get_unique_id() {
        Ok(uniq) => {
            if !uniq.is_empty() {
                println!("Unique ID: {uniq}");
            } else {
                println!("Unique ID: (none)");
            }
        }
        Err(e) => println!("Failed to get unique ID: {e}"),
    }

    // Get report descriptor
    println!("\nReport Descriptor:");
    match device.get_report_descriptor() {
        Ok(desc) => {
            println!("  Size: {} bytes", desc.size);
            if !desc.is_empty() {
                println!("  Data (first 256 bytes):");
                print_hex_dump(desc.as_bytes(), 256);
            }
        }
        Err(e) => println!("  Failed to get report descriptor: {e}"),
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("HID Device Advanced Information Example");
    println!("======================================");

    // Get all HID devices
    let devices = enumerate()?;

    if devices.is_empty() {
        println!("\nNo HID devices found.");
        println!(
            "Note: You may need to run this as root or add your user to the appropriate group."
        );
        return Ok(());
    }

    println!("\nFound {} HID device(s)", devices.len());

    // Show detailed info for up to 3 devices (to keep output manageable)
    let max_devices = 3;
    for (i, device_info) in devices.iter().take(max_devices).enumerate() {
        if i > 0 {
            println!("\n----------------------------------------");
        }
        show_device_info(device_info)?;
    }

    if devices.len() > max_devices {
        println!("\n... and {} more device(s)", devices.len() - max_devices);
        println!("(Showing only first {max_devices} devices to keep output manageable)");
    }

    Ok(())
}