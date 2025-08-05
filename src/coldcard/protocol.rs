//! Coldcard communication protocol implementation

use super::constants::*;
use crate::protocol::frame_packets;
use crate::{Error, HidDevice, Result};
use std::time::Duration;

/// Coldcard device handle
pub struct ColdcardDevice {
    device: HidDevice,
}

impl ColdcardDevice {
    /// Open the first available Coldcard device
    pub fn open() -> Result<Self> {
        let device = HidDevice::open_first(COINKITE_VID, COLDCARD_PID)?;
        Ok(Self { device })
    }

    /// Open a specific Coldcard device
    pub fn open_path(path: &str) -> Result<Self> {
        let device = HidDevice::open_path(path)?;

        // Verify it's actually a Coldcard
        let info = device.info();
        if info.vendor_id != COINKITE_VID || info.product_id != COLDCARD_PID {
            return Err(Error::InvalidParameter(format!(
                "Device at {} is not a Coldcard (VID: {:04x}, PID: {:04x})",
                path, info.vendor_id, info.product_id
            )));
        }

        Ok(Self { device })
    }

    /// Get device info
    pub fn info(&self) -> &crate::DeviceInfo {
        self.device.info()
    }

    /// Execute a ping command
    pub fn ping(&mut self, msg: &[u8]) -> Result<Vec<u8>> {
        if msg.len() > 256 {
            return Err(Error::InvalidParameter(
                "Ping message too long (max 256 bytes)".to_string(),
            ));
        }

        let mut protocol = ColdcardProtocol::new(&mut self.device);
        protocol.send_command(commands::PING, Some(msg))
    }

    /// Get Coldcard version information
    pub fn get_version(&mut self) -> Result<String> {
        let mut protocol = ColdcardProtocol::new(&mut self.device);
        let response = protocol.send_command(commands::VERSION, None)?;

        String::from_utf8(response)
            .map_err(|_| Error::InvalidData("Invalid UTF-8 in version response".to_string()))
    }

    /// Get Coldcard status
    pub fn get_status(&mut self) -> Result<Vec<u8>> {
        let mut protocol = ColdcardProtocol::new(&mut self.device);
        protocol.send_command(commands::STATUS, None)
    }

    /// Reboot the Coldcard
    pub fn reboot(&mut self) -> Result<()> {
        let mut protocol = ColdcardProtocol::new(&mut self.device);
        protocol.send_command(commands::REBOOT, None)?;
        Ok(())
    }
}

/// Low-level Coldcard protocol handler
pub struct ColdcardProtocol<'a> {
    device: &'a mut HidDevice,
}

impl<'a> ColdcardProtocol<'a> {
    /// Create a new protocol handler
    pub fn new(device: &'a mut HidDevice) -> Self {
        Self { device }
    }

    /// Send a command and receive response
    pub fn send_command(&mut self, cmd: &[u8; 4], data: Option<&[u8]>) -> Result<Vec<u8>> {
        // Build request
        let mut request = cmd.to_vec();
        if let Some(data) = data {
            if request.len() + data.len() > MAX_MSG_SIZE {
                return Err(Error::InvalidParameter(format!(
                    "Message too large: {} bytes (max {})",
                    request.len() + data.len(),
                    MAX_MSG_SIZE
                )));
            }
            request.extend_from_slice(data);
        }

        // Frame into packets
        let packets = frame_packets(&request, PACKET_SIZE);

        // Send all packets
        for packet in packets {
            self.device.write(&packet)?;
        }

        // Read response packets
        let mut response_packets = Vec::new();
        let mut response_complete = false;

        // Set a reasonable timeout for reading
        let timeout = Duration::from_secs(5);

        while !response_complete {
            let mut packet = vec![0u8; PACKET_SIZE];
            let n = self.device.read_timeout(&mut packet, timeout)?;

            if n == 0 {
                return Err(Error::Disconnected);
            }

            // Check if this is the last packet
            if packet[0] & 0x80 != 0 {
                response_complete = true;
            }

            response_packets.push(packet);

            // Safety check to prevent infinite loops
            if response_packets.len() > MAX_MSG_SIZE / PACKET_SIZE {
                return Err(Error::Protocol(
                    "Response too large or missing end marker".to_string(),
                ));
            }
        }

        // Unframe the response
        crate::protocol::unframe_packets(&response_packets)
    }
}

impl std::fmt::Debug for ColdcardDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColdcardDevice")
            .field("info", self.device.info())
            .finish()
    }
}
