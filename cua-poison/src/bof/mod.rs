//! No-std BOF implementation for COFFLoader compatibility
//!
//! This module provides a BOF-compatible implementation for session poisoning.

#![cfg(feature = "bof")]

mod beacon;
mod poison;
mod winapi;

pub use beacon::entry;
pub use beacon::go;
