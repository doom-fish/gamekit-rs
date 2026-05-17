use core::ffi::c_char;

unsafe extern "C" {
    pub fn gk_leaderboard_sets_json(out_json: *mut *mut c_char, out_error: *mut *mut c_char)
        -> i32;

    pub fn gk_leaderboard_set_load_leaderboards_json(
        identifier: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_leaderboard_set_load_image_json(
        identifier: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
}
