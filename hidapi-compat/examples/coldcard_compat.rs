//! Example showing hidapi compatibility for Coldcard-like usage

use hidapi_compat::{HidApi, HidResult};

const COINKITE_VID: u16 = 0xd13e;
const CKCC_PID: u16 = 0xcc10;

fn main() -> HidResult<()> {
    println!("Testing hidapi compatibility for Coldcard usage pattern");
    println!("=======================================================\n");

    // Create API instance (like rust-coldcard does)
    let mut api = HidApi::new()?;
    println!("✓ Created HidApi instance");

    // Refresh devices (like rust-coldcard's detect method)
    api.refresh_devices()?;
    println!("✓ Refreshed device list");

    // List devices and filter for Coldcard
    let coldcards: Vec<_> = api
        .device_list()
        .filter(|dev| dev.vendor_id() == COINKITE_VID && dev.product_id() == CKCC_PID)
        .collect();

    println!("\nFound {} Coldcard device(s)", coldcards.len());

    // Print device info like rust-coldcard does
    for (i, device) in coldcards.iter().enumerate() {
        println!("\nColdcard #{}:", i + 1);
        println!("  Path: {:?}", device.path());
        println!("  Serial: {:?}", device.serial_number());
        println!("  Manufacturer: {:?}", device.manufacturer_string());
        println!("  Product: {:?}", device.product_string());
    }

    // Try to open a device by serial (like rust-coldcard)
    if let Some(first_device) = coldcards.first() {
        if let Some(serial) = first_device.serial_number() {
            println!("\nTrying to open device with serial: {}", serial);

            match api.open_serial(COINKITE_VID, CKCC_PID, serial) {
                Ok(mut device) => {
                    println!("✓ Opened device successfully");

                    // Test write operation (ping-like)
                    let ping_data = vec![0x00, b'p', b'i', b'n', b'g'];
                    match device.write(&ping_data) {
                        Ok(n) => println!("✓ Wrote {} bytes", n),
                        Err(e) => println!("✗ Write failed: {:?}", e),
                    }

                    // Test read with timeout
                    let mut buf = vec![0u8; 64];
                    match device.read_timeout(&mut buf, 100) {
                        Ok(n) => println!("✓ Read {} bytes", n),
                        Err(e) => println!("✗ Read failed: {:?}", e),
                    }
                }
                Err(e) => {
                    println!("✗ Failed to open device: {:?}", e);
                }
            }
        }
    } else {
        println!("\nNo Coldcard devices found to test with");
    }

    Ok(())
}
