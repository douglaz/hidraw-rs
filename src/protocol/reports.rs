//! HID report structures and parsing

use crate::{Error, Result};

/// HID report types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    Input,
    Output,
    Feature,
}

/// HID report structure
#[derive(Debug, Clone)]
pub struct HidReport {
    pub report_id: u8,
    pub report_type: ReportType,
    pub data: Vec<u8>,
}

impl HidReport {
    /// Create a new HID report
    pub fn new(report_id: u8, report_type: ReportType, data: Vec<u8>) -> Self {
        Self {
            report_id,
            report_type,
            data,
        }
    }

    /// Create an input report
    pub fn input(report_id: u8, data: Vec<u8>) -> Self {
        Self::new(report_id, ReportType::Input, data)
    }

    /// Create an output report
    pub fn output(report_id: u8, data: Vec<u8>) -> Self {
        Self::new(report_id, ReportType::Output, data)
    }

    /// Create a feature report
    pub fn feature(report_id: u8, data: Vec<u8>) -> Self {
        Self::new(report_id, ReportType::Feature, data)
    }

    /// Convert report to bytes for transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.data.len() + 1);
        bytes.push(self.report_id);
        bytes.extend_from_slice(&self.data);
        bytes
    }

    /// Parse a report from bytes
    pub fn from_bytes(bytes: &[u8], report_type: ReportType) -> Result<Self> {
        if bytes.is_empty() {
            return Err(Error::InvalidData("Empty report data".to_string()));
        }

        Ok(Self {
            report_id: bytes[0],
            report_type,
            data: bytes[1..].to_vec(),
        })
    }
}
