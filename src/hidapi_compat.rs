//! Re-export hidapi-compat types when the feature is enabled
//!
//! This module provides a convenient way to use hidapi-compat types
//! through the main hidraw-rs crate when the `hidapi-compat` feature is enabled.

#[cfg(feature = "hidapi-compat")]
pub use hidapi_compat::*;
