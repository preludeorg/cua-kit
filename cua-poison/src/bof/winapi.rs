//! Windows API bindings with LIBRARY$Function naming for COFFLoader
//!
//! COFFLoader resolves symbols in format: __imp_KERNEL32$FunctionName

// Allow non-snake_case to match Win32 API naming conventions
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::ffi::c_void;

pub type HANDLE = *mut c_void;
pub type DWORD = u32;
pub type BOOL = i32;
pub type LPCSTR = *const u8;
pub type LPSTR = *mut u8;

pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

// File access constants
pub const GENERIC_READ: DWORD = 0x80000000;
pub const GENERIC_WRITE: DWORD = 0x40000000;
pub const FILE_SHARE_READ: DWORD = 0x00000001;
pub const FILE_SHARE_WRITE: DWORD = 0x00000002;
pub const OPEN_EXISTING: DWORD = 3;
pub const FILE_ATTRIBUTE_NORMAL: DWORD = 0x80;
pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x10;
pub const FILE_BEGIN: DWORD = 0;
pub const FILE_END: DWORD = 2;

// Memory constants
pub const MEM_COMMIT: DWORD = 0x00001000;
pub const MEM_RESERVE: DWORD = 0x00002000;
pub const MEM_RELEASE: DWORD = 0x00008000;
pub const PAGE_READWRITE: DWORD = 0x04;

#[repr(C)]
pub struct SYSTEMTIME {
    pub wYear: u16,
    pub wMonth: u16,
    pub wDayOfWeek: u16,
    pub wDay: u16,
    pub wHour: u16,
    pub wMinute: u16,
    pub wSecond: u16,
    pub wMilliseconds: u16,
}

impl SYSTEMTIME {
    pub const fn zeroed() -> Self {
        Self {
            wYear: 0,
            wMonth: 0,
            wDayOfWeek: 0,
            wDay: 0,
            wHour: 0,
            wMinute: 0,
            wSecond: 0,
            wMilliseconds: 0,
        }
    }
}

#[repr(C)]
pub struct FILETIME {
    pub dwLowDateTime: DWORD,
    pub dwHighDateTime: DWORD,
}

#[repr(C)]
pub struct WIN32_FIND_DATAA {
    pub dwFileAttributes: DWORD,
    pub ftCreationTime: FILETIME,
    pub ftLastAccessTime: FILETIME,
    pub ftLastWriteTime: FILETIME,
    pub nFileSizeHigh: DWORD,
    pub nFileSizeLow: DWORD,
    pub dwReserved0: DWORD,
    pub dwReserved1: DWORD,
    pub cFileName: [u8; 260],
    pub cAlternateFileName: [u8; 14],
}

impl WIN32_FIND_DATAA {
    pub const fn zeroed() -> Self {
        Self {
            dwFileAttributes: 0,
            ftCreationTime: FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            },
            ftLastAccessTime: FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            },
            ftLastWriteTime: FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            },
            nFileSizeHigh: 0,
            nFileSizeLow: 0,
            dwReserved0: 0,
            dwReserved1: 0,
            cFileName: [0; 260],
            cAlternateFileName: [0; 14],
        }
    }

    /// Get the filename as a string
    pub fn file_name_str(&self) -> &str {
        let len = self.cFileName.iter().position(|&b| b == 0).unwrap_or(260);
        core::str::from_utf8(&self.cFileName[..len]).unwrap_or("")
    }
}

// Function pointer types
type CreateFileAFn =
    unsafe extern "system" fn(LPCSTR, DWORD, DWORD, *mut c_void, DWORD, DWORD, HANDLE) -> HANDLE;
type ReadFileFn =
    unsafe extern "system" fn(HANDLE, *mut u8, DWORD, *mut DWORD, *mut c_void) -> BOOL;
type WriteFileFn =
    unsafe extern "system" fn(HANDLE, *const u8, DWORD, *mut DWORD, *mut c_void) -> BOOL;
type CloseHandleFn = unsafe extern "system" fn(HANDLE) -> BOOL;
type GetFileSizeFn = unsafe extern "system" fn(HANDLE, *mut DWORD) -> DWORD;
type SetFilePointerFn = unsafe extern "system" fn(HANDLE, i32, *mut i32, DWORD) -> DWORD;
type GetProcessHeapFn = unsafe extern "system" fn() -> HANDLE;
type HeapAllocFn = unsafe extern "system" fn(HANDLE, DWORD, usize) -> *mut u8;
type HeapFreeFn = unsafe extern "system" fn(HANDLE, DWORD, *mut u8) -> BOOL;
type GetLastErrorFn = unsafe extern "system" fn() -> DWORD;
type GetSystemTimeFn = unsafe extern "system" fn(*mut SYSTEMTIME);
type GetEnvironmentVariableAFn = unsafe extern "system" fn(LPCSTR, LPSTR, DWORD) -> DWORD;
type FindFirstFileAFn = unsafe extern "system" fn(LPCSTR, *mut WIN32_FIND_DATAA) -> HANDLE;
type FindNextFileAFn = unsafe extern "system" fn(HANDLE, *mut WIN32_FIND_DATAA) -> BOOL;
type FindCloseFn = unsafe extern "system" fn(HANDLE) -> BOOL;

// advapi32 function for random number generation
type RtlGenRandomFn = unsafe extern "system" fn(*mut u8, DWORD) -> BOOL;

