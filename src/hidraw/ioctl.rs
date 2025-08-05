//! Safe wrappers for ioctl system calls

use crate::{Error, Result};
use std::os::unix::io::RawFd;

/// Perform an ioctl read operation
pub unsafe fn ioctl_read<T>(fd: RawFd, request: u32, arg: *mut T) -> Result<i32> {
    #[cfg(target_env = "musl")]
    let ret = libc::ioctl(fd, request as libc::c_int, arg);
    #[cfg(not(target_env = "musl"))]
    let ret = libc::ioctl(fd, request as libc::c_ulong, arg);
    
    if ret < 0 {
        Err(Error::Io(std::io::Error::last_os_error()))
    } else {
        Ok(ret)
    }
}

/// Perform an ioctl write operation
pub unsafe fn ioctl_write<T>(fd: RawFd, request: u32, arg: *const T) -> Result<i32> {
    #[cfg(target_env = "musl")]
    let ret = libc::ioctl(fd, request as libc::c_int, arg);
    #[cfg(not(target_env = "musl"))]
    let ret = libc::ioctl(fd, request as libc::c_ulong, arg);
    
    if ret < 0 {
        Err(Error::Io(std::io::Error::last_os_error()))
    } else {
        Ok(ret)
    }
}

/// Read an integer value via ioctl
pub unsafe fn ioctl_read_int(fd: RawFd, request: u32) -> Result<i32> {
    let mut value: i32 = 0;
    ioctl_read(fd, request, &mut value)?;
    Ok(value)
}

/// Read a buffer via ioctl
pub unsafe fn ioctl_read_buf(fd: RawFd, request: u32, buf: &mut [u8]) -> Result<usize> {
    #[cfg(target_env = "musl")]
    let ret = libc::ioctl(fd, request as libc::c_int, buf.as_mut_ptr());
    #[cfg(not(target_env = "musl"))]
    let ret = libc::ioctl(fd, request as libc::c_ulong, buf.as_mut_ptr());
    
    if ret < 0 {
        Err(Error::Io(std::io::Error::last_os_error()))
    } else {
        Ok(ret as usize)
    }
}

/// Write a buffer via ioctl
pub unsafe fn ioctl_write_buf(fd: RawFd, request: u32, buf: &[u8]) -> Result<usize> {
    #[cfg(target_env = "musl")]
    let ret = libc::ioctl(fd, request as libc::c_int, buf.as_ptr());
    #[cfg(not(target_env = "musl"))]
    let ret = libc::ioctl(fd, request as libc::c_ulong, buf.as_ptr());
    
    if ret < 0 {
        Err(Error::Io(std::io::Error::last_os_error()))
    } else {
        Ok(ret as usize)
    }
}