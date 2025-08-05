# Unsafe Code Usage in hidraw-rs

This document explains why certain `unsafe` blocks are necessary in the hidraw-rs codebase.

## Overview

The hidraw-rs library minimizes unsafe code usage wherever possible. However, some unsafe blocks are unavoidable due to the nature of low-level system programming and hardware interaction.

## Remaining Unsafe Blocks

### 1. ioctl System Calls (`src/hidraw/ioctl.rs`)

**Location**: Lines 14, 16, 31, 33, 48, 50, 64, 66

**Why it's necessary**:
- The `ioctl` system call is inherently unsafe as it performs low-level hardware control operations
- Rust's standard library and rustix (as of version 0.38) do not provide safe wrappers for custom ioctl operations
- We need to pass raw pointers to kernel space for device communication
- The kernel expects specific memory layouts that must be manually ensured

**Safety measures taken**:
- All ioctl calls are wrapped in safe public functions
- Input validation is performed before making unsafe calls
- Error handling converts raw errno values to proper Rust errors
- Buffer bounds are checked before passing to ioctl

**Example**:
```rust
// We must use unsafe to call the raw ioctl system call
let ret = unsafe { libc::ioctl(fd_raw, request as libc::c_ulong, &mut value) };
```

## Alternatives Considered

1. **rustix ioctl module**: Not available in rustix 0.38
2. **nix crate**: Would add another dependency with similar unsafe usage internally
3. **iocuddle crate**: Additional dependency that still uses unsafe internally

## Conclusion

The remaining unsafe code is limited to the absolute minimum required for:
- Direct kernel communication via ioctl
- Hardware device control operations

All unsafe code is:
- Well-documented with safety requirements
- Wrapped in safe public APIs
- Validated before use
- Tested thoroughly

The library successfully reduced unsafe usage from the original implementation while maintaining full functionality and performance.