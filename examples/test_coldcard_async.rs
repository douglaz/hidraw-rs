//! Test async HID operations with Coldcard

#[cfg(feature = "async")]
use hidraw_rs::async_io::AsyncHidDevice;
#[cfg(feature = "async")]
use hidraw_rs::coldcard::{COINKITE_VID, COLDCARD_PID};
#[cfg(feature = "async")]
use hidraw_rs::prelude::*;
#[cfg(feature = "async")]
use std::time::Duration;

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature. Run with:");
    eprintln!("  cargo run --example test_coldcard_async --features async");
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<()> {
    println!("Testing async HID operations with Coldcard");
    println!("=========================================\n");

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

    // Open the device asynchronously
    println!("\nOpening device asynchronously...");
    let mut device = AsyncHidDevice::open(device_info).await?;
    println!("Device opened successfully!");

    // Test 1: Async write
    println!("\nTest 1: Async write (ping command)...");
    let ping_cmd = b"ping";
    let test_data = b"Async test!";
    let mut packet = vec![0u8; 64];
    packet[0] = (ping_cmd.len() + test_data.len()) as u8 | 0x80;
    packet[1..5].copy_from_slice(ping_cmd);
    packet[5..5 + test_data.len()].copy_from_slice(test_data);

    match device.write(&packet).await {
        Ok(n) => println!("Wrote {n} bytes asynchronously"),
        Err(e) => println!("Async write error: {e}"),
    }

    // Test 2: Async read with timeout
    println!("\nTest 2: Async read with timeout (500ms)...");
    let mut response = vec![0u8; 64];

    match device
        .read_timeout(&mut response, Duration::from_millis(500))
        .await
    {
        Ok(n) => {
            println!("Read {n} bytes asynchronously");
            let len = (response[0] & 0x3F) as usize;
            if len > 0 && len < 64 {
                println!(
                    "Response: {response:?}",
                    response = String::from_utf8_lossy(&response[1..=len])
                );
            }
        }
        Err(Error::Timeout) => println!("Async read timed out"),
        Err(e) => println!("Async read error: {e}"),
    }

    // Test 3: Async write with timeout
    println!("\nTest 3: Async write with timeout (version command)...");
    let version_cmd = b"vers";
    let mut packet = vec![0u8; 64];
    packet[0] = version_cmd.len() as u8 | 0x80;
    packet[1..5].copy_from_slice(version_cmd);

    match device
        .write_timeout(&packet, Duration::from_millis(100))
        .await
    {
        Ok(n) => println!("Wrote {n} bytes with timeout"),
        Err(e) => println!("Write timeout error: {e}"),
    }

    // Read version response
    println!("\nReading version response asynchronously...");
    match device
        .read_timeout(&mut response, Duration::from_millis(1000))
        .await
    {
        Ok(n) => {
            println!("Read {n} bytes");
            let len = (response[0] & 0x3F) as usize;
            if len > 0 && len < 64 {
                println!(
                    "Version: {version}",
                    version = String::from_utf8_lossy(&response[1..=len])
                );
            }
        }
        Err(e) => println!("Read error: {e}"),
    }

    // Test 4: Concurrent operations (demonstrate async capabilities)
    println!("\nTest 4: Concurrent timeout operations...");

    let mut handles = vec![];

    // Spawn 3 async read operations that will timeout
    for i in 0..3 {
        let handle = tokio::spawn(async move {
            println!("  Task {i} starting...");
            // Simulate read that will timeout
            tokio::time::sleep(Duration::from_millis(50 * i as u64)).await;
            println!("  Task {i} completed");
            i
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let result = handle.await.unwrap();
        println!("Task {result} result collected");
    }

    println!("\nAll async tests completed!");
    Ok(())
}
