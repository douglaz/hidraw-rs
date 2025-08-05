//! Test that hidapi-compat properly translates timeouts to match hidapi behavior

use hidapi_compat::{HidApi, HidDevice};
use std::time::Duration;

#[test]
fn test_timeout_returns_zero() {
    // This test verifies that when a timeout occurs, we return Ok(0)
    // to match hidapi behavior, not an error

    let api = match HidApi::new() {
        Ok(api) => api,
        Err(_) => {
            println!("Skipping test - no HID support available");
            return;
        }
    };

    // Try to find any HID device for testing
    let devices: Vec<_> = api.device_list().collect();
    if devices.is_empty() {
        println!("Skipping test - no HID devices found");
        return;
    }

    // Try to open the first device
    let device_info = &devices[0];
    let mut device = match device_info.open_device(&api) {
        Ok(dev) => dev,
        Err(_) => {
            println!("Skipping test - couldn't open device");
            return;
        }
    };

    // Test read_timeout with a very short timeout
    let mut buf = [0u8; 64];
    match device.read_timeout(&mut buf, 1) {
        Ok(0) => {
            // This is the expected behavior - timeout returns 0
            println!("✓ Timeout correctly returned Ok(0)");
        }
        Ok(n) => {
            // Device actually had data ready
            println!("Device returned {} bytes (had data ready)", n);
        }
        Err(e) => {
            panic!(
                "read_timeout should not return error on timeout, got: {:?}",
                e
            );
        }
    }

    // Test non-blocking read
    device.set_blocking_mode(false).unwrap();
    match device.read(&mut buf) {
        Ok(0) => {
            println!("✓ Non-blocking read correctly returned Ok(0)");
        }
        Ok(n) => {
            println!("Device returned {} bytes (had data ready)", n);
        }
        Err(e) => {
            panic!("Non-blocking read should not return error, got: {:?}", e);
        }
    }
}

#[test]
fn test_timeout_behavior_documented() {
    // This test just documents the expected behavior
    println!("hidapi behavior on timeout:");
    println!("- read_timeout() returns Ok(0) when no data available within timeout");
    println!("- This is NOT an error condition");
    println!("- Applications like rust-coldcard depend on this behavior");
}
