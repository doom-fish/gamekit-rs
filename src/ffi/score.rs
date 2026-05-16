use core::ffi::c_char;

unsafe extern "C" {
    pub fn gk_score_report_json(
        scores_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
}
