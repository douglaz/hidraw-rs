# Migration Guide: hidapi to hidapi-compat

This guide helps you migrate from the `hidapi` crate to `hidapi-compat`, which uses `hidraw-rs` as its backend.

## Quick Migration (Drop-in Replacement)

The easiest way to migrate is to use `hidapi-compat` as a drop-in replacement:

### Step 1: Update Cargo.toml

Replace your hidapi dependency:

```toml
# OLD:
[dependencies]
hidapi = "2.5.1"

# NEW - Option 1: From local path
[dependencies]
hidapi = { package = "hidapi-compat", path = "path/to/hidraw-rs/hidapi-compat" }

# NEW - Option 2: From git
[dependencies]
hidapi = { package = "hidapi-compat", git = "https://github.com/yourusername/hidraw-rs" }
```

### Step 2: Build and Test

That's it! No code changes required. Your existing code will work as-is:

```bash
cargo build
cargo test
```

## Feature Comparison

### Fully Supported Features

✅ **API Methods**
- `HidApi::new()`
- `HidApi::refresh_devices()`
- `HidApi::device_list()`
- `HidApi::devices()` (deprecated)
- `HidApi::open(vendor_id, product_id)`
- `HidApi::open_serial(vendor_id, product_id, serial)`
- `HidApi::open_path(path)`

✅ **Device Operations**
- `HidDevice::write(data)`
- `HidDevice::read(buffer)`
- `HidDevice::read_timeout(buffer, timeout_ms)`
- `HidDevice::send_feature_report(data)`
- `HidDevice::get_feature_report(buffer)`
- `HidDevice::set_blocking_mode(blocking)`

✅ **Device Information**
- `DeviceInfo::path()`
- `DeviceInfo::vendor_id()`
- `DeviceInfo::product_id()`
- `DeviceInfo::serial_number()`
- `DeviceInfo::manufacturer_string()`
- `DeviceInfo::product_string()`
- `DeviceInfo::interface_number()`

### Limited Support

⚠️ **These features have limited support due to platform differences:**

- `DeviceInfo::release_number()` - Always returns 0
- `DeviceInfo::usage_page()` - Always returns 0
- `DeviceInfo::usage()` - Always returns 0
- `HidDevice::get_indexed_string()` - Always returns None
- `HidDevice::get_last_error()` - Always returns None

These limitations rarely affect normal usage.

## Benefits of Migration

### 1. **Musl Compatibility**
```bash
# Works perfectly with musl static linking
cargo build --target x86_64-unknown-linux-musl
```

### 2. **Better Error Handling**
```rust
// hidapi-compat provides more detailed error information
match device.write(&data) {
    Err(HidError::Timeout) => println!("Operation timed out"),
    Err(HidError::Disconnected) => println!("Device disconnected"),
    Err(e) => println!("Other error: {}", e),
    Ok(n) => println!("Wrote {} bytes", n),
}
```

### 3. **No Buffer Overflow Risk**
The Rust implementation guarantees memory safety.

### 4. **Smaller Binary Size**
No C library dependencies means smaller binaries.

## Common Use Cases

### Coldcard Wallet (rust-coldcard)

```toml
[dependencies]
# Just replace the hidapi dependency
hidapi = { package = "hidapi-compat", path = "../hidraw-rs/hidapi-compat" }
# All other dependencies remain the same
aes = "0.8.3"
base58 = "0.2.0"
# ... etc
```

### Generic HID Device

```rust
// No code changes needed!
use hidapi::{HidApi, HidDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = HidApi::new()?;
    
    for device in api.device_list() {
        println!("Device: {:04x}:{:04x}", 
            device.vendor_id(), 
            device.product_id()
        );
    }
    
    Ok(())
}
```

## Troubleshooting

### Permission Errors

If you get permission errors, ensure your user has access to hidraw devices:

```bash
# Add your user to the plugdev group
sudo usermod -a -G plugdev $USER

# Or create a udev rule
echo 'KERNEL=="hidraw*", MODE="0666"' | sudo tee /etc/udev/rules.d/99-hidraw.rules
sudo udevadm control --reload-rules
```

### Missing Devices

hidapi-compat only supports Linux. On other platforms, you'll need to use the original hidapi.

### Build Errors

Make sure you have the required dependencies:

```bash
# Debian/Ubuntu
sudo apt-get install pkg-config

# Fedora
sudo dnf install pkg-config
```

## Advanced Migration

If you want to migrate to the native hidraw-rs API for better performance and features:

```rust
// OLD (hidapi)
use hidapi::{HidApi, HidDevice};
let api = HidApi::new()?;
let device = api.open(0x1234, 0x5678)?;

// NEW (hidraw-rs native)
use hidraw_rs::prelude::*;
let device = HidDevice::open_first(0x1234, 0x5678)?;
```

The native API provides additional features like async support:

```rust
use hidraw_rs::async_io::AsyncHidDevice;

let device = AsyncHidDevice::open_first(0x1234, 0x5678).await?;
let data = device.read_timeout(&mut buf, Duration::from_secs(1)).await?;
```

## Support

If you encounter any issues during migration:

1. Check the [examples](examples/) directory
2. Review the [test suite](hidapi-compat/tests/)
3. Open an issue on GitHub

## Conclusion

Migrating from hidapi to hidapi-compat is designed to be seamless. In most cases, you only need to update your Cargo.toml dependency. The compatibility layer handles all the API translation, giving you the benefits of hidraw-rs without any code changes.