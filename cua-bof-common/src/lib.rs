//! Shared BOF infrastructure for CUA-Kit
//!
//! Provides common allocator, intrinsics, and heap functions
//! for `no_std` BOF builds.
//!
//! This crate is Windows-only as BOF (Beacon Object File) format
//! is specific to Cobalt Strike on Windows.

#![no_std]

// BOF infrastructure is Windows-only
#[cfg(not(windows))]
compile_error!("cua-bof-common only supports Windows targets. BOF format is Windows-specific.");

#[cfg(windows)]
pub mod alloc;
#[cfg(windows)]
pub mod heap;
#[cfg(windows)]
pub mod intrinsics;

#[cfg(windows)]
pub use alloc::BofAllocator;
#[cfg(windows)]
pub use heap::{BOOL, DWORD, HANDLE};
