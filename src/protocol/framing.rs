//! Message framing for multi-packet HID communication

use crate::{Error, Result};

/// Default HID packet size (64 bytes is common for USB HID)
#[allow(dead_code)]
pub const DEFAULT_PACKET_SIZE: usize = 64;

/// Frame data into HID packets
///
/// The first byte of each packet contains:
/// - Bits 0-5: Length of data in this packet (0-63)
/// - Bit 7: Last packet flag (1 if this is the last packet)
pub fn frame_packets(data: &[u8], packet_size: usize) -> Vec<Vec<u8>> {
    if data.is_empty() {
        return vec![vec![0x80; packet_size]]; // Single empty packet marked as last
    }

    let mut packets = Vec::new();
    let payload_size = packet_size - 1; // Reserve first byte for header

    for (i, chunk) in data.chunks(payload_size).enumerate() {
        let mut packet = vec![0u8; packet_size];

        // Set length in lower 6 bits
        packet[0] = chunk.len() as u8;

        // Set last packet flag if this is the last chunk
        if (i + 1) * payload_size >= data.len() {
            packet[0] |= 0x80;
        }

        // Copy data
        packet[1..1 + chunk.len()].copy_from_slice(chunk);

        packets.push(packet);
    }

    packets
}

/// Unframe HID packets back into continuous data
pub fn unframe_packets(packets: &[Vec<u8>]) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut found_last = false;

    for packet in packets {
        if packet.is_empty() {
            return Err(Error::InvalidData("Empty packet".to_string()));
        }

        let header = packet[0];
        let length = (header & 0x3F) as usize;
        let is_last = (header & 0x80) != 0;

        if length > packet.len() - 1 {
            return Err(Error::InvalidData(format!(
                "Packet length {} exceeds available data {}",
                length,
                packet.len() - 1
            )));
        }

        data.extend_from_slice(&packet[1..1 + length]);

        if is_last {
            found_last = true;
            break;
        }
    }

    if !found_last && !packets.is_empty() {
        return Err(Error::InvalidData(
            "No last packet marker found".to_string(),
        ));
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_single_packet() -> Result<()> {
        let data = vec![1, 2, 3, 4, 5];
        let packets = frame_packets(&data, 64);

        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0][0], 0x85); // Length 5 + last packet flag
        assert_eq!(&packets[0][1..6], &data[..]);

        Ok(())
    }

    #[test]
    fn test_frame_multiple_packets() -> Result<()> {
        let data = vec![0u8; 100];
        let packets = frame_packets(&data, 64);

        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0][0], 63); // Full packet, not last
        assert_eq!(packets[1][0], 0x80 | 37); // 37 bytes + last flag

        Ok(())
    }

    #[test]
    fn test_unframe_packets() -> Result<()> {
        let original = vec![1, 2, 3, 4, 5];
        let packets = frame_packets(&original, 64);
        let unframed = unframe_packets(&packets)?;

        assert_eq!(unframed, original);

        Ok(())
    }
}
