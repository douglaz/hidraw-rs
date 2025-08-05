//! Coldcard hardware wallet support

mod protocol;
mod constants;

pub use protocol::{ColdcardDevice, ColdcardProtocol};
pub use constants::{COINKITE_VID, COLDCARD_PID};