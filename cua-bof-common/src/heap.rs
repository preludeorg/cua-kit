//! Heap functions for BOF allocator
//!
//! Provides Windows heap API access via `COFFLoader` symbol resolution.

#![allow(clippy::missing_safety_doc)]

use core::ffi::c_void;

pub type HANDLE = *mut c_void;
pub type DWORD = u32;
pub type BOOL = i32;

// Function pointer types
type GetProcessHeapFn = unsafe extern "system" fn() -> HANDLE;
type HeapAllocFn = unsafe extern "system" fn(HANDLE, DWORD, usize) -> *mut u8;
type HeapFreeFn = unsafe extern "system" fn(HANDLE, DWORD, *mut u8) -> BOOL;

// Windows API symbols with KERNEL32$ prefix for COFFLoader
unsafe extern "C" {
    #[link_name = "__imp_KERNEL32$GetProcessHeap"]
    static GET_PROCESS_HEAP: GetProcessHeapFn;

    #[link_name = "__imp_KERNEL32$HeapAlloc"]
    static HEAP_ALLOC: HeapAllocFn;

    #[link_name = "__imp_KERNEL32$HeapFree"]
    static HEAP_FREE: HeapFreeFn;
}

#[must_use]
pub unsafe fn get_process_heap() -> HANDLE {
    unsafe { (GET_PROCESS_HEAP)() }
}

pub unsafe fn heap_alloc(heap: HANDLE, flags: DWORD, size: usize) -> *mut u8 {
    unsafe { (HEAP_ALLOC)(heap, flags, size) }
}

pub unsafe fn heap_free(heap: HANDLE, flags: DWORD, ptr: *mut u8) -> BOOL {
    unsafe { (HEAP_FREE)(heap, flags, ptr) }
}
