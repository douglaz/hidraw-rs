//! Hybrid ioctl implementation using rustix for fixed-size and libc for runtime-sized operations
//!
//! This module provides a unified interface for ioctl operations, using:
//! - rustix: For fixed-size operations where the size is known at compile time
//! - libc: For runtime-sized operations where the size is computed dynamically

// Re-export rustix implementations for fixed-size operations
pub use ioctl_rustix::{
    get_raw_info, get_raw_name, get_raw_phys, get_raw_uniq, get_report_descriptor,
    get_report_descriptor_size,
};

// Re-export libc implementations for runtime-sized operations
pub use ioctl_libc::{ioctl_read_buf, ioctl_write_buf};

// Import modules
use super::{ioctl_libc, ioctl_rustix};
