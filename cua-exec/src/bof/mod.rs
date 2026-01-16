//! No-std BOF implementation for COFFLoader compatibility
//!
//! This module provides a BOF-compatible implementation that uses the
//! LIBRARY$Function symbol naming convention expected by COFFLoader.

#![cfg(feature = "bof")]

mod beacon;
mod process;
mod winapi;

// Use intrinsics from shared crate (needed for no_std support)
#[allow(unused_imports)]
pub use cua_bof_common::intrinsics;

pub use beacon::entry;
pub use beacon::go;
