# hidraw-rs

[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

A Rust HID library for Linux using the hidraw kernel interface. Provides direct hardware communication with minimal dependencies.

## Features

- 🦀 **Rust implementation** with minimal dependencies (libc + rustix for safer system calls)
- 🐧 **Direct hidraw kernel interface** - no udev or libusb needed
- 🔒 **Memory safe** - minimal unsafe code using rustix for safe system call wrappers
- 📦 **Static linking support** - works perfectly with musl for standalone binaries
- ⚡ **Async I/O support** - optional tokio integration for async operations
- ⏱️ **Comprehensive timeout support** - for both reads and writes
- 🔧 **Hardware wallet ready** - tested with Coldcard devices
- 🚀 **Zero-copy operations** where possible

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
hidraw-rs = "0.1.0"

# For async support
hidraw-rs = { version = "0.1.0", features = ["async"] }
```

List all HID devices:

```rust
use hidraw_rs::prelude::*;

fn main() -> Result<()> {
    let devices = enumerate()?;
    
    for device in devices {
        println!("Found device:");
        println!("  VID: {:04x}, PID: {:04x}", device.vendor_id, device.product_id);
        println!("  Product: {}", device.product.as_deref().unwrap_or("Unknown"));
        println!("  Path: {}", device.path.display());
    }
    
    Ok(())
}
```

## Examples

### Basic HID Communication

```rust
use hidraw_rs::prelude::*;
use std::time::Duration;

fn main() -> Result<()> {
    // Open device by vendor/product ID
    let device_info = find_devices(0x1234, 0x5678)?
        .into_iter()
        .next()
        .ok_or(Error::DeviceNotFound)?;
    
    let mut device = HidDevice::open(&device_info)?;
    
    // Write data
    let report = vec![0x00, 0x01, 0x02, 0x03];
    device.write(&report)?;
    
    // Read with timeout
    let mut buf = vec![0u8; 64];
    match device.read_timeout(&mut buf, Duration::from_millis(1000)) {
        Ok(size) => println!("Read {} bytes", size),
        Err(Error::Timeout) => println!("Read timed out"),
        Err(e) => return Err(e),
    }
    
    Ok(())
}
```

### Async Operations

```rust
use hidraw_rs::prelude::*;
use hidraw_rs::async_io::AsyncHidDevice;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Find device
    let device_info = find_devices(0x1234, 0x5678)?
        .into_iter()
        .next()
        .ok_or(Error::DeviceNotFound)?;
    
    // Open async device
    let mut device = AsyncHidDevice::open(&device_info).await?;
    
    // Async write
    device.write(&[0x00, 0x01, 0x02]).await?;
    
    // Async read with timeout
    let mut buf = vec![0u8; 64];
    let size = device.read_timeout(&mut buf, Duration::from_millis(100)).await?;
    println!("Read {} bytes asynchronously", size);
    
    Ok(())
}
```

### Hardware Wallet Communication (Coldcard)

```rust
use hidraw_rs::prelude::*;
use hidraw_rs::coldcard::{COINKITE_VID, COLDCARD_PID};

fn main() -> Result<()> {
    // Find Coldcard device
    let coldcard = find_devices(COINKITE_VID, COLDCARD_PID)?
        .into_iter()
        .next()
        .ok_or(Error::DeviceNotFound)?;
    
    let mut device = HidDevice::open(&coldcard)?;
    
    // Send ping command
    let ping_cmd = b"ping";
    let test_data = b"Hello Coldcard!";
    let mut packet = vec![0u8; 64];
    packet[0] = (ping_cmd.len() + test_data.len()) as u8 | 0x80;
    packet[1..5].copy_from_slice(ping_cmd);
    packet[5..5 + test_data.len()].copy_from_slice(test_data);
    
    device.write(&packet)?;
    
    // Read response
    let mut response = vec![0u8; 64];
    device.read_timeout(&mut response, Duration::from_secs(1))?;
    
    Ok(())
}
```

## Building

### Standard Build

```bash
cargo build --release
```

### Static Binary with musl

```bash
# Add musl target
rustup target add x86_64-unknown-linux-musl

# Build static binary
cargo build --release --target x86_64-unknown-linux-musl

# Verify static linking
ldd target/x86_64-unknown-linux-musl/release/examples/list_devices
# Should output: "statically linked"
```

### With Nix

```bash
# Enter development shell
nix develop

