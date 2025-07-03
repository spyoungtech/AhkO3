use std::ffi::{OsString, c_longlong, c_char};


use std::fmt;
use std::error::Error;

/// Represents an error that occurred during AHK interop operations
#[derive(Debug)]
pub struct AHKError {
    message: String,
    // Optional: add more fields like error type, source location, etc.
}

impl AHKError {
    pub fn new<S: Into<String>>(message: S) -> Self {
        AHKError {
            message: message.into(),
        }
    }
}

impl fmt::Display for AHKError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AHK Error: {}", self.message)
    }
}

impl Error for AHKError {}

/// A specialized Result type for AHK operations.
/// The Err variant contains an AHKError.
pub type AHKResult<T> = Result<T, AHKError>;



// Implementation of common error cases
impl AHKError {
    pub fn type_error<S: Into<String>>(message: S) -> AHKError {
        AHKError::new(format!("TypeError: {}", message.into()))
    }

    pub fn value_error<S: Into<String>>(message: S) -> AHKError {
        AHKError::new(format!("ValueError: {}", message.into()))
    }

    pub fn runtime_error<S: Into<String>>(message: S) -> AHKError {
        AHKError::new(format!("RuntimeError: {}", message.into()))
    }
}



type AHKWstr = *const u16;



pub fn ahk_str_to_string(ahk_str: AHKWstr) -> Result<String, i64> {
    if ahk_str.is_null() {
        return Err(-1);
    }
    let mut length = 0usize;
    unsafe {
        while *ahk_str.add(length) != 0 {
            length += 1;
        }
    }

    let slice = unsafe { std::slice::from_raw_parts(ahk_str, length) };
    Ok(String::from_utf16_lossy(slice))
}


pub fn string_to_utf16_null(s: String) -> Box<[u16]> {
    s.encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>()
        .into_boxed_slice()
}

// Function to convert a Rust String to a pointer that AHK can understand
pub fn string_to_ahk_ptr(s: String) -> *const u16 {
    let boxed = string_to_utf16_null(s);
    Box::leak(boxed).as_ptr()
}


#[unsafe(no_mangle)]
pub extern "C" fn ahko3_free_string_ptr(ptr: *mut u16) -> std::ffi::c_longlong {
    if !ptr.is_null() {
        unsafe {
            // Find the length of the string
            let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
            // Reconstruct the Box and drop it
            drop(Box::from_raw(std::slice::from_raw_parts_mut(ptr, len + 1)));
        }
    }
    0
}

pub use ahko3_macros::{ahkfunction};
