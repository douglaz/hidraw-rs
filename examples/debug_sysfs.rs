//! Debug sysfs path resolution

use std::fs;
use std::path::Path;

fn main() {
    let device_path = Path::new("/sys/class/hidraw/hidraw12/device");

    println!("Debugging sysfs path resolution for hidraw12");
    println!("============================================\n");

    println!("Device path: {}", device_path.display());
    println!("Is symlink: {}", device_path.is_symlink());

    if device_path.is_symlink() {
        match fs::read_link(device_path) {
            Ok(target) => {
                println!("Symlink target: {}", target.display());
                println!("Is relative: {}", target.is_relative());

                // Resolve to absolute path
                let absolute = if target.is_relative() {
                    device_path.parent().unwrap().join(&target)
                } else {
                    target.clone()
                };

                println!("Absolute path: {}", absolute.display());

                // Canonicalize the path to resolve .. components
                let canonical_path = match fs::canonicalize(&absolute) {
                    Ok(canonical) => {
                        println!("Canonical path: {}", canonical.display());
                        canonical
                    }
                    Err(e) => {
                        println!("Failed to canonicalize: {}", e);
                        absolute
                    }
                };

                // Also try going up from the canonical path of the device symlink itself
                if let Ok(canonical_device) = fs::canonicalize(device_path) {
                    println!(
                        "\nAlternative: canonical device path: {}",
                        canonical_device.display()
                    );

                    // Walk up from here
                    let mut alt_current = canonical_device;
                    for i in 0..10 {
                        println!("\nAlt Level {}: {}", i, alt_current.display());

                        let vendor_path = alt_current.join("idVendor");
                        println!("  Checking for idVendor: {}", vendor_path.exists());

                        if vendor_path.exists() {
                            println!("  FOUND! Reading vendor/product IDs...");
                            if let Ok(vendor) = fs::read_to_string(&vendor_path) {
                                println!("  Vendor ID: {}", vendor.trim());
                            }
                            if let Ok(product) = fs::read_to_string(alt_current.join("idProduct")) {
                                println!("  Product ID: {}", product.trim());
                            }
                            break;
                        }

                        if let Some(parent) = alt_current.parent() {
                            alt_current = parent.to_path_buf();
                        } else {
                            println!("  No parent - reached root");
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error reading symlink: {}", e);
            }
        }
    }
}
