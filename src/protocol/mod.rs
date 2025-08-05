//! HID protocol implementation

mod framing;
mod reports;

pub use framing::{frame_packets, unframe_packets};
pub use reports::{HidReport, ReportType};
