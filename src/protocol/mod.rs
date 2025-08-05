//! HID protocol implementation

mod reports;
mod framing;

pub use reports::{ReportType, HidReport};
pub use framing::{frame_packets, unframe_packets};