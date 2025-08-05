//! Rustix-based ioctl implementations for fixed-size operations
//!
//! This module uses rustix 1.0's ioctl module for type-safe ioctl operations
//! where the size is known at compile time.

use crate::{Error, Result};
use rustix::fd::AsFd;
use rustix::ioctl::{self, Getter};

// Import the opcode constants and types
use crate::hidraw::sys::{
    HidrawReportDescriptor, HIDIOCGRAWINFO, HIDIOCGRAWNAME, HIDIOCGRAWPHYS, HIDIOCGRAWUNIQ,
    HIDIOCGRDESC, HIDIOCGRDESCSIZE,
};

/// Get report descriptor size using rustix
#[allow(dead_code)]
pub fn get_report_descriptor_size<Fd: AsFd>(fd: Fd) -> Result<u32> {
    // SAFETY: HIDIOCGRDESCSIZE is a valid opcode for getting a u32
    let getter = unsafe { Getter::<{ HIDIOCGRDESCSIZE }, u32>::new() };
    unsafe { ioctl::ioctl(&fd, getter).map_err(|e| Error::Io(e.into())) }
}

/// Get raw device info using rustix
#[allow(dead_code)]
pub fn get_raw_info<Fd: AsFd>(fd: Fd) -> Result<crate::hidraw::sys::HidrawDevInfo> {
    // SAFETY: HIDIOCGRAWINFO is a valid opcode for getting HidrawDevInfo
    let getter = unsafe { Getter::<{ HIDIOCGRAWINFO }, crate::hidraw::sys::HidrawDevInfo>::new() };
    unsafe { ioctl::ioctl(&fd, getter).map_err(|e| Error::Io(e.into())) }
}

/// Get raw device name using rustix (fixed 256-byte buffer)
pub fn get_raw_name<Fd: AsFd>(fd: Fd) -> Result<String> {
    // SAFETY: HIDIOCGRAWNAME is a valid opcode for getting a 256-byte buffer
    let getter = unsafe { Getter::<{ HIDIOCGRAWNAME }, [u8; 256]>::new() };
    let buf = unsafe { ioctl::ioctl(&fd, getter).map_err(|e| Error::Io(e.into()))? };

    // Find null terminator and convert to string
    let len = buf.iter().position(|&b| b == 0).unwrap_or(256);
    String::from_utf8(buf[..len].to_vec())
        .map_err(|_| Error::Parse("Invalid UTF-8 in device name".to_string()))
}

/// Get raw physical info using rustix
#[allow(dead_code)]
pub fn get_raw_phys<Fd: AsFd>(fd: Fd) -> Result<String> {
    // SAFETY: HIDIOCGRAWPHYS is a valid opcode for getting a 256-byte buffer
    let getter = unsafe { Getter::<{ HIDIOCGRAWPHYS }, [u8; 256]>::new() };
    let buf = unsafe { ioctl::ioctl(&fd, getter).map_err(|e| Error::Io(e.into()))? };

    // Find null terminator and convert to string
    let len = buf.iter().position(|&b| b == 0).unwrap_or(256);
    String::from_utf8(buf[..len].to_vec())
        .map_err(|_| Error::Parse("Invalid UTF-8 in physical info".to_string()))
}

/// Get raw unique ID using rustix
#[allow(dead_code)]
pub fn get_raw_uniq<Fd: AsFd>(fd: Fd) -> Result<String> {
    // SAFETY: HIDIOCGRAWUNIQ is a valid opcode for getting a 256-byte buffer
    let getter = unsafe { Getter::<{ HIDIOCGRAWUNIQ }, [u8; 256]>::new() };
    let buf = unsafe { ioctl::ioctl(&fd, getter).map_err(|e| Error::Io(e.into()))? };

    // Find null terminator and convert to string
    let len = buf.iter().position(|&b| b == 0).unwrap_or(256);
    String::from_utf8(buf[..len].to_vec())
        .map_err(|_| Error::Parse("Invalid UTF-8 in unique ID".to_string()))
}

/// Get report descriptor using rustix
#[allow(dead_code)]
pub fn get_report_descriptor<Fd: AsFd>(fd: Fd) -> Result<HidrawReportDescriptor> {
    // SAFETY: HIDIOCGRDESC is a valid opcode for getting HidrawReportDescriptor
    let getter = unsafe { Getter::<{ HIDIOCGRDESC }, HidrawReportDescriptor>::new() };
    unsafe { ioctl::ioctl(&fd, getter).map_err(|e| Error::Io(e.into())) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_opcodes() {
        // Just verify the constants are accessible
        let _ = HIDIOCGRDESCSIZE;
        let _ = HIDIOCGRAWINFO;
        let _ = HIDIOCGRAWNAME;
        let _ = HIDIOCGRAWPHYS;
        let _ = HIDIOCGRAWUNIQ;
        let _ = HIDIOCGRDESC;
    }
}
