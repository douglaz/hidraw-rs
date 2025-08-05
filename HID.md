# hidraw-rs Implementation Plan

## Overview

This document outlines a plan to create a Rust library for HID (Human Interface Device) communication that directly interfaces with Linux's hidraw kernel API. This will eliminate the dependency on the problematic hidapi C library and avoid buffer overflow issues currently preventing Coldcard integration.

## Motivation

- The current hidapi C library has a buffer overflow when communicating with Coldcard devices
- Musl static linking is problematic with existing HID libraries due to libudev dependencies
- No existing Rust HID library provides the functionality needed for hardware wallet communication
- A Rust solution with minimal dependencies would provide memory safety and better error handling

## Architecture

### Project Structure

```
hidraw-rs/
├── Cargo.toml
├── src/
│   ├── lib.rs              // Public API and re-exports
│   ├── error.rs            // Error types and handling
│   ├── hidraw/
│   │   ├── mod.rs          // Linux hidraw backend
│   │   ├── device.rs       // Low-level device operations
│   │   ├── ioctl.rs        // ioctl system call bindings
│   │   ├── sys.rs          // System-level constants and structures
│   │   └── enumerate.rs    // Device discovery and enumeration
│   ├── protocol/
│   │   ├── mod.rs          // HID protocol implementation
│   │   ├── reports.rs      // HID report parsing and construction
│   │   ├── descriptors.rs  // HID descriptor parsing
│   │   └── framing.rs      // Multi-packet message handling
│   ├── device.rs           // High-level HID device interface
│   ├── async_io.rs         // Async I/O support (optional feature)
│   └── coldcard/
│       ├── mod.rs          // Coldcard-specific protocol
│       ├── protocol.rs     // Coldcard command/response handling
│       └── constants.rs    // Coldcard USB IDs and protocol constants
├── examples/
│   ├── list_devices.rs     // Enumerate all HID devices
│   ├── basic_hid.rs        // Basic HID communication
│   └── coldcard_ping.rs    // Coldcard-specific example
└── tests/
    ├── integration.rs      // Integration tests
    └── mock_device.rs      // Mock HID device for testing
```

## Core Components

### 1. Low-Level hidraw Interface

```rust
use std::fs::File;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::{Path, PathBuf};

/// Low-level hidraw device handle
pub struct HidrawDevice {
    file: File,
    path: PathBuf,
    report_size: usize,
}

impl HidrawDevice {
    /// Open a hidraw device by path
    pub fn open(path: &Path) -> Result<Self, Error> {
        let file = File::options()
            .read(true)
            .write(true)
            .open(path)?;
        
        // Get report descriptor size via ioctl
        let report_size = unsafe {
            let mut size: i32 = 0;
            ioctl_read(file.as_raw_fd(), HIDIOCGRDESCSIZE, &mut size)?;
            size as usize
        };
        
        Ok(Self {
            file,
            path: path.to_owned(),
            report_size,
        })
    }
    
    /// Read a HID report (blocking)
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        use std::io::Read;
        Ok(self.file.read(buf)?)
    }
    
    /// Write a HID report
    pub fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        use std::io::Write;
        Ok(self.file.write(data)?)
    }
    
    /// Get a feature report
    pub fn get_feature_report(&mut self, report_id: u8, buf: &mut [u8]) -> Result<usize, Error> {
        buf[0] = report_id;
        unsafe {
            let res = ioctl_read(
                self.file.as_raw_fd(),
                HIDIOCGFEATURE(buf.len()),
                buf.as_mut_ptr(),
            )?;
            Ok(res as usize)
        }
    }
    
    /// Send a feature report
    pub fn send_feature_report(&mut self, data: &[u8]) -> Result<(), Error> {
        unsafe {
            ioctl_write(
                self.file.as_raw_fd(),
                HIDIOCSFEATURE(data.len()),
                data.as_ptr(),
            )?;
        }
        Ok(())
    }
}
```

### 2. Device Enumeration

```rust
use std::fs;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub path: PathBuf,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub interface_number: i32,
}

/// Enumerate all HID devices on the system
pub fn enumerate() -> Result<Vec<DeviceInfo>, Error> {
    let mut devices = Vec::new();
    
    // Read /sys/class/hidraw/ directory
    for entry in fs::read_dir("/sys/class/hidraw")? {
        let entry = entry?;
        let name = entry.file_name();
        let device_path = PathBuf::from("/dev").join(&name);
        
        // Read device info from sysfs
        let sysfs_path = entry.path().join("device");
        
        // Read vendor ID
        let vendor_id = read_hex_attr(&sysfs_path.join("../idVendor"))?;
        let product_id = read_hex_attr(&sysfs_path.join("../idProduct"))?;
        
        // Read string descriptors
        let manufacturer = read_string_attr(&sysfs_path.join("../manufacturer")).ok();
        let product = read_string_attr(&sysfs_path.join("../product")).ok();
        let serial = read_string_attr(&sysfs_path.join("../serial")).ok();
        
        devices.push(DeviceInfo {
            path: device_path,
            vendor_id,
            product_id,
            serial_number: serial,
            manufacturer,
            product,
            interface_number: 0, // TODO: Parse from sysfs
        });
    }
    
    Ok(devices)
}

/// Find devices matching vendor and product ID
pub fn find_devices(vendor_id: u16, product_id: u16) -> Result<Vec<DeviceInfo>, Error> {
    Ok(enumerate()?
        .into_iter()
        .filter(|d| d.vendor_id == vendor_id && d.product_id == product_id)
        .collect())
}
```

