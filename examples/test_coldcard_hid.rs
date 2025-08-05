//! Test HID operations specifically with Coldcard

use hidraw_rs::coldcard::{COINKITE_VID, COLDCARD_PID};
use hidraw_rs::prelude::*;
use std::time::Duration;

fn main() -> Result<()> {
    println!("Testing HID operations with Coldcard");
    println!("====================================\n");

    // Find Coldcard device
    let devices = find_devices(COINKITE_VID, COLDCARD_PID)?;

    if devices.is_empty() {
        eprintln!("No Coldcard devices found!");
        return Ok(());
    }

    let device_info = &devices[0];
    println!("Found Coldcard:");
    println!("  Path: {path}", path = device_info.path.display());
    println!("  Product: {product:?}", product = device_info.product);
    println!("  Serial: {serial:?}", serial = device_info.serial_number);

    // Open the device
    println!("\nOpening device...");
    let mut device = HidDevice::open(device_info)?;
    println!("Device opened successfully!");

    // Test 1: Write a simple command (ping)
    println!("\nTest 1: Sending ping command...");
    let ping_cmd = b"ping";
    let test_data = b"Test from HID!";
    let mut packet = vec![0u8; 64];
    packet[0] = (ping_cmd.len() + test_data.len()) as u8 | 0x80; // Length + last packet flag
    packet[1..5].copy_from_slice(ping_cmd);
    packet[5..5 + test_data.len()].copy_from_slice(test_data);

    match device.write(&packet) {
        Ok(n) => println!("Wrote {n} bytes"),
        Err(e) => println!("Write error: {e}"),
    }

    // Test 2: Read response with timeout
    println!("\nTest 2: Reading response (500ms timeout)...");
    let mut response = vec![0u8; 64];

    match device.read_timeout(&mut response, Duration::from_millis(500)) {
        Ok(n) => {
            println!("Read {n} bytes");
            let len = (response[0] & 0x3F) as usize;
            println!("Response length field: {len}");
            if len > 0 && len < 64 {
                println!(
                    "Response data: {data:?}",
                    data = String::from_utf8_lossy(&response[1..=len])
                );
            }
        }
        Err(Error::Timeout) => println!("Read timed out"),
        Err(e) => println!("Read error: {e}"),
    }

    // Test 3: Write with timeout
    println!("\nTest 3: Write with timeout (100ms)...");
    let version_cmd = b"vers";
    let mut packet = vec![0u8; 64];
    packet[0] = version_cmd.len() as u8 | 0x80;
    packet[1..5].copy_from_slice(version_cmd);

    match device.write_timeout(&packet, Duration::from_millis(100)) {
        Ok(n) => println!("Wrote {n} bytes with timeout"),
        Err(e) => println!("Write timeout error: {e}"),
    }

    // Read version response
    println!("\nReading version response...");
    match device.read_timeout(&mut response, Duration::from_millis(1000)) {
        Ok(n) => {
            println!("Read {n} bytes");
            let len = (response[0] & 0x3F) as usize;
            if len > 0 && len < 64 {
                println!("Version: {version}", version = String::from_utf8_lossy(&response[1..=len]));
            }
        }
        Err(e) => println!("Read error: {e}"),
    }

    // Test 4: Feature reports (may not be supported by Coldcard)
    println!("\nTest 4: Testing feature reports...");
    let mut feature_buf = vec![0u8; 64];
    match device.get_feature_report(0x00, &mut feature_buf) {
        Ok(n) => {
            println!("Got feature report, {n} bytes");
            println!("Data: {:02x?}", &feature_buf[..n.min(16)]);
        }
        Err(e) => {
            println!(
                "Get feature report error: {e} (this is normal for Coldcard)"
            );
        }
    }

    // Test 5: Multiple reads to test continuous operation
    println!("\nTest 5: Multiple read operations...");
    for i in 0..3 {
        println!("\nRead attempt {attempt}", attempt = i + 1);
        match device.read_timeout(&mut response, Duration::from_millis(100)) {
            Ok(n) => println!("Read {} bytes", n),
            Err(Error::Timeout) => println!("Timed out (expected if no data)"),
            Err(e) => println!("Error: {e}"),
        }
    }

    println!("\nAll tests completed!");
    Ok(())
}
