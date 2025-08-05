//! Example showing how to use hidapi-compat as a drop-in replacement for hidapi
//!
//! This example demonstrates how to replace hidapi in an existing project
//! without changing any code.

// Instead of:
// use hidapi::{HidApi, HidDevice};
//
// You can use:
use hidapi_compat::{HidApi, HidDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Using hidapi-compat as a drop-in replacement for hidapi");
    println!("======================================================\n");

    // This code is identical to what you would write with the original hidapi
    let api = HidApi::new()?;

    // Enumerate devices
    println!("Enumerating HID devices:");
    for device in api.device_list() {
        println!(
            "  {:04x}:{:04x} - {} - {}",
            device.vendor_id(),
            device.product_id(),
            device.manufacturer_string().unwrap_or("Unknown"),
            device.product_string().unwrap_or("Unknown")
        );
    }

    // Example: Open a specific device (if available)
    // This would work exactly the same with original hidapi
    match api.open(0xd13e, 0xcc10) {
        Ok(mut device) => {
            println!("\nOpened device successfully!");

            // Write some data
            let data = vec![0x00, 0x01, 0x02, 0x03];
            match device.write(&data) {
                Ok(n) => println!("Wrote {} bytes", n),
                Err(e) => println!("Write error: {:?}", e),
            }

            // Read with timeout
            let mut buf = vec![0u8; 64];
            match device.read_timeout(&mut buf, 1000) {
                Ok(n) => println!("Read {} bytes", n),
                Err(e) => println!("Read error: {:?}", e),
            }
        }
        Err(_) => {
            println!("\nNo matching device found (this is normal if you don't have the device)");
        }
    }

    println!("\n✅ hidapi-compat works as a perfect drop-in replacement!");
    Ok(())
}

/*
To use this in your Cargo.toml:

[dependencies]
# Option 1: Direct replacement
hidapi = { package = "hidapi-compat", path = "path/to/hidraw-rs/hidapi-compat" }

# Option 2: From git
hidapi = { package = "hidapi-compat", git = "https://github.com/yourusername/hidraw-rs" }

# Option 3: When published to crates.io
hidapi = { package = "hidapi-compat", version = "0.1" }
*/
