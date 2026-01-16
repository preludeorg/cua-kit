//! File system operations for no_std BOF

#![allow(dead_code)]

extern crate alloc;

use super::winapi::*;
use alloc::string::String;
use alloc::vec::Vec;

/// Convert a Rust string to a null-terminated UTF-16 wide string
pub fn to_wide(s: &str) -> Vec<u16> {
    let mut wide: Vec<u16> = s.encode_utf16().collect();
    wide.push(0);
    wide
}

/// Convert UTF-16 wide string to Rust String (stops at null)
pub fn from_wide(wide: &[u16]) -> String {
    let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..len])
}

/// Check if a path exists and is a directory
pub fn is_dir(path: &str) -> bool {
    let wide = to_wide(path);
    unsafe {
        let attrs = get_file_attributes_w(wide.as_ptr());
        attrs != INVALID_FILE_ATTRIBUTES && (attrs & FILE_ATTRIBUTE_DIRECTORY) != 0
    }
}

/// Check if a path exists (file or directory)
pub fn exists(path: &str) -> bool {
    let wide = to_wide(path);
    unsafe { get_file_attributes_w(wide.as_ptr()) != INVALID_FILE_ATTRIBUTES }
}

/// Read a file's contents as a String
pub fn read_to_string(path: &str) -> Option<String> {
    let wide = to_wide(path);
    unsafe {
        let handle = create_file_w(
            wide.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            core::ptr::null_mut(),
        );

        if handle == INVALID_HANDLE_VALUE {
            return None;
        }

        let size = get_file_size(handle, core::ptr::null_mut());
        if size == 0xFFFFFFFF || size == 0 {
            close_handle(handle);
            return None;
        }

        // Limit file size to 1MB for safety
        if size > 1024 * 1024 {
            close_handle(handle);
            return None;
        }

        let mut buffer = Vec::with_capacity(size as usize + 1);
        buffer.resize(size as usize, 0u8);

        let mut bytes_read: DWORD = 0;
        let result = read_file(
            handle,
            buffer.as_mut_ptr(),
            size,
            &mut bytes_read,
            core::ptr::null_mut(),
        );

        close_handle(handle);

        if result == 0 {
            return None;
        }

        buffer.truncate(bytes_read as usize);
        String::from_utf8(buffer).ok()
    }
}

/// List directories in a path (returns directory names only)
pub fn list_dirs(path: &str) -> Vec<String> {
    let mut result = Vec::new();
    let search_path = if path.ends_with('\\') || path.ends_with('/') {
        alloc::format!("{}*", path)
    } else {
        alloc::format!("{}\\*", path)
    };

    let wide = to_wide(&search_path);
    let mut fd = WIN32_FIND_DATAW::zeroed();

    unsafe {
        let handle = find_first_file_w(wide.as_ptr(), &mut fd);
        if handle == INVALID_HANDLE_VALUE {
            return result;
        }

        loop {
            if (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                let name = from_wide(&fd.cFileName);
                // Skip . and ..
                if name != "." && name != ".." {
                    result.push(name);
                }
            }

            if find_next_file_w(handle, &mut fd) == 0 {
                break;
            }
        }

        find_close(handle);
    }

    result
}

/// List files matching a pattern in a directory
pub fn list_files(path: &str, pattern: &str) -> Vec<String> {
    let mut result = Vec::new();
    let search_path = if path.ends_with('\\') || path.ends_with('/') {
        alloc::format!("{}{}", path, pattern)
    } else {
        alloc::format!("{}\\{}", path, pattern)
    };

    let wide = to_wide(&search_path);
    let mut fd = WIN32_FIND_DATAW::zeroed();

    unsafe {
        let handle = find_first_file_w(wide.as_ptr(), &mut fd);
        if handle == INVALID_HANDLE_VALUE {
            return result;
        }

        loop {
            if (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0 {
                let name = from_wide(&fd.cFileName);
                result.push(name);
            }

            if find_next_file_w(handle, &mut fd) == 0 {
                break;
            }
        }

        find_close(handle);
    }

    result
}

/// Get environment variable value
pub fn get_env(name: &str) -> Option<String> {
    let wide_name = to_wide(name);
    let mut buffer = [0u16; 512];

    unsafe {
        let len = get_environment_variable_w(
            wide_name.as_ptr(),
            buffer.as_mut_ptr(),
            buffer.len() as DWORD,
        );

        if len == 0 || len >= buffer.len() as DWORD {
            return None;
        }

        Some(from_wide(&buffer[..len as usize]))
    }
}

/// Get user home directory
pub fn get_user_home(user: &str) -> String {
    alloc::format!("C:\\Users\\{}", user)
}

/// Join path components
pub fn join_path(base: &str, component: &str) -> String {
    if base.ends_with('\\') || base.ends_with('/') {
        alloc::format!("{}{}", base, component)
    } else {
        alloc::format!("{}\\{}", base, component)
    }
}
