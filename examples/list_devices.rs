//! Example: List all HID devices on the system

use hidraw_rs::prelude::*;

fn main() -> Result<()> {
    println!("Enumerating HID devices...\n");

    // Get all HID devices
    let devices = enumerate()?;

    if devices.is_empty() {
        println!("No HID devices found.");
        println!("\nNote: You may need to run this as root or add your user to the appropriate group.");
        return Ok(());
    }

    println!("Found {} HID device(s):\n", devices.len());

    // Display each device
    for (i, device) in devices.iter().enumerate() {
        println!("Device {}:", i + 1);
        println!("  Path: {}", device.path.display());
        println!("  Vendor ID: 0x{:04x}", device.vendor_id);
        println!("  Product ID: 0x{:04x}", device.product_id);
        
        if let Some(manufacturer) = &device.manufacturer {
            println!("  Manufacturer: {}", manufacturer);
        }
        
        if let Some(product) = &device.product {
            println!("  Product: {}", product);
        }
        
        if let Some(serial) = &device.serial_number {
            println!("  Serial: {}", serial);
        }
        
        println!("  Interface: {}", device.interface_number);
        println!();
    }

    // Look for specific devices
    println!("Looking for Coldcard devices...");
    let coldcards = find_devices(0xd13e, 0xcc10)?;
    
    if coldcards.is_empty() {
        println!("No Coldcard devices found.");
    } else {
        println!("Found {} Coldcard device(s)!", coldcards.len());
        for device in &coldcards {
            println!("  - {}", device.path.display());
        }
    }

    Ok(())
}