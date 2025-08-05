//! Device enumeration and discovery functionality

use crate::{DeviceInfo, Error, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Enumerate all HID devices on the system
pub fn enumerate() -> Result<Vec<DeviceInfo>> {
    let mut devices = Vec::new();

    // Check if /sys/class/hidraw exists
    let hidraw_class = Path::new("/sys/class/hidraw");
    if !hidraw_class.exists() {
        return Err(Error::NotSupported(
            "hidraw not available on this system".to_string(),
        ));
    }

    // Read /sys/class/hidraw/ directory
    for entry in fs::read_dir(hidraw_class)? {
        let entry = entry?;
        let name = entry.file_name();
        let device_path = PathBuf::from("/dev").join(&name);

        // Skip if the device file doesn't exist
        if !device_path.exists() {
            continue;
        }

        // Try to get device info
        match get_device_info(&device_path) {
            Ok(info) => devices.push(info),
            Err(_) => {
                // Skip devices we can't read info from
                continue;
            }
        }
    }

    Ok(devices)
}

/// Get device information from a hidraw device path
pub fn get_device_info(device_path: &Path) -> Result<DeviceInfo> {
    // Extract device name from path (e.g., "hidraw0" from "/dev/hidraw0")
    let device_name = device_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::InvalidPath("Invalid device path".to_string()))?;

    // Construct sysfs path
    let sysfs_base = PathBuf::from("/sys/class/hidraw").join(device_name);
    if !sysfs_base.exists() {
        return Err(Error::InvalidPath(format!(
            "sysfs path {} does not exist",
            sysfs_base.display()
        )));
    }

    // Follow the device symlink
    let device_sysfs = sysfs_base.join("device");

    // Find the USB device directory by walking up the hierarchy
    let usb_device_path = find_usb_device_path(&device_sysfs)?;

    // Read USB device attributes
    let vendor_id = read_hex_attr(&usb_device_path.join("idVendor"))?;
    let product_id = read_hex_attr(&usb_device_path.join("idProduct"))?;

    // Read string descriptors (may not exist)
    let manufacturer = read_string_attr(&usb_device_path.join("manufacturer")).ok();
    let product = read_string_attr(&usb_device_path.join("product")).ok();
    let serial = read_string_attr(&usb_device_path.join("serial")).ok();

    // Try to get interface number
    let interface_number = get_interface_number(&device_sysfs).unwrap_or(0);

    Ok(DeviceInfo {
        path: device_path.to_owned(),
        vendor_id,
        product_id,
        serial_number: serial,
        manufacturer,
        product,
        interface_number,
    })
}

/// Find the USB device path by walking up the sysfs hierarchy
fn find_usb_device_path(start_path: &Path) -> Result<PathBuf> {
    // Canonicalize the path to resolve symlinks and .. components
    let mut current = fs::canonicalize(start_path)
        .map_err(|e| Error::Parse(format!("Failed to canonicalize path: {e}")))?;

    // Walk up the directory tree looking for idVendor file
    for _ in 0..10 {
        // Limit depth to prevent infinite loops
        // Check if this directory has idVendor (indicating it's a USB device)
        if current.join("idVendor").exists() {
            return Ok(current);
        }

        // Go up one level
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    Err(Error::Parse(
        "Could not find USB device in sysfs".to_string(),
    ))
}

/// Get the interface number from the device path
fn get_interface_number(device_path: &Path) -> Result<i32> {
    // The interface number is often in the path like: .../1-1.4:1.0/...
    // Where the last number after the colon is the interface number

    if let Some(path_str) = device_path.to_str() {
        // Look for pattern like ":1.0" in the path
        for component in path_str.split('/') {
            if let Some(colon_pos) = component.rfind(':') {
                if let Some(dot_pos) = component[colon_pos + 1..].find('.') {
                    let interface_str = &component[colon_pos + 1..colon_pos + 1 + dot_pos];
                    if let Ok(num) = interface_str.parse::<i32>() {
                        return Ok(num);
                    }
                }
            }
        }
    }

    Err(Error::Parse(
        "Could not determine interface number".to_string(),
    ))
}

/// Read a hexadecimal value from a sysfs attribute file
fn read_hex_attr(path: &Path) -> Result<u16> {
    let content = fs::read_to_string(path)
        .map_err(|_| Error::Parse(format!("Could not read {}", path.display())))?;

    let trimmed = content.trim();
    u16::from_str_radix(trimmed, 16)
        .map_err(|_| Error::Parse(format!("Invalid hex value: {trimmed}")))
}

/// Read a string value from a sysfs attribute file
fn read_string_attr(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content.trim().to_string())
}