### 3. HID Protocol Layer

```rust
/// High-level HID device interface
pub struct HidDevice {
    raw: HidrawDevice,
    info: DeviceInfo,
    read_timeout: Option<Duration>,
}

impl HidDevice {
    /// Open a HID device from DeviceInfo
    pub fn open(info: &DeviceInfo) -> Result<Self, Error> {
        let raw = HidrawDevice::open(&info.path)?;
        Ok(Self {
            raw,
            info: info.clone(),
            read_timeout: None,
        })
    }
    
    /// Set read timeout
    pub fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.read_timeout = timeout;
    }
    
    /// Read with timeout support
    pub fn read_timeout(&mut self, buf: &mut [u8], timeout: Duration) -> Result<usize, Error> {
        use std::os::unix::io::AsRawFd;
        
        // Use poll() for timeout
        let mut pollfd = libc::pollfd {
            fd: self.raw.file.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        
        let timeout_ms = timeout.as_millis() as i32;
        let ret = unsafe { libc::poll(&mut pollfd, 1, timeout_ms) };
        
        if ret < 0 {
            return Err(Error::Io(std::io::Error::last_os_error()));
        } else if ret == 0 {
            return Err(Error::Timeout);
        }
        
        self.raw.read(buf)
    }
}
```

### 4. Coldcard-Specific Implementation

```rust
pub mod coldcard {
    use super::*;
    
    pub const COINKITE_VID: u16 = 0xd13e;
    pub const COLDCARD_PID: u16 = 0xcc10;
    
    /// Coldcard communication protocol
    pub struct ColdcardProtocol {
        device: HidDevice,
    }
    
    impl ColdcardProtocol {
        pub fn open() -> Result<Self, Error> {
            let devices = find_devices(COINKITE_VID, COLDCARD_PID)?;
            let device_info = devices.into_iter().next()
                .ok_or(Error::DeviceNotFound)?;
            
            let device = HidDevice::open(&device_info)?;
            Ok(Self { device })
        }
        
        /// Send a command and receive response
        pub fn send_command(&mut self, cmd: &[u8; 4], data: Option<&[u8]>) -> Result<Vec<u8>, Error> {
            // Build request
            let mut request = cmd.to_vec();
            if let Some(data) = data {
                request.extend_from_slice(data);
            }
            
            // Send in 64-byte packets
            for chunk in request.chunks(63) {
                let mut packet = vec![0u8; 64];
                packet[0] = chunk.len() as u8;
                if chunk.len() == request.len() || request.len() <= 63 {
                    packet[0] |= 0x80; // Last packet flag
                }
                packet[1..=chunk.len()].copy_from_slice(chunk);
                
                self.device.raw.write(&packet)?;
            }
            
            // Read response
            let mut response = Vec::new();
            loop {
                let mut packet = vec![0u8; 64];
                self.device.read_timeout(&mut packet, Duration::from_secs(5))?;
                
                let len = (packet[0] & 0x3F) as usize;
                let is_last = packet[0] & 0x80 != 0;
                
                response.extend_from_slice(&packet[1..=len]);
                
                if is_last {
                    break;
                }
            }
            
            Ok(response)
        }
    }
}
```

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [ ] Set up project structure and dependencies
- [ ] Implement basic ioctl bindings using libc
- [ ] Create low-level hidraw device operations
- [ ] Test with simple HID devices (keyboard/mouse)

### Phase 2: Device Discovery (Week 3)
- [ ] Implement sysfs parsing for device enumeration
- [ ] Extract vendor/product IDs and descriptors
- [ ] Create device filtering and search functions
- [ ] Test device discovery with various HID devices

### Phase 3: HID Protocol (Week 4-5)
- [ ] Implement HID report handling
- [ ] Add timeout support using poll()
- [ ] Implement feature report get/set
- [ ] Create high-level device interface

### Phase 4: Coldcard Integration (Week 6-7)
- [ ] Implement Coldcard packet framing protocol
- [ ] Handle multi-packet messages
- [ ] Add Coldcard-specific commands
- [ ] Test with actual Coldcard device

