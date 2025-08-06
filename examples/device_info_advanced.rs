//! Example: Get advanced device information using low-level ioctl functions
//!
//! This example demonstrates how to use the lower-level hidraw API to get
//! detailed device information including physical location, unique ID, and
//! report descriptors.

use hidraw_rs::hidraw::{self, HidrawDevice};
use hidraw_rs::{Error, Result};
use std::path::Path;

fn print_hex_dump(data: &[u8], max_bytes: usize) {
    let bytes_to_show = data.len().min(max_bytes);
    for (i, chunk) in data[..bytes_to_show].chunks(16).enumerate() {
        print!("{:04x}: ", i * 16);

        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                print!(" ");
            }
            print!("{:02x} ", byte);
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

fn show_device_info(path: &Path) -> Result<()> {
    println!("\n=== Device: {} ===", path.display());

    // Open the device
    let device = match HidrawDevice::open(path) {
        Ok(d) => d,
        Err(Error::PermissionDenied) => {
            println!("Permission denied. Try running with sudo.");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    // Get basic info using the public API
    match device.get_raw_info() {
        Ok(info) => {
            println!("\nBasic Info:");
            println!("  Bus Type: 0x{:02x}", info.bustype);
            println!("  Vendor ID: 0x{:04x}", info.vendor);
            println!("  Product ID: 0x{:04x}", info.product);
        }
        Err(e) => println!("Failed to get basic info: {}", e),
    }

    // Get device name
    match device.get_raw_name() {
        Ok(name) => {
            println!("  Device Name: {}", name);
        }
        Err(e) => println!("Failed to get device name: {}", e),
    }

    // Now use the low-level ioctl functions directly for advanced info

    // Get physical location
    match hidraw_rs::hidraw::ioctl::get_raw_phys(&device) {
        Ok(phys) => {
            println!("\nPhysical Location: {}", phys);
        }
        Err(e) => println!("\nFailed to get physical location: {}", e),
    }

    // Get unique ID
    match hidraw_rs::hidraw::ioctl::get_raw_uniq(&device) {
        Ok(uniq) => {
            if !uniq.is_empty() {
                println!("Unique ID: {}", uniq);
            } else {
                println!("Unique ID: (none)");
            }
        }
        Err(e) => println!("Failed to get unique ID: {}", e),
    }

    // Get report descriptor
    println!("\nReport Descriptor:");
    match hidraw_rs::hidraw::ioctl::get_report_descriptor(&device) {
        Ok(desc) => {
            println!("  Size: {} bytes", desc.size);
            if desc.size > 0 {
                println!("  Data (first 256 bytes):");
                print_hex_dump(&desc.value[..desc.size as usize], 256);
            }
        }
        Err(e) => println!("  Failed to get report descriptor: {}", e),
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("HID Device Advanced Information Example");
    println!("======================================");

    // Get all HID devices
    let devices = hidraw::enumerate()?;

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
        show_device_info(&device_info.path)?;
    }

    if devices.len() > max_devices {
        println!("\n... and {} more device(s)", devices.len() - max_devices);
        println!(
            "(Showing only first {} devices to keep output manageable)",
            max_devices
        );
    }

    Ok(())
}
