use core::ffi::c_char;

unsafe extern "C" {
    pub fn gk_string_free(s: *mut c_char);
}

pub mod status {
    pub const OK: i32 = 0;
    pub const TIMED_OUT: i32 = -2;
    pub const NOT_AUTHENTICATED: i32 = -3;
    pub const FRAMEWORK_ERROR: i32 = -4;
    pub const NOT_FOUND: i32 = -5;
    pub const UNAVAILABLE: i32 = -6;
    pub const UNKNOWN: i32 = -99;
}
