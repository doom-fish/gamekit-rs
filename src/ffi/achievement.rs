use core::ffi::c_char;

unsafe extern "C" {
    pub fn gk_achievement_load_json(
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_achievement_descriptions_json(
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_achievement_report_json(
        achievements_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_achievement_reset(out_error: *mut *mut c_char) -> i32;
}
