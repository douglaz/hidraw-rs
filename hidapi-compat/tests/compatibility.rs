//! Tests for hidapi compatibility

use hidapi_compat::{HidApi, HidError};

#[test]
fn test_api_creation() {
    // This might fail if no HID devices are present, which is OK for the test
    match HidApi::new() {
        Ok(api) => {
            println!("HidApi created successfully");
            // Test device enumeration
            let devices: Vec<_> = api.device_list().collect();
            println!("Found {count} devices", count = devices.len());
        }
        Err(e) => {
            println!("Failed to create HidApi: {e:?}");
            // This is expected in CI environments without HID devices
        }
    }
}

#[test]
fn test_error_types() {
    // Test that our error types exist and can be created
    let _err1 = HidError::HidApiError {
        message: "test".to_string(),
    };
    let _err2 = HidError::OpenHidDeviceError;
    let _err3 = HidError::InvalidZeroSizeData;
}

#[test]
fn test_device_info() {
    // Test that we can create and use DeviceInfo
    match HidApi::new() {
        Ok(api) => {
            for device in api.device_list() {
                // Test all accessor methods
                let _path = device.path();
                let _vid = device.vendor_id();
                let _pid = device.product_id();
                let _serial = device.serial_number();
                let _manufacturer = device.manufacturer_string();
                let _product = device.product_string();
                let _usage_page = device.usage_page();
                let _usage = device.usage();
                let _interface = device.interface_number();

                println!(
                    "Device: VID={vid:04x} PID={pid:04x}",
                    vid = device.vendor_id(),
                    pid = device.product_id()
                );
            }
        }
        Err(_) => {
            // Expected in environments without HID devices
        }
    }
}
