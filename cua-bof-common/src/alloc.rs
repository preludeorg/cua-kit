//! Simple heap allocator for `no_std` BOF using Windows `HeapAlloc`

use super::heap;
use core::alloc::{GlobalAlloc, Layout};

pub struct BofAllocator;

unsafe impl GlobalAlloc for BofAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let h = heap::get_process_heap();
            if h.is_null() {
                return core::ptr::null_mut();
            }
            heap::heap_alloc(h, 0, layout.size())
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe {
            let h = heap::get_process_heap();
            if !h.is_null() && !ptr.is_null() {
                heap::heap_free(h, 0, ptr);
            }
        }
    }
}
