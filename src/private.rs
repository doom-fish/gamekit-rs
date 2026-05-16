#![allow(clippy::missing_errors_doc)]

use core::ffi::c_char;
use std::ffi::{CStr, CString};

use base64::Engine;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::GameKitError;
use crate::ffi;

pub fn cstring_from_str(value: &str, context: &str) -> Result<CString, GameKitError> {
    CString::new(value).map_err(|error| {
        GameKitError::Unknown(format!("{context} contains an embedded NUL byte: {error}"))
    })
}

pub fn json_cstring<T: Serialize + ?Sized>(
    value: &T,
    context: &str,
) -> Result<CString, GameKitError> {
    let json = serde_json::to_string(value).map_err(|error| {
        GameKitError::Unknown(format!("failed to encode {context} as JSON: {error}"))
    })?;
    cstring_from_str(&json, context)
}

pub fn decode_base64(value: &str, context: &str) -> Result<Vec<u8>, GameKitError> {
    base64::engine::general_purpose::STANDARD
        .decode(value)
        .map_err(|error| GameKitError::Unknown(format!("failed to decode {context} bytes: {error}")))
}

pub unsafe fn take_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let string = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    ffi::gk_string_free(ptr);
    Some(string)
}

pub unsafe fn parse_json_ptr<T: DeserializeOwned>(
    ptr: *mut c_char,
    context: &str,
) -> Result<T, GameKitError> {
    let json = take_string(ptr)
        .ok_or_else(|| GameKitError::Unknown(format!("missing JSON payload for {context}")))?;
    serde_json::from_str(&json).map_err(|error| {
        GameKitError::Unknown(format!(
            "failed to parse {context} JSON: {error}; payload={json}"
        ))
    })
}

pub unsafe fn error_from_status(status: i32, err_msg: *mut c_char) -> GameKitError {
    crate::error::from_swift(status, err_msg)
}
