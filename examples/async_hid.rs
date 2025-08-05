//! Example: Async HID device communication
//!
//! This example demonstrates using the async API for HID device communication.
//! Run with: cargo run --example async_hid --features async

#[cfg(feature = "async")]
use hidraw_rs::async_io::AsyncHidDevice;
#[cfg(feature = "async")]
use hidraw_rs::prelude::*;
#[cfg(feature = "async")]
use std::time::Duration;

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature. Run with:");
    eprintln!("  cargo run --example async_hid --features async");
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<()> {
    println!("Async HID Device Example");
    println!("========================\n");

    // Enumerate devices (sync operation)
    let devices = enumerate()?;

    if devices.is_empty() {
        eprintln!("No HID devices found!");
        eprintln!("Note: You may need to run this as root or add udev rules.");
        return Ok(());
    }

    println!("Found {count} HID device(s):", count = devices.len());
    for (i, device) in devices.iter().enumerate() {
        println!("  [{i}] {name}", name = device.display_name());
        println!("      Path: {path}", path = device.path.display());
    }

    // Use the first device
    let device_info = &devices[0];
    println!(
        "\nOpening device: {name}",
        name = device_info.display_name()
    );

    // Open the device asynchronously
    let mut device = AsyncHidDevice::open(device_info).await?;
    println!("Device opened successfully!");

    // Example: Write data with timeout
    println!("\nWriting test data with 500ms timeout...");
    let test_data = vec![0x00, 0x01, 0x02, 0x03, 0x04];

    match device
        .write_timeout(&test_data, Duration::from_millis(500))
        .await
    {
        Ok(n) => println!("Wrote {n} bytes"),
        Err(Error::Timeout) => println!("Write timed out"),
        Err(e) => eprintln!("Write failed: {e}"),
    }

    // Example: Read with short timeout
    println!("\nReading with 100ms timeout...");
    let mut buffer = vec![0u8; 64];

    match device
        .read_timeout(&mut buffer, Duration::from_millis(100))
        .await
    {
        Ok(n) => {
            println!("Read {n} bytes:");
            println!("Data: {:02x?}", &buffer[..n]);
        }
        Err(Error::Timeout) => {
            println!("Read timed out (expected for most devices without continuous input)");
        }
        Err(e) => {
            eprintln!("Read failed: {e}");
        }
    }

    // Example: Multiple concurrent operations
    println!("\nDemonstrating concurrent operations...");

    // Note: In a real application, you'd need proper synchronization
    // for concurrent access to the same device

    // Example: Feature reports (still synchronous as ioctl doesn't have async variant)
    println!("\nTrying to get feature report 0x01...");
    let mut feature_buf = vec![0u8; 64];

    match device.get_feature_report(0x01, &mut feature_buf) {
        Ok(n) => {
            println!("Got feature report, {} bytes:", n);
            println!("Data: {:02x?}", &feature_buf[..n]);
        }
        Err(e) => {
            println!(
                "Get feature report failed: {} (this is normal for many devices)",
                e
            );
        }
    }

    // Example: Send feature report
    println!("\nTrying to send feature report...");
    let feature_data = vec![0x01, 0x00, 0xFF]; // Report ID 0x01

    match device.send_feature_report(&feature_data) {
        Ok(()) => println!("Feature report sent successfully"),
        Err(e) => println!(
            "Send feature report failed: {} (this is normal for many devices)",
            e
        ),
    }

    println!("\nAsync HID example completed!");
    Ok(())
}