### Phase 5: Polish and Testing (Week 8)
- [ ] Add comprehensive error handling
- [ ] Write unit and integration tests
- [ ] Create documentation and examples
- [ ] Performance optimization

## Technical Challenges and Solutions

### 1. ioctl Implementation

```rust
// Define ioctl numbers
const IOC_NRBITS: u32 = 8;
const IOC_TYPEBITS: u32 = 8;
const IOC_SIZEBITS: u32 = 14;
const IOC_DIRBITS: u32 = 2;

const IOC_NRSHIFT: u32 = 0;
const IOC_TYPESHIFT: u32 = IOC_NRSHIFT + IOC_NRBITS;
const IOC_SIZESHIFT: u32 = IOC_TYPESHIFT + IOC_TYPEBITS;
const IOC_DIRSHIFT: u32 = IOC_SIZESHIFT + IOC_SIZEBITS;

const IOC_READ: u32 = 2;
const IOC_WRITE: u32 = 1;

const fn _IOC(dir: u32, type_: u32, nr: u32, size: u32) -> u32 {
    (dir << IOC_DIRSHIFT) |
    (type_ << IOC_TYPESHIFT) |
    (nr << IOC_NRSHIFT) |
    (size << IOC_SIZESHIFT)
}

const fn _IOR(type_: u32, nr: u32, size: u32) -> u32 {
    _IOC(IOC_READ, type_, nr, size)
}

// HID ioctl constants
const HIDIOCGRDESCSIZE: u32 = _IOR(b'H' as u32, 0x01, 4);
const HIDIOCGRDESC: u32 = _IOR(b'H' as u32, 0x02, 4);

// System call wrapper
unsafe fn ioctl_read<T>(fd: RawFd, request: u32, arg: *mut T) -> Result<i32, Error> {
    let ret = libc::ioctl(fd, request as libc::c_ulong, arg);
    if ret < 0 {
        Err(Error::Io(std::io::Error::last_os_error()))
    } else {
        Ok(ret)
    }
}
```

### 2. Async I/O Support

```rust
#[cfg(feature = "async")]
pub mod async_io {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::fs::File;
    
    pub struct AsyncHidDevice {
        file: File,
        info: DeviceInfo,
    }
    
    impl AsyncHidDevice {
        pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
            Ok(self.file.read(buf).await?)
        }
        
        pub async fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
            Ok(self.file.write(data).await?)
        }
    }
}
```

### 3. Musl Compatibility

- Minimal external dependencies (only libc for system calls)
- Uses std library and libc for system calls
- All string parsing done in pure Rust
- File I/O through std::fs

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_enumeration() {
        let devices = enumerate().unwrap();
        // Should find at least some HID devices on most systems
        assert!(!devices.is_empty());
    }
    
    #[test]
    fn test_packet_framing() {
        let data = vec![0u8; 100];
        let packets = frame_packets(&data);
        assert_eq!(packets.len(), 2); // Should split into 2 packets
        assert_eq!(packets[0][0] & 0x80, 0); // First packet, not last
        assert_eq!(packets[1][0] & 0x80, 0x80); // Last packet
    }
}
```

### Integration Tests
- Test with real HID devices
- Mock Coldcard device for CI testing
- Stress test concurrent operations
- Test error recovery scenarios

## API Examples

### Basic Usage
```rust
use hidraw_rs::prelude::*;

// Find and open a device
let device = HidDevice::open_path("/dev/hidraw0")?;

// Or by vendor/product ID
let device = HidDevice::open(0xd13e, 0xcc10)?;

// Read with timeout
let mut buf = vec![0u8; 64];
let n = device.read_timeout(&mut buf, Duration::from_secs(1))?;

// Write data
device.write(&[0x00, 0x01, 0x02])?;
```

### Coldcard Specific
```rust
use hidraw_rs::coldcard::ColdcardDevice;

// Open Coldcard
let mut coldcard = ColdcardDevice::open()?;

// Send ping command
let response = coldcard.ping(b"Hello")?;

// Get version
let version = coldcard.get_version()?;
println!("Coldcard version: {}", version);
```

## Benefits

1. **Minimal Dependencies** - Only libc for system calls, works with musl
2. **Memory Safe** - No buffer overflows possible
3. **Modern API** - Uses Result types and iterators
4. **Async Support** - Optional tokio integration
5. **Well Tested** - Comprehensive test suite
6. **Cross Platform** - Designed for future Windows/macOS support

## Future Enhancements

- Windows support using Win32 HID API
- macOS support using IOKit
- HID report descriptor parsing
- USB hotplug support via udev monitoring
- WebHID browser support via WASM

## Dependencies

### Required
- `std` library
- `libc` - For system calls

### Optional
- `tokio` - For async I/O support
- `tracing` - For debug logging
- `serde` - For device info serialization

## License

This library will be dual-licensed under MIT/Apache-2.0, consistent with the Rust ecosystem.