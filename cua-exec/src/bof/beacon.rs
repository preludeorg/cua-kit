//! Beacon API bindings for COFFLoader compatibility
//!
//! Uses __imp_BeaconXxx symbols that COFFLoader provides internally.

#![allow(dead_code)]
#![allow(unused_imports)]

use core::ffi::c_void;

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
#[allow(dead_code)]
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

/// Output a string with newline
#[allow(dead_code)]
pub fn output_line(s: &str) {
    output(s);
    output("\n");
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
unsafe fn extract_string(parser: *mut DataParser) -> &'static str {
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn go(args: *mut u8, args_len: i32) {
    unsafe {
        let mut parser = DataParser::new();

        if !args.is_null() && args_len > 0 {
            beacon_data_parse(&mut parser, args, args_len);
        }

        // Parse tool type (int: 0=Claude, 1=Codex)
        let tool = beacon_data_int(&mut parser) as u8;

        // Parse prompt (z-string)
        let prompt = extract_string(&mut parser);

        // Parse session_id (z-string, may be empty)
        let session_id = extract_string(&mut parser);

        if prompt.is_empty() {
            output_error("No prompt provided");
            return;
        }

        // Execute tool
        let session = if session_id.is_empty() {
            None
        } else {
            Some(session_id)
        };

        super::process::execute_tool(prompt, tool, session);
    }
}

/// Alternative entry point
#[unsafe(no_mangle)]
pub unsafe extern "C" fn entry(args: *mut u8, args_len: i32) {
    unsafe { go(args, args_len) }
}
