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

        // Get report descriptor size via ioctl
        let report_size = ioctl::ioctl_read_int(&file, sys::HIDIOCGRDESCSIZE)? as usize;

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
        let mut info = sys::HidrawDevInfo {
            bustype: 0,
            vendor: 0,
            product: 0,
        };

        ioctl::ioctl_read(&self.file, sys::HIDIOCGRAWINFO, &mut info)?;

        Ok(info)
    }

    /// Get device name
    pub fn get_raw_name(&self) -> Result<String> {
        let mut buf = vec![0u8; 256];

        let len = ioctl::ioctl_read_buf(&self.file, sys::HIDIOCGRAWNAME, &mut buf)?;

        // Truncate at null terminator or actual length
        if let Some(null_pos) = buf.iter().position(|&b| b == 0) {
            buf.truncate(null_pos);
        } else {
            buf.truncate(len);
        }

        String::from_utf8(buf).map_err(|_| Error::Parse("Invalid UTF-8 in device name".to_string()))
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
