//! Rustix-based ioctl implementations for fixed-size operations
//!
//! This module uses rustix 1.0's ioctl module for type-safe ioctl operations
//! where the size is known at compile time.

use crate::{Error, Result};
use rustix::fd::AsFd;
use rustix::ioctl::{self, Getter};

// Import the opcode constants
use crate::hidraw::sys::{HIDIOCGRAWINFO, HIDIOCGRAWNAME, HIDIOCGRDESCSIZE};

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

/// Get raw device name using rustix
/// This uses the libc version as rustix patterns are complex for variable-sized buffers
#[allow(dead_code)]
pub fn get_raw_name<Fd: AsFd>(fd: Fd, buf: &mut [u8]) -> Result<usize> {
    // For buffer operations, we keep using the libc version
    // rustix's patterns don't handle variable-sized buffers as cleanly
    crate::hidraw::ioctl::ioctl_read_buf(fd, HIDIOCGRAWNAME, buf)
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
    }
}
