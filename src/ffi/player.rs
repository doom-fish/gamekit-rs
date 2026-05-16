use core::ffi::c_char;

unsafe extern "C" {
    pub fn gk_player_anonymous_guest_json(
        identifier: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
}
