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

// Process creation constants
pub const CREATE_NO_WINDOW: DWORD = 0x08000000;
pub const STARTF_USESTDHANDLES: DWORD = 0x00000100;
pub const STARTF_USESHOWWINDOW: DWORD = 0x00000001;
pub const SW_HIDE: u16 = 0;
pub const WAIT_TIMEOUT: DWORD = 0x00000102;
pub const MEM_COMMIT: DWORD = 0x00001000;
pub const MEM_RESERVE: DWORD = 0x00002000;
pub const MEM_RELEASE: DWORD = 0x00008000;
pub const PAGE_READWRITE: DWORD = 0x04;

#[repr(C)]
pub struct SECURITY_ATTRIBUTES {
    pub nLength: DWORD,
    pub lpSecurityDescriptor: *mut c_void,
    pub bInheritHandle: BOOL,
}

impl SECURITY_ATTRIBUTES {
    pub const fn zeroed() -> Self {
        Self {
            nLength: 0,
            lpSecurityDescriptor: core::ptr::null_mut(),
            bInheritHandle: 0,
        }
    }
}

#[repr(C)]
pub struct STARTUPINFOA {
    pub cb: DWORD,
    pub lpReserved: LPSTR,
    pub lpDesktop: LPSTR,
    pub lpTitle: LPSTR,
    pub dwX: DWORD,
    pub dwY: DWORD,
    pub dwXSize: DWORD,
    pub dwYSize: DWORD,
    pub dwXCountChars: DWORD,
    pub dwYCountChars: DWORD,
    pub dwFillAttribute: DWORD,
    pub dwFlags: DWORD,
    pub wShowWindow: u16,
    pub cbReserved2: u16,
    pub lpReserved2: *mut u8,
    pub hStdInput: HANDLE,
    pub hStdOutput: HANDLE,
    pub hStdError: HANDLE,
}

impl STARTUPINFOA {
    pub const fn zeroed() -> Self {
        Self {
            cb: 0,
            lpReserved: core::ptr::null_mut(),
            lpDesktop: core::ptr::null_mut(),
            lpTitle: core::ptr::null_mut(),
            dwX: 0,
            dwY: 0,
            dwXSize: 0,
            dwYSize: 0,
            dwXCountChars: 0,
            dwYCountChars: 0,
            dwFillAttribute: 0,
            dwFlags: 0,
            wShowWindow: 0,
            cbReserved2: 0,
            lpReserved2: core::ptr::null_mut(),
            hStdInput: core::ptr::null_mut(),
            hStdOutput: core::ptr::null_mut(),
            hStdError: core::ptr::null_mut(),
        }
    }
}

#[repr(C)]
pub struct PROCESS_INFORMATION {
    pub hProcess: HANDLE,
    pub hThread: HANDLE,
    pub dwProcessId: DWORD,
    pub dwThreadId: DWORD,
}

impl PROCESS_INFORMATION {
    pub const fn zeroed() -> Self {
        Self {
            hProcess: core::ptr::null_mut(),
            hThread: core::ptr::null_mut(),
            dwProcessId: 0,
            dwThreadId: 0,
        }
    }
}

// Function pointer types
type CreateProcessAFn = unsafe extern "system" fn(
    LPCSTR,
    LPSTR,
    *mut SECURITY_ATTRIBUTES,
    *mut SECURITY_ATTRIBUTES,
    BOOL,
    DWORD,
    *mut c_void,
    LPCSTR,
    *mut STARTUPINFOA,
    *mut PROCESS_INFORMATION,
) -> BOOL;
type CreatePipeFn =
    unsafe extern "system" fn(*mut HANDLE, *mut HANDLE, *mut SECURITY_ATTRIBUTES, DWORD) -> BOOL;
type WaitForSingleObjectFn = unsafe extern "system" fn(HANDLE, DWORD) -> DWORD;
type ReadFileFn =
    unsafe extern "system" fn(HANDLE, *mut u8, DWORD, *mut DWORD, *mut c_void) -> BOOL;
type CloseHandleFn = unsafe extern "system" fn(HANDLE) -> BOOL;
type GetProcessHeapFn = unsafe extern "system" fn() -> HANDLE;
type HeapAllocFn = unsafe extern "system" fn(HANDLE, DWORD, usize) -> *mut u8;
type HeapFreeFn = unsafe extern "system" fn(HANDLE, DWORD, *mut u8) -> BOOL;
type GetLastErrorFn = unsafe extern "system" fn() -> DWORD;

// Windows API symbols with KERNEL32$ prefix for COFFLoader
unsafe extern "C" {
    #[link_name = "__imp_KERNEL32$CreateProcessA"]
    static CREATE_PROCESS_A: CreateProcessAFn;

    #[link_name = "__imp_KERNEL32$CreatePipe"]
    static CREATE_PIPE: CreatePipeFn;

    #[link_name = "__imp_KERNEL32$WaitForSingleObject"]
    static WAIT_FOR_SINGLE_OBJECT: WaitForSingleObjectFn;

    #[link_name = "__imp_KERNEL32$ReadFile"]
    static READ_FILE: ReadFileFn;

    #[link_name = "__imp_KERNEL32$CloseHandle"]
    static CLOSE_HANDLE: CloseHandleFn;

    #[link_name = "__imp_KERNEL32$GetProcessHeap"]
    static GET_PROCESS_HEAP: GetProcessHeapFn;

    #[link_name = "__imp_KERNEL32$HeapAlloc"]
    static HEAP_ALLOC: HeapAllocFn;

    #[link_name = "__imp_KERNEL32$HeapFree"]
    static HEAP_FREE: HeapFreeFn;

    #[link_name = "__imp_KERNEL32$GetLastError"]
    static GET_LAST_ERROR: GetLastErrorFn;
}

// Safe wrappers
pub unsafe fn create_process_a(
    cmd_line: LPSTR,
    inherit_handles: bool,
    creation_flags: DWORD,
    startup_info: *mut STARTUPINFOA,
    process_info: *mut PROCESS_INFORMATION,
) -> BOOL {
    unsafe {
        (CREATE_PROCESS_A)(
            core::ptr::null(),
            cmd_line,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            if inherit_handles { 1 } else { 0 },
            creation_flags,
            core::ptr::null_mut(),
            core::ptr::null(),
            startup_info,
            process_info,
        )
    }
}

pub unsafe fn create_pipe(
    read_handle: *mut HANDLE,
    write_handle: *mut HANDLE,
    security_attrs: *mut SECURITY_ATTRIBUTES,
    size: DWORD,
) -> BOOL {
    unsafe { (CREATE_PIPE)(read_handle, write_handle, security_attrs, size) }
}

pub unsafe fn wait_for_single_object(handle: HANDLE, milliseconds: DWORD) -> DWORD {
    unsafe { (WAIT_FOR_SINGLE_OBJECT)(handle, milliseconds) }
}

pub unsafe fn read_file(
    handle: HANDLE,
    buffer: *mut u8,
    size: DWORD,
    bytes_read: *mut DWORD,
) -> BOOL {
    unsafe { (READ_FILE)(handle, buffer, size, bytes_read, core::ptr::null_mut()) }
}

pub unsafe fn close_handle(handle: HANDLE) -> BOOL {
    unsafe { (CLOSE_HANDLE)(handle) }
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
