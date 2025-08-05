//! Coldcard hardware wallet support

mod constants;
mod protocol;

pub use constants::{COINKITE_VID, COLDCARD_PID};
pub use protocol::{ColdcardDevice, ColdcardProtocol};
