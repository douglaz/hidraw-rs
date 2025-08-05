//! Debug version of device enumeration to troubleshoot issues

use std::fs;
use std::path::Path;

fn main() {
    println!("Debug HID Device Enumeration");
    println!("============================\n");

    // Check /sys/class/hidraw
    let hidraw_class = Path::new("/sys/class/hidraw");
    println!("Checking {}", hidraw_class.display());

    if !hidraw_class.exists() {
        eprintln!("ERROR: /sys/class/hidraw does not exist!");
        return;
    }

    // List all hidraw entries
    match fs::read_dir(hidraw_class) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let device_path = Path::new("/dev").join(&name);

                println!("\nFound: {}", name.to_string_lossy());
                println!("  Device path: {}", device_path.display());
                println!("  Exists: {}", device_path.exists());

                // Check permissions
                if let Ok(metadata) = fs::metadata(&device_path) {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = metadata.permissions().mode();
                    println!("  Permissions: {:o}", mode & 0o777);

                    // Check if we can open it
                    match fs::File::open(&device_path) {
                        Ok(_) => println!("  Can open: YES"),
                        Err(e) => println!("  Can open: NO ({})", e),
                    }
                }

                // Check sysfs
                let sysfs_path = hidraw_class.join(&name).join("device").join("uevent");
                if let Ok(uevent) = fs::read_to_string(&sysfs_path) {
                    println!("  Device info:");
                    for line in uevent.lines() {
                        if line.starts_with("HID_NAME=") || line.starts_with("HID_ID=") {
                            println!("    {}", line);
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("ERROR reading /sys/class/hidraw: {}", e);
        }
    }

    println!("\n\nNow trying hidraw-rs enumeration...");
    match hidraw_rs::enumerate() {
        Ok(devices) => {
            println!("Found {} devices", devices.len());
            for device in devices {
                println!("\nDevice: {}", device.display_name());
                println!("  Path: {}", device.path.display());
                println!(
                    "  VID:PID: {:04x}:{:04x}",
                    device.vendor_id, device.product_id
                );
            }
        }
        Err(e) => {
            eprintln!("Enumeration failed: {}", e);
        }
    }

    // Try to get device info directly for hidraw12
    println!("\n\nTrying to get device info for /dev/hidraw12 directly...");
    match hidraw_rs::hidraw::get_device_info(Path::new("/dev/hidraw12")) {
        Ok(info) => {
            println!("Success!");
            println!("  VID:PID: {:04x}:{:04x}", info.vendor_id, info.product_id);
            println!("  Manufacturer: {:?}", info.manufacturer);
            println!("  Product: {:?}", info.product);
            println!("  Serial: {:?}", info.serial_number);
        }
        Err(e) => {
            eprintln!("Failed to get device info: {}", e);
        }
    }
}
