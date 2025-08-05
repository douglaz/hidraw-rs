//! HID device implementation compatible with hidapi

use crate::error::{HidError, HidResult};
use std::time::Duration;

/// HID device handle compatible with hidapi
pub struct HidDevice {
    inner: hidraw_rs::HidDevice,
    blocking: bool,
}

impl HidDevice {
    /// Create a new HidDevice from a hidraw-rs device
    pub(crate) fn new(device: hidraw_rs::HidDevice) -> Self {
        Self {
            inner: device,
            blocking: true, // hidapi defaults to blocking mode
        }
    }

    /// Write data to the device
    ///
    /// The first byte should be the report ID. For devices that don't use report IDs,
    /// set the first byte to 0.
    pub fn write(&mut self, data: &[u8]) -> HidResult<usize> {
        if data.is_empty() {
            return Err(HidError::InvalidZeroSizeData);
        }

        match self.inner.write(data) {
            Ok(n) => {
                if n == data.len() {
                    Ok(n)
                } else {
                    Err(HidError::IncompleteSendError {
                        sent: n,
                        all: data.len(),
                    })
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Read data from the device
    pub fn read(&mut self, buf: &mut [u8]) -> HidResult<usize> {
        if self.blocking {
            // In blocking mode, wait indefinitely
            self.inner.read(buf).map_err(Into::into)
        } else {
            // In non-blocking mode, return immediately
            match self.inner.read_timeout(buf, Duration::from_millis(0)) {
                Ok(n) => Ok(n),
                Err(hidraw_rs::Error::Timeout) => {
                    // hidapi returns 0 for non-blocking read with no data
                    Ok(0)
                }
                Err(e) => Err(e.into()),
            }
        }
    }

    /// Read data from the device with a timeout
    ///
    /// `timeout` is in milliseconds. 0 means non-blocking, -1 means blocking indefinitely.
    pub fn read_timeout(&mut self, buf: &mut [u8], timeout: i32) -> HidResult<usize> {
        if timeout < 0 {
            // Negative timeout means block indefinitely
            self.inner.read(buf).map_err(Into::into)
        } else {
            let duration = Duration::from_millis(timeout as u64);
            match self.inner.read_timeout(buf, duration) {
                Ok(n) => Ok(n),
                Err(hidraw_rs::Error::Timeout) => {
                    // hidapi returns 0 when timeout occurs with no data
                    // This is what rust-coldcard expects for resync operations
                    Ok(0)
                }
                Err(e) => Err(e.into()),
            }
        }
    }

    /// Send a feature report to the device
    pub fn send_feature_report(&mut self, data: &[u8]) -> HidResult<()> {
        if data.is_empty() {
            return Err(HidError::InvalidZeroSizeData);
        }

        self.inner
            .send_feature_report(data)
            .map_err(|_| HidError::SendFeatureReportError)?;
        Ok(())
    }

    /// Get a feature report from the device
    ///
    /// The first byte should be the report ID.
    pub fn get_feature_report(&mut self, buf: &mut [u8]) -> HidResult<usize> {
        if buf.is_empty() {
            return Err(HidError::InvalidZeroSizeData);
        }

        let report_id = buf[0];
        self.inner
            .get_feature_report(report_id, buf)
            .map_err(|_| HidError::GetFeatureReportError)
    }

    /// Set the device to blocking or non-blocking mode
    pub fn set_blocking_mode(&mut self, blocking: bool) -> HidResult<()> {
        self.blocking = blocking;
        Ok(())
    }

    /// Get the last error string (not implemented in hidraw-rs)
    pub fn get_last_error(&self) -> Option<String> {
        None
    }

    /// Get manufacturer string
    pub fn get_manufacturer_string(&self) -> HidResult<Option<String>> {
        Ok(self.inner.info().manufacturer.clone())
    }

    /// Get product string
    pub fn get_product_string(&self) -> HidResult<Option<String>> {
        Ok(self.inner.info().product.clone())
    }

    /// Get serial number string
    pub fn get_serial_number_string(&self) -> HidResult<Option<String>> {
        Ok(self.inner.info().serial_number.clone())
    }

    /// Get indexed string (not commonly used)
    pub fn get_indexed_string(&self, _index: i32) -> HidResult<Option<String>> {
        // hidraw-rs doesn't support indexed strings
        Ok(None)
    }
}
