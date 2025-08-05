//! Device information structures compatible with hidapi

use std::ffi::{CStr, CString};

/// Device information structure that matches hidapi's API
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    path: CString,
    vendor_id: u16,
    product_id: u16,
    serial_number: Option<String>,
    release_number: u16,
    manufacturer_string: Option<String>,
    product_string: Option<String>,
    usage_page: u16,
    usage: u16,
    interface_number: i32,
}

impl DeviceInfo {
    /// Create a new DeviceInfo from hidraw-rs DeviceInfo
    pub(crate) fn from_hidraw(info: &hidraw_rs::DeviceInfo) -> Self {
        Self {
            path: CString::new(info.path.to_string_lossy().as_bytes()).unwrap_or_default(),
            vendor_id: info.vendor_id,
            product_id: info.product_id,
            serial_number: info.serial_number.clone(),
            // hidraw-rs doesn't provide release_number, usage_page, or usage
            // Set to defaults that match typical HID behavior
            release_number: 0,
            manufacturer_string: info.manufacturer.clone(),
            product_string: info.product.clone(),
            usage_page: 0,
            usage: 0,
            interface_number: info.interface_number,
        }
    }

    /// Get the device path
    pub fn path(&self) -> &CStr {
        &self.path
    }

    /// Get the vendor ID
    pub fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    /// Get the product ID
    pub fn product_id(&self) -> u16 {
        self.product_id
    }

    /// Get the serial number
    pub fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }

    /// Get the release number
    pub fn release_number(&self) -> u16 {
        self.release_number
    }

    /// Get the manufacturer string
    pub fn manufacturer_string(&self) -> Option<&str> {
        self.manufacturer_string.as_deref()
    }

    /// Get the product string
    pub fn product_string(&self) -> Option<&str> {
        self.product_string.as_deref()
    }

    /// Get the usage page
    pub fn usage_page(&self) -> u16 {
        self.usage_page
    }

    /// Get the usage
    pub fn usage(&self) -> u16 {
        self.usage
    }

    /// Get the interface number
    pub fn interface_number(&self) -> i32 {
        self.interface_number
    }

    /// Open this device
    pub fn open_device(&self, api: &crate::HidApi) -> crate::Result<crate::HidDevice> {
        api.open_path(&self.path)
    }
}

/// Iterator over device information, compatible with hidapi
pub struct DeviceInfoList {
    devices: Vec<DeviceInfo>,
    index: usize,
}

impl DeviceInfoList {
    pub(crate) fn new(devices: Vec<DeviceInfo>) -> Self {
        Self { devices, index: 0 }
    }
}

impl Iterator for DeviceInfoList {
    type Item = DeviceInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.devices.len() {
            let device = self.devices[self.index].clone();
            self.index += 1;
            Some(device)
        } else {
            None
        }
    }
}