# Build with all dependencies
cargo build --release
```

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux x86_64 | ✅ Fully supported | Primary platform |
| Linux ARM | ✅ Supported | Tested on Raspberry Pi |
| Linux musl | ✅ Fully supported | Static linking works perfectly |
| macOS | ❌ Not supported | Planned for future release |
| Windows | ❌ Not supported | Planned for future release |

## API Documentation

### Core Types

- `DeviceInfo` - Information about a HID device (VID, PID, path, etc.)
- `HidDevice` - High-level HID device handle for synchronous I/O
- `HidrawDevice` - Low-level hidraw device for direct kernel access
- `AsyncHidDevice` - Async version of HidDevice (requires `async` feature)

### Main Functions

- `enumerate()` - List all HID devices on the system
- `find_devices(vid, pid)` - Find devices by vendor/product ID
- `HidDevice::open(info)` - Open a device from DeviceInfo
- `HidDevice::open_path(path)` - Open a device by path directly

### Error Handling

All operations return `Result<T, Error>` with comprehensive error types:

- `Error::DeviceNotFound` - No matching device found
- `Error::PermissionDenied` - Insufficient permissions (try sudo)
- `Error::Timeout` - Operation timed out
- `Error::Io(io::Error)` - Underlying I/O error

## Performance

- **Zero allocations** in hot paths
- **Direct kernel interface** - no intermediate libraries
- **Efficient polling** for timeouts using poll() syscall
- **Async support** via tokio for concurrent operations

## Safety

This library prioritizes safety:

- **Minimal unsafe code**: Uses `rustix` crate for safe wrappers around system calls
- **Safe abstractions**: Poll operations use rustix's safe `poll()` wrapper
- **Safe file descriptor handling**: File descriptor duplication uses rustix's `dup()` 
- **Reduced attack surface**: Unsafe code is limited to essential ioctl operations
- **Type safety**: Uses Rust's type system and rustix's `AsFd` trait for I/O safety
- Buffer sizes are validated
- Integer overflows in timeout calculations are prevented
- Connection state is properly tracked

## Running Examples

The repository includes several examples:

```bash
# List all HID devices
cargo run --example list_devices

# Basic HID operations demo
cargo run --example basic_hid

# Test with Coldcard wallet
cargo run --example coldcard_ping

# Async operations demo (requires async feature)
cargo run --example async_hid --features async

# Note: Some examples may require sudo for device access
sudo cargo run --example coldcard_ping
```

## Troubleshooting

### Permission Denied

If you get permission errors, you have several options:

1. Run with sudo (for testing only)
2. Add udev rules for your devices:

```bash
# Create udev rule
echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1234", ATTRS{idProduct}=="5678", MODE="0666"' | \
    sudo tee /etc/udev/rules.d/99-hidraw.rules

# Reload rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

3. Add your user to the appropriate group

### Device Not Found

- Ensure the device is connected
- Check `lsusb` to verify the device appears
- Verify the VID/PID are correct
- Some devices may require initialization

### Timeout Issues

- Increase timeout duration
- Check if device requires specific report formats
- Verify device is responsive using other tools

## Comparison with hidapi

| Feature | hidraw-rs | hidapi |
|---------|-----------|---------|
| Dependencies | Minimal (libc only) | libusb/libudev |
| Memory Safety | ✅ Rust guarantees | ⚠️ C library |
| Buffer Overflows | ✅ Impossible | ⚠️ Possible |
| Static Linking | ✅ Works with musl | ❌ Complex |
| Async Support | ✅ Native | ❌ None |
| API Design | ✅ Idiomatic Rust | ⚠️ C-style |
| Platform Support | 🐧 Linux only | 🌍 Cross-platform |

## hidapi Compatibility

This project includes `hidapi-compat`, a drop-in replacement for the `hidapi` crate. This allows existing projects using `hidapi` to switch to `hidraw-rs` without any code changes.

### Using hidapi-compat

To replace `hidapi` in your project, simply change your `Cargo.toml`:

```toml
# Replace this:
hidapi = "2.5.1"

# With this:
hidapi = { package = "hidapi-compat", path = "path/to/hidraw-rs/hidapi-compat" }

# Or from git:
hidapi = { package = "hidapi-compat", git = "https://github.com/yourusername/hidraw-rs" }
```

No code changes required! Your existing hidapi code will work as-is.

### Example: rust-coldcard Migration

The `hidapi-compat` crate has been tested with `rust-coldcard`:

```toml
[dependencies]
# Simply replace the hidapi dependency
hidapi = { package = "hidapi-compat", path = "../hidraw-rs/hidapi-compat" }
```

All functionality works without modification, while gaining the benefits of:
- Musl static linking support
- Better error handling
- No buffer overflow risks
- Improved performance

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/hidraw-rs
cd hidraw-rs

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run --example list_devices
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests (requires connected HID device)
sudo cargo test -- --ignored

# All tests with async features
cargo test --all-features
```

## License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Linux kernel hidraw documentation
- Rust embedded community for inspiration
- Coinkite for Coldcard hardware wallet testing

## Changelog

### v0.1.0 (2025-08-04)

- Initial release
- Full hidraw support for Linux
- Sync and async APIs
- Coldcard hardware wallet support
- Static linking with musl
- Comprehensive timeout handling