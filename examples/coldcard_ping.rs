//! Example: Ping a Coldcard device

use hidraw_rs::coldcard::{ColdcardDevice, COINKITE_VID, COLDCARD_PID};
use hidraw_rs::prelude::*;

fn main() -> Result<()> {
    println!("Looking for Coldcard devices...");
    
    // Find Coldcard devices
    let devices = find_devices(COINKITE_VID, COLDCARD_PID)?;
    
    if devices.is_empty() {
        eprintln!("No Coldcard devices found!");
        eprintln!("\nMake sure:");
        eprintln!("1. Your Coldcard is connected via USB");
        eprintln!("2. You have permission to access the device");
        eprintln!("3. The device is not in use by another application");
        return Ok(());
    }

    println!("Found {} Coldcard device(s)", devices.len());
    
    // Open the first Coldcard
    println!("\nOpening Coldcard...");
    let mut coldcard = ColdcardDevice::open()?;
    
    // Display device info
    let info = coldcard.info();
    println!("Device info:");
    println!("  Path: {}", info.path.display());
    if let Some(product) = &info.product {
        println!("  Product: {}", product);
    }
    if let Some(serial) = &info.serial_number {
        println!("  Serial: {}", serial);
    }

    // Get version
    println!("\nGetting version...");
    match coldcard.get_version() {
        Ok(version) => println!("Version: {}", version),
        Err(e) => eprintln!("Failed to get version: {}", e),
    }

    // Send ping
    println!("\nSending ping...");
    let ping_msg = b"Hello from pure-rust-hid!";
    match coldcard.ping(ping_msg) {
        Ok(response) => {
            println!("Ping response: {:?}", String::from_utf8_lossy(&response));
        }
        Err(e) => {
            eprintln!("Ping failed: {}", e);
        }
    }

    // Get status
    println!("\nGetting status...");
    match coldcard.get_status() {
        Ok(status) => {
            println!("Status received, {} bytes", status.len());
            if !status.is_empty() {
                // Try to parse as text first
                if let Ok(text) = String::from_utf8(status.clone()) {
                    println!("Status: {}", text);
                } else {
                    println!("Status (hex): {:02x?}", status);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get status: {}", e);
        }
    }

    println!("\nDone!");
    Ok(())
}