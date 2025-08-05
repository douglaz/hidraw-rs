//! Async I/O support for HID devices
//!
//! This module provides async versions of HID device operations using tokio.

use crate::hidraw::{sys, HidrawDevice};
use crate::{DeviceInfo, Error, Result};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Async version of HidrawDevice
pub struct AsyncHidrawDevice {
    file: File,
    path: PathBuf,
    report_size: usize,
}

impl AsyncHidrawDevice {
    /// Open a hidraw device by path
    pub async fn open(path: &Path) -> Result<Self> {
        // First open synchronously to perform all the checks and ioctls
        let sync_device = HidrawDevice::open(path)?;

        // Extract the file descriptor and convert to tokio File
        let raw_fd = sync_device.as_raw_fd();
        let file = unsafe {
            // Duplicate the fd to avoid closing it when sync_device is dropped
            let new_fd = libc::dup(raw_fd);
            if new_fd < 0 {
                return Err(Error::Io(std::io::Error::last_os_error()));
            }
            File::from_raw_fd(new_fd)
        };

        Ok(Self {
            file,
            path: path.to_owned(),
            report_size: sync_device.report_size(),
        })
    }

    /// Get the device path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the report descriptor size
    pub fn report_size(&self) -> usize {
        self.report_size
    }

    /// Read a HID report asynchronously
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.file.read(buf).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                Error::Disconnected
            } else {
                Error::Io(e)
            }
        })
    }

    /// Write a HID report asynchronously
    pub async fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.file.write(data).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::BrokenPipe {
                Error::Disconnected
            } else {
                Error::Io(e)
            }
        })
    }

    /// Read with timeout
    pub async fn read_timeout(&mut self, buf: &mut [u8], timeout: Duration) -> Result<usize> {
        tokio::time::timeout(timeout, self.read(buf))
            .await
            .map_err(|_| Error::Timeout)?
    }

    /// Write with timeout
    pub async fn write_timeout(&mut self, data: &[u8], timeout: Duration) -> Result<usize> {
        tokio::time::timeout(timeout, self.write(data))
            .await
            .map_err(|_| Error::Timeout)?
    }

    /// Get a feature report (synchronous - ioctl doesn't have async variant)
    pub fn get_feature_report(&self, report_id: u8, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() {
            return Err(Error::InvalidParameter(
                "Buffer cannot be empty".to_string(),
            ));
        }

        // First byte must be the report ID
        buf[0] = report_id;

        unsafe {
            let res = crate::hidraw::ioctl::ioctl_read_buf(
                self.file.as_raw_fd(),
                sys::hidiocgfeature(buf.len()),
                buf,
            )?;
            Ok(res)
        }
    }

    /// Send a feature report (synchronous - ioctl doesn't have async variant)
    pub fn send_feature_report(&self, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Err(Error::InvalidParameter("Data cannot be empty".to_string()));
        }

        unsafe {
            crate::hidraw::ioctl::ioctl_write_buf(
                self.file.as_raw_fd(),
                sys::hidiocsfeature(data.len()),
                data,
            )?;
        }
        Ok(())
    }
}

impl AsRawFd for AsyncHidrawDevice {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.file.as_raw_fd()
    }
}

/// Async version of HidDevice
pub struct AsyncHidDevice {
    raw: AsyncHidrawDevice,
    info: DeviceInfo,
}

impl AsyncHidDevice {
    /// Open a HID device from DeviceInfo
    pub async fn open(info: &DeviceInfo) -> Result<Self> {
        let raw = AsyncHidrawDevice::open(&info.path).await?;
        Ok(Self {
            raw,
            info: info.clone(),
        })
    }

    /// Open a HID device by path
    pub async fn open_path(path: &str) -> Result<Self> {
        let path = PathBuf::from(path);
        let raw = AsyncHidrawDevice::open(&path).await?;

        // Try to get device info from sysfs
        let info = crate::hidraw::get_device_info(&path)?;

        Ok(Self { raw, info })
    }

    /// Open the first device matching vendor and product ID
    pub async fn open_first(vendor_id: u16, product_id: u16) -> Result<Self> {
        let devices = crate::find_devices(vendor_id, product_id)?;
        let device_info = devices.into_iter().next().ok_or(Error::DeviceNotFound)?;

        Self::open(&device_info).await
    }

    /// Get device information
    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }

    /// Read data from the device
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.raw.read(buf).await
    }

    /// Read with timeout
    pub async fn read_timeout(&mut self, buf: &mut [u8], timeout: Duration) -> Result<usize> {
        self.raw.read_timeout(buf, timeout).await
    }

    /// Write data to the device
    pub async fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.raw.write(data).await
    }

    /// Write with timeout
    pub async fn write_timeout(&mut self, data: &[u8], timeout: Duration) -> Result<usize> {
        self.raw.write_timeout(data, timeout).await
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
        self.raw.as_raw_fd()
    }
}

impl std::fmt::Debug for AsyncHidDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncHidDevice")
            .field("info", &self.info)
            .finish()
    }
}
