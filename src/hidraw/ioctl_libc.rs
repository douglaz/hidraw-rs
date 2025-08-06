//! Safe wrappers for ioctl system calls using libc for runtime-sized operations

use crate::{Error, Result};
use rustix::fd::{AsFd, AsRawFd};

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
