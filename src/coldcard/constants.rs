//! Coldcard USB constants and protocol definitions

/// Coinkite vendor ID
pub const COINKITE_VID: u16 = 0xd13e;

/// Coldcard product ID
pub const COLDCARD_PID: u16 = 0xcc10;

/// Maximum message size for Coldcard
pub const MAX_MSG_SIZE: usize = 4096;

/// HID packet size for Coldcard
pub const PACKET_SIZE: usize = 64;

/// Coldcard commands
pub mod commands {
    /// Ping command
    pub const PING: &[u8; 4] = b"ping";
    
    /// Version command
    pub const VERSION: &[u8; 4] = b"vers";
    
    /// Reboot command
    pub const REBOOT: &[u8; 4] = b"rebo";
    
    /// Check status
    pub const STATUS: &[u8; 4] = b"stat";
    
    /// Get XPub
    pub const GET_XPUB: &[u8; 4] = b"xpub";
    
    /// Sign transaction
    pub const SIGN_TX: &[u8; 4] = b"stxn";
    
    /// Get address
    pub const GET_ADDR: &[u8; 4] = b"addr";
}