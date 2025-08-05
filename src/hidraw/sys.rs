//! System-level constants and structures for hidraw

// ioctl constants calculation without libc dependency
const IOC_NRBITS: u32 = 8;
const IOC_TYPEBITS: u32 = 8;
const IOC_SIZEBITS: u32 = 14;
const IOC_DIRBITS: u32 = 2;

const IOC_NRSHIFT: u32 = 0;
const IOC_TYPESHIFT: u32 = IOC_NRSHIFT + IOC_NRBITS;
const IOC_SIZESHIFT: u32 = IOC_TYPESHIFT + IOC_TYPEBITS;
const IOC_DIRSHIFT: u32 = IOC_SIZESHIFT + IOC_SIZEBITS;

const IOC_NONE: u32 = 0;
const IOC_WRITE: u32 = 1;
const IOC_READ: u32 = 2;

/// Generate ioctl command number
const fn _ioc(dir: u32, type_: u32, nr: u32, size: u32) -> u32 {
    (dir << IOC_DIRSHIFT) |
    (type_ << IOC_TYPESHIFT) |
    (nr << IOC_NRSHIFT) |
    (size << IOC_SIZESHIFT)
}

/// ioctl command for read operations
const fn _ior(type_: u32, nr: u32, size: u32) -> u32 {
    _ioc(IOC_READ, type_, nr, size)
}

/// ioctl command for write operations
const fn _iow(type_: u32, nr: u32, size: u32) -> u32 {
    _ioc(IOC_WRITE, type_, nr, size)
}

/// ioctl command for read/write operations
const fn _iowr(type_: u32, nr: u32, size: u32) -> u32 {
    _ioc(IOC_READ | IOC_WRITE, type_, nr, size)
}

// HID ioctl command numbers
const HID_TYPE: u32 = b'H' as u32;

/// Get report descriptor size
pub const HIDIOCGRDESCSIZE: u32 = _ior(HID_TYPE, 0x01, 4);

/// Get report descriptor
pub const HIDIOCGRDESC: u32 = _ior(HID_TYPE, 0x02, 4096);

/// Get raw device info
pub const HIDIOCGRAWINFO: u32 = _ior(HID_TYPE, 0x03, 8);

/// Get raw device name
pub const HIDIOCGRAWNAME: u32 = _ior(HID_TYPE, 0x04, 256);

/// Get raw physical info
pub const HIDIOCGRAWPHYS: u32 = _ior(HID_TYPE, 0x05, 256);

/// Send feature report
pub fn hidiocgfeature(len: usize) -> u32 {
    _iowr(HID_TYPE, 0x06, len as u32)
}

/// Set feature report
pub fn hidiocsfeature(len: usize) -> u32 {
    _iowr(HID_TYPE, 0x07, len as u32)
}

/// Get raw unique ID
pub const HIDIOCGRAWUNIQ: u32 = _ior(HID_TYPE, 0x08, 256);

/// Raw device info structure
#[repr(C)]
pub struct HidrawDevInfo {
    pub bustype: u32,
    pub vendor: i16,
    pub product: i16,
}

/// Report descriptor structure
#[repr(C)]
pub struct HidrawReportDescriptor {
    pub size: u32,
    pub value: [u8; 4096],
}