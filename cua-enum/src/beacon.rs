//! Beacon Object File (BOF) support module
//!
//! Provides the BOF entry point and Beacon API bindings for Cobalt Strike integration.
//! Uses dynamic function resolution following Beacon's standard.

/// Beacon output type constants
#[allow(dead_code)]
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
    pub fn new() -> Self {
        Self {
            original: std::ptr::null_mut(),
            buffer: std::ptr::null_mut(),
            length: 0,
            size: 0,
        }
    }
}

impl Default for DataParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Dynamic function resolution for Beacon APIs
/// These are resolved at runtime when running as a BOF
/// Uses static function pointers with __imp_* symbols for COFF loader compatibility
#[cfg(feature = "bof")]
mod beacon_api {
    use super::*;

    // Function pointer types for Beacon API
    type BeaconOutputFn = unsafe extern "C" fn(i32, *const u8, i32);
    type BeaconDataParseFn = unsafe extern "C" fn(*mut DataParser, *mut u8, i32);
    type BeaconDataIntFn = unsafe extern "C" fn(*mut DataParser) -> i32;
    type BeaconDataExtractFn = unsafe extern "C" fn(*mut DataParser, *mut i32) -> *mut u8;

    // Import symbols using __imp_* naming convention for COFF loaders
    extern "C" {
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
        (BEACON_OUTPUT)(typ, data, len);
    }

    /// Initialize data parser with buffer
    pub unsafe fn beacon_data_parse(parser: *mut DataParser, buffer: *mut u8, size: i32) {
        (BEACON_DATA_PARSE)(parser, buffer, size);
    }

    /// Extract integer from data parser
    pub unsafe fn beacon_data_int(parser: *mut DataParser) -> i32 {
        (BEACON_DATA_INT)(parser)
    }

    /// Extract byte buffer from data parser
    pub unsafe fn beacon_data_extract(parser: *mut DataParser, size: *mut i32) -> *mut u8 {
        (BEACON_DATA_EXTRACT)(parser, size)
    }
}

/// Output text to Beacon console (BOF mode) or stdout (EXE mode)
pub fn output(text: &str) {
    #[cfg(feature = "bof")]
    unsafe {
        let c_str = format!("{}\0", text);
        beacon_api::beacon_output(CALLBACK_OUTPUT_UTF8, c_str.as_ptr(), c_str.len() as i32 - 1);
    }

    #[cfg(not(feature = "bof"))]
    {
        print!("{}", text);
    }
}

/// Output a line of text with newline
pub fn output_line(text: &str) {
    #[cfg(feature = "bof")]
    unsafe {
        let c_str = format!("{}\n\0", text);
        beacon_api::beacon_output(CALLBACK_OUTPUT_UTF8, c_str.as_ptr(), c_str.len() as i32 - 1);
    }

    #[cfg(not(feature = "bof"))]
    {
        println!("{}", text);
    }
}

/// Output an error message
pub fn output_error(text: &str) {
    #[cfg(feature = "bof")]
    unsafe {
        let c_str = format!("[!] {}\n\0", text);
        beacon_api::beacon_output(CALLBACK_ERROR, c_str.as_ptr(), c_str.len() as i32 - 1);
    }

    #[cfg(not(feature = "bof"))]
    {
        eprintln!("[!] {}", text);
    }
}

/// BOF entry point - called by Cobalt Strike's inline-execute
/// Arguments are passed via Beacon's data API
///
/// Argument protocol:
/// - Int (4 bytes): flags (bit 0 = JSON output)
/// - String (Z): optional target username
#[no_mangle]
#[cfg(feature = "bof")]
pub unsafe extern "C" fn go(args: *mut u8, args_len: i32) {
    let mut parser = DataParser::new();

    if !args.is_null() && args_len > 0 {
        beacon_api::beacon_data_parse(&mut parser, args, args_len);
    }

    // Parse flags: first int determines output format (bit 0 = JSON)
    let flags = if parser.length >= 4 {
        beacon_api::beacon_data_int(&mut parser)
    } else {
        0
    };

    let json_output = (flags & 1) != 0;

    // Parse optional username string
    let target_user: Option<String> = if parser.length > 0 {
        let mut len: i32 = 0;
        let ptr = beacon_api::beacon_data_extract(&mut parser, &mut len);
        if !ptr.is_null() && len > 1 {
            let slice = core::slice::from_raw_parts(ptr, len as usize);
            let end = slice.iter().position(|&b| b == 0).unwrap_or(len as usize);
            if end > 0 {
                Some(String::from_utf8_lossy(&slice[..end]).into_owned())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Run the enumeration with parsed arguments
    crate::run_enumeration(json_output, target_user.as_deref());
}

/// Alternative entry point name for compatibility
#[no_mangle]
#[cfg(feature = "bof")]
pub unsafe extern "C" fn entry(args: *mut u8, args_len: i32) {
    go(args, args_len);
}
