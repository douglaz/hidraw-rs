//! Example: Basic HID device communication

use hidraw_rs::prelude::*;
use std::time::Duration;

fn main() -> Result<()> {
    // Get the first HID device
    let devices = enumerate()?;

    if devices.is_empty() {
        eprintln!("No HID devices found!");
        return Ok(());
    }

    // Use the first device or specify a path
    let device_info = &devices[0];
    println!("Opening device: {name}", name = device_info.display_name());
    println!("Path: {path}", path = device_info.path.display());

    // Open the device
    let mut device = HidDevice::open(device_info)?;

    // Set a read timeout
    device.set_read_timeout(Some(Duration::from_secs(1)));

    // Example: Send a simple output report
    println!("\nSending test data...");
    let test_data = vec![0x00, 0x01, 0x02, 0x03];
    match device.write(&test_data) {
        Ok(n) => println!("Wrote {n} bytes"),
        Err(e) => eprintln!("Write failed: {e}"),
    }

    // Example: Write with explicit timeout
    println!("\nSending data with 500ms timeout...");
    let test_data2 = vec![0x04, 0x05, 0x06, 0x07];
    match device.write_timeout(&test_data2, Duration::from_millis(500)) {
        Ok(n) => println!("Wrote {n} bytes with timeout"),
        Err(Error::Timeout) => println!("Write timed out"),
        Err(e) => eprintln!("Write failed: {e}"),
    }

    // Example: Read input report with timeout
    println!("\nReading from device (1 second timeout)...");
    let mut buffer = vec![0u8; 64];

    match device.read(&mut buffer) {
        Ok(n) => {
            println!("Read {n} bytes:");
            println!("Data: {:02x?}", &buffer[..n]);
        }
        Err(Error::Timeout) => {
            println!("Read timed out (no data available)");
        }
        Err(e) => {
            eprintln!("Read failed: {e}");
        }
    }

    // Example: Read with explicit timeout
    println!("\nReading with explicit 200ms timeout...");
    match device.read_timeout(&mut buffer, Duration::from_millis(200)) {
        Ok(n) => {
            println!("Read {n} bytes:");
            println!("Data: {:02x?}", &buffer[..n]);
        }
        Err(Error::Timeout) => {
            println!("Read timed out after 200ms");
        }
        Err(e) => {
            eprintln!("Read failed: {e}");
        }
    }

    // Example: Get/Set feature report
    println!("\nTrying to get feature report 0x01...");
    let mut feature_buf = vec![0u8; 64];
    match device.get_feature_report(0x01, &mut feature_buf) {
        Ok(n) => {
            println!("Got feature report, {n} bytes:");
            println!("Data: {:02x?}", &feature_buf[..n]);
        }
        Err(e) => {
            eprintln!("Get feature report failed: {e}");
        }
    }

    Ok(())
}
