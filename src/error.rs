//! Error types for the hidraw-rs library

use std::io;
use thiserror::Error;

/// Result type alias for the library
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for HID operations
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error from system calls
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Device not found
    #[error("HID device not found")]
    DeviceNotFound,

    /// Invalid device path
    #[error("Invalid device path: {0}")]
    InvalidPath(String),

    /// Timeout during read operation
    #[error("Read timeout")]
    Timeout,

    /// Invalid data received
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Feature not supported
    #[error("Feature not supported: {0}")]
    NotSupported(String),

    /// Permission denied
    #[error("Permission denied accessing device")]
    PermissionDenied,

    /// Device disconnected
    #[error("Device disconnected")]
    Disconnected,

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Buffer too small
    #[error("Buffer too small: needed {needed} bytes, got {got} bytes")]
    BufferTooSmall { needed: usize, got: usize },

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// System call error
    #[error("System call failed: {0}")]
    SystemCall(String),

    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),
}

impl Error {
    /// Create an I/O error with custom message
    pub fn io_error(msg: &str) -> Self {
        Error::Io(io::Error::new(io::ErrorKind::Other, msg))
    }

    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self, Error::Timeout)
    }

    /// Check if this is a permission error
    pub fn is_permission_denied(&self) -> bool {
        match self {
            Error::PermissionDenied => true,
            Error::Io(e) => e.kind() == io::ErrorKind::PermissionDenied,
            _ => false,
        }
    }

    /// Check if device is disconnected
    pub fn is_disconnected(&self) -> bool {
        match self {
            Error::Disconnected => true,
            Error::Io(e) => matches!(e.kind(), 
                io::ErrorKind::UnexpectedEof | 
                io::ErrorKind::BrokenPipe |
                io::ErrorKind::NotConnected
            ),
            _ => false,
        }
    }
}

/// Convert from errno to Error
impl From<libc::c_int> for Error {
    fn from(errno: libc::c_int) -> Self {
        Error::Io(io::Error::from_raw_os_error(errno))
    }
}