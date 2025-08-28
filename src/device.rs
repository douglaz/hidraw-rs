//! High-level HID device interface

use crate::hidraw::HidrawDevice;
use crate::{Error, Result};
use std::path::PathBuf;
use std::time::Duration;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Information about a HID device
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DeviceInfo {
    /// Device path (e.g., /dev/hidraw0)
    pub path: PathBuf,
    /// USB vendor ID
    pub vendor_id: u16,
    /// USB product ID
    pub product_id: u16,
    /// Serial number (if available)
    pub serial_number: Option<String>,
    /// Manufacturer name (if available)
    pub manufacturer: Option<String>,
    /// Product name (if available)
    pub product: Option<String>,
    /// Interface number
    pub interface_number: i32,
}

impl DeviceInfo {
    /// Check if this device matches the given vendor and product IDs
    pub fn matches(&self, vendor_id: u16, product_id: u16) -> bool {
        self.vendor_id == vendor_id && self.product_id == product_id
    }

    /// Get a display name for the device
    pub fn display_name(&self) -> String {
        if let Some(product) = &self.product {
            format!(
                "{} ({:04x}:{:04x})",
                product, self.vendor_id, self.product_id
            )
        } else {
            format!("HID Device {:04x}:{:04x}", self.vendor_id, self.product_id)
        }
    }
}

/// High-level HID device interface
pub struct HidDevice {
    raw: HidrawDevice,
    info: DeviceInfo,
    read_timeout: Option<Duration>,
}

impl HidDevice {
    /// Open a HID device from DeviceInfo
    pub fn open(info: &DeviceInfo) -> Result<Self> {
        let raw = HidrawDevice::open(&info.path)?;
        Ok(Self {
            raw,
            info: info.clone(),
            read_timeout: None,
        })
    }

    /// Open a HID device by path
    pub fn open_path(path: &str) -> Result<Self> {
        let path = PathBuf::from(path);
        let raw = HidrawDevice::open(&path)?;

        // Try to get device info from sysfs
        let info = crate::hidraw::get_device_info(&path)?;

        Ok(Self {
            raw,
            info,
            read_timeout: None,
        })
    }

    /// Open the first device matching vendor and product ID
    pub fn open_first(vendor_id: u16, product_id: u16) -> Result<Self> {
        let devices = crate::find_devices(vendor_id, product_id)?;
        let device_info = devices.into_iter().next().ok_or(Error::DeviceNotFound)?;

        Self::open(&device_info)
    }

    /// Get device information
    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }

    /// Set read timeout
    pub fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.read_timeout = timeout;
    }

    /// Read data from the device
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if let Some(timeout) = self.read_timeout {
            self.read_timeout_impl(buf, timeout)
        } else {
            self.raw.read(buf)
        }
    }

    /// Read with explicit timeout
    pub fn read_timeout(&mut self, buf: &mut [u8], timeout: Duration) -> Result<usize> {
        self.read_timeout_impl(buf, timeout)
    }

    /// Write data to the device
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.raw.write(data)
    }

    /// Write with explicit timeout
    pub fn write_timeout(&mut self, data: &[u8], timeout: Duration) -> Result<usize> {
        self.write_timeout_impl(data, timeout)
    }

    /// Get a feature report
    pub fn get_feature_report(&mut self, report_id: u8, buf: &mut [u8]) -> Result<usize> {
        self.raw.get_feature_report(report_id, buf)
    }

    /// Send a feature report
    pub fn send_feature_report(&mut self, data: &[u8]) -> Result<()> {
        self.raw.send_feature_report(data)
    }

    /// Get the raw file descriptor (for advanced usage)
    pub fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        use std::os::unix::io::AsRawFd;
        self.raw.as_raw_fd()
    }

    /// Internal implementation of read with timeout
    fn read_timeout_impl(&mut self, buf: &mut [u8], timeout: Duration) -> Result<usize> {
        use rustix::event::{PollFd, PollFlags, poll};

        if buf.is_empty() {
            return Err(Error::InvalidParameter(
                "Buffer cannot be empty".to_string(),
            ));
        }

        // Convert timeout to Timespec for rustix 1.0
        let timeout_spec = rustix::time::Timespec {
            tv_sec: timeout.as_secs() as i64,
            tv_nsec: timeout.subsec_nanos() as i64,
        };

        // Use rustix's safe poll wrapper
        let mut fds = [PollFd::new(&self.raw, PollFlags::IN)];

        let n = poll(&mut fds, Some(&timeout_spec)).map_err(|e| Error::Io(e.into()))?;

        if n == 0 {
            return Err(Error::Timeout);
        }

        // Check for error conditions
        let revents = fds[0].revents();
        if revents.contains(PollFlags::ERR) {
            return Err(Error::io_error("Poll error on device"));
        }
        if revents.contains(PollFlags::HUP) {
            return Err(Error::Disconnected);
        }

        self.raw.read(buf)
    }

    /// Internal implementation of write with timeout
    fn write_timeout_impl(&mut self, data: &[u8], timeout: Duration) -> Result<usize> {
        use rustix::event::{PollFd, PollFlags, poll};

        // Convert timeout to Timespec for rustix 1.0
        let timeout_spec = rustix::time::Timespec {
            tv_sec: timeout.as_secs() as i64,
            tv_nsec: timeout.subsec_nanos() as i64,
        };

        // Use rustix's safe poll wrapper
        let mut fds = [PollFd::new(&self.raw, PollFlags::OUT)];

        let n = poll(&mut fds, Some(&timeout_spec)).map_err(|e| Error::Io(e.into()))?;

        if n == 0 {
            return Err(Error::Timeout);
        }

        // Check for error conditions
        let revents = fds[0].revents();
        if revents.contains(PollFlags::ERR) {
            return Err(Error::io_error("Poll error on device"));
        }
        if revents.contains(PollFlags::HUP) {
            return Err(Error::Disconnected);
        }

        self.raw.write(data)
    }

    /// Get the physical location of the device (e.g., USB port path)
    ///
    /// This returns a string describing the physical path to the device,
    /// such as "usb-0000:00:14.0-1/input0".
    pub fn get_physical_info(&self) -> Result<String> {
        self.raw.get_raw_phys()
    }

    /// Get the unique ID of the device
    ///
    /// This returns a device-specific unique identifier if available.
    /// Not all devices provide a unique ID.
    pub fn get_unique_id(&self) -> Result<String> {
        self.raw.get_raw_uniq()
    }

    /// Get the HID report descriptor
    ///
    /// The report descriptor defines the format and meaning of data
    /// transferred between the device and host.
    pub fn get_report_descriptor(&self) -> Result<ReportDescriptor> {
        let raw_desc = self.raw.get_report_descriptor()?;
        Ok(ReportDescriptor {
            size: raw_desc.size as usize,
            data: raw_desc.value.to_vec(),
        })
    }
}

/// HID Report Descriptor
///
/// Contains the binary report descriptor data that defines the format
/// of HID reports for this device.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ReportDescriptor {
    /// Size of the descriptor in bytes
    pub size: usize,
    /// Raw descriptor data
    pub data: Vec<u8>,
}

impl ReportDescriptor {
    /// Get a slice of the valid descriptor data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.size.min(self.data.len())]
    }

    /// Check if the descriptor is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl std::fmt::Debug for HidDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HidDevice")
            .field("info", &self.info)
            .field("read_timeout", &self.read_timeout)
            .finish()
    }
}
