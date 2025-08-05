//! HidApi implementation compatible with hidapi

use crate::{DeviceInfo, DeviceInfoList, HidDevice, HidError, HidResult};
use std::ffi::CStr;

/// Main hidapi context
pub struct HidApi {
    devices: Vec<DeviceInfo>,
}

impl HidApi {
    /// Create a new hidapi instance
    ///
    /// This initializes the HID API and caches the device list.
    pub fn new() -> HidResult<Self> {
        let devices = Self::enumerate_devices()?;
        Ok(Self { devices })
    }

    /// Refresh the device list
    pub fn refresh_devices(&mut self) -> HidResult<()> {
        self.devices = Self::enumerate_devices()?;
        Ok(())
    }

    /// Get a list of all connected HID devices
    pub fn device_list(&self) -> DeviceInfoList {
        DeviceInfoList::new(self.devices.clone())
    }

    /// Get a slice of all connected devices (deprecated in hidapi)
    #[deprecated(note = "Use device_list() instead")]
    pub fn devices(&self) -> &[DeviceInfo] {
        &self.devices
    }

    /// Open a device by vendor ID and product ID
    pub fn open(&self, vendor_id: u16, product_id: u16) -> HidResult<HidDevice> {
        // Find the first matching device
        let device_info = self
            .devices
            .iter()
            .find(|d| d.vendor_id() == vendor_id && d.product_id() == product_id)
            .ok_or(HidError::OpenHidDeviceError)?;

        self.open_path(device_info.path())
    }

    /// Open a device by vendor ID, product ID, and serial number
    pub fn open_serial(
        &self,
        vendor_id: u16,
        product_id: u16,
        serial: &str,
    ) -> HidResult<HidDevice> {
        // Find the matching device with the specified serial
        let device_info = self
            .devices
            .iter()
            .find(|d| {
                d.vendor_id() == vendor_id
                    && d.product_id() == product_id
                    && d.serial_number() == Some(serial)
            })
            .ok_or(HidError::OpenHidDeviceError)?;

        self.open_path(device_info.path())
    }

    /// Open a device by its path
    pub fn open_path(&self, path: &CStr) -> HidResult<HidDevice> {
        let path_str = path.to_str().map_err(|_| HidError::HidApiError {
            message: "Invalid device path".to_string(),
        })?;

        let device =
            hidraw_rs::HidDevice::open_path(path_str).map_err(|_| HidError::OpenHidDeviceError)?;

        Ok(HidDevice::new(device))
    }

    /// Enumerate all HID devices
    fn enumerate_devices() -> HidResult<Vec<DeviceInfo>> {
        let hidraw_devices = hidraw_rs::enumerate().map_err(|_| HidError::InitializationError)?;

        let devices: Vec<DeviceInfo> = hidraw_devices
            .into_iter()
            .map(|d| DeviceInfo::from_hidraw(&d))
            .collect();

        Ok(devices)
    }
}

// Implement AsRef for compatibility
impl AsRef<HidApi> for HidApi {
    fn as_ref(&self) -> &HidApi {
        self
    }
}
