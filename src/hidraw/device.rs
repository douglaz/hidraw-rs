//! Low-level hidraw device operations

use crate::hidraw::{ioctl, sys};
use crate::{Error, Result};
use rustix::fd::AsFd;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::fs::FileTypeExt;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::{Path, PathBuf};

/// Low-level hidraw device handle
pub struct HidrawDevice {
    file: File,
    path: PathBuf,
    report_size: usize,
}

impl HidrawDevice {
    /// Open a hidraw device by path
    pub fn open(path: &Path) -> Result<Self> {
        // Check if path exists and is a character device
        let metadata = std::fs::metadata(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Error::DeviceNotFound,
            std::io::ErrorKind::PermissionDenied => Error::PermissionDenied,
            _ => Error::Io(e),
        })?;

        // On Linux, hidraw devices are character devices
        if !metadata.file_type().is_char_device() {
            return Err(Error::InvalidPath(format!(
                "{} is not a character device",
                path.display()
            )));
        }

        // Open the device file
        let file = File::options()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::PermissionDenied => Error::PermissionDenied,
                _ => Error::Io(e),
            })?;

        // Get report descriptor size via ioctl (using rustix)
        let report_size = ioctl::get_report_descriptor_size(&file)? as usize;

        Ok(Self {
            file,
            path: path.to_owned(),
            report_size,
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

    /// Read a HID report (blocking)
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.file.read(buf).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                Error::Disconnected
            } else {
                Error::Io(e)
            }
        })
    }

    /// Write a HID report
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        if data.is_empty() {
            return Err(Error::InvalidParameter(
                "Cannot write empty data".to_string(),
            ));
        }

        // Check if data exceeds typical HID report size
        if data.len() > 4096 {
            return Err(Error::InvalidParameter(format!(
                "Data too large: {} bytes (max 4096)",
                data.len()
            )));
        }

        self.file.write(data).map_err(|e| match e.kind() {
            std::io::ErrorKind::BrokenPipe => Error::Disconnected,
            std::io::ErrorKind::PermissionDenied => Error::PermissionDenied,
            std::io::ErrorKind::NotConnected => Error::Disconnected,
            _ => Error::Io(e),
        })
    }

    /// Get a feature report
    pub fn get_feature_report(&mut self, report_id: u8, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() {
            return Err(Error::InvalidParameter(
                "Buffer cannot be empty".to_string(),
            ));
        }

        // First byte must be the report ID
        buf[0] = report_id;

        let res = ioctl::ioctl_read_buf(&self.file, sys::hidiocgfeature(buf.len()), buf)?;
        Ok(res)
    }

    /// Send a feature report
    pub fn send_feature_report(&mut self, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Err(Error::InvalidParameter("Data cannot be empty".to_string()));
        }

        ioctl::ioctl_write_buf(&self.file, sys::hidiocsfeature(data.len()), data)?;
        Ok(())
    }

    /// Get device info via ioctl
    pub fn get_raw_info(&self) -> Result<sys::HidrawDevInfo> {
        // Using rustix for fixed-size struct
        ioctl::get_raw_info(&self.file)
    }

    /// Get device name
    pub fn get_raw_name(&self) -> Result<String> {
        // Using rustix for fixed 256-byte buffer
        ioctl::get_raw_name(&self.file)
    }

    /// Get physical device location
    pub fn get_raw_phys(&self) -> Result<String> {
        ioctl::get_raw_phys(&self.file)
    }

    /// Get unique device ID
    pub fn get_raw_uniq(&self) -> Result<String> {
        ioctl::get_raw_uniq(&self.file)
    }

    /// Get report descriptor
    pub fn get_report_descriptor(&self) -> Result<sys::HidrawReportDescriptor> {
        ioctl::get_report_descriptor(&self.file)
    }
}

impl AsRawFd for HidrawDevice {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

impl AsFd for HidrawDevice {
    fn as_fd(&self) -> rustix::fd::BorrowedFd<'_> {
        self.file.as_fd()
    }
}

impl std::fmt::Debug for HidrawDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HidrawDevice")
            .field("path", &self.path)
            .field("report_size", &self.report_size)
            .finish()
    }
}
