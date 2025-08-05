//! Error types compatible with hidapi

use thiserror::Error;

pub type HidResult<T> = std::result::Result<T, HidError>;

#[derive(Error, Debug)]
pub enum HidError {
    #[error("hidapi error: {message}")]
    HidApiError { message: String },

    #[error("hidapi error")]
    HidApiErrorEmpty,

    #[error("Failed to initialize hidapi")]
    InitializationError,

    #[error("Failed to open HID device")]
    OpenHidDeviceError,

    #[error("Invalid zero-sized data")]
    InvalidZeroSizeData,

    #[error("Incomplete send: {sent} sent, {all} expected")]
    IncompleteSendError { sent: usize, all: usize },

    #[error("Failed to set blocking mode")]
    SetBlockingModeError,

    #[error("Failed to open HID device with device info")]
    OpenHidDeviceWithDeviceInfoError { device_info: Box<crate::DeviceInfo> },

    #[error("Failed to send feature report")]
    SendFeatureReportError,

    #[error("Failed to get feature report")]
    GetFeatureReportError,

    #[error("Failed to convert from wide char")]
    FromWideCharError { wide_char: Vec<u16> },

    #[error("IO error: {error}")]
    IoError {
        #[from]
        error: std::io::Error,
    },

    #[error("Failed to convert C string")]
    CStringError(#[from] std::ffi::NulError),
}

impl HidError {
    /// Get the last error that happened, if any.
    pub fn get_last_error(_device: &crate::HidDevice) -> Option<String> {
        // hidraw-rs doesn't have a last error concept like hidapi
        // Return None to indicate no error available
        None
    }
}

// Convert from hidraw-rs errors
impl From<hidraw_rs::Error> for HidError {
    fn from(err: hidraw_rs::Error) -> Self {
        match err {
            hidraw_rs::Error::Io(io_err) => HidError::IoError { error: io_err },
            hidraw_rs::Error::DeviceNotFound => HidError::OpenHidDeviceError,
            hidraw_rs::Error::PermissionDenied => HidError::HidApiError {
                message: "Permission denied".to_string(),
            },
            hidraw_rs::Error::InvalidParameter(msg) => HidError::HidApiError { message: msg },
            hidraw_rs::Error::Timeout => HidError::HidApiError {
                message: "Operation timed out".to_string(),
            },
            hidraw_rs::Error::NotSupported(msg) => HidError::HidApiError { message: msg },
            hidraw_rs::Error::BufferTooSmall { needed, got } => HidError::HidApiError {
                message: format!("Buffer too small: needed {needed} bytes, got {got} bytes"),
            },
            hidraw_rs::Error::InvalidPath(msg) => HidError::HidApiError { message: msg },
            hidraw_rs::Error::InvalidData(msg) => HidError::HidApiError { message: msg },
            hidraw_rs::Error::Disconnected => HidError::HidApiError {
                message: "Device disconnected".to_string(),
            },
            hidraw_rs::Error::Protocol(msg) => HidError::HidApiError { message: msg },
            hidraw_rs::Error::SystemCall(msg) => HidError::HidApiError { message: msg },
            hidraw_rs::Error::Parse(msg) => HidError::HidApiError { message: msg },
        }
    }
}
