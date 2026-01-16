//! Windows API bindings with LIBRARY$Function naming for COFFLoader
//!
//! COFFLoader resolves symbols in format: __imp_KERNEL32$FunctionName

// Allow non-snake_case to match Win32 API naming conventions
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::ffi::c_void;

pub type HANDLE = *mut c_void;
pub type HMODULE = *mut c_void;
pub type DWORD = u32;
pub type BOOL = i32;
pub type LPCSTR = *const u8;
pub type LPCWSTR = *const u16;
pub type LPWSTR = *mut u16;

pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
pub const MAX_PATH: usize = 260;
pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x10;
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF;
pub const GENERIC_READ: DWORD = 0x80000000;
pub const FILE_SHARE_READ: DWORD = 0x00000001;
pub const OPEN_EXISTING: DWORD = 3;
pub const FILE_ATTRIBUTE_NORMAL: DWORD = 0x80;

#[repr(C)]
pub struct WIN32_FIND_DATAW {
    pub dwFileAttributes: DWORD,
    pub ftCreationTime: [u32; 2],
    pub ftLastAccessTime: [u32; 2],
    pub ftLastWriteTime: [u32; 2],
    pub nFileSizeHigh: DWORD,
    pub nFileSizeLow: DWORD,
    pub dwReserved0: DWORD,
    pub dwReserved1: DWORD,
    pub cFileName: [u16; MAX_PATH],
    pub cAlternateFileName: [u16; 14],
}

impl WIN32_FIND_DATAW {
    pub const fn zeroed() -> Self {
        Self {
            dwFileAttributes: 0,
            ftCreationTime: [0; 2],
            ftLastAccessTime: [0; 2],
            ftLastWriteTime: [0; 2],
            nFileSizeHigh: 0,
            nFileSizeLow: 0,
            dwReserved0: 0,
            dwReserved1: 0,
            cFileName: [0; MAX_PATH],
            cAlternateFileName: [0; 14],
        }
    }
}

// Function pointer types
type FindFirstFileWFn = unsafe extern "system" fn(LPCWSTR, *mut WIN32_FIND_DATAW) -> HANDLE;
type FindNextFileWFn = unsafe extern "system" fn(HANDLE, *mut WIN32_FIND_DATAW) -> BOOL;
type FindCloseFn = unsafe extern "system" fn(HANDLE) -> BOOL;
type GetFileAttributesWFn = unsafe extern "system" fn(LPCWSTR) -> DWORD;
type CreateFileWFn =
    unsafe extern "system" fn(LPCWSTR, DWORD, DWORD, *mut c_void, DWORD, DWORD, HANDLE) -> HANDLE;
type ReadFileFn =
    unsafe extern "system" fn(HANDLE, *mut u8, DWORD, *mut DWORD, *mut c_void) -> BOOL;
type CloseHandleFn = unsafe extern "system" fn(HANDLE) -> BOOL;
type GetFileSizeFn = unsafe extern "system" fn(HANDLE, *mut DWORD) -> DWORD;
type GetProcessHeapFn = unsafe extern "system" fn() -> HANDLE;
type HeapAllocFn = unsafe extern "system" fn(HANDLE, DWORD, usize) -> *mut u8;
type HeapFreeFn = unsafe extern "system" fn(HANDLE, DWORD, *mut u8) -> BOOL;
type GetEnvironmentVariableWFn = unsafe extern "system" fn(LPCWSTR, LPWSTR, DWORD) -> DWORD;

// Windows API symbols with KERNEL32$ prefix for COFFLoader
unsafe extern "C" {
    #[link_name = "__imp_KERNEL32$FindFirstFileW"]
    static FIND_FIRST_FILE_W: FindFirstFileWFn;

    #[link_name = "__imp_KERNEL32$FindNextFileW"]
    static FIND_NEXT_FILE_W: FindNextFileWFn;

    #[link_name = "__imp_KERNEL32$FindClose"]
    static FIND_CLOSE: FindCloseFn;

    #[link_name = "__imp_KERNEL32$GetFileAttributesW"]
    static GET_FILE_ATTRIBUTES_W: GetFileAttributesWFn;

    #[link_name = "__imp_KERNEL32$CreateFileW"]
    static CREATE_FILE_W: CreateFileWFn;

    #[link_name = "__imp_KERNEL32$ReadFile"]
    static READ_FILE: ReadFileFn;

    #[link_name = "__imp_KERNEL32$CloseHandle"]
    static CLOSE_HANDLE: CloseHandleFn;

    #[link_name = "__imp_KERNEL32$GetFileSize"]
    static GET_FILE_SIZE: GetFileSizeFn;

    #[link_name = "__imp_KERNEL32$GetProcessHeap"]
    static GET_PROCESS_HEAP: GetProcessHeapFn;

    #[link_name = "__imp_KERNEL32$HeapAlloc"]
    static HEAP_ALLOC: HeapAllocFn;

    #[link_name = "__imp_KERNEL32$HeapFree"]
    static HEAP_FREE: HeapFreeFn;

    #[link_name = "__imp_KERNEL32$GetEnvironmentVariableW"]
    static GET_ENVIRONMENT_VARIABLE_W: GetEnvironmentVariableWFn;
}

// Safe wrappers
pub unsafe fn find_first_file_w(path: LPCWSTR, data: *mut WIN32_FIND_DATAW) -> HANDLE {
    unsafe { (FIND_FIRST_FILE_W)(path, data) }
}

pub unsafe fn find_next_file_w(handle: HANDLE, data: *mut WIN32_FIND_DATAW) -> BOOL {
    unsafe { (FIND_NEXT_FILE_W)(handle, data) }
}

pub unsafe fn find_close(handle: HANDLE) -> BOOL {
    unsafe { (FIND_CLOSE)(handle) }
}

pub unsafe fn get_file_attributes_w(path: LPCWSTR) -> DWORD {
    unsafe { (GET_FILE_ATTRIBUTES_W)(path) }
}

pub unsafe fn create_file_w(
    path: LPCWSTR,
    access: DWORD,
    share: DWORD,
    security: *mut c_void,
    disposition: DWORD,
    flags: DWORD,
    template: HANDLE,
) -> HANDLE {
    unsafe { (CREATE_FILE_W)(path, access, share, security, disposition, flags, template) }
}

pub unsafe fn read_file(
    handle: HANDLE,
    buffer: *mut u8,
    size: DWORD,
    bytes_read: *mut DWORD,
    overlapped: *mut c_void,
) -> BOOL {
    unsafe { (READ_FILE)(handle, buffer, size, bytes_read, overlapped) }
}

pub unsafe fn close_handle(handle: HANDLE) -> BOOL {
    unsafe { (CLOSE_HANDLE)(handle) }
}

pub unsafe fn get_file_size(handle: HANDLE, high: *mut DWORD) -> DWORD {
    unsafe { (GET_FILE_SIZE)(handle, high) }
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

pub unsafe fn get_environment_variable_w(name: LPCWSTR, buffer: LPWSTR, size: DWORD) -> DWORD {
    unsafe { (GET_ENVIRONMENT_VARIABLE_W)(name, buffer, size) }
}
