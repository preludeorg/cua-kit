//! Compiler intrinsics for `no_std` BOF
//!
//! Provides memcpy, memset, memcmp that the compiler expects

#![allow(clippy::missing_safety_doc)]

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        let mut i = 0;
        while i < n {
            *dest.add(i) = *src.add(i);
            i += 1;
        }
        dest
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        if src < dest.cast_const() {
            // Copy backwards
            let mut i = n;
            while i > 0 {
                i -= 1;
                *dest.add(i) = *src.add(i);
            }
        } else {
            // Copy forwards
            let mut i = 0;
            while i < n {
                *dest.add(i) = *src.add(i);
                i += 1;
            }
        }
        dest
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    unsafe {
        let mut i = 0;
        let c = c as u8;
        while i < n {
            *dest.add(i) = c;
            i += 1;
        }
        dest
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    unsafe {
        let mut i = 0;
        while i < n {
            let a = *s1.add(i);
            let b = *s2.add(i);
            if a != b {
                return i32::from(a) - i32::from(b);
            }
            i += 1;
        }
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    unsafe {
        let mut len = 0;
        while *s.add(len) != 0 {
            len += 1;
        }
        len
    }
}

// C++ exception handler stub - BOFs don't use exceptions
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __CxxFrameHandler3() -> i32 {
    0
}
