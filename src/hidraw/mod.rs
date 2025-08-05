//! Linux hidraw backend implementation

mod device;
mod enumerate;
pub(crate) mod ioctl;
pub(crate) mod ioctl_libc;
pub(crate) mod ioctl_rustix;
pub(crate) mod sys;

pub use device::HidrawDevice;
pub use enumerate::{enumerate, get_device_info};

// Re-export system constants that might be useful
pub use sys::{HIDIOCGRDESC, HIDIOCGRDESCSIZE};
