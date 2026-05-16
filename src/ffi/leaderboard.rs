use core::ffi::c_char;

unsafe extern "C" {
    pub fn gk_leaderboard_load_json(
        ids_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_leaderboard_load_previous_occurrence_json(
        leaderboard_id: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_leaderboard_submit_score(
        score: i64,
        context: u64,
        ids_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_leaderboard_submit_score_for_id(
        score: i64,
        context: u64,
        leaderboard_id: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
}
