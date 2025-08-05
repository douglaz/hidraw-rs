//! Integration tests for hidraw-rs

use hidraw_rs::prelude::*;
use hidraw_rs::protocol::{frame_packets, unframe_packets};

#[cfg(feature = "async")]
use hidraw_rs::async_io::AsyncHidDevice;

#[test]
fn test_library_imports() {
    // Just verify the library compiles and exports work
    let _ = enumerate();
}

#[test]
fn test_error_types() {
    // Test error creation and matching
    let err = Error::DeviceNotFound;
    assert!(matches!(err, Error::DeviceNotFound));

    let err = Error::Timeout;
    assert!(err.is_timeout());

    let err = Error::PermissionDenied;
    assert!(err.is_permission_denied());
}

#[test]
fn test_packet_framing() -> Result<()> {
    // Test single packet
    let data = vec![1, 2, 3, 4, 5];
    let packets = frame_packets(&data, 64);
    assert_eq!(packets.len(), 1);

    // Test round trip
    let unframed = unframe_packets(&packets)?;
    assert_eq!(unframed, data);

    // Test multiple packets
    let large_data = vec![0xAA; 150];
    let packets = frame_packets(&large_data, 64);
    assert_eq!(packets.len(), 3); // 63 + 63 + 24 bytes

    let unframed = unframe_packets(&packets)?;
    assert_eq!(unframed, large_data);

    Ok(())
}

#[test]
fn test_empty_data_framing() -> Result<()> {
    let data = vec![];
    let packets = frame_packets(&data, 64);
    assert_eq!(packets.len(), 1);
    assert_eq!(packets[0][0], 0x80); // Empty packet marked as last

    let unframed = unframe_packets(&packets)?;
    assert_eq!(unframed, data);

    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_device_enumeration() {
    // Test that we can enumerate devices without errors
    let devices = enumerate();
    assert!(devices.is_ok());
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_timeout() {
    use std::time::Duration;

    // This test requires a real device, so we'll just test the API compiles
    // In a real test environment, you'd open an actual device
    if let Ok(devices) = enumerate() {
        if let Some(info) = devices.first() {
            // Try to open the device
            if let Ok(mut device) = AsyncHidDevice::open(info).await {
                let mut buf = vec![0u8; 64];

                // Test read with very short timeout
                let result = device
                    .read_timeout(&mut buf, Duration::from_millis(1))
                    .await;

                // We expect either success or timeout
                match result {
                    Ok(_) => {}               // Device responded very quickly
                    Err(Error::Timeout) => {} // Expected timeout
                    Err(e) => panic!("Unexpected error: {e:?}"),
                }
            }
        }
    }
}

#[cfg(feature = "async")]
#[test]
fn test_device_info_display() {
    let info = DeviceInfo {
        path: "/dev/hidraw0".into(),
        vendor_id: 0x1234,
        product_id: 0x5678,
        serial_number: Some("SN123456".to_string()),
        manufacturer: Some("Test Manufacturer".to_string()),
        product: Some("Test Device".to_string()),
        interface_number: 0,
    };

    assert_eq!(info.display_name(), "Test Device (1234:5678)");
    assert!(info.matches(0x1234, 0x5678));
    assert!(!info.matches(0x1234, 0x0000));
}

#[test]
fn test_error_edge_cases() -> Result<()> {
    // Test buffer size error
    let err = Error::BufferTooSmall {
        needed: 100,
        got: 50,
    };
    match err {
        Error::BufferTooSmall { needed, got } => {
            assert_eq!(needed, 100);
            assert_eq!(got, 50);
        }
        _ => panic!("Wrong error type"),
    }

    // Test disconnected detection
    let err = Error::Disconnected;
    assert!(err.is_disconnected());

    // Test IO error disconnected detection
    let io_err = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "EOF");
    let err = Error::Io(io_err);
    assert!(err.is_disconnected());

    Ok(())
}
