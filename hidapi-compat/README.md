# hidapi-compat

A hidapi-compatible interface for hidraw-rs, providing a drop-in replacement for the hidapi crate.

## Overview

This crate provides API compatibility with the popular `hidapi` crate while using `hidraw-rs` as the backend. This allows projects that depend on `hidapi` to switch to `hidraw-rs` without code changes, gaining benefits such as:

- **Musl compatibility** - Works with static linking
- **No buffer overflow issues** - Memory safe Rust implementation  
- **Better error handling** - More detailed error types
- **Minimal dependencies** - Only depends on libc through hidraw-rs
- **Better performance** - No C library overhead

## Usage

To use `hidapi-compat` as a drop-in replacement for `hidapi`, simply replace the dependency in your `Cargo.toml`:

```toml
# Old:
hidapi = "2.5.1"

# New:
hidapi = { package = "hidapi-compat", version = "0.1.0" }
```

No code changes are required! Your existing hidapi code will work as-is.

## Example

```rust
use hidapi::{HidApi, HidDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create API instance
    let api = HidApi::new()?;
    
    // List all devices
    for device in api.device_list() {
        println!("Device: {:04x}:{:04x}", 
            device.vendor_id(), 
            device.product_id()
        );
    }
    
    // Open a device
    let device = api.open(0x1234, 0x5678)?;
    
    // Write data
    let data = vec![0x00, 0x01, 0x02, 0x03];
    device.write(&data)?;
    
    // Read data
    let mut buf = vec![0u8; 64];
    let n = device.read_timeout(&mut buf, 1000)?;
    println!("Read {} bytes", n);
    
    Ok(())
}
```

## Supported Features

The following hidapi functionality is fully supported:

- `HidApi::new()` - Create new API instance
- `HidApi::refresh_devices()` - Refresh device list
- `HidApi::device_list()` - Get device iterator
- `HidApi::open()` - Open by VID/PID
- `HidApi::open_serial()` - Open by VID/PID/serial
- `HidApi::open_path()` - Open by device path
- `HidDevice::write()` - Write data
- `HidDevice::read()` - Read data
- `HidDevice::read_timeout()` - Read with timeout
- `HidDevice::send_feature_report()` - Send feature report
- `HidDevice::get_feature_report()` - Get feature report
- `HidDevice::set_blocking_mode()` - Set blocking/non-blocking
- All `DeviceInfo` accessors

## Limitations

Some hidapi features have limited support due to platform differences:

- `release_number`, `usage_page`, and `usage` in `DeviceInfo` are set to 0
- `get_indexed_string()` returns `None` (rarely used)
- `get_last_error()` returns `None` (hidraw-rs has better error reporting)

## Testing with rust-coldcard

This crate has been tested as a replacement for hidapi in the rust-coldcard project:

```toml
[dependencies]
# Replace hidapi with hidapi-compat
hidapi = { package = "hidapi-compat", path = "../hidraw-rs/hidapi-compat" }
```

All rust-coldcard functionality works without modification.

## License

This project is licensed under the same terms as hidraw-rs: MIT OR Apache-2.0