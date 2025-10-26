//! MfaDevice Resource Module
//!
//! This module provides self-contained handling of IAM MFA device resources.

pub mod builder;
pub mod model;
pub mod operations;
pub mod requests;

pub use model::MfaDevice;
pub use requests::{EnableMfaDeviceRequest, ListMfaDevicesRequest};
