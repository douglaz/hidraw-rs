//! hidapi-compatible interface for hidraw-rs
//!
//! This crate provides a drop-in replacement for the hidapi crate,
//! using hidraw-rs as the backend. It maintains API compatibility
//! with hidapi while providing the benefits of hidraw-rs such as
//! musl compatibility and better error handling.

pub mod api;
pub mod device;
pub mod device_info;
pub mod error;

pub use api::HidApi;
pub use device::HidDevice;
pub use device_info::{DeviceInfo, DeviceInfoList};
pub use error::{HidError, HidResult};

// Re-export as hidapi does
pub type Result<T> = std::result::Result<T, HidError>;
