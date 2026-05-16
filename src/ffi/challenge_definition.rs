use core::ffi::c_char;

unsafe extern "C" {
    pub fn gk_challenge_definitions_json(
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_challenge_definition_has_active(
        identifier: *const c_char,
        out_active: *mut bool,
        out_error: *mut *mut c_char,
    ) -> i32;
}
