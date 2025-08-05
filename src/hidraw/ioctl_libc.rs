//! Safe wrappers for ioctl system calls using rustix

use crate::{Error, Result};
use rustix::fd::{AsFd, AsRawFd};

/// Perform an ioctl read operation for getting an integer
#[allow(dead_code)]
pub fn ioctl_read_int<Fd: AsFd>(fd: Fd, request: u32) -> Result<i32> {
    // For custom ioctls, we still need to use unsafe libc directly
    // as rustix doesn't provide a way to create custom Getter with runtime request values
    let fd_raw = fd.as_fd().as_raw_fd();
    let mut value: i32 = 0;

    #[cfg(target_env = "musl")]
    let ret = unsafe { libc::ioctl(fd_raw, request as libc::c_int, &mut value) };
    #[cfg(not(target_env = "musl"))]
    let ret = unsafe { libc::ioctl(fd_raw, request as libc::c_ulong, &mut value) };

    if ret < 0 {
        Err(Error::Io(std::io::Error::last_os_error()))
    } else {
        Ok(value)
    }
}

/// Read a buffer via ioctl
pub fn ioctl_read_buf<Fd: AsFd>(fd: Fd, request: u32, buf: &mut [u8]) -> Result<usize> {
    // For buffer operations, we need to use the raw ioctl with proper handling
    let fd_raw = fd.as_fd().as_raw_fd();

    #[cfg(target_env = "musl")]
    let ret = unsafe { libc::ioctl(fd_raw, request as libc::c_int, buf.as_mut_ptr()) };
    #[cfg(not(target_env = "musl"))]
    let ret = unsafe { libc::ioctl(fd_raw, request as libc::c_ulong, buf.as_mut_ptr()) };

    if ret < 0 {
        Err(Error::Io(std::io::Error::last_os_error()))
    } else {
        Ok(ret as usize)
    }
}

/// Write a buffer via ioctl
pub fn ioctl_write_buf<Fd: AsFd>(fd: Fd, request: u32, buf: &[u8]) -> Result<usize> {
    // For buffer operations, we need to use the raw ioctl with proper handling
    let fd_raw = fd.as_fd().as_raw_fd();

    #[cfg(target_env = "musl")]
    let ret = unsafe { libc::ioctl(fd_raw, request as libc::c_int, buf.as_ptr()) };
    #[cfg(not(target_env = "musl"))]
    let ret = unsafe { libc::ioctl(fd_raw, request as libc::c_ulong, buf.as_ptr()) };

    if ret < 0 {
        Err(Error::Io(std::io::Error::last_os_error()))
    } else {
        Ok(ret as usize)
    }
}

/// Perform an ioctl read operation for structured data
#[allow(dead_code)]
pub fn ioctl_read<Fd: AsFd, T>(fd: Fd, request: u32, arg: &mut T) -> Result<i32> {
    let fd_raw = fd.as_fd().as_raw_fd();

    #[cfg(target_env = "musl")]
    let ret = unsafe { libc::ioctl(fd_raw, request as libc::c_int, arg as *mut T) };
    #[cfg(not(target_env = "musl"))]
    let ret = unsafe { libc::ioctl(fd_raw, request as libc::c_ulong, arg as *mut T) };

    if ret < 0 {
        Err(Error::Io(std::io::Error::last_os_error()))
    } else {
        Ok(ret)
    }
}
