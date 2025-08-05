//! Rust HID library for Linux hidraw interface
//!
//! This library provides a Rust implementation for communicating with HID devices
//! on Linux through the hidraw kernel interface with minimal dependencies (only libc for system calls).
//!
//! # Features
//! - Rust implementation with minimal dependencies (only libc)
//! - Direct hidraw kernel interface
//! - Support for synchronous and async I/O
//! - Memory safe
//! - Works with musl static linking
//!
//! # Example
//! ```no_run
//! use hidraw_rs::prelude::*;
//! use std::time::Duration;
//!
//! fn main() -> Result<()> {
//!     // Find all HID devices
//!     let devices = enumerate()?;
//!     
//!     // Open a specific device
//!     if let Some(info) = devices.first() {
//!         let mut device = HidDevice::open(info)?;
//!         
//!         // Read with timeout
//!         let mut buf = vec![0u8; 64];
//!         let n = device.read_timeout(&mut buf, Duration::from_secs(1))?;
//!         println!("Read {} bytes", n);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod device;
pub mod error;
pub mod hidraw;
pub mod protocol;

#[cfg(feature = "async")]
pub mod async_io;

pub mod coldcard;


// Re-exports for convenience
pub use device::{DeviceInfo, HidDevice};
pub use error::{Error, Result};
pub use hidraw::enumerate;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{enumerate, find_devices};
    pub use crate::{DeviceInfo, HidDevice};
    pub use crate::{Error, Result};
}

/// Find devices matching vendor and product ID
pub fn find_devices(vendor_id: u16, product_id: u16) -> Result<Vec<DeviceInfo>> {
    Ok(enumerate()?
        .into_iter()
        .filter(|d| d.vendor_id == vendor_id && d.product_id == product_id)
        .collect())
}
