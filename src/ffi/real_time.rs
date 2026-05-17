use core::ffi::{c_char, c_void};

unsafe extern "C" {
    pub fn gk_matchmaker_find_match_json(
        request_json: *const c_char,
        callback: Option<unsafe extern "C" fn(*mut c_void, *const c_char, i32)>,
        refcon: *mut c_void,
        out_match_ptr: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_matchmaker_find_hosted_players_json(
        request_json: *const c_char,
        callback: Option<unsafe extern "C" fn(*mut c_void, *const c_char, i32)>,
        refcon: *mut c_void,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_matchmaker_find_matched_players_json(
        request_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_matchmaker_add_players_to_match(
        ptr: *mut c_void,
        request_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_matchmaker_cancel();
    pub fn gk_matchmaker_finish(ptr: *mut c_void);

    pub fn gk_matchmaker_query_player_group_activity(
        player_group: usize,
        out_activity: *mut i64,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn gk_matchmaker_query_activity(out_activity: *mut i64, out_error: *mut *mut c_char)
        -> i32;

    pub fn gk_matchmaker_max_players_allowed(match_type: i32) -> usize;
}
