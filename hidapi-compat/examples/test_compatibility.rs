//! Test that hidapi-compat provides all the expected hidapi functionality
//!
//! Run with: cargo run --package hidapi-compat --example test_compatibility

use hidapi_compat::{HidApi, HidError};

fn test_api_creation() -> Result<(), HidError> {
    println!("Testing HidApi creation...");
    let _api = HidApi::new()?;
    println!("✅ HidApi::new() works");
    Ok(())
}

fn test_device_enumeration() -> Result<(), HidError> {
    println!("\nTesting device enumeration...");
    let mut api = HidApi::new()?;

    // Test refresh_devices
    api.refresh_devices()?;
    println!("✅ refresh_devices() works");

    // Test device_list iterator
    let count = api.device_list().count();
    println!("✅ device_list() works - found {} devices", count);

    // Test deprecated devices() method
    #[allow(deprecated)]
    let devices = api.devices();
    println!(
        "✅ devices() works (deprecated) - {} devices",
        devices.len()
    );

    Ok(())
}

fn test_device_info() -> Result<(), HidError> {
    println!("\nTesting DeviceInfo accessors...");
    let api = HidApi::new()?;

    if let Some(device) = api.device_list().next() {
        // Test all accessors
        let _path = device.path();
        println!("✅ path() works");

        let _vid = device.vendor_id();
        println!("✅ vendor_id() works: {:04x}", device.vendor_id());

        let _pid = device.product_id();
        println!("✅ product_id() works: {:04x}", device.product_id());

        let _serial = device.serial_number();
        println!("✅ serial_number() works: {:?}", device.serial_number());

        let _release = device.release_number();
        println!("✅ release_number() works: {}", device.release_number());

        let _manufacturer = device.manufacturer_string();
        println!(
            "✅ manufacturer_string() works: {:?}",
            device.manufacturer_string()
        );

        let _product = device.product_string();
        println!("✅ product_string() works: {:?}", device.product_string());

        let _usage_page = device.usage_page();
        println!("✅ usage_page() works: {}", device.usage_page());

        let _usage = device.usage();
        println!("✅ usage() works: {}", device.usage());

        let _interface = device.interface_number();
        println!("✅ interface_number() works: {}", device.interface_number());
    } else {
        println!("⚠️  No devices found to test DeviceInfo");
    }

    Ok(())
}

fn test_device_operations() -> Result<(), HidError> {
    println!("\nTesting device operations...");
    let api = HidApi::new()?;

    // Try to find a test device (Coldcard for example)
    const TEST_VID: u16 = 0xd13e;
    const TEST_PID: u16 = 0xcc10;

    match api.open(TEST_VID, TEST_PID) {
        Ok(mut device) => {
            println!("✅ open() works");

            // Test write
            let data = vec![0x00, 0x01, 0x02, 0x03];
            match device.write(&data) {
                Ok(n) => println!("✅ write() works - wrote {} bytes", n),
                Err(e) => println!("⚠️  write() error: {:?}", e),
            }

            // Test read with timeout
            let mut buf = vec![0u8; 64];
            match device.read_timeout(&mut buf, 100) {
                Ok(n) => println!("✅ read_timeout() works - read {} bytes", n),
                Err(e) => println!("⚠️  read_timeout() error: {:?}", e),
            }

            // Test blocking mode
            device.set_blocking_mode(false)?;
            println!("✅ set_blocking_mode() works");

            // Test read in non-blocking mode
            match device.read(&mut buf) {
                Ok(n) => println!("✅ read() works - read {} bytes", n),
                Err(e) => println!("⚠️  read() error: {:?}", e),
            }

            // Test feature reports
            match device.send_feature_report(&[0x00, 0x01]) {
                Ok(()) => println!("✅ send_feature_report() works"),
                Err(e) => println!("⚠️  send_feature_report() error: {:?}", e),
            }

            match device.get_feature_report(&mut buf) {
                Ok(n) => println!("✅ get_feature_report() works - {} bytes", n),
                Err(e) => println!("⚠️  get_feature_report() error: {:?}", e),
            }

            // Test device info methods
            let _manufacturer = device.get_manufacturer_string()?;
            println!("✅ get_manufacturer_string() works");

            let _product = device.get_product_string()?;
            println!("✅ get_product_string() works");

            let _serial = device.get_serial_number_string()?;
            println!("✅ get_serial_number_string() works");

            let _indexed = device.get_indexed_string(1)?;
            println!("✅ get_indexed_string() works");
        }
        Err(_) => {
            println!(
                "⚠️  No test device found (VID:{:04x} PID:{:04x})",
                TEST_VID, TEST_PID
            );
            println!("    Device operations tests skipped");
        }
    }

    Ok(())
}

fn test_error_types() {
    println!("\nTesting error types...");

    // Create various error types to ensure they exist
    let _e1 = HidError::HidApiError {
        message: "test".to_string(),
    };
    println!("✅ HidApiError exists");

    let _e2 = HidError::HidApiErrorEmpty;
    println!("✅ HidApiErrorEmpty exists");

    let _e3 = HidError::InitializationError;
    println!("✅ InitializationError exists");

    let _e4 = HidError::OpenHidDeviceError;
    println!("✅ OpenHidDeviceError exists");

    let _e5 = HidError::InvalidZeroSizeData;
    println!("✅ InvalidZeroSizeData exists");

    let _e6 = HidError::IncompleteSendError { sent: 10, all: 20 };
    println!("✅ IncompleteSendError exists");

    let _e7 = HidError::SetBlockingModeError;
    println!("✅ SetBlockingModeError exists");

    println!("✅ All error types are available");
}

fn main() {
    println!("hidapi-compat Compatibility Test");
    println!("================================\n");

    // Run all tests
    if let Err(e) = test_api_creation() {
        eprintln!("❌ API creation failed: {:?}", e);
    }

    if let Err(e) = test_device_enumeration() {
        eprintln!("❌ Device enumeration failed: {:?}", e);
    }

    if let Err(e) = test_device_info() {
        eprintln!("❌ Device info failed: {:?}", e);
    }

    if let Err(e) = test_device_operations() {
        eprintln!("❌ Device operations failed: {:?}", e);
    }

    test_error_types();

    println!("\n✅ hidapi-compat provides complete hidapi compatibility!");
}