// Windows API symbols with KERNEL32$ prefix for COFFLoader
unsafe extern "C" {
    #[link_name = "__imp_KERNEL32$CreateFileA"]
    static CREATE_FILE_A: CreateFileAFn;

    #[link_name = "__imp_KERNEL32$ReadFile"]
    static READ_FILE: ReadFileFn;

    #[link_name = "__imp_KERNEL32$WriteFile"]
    static WRITE_FILE: WriteFileFn;

    #[link_name = "__imp_KERNEL32$CloseHandle"]
    static CLOSE_HANDLE: CloseHandleFn;

    #[link_name = "__imp_KERNEL32$GetFileSize"]
    static GET_FILE_SIZE: GetFileSizeFn;

    #[link_name = "__imp_KERNEL32$SetFilePointer"]
    static SET_FILE_POINTER: SetFilePointerFn;

    #[link_name = "__imp_KERNEL32$GetProcessHeap"]
    static GET_PROCESS_HEAP: GetProcessHeapFn;

    #[link_name = "__imp_KERNEL32$HeapAlloc"]
    static HEAP_ALLOC: HeapAllocFn;

    #[link_name = "__imp_KERNEL32$HeapFree"]
    static HEAP_FREE: HeapFreeFn;

    #[link_name = "__imp_KERNEL32$GetLastError"]
    static GET_LAST_ERROR: GetLastErrorFn;

    #[link_name = "__imp_KERNEL32$GetSystemTime"]
    static GET_SYSTEM_TIME: GetSystemTimeFn;

    #[link_name = "__imp_KERNEL32$GetEnvironmentVariableA"]
    static GET_ENVIRONMENT_VARIABLE_A: GetEnvironmentVariableAFn;

    #[link_name = "__imp_KERNEL32$FindFirstFileA"]
    static FIND_FIRST_FILE_A: FindFirstFileAFn;

    #[link_name = "__imp_KERNEL32$FindNextFileA"]
    static FIND_NEXT_FILE_A: FindNextFileAFn;

    #[link_name = "__imp_KERNEL32$FindClose"]
    static FIND_CLOSE: FindCloseFn;

    // advapi32 for random number generation (SystemFunction036 is RtlGenRandom)
    #[link_name = "__imp_ADVAPI32$SystemFunction036"]
    static RTL_GEN_RANDOM: RtlGenRandomFn;
}

// Safe wrappers
pub unsafe fn create_file_a(
    filename: LPCSTR,
    access: DWORD,
    share_mode: DWORD,
    creation_disposition: DWORD,
    flags: DWORD,
) -> HANDLE {
    unsafe {
        (CREATE_FILE_A)(
            filename,
            access,
            share_mode,
            core::ptr::null_mut(),
            creation_disposition,
            flags,
            core::ptr::null_mut(),
        )
    }
}

pub unsafe fn read_file(
    handle: HANDLE,
    buffer: *mut u8,
    size: DWORD,
    bytes_read: *mut DWORD,
) -> BOOL {
    unsafe { (READ_FILE)(handle, buffer, size, bytes_read, core::ptr::null_mut()) }
}

pub unsafe fn write_file(
    handle: HANDLE,
    buffer: *const u8,
    size: DWORD,
    bytes_written: *mut DWORD,
) -> BOOL {
    unsafe { (WRITE_FILE)(handle, buffer, size, bytes_written, core::ptr::null_mut()) }
}

pub unsafe fn close_handle(handle: HANDLE) -> BOOL {
    unsafe { (CLOSE_HANDLE)(handle) }
}

pub unsafe fn get_file_size(handle: HANDLE) -> DWORD {
    unsafe { (GET_FILE_SIZE)(handle, core::ptr::null_mut()) }
}

pub unsafe fn set_file_pointer(handle: HANDLE, distance: i32, method: DWORD) -> DWORD {
    unsafe { (SET_FILE_POINTER)(handle, distance, core::ptr::null_mut(), method) }
}

pub unsafe fn get_process_heap() -> HANDLE {
    unsafe { (GET_PROCESS_HEAP)() }
}

pub unsafe fn heap_alloc(heap: HANDLE, flags: DWORD, size: usize) -> *mut u8 {
    unsafe { (HEAP_ALLOC)(heap, flags, size) }
}

pub unsafe fn heap_free(heap: HANDLE, flags: DWORD, ptr: *mut u8) -> BOOL {
    unsafe { (HEAP_FREE)(heap, flags, ptr) }
}

pub unsafe fn get_last_error() -> DWORD {
    unsafe { (GET_LAST_ERROR)() }
}

pub unsafe fn get_system_time(time: *mut SYSTEMTIME) {
    unsafe { (GET_SYSTEM_TIME)(time) };
}

/// Generate random bytes using RtlGenRandom (SystemFunction036)
pub unsafe fn rtl_gen_random(buffer: *mut u8, len: DWORD) -> BOOL {
    unsafe { (RTL_GEN_RANDOM)(buffer, len) }
}

/// Get environment variable
pub unsafe fn get_environment_variable_a(name: LPCSTR, buffer: *mut u8, size: DWORD) -> DWORD {
    unsafe { (GET_ENVIRONMENT_VARIABLE_A)(name, buffer, size) }
}

/// Find first file matching pattern
pub unsafe fn find_first_file_a(pattern: LPCSTR, find_data: *mut WIN32_FIND_DATAA) -> HANDLE {
    unsafe { (FIND_FIRST_FILE_A)(pattern, find_data) }
}

/// Find next file
pub unsafe fn find_next_file_a(handle: HANDLE, find_data: *mut WIN32_FIND_DATAA) -> BOOL {
    unsafe { (FIND_NEXT_FILE_A)(handle, find_data) }
}

/// Close find handle
pub unsafe fn find_close(handle: HANDLE) -> BOOL {
    unsafe { (FIND_CLOSE)(handle) }
}
